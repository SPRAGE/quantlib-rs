//! Vanilla and European option instruments.
//!
//! Translates `ql/instruments/vanillaoption.hpp`,
//! `ql/instruments/oneassetoption.hpp`.

use crate::exercise::Exercise;
use crate::instrument::{Instrument, PricingEngine, PricingResults};
use crate::payoff::{OptionType, PlainVanillaPayoff, StrikedPayoff};
use ql_core::{errors::Result, Real};
use ql_time::Date;
use std::sync::Arc;

// ────────────────────────────────────────────────────────────────────────────
// Option arguments (sent to pricing engines)
// ────────────────────────────────────────────────────────────────────────────

/// Arguments needed for pricing a one-asset option.
///
/// Corresponds to `QuantLib::OneAssetOption::arguments`.
#[derive(Debug, Clone)]
pub struct VanillaOptionArguments {
    /// The payoff.
    pub payoff: Arc<dyn StrikedPayoff>,
    /// The exercise specification.
    pub exercise: Exercise,
}

// ────────────────────────────────────────────────────────────────────────────
// VanillaOption
// ────────────────────────────────────────────────────────────────────────────

/// A plain vanilla option on a single underlying asset.
///
/// Corresponds to `QuantLib::VanillaOption` / `QuantLib::EuropeanOption`.
#[derive(Debug)]
pub struct VanillaOption {
    /// The payoff function.
    payoff: Arc<dyn StrikedPayoff>,
    /// The exercise specification.
    exercise: Exercise,
}

impl VanillaOption {
    /// Create a new vanilla option.
    pub fn new(payoff: Arc<dyn StrikedPayoff>, exercise: Exercise) -> Self {
        Self { payoff, exercise }
    }

    /// Convenience: create a European call/put.
    pub fn european(option_type: OptionType, strike: Real, expiry: Date) -> Self {
        Self {
            payoff: Arc::new(PlainVanillaPayoff::new(option_type, strike)),
            exercise: Exercise::european(expiry),
        }
    }

    /// The strike price.
    pub fn strike(&self) -> Real {
        self.payoff.strike()
    }

    /// The option type (call/put).
    pub fn option_type(&self) -> OptionType {
        self.payoff.option_type()
    }

    /// The payoff.
    pub fn payoff(&self) -> &dyn StrikedPayoff {
        &*self.payoff
    }

    /// The exercise.
    pub fn exercise(&self) -> &Exercise {
        &self.exercise
    }

    /// Get the arguments for a pricing engine.
    pub fn arguments(&self) -> VanillaOptionArguments {
        VanillaOptionArguments {
            payoff: Arc::clone(&self.payoff),
            exercise: self.exercise.clone(),
        }
    }

    /// Price this option using the given engine.
    pub fn price(
        &self,
        engine: &dyn PricingEngine<VanillaOptionArguments>,
    ) -> Result<PricingResults> {
        engine.calculate(&self.arguments())
    }
}

impl Instrument for VanillaOption {
    fn is_expired(&self) -> bool {
        false // would need Settings::evaluation_date()
    }

    fn maturity_date(&self) -> Option<Date> {
        Some(self.exercise.last_date())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// BarrierOption types (just the definitions, engines come later)
// ────────────────────────────────────────────────────────────────────────────

/// Barrier type.
///
/// Corresponds to `QuantLib::Barrier::Type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BarrierType {
    /// Down-and-in: becomes active when price drops below barrier.
    DownIn,
    /// Up-and-in: becomes active when price rises above barrier.
    UpIn,
    /// Down-and-out: expires when price drops below barrier.
    DownOut,
    /// Up-and-out: expires when price rises above barrier.
    UpOut,
}

/// Arguments for a barrier option.
#[derive(Debug, Clone)]
pub struct BarrierOptionArguments {
    /// The payoff.
    pub payoff: Arc<dyn StrikedPayoff>,
    /// The exercise specification.
    pub exercise: Exercise,
    /// Barrier type.
    pub barrier_type: BarrierType,
    /// Barrier level.
    pub barrier: Real,
    /// Cash rebate paid when barrier is hit (for out) or at expiry (for in).
    pub rebate: Real,
}

/// A barrier option.
///
/// Corresponds to `QuantLib::BarrierOption`.
#[derive(Debug)]
pub struct BarrierOption {
    payoff: Arc<dyn StrikedPayoff>,
    exercise: Exercise,
    /// Barrier type.
    pub barrier_type: BarrierType,
    /// Barrier level.
    pub barrier: Real,
    /// Rebate.
    pub rebate: Real,
}

impl BarrierOption {
    /// Create a new barrier option.
    pub fn new(
        payoff: Arc<dyn StrikedPayoff>,
        exercise: Exercise,
        barrier_type: BarrierType,
        barrier: Real,
        rebate: Real,
    ) -> Self {
        Self {
            payoff,
            exercise,
            barrier_type,
            barrier,
            rebate,
        }
    }

    /// The strike.
    pub fn strike(&self) -> Real {
        self.payoff.strike()
    }

    /// The option type.
    pub fn option_type(&self) -> OptionType {
        self.payoff.option_type()
    }

    /// The exercise.
    pub fn exercise(&self) -> &Exercise {
        &self.exercise
    }

    /// Get engine arguments.
    pub fn arguments(&self) -> BarrierOptionArguments {
        BarrierOptionArguments {
            payoff: Arc::clone(&self.payoff),
            exercise: self.exercise.clone(),
            barrier_type: self.barrier_type,
            barrier: self.barrier,
            rebate: self.rebate,
        }
    }
}

impl Instrument for BarrierOption {
    fn is_expired(&self) -> bool {
        false
    }

    fn maturity_date(&self) -> Option<Date> {
        Some(self.exercise.last_date())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exercise::ExerciseType;

    #[test]
    fn european_call_construction() {
        let expiry = Date::from_ymd(2026, 6, 15).unwrap();
        let opt = VanillaOption::european(OptionType::Call, 100.0, expiry);
        assert_eq!(opt.strike(), 100.0);
        assert_eq!(opt.option_type(), OptionType::Call);
        assert_eq!(opt.exercise().exercise_type(), ExerciseType::European);
        assert_eq!(opt.exercise().last_date(), expiry);
    }

    #[test]
    fn european_put_construction() {
        let expiry = Date::from_ymd(2026, 6, 15).unwrap();
        let opt = VanillaOption::european(OptionType::Put, 100.0, expiry);
        assert_eq!(opt.option_type(), OptionType::Put);
    }

    #[test]
    fn barrier_option_construction() {
        let expiry = Date::from_ymd(2026, 6, 15).unwrap();
        let payoff = Arc::new(PlainVanillaPayoff::new(OptionType::Call, 100.0));
        let exercise = Exercise::european(expiry);
        let opt = BarrierOption::new(payoff, exercise, BarrierType::DownOut, 80.0, 0.0);
        assert_eq!(opt.barrier_type, BarrierType::DownOut);
        assert!((opt.barrier - 80.0).abs() < 1e-15);
        assert_eq!(opt.strike(), 100.0);
    }

    #[test]
    fn vanilla_option_arguments() {
        let expiry = Date::from_ymd(2026, 6, 15).unwrap();
        let opt = VanillaOption::european(OptionType::Call, 100.0, expiry);
        let args = opt.arguments();
        assert!((args.payoff.strike() - 100.0).abs() < 1e-15);
        assert_eq!(args.exercise.exercise_type(), ExerciseType::European);
    }
}
