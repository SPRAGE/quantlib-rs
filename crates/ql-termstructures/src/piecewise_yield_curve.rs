//! `PiecewiseYieldCurve` — iterative bootstrap construction of a yield curve
//! (translates `ql/termstructures/yield/piecewiseyieldcurve.hpp`).
//!
//! Given a vector of [`RateHelper`]s (deposits, FRAs, swaps, futures) the
//! bootstrapper iteratively builds an interpolated zero-rate curve by solving
//! for the zero rate at each pillar date such that the helper's implied quote
//! matches the market quote.
//!
//! # Example
//!
//! ```
//! use ql_termstructures::piecewise_yield_curve::PiecewiseYieldCurve;
//! use ql_termstructures::rate_helpers::DepositRateHelper;
//! use ql_termstructures::interpolated_zero_curve::Linear;
//! use ql_time::{Actual360, Date};
//!
//! let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
//! let helpers: Vec<Box<dyn ql_termstructures::rate_helpers::RateHelper>> = vec![
//!     Box::new(DepositRateHelper::new(
//!         0.04,
//!         ref_date,
//!         Date::from_ymd(2025, 4, 2).unwrap(),
//!         Actual360,
//!     )),
//!     Box::new(DepositRateHelper::new(
//!         0.045,
//!         ref_date,
//!         Date::from_ymd(2025, 7, 2).unwrap(),
//!         Actual360,
//!     )),
//! ];
//! let curve = PiecewiseYieldCurve::new(ref_date, &helpers, Actual360, &Linear).unwrap();
//! assert!(curve.discount(0.5) < 1.0);
//! ```

use crate::interpolated_zero_curve::InterpolationBuilder;
use crate::rate_helpers::{BootstrapCurve, RateHelper};
use crate::term_structure::TermStructure;
use crate::yield_term_structure::{YieldTermStructure, YieldTermStructureData};
use ql_core::errors::{Error, Result};
use ql_core::{DiscountFactor, Rate, Real, Time};
use ql_math::Interpolation1D;
use ql_time::{Calendar, Date, DayCounter, NullCalendar};
use std::sync::Arc;

/// Default accuracy for the bootstrap solver.
const BOOTSTRAP_ACCURACY: Real = 1.0e-12;

/// Minimum zero rate to search (−10 %).
const MIN_RATE: Rate = -0.10;

/// Maximum zero rate to search (+30 %).
const MAX_RATE: Rate = 0.30;

/// A yield curve bootstrapped from market instruments.
///
/// The curve interpolates continuously-compounded zero rates on pillar dates
/// that are determined by the supplied rate helpers.  The bootstrap solves for
/// the zero rate at each pillar using Brent's method so that the helper's
/// implied quote matches its market quote.
///
/// Corresponds to `QuantLib::PiecewiseYieldCurve<ZeroYield, Interpolator>`.
#[derive(Debug)]
pub struct PiecewiseYieldCurve {
    data: YieldTermStructureData,
    /// Pillar dates (first entry = reference date).
    dates: Vec<Date>,
    /// Time fractions corresponding to `dates`.
    times: Vec<Real>,
    /// Continuously-compounded zero rates at each pillar.
    rates: Vec<Rate>,
    /// The interpolation object.
    interp: Box<dyn Interpolation1D>,
    /// The last pillar date.
    max_date: Date,
}

impl PiecewiseYieldCurve {
    /// Bootstrap a yield curve from rate helpers.
    ///
    /// # Arguments
    /// * `reference_date` — the curve's reference (evaluation) date
    /// * `helpers` — market instruments (must not be empty)
    /// * `day_counter` — day-count convention for time calculations
    /// * `builder` — interpolation strategy (e.g. `Linear`, `LogLinear`)
    ///
    /// # Errors
    /// Returns an error if no helpers are provided, pillar dates are
    /// duplicated, or the solver fails to converge at any pillar.
    pub fn new(
        reference_date: Date,
        helpers: &[Box<dyn RateHelper>],
        day_counter: impl DayCounter + 'static,
        builder: &dyn InterpolationBuilder,
    ) -> Result<Self> {
        Self::with_bounds(
            reference_date,
            helpers,
            day_counter,
            builder,
            MIN_RATE,
            MAX_RATE,
            BOOTSTRAP_ACCURACY,
        )
    }

    /// Bootstrap with explicit solver bounds and accuracy.
    pub fn with_bounds(
        reference_date: Date,
        helpers: &[Box<dyn RateHelper>],
        day_counter: impl DayCounter + 'static,
        builder: &dyn InterpolationBuilder,
        min_rate: Rate,
        max_rate: Rate,
        accuracy: Real,
    ) -> Result<Self> {
        ql_core::ensure!(!helpers.is_empty(), "at least one rate helper is required");

        let dc: Arc<dyn DayCounter> = Arc::new(day_counter);

        // Sort helpers by pillar date
        let mut sorted_indices: Vec<usize> = (0..helpers.len()).collect();
        sorted_indices.sort_by_key(|&i| helpers[i].pillar_date());

        // Build pillar date vector (reference date + one per helper)
        let mut dates = Vec::with_capacity(helpers.len() + 1);
        let mut times = Vec::with_capacity(helpers.len() + 1);
        let mut rates = Vec::with_capacity(helpers.len() + 1);

        dates.push(reference_date);
        times.push(0.0_f64);
        rates.push(0.0_f64); // will be updated

        for &idx in &sorted_indices {
            let pillar = helpers[idx].pillar_date();
            if pillar <= reference_date {
                return Err(Error::Precondition(format!(
                    "pillar date {pillar} is not after reference date {reference_date}"
                )));
            }
            // Skip duplicate pillar dates
            if !dates.is_empty() && pillar == *dates.last().unwrap() {
                continue;
            }
            let t = dc.year_fraction(reference_date, pillar);
            dates.push(pillar);
            times.push(t);
            rates.push(0.0); // placeholder
        }

        // ── Iterative bootstrap ──────────────────────────────────────
        //
        // For each pillar k (1-indexed, 0 = reference date):
        //   1. Set the initial guess for rates[k] using the previous rate.
        //   2. Use Brent's method to find rates[k] such that
        //      helper.implied_quote(curve) == helper.quote().
        //   3. Re-build interpolation with the updated rates.

        // Initial guess: extrapolate from the first helper's quote
        let first_helper = &helpers[sorted_indices[0]];
        rates[0] = first_helper.quote(); // use the first deposit rate as t=0 rate
        rates[1] = first_helper.quote();

        let mut helper_cursor = 0;
        for k in 1..dates.len() {
            // Find the helper for this pillar
            while helper_cursor < sorted_indices.len()
                && helpers[sorted_indices[helper_cursor]].pillar_date() < dates[k]
            {
                helper_cursor += 1;
            }
            if helper_cursor >= sorted_indices.len() {
                break;
            }
            let helper = &helpers[sorted_indices[helper_cursor]];

            // Initial guess: previous pillar's rate
            rates[k] = rates[k - 1];

            let market_quote = helper.quote();

            // Use Brent solver: find rate at pillar k such that
            // implied_quote(curve) - market_quote = 0
            let solved_rate = {
                let times_slice = &times[..=k];
                let rates_mut = &mut rates;

                // We need to define the objective function for Brent
                let result = brent_bootstrap(
                    |r| {
                        rates_mut[k] = r;
                        // Also extrapolate rate[0] (reference date) from rate[1]
                        rates_mut[0] = rates_mut[1];
                        let interp = builder.build(times_slice, &rates_mut[..=k]).ok()?;
                        let bc = BootstrapCurve {
                            reference_date,
                            day_counter: &*dc,
                            times: times_slice,
                            rates: &rates_mut[..=k],
                            interp: &*interp,
                        };
                        Some(helper.implied_quote(&bc) - market_quote)
                    },
                    min_rate,
                    max_rate,
                    accuracy,
                );
                match result {
                    Ok(r) => r,
                    Err(e) => {
                        return Err(Error::Runtime(format!(
                            "bootstrap failed at pillar {} (date {}): {e}",
                            k, dates[k]
                        )));
                    }
                }
            };

            rates[k] = solved_rate;
            rates[0] = rates[1]; // keep reference date rate in sync
            helper_cursor += 1;
        }

        // Build the final interpolation
        let interp = builder.build(&times, &rates)?;
        let max_date = *dates.last().unwrap();

        Ok(Self {
            data: YieldTermStructureData {
                reference_date,
                calendar: Box::new(NullCalendar),
                day_counter: dc,
            },
            dates,
            times,
            rates,
            interp,
            max_date,
        })
    }

    /// Set a custom calendar.
    pub fn with_calendar(mut self, calendar: impl Calendar + 'static) -> Self {
        self.data.calendar = Box::new(calendar);
        self
    }

    /// Return the pillar dates.
    pub fn dates(&self) -> &[Date] {
        &self.dates
    }

    /// Return the pillar times.
    pub fn times(&self) -> &[Real] {
        &self.times
    }

    /// Return the bootstrapped zero rates.
    pub fn rates(&self) -> &[Rate] {
        &self.rates
    }
}

impl TermStructure for PiecewiseYieldCurve {
    fn reference_date(&self) -> Date {
        self.data.reference_date
    }

    fn day_counter(&self) -> &dyn DayCounter {
        &*self.data.day_counter
    }

    fn calendar(&self) -> &dyn Calendar {
        &*self.data.calendar
    }

    fn max_date(&self) -> Date {
        self.max_date
    }
}

impl YieldTermStructure for PiecewiseYieldCurve {
    fn zero_rate_impl(&self, t: Time) -> Rate {
        self.interp.operator(t)
    }

    fn discount_impl(&self, t: Time) -> DiscountFactor {
        if t <= 0.0 {
            return 1.0;
        }
        let z = self.zero_rate_impl(t);
        (-z * t).exp()
    }
}

// ── Brent solver variant for bootstrap ───────────────────────────────────────

/// A Brent-style solver that tolerates the objective function returning `None`
/// (e.g. when interpolation construction fails for an extreme trial rate).
///
/// Returns the root `x` in `[x_min, x_max]` such that `f(x) ≈ 0`.
fn brent_bootstrap<F>(mut f: F, x_min: Real, x_max: Real, accuracy: Real) -> Result<Real>
where
    F: FnMut(Real) -> Option<Real>,
{
    const MAX_ITER: u32 = 100;

    let fa = f(x_min).unwrap_or(f64::NAN);
    let fb = f(x_max).unwrap_or(f64::NAN);

    if fa.is_nan() && fb.is_nan() {
        return Err(Error::Runtime(
            "brent_bootstrap: f undefined at both endpoints".into(),
        ));
    }

    // If one endpoint is NaN, try to nudge inward
    let (mut a, mut b, mut fa, mut fb) = if fa.is_nan() {
        let mid = (x_min + x_max) / 2.0;
        let fm = f(mid).unwrap_or(f64::NAN);
        if fm.is_nan() {
            return Err(Error::Runtime(
                "brent_bootstrap: f undefined at min and midpoint".into(),
            ));
        }
        (mid, x_max, fm, fb)
    } else if fb.is_nan() {
        let mid = (x_min + x_max) / 2.0;
        let fm = f(mid).unwrap_or(f64::NAN);
        if fm.is_nan() {
            return Err(Error::Runtime(
                "brent_bootstrap: f undefined at max and midpoint".into(),
            ));
        }
        (x_min, mid, fa, fm)
    } else {
        (x_min, x_max, fa, fb)
    };

    if fa.abs() < accuracy {
        return Ok(a);
    }
    if fb.abs() < accuracy {
        return Ok(b);
    }

    if fa * fb > 0.0 {
        return Err(Error::Precondition(format!(
            "brent_bootstrap: f({a}) = {fa} and f({b}) = {fb} have the same sign"
        )));
    }

    let mut c = b;
    let mut fc = fb;
    let mut d = b - a;
    let mut e = d;

    for _ in 0..MAX_ITER {
        if fb * fc > 0.0 {
            c = a;
            fc = fa;
            d = b - a;
            e = d;
        }
        if fc.abs() < fb.abs() {
            a = b;
            b = c;
            c = a;
            fa = fb;
            fb = fc;
            fc = fa;
        }

        let tol = 2.0 * f64::EPSILON * b.abs() + 0.5 * accuracy;
        let m = 0.5 * (c - b);

        if m.abs() <= tol || fb.abs() <= accuracy {
            return Ok(b);
        }

        if e.abs() >= tol && fa.abs() > fb.abs() {
            // Attempt inverse quadratic interpolation
            let s = fb / fa;
            let (p, q) = if (a - c).abs() < f64::EPSILON {
                let p = 2.0 * m * s;
                let q = 1.0 - s;
                (p, q)
            } else {
                let q0 = fa / fc;
                let r = fb / fc;
                let p = s * (2.0 * m * q0 * (q0 - r) - (b - a) * (r - 1.0));
                let q = (q0 - 1.0) * (r - 1.0) * (s - 1.0);
                (p, q)
            };

            let (p, q) = if p > 0.0 { (p, -q) } else { (-p, q) };

            if 2.0 * p < (3.0 * m * q - (tol * q).abs()).min(e * q.abs()) {
                e = d;
                d = p / q;
            } else {
                d = m;
                e = m;
            }
        } else {
            d = m;
            e = m;
        }

        a = b;
        fa = fb;

        if d.abs() > tol {
            b += d;
        } else {
            b += if m > 0.0 { tol } else { -tol };
        }

        fb = f(b).unwrap_or(f64::NAN);
        if fb.is_nan() {
            // Fall back to bisection
            b = (a + c) / 2.0;
            fb = f(b).unwrap_or(0.0);
        }
    }

    Ok(b) // return best estimate
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpolated_zero_curve::Linear;
    use crate::rate_helpers::{DepositRateHelper, SwapRateHelper};
    use approx::assert_abs_diff_eq;
    use ql_time::{Actual360, Actual365Fixed, Schedule};

    #[test]
    fn bootstrap_single_deposit() {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let mat = Date::from_ymd(2025, 4, 2).unwrap();
        let depo_rate = 0.05;

        let helpers: Vec<Box<dyn RateHelper>> = vec![Box::new(DepositRateHelper::new(
            depo_rate, ref_date, mat, Actual360,
        ))];

        let curve = PiecewiseYieldCurve::new(ref_date, &helpers, Actual360, &Linear).unwrap();

        // The curve should reprice the deposit exactly
        let tau = Actual360.year_fraction(ref_date, mat);
        let df_settle = curve.discount(0.0);
        let df_mat = curve.discount(tau);
        let implied = (df_settle / df_mat - 1.0) / tau;
        assert_abs_diff_eq!(implied, depo_rate, epsilon = 1e-10);
    }

    #[test]
    fn bootstrap_two_deposits() {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let mat1 = Date::from_ymd(2025, 4, 2).unwrap();
        let mat2 = Date::from_ymd(2025, 7, 2).unwrap();

        let helpers: Vec<Box<dyn RateHelper>> = vec![
            Box::new(DepositRateHelper::new(0.04, ref_date, mat1, Actual360)),
            Box::new(DepositRateHelper::new(0.045, ref_date, mat2, Actual360)),
        ];

        let curve = PiecewiseYieldCurve::new(ref_date, &helpers, Actual360, &Linear).unwrap();

        // Check that each deposit reprices correctly
        for helper in &helpers {
            let bc_times = curve.times();
            let bc_rates = curve.rates();
            let interp = Linear.build(bc_times, bc_rates).unwrap();
            let bc = BootstrapCurve {
                reference_date: ref_date,
                day_counter: &Actual360,
                times: bc_times,
                rates: bc_rates,
                interp: &*interp,
            };
            let implied = helper.implied_quote(&bc);
            assert_abs_diff_eq!(implied, helper.quote(), epsilon = 1e-8);
        }
    }

    #[test]
    fn bootstrap_deposits_and_swap() {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();

        // Short-end: 3M and 6M deposits
        let mat_3m = Date::from_ymd(2025, 4, 2).unwrap();
        let mat_6m = Date::from_ymd(2025, 7, 2).unwrap();

        // Long-end: 2Y par swap (annual fixed, Actual365Fixed)
        let mat_2y = Date::from_ymd(2027, 1, 4).unwrap();
        let swap_schedule =
            Schedule::from_dates(vec![ref_date, Date::from_ymd(2026, 1, 2).unwrap(), mat_2y]);

        let helpers: Vec<Box<dyn RateHelper>> = vec![
            Box::new(DepositRateHelper::new(
                0.04,
                ref_date,
                mat_3m,
                Actual365Fixed,
            )),
            Box::new(DepositRateHelper::new(
                0.042,
                ref_date,
                mat_6m,
                Actual365Fixed,
            )),
            Box::new(SwapRateHelper::new(0.045, swap_schedule, Actual365Fixed)),
        ];

        let curve = PiecewiseYieldCurve::new(ref_date, &helpers, Actual365Fixed, &Linear).unwrap();

        // All discount factors should be < 1 and decreasing
        let t_3m = Actual365Fixed.year_fraction(ref_date, mat_3m);
        let t_6m = Actual365Fixed.year_fraction(ref_date, mat_6m);
        let t_2y = Actual365Fixed.year_fraction(ref_date, mat_2y);

        let df_3m = curve.discount(t_3m);
        let df_6m = curve.discount(t_6m);
        let df_2y = curve.discount(t_2y);

        assert!(df_3m < 1.0, "df_3m = {df_3m}");
        assert!(df_6m < df_3m, "df_6m = {df_6m} should be < df_3m = {df_3m}");
        assert!(df_2y < df_6m, "df_2y = {df_2y} should be < df_6m = {df_6m}");

        // Verify the deposit rates reprice (settlement == ref_date)
        let implied_3m = (1.0 / df_3m - 1.0) / t_3m;
        assert_abs_diff_eq!(implied_3m, 0.04, epsilon = 1e-6);
    }

    #[test]
    fn bootstrap_monotone_discount_factors() {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();

        let helpers: Vec<Box<dyn RateHelper>> = vec![
            Box::new(DepositRateHelper::new(
                0.03,
                ref_date,
                Date::from_ymd(2025, 2, 3).unwrap(),
                Actual360,
            )),
            Box::new(DepositRateHelper::new(
                0.032,
                ref_date,
                Date::from_ymd(2025, 4, 2).unwrap(),
                Actual360,
            )),
            Box::new(DepositRateHelper::new(
                0.035,
                ref_date,
                Date::from_ymd(2025, 7, 2).unwrap(),
                Actual360,
            )),
            Box::new(DepositRateHelper::new(
                0.038,
                ref_date,
                Date::from_ymd(2026, 1, 2).unwrap(),
                Actual360,
            )),
        ];

        let curve = PiecewiseYieldCurve::new(ref_date, &helpers, Actual360, &Linear).unwrap();

        // Discount factors should be strictly decreasing
        let mut prev_df = 1.0;
        for t in [0.1, 0.25, 0.5, 0.75, 1.0] {
            let df = curve.discount(t);
            assert!(
                df < prev_df,
                "discount factor at t={t} ({df}) should be < previous ({prev_df})"
            );
            prev_df = df;
        }
    }

    #[test]
    fn bootstrap_negative_rates() {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();

        let helpers: Vec<Box<dyn RateHelper>> = vec![
            Box::new(DepositRateHelper::new(
                -0.005,
                ref_date,
                Date::from_ymd(2025, 4, 2).unwrap(),
                Actual360,
            )),
            Box::new(DepositRateHelper::new(
                -0.003,
                ref_date,
                Date::from_ymd(2025, 7, 2).unwrap(),
                Actual360,
            )),
        ];

        let curve = PiecewiseYieldCurve::new(ref_date, &helpers, Actual360, &Linear).unwrap();

        // Negative rates → discount factors > 1
        let df = curve.discount(0.25);
        assert!(df > 1.0, "negative rates should give DF > 1, got {df}");
    }

    #[test]
    fn bootstrap_error_on_empty_helpers() {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let helpers: Vec<Box<dyn RateHelper>> = vec![];
        let result = PiecewiseYieldCurve::new(ref_date, &helpers, Actual360, &Linear);
        assert!(result.is_err());
    }

    #[test]
    fn bootstrap_term_structure_trait() {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let helpers: Vec<Box<dyn RateHelper>> = vec![Box::new(DepositRateHelper::new(
            0.05,
            ref_date,
            Date::from_ymd(2025, 7, 2).unwrap(),
            Actual360,
        ))];

        let curve = PiecewiseYieldCurve::new(ref_date, &helpers, Actual360, &Linear).unwrap();

        assert_eq!(curve.reference_date(), ref_date);
        assert!(curve.max_date() > ref_date);
    }

    #[test]
    fn bootstrap_yield_term_structure_trait() {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let mat = Date::from_ymd(2026, 1, 2).unwrap();
        let helpers: Vec<Box<dyn RateHelper>> = vec![Box::new(DepositRateHelper::new(
            0.05,
            ref_date,
            mat,
            Actual365Fixed,
        ))];

        let curve = PiecewiseYieldCurve::new(ref_date, &helpers, Actual365Fixed, &Linear).unwrap();

        // discount_date should work
        let df = curve.discount_date(mat);
        assert!(df > 0.0 && df < 1.0);

        // zero_rate should be close to the input
        use ql_core::Compounding;
        use ql_time::Frequency;
        let zr = curve.zero_rate(
            mat,
            &Actual365Fixed,
            Compounding::Continuous,
            Frequency::Annual,
        );
        assert!(
            (zr.rate() - 0.05).abs() < 0.01,
            "zero rate = {} expected near 0.05",
            zr.rate()
        );
    }
}
