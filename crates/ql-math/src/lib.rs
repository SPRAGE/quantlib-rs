//! # ql-math
//!
//! Mathematical utilities: interpolation, solvers, optimisation,
//! matrix/array newtypes (over nalgebra), distributions (via statrs),
//! and random number generation.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

// ── Modules ───────────────────────────────────────────────────────────────────

/// Dynamically-sized 1D vector of reals.
pub mod array;

/// Floating-point comparison utilities.
pub mod comparison;

/// Copula functions.
pub mod copulas;

/// Probability distributions.
pub mod distributions;

/// Numerical integration.
pub mod integrals;

/// 1D interpolation schemes.
pub mod interpolations;

/// Dynamically-sized 2D matrix of reals.
pub mod matrix;

/// Matrix decomposition and utility functions.
pub mod matrix_utilities;

/// Ordinary differential equation solvers.
pub mod ode;

/// Optimization framework.
pub mod optimization;

/// Random number generators.
pub mod random_numbers;

/// Rounding conventions.
pub mod rounding;

/// 1D root-finding solvers.
pub mod solvers1d;

/// Statistics accumulators.
pub mod statistics;

// ── Convenience re-exports ────────────────────────────────────────────────────

pub use array::Array;
pub use comparison::{close, close_enough};
pub use distributions::{
    normal_cdf, normal_cdf_inverse, normal_pdf,
    BetaDistribution, BinomialDistribution, ChiSquareDistribution, GammaDistribution,
    PoissonDistribution, StudentTDistribution,
};
pub use interpolations::{
    akima::AkimaSpline,
    monotone_cubic::MonotoneCubicSpline,
    CubicNaturalSpline, FlatInterpolation, ForwardFlatInterpolation,
    Interpolation1D, LagrangeInterpolation, LinearInterpolation,
    LogLinearInterpolation,
};
pub use matrix::Matrix;
pub use rounding::{round, Rounding};
pub use statistics::{
    ConvergenceStatistics, GeneralStatistics, IncrementalStatistics, SequenceStatistics, Statistics,
};
