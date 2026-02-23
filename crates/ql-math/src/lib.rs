//! # ql-math
//!
//! Mathematical utilities: interpolation, solvers, optimisation,
//! matrix/array newtypes (over nalgebra), distributions (via statrs),
//! and random number generation.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

// ── Modules ───────────────────────────────────────────────────────────────────

/// Floating-point comparison utilities.
pub mod comparison;

/// Probability distributions.
pub mod distributions;

/// 1D interpolation schemes.
pub mod interpolations;

/// Random number generators.
pub mod random_numbers;

/// Rounding conventions.
pub mod rounding;

/// 1D root-finding solvers.
pub mod solvers1d;

/// Statistics accumulators.
pub mod statistics;

// ── Convenience re-exports ────────────────────────────────────────────────────

pub use comparison::{close, close_enough};
pub use distributions::{normal_cdf, normal_cdf_inverse, normal_pdf};
pub use rounding::{round, Rounding};
