//! # ql-methods
//!
//! Numerical methods: lattice/tree builders, finite-difference grids,
//! and Monte Carlo simulation framework.
//!
//! Translates `ql/methods/` — lattice methods (binomial & trinomial trees),
//! Monte Carlo simulation, and finite difference PDE solvers.
//!
//! # Modules
//!
//! * [`lattice`] — binomial/trinomial trees and backward-induction pricing
//! * [`monte_carlo`] — path generation, path pricing, MC model orchestrator
//! * [`finite_differences`] — tridiagonal solver and 1-D BS PDE solver

#![warn(missing_docs)]
#![forbid(unsafe_code)]

// ── Modules ───────────────────────────────────────────────────────────────────

/// Lattice methods: binomial trees, trinomial trees, backward induction.
pub mod lattice;

/// Monte Carlo simulation: path generation, pricing, statistics.
pub mod monte_carlo;

/// Finite difference methods: tridiagonal solver, 1-D PDE solver.
pub mod finite_differences;

// ── Convenience re-exports ────────────────────────────────────────────────────

pub use finite_differences::{Fdm1dSolver, FdmScheme, TridiagonalOperator};
pub use lattice::{
    price_american, price_american_trinomial, price_european, price_european_trinomial,
    BinomialTree, TimeGrid, TrinomialTree,
};
pub use monte_carlo::{
    AntitheticPathGenerator, EuropeanPathPricer, MonteCarloModel, Path, PathGenerator, PathPricer,
    mc_european_price,
};
