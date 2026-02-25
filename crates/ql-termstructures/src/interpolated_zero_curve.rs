//! `InterpolatedZeroCurve` — a yield term structure bootstrapped from zero rates
//! (translates `ql/termstructures/yield/zeroyieldstructure.hpp` +
//! `ql/termstructures/yield/interpolatedzerocurve.hpp`).
//!
//! The curve stores (date, zero-rate) pairs and interpolates zero rates as a
//! function of time.  Discount factors are computed as `P(t) = exp(-z(t) * t)`.

use crate::term_structure::TermStructure;
use crate::yield_term_structure::{YieldTermStructure, YieldTermStructureData};
use ql_core::{errors::Result, Rate, Real, Time};
use ql_math::Interpolation1D;
use ql_time::{Calendar, Date, DayCounter, NullCalendar};
use std::sync::Arc;

/// A yield curve defined by zero rates at known dates.
///
/// Interpolation of the zero rate is delegated to a pluggable `Interpolation1D`
/// implementation (linear, log-linear, cubic, etc.).
///
/// Corresponds to `QuantLib::InterpolatedZeroCurve<Interpolator>`.
#[derive(Debug)]
pub struct InterpolatedZeroCurve {
    data: YieldTermStructureData,
    /// Dates of the zero-rate pillars (excluding reference date).
    dates: Vec<Date>,
    /// Time fractions corresponding to `dates`.
    times: Vec<Real>,
    /// Zero rates at `dates`.
    rates: Vec<Rate>,
    /// The interpolation object.
    interp: Box<dyn Interpolation1D>,
    /// The earliest pillar date (used for max_date).
    max_date: Date,
}

/// Trait for creating an interpolation from `(xs, ys)` slices.
///
/// This lets callers choose the interpolation method (linear, log-linear, etc.)
/// without the curve needing to know the concrete type.
pub trait InterpolationBuilder: std::fmt::Debug {
    /// Build an interpolation from the given x and y values.
    fn build(&self, xs: &[Real], ys: &[Real]) -> Result<Box<dyn Interpolation1D>>;
}

/// Linear interpolation builder.
#[derive(Debug, Clone, Copy)]
pub struct Linear;

impl InterpolationBuilder for Linear {
    fn build(&self, xs: &[Real], ys: &[Real]) -> Result<Box<dyn Interpolation1D>> {
        Ok(Box::new(ql_math::LinearInterpolation::new(xs, ys)?))
    }
}

/// Log-linear interpolation builder.
#[derive(Debug, Clone, Copy)]
pub struct LogLinear;

impl InterpolationBuilder for LogLinear {
    fn build(&self, xs: &[Real], ys: &[Real]) -> Result<Box<dyn Interpolation1D>> {
        Ok(Box::new(ql_math::LogLinearInterpolation::new(xs, ys)?))
    }
}

/// Cubic natural spline interpolation builder.
#[derive(Debug, Clone, Copy)]
pub struct CubicNatural;

impl InterpolationBuilder for CubicNatural {
    fn build(&self, xs: &[Real], ys: &[Real]) -> Result<Box<dyn Interpolation1D>> {
        Ok(Box::new(ql_math::CubicNaturalSpline::new(xs, ys)?))
    }
}

impl InterpolatedZeroCurve {
    /// Build a zero-rate curve from dates and corresponding zero rates.
    ///
    /// The first date should be the reference date (its zero rate is typically
    /// set to the rate at the first pillar, or zero).  Dates must be sorted in
    /// ascending order.
    ///
    /// # Arguments
    /// * `dates` — pillar dates (first entry = reference date)
    /// * `rates` — continuously-compounded zero rates at each date
    /// * `day_counter` — used for date → time conversion
    /// * `builder` — interpolation strategy (e.g. `Linear`, `LogLinear`)
    pub fn new(
        dates: &[Date],
        rates: &[Rate],
        day_counter: impl DayCounter + 'static,
        builder: &dyn InterpolationBuilder,
    ) -> Result<Self> {
        ql_core::ensure!(
            dates.len() >= 2,
            "need at least 2 dates (reference + 1 pillar)"
        );
        ql_core::ensure!(
            dates.len() == rates.len(),
            "dates and rates must have the same length"
        );

        let reference_date = dates[0];
        let dc: Arc<dyn DayCounter> = Arc::new(day_counter);

        let times: Vec<Real> = dates
            .iter()
            .map(|&d| dc.year_fraction(reference_date, d))
            .collect();

        let interp = builder.build(&times, rates)?;
        let max_date = *dates.last().unwrap();

        Ok(Self {
            data: YieldTermStructureData {
                reference_date,
                calendar: Box::new(NullCalendar),
                day_counter: dc,
            },
            dates: dates.to_vec(),
            times,
            rates: rates.to_vec(),
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

    /// Return the pillar zero rates.
    pub fn rates(&self) -> &[Rate] {
        &self.rates
    }
}

impl TermStructure for InterpolatedZeroCurve {
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

impl YieldTermStructure for InterpolatedZeroCurve {
    fn zero_rate_impl(&self, t: Time) -> Rate {
        self.interp.operator(t)
    }

    fn discount_impl(&self, t: Time) -> f64 {
        if t == 0.0 {
            return 1.0;
        }
        let z = self.zero_rate_impl(t);
        (-z * t).exp()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use ql_time::Actual365Fixed;

    fn sample_dates_rates() -> (Vec<Date>, Vec<Rate>) {
        let dates = vec![
            Date::from_ymd(2025, 1, 2).unwrap(),
            Date::from_ymd(2025, 7, 2).unwrap(),
            Date::from_ymd(2026, 1, 2).unwrap(),
            Date::from_ymd(2027, 1, 2).unwrap(),
            Date::from_ymd(2030, 1, 2).unwrap(),
        ];
        let rates = vec![0.02, 0.025, 0.03, 0.035, 0.04];
        (dates, rates)
    }

    #[test]
    fn zero_curve_linear_discount_at_ref_date() {
        let (dates, rates) = sample_dates_rates();
        let curve = InterpolatedZeroCurve::new(&dates, &rates, Actual365Fixed, &Linear).unwrap();

        assert_abs_diff_eq!(curve.discount(0.0), 1.0, epsilon = 1e-15);
    }

    #[test]
    fn zero_curve_linear_pillars() {
        let (dates, rates) = sample_dates_rates();
        let curve = InterpolatedZeroCurve::new(&dates, &rates, Actual365Fixed, &Linear).unwrap();

        // At each pillar, the zero rate should match
        for (i, &d) in dates.iter().enumerate() {
            let t = curve.time_from_reference(d);
            assert_abs_diff_eq!(curve.zero_rate_impl(t), rates[i], epsilon = 1e-10);
        }
    }

    #[test]
    fn zero_curve_linear_interpolation() {
        let (dates, rates) = sample_dates_rates();
        let curve = InterpolatedZeroCurve::new(&dates, &rates, Actual365Fixed, &Linear).unwrap();

        // At half a year (first pillar time)
        let t = curve.time_from_reference(dates[1]);
        let z = curve.zero_rate_impl(t);
        // Should return the rate at that pillar
        assert_abs_diff_eq!(z, 0.025, epsilon = 1e-10);

        // Between pillars: 0.75 years (between 0.5 and 1.0 year pillars)
        let t_mid = 0.75;
        let z_mid = curve.zero_rate_impl(t_mid);
        // Linear interpolation between 0.025 and 0.03
        assert!(z_mid > 0.025 && z_mid < 0.03);
    }

    #[test]
    fn zero_curve_discount_consistency() {
        let (dates, rates) = sample_dates_rates();
        let curve = InterpolatedZeroCurve::new(&dates, &rates, Actual365Fixed, &Linear).unwrap();

        // Check P(t) = exp(-z(t) * t) at a few points
        for t in [0.25, 0.5, 1.0, 2.5, 5.0] {
            let z = curve.zero_rate_impl(t);
            let expected_df = (-z * t).exp();
            assert_abs_diff_eq!(curve.discount(t), expected_df, epsilon = 1e-12);
        }
    }
}
