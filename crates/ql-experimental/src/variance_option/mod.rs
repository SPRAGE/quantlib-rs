//! Variance option framework.
//!
//! Translates `ql/experimental/varianceoption/` —
//! variance options priced via integral Heston engine.

pub mod engine;

pub use engine::IntegralHestonVarianceOptionEngine;
