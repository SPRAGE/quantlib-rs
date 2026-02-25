//! Variance Gamma model and analytic pricing engine.
//!
//! Translates `ql/experimental/variancegamma/analyticvariancegammaengine.hpp`.
//!
//! The Variance Gamma engine prices European vanilla options by integrating a
//! Black-Scholes price weighted by a Gamma probability density over the
//! subordinated time variable.
//!
//! # References
//!
//! D. Madan, P. Carr, E. Chang (1998), "The Variance Gamma Process and Option
//! Pricing", *European Finance Review* 2, 79–105.

pub mod engine;
pub mod model;

pub use engine::VarianceGammaEngine;
pub use model::VarianceGammaModel;
