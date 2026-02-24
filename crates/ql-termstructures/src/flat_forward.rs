//! `FlatForward` — a yield term structure with a constant forward rate
//! (translates `ql/termstructures/yield/flatforward.hpp`).
//!
//! This is the simplest possible yield curve: a constant continuously-compounded
//! rate that applies for all maturities.

use crate::term_structure::TermStructure;
use crate::yield_term_structure::{YieldTermStructure, YieldTermStructureData};
use ql_core::{Compounding, Rate, Time};
use ql_time::{
    Actual365Fixed, Calendar, Date, DayCounter, Frequency, InterestRate, NullCalendar,
};
use std::sync::Arc;

/// A flat (constant) forward-rate yield term structure.
///
/// Discount factors are computed as `P(t) = exp(-r * t)` where `r` is the
/// continuously-compounded equivalent of the supplied rate.
///
/// Corresponds to `QuantLib::FlatForward`.
#[derive(Debug)]
pub struct FlatForward {
    data: YieldTermStructureData,
    /// The continuously-compounded flat rate.
    rate: Rate,
}

impl FlatForward {
    /// Create a flat-forward curve from a given rate and compounding convention.
    ///
    /// The rate is immediately converted to the equivalent continuous rate.
    pub fn new(
        reference_date: Date,
        rate: Rate,
        day_counter: impl DayCounter + 'static,
        compounding: Compounding,
        frequency: Frequency,
    ) -> Self {
        // Convert the given rate to a continuously-compounded rate
        // by creating a temporary InterestRate and computing its compound
        // factor over 1 year.
        let ir = InterestRate::new(rate, Actual365Fixed, compounding, frequency);
        let continuous_rate = if rate.abs() < f64::EPSILON {
            0.0
        } else {
            // compound factor over 1 year, then take log
            let cf = ir.compound_factor_time(1.0);
            cf.ln()
        };
        Self {
            data: YieldTermStructureData {
                reference_date,
                calendar: Box::new(NullCalendar),
                day_counter: Arc::new(day_counter),
            },
            rate: continuous_rate,
        }
    }

    /// Create a flat-forward curve assuming continuous compounding.
    pub fn continuous(
        reference_date: Date,
        rate: Rate,
        day_counter: impl DayCounter + 'static,
    ) -> Self {
        Self::new(
            reference_date,
            rate,
            day_counter,
            Compounding::Continuous,
            Frequency::NoFrequency,
        )
    }

    /// Create a flat-forward curve with a specific calendar.
    pub fn with_calendar(
        mut self,
        calendar: impl Calendar + 'static,
    ) -> Self {
        self.data.calendar = Box::new(calendar);
        self
    }

    /// The continuously-compounded flat rate.
    pub fn rate(&self) -> Rate {
        self.rate
    }
}

impl TermStructure for FlatForward {
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
        Date::MAX
    }
}

impl YieldTermStructure for FlatForward {
    fn discount_impl(&self, t: Time) -> f64 {
        (-self.rate * t).exp()
    }

    fn zero_rate_impl(&self, _t: Time) -> Rate {
        self.rate
    }

    fn forward_rate_impl(&self, _t: Time) -> Rate {
        self.rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use ql_time::Actual365Fixed;

    #[test]
    fn flat_forward_discount() {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let curve = FlatForward::continuous(ref_date, 0.05, Actual365Fixed);

        // At reference date, discount = 1
        assert_abs_diff_eq!(curve.discount(0.0), 1.0, epsilon = 1e-15);
        // At 1 year, discount = exp(-0.05)
        assert_abs_diff_eq!(curve.discount(1.0), (-0.05_f64).exp(), epsilon = 1e-12);
        // At 10 years
        assert_abs_diff_eq!(curve.discount(10.0), (-0.5_f64).exp(), epsilon = 1e-12);
    }

    #[test]
    fn flat_forward_zero_rate() {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let curve = FlatForward::continuous(ref_date, 0.03, Actual365Fixed);

        assert_abs_diff_eq!(curve.zero_rate_impl(0.5), 0.03, epsilon = 1e-15);
        assert_abs_diff_eq!(curve.zero_rate_impl(5.0), 0.03, epsilon = 1e-15);
    }

    #[test]
    fn flat_forward_forward_rate() {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let curve = FlatForward::continuous(ref_date, 0.04, Actual365Fixed);

        // Forward rate is constant everywhere
        assert_abs_diff_eq!(curve.forward_rate_impl(0.0), 0.04, epsilon = 1e-15);
        assert_abs_diff_eq!(curve.forward_rate_impl(3.0), 0.04, epsilon = 1e-15);
    }

    #[test]
    fn flat_forward_with_annual_compounding() {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let curve = FlatForward::new(
            ref_date,
            0.05,
            Actual365Fixed,
            Compounding::Compounded,
            Frequency::Annual,
        );
        // Annual 5% → continuous = ln(1.05) ≈ 0.04879
        let expected = (1.05_f64).ln();
        assert_abs_diff_eq!(curve.rate(), expected, epsilon = 1e-12);
    }

    #[test]
    fn flat_forward_discount_date() {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let curve = FlatForward::continuous(ref_date, 0.05, Actual365Fixed);

        // Discount at reference date = 1
        assert_abs_diff_eq!(curve.discount_date(ref_date), 1.0, epsilon = 1e-15);

        // 1 year later
        let d1 = Date::from_ymd(2026, 1, 2).unwrap();
        let t = curve.time_from_reference(d1);
        assert_abs_diff_eq!(curve.discount_date(d1), (-0.05 * t).exp(), epsilon = 1e-10);
    }

    #[test]
    fn flat_forward_zero_rate_output() {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let dc = Actual365Fixed;
        let curve = FlatForward::continuous(ref_date, 0.05, Actual365Fixed);

        let d1 = Date::from_ymd(2026, 1, 2).unwrap();
        let zr = curve.zero_rate(d1, &dc, Compounding::Continuous, Frequency::NoFrequency);
        assert_abs_diff_eq!(zr.rate(), 0.05, epsilon = 1e-10);
    }
}
