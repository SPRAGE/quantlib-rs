//! Discounting bond pricing engine.
//!
//! Translates `ql/pricingengines/bond/discountingbondengine.hpp`.
//!
//! Prices bonds by discounting future cash flows using a given yield curve.
//! NPV = Σ cf.amount() × discount(cf.date()) for all future cash flows.

use std::sync::Arc;

use ql_cashflows::CashFlow;
use ql_core::{errors::Result, Real};
use ql_instruments::{Bond, PricingResults};
use ql_termstructures::YieldTermStructure;
use ql_time::Date;

/// Discounting bond pricing engine.
///
/// Computes the NPV of a bond by summing the discounted values of each
/// future cash flow:
///
/// $$\text{NPV} = \sum_{i : t_i > t_\text{settle}} c_i \cdot d(t_i)$$
///
/// where $c_i$ is the amount and $d(t_i)$ is the discount factor.
///
/// Corresponds to `QuantLib::DiscountingBondEngine`.
#[derive(Debug)]
pub struct DiscountingBondEngine {
    discount_curve: Arc<dyn YieldTermStructure>,
}

impl DiscountingBondEngine {
    /// Create a new engine with the given discount curve.
    pub fn new(discount_curve: Arc<dyn YieldTermStructure>) -> Self {
        Self { discount_curve }
    }

    /// Price a bond given its cash flows and settlement date.
    pub fn price(
        &self,
        cashflows: &[Box<dyn CashFlow>],
        settlement: Date,
    ) -> Result<PricingResults> {
        let mut npv = 0.0;
        for cf in cashflows {
            if cf.date() > settlement {
                let df = self.discount_curve.discount_date(cf.date());
                npv += cf.amount() * df;
            }
        }
        // Divide by the settlement discount factor to get a "dirty" price
        // relative to the settlement date (standard bond convention).
        let settle_df = self.discount_curve.discount_date(settlement);
        let dirty_price = if settle_df > 0.0 {
            npv / settle_df
        } else {
            npv
        };

        Ok(PricingResults::from_npv(npv)
            .with_result("dirty_price", dirty_price)
            .with_result("settlement_df", settle_df))
    }

    /// Convenience method: price a [`Bond`] using its embedded cash flows.
    pub fn price_bond(&self, bond: &Bond, settlement: Date) -> Result<PricingResults> {
        self.price(&bond.cashflows, settlement)
    }
}

/// Compute the clean price of a bond from its dirty price and accrued interest.
pub fn clean_price(dirty_price: Real, accrued: Real) -> Real {
    dirty_price - accrued
}

#[cfg(test)]
mod tests {
    use super::*;
    use ql_cashflows::SimpleCashFlow;
    use ql_termstructures::FlatForward;
    use ql_time::Actual365Fixed;

    #[test]
    fn discount_single_cashflow() {
        let ref_date = Date::from_ymd(2025, 1, 15).unwrap();
        let curve = Arc::new(FlatForward::continuous(ref_date, 0.05, Actual365Fixed));
        let engine = DiscountingBondEngine::new(curve);

        let maturity = Date::from_ymd(2026, 1, 15).unwrap();
        let cashflows: Vec<Box<dyn CashFlow>> = vec![
            Box::new(SimpleCashFlow::new(105.0, maturity)),
        ];

        let result = engine.price(&cashflows, ref_date).unwrap();
        // NPV ≈ 105 * exp(-0.05)
        let expected = 105.0 * (-0.05_f64).exp();
        assert!(
            (result.npv - expected).abs() < 0.01,
            "npv={}, expected={}",
            result.npv,
            expected
        );
    }

    #[test]
    fn discount_coupon_bond() {
        let ref_date = Date::from_ymd(2025, 1, 15).unwrap();
        let curve = Arc::new(FlatForward::continuous(ref_date, 0.05, Actual365Fixed));
        let engine = DiscountingBondEngine::new(curve.clone());

        // Simple bond: semi-annual 4% coupon on $100 face for 2 years + principal
        let dates = [
            Date::from_ymd(2025, 7, 15).unwrap(),
            Date::from_ymd(2026, 1, 15).unwrap(),
            Date::from_ymd(2026, 7, 15).unwrap(),
            Date::from_ymd(2027, 1, 15).unwrap(),
        ];
        let coupon = 2.0; // 4% / 2
        let mut cashflows: Vec<Box<dyn CashFlow>> = dates
            .iter()
            .map(|&d| Box::new(SimpleCashFlow::new(coupon, d)) as Box<dyn CashFlow>)
            .collect();
        // Add principal at maturity
        cashflows.push(Box::new(SimpleCashFlow::new(100.0, dates[3])));

        let result = engine.price(&cashflows, ref_date).unwrap();
        // Bond with lower coupon than market rate: price < par
        assert!(result.npv < 100.0, "npv = {} (should be < 100)", result.npv);
        assert!(result.npv > 90.0, "npv = {} (sanity check)", result.npv);
    }

    #[test]
    fn discount_zero_coupon_bond() {
        let ref_date = Date::from_ymd(2025, 1, 15).unwrap();
        let rate = 0.03;
        let curve = Arc::new(FlatForward::continuous(ref_date, rate, Actual365Fixed));
        let engine = DiscountingBondEngine::new(curve);

        let maturity = Date::from_ymd(2030, 1, 15).unwrap();
        let cashflows: Vec<Box<dyn CashFlow>> = vec![
            Box::new(SimpleCashFlow::new(100.0, maturity)),
        ];

        let result = engine.price(&cashflows, ref_date).unwrap();
        // 5-year zero: NPV = 100 * exp(-0.03 * 5)
        let expected = 100.0 * (-rate * 5.0_f64).exp();
        assert!(
            (result.npv - expected).abs() < 0.1,
            "npv={}, expected={}",
            result.npv,
            expected
        );
    }

    #[test]
    fn past_cashflows_excluded() {
        let ref_date = Date::from_ymd(2025, 6, 15).unwrap();
        let curve = Arc::new(FlatForward::continuous(ref_date, 0.05, Actual365Fixed));
        let engine = DiscountingBondEngine::new(curve);

        let past = Date::from_ymd(2025, 1, 15).unwrap();
        let future = Date::from_ymd(2026, 1, 15).unwrap();
        let cashflows: Vec<Box<dyn CashFlow>> = vec![
            Box::new(SimpleCashFlow::new(3.0, past)),
            Box::new(SimpleCashFlow::new(103.0, future)),
        ];

        let result = engine.price(&cashflows, ref_date).unwrap();
        // Only the future cash flow should be included
        assert!(result.npv > 95.0, "npv = {}", result.npv);
        assert!(result.npv < 105.0, "npv = {}", result.npv);
    }
}
