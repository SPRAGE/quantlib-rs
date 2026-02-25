//! `LocalVolTermStructure` — local-volatility term structures
//! (translates `ql/termstructures/volatility/equityfx/localvoltermstructure.hpp`).
//!
//! Provides the `LocalVolTermStructure` trait and `LocalConstantVol`.

use crate::term_structure::TermStructure;
use crate::volatility_term_structure::VolatilityTermStructure;
use crate::yield_term_structure::YieldTermStructureData;
use ql_core::{Real, Time, Volatility};
use ql_time::{Calendar, Date, DayCounter, NullCalendar};
use std::sync::Arc;

/// A local-volatility term structure: `σ_local(t, S)`.
///
/// Corresponds to `QuantLib::LocalVolTermStructure`.
pub trait LocalVolTermStructure: VolatilityTermStructure {
    /// Return the local volatility for time `t` and underlying price `underlying`.
    fn local_vol_impl(&self, t: Time, underlying: Real) -> Volatility;

    /// Local volatility for a date and underlying price.
    fn local_vol(&self, date: Date, underlying: Real) -> Volatility {
        let t = self.time_from_reference(date);
        self.local_vol_impl(t, underlying)
    }

    /// Local volatility for a time and underlying price.
    fn local_vol_time(&self, t: Time, underlying: Real) -> Volatility {
        self.local_vol_impl(t, underlying)
    }
}

// ── LocalConstantVol ──────────────────────────────────────────────────────────

/// A constant local volatility surface.
///
/// `σ_local(t, S) = constant` for all `t` and `S`.
///
/// Corresponds to `QuantLib::LocalConstantVol`.
#[derive(Debug)]
pub struct LocalConstantVol {
    data: YieldTermStructureData,
    volatility: Volatility,
}

impl LocalConstantVol {
    /// Create a constant local vol surface.
    pub fn new(
        reference_date: Date,
        volatility: Volatility,
        day_counter: impl DayCounter + 'static,
    ) -> Self {
        Self {
            data: YieldTermStructureData {
                reference_date,
                calendar: Box::new(NullCalendar),
                day_counter: Arc::new(day_counter),
            },
            volatility,
        }
    }

    /// Create with a specific calendar.
    pub fn with_calendar(mut self, calendar: impl Calendar + 'static) -> Self {
        self.data.calendar = Box::new(calendar);
        self
    }

    /// The constant volatility value.
    pub fn volatility(&self) -> Volatility {
        self.volatility
    }
}

impl TermStructure for LocalConstantVol {
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

impl VolatilityTermStructure for LocalConstantVol {
    fn min_strike(&self) -> Real {
        f64::NEG_INFINITY
    }

    fn max_strike(&self) -> Real {
        f64::INFINITY
    }
}

impl LocalVolTermStructure for LocalConstantVol {
    fn local_vol_impl(&self, _t: Time, _underlying: Real) -> Volatility {
        self.volatility
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use ql_time::Actual365Fixed;

    #[test]
    fn local_constant_vol_value() {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let surface = LocalConstantVol::new(ref_date, 0.30, Actual365Fixed);

        assert_abs_diff_eq!(surface.local_vol_impl(1.0, 100.0), 0.30, epsilon = 1e-15);
        assert_abs_diff_eq!(surface.local_vol_impl(5.0, 50.0), 0.30, epsilon = 1e-15);
    }

    #[test]
    fn local_constant_vol_at_date() {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let surface = LocalConstantVol::new(ref_date, 0.25, Actual365Fixed);

        let d1 = Date::from_ymd(2026, 1, 2).unwrap();
        assert_abs_diff_eq!(surface.local_vol(d1, 105.0), 0.25, epsilon = 1e-15);
    }

    #[test]
    fn local_constant_vol_strike_range() {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let surface = LocalConstantVol::new(ref_date, 0.20, Actual365Fixed);

        assert!(surface.min_strike() < 0.0);
        assert!(surface.max_strike() > 1e10);
    }
}
