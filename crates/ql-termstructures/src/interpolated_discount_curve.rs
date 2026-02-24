//! `InterpolatedDiscountCurve` — a yield term structure bootstrapped from
//! discount factors (translates
//! `ql/termstructures/yield/interpolateddiscountcurve.hpp`).
//!
//! The curve stores (date, discount-factor) pairs and interpolates them as a
//! function of time.  Zero rates and forward rates are derived from `P(t)`.

use crate::term_structure::TermStructure;
use crate::yield_term_structure::{YieldTermStructure, YieldTermStructureData};
use crate::interpolated_zero_curve::InterpolationBuilder;
use ql_core::{errors::Result, DiscountFactor, Real, Time};
use ql_math::Interpolation1D;
use ql_time::{Calendar, Date, DayCounter, NullCalendar};
use std::sync::Arc;

/// A yield curve defined by discount factors at known dates.
///
/// Log-linear interpolation on discounts gives piecewise-constant forward rates;
/// linear interpolation on discounts is also commonly used.
///
/// Corresponds to `QuantLib::InterpolatedDiscountCurve<Interpolator>`.
#[derive(Debug)]
pub struct InterpolatedDiscountCurve {
    data: YieldTermStructureData,
    dates: Vec<Date>,
    times: Vec<Real>,
    discounts: Vec<DiscountFactor>,
    interp: Box<dyn Interpolation1D>,
    max_date: Date,
}

impl InterpolatedDiscountCurve {
    /// Build a discount-factor curve from dates and corresponding discount factors.
    ///
    /// The first date must be the reference date with a discount factor of 1.0.
    /// Dates must be sorted in ascending order with strictly decreasing
    /// discount factors.
    ///
    /// # Arguments
    /// * `dates` — pillar dates (first entry = reference date)
    /// * `discounts` — discount factors at each date (first must be 1.0)
    /// * `day_counter` — used for date → time conversion
    /// * `builder` — interpolation strategy
    pub fn new(
        dates: &[Date],
        discounts: &[DiscountFactor],
        day_counter: impl DayCounter + 'static,
        builder: &dyn InterpolationBuilder,
    ) -> Result<Self> {
        ql_core::ensure!(
            dates.len() >= 2,
            "need at least 2 dates (reference + 1 pillar)"
        );
        ql_core::ensure!(
            dates.len() == discounts.len(),
            "dates and discounts must have the same length"
        );
        ql_core::ensure!(
            (discounts[0] - 1.0).abs() < 1e-12,
            "first discount factor must be 1.0"
        );

        let reference_date = dates[0];
        let dc: Arc<dyn DayCounter> = Arc::new(day_counter);

        let times: Vec<Real> = dates
            .iter()
            .map(|&d| dc.year_fraction(reference_date, d))
            .collect();

        let interp = builder.build(&times, discounts)?;
        let max_date = *dates.last().unwrap();

        Ok(Self {
            data: YieldTermStructureData {
                reference_date,
                calendar: Box::new(NullCalendar),
                day_counter: dc,
            },
            dates: dates.to_vec(),
            times,
            discounts: discounts.to_vec(),
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

    /// Return the pillar discount factors.
    pub fn discounts(&self) -> &[DiscountFactor] {
        &self.discounts
    }
}

impl TermStructure for InterpolatedDiscountCurve {
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

impl YieldTermStructure for InterpolatedDiscountCurve {
    fn discount_impl(&self, t: Time) -> DiscountFactor {
        if t == 0.0 {
            return 1.0;
        }
        self.interp.operator(t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpolated_zero_curve::LogLinear;
    use approx::assert_abs_diff_eq;
    use ql_time::Actual365Fixed;

    fn sample_dates_discounts() -> (Vec<Date>, Vec<DiscountFactor>) {
        // 5% flat continuous rate → P(t) = exp(-0.05 * t)
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let dates = vec![
            ref_date,
            Date::from_ymd(2025, 7, 2).unwrap(),
            Date::from_ymd(2026, 1, 2).unwrap(),
            Date::from_ymd(2027, 1, 2).unwrap(),
            Date::from_ymd(2030, 1, 2).unwrap(),
        ];
        let dc = Actual365Fixed;
        let discounts: Vec<DiscountFactor> = dates
            .iter()
            .map(|&d| {
                let t = dc.year_fraction(ref_date, d);
                (-0.05 * t).exp()
            })
            .collect();
        (dates, discounts)
    }

    #[test]
    fn discount_curve_at_ref_date() {
        let (dates, discounts) = sample_dates_discounts();
        let curve = InterpolatedDiscountCurve::new(
            &dates,
            &discounts,
            Actual365Fixed,
            &LogLinear,
        )
        .unwrap();

        assert_abs_diff_eq!(curve.discount(0.0), 1.0, epsilon = 1e-15);
    }

    #[test]
    fn discount_curve_at_pillars() {
        let (dates, discounts) = sample_dates_discounts();
        let curve = InterpolatedDiscountCurve::new(
            &dates,
            &discounts,
            Actual365Fixed,
            &LogLinear,
        )
        .unwrap();

        for (i, &d) in dates.iter().enumerate() {
            let t = curve.time_from_reference(d);
            assert_abs_diff_eq!(curve.discount(t), discounts[i], epsilon = 1e-10);
        }
    }

    #[test]
    fn discount_curve_log_linear_implies_flat_rate() {
        // With log-linear interpolation on discount factors generated from a
        // flat rate, the interpolated curve should reproduce that flat rate.
        let (dates, discounts) = sample_dates_discounts();
        let curve = InterpolatedDiscountCurve::new(
            &dates,
            &discounts,
            Actual365Fixed,
            &LogLinear,
        )
        .unwrap();

        // Check zero rate at an intermediate point
        let t = 1.5;
        let z = curve.zero_rate_impl(t);
        assert_abs_diff_eq!(z, 0.05, epsilon = 1e-8);
    }

    #[test]
    fn discount_curve_forward_rate() {
        let (dates, discounts) = sample_dates_discounts();
        let curve = InterpolatedDiscountCurve::new(
            &dates,
            &discounts,
            Actual365Fixed,
            &LogLinear,
        )
        .unwrap();

        // For a flat 5% curve, instantaneous forward should be ~5%
        let f = curve.forward_rate_impl(2.0);
        assert_abs_diff_eq!(f, 0.05, epsilon = 1e-4);
    }
}
