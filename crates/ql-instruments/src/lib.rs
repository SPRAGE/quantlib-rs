//! # ql-instruments
//!
//! Financial instruments: bonds, swaps, options, caps/floors, swaptions, etc.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

pub mod bond;
pub mod exercise;
pub mod instrument;
pub mod option;
pub mod payoff;
pub mod swap;

pub use bond::{fixed_rate_bond, floating_rate_bond, zero_coupon_bond, Bond, BondArguments};
pub use exercise::{Exercise, ExerciseType};
pub use instrument::{Instrument, PricingEngine, PricingResults};
pub use option::{
    BarrierOption, BarrierOptionArguments, BarrierType, VanillaOption, VanillaOptionArguments,
};
pub use payoff::{
    AssetOrNothingPayoff, CashOrNothingPayoff, GapPayoff, OptionType, Payoff, PlainVanillaPayoff,
    StrikedPayoff,
};
pub use swap::{Swap, SwapArguments, SwapType, VanillaSwap};
