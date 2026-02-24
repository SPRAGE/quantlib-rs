//! Option payoff hierarchy.
//!
//! Translates `ql/instruments/payoffs.hpp` and `ql/option.hpp` (Option::Type).
//!
//! Payoffs describe the terminal (or exercise) payoff of an option as a
//! function of the underlying asset price.

use ql_core::Real;
use std::fmt;

/// Option type (call or put).
///
/// Corresponds to `QuantLib::Option::Type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OptionType {
    /// A call option (right to buy).
    Call,
    /// A put option (right to sell).
    Put,
}

impl OptionType {
    /// +1 for Call, −1 for Put.
    pub fn sign(self) -> Real {
        match self {
            OptionType::Call => 1.0,
            OptionType::Put => -1.0,
        }
    }
}

impl fmt::Display for OptionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OptionType::Call => write!(f, "Call"),
            OptionType::Put => write!(f, "Put"),
        }
    }
}

/// Base trait for option payoffs.
///
/// Corresponds to `QuantLib::Payoff`.
pub trait Payoff: fmt::Debug + Send + Sync {
    /// Compute the payoff given the underlying price at exercise/expiry.
    fn value(&self, price: Real) -> Real;

    /// Human-readable name.
    fn name(&self) -> &str;

    /// Human-readable description.
    fn description(&self) -> String {
        self.name().to_string()
    }
}

/// A payoff depending on a strike price.
///
/// Corresponds to `QuantLib::StrikedTypePayoff`.
pub trait StrikedPayoff: Payoff {
    /// The strike price.
    fn strike(&self) -> Real;

    /// The option type (call / put).
    fn option_type(&self) -> OptionType;
}

/// Standard "plain vanilla" European/American option payoff.
///
/// `payoff = max(φ(S − K), 0)` where `φ = +1` for Call, `−1` for Put.
///
/// Corresponds to `QuantLib::PlainVanillaPayoff`.
#[derive(Debug, Clone)]
pub struct PlainVanillaPayoff {
    /// Option type.
    pub option_type: OptionType,
    /// Strike price.
    pub strike: Real,
}

impl PlainVanillaPayoff {
    /// Create a new plain vanilla payoff.
    pub fn new(option_type: OptionType, strike: Real) -> Self {
        Self {
            option_type,
            strike,
        }
    }
}

impl Payoff for PlainVanillaPayoff {
    fn value(&self, price: Real) -> Real {
        (self.option_type.sign() * (price - self.strike)).max(0.0)
    }

    fn name(&self) -> &str {
        "Vanilla"
    }

    fn description(&self) -> String {
        format!("{} {} @ {}", self.name(), self.option_type, self.strike)
    }
}

impl StrikedPayoff for PlainVanillaPayoff {
    fn strike(&self) -> Real {
        self.strike
    }

    fn option_type(&self) -> OptionType {
        self.option_type
    }
}

/// Cash-or-nothing payoff: pays a fixed amount if in the money.
///
/// `payoff = cashPayoff` if `φ(S − K) > 0`, else 0.
///
/// Corresponds to `QuantLib::CashOrNothingPayoff`.
#[derive(Debug, Clone)]
pub struct CashOrNothingPayoff {
    /// Option type.
    pub option_type: OptionType,
    /// Strike price.
    pub strike: Real,
    /// Fixed cash payoff.
    pub cash_payoff: Real,
}

impl CashOrNothingPayoff {
    /// Create a new cash-or-nothing payoff.
    pub fn new(option_type: OptionType, strike: Real, cash_payoff: Real) -> Self {
        Self {
            option_type,
            strike,
            cash_payoff,
        }
    }
}

impl Payoff for CashOrNothingPayoff {
    fn value(&self, price: Real) -> Real {
        if self.option_type.sign() * (price - self.strike) > 0.0 {
            self.cash_payoff
        } else {
            0.0
        }
    }

    fn name(&self) -> &str {
        "CashOrNothing"
    }
}

impl StrikedPayoff for CashOrNothingPayoff {
    fn strike(&self) -> Real {
        self.strike
    }

    fn option_type(&self) -> OptionType {
        self.option_type
    }
}

/// Asset-or-nothing payoff: pays the underlying price if in the money.
///
/// `payoff = S` if `φ(S − K) > 0`, else 0.
///
/// Corresponds to `QuantLib::AssetOrNothingPayoff`.
#[derive(Debug, Clone)]
pub struct AssetOrNothingPayoff {
    /// Option type.
    pub option_type: OptionType,
    /// Strike price.
    pub strike: Real,
}

impl AssetOrNothingPayoff {
    /// Create a new asset-or-nothing payoff.
    pub fn new(option_type: OptionType, strike: Real) -> Self {
        Self {
            option_type,
            strike,
        }
    }
}

impl Payoff for AssetOrNothingPayoff {
    fn value(&self, price: Real) -> Real {
        if self.option_type.sign() * (price - self.strike) > 0.0 {
            price
        } else {
            0.0
        }
    }

    fn name(&self) -> &str {
        "AssetOrNothing"
    }
}

impl StrikedPayoff for AssetOrNothingPayoff {
    fn strike(&self) -> Real {
        self.strike
    }

    fn option_type(&self) -> OptionType {
        self.option_type
    }
}

/// Gap payoff: `max(φ(S − K₂), 0)` when triggered by `φ(S − K₁) > 0`.
///
/// Corresponds to `QuantLib::GapPayoff`.
#[derive(Debug, Clone)]
pub struct GapPayoff {
    /// Option type.
    pub option_type: OptionType,
    /// Trigger strike.
    pub strike: Real,
    /// Second strike (used in payoff calculation).
    pub second_strike: Real,
}

impl GapPayoff {
    /// Create a new gap payoff.
    pub fn new(option_type: OptionType, strike: Real, second_strike: Real) -> Self {
        Self {
            option_type,
            strike,
            second_strike,
        }
    }
}

impl Payoff for GapPayoff {
    fn value(&self, price: Real) -> Real {
        if self.option_type.sign() * (price - self.strike) >= 0.0 {
            self.option_type.sign() * (price - self.second_strike)
        } else {
            0.0
        }
    }

    fn name(&self) -> &str {
        "Gap"
    }
}

impl StrikedPayoff for GapPayoff {
    fn strike(&self) -> Real {
        self.strike
    }

    fn option_type(&self) -> OptionType {
        self.option_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_vanilla_call() {
        let p = PlainVanillaPayoff::new(OptionType::Call, 100.0);
        assert!((p.value(110.0) - 10.0).abs() < 1e-15);
        assert!((p.value(90.0) - 0.0).abs() < 1e-15);
        assert!((p.value(100.0) - 0.0).abs() < 1e-15);
    }

    #[test]
    fn plain_vanilla_put() {
        let p = PlainVanillaPayoff::new(OptionType::Put, 100.0);
        assert!((p.value(90.0) - 10.0).abs() < 1e-15);
        assert!((p.value(110.0) - 0.0).abs() < 1e-15);
    }

    #[test]
    fn cash_or_nothing_call() {
        let p = CashOrNothingPayoff::new(OptionType::Call, 100.0, 1.0);
        assert!((p.value(110.0) - 1.0).abs() < 1e-15);
        assert!((p.value(90.0) - 0.0).abs() < 1e-15);
    }

    #[test]
    fn asset_or_nothing_put() {
        let p = AssetOrNothingPayoff::new(OptionType::Put, 100.0);
        assert!((p.value(90.0) - 90.0).abs() < 1e-15);
        assert!((p.value(110.0) - 0.0).abs() < 1e-15);
    }

    #[test]
    fn gap_payoff() {
        let p = GapPayoff::new(OptionType::Call, 100.0, 95.0);
        // S = 110: trigger (110-100)>=0, payoff = 110 - 95 = 15
        assert!((p.value(110.0) - 15.0).abs() < 1e-15);
        // S = 90: trigger (90-100)<0, payoff = 0
        assert!((p.value(90.0) - 0.0).abs() < 1e-15);
    }
}
