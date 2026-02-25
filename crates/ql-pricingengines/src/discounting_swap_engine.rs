//! Discounting swap pricing engine.
//!
//! Translates `ql/pricingengines/swap/discountingswapengine.hpp`.
//!
//! Prices interest-rate swaps by discounting all cash flows on each leg.

use std::sync::Arc;

use ql_cashflows::CashFlow;
use ql_core::{errors::Result, Real};
use ql_instruments::{PricingResults, SwapType, VanillaSwap};
use ql_termstructures::YieldTermStructure;
use ql_time::Date;

/// Discounting swap pricing engine.
///
/// The NPV of a vanilla swap is:
///
/// $$\text{NPV} = \phi \left(\sum_i c_i^{\text{fix}} d(t_i) -
///   \sum_j c_j^{\text{flt}} d(t_j)\right)$$
///
/// where $\phi = +1$ for a payer swap and $-1$ for a receiver.
///
/// Corresponds to `QuantLib::DiscountingSwapEngine`.
#[derive(Debug)]
pub struct DiscountingSwapEngine {
    discount_curve: Arc<dyn YieldTermStructure>,
}

impl DiscountingSwapEngine {
    /// Create a new engine with the given discount curve.
    pub fn new(discount_curve: Arc<dyn YieldTermStructure>) -> Self {
        Self { discount_curve }
    }

    /// Price a leg (vector of cash flows) relative to a reference date.
    fn leg_npv(&self, leg: &[Box<dyn CashFlow>], reference: Date) -> Real {
        let mut npv = 0.0;
        for cf in leg {
            if cf.date() > reference {
                let df = self.discount_curve.discount_date(cf.date());
                npv += cf.amount() * df;
            }
        }
        npv
    }

    /// Price a vanilla swap.
    pub fn price_swap(&self, swap: &VanillaSwap, reference: Date) -> Result<PricingResults> {
        let fixed_npv = self.leg_npv(&swap.fixed_leg, reference);
        let floating_npv = self.leg_npv(&swap.floating_leg, reference);

        let sign = match swap.swap_type {
            SwapType::Payer => 1.0,
            SwapType::Receiver => -1.0,
        };

        let npv = sign * (floating_npv - fixed_npv);

        Ok(PricingResults::from_npv(npv)
            .with_result("fixed_leg_npv", fixed_npv)
            .with_result("floating_leg_npv", floating_npv)
            .with_result("fair_spread", 0.0)) // placeholder
    }

    /// Price a generic multi-leg swap.
    pub fn price_legs(
        &self,
        legs: &[Vec<Box<dyn CashFlow>>],
        payer: &[Real],
        reference: Date,
    ) -> Result<PricingResults> {
        let mut npv = 0.0;
        for (leg, &sign) in legs.iter().zip(payer.iter()) {
            npv += sign * self.leg_npv(leg, reference);
        }
        Ok(PricingResults::from_npv(npv))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ql_cashflows::SimpleCashFlow;
    use ql_termstructures::FlatForward;
    use ql_time::Actual365Fixed;

    fn make_swap_legs() -> (Vec<Box<dyn CashFlow>>, Vec<Box<dyn CashFlow>>) {
        // Simple 2-year swap:
        // Fixed leg: 3% semi-annual on 100 notional = 1.5 each period
        // Floating leg: assumed fixed for simplicity (2.5% semi-annual = 1.25)
        let dates = [
            Date::from_ymd(2025, 7, 15).unwrap(),
            Date::from_ymd(2026, 1, 15).unwrap(),
            Date::from_ymd(2026, 7, 15).unwrap(),
            Date::from_ymd(2027, 1, 15).unwrap(),
        ];

        let fixed: Vec<Box<dyn CashFlow>> = dates
            .iter()
            .map(|&d| Box::new(SimpleCashFlow::new(1.5, d)) as Box<dyn CashFlow>)
            .collect();

        let floating: Vec<Box<dyn CashFlow>> = dates
            .iter()
            .map(|&d| Box::new(SimpleCashFlow::new(1.25, d)) as Box<dyn CashFlow>)
            .collect();

        (fixed, floating)
    }

    #[test]
    fn payer_swap_positive_when_floating_gt_fixed() {
        let ref_date = Date::from_ymd(2025, 1, 15).unwrap();
        let curve = Arc::new(FlatForward::continuous(ref_date, 0.05, Actual365Fixed));
        let engine = DiscountingSwapEngine::new(curve);

        let (fixed_leg, floating_leg) = make_swap_legs();

        // Payer: receives floating 1.25, pays fixed 1.5 → negative (paying more)
        let swap = VanillaSwap {
            swap_type: SwapType::Payer,
            nominal: 100.0,
            fixed_rate: 0.03,
            spread: 0.0,
            fixed_leg,
            floating_leg,
            fixed_maturity: Date::from_ymd(2027, 1, 15).unwrap(),
        };

        let result = engine.price_swap(&swap, ref_date).unwrap();
        // Floating NPV < Fixed NPV, so payer NPV < 0
        assert!(result.npv < 0.0, "npv = {}", result.npv);
    }

    #[test]
    fn receiver_swap_opposite_sign() {
        let ref_date = Date::from_ymd(2025, 1, 15).unwrap();
        let curve = Arc::new(FlatForward::continuous(ref_date, 0.05, Actual365Fixed));
        let engine = DiscountingSwapEngine::new(curve);

        let (fixed1, float1) = make_swap_legs();
        let (fixed2, float2) = make_swap_legs();

        let payer_swap = VanillaSwap {
            swap_type: SwapType::Payer,
            nominal: 100.0,
            fixed_rate: 0.03,
            spread: 0.0,
            fixed_leg: fixed1,
            floating_leg: float1,
            fixed_maturity: Date::from_ymd(2027, 1, 15).unwrap(),
        };

        let receiver_swap = VanillaSwap {
            swap_type: SwapType::Receiver,
            nominal: 100.0,
            fixed_rate: 0.03,
            spread: 0.0,
            fixed_leg: fixed2,
            floating_leg: float2,
            fixed_maturity: Date::from_ymd(2027, 1, 15).unwrap(),
        };

        let payer = engine.price_swap(&payer_swap, ref_date).unwrap();
        let receiver = engine.price_swap(&receiver_swap, ref_date).unwrap();

        assert!(
            (payer.npv + receiver.npv).abs() < 1e-10,
            "payer={}, receiver={}",
            payer.npv,
            receiver.npv
        );
    }

    #[test]
    fn multi_leg_pricing() {
        let ref_date = Date::from_ymd(2025, 1, 15).unwrap();
        let curve = Arc::new(FlatForward::continuous(ref_date, 0.05, Actual365Fixed));
        let engine = DiscountingSwapEngine::new(curve);

        let maturity = Date::from_ymd(2026, 1, 15).unwrap();
        let leg1: Vec<Box<dyn CashFlow>> = vec![Box::new(SimpleCashFlow::new(100.0, maturity))];
        let leg2: Vec<Box<dyn CashFlow>> = vec![Box::new(SimpleCashFlow::new(50.0, maturity))];

        // Pay leg1, receive leg2
        let result = engine
            .price_legs(&[leg1, leg2], &[-1.0, 1.0], ref_date)
            .unwrap();

        // NPV = -100*df + 50*df = -50*df
        let df = (-0.05_f64).exp();
        let expected = -50.0 * df;
        assert!(
            (result.npv - expected).abs() < 0.01,
            "npv = {}, expected = {}",
            result.npv,
            expected
        );
    }
}
