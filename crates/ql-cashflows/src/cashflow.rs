//! `CashFlow` trait — the base for all cash-flow types.
//!
//! Translates `ql/cashflows/cashflow.hpp`.
//!
//! A cash flow is an amount of money paid or received at a specific date.

use ql_core::Real;
use ql_time::Date;
use std::fmt;

/// Base trait for all cash flows.
///
/// A cash flow knows its payment date and can compute the amount
/// paid on that date. For coupons, the amount depends on a rate;
/// for simple cash flows, it is a fixed value.
///
/// Corresponds to `QuantLib::CashFlow`.
pub trait CashFlow: fmt::Debug + Send + Sync {
    /// The date on which this cash flow is paid.
    fn date(&self) -> Date;

    /// The amount of cash paid on the payment date.
    fn amount(&self) -> Real;

    /// Whether this cash flow has already occurred relative to `ref_date`.
    /// Uses a strict "less-than" comparison: a flow on `ref_date` has NOT
    /// yet occurred.
    fn has_occurred(&self, ref_date: Date) -> bool {
        self.date() < ref_date
    }

    /// Whether this cash flow is still pending (tradeable) relative to
    /// `ref_date`.
    fn is_trading_cashflow(&self, ref_date: Date) -> bool {
        !self.has_occurred(ref_date)
    }
}

/// A `Leg` is a sequence of cash flows.
///
/// Corresponds to `QuantLib::Leg` (= `std::vector<ext::shared_ptr<CashFlow>>`).
pub type Leg = Vec<Box<dyn CashFlow>>;

/// A simple cash flow: a fixed amount at a fixed date.
///
/// Corresponds to `QuantLib::SimpleCashFlow`.
#[derive(Debug, Clone)]
pub struct SimpleCashFlow {
    /// The payment amount.
    pub amount: Real,
    /// The payment date.
    pub date: Date,
}

impl SimpleCashFlow {
    /// Create a new simple cash flow.
    pub fn new(amount: Real, date: Date) -> Self {
        Self { amount, date }
    }
}

impl CashFlow for SimpleCashFlow {
    fn date(&self) -> Date {
        self.date
    }

    fn amount(&self) -> Real {
        self.amount
    }
}

/// A redemption (notional repayment) at a specific date.
///
/// Corresponds to `QuantLib::Redemption`.
#[derive(Debug, Clone)]
pub struct Redemption {
    /// The redemption amount.
    pub amount: Real,
    /// The redemption date.
    pub date: Date,
}

impl Redemption {
    /// Create a new redemption cash flow.
    pub fn new(amount: Real, date: Date) -> Self {
        Self { amount, date }
    }
}

impl CashFlow for Redemption {
    fn date(&self) -> Date {
        self.date
    }

    fn amount(&self) -> Real {
        self.amount
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_cashflow() {
        let d = Date::from_ymd(2025, 6, 15).unwrap();
        let cf = SimpleCashFlow::new(100.0, d);
        assert!((cf.amount() - 100.0).abs() < 1e-15);
        assert_eq!(cf.date(), d);
    }

    #[test]
    fn has_occurred() {
        let d = Date::from_ymd(2025, 6, 15).unwrap();
        let cf = SimpleCashFlow::new(100.0, d);
        let before = Date::from_ymd(2025, 6, 14).unwrap();
        let on = Date::from_ymd(2025, 6, 15).unwrap();
        let after = Date::from_ymd(2025, 6, 16).unwrap();
        assert!(!cf.has_occurred(before));
        assert!(!cf.has_occurred(on)); // on date: not yet occurred
        assert!(cf.has_occurred(after));
    }

    #[test]
    fn redemption() {
        let d = Date::from_ymd(2030, 1, 15).unwrap();
        let r = Redemption::new(1000.0, d);
        assert!((r.amount() - 1000.0).abs() < 1e-15);
        assert_eq!(r.date(), d);
    }
}
