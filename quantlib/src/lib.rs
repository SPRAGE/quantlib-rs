//! # quantlib
//!
//! A complete Rust translation of the [QuantLib](https://www.quantlib.org/)
//! quantitative finance library.
//!
//! This crate is a **façade** that re-exports all public items from the
//! underlying workspace crates. Application code should depend on this
//! crate rather than the individual `ql-*` crates.
//!
//! ## Quick start
//!
//! ```toml
//! [dependencies]
//! quantlib = "0.1"
//! ```
//!
//! ```rust
//! use quantlib::core::Real;
//!
//! let rate: Real = 0.05;
//! assert!((rate - 0.05).abs() < f64::EPSILON);
//! ```

#![warn(missing_docs)]
#![forbid(unsafe_code)]

/// Core types, aliases, and error definitions.
pub use ql_core as core;

/// Date, calendar, day counter, and schedule types.
pub use ql_time as time;

/// Mathematical utilities: interpolation, optimisation, RNG.
pub use ql_math as math;

/// Currency definitions.
pub use ql_currencies as currencies;

/// Market quotes.
pub use ql_quotes as quotes;

/// Market index definitions.
pub use ql_indexes as indexes;

/// Term structure implementations.
pub use ql_termstructures as termstructures;

/// Stochastic process definitions.
pub use ql_processes as processes;

/// Calibratable financial models.
pub use ql_models as models;

/// Numerical methods (lattices, FDM, Monte Carlo).
pub use ql_methods as methods;

/// Cash flows and coupons.
pub use ql_cashflows as cashflows;

/// Financial instruments.
pub use ql_instruments as instruments;

/// Pricing engines.
pub use ql_pricingengines as pricingengines;

/// Experimental / unstable modules.
pub use ql_experimental as experimental;

/// Legacy / deprecated modules.
pub use ql_legacy as legacy;
