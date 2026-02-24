//! Inflation-linked coupons (translates parts of `ql/cashflows/inflationcoupon.hpp`).
//!
//! Provides:
//! * `CPICoupon` — a coupon whose notional is adjusted by a CPI ratio.
//! * `YoYInflationCoupon` — a coupon paying a year-on-year inflation rate.

use crate::cashflow::CashFlow;
use crate::coupon::Coupon;
use ql_core::Real;
use ql_time::{Actual365Fixed, Date, DayCounter};

// ── CPICoupon ─────────────────────────────────────────────────────────────────

/// A CPI-linked coupon — the notional is scaled by `CPI(observation) / base_CPI`.
///
/// Corresponds to `QuantLib::CPICoupon`.
#[derive(Debug, Clone)]
pub struct CPICoupon {
    /// Accrual start date.
    pub accrual_start: Date,
    /// Accrual end date (= payment date for simplicity).
    pub accrual_end: Date,
    /// Fixed coupon rate applied to inflation-adjusted notional.
    pub fixed_rate: Real,
    /// Base CPI level (typically at inception).
    pub base_cpi: Real,
    /// Observed CPI level for this coupon period.
    pub observation_cpi: Real,
    /// Original (un-adjusted) notional.
    pub notional: Real,
}

impl CPICoupon {
    /// The inflation-adjusted notional.
    pub fn adjusted_notional(&self) -> Real {
        self.notional * self.observation_cpi / self.base_cpi
    }
}

impl CashFlow for CPICoupon {
    fn date(&self) -> Date {
        self.accrual_end
    }
    fn amount(&self) -> Real {
        self.adjusted_notional() * self.fixed_rate
    }
}

impl Coupon for CPICoupon {
    fn nominal(&self) -> Real {
        self.notional
    }
    fn accrual_start_date(&self) -> Date {
        self.accrual_start
    }
    fn accrual_end_date(&self) -> Date {
        self.accrual_end
    }
    fn accrual_period(&self) -> Real {
        Actual365Fixed.year_fraction(self.accrual_start, self.accrual_end)
    }
    fn day_counter(&self) -> &dyn DayCounter {
        &Actual365Fixed
    }
    fn rate(&self) -> Real {
        self.fixed_rate * self.observation_cpi / self.base_cpi
    }
}

// ── YoYInflationCoupon ────────────────────────────────────────────────────────

/// A year-on-year inflation coupon — pays `notional × yoy_rate × daycount_fraction`.
///
/// Corresponds to `QuantLib::YoYInflationCoupon`.
#[derive(Debug, Clone)]
pub struct YoYInflationCoupon {
    /// Accrual start date.
    pub accrual_start: Date,
    /// Accrual end date.
    pub accrual_end: Date,
    /// The YoY rate observed for this period.
    pub yoy_rate: Real,
    /// Day-count fraction for this period.
    pub day_count_fraction: Real,
    /// Notional.
    pub notional: Real,
}

impl CashFlow for YoYInflationCoupon {
    fn date(&self) -> Date {
        self.accrual_end
    }
    fn amount(&self) -> Real {
        self.notional * self.yoy_rate * self.day_count_fraction
    }
}

impl Coupon for YoYInflationCoupon {
    fn nominal(&self) -> Real {
        self.notional
    }
    fn accrual_start_date(&self) -> Date {
        self.accrual_start
    }
    fn accrual_end_date(&self) -> Date {
        self.accrual_end
    }
    fn accrual_period(&self) -> Real {
        self.day_count_fraction
    }
    fn day_counter(&self) -> &dyn DayCounter {
        &Actual365Fixed
    }
    fn rate(&self) -> Real {
        self.yoy_rate
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpi_coupon_adjusted_notional() {
        let coupon = CPICoupon {
            accrual_start: Date::from_ymd(2024, 1, 1).unwrap(),
            accrual_end: Date::from_ymd(2025, 1, 1).unwrap(),
            fixed_rate: 0.02,
            base_cpi: 300.0,
            observation_cpi: 309.0,
            notional: 1_000_000.0,
        };
        // adjusted notional = 1M * 309/300 = 1_030_000
        let adj = coupon.adjusted_notional();
        assert!((adj - 1_030_000.0).abs() < 0.01);
        // amount = 1_030_000 * 0.02 = 20_600
        assert!((coupon.amount() - 20_600.0).abs() < 0.01);
    }

    #[test]
    fn cpi_coupon_rate_reflects_inflation() {
        let coupon = CPICoupon {
            accrual_start: Date::from_ymd(2024, 1, 1).unwrap(),
            accrual_end: Date::from_ymd(2025, 1, 1).unwrap(),
            fixed_rate: 0.02,
            base_cpi: 300.0,
            observation_cpi: 309.0,
            notional: 1_000_000.0,
        };
        // rate() = 0.02 * 309/300 = 0.0206
        assert!((coupon.rate() - 0.0206).abs() < 1e-10);
    }

    #[test]
    fn yoy_coupon_amount() {
        let coupon = YoYInflationCoupon {
            accrual_start: Date::from_ymd(2024, 1, 1).unwrap(),
            accrual_end: Date::from_ymd(2025, 1, 1).unwrap(),
            yoy_rate: 0.03,
            day_count_fraction: 1.0,
            notional: 1_000_000.0,
        };
        // 1M * 0.03 * 1.0 = 30_000
        assert!((coupon.amount() - 30_000.0).abs() < 0.01);
    }

    #[test]
    fn yoy_coupon_half_year() {
        let coupon = YoYInflationCoupon {
            accrual_start: Date::from_ymd(2024, 1, 1).unwrap(),
            accrual_end: Date::from_ymd(2024, 7, 1).unwrap(),
            yoy_rate: 0.04,
            day_count_fraction: 0.5,
            notional: 2_000_000.0,
        };
        // 2M * 0.04 * 0.5 = 40_000
        assert!((coupon.amount() - 40_000.0).abs() < 0.01);
    }
}
