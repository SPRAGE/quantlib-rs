//! Inflation term structures (translates `ql/termstructures/inflation/`).
//!
//! Provides:
//! * `InflationTermStructure` — base trait for all inflation curves.
//! * `ZeroInflationTermStructure` — zero-coupon (CPI) inflation curves.
//! * `YoYInflationTermStructure` — year-on-year inflation curves.
//! * `FlatZeroInflationCurve` — constant zero-inflation-rate curve.
//! * `FlatYoYInflationCurve` — constant YoY-inflation-rate curve.

use crate::term_structure::TermStructure;
use ql_core::{Rate, Time};
use ql_time::{
    Actual365Fixed, Calendar, Date, DayCounter, Frequency, NullCalendar, Period, TimeUnit,
};

// ── Base trait ────────────────────────────────────────────────────────────────

/// Common interface for inflation term structures.
///
/// Corresponds to `QuantLib::InflationTermStructure`.
pub trait InflationTermStructure: TermStructure {
    /// Observation frequency (e.g. `Monthly`).
    fn frequency(&self) -> Frequency;

    /// Observation lag — the time between the reference period end and the
    /// date when the fixing becomes available.
    fn observation_lag(&self) -> Period;

    /// The base date — the earliest date of the reference period for which a
    /// fixing is available (reference date minus observation lag).
    fn base_date(&self) -> Date {
        let lag = self.observation_lag();
        self.reference_date()
            .advance(-lag.length, lag.unit)
            .expect("base_date subtraction")
    }

    /// The base rate (zero-coupon or YoY) at the base date.
    fn base_rate(&self) -> Rate;
}

// ── Zero-inflation ────────────────────────────────────────────────────────────

/// A zero-coupon inflation curve.
///
/// Returns the compounded CPI-style zero-inflation rate for a given time.
///
/// Corresponds to `QuantLib::ZeroInflationTermStructure`.
pub trait ZeroInflationTermStructure: InflationTermStructure {
    /// The zero-inflation rate for time `t` (year fraction from base date).
    fn zero_rate_impl(&self, t: Time) -> Rate;

    /// The zero-inflation rate for a given date.
    fn zero_rate(&self, date: Date) -> Rate {
        let t = self.time_from_reference(date);
        self.zero_rate_impl(t)
    }
}

/// Constant zero-inflation rate.
///
/// Analogous to `FlatForward` for yield curves.
#[derive(Debug)]
pub struct FlatZeroInflationCurve {
    reference_date: Date,
    rate: Rate,
    frequency: Frequency,
    observation_lag: Period,
}

impl FlatZeroInflationCurve {
    /// Create a flat zero-inflation curve.
    pub fn new(reference_date: Date, rate: Rate, frequency: Frequency, observation_lag: Period) -> Self {
        Self {
            reference_date,
            rate,
            frequency,
            observation_lag,
        }
    }
}

impl TermStructure for FlatZeroInflationCurve {
    fn reference_date(&self) -> Date {
        self.reference_date
    }
    fn day_counter(&self) -> &dyn DayCounter {
        &Actual365Fixed
    }
    fn calendar(&self) -> &dyn Calendar {
        &NullCalendar
    }
    fn max_date(&self) -> Date {
        Date::MAX
    }
}

impl InflationTermStructure for FlatZeroInflationCurve {
    fn frequency(&self) -> Frequency {
        self.frequency
    }
    fn observation_lag(&self) -> Period {
        self.observation_lag
    }
    fn base_rate(&self) -> Rate {
        self.rate
    }
}

impl ZeroInflationTermStructure for FlatZeroInflationCurve {
    fn zero_rate_impl(&self, _t: Time) -> Rate {
        self.rate
    }
}

// ── YoY-inflation ─────────────────────────────────────────────────────────────

/// A year-on-year inflation curve.
///
/// Returns the year-on-year inflation rate for a given time.
///
/// Corresponds to `QuantLib::YoYInflationTermStructure`.
pub trait YoYInflationTermStructure: InflationTermStructure {
    /// The YoY inflation rate for time `t`.
    fn yoy_rate_impl(&self, t: Time) -> Rate;

    /// The YoY inflation rate for a given date.
    fn yoy_rate(&self, date: Date) -> Rate {
        let t = self.time_from_reference(date);
        self.yoy_rate_impl(t)
    }
}

/// Constant YoY inflation rate.
#[derive(Debug)]
pub struct FlatYoYInflationCurve {
    reference_date: Date,
    rate: Rate,
    frequency: Frequency,
    observation_lag: Period,
}

impl FlatYoYInflationCurve {
    /// Create a flat YoY inflation curve.
    pub fn new(reference_date: Date, rate: Rate, frequency: Frequency, observation_lag: Period) -> Self {
        Self {
            reference_date,
            rate,
            frequency,
            observation_lag,
        }
    }
}

impl TermStructure for FlatYoYInflationCurve {
    fn reference_date(&self) -> Date {
        self.reference_date
    }
    fn day_counter(&self) -> &dyn DayCounter {
        &Actual365Fixed
    }
    fn calendar(&self) -> &dyn Calendar {
        &NullCalendar
    }
    fn max_date(&self) -> Date {
        Date::MAX
    }
}

impl InflationTermStructure for FlatYoYInflationCurve {
    fn frequency(&self) -> Frequency {
        self.frequency
    }
    fn observation_lag(&self) -> Period {
        self.observation_lag
    }
    fn base_rate(&self) -> Rate {
        self.rate
    }
}

impl YoYInflationTermStructure for FlatYoYInflationCurve {
    fn yoy_rate_impl(&self, _t: Time) -> Rate {
        self.rate
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_zero_inflation_curve() {
        let ref_date = Date::from_ymd(2024, 1, 1).unwrap();
        let lag = Period::new(3, TimeUnit::Months);
        let curve = FlatZeroInflationCurve::new(ref_date, 0.025, Frequency::Monthly, lag);
        assert_eq!(curve.frequency(), Frequency::Monthly);
        assert_eq!(curve.observation_lag(), lag);
        assert!((curve.base_rate() - 0.025).abs() < 1e-12);

        let d = Date::from_ymd(2025, 1, 1).unwrap();
        assert!((curve.zero_rate(d) - 0.025).abs() < 1e-12);
    }

    #[test]
    fn flat_yoy_inflation_curve() {
        let ref_date = Date::from_ymd(2024, 1, 1).unwrap();
        let lag = Period::new(3, TimeUnit::Months);
        let curve = FlatYoYInflationCurve::new(ref_date, 0.03, Frequency::Monthly, lag);
        assert!((curve.base_rate() - 0.03).abs() < 1e-12);

        let d = Date::from_ymd(2026, 6, 15).unwrap();
        assert!((curve.yoy_rate(d) - 0.03).abs() < 1e-12);
    }

    #[test]
    fn base_date_accounts_for_lag() {
        let ref_date = Date::from_ymd(2024, 6, 1).unwrap();
        let lag = Period::new(3, TimeUnit::Months);
        let curve = FlatZeroInflationCurve::new(ref_date, 0.02, Frequency::Monthly, lag);
        let base = curve.base_date();
        // 2024-06-01 minus 3 months = 2024-03-01
        assert_eq!(base, Date::from_ymd(2024, 3, 1).unwrap());
    }

    #[test]
    fn zero_inflation_time_from_reference() {
        let ref_date = Date::from_ymd(2024, 1, 1).unwrap();
        let curve = FlatZeroInflationCurve::new(
            ref_date,
            0.02,
            Frequency::Monthly,
            Period::new(3, TimeUnit::Months),
        );
        let t = curve.time_from_reference(Date::from_ymd(2025, 1, 1).unwrap());
        // Actual365Fixed: 366/365 ≈ 1.002739
        assert!((t - 366.0 / 365.0).abs() < 1e-6);
    }
}
