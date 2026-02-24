//! # ql-pricingengines
//!
//! Pricing engines: analytic, lattice, finite-difference, and Monte Carlo
//! implementations for all instrument families.
//!
//! ## Engines
//!
//! - [`AnalyticEuropeanEngine`] — Black-Scholes-Merton closed-form for European options
//! - [`AnalyticHestonEngine`] — Semi-analytic Heston engine (Gauss-Laguerre integration)
//! - [`BaroneAdesiWhaleyEngine`] — Quadratic approximation for American options
//! - [`AnalyticBarrierEngine`] — Reiner-Rubinstein barrier option engine
//! - [`DiscountingBondEngine`] — Discounted cash flow engine for bonds
//! - [`DiscountingSwapEngine`] — Discounted cash flow engine for swaps

#![warn(missing_docs)]
#![forbid(unsafe_code)]

pub mod analytic_barrier_engine;
pub mod analytic_european_engine;
pub mod analytic_heston_engine;
pub mod barone_adesi_whaley_engine;
pub mod discounting_bond_engine;
pub mod discounting_swap_engine;

pub use analytic_barrier_engine::{analytic_barrier_price, AnalyticBarrierEngine};
pub use analytic_european_engine::{black_scholes_merton, AnalyticEuropeanEngine};
pub use analytic_heston_engine::{heston_price, AnalyticHestonEngine};
pub use barone_adesi_whaley_engine::{barone_adesi_whaley, BaroneAdesiWhaleyEngine};
pub use discounting_bond_engine::{clean_price, DiscountingBondEngine};
pub use discounting_swap_engine::DiscountingSwapEngine;
