//! # ql-experimental
//!
//! Experimental and unstable extensions (credit derivatives, exotic options,
//! variance swaps, etc.).

#![warn(missing_docs)]
#![forbid(unsafe_code)]

/// Variance Gamma model and pricing engines.
pub mod variancegamma;

/// Catastrophe bond framework.
pub mod catbonds;

/// Variance option (realized variance derivative).
pub mod variance_option;

pub use variancegamma::{VarianceGammaEngine, VarianceGammaModel};

pub use catbonds::{
    BetaRisk, CatRisk, CatSimulation, DigitalNotionalRisk, EventSet, NotionalPath, NotionalRisk,
    ProportionalNotionalRisk,
};

pub use variance_option::IntegralHestonVarianceOptionEngine;
