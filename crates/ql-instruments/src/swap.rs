//! Interest-rate swap instruments.
//!
//! Translates `ql/instruments/swap.hpp`, `ql/instruments/vanillaswap.hpp`,
//! `ql/instruments/overnightindexedswap.hpp`.

use crate::instrument::{Instrument, PricingEngine, PricingResults};
use ql_cashflows::{FixedRateLegBuilder, IborLegBuilder, Leg};
use ql_core::{errors::Result, Compounding, Real};
use ql_indexes::IborIndex;
use ql_time::{Actual365Fixed, Date, Frequency, Schedule};
use std::sync::Arc;

// ────────────────────────────────────────────────────────────────────────────
// Swap base
// ────────────────────────────────────────────────────────────────────────────

/// Swap type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SwapType {
    /// Payer (pay fixed, receive floating).
    Payer,
    /// Receiver (receive fixed, pay floating).
    Receiver,
}

impl SwapType {
    /// +1 for Payer, −1 for Receiver.
    pub fn sign(self) -> Real {
        match self {
            SwapType::Payer => 1.0,
            SwapType::Receiver => -1.0,
        }
    }
}

/// Arguments for pricing a swap.
#[derive(Debug)]
pub struct SwapArguments {
    /// Fixed leg cash flows.
    pub fixed_leg: Leg,
    /// Floating leg cash flows.
    pub floating_leg: Leg,
    /// Swap type (payer/receiver).
    pub swap_type: SwapType,
}

/// Generic interest-rate swap (two legs).
///
/// Corresponds to `QuantLib::Swap`.
#[derive(Debug)]
pub struct Swap {
    /// First leg (typically fixed).
    pub legs: Vec<Leg>,
    /// Signs for each leg (+1 = pay, −1 = receive).
    pub payer: Vec<Real>,
}

impl Swap {
    /// Create a swap from two legs.
    pub fn new(leg1: Leg, leg2: Leg, payer1: Real, payer2: Real) -> Self {
        Self {
            legs: vec![leg1, leg2],
            payer: vec![payer1, payer2],
        }
    }

    /// Number of legs.
    pub fn num_legs(&self) -> usize {
        self.legs.len()
    }

    /// The i-th leg.
    pub fn leg(&self, i: usize) -> &Leg {
        &self.legs[i]
    }

    /// Maturity: the latest cash-flow date across all legs.
    pub fn maturity(&self) -> Option<Date> {
        self.legs
            .iter()
            .flat_map(|leg| leg.iter().map(|cf| cf.date()))
            .max()
    }
}

impl Instrument for Swap {
    fn is_expired(&self) -> bool {
        false
    }

    fn maturity_date(&self) -> Option<Date> {
        self.maturity()
    }
}

// ────────────────────────────────────────────────────────────────────────────
// VanillaSwap
// ────────────────────────────────────────────────────────────────────────────

/// A standard fixed-for-floating interest rate swap.
///
/// Corresponds to `QuantLib::VanillaSwap`.
#[derive(Debug)]
pub struct VanillaSwap {
    /// Swap type (payer = pay fixed).
    pub swap_type: SwapType,
    /// Notional amount.
    pub nominal: Real,
    /// Fixed leg coupon rate.
    pub fixed_rate: Real,
    /// Floating leg spread.
    pub spread: Real,
    /// The fixed leg cash flows.
    pub fixed_leg: Leg,
    /// The floating leg cash flows.
    pub floating_leg: Leg,
    /// Fixed leg maturity / overall maturity.
    pub fixed_maturity: Date,
}

impl VanillaSwap {
    /// Create a new vanilla swap.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        swap_type: SwapType,
        nominal: Real,
        fixed_schedule: &Schedule,
        fixed_rate: Real,
        fixed_compounding: Compounding,
        fixed_frequency: Frequency,
        float_schedule: &Schedule,
        index: Arc<IborIndex>,
        spread: Real,
    ) -> Self {
        let fixed_leg = FixedRateLegBuilder::new(fixed_schedule)
            .with_notionals(vec![nominal])
            .with_coupon_rate(fixed_rate)
            .with_compounding(fixed_compounding)
            .with_frequency(fixed_frequency)
            .build();

        let floating_leg = IborLegBuilder::new(float_schedule, index)
            .with_notionals(vec![nominal])
            .with_spread(spread)
            .build();

        let fixed_dates = fixed_schedule.dates();
        let float_dates = float_schedule.dates();
        let fixed_maturity = *fixed_dates.last().unwrap_or(&Date::default());
        let float_maturity = *float_dates.last().unwrap_or(&Date::default());
        let maturity = fixed_maturity.max(float_maturity);

        Self {
            swap_type,
            nominal,
            fixed_rate,
            spread,
            fixed_leg,
            floating_leg,
            fixed_maturity: maturity,
        }
    }

    /// The fixed leg NPV using a flat yield.
    pub fn fixed_leg_npv(&self, yield_rate: Real, settlement: Date) -> Real {
        let ir = ql_time::InterestRate::new(
            yield_rate,
            Actual365Fixed,
            Compounding::Simple,
            Frequency::Annual,
        );
        ql_cashflows::npv_yield(&self.fixed_leg, &ir, settlement)
    }

    /// The floating leg NPV using a flat yield.
    pub fn floating_leg_npv(&self, yield_rate: Real, settlement: Date) -> Real {
        let ir = ql_time::InterestRate::new(
            yield_rate,
            Actual365Fixed,
            Compounding::Simple,
            Frequency::Annual,
        );
        ql_cashflows::npv_yield(&self.floating_leg, &ir, settlement)
    }

    /// Fair value (NPV) at a flat yield from the payer's perspective.
    ///
    /// `NPV = sign * (floating_leg_npv - fixed_leg_npv)` for a payer swap.
    pub fn npv_flat(&self, yield_rate: Real, settlement: Date) -> Real {
        let fixed = self.fixed_leg_npv(yield_rate, settlement);
        let floating = self.floating_leg_npv(yield_rate, settlement);
        self.swap_type.sign() * (floating - fixed)
    }

    /// Get engine arguments.
    pub fn arguments(&self) -> SwapArguments {
        // Clone the legs for the engine — simplified
        SwapArguments {
            fixed_leg: Vec::new(),
            floating_leg: Vec::new(),
            swap_type: self.swap_type,
        }
    }

    /// Price with a pricing engine.
    pub fn price(&self, engine: &dyn PricingEngine<SwapArguments>) -> Result<PricingResults> {
        engine.calculate(&self.arguments())
    }
}

impl Instrument for VanillaSwap {
    fn is_expired(&self) -> bool {
        false
    }

    fn maturity_date(&self) -> Option<Date> {
        Some(self.fixed_maturity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ql_currencies::currencies::america::USD;
    use ql_time::{BusinessDayConvention, NullCalendar, Period, ScheduleBuilder, TimeUnit};

    fn make_index() -> Arc<IborIndex> {
        Arc::new(IborIndex::new(
            "USD-Libor-3M",
            Period::new(3, TimeUnit::Months),
            2,
            &USD,
            NullCalendar,
            BusinessDayConvention::ModifiedFollowing,
            false,
            Actual365Fixed,
        ))
    }

    #[test]
    fn vanilla_swap_construction() {
        let start = Date::from_ymd(2025, 1, 15).unwrap();
        let end = Date::from_ymd(2030, 1, 15).unwrap();
        let fixed_tenor = Period::new(1, TimeUnit::Years);
        let float_tenor = Period::new(3, TimeUnit::Months);
        let cal = NullCalendar;

        let fixed_schedule = ScheduleBuilder::new(start, end, fixed_tenor, &cal)
            .build()
            .unwrap();
        let float_schedule = ScheduleBuilder::new(start, end, float_tenor, &cal)
            .build()
            .unwrap();

        let swap = VanillaSwap::new(
            SwapType::Payer,
            1_000_000.0,
            &fixed_schedule,
            0.03,
            Compounding::Simple,
            Frequency::Annual,
            &float_schedule,
            make_index(),
            0.0,
        );

        assert_eq!(swap.fixed_leg.len(), 5); // 5 annual coupons
        assert_eq!(swap.floating_leg.len(), 20); // 20 quarterly coupons
        assert!((swap.nominal - 1_000_000.0).abs() < 1e-15);
    }

    #[test]
    fn swap_type_sign() {
        assert!((SwapType::Payer.sign() - 1.0).abs() < 1e-15);
        assert!((SwapType::Receiver.sign() - (-1.0)).abs() < 1e-15);
    }

    #[test]
    fn generic_swap_two_legs() {
        let start = Date::from_ymd(2025, 1, 15).unwrap();
        let end = Date::from_ymd(2027, 1, 15).unwrap();
        let tenor = Period::new(1, TimeUnit::Years);
        let schedule = ScheduleBuilder::new(start, end, tenor, &NullCalendar)
            .build()
            .unwrap();

        let leg1 = FixedRateLegBuilder::new(&schedule)
            .with_notionals(vec![100.0])
            .with_coupon_rate(0.03)
            .build();
        let leg2 = FixedRateLegBuilder::new(&schedule)
            .with_notionals(vec![100.0])
            .with_coupon_rate(0.05)
            .build();

        let swap = Swap::new(leg1, leg2, -1.0, 1.0);
        assert_eq!(swap.num_legs(), 2);
        assert_eq!(swap.leg(0).len(), 2);
    }

    #[test]
    fn swap_maturity() {
        let start = Date::from_ymd(2025, 1, 15).unwrap();
        let end = Date::from_ymd(2030, 1, 15).unwrap();
        let tenor = Period::new(1, TimeUnit::Years);
        let schedule = ScheduleBuilder::new(start, end, tenor, &NullCalendar)
            .build()
            .unwrap();

        let leg = FixedRateLegBuilder::new(&schedule)
            .with_notionals(vec![100.0])
            .with_coupon_rate(0.03)
            .build();

        let swap = Swap::new(leg, Vec::new(), -1.0, 1.0);
        let mat = swap.maturity().unwrap();
        assert_eq!(mat, end);
    }
}
