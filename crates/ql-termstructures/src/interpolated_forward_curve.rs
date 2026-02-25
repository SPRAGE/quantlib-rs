//! `InterpolatedForwardCurve` — a yield term structure bootstrapped from
//! instantaneous forward rates (translates
//! `ql/termstructures/yield/interpolatedforwardcurve.hpp`).
//!
//! The curve stores (date, forward-rate) pairs and interpolates them as a
//! function of time.  Discount factors are computed by integrating the forward
//! rate: `P(t) = exp(−∫₀ᵗ f(s) ds)`.

use crate::interpolated_zero_curve::InterpolationBuilder;
use crate::term_structure::TermStructure;
use crate::yield_term_structure::{YieldTermStructure, YieldTermStructureData};
use ql_core::{errors::Result, Rate, Real, Time};
use ql_math::Interpolation1D;
use ql_time::{Calendar, Date, DayCounter, NullCalendar};
use std::sync::Arc;

/// A yield curve defined by instantaneous forward rates at known dates.
///
/// Discount factors are obtained by numerical integration of the interpolated
/// forward-rate curve using the trapezoidal rule.
///
/// Corresponds to `QuantLib::InterpolatedForwardCurve<Interpolator>`.
#[derive(Debug)]
pub struct InterpolatedForwardCurve {
    data: YieldTermStructureData,
    dates: Vec<Date>,
    times: Vec<Real>,
    forwards: Vec<Rate>,
    interp: Box<dyn Interpolation1D>,
    max_date: Date,
}

impl InterpolatedForwardCurve {
    /// Build a forward-rate curve from dates and corresponding instantaneous
    /// forward rates.
    ///
    /// The first date must be the reference date.
    /// Dates must be sorted in ascending order.
    ///
    /// # Arguments
    /// * `dates` — pillar dates (first entry = reference date)
    /// * `forwards` — instantaneous forward rates at each date
    /// * `day_counter` — used for date → time conversion
    /// * `builder` — interpolation strategy
    pub fn new(
        dates: &[Date],
        forwards: &[Rate],
        day_counter: impl DayCounter + 'static,
        builder: &dyn InterpolationBuilder,
    ) -> Result<Self> {
        ql_core::ensure!(
            dates.len() >= 2,
            "need at least 2 dates (reference + 1 pillar)"
        );
        ql_core::ensure!(
            dates.len() == forwards.len(),
            "dates and forwards must have the same length"
        );

        let reference_date = dates[0];
        let dc: Arc<dyn DayCounter> = Arc::new(day_counter);

        let times: Vec<Real> = dates
            .iter()
            .map(|&d| dc.year_fraction(reference_date, d))
            .collect();

        let interp = builder.build(&times, forwards)?;
        let max_date = *dates.last().unwrap();

        Ok(Self {
            data: YieldTermStructureData {
                reference_date,
                calendar: Box::new(NullCalendar),
                day_counter: dc,
            },
            dates: dates.to_vec(),
            times,
            forwards: forwards.to_vec(),
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

    /// Return the pillar forward rates.
    pub fn forwards(&self) -> &[Rate] {
        &self.forwards
    }

    /// Compute ∫₀ᵗ f(s) ds using the trapezoidal rule on the stored pillar
    /// data plus the interpolated endpoint.
    fn integrate_forward(&self, t: Time) -> Real {
        if t <= 0.0 {
            return 0.0;
        }

        let mut integral = 0.0;
        let mut prev_t = 0.0;
        let mut prev_f = self.interp.operator(0.0);

        for &ti in &self.times {
            if ti <= 0.0 {
                continue;
            }
            if ti >= t {
                break;
            }
            let fi = self.interp.operator(ti);
            integral += 0.5 * (prev_f + fi) * (ti - prev_t);
            prev_t = ti;
            prev_f = fi;
        }

        // Final segment from prev_t to t
        let ft = self.interp.operator(t);
        integral += 0.5 * (prev_f + ft) * (t - prev_t);

        integral
    }
}

impl TermStructure for InterpolatedForwardCurve {
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

impl YieldTermStructure for InterpolatedForwardCurve {
    fn forward_rate_impl(&self, t: Time) -> Rate {
        self.interp.operator(t)
    }

    fn discount_impl(&self, t: Time) -> f64 {
        if t == 0.0 {
            return 1.0;
        }
        (-self.integrate_forward(t)).exp()
    }

    fn zero_rate_impl(&self, t: Time) -> Rate {
        if t == 0.0 {
            return self.forward_rate_impl(0.0);
        }
        self.integrate_forward(t) / t
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpolated_zero_curve::Linear;
    use approx::assert_abs_diff_eq;
    use ql_time::Actual365Fixed;

    fn flat_forward_data() -> (Vec<Date>, Vec<Rate>) {
        // Constant 5% forward rate at all pillars
        let dates = vec![
            Date::from_ymd(2025, 1, 2).unwrap(),
            Date::from_ymd(2025, 7, 2).unwrap(),
            Date::from_ymd(2026, 1, 2).unwrap(),
            Date::from_ymd(2027, 1, 2).unwrap(),
            Date::from_ymd(2030, 1, 2).unwrap(),
        ];
        let forwards = vec![0.05, 0.05, 0.05, 0.05, 0.05];
        (dates, forwards)
    }

    #[test]
    fn forward_curve_discount_at_zero() {
        let (dates, forwards) = flat_forward_data();
        let curve =
            InterpolatedForwardCurve::new(&dates, &forwards, Actual365Fixed, &Linear).unwrap();

        assert_abs_diff_eq!(curve.discount(0.0), 1.0, epsilon = 1e-15);
    }

    #[test]
    fn forward_curve_flat_forward_discount() {
        let (dates, forwards) = flat_forward_data();
        let curve =
            InterpolatedForwardCurve::new(&dates, &forwards, Actual365Fixed, &Linear).unwrap();

        // For a flat 5% forward, P(t) = exp(-0.05*t)
        for t in [0.5_f64, 1.0, 2.0, 5.0] {
            let expected = (-0.05 * t).exp();
            assert_abs_diff_eq!(curve.discount(t), expected, epsilon = 1e-8_f64);
        }
    }

    #[test]
    fn forward_curve_flat_forward_zero_rate() {
        let (dates, forwards) = flat_forward_data();
        let curve =
            InterpolatedForwardCurve::new(&dates, &forwards, Actual365Fixed, &Linear).unwrap();

        // Flat forward → zero rate also equals 5%
        for t in [0.5, 1.0, 2.0, 5.0] {
            assert_abs_diff_eq!(curve.zero_rate_impl(t), 0.05, epsilon = 1e-8);
        }
    }

    #[test]
    fn forward_curve_upward_sloping() {
        // Forward rates: 3%, 4%, 5%, 6%
        let dates = vec![
            Date::from_ymd(2025, 1, 2).unwrap(),
            Date::from_ymd(2026, 1, 2).unwrap(),
            Date::from_ymd(2027, 1, 2).unwrap(),
            Date::from_ymd(2030, 1, 2).unwrap(),
        ];
        let forwards = vec![0.03, 0.04, 0.05, 0.06];
        let curve =
            InterpolatedForwardCurve::new(&dates, &forwards, Actual365Fixed, &Linear).unwrap();

        // Zero rate over 1 year should be between 3% and 4%
        let z1 = curve.zero_rate_impl(1.0);
        assert!(z1 > 0.03 && z1 < 0.04, "z1 = {z1}");

        // Discount should decrease over time
        let d1 = curve.discount(1.0);
        let d2 = curve.discount(2.0);
        let d5 = curve.discount(5.0);
        assert!(d1 > d2);
        assert!(d2 > d5);
    }
}
