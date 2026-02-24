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

/// Analytical exotic option pricing engines.
pub mod exoticoptions;

/// ZABR (Zeta Alpha Beta Rho) model — generalised SABR with gamma parameter.
pub mod zabr;

/// No-arbitrage SABR model — density-based approach to avoid butterfly arbitrage.
pub mod noarb_sabr;

pub use variancegamma::{VarianceGammaEngine, VarianceGammaModel};

pub use catbonds::{
    BetaRisk, CatRisk, CatSimulation, DigitalNotionalRisk, EventSet, NotionalPath, NotionalRisk,
    ProportionalNotionalRisk,
};

pub use variance_option::IntegralHestonVarianceOptionEngine;

pub use exoticoptions::{
    AnalyticCompoundOptionEngine, AnalyticComplexChooserEngine,
    AnalyticHolderExtensibleOptionEngine, AnalyticSimpleChooserEngine,
    AnalyticTwoAssetCorrelationEngine, AnalyticWriterExtensibleOptionEngine,
};

pub use zabr::{ZabrEvaluationMethod, ZabrModel, ZabrParameters, ZabrSmileSection};

pub use noarb_sabr::{NoArbSabrModel, NoArbSabrParameters, NoArbSabrSmileSection};
