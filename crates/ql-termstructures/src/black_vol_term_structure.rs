//! `BlackVolTermStructure` — Black-volatility term structures
//! (translates `ql/termstructures/volatility/equityfx/blackvoltermstructure.hpp`).
//!
//! Provides the `BlackVolTermStructure` trait and concrete implementations:
//! * `BlackConstantVol` — a flat Black volatility surface.

use crate::term_structure::TermStructure;
use crate::volatility_term_structure::VolatilityTermStructure;
use crate::yield_term_structure::YieldTermStructureData;
use ql_core::{Real, Time, Volatility};
use ql_time::{Calendar, Date, DayCounter, NullCalendar};
use std::sync::Arc;

/// A Black-volatility term structure.
///
/// Implementors must provide **exactly one** of:
/// * [`black_vol_impl`](BlackVolTermStructure::black_vol_impl) — σ(t, k)
/// * [`black_variance_impl`](BlackVolTermStructure::black_variance_impl) — σ²·t
///
/// The other is derived automatically.
///
/// Corresponds to `QuantLib::BlackVolTermStructure`.
pub trait BlackVolTermStructure: VolatilityTermStructure {
    /// Return the Black volatility for time `t` and strike `strike`.
    fn black_vol_impl(&self, t: Time, strike: Real) -> Volatility {
        let var = self.black_variance_impl(t, strike);
        if t <= 0.0 {
            return 0.0;
        }
        (var / t).sqrt()
    }

    /// Return the Black variance `σ²·t` for time `t` and strike `strike`.
    fn black_variance_impl(&self, t: Time, strike: Real) -> Real {
        let vol = self.black_vol_impl(t, strike);
        vol * vol * t
    }

    /// Black volatility for a date and strike.
    fn black_vol(&self, date: Date, strike: Real) -> Volatility {
        let t = self.time_from_reference(date);
        self.black_vol_impl(t, strike)
    }

    /// Black variance for a date and strike.
    fn black_variance(&self, date: Date, strike: Real) -> Real {
        let t = self.time_from_reference(date);
        self.black_variance_impl(t, strike)
    }

    /// Black volatility for a time and strike.
    fn black_vol_time(&self, t: Time, strike: Real) -> Volatility {
        self.black_vol_impl(t, strike)
    }

    /// Black variance for a time and strike.
    fn black_variance_time(&self, t: Time, strike: Real) -> Real {
        self.black_variance_impl(t, strike)
    }
}

// ── BlackConstantVol ──────────────────────────────────────────────────────────

/// A flat (constant) Black volatility surface.
///
/// `σ(t, K) = constant` for all `t > 0` and all strikes `K`.
///
/// Corresponds to `QuantLib::BlackConstantVol`.
#[derive(Debug)]
pub struct BlackConstantVol {
    data: YieldTermStructureData,
    volatility: Volatility,
}

impl BlackConstantVol {
    /// Create a constant Black vol surface.
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

impl TermStructure for BlackConstantVol {
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

impl VolatilityTermStructure for BlackConstantVol {
    fn min_strike(&self) -> Real {
        f64::NEG_INFINITY
    }

    fn max_strike(&self) -> Real {
        f64::INFINITY
    }
}

impl BlackVolTermStructure for BlackConstantVol {
    fn black_vol_impl(&self, _t: Time, _strike: Real) -> Volatility {
        self.volatility
    }

    fn black_variance_impl(&self, t: Time, _strike: Real) -> Real {
        self.volatility * self.volatility * t
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use ql_time::Actual365Fixed;

    #[test]
    fn constant_vol_value() {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let surface = BlackConstantVol::new(ref_date, 0.20, Actual365Fixed);

        assert_abs_diff_eq!(surface.black_vol_impl(1.0, 100.0), 0.20, epsilon = 1e-15);
        assert_abs_diff_eq!(surface.black_vol_impl(5.0, 50.0), 0.20, epsilon = 1e-15);
    }

    #[test]
    fn constant_vol_variance() {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let surface = BlackConstantVol::new(ref_date, 0.20, Actual365Fixed);

        // Variance = σ² × t
        assert_abs_diff_eq!(
            surface.black_variance_impl(2.0, 100.0),
            0.04 * 2.0,
            epsilon = 1e-15
        );
    }

    #[test]
    fn constant_vol_at_date() {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let surface = BlackConstantVol::new(ref_date, 0.25, Actual365Fixed);

        let d1 = Date::from_ymd(2026, 1, 2).unwrap();
        let vol = surface.black_vol(d1, 100.0);
        assert_abs_diff_eq!(vol, 0.25, epsilon = 1e-15);
    }

    #[test]
    fn constant_vol_strike_range() {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let surface = BlackConstantVol::new(ref_date, 0.20, Actual365Fixed);

        assert!(surface.min_strike() < 0.0);
        assert!(surface.max_strike() > 1e10);
    }
}
