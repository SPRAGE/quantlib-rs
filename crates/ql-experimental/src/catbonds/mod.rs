//! Catastrophe bond framework.
//!
//! Translates `ql/experimental/catbonds/` — catastrophe risk simulation,
//! notional risk models, and the data structures needed for catastrophe bond pricing.

pub mod cat_risk;
pub mod notional_risk;

pub use cat_risk::{BetaRisk, CatRisk, CatSimulation, EventSet};
pub use notional_risk::{
    DigitalNotionalRisk, NotionalPath, NotionalRisk, ProportionalNotionalRisk,
};
