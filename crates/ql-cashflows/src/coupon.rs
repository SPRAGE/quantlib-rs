//! `Coupon` trait — base for all interest-rate coupons.
//!
//! Translates `ql/cashflows/coupon.hpp`.
//!
//! A coupon is a cash flow that accrues interest over an accrual period
//! `[accrual_start, accrual_end)` and pays on a payment date.

use crate::cashflow::CashFlow;
use ql_core::Real;
use ql_time::{Date, DayCounter};

/// Base trait for interest-rate coupons.
///
/// Corresponds to `QuantLib::Coupon`.
pub trait Coupon: CashFlow {
    /// The notional (face) amount.
    fn nominal(&self) -> Real;

    /// Start of the accrual period.
    fn accrual_start_date(&self) -> Date;

    /// End of the accrual period.
    fn accrual_end_date(&self) -> Date;

    /// Reference period start (may differ for irregular coupons).
    fn reference_period_start(&self) -> Date {
        self.accrual_start_date()
    }

    /// Reference period end (may differ for irregular coupons).
    fn reference_period_end(&self) -> Date {
        self.accrual_end_date()
    }

    /// The accrual period in year-fraction units.
    fn accrual_period(&self) -> Real;

    /// The day counter used for accrual.
    fn day_counter(&self) -> &dyn DayCounter;

    /// The annualized rate of the coupon.
    fn rate(&self) -> Real;

    /// Accrued amount from the accrual start to the given date.
    fn accrued_amount(&self, date: Date) -> Real {
        if date <= self.accrual_start_date() || date > self.accrual_end_date() {
            return 0.0;
        }
        let full_amount = self.amount();
        if date >= self.accrual_end_date() {
            return full_amount;
        }
        let dc = self.day_counter();
        let accrued_fraction = dc.year_fraction(self.accrual_start_date(), date)
            / dc.year_fraction(self.accrual_start_date(), self.accrual_end_date());
        full_amount * accrued_fraction
    }
}
