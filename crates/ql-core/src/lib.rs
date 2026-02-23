//! # ql-core
//!
//! Core types, traits, and error definitions for quantlib-rs.
//!
//! This crate provides the foundational building blocks shared across all
//! other crates in the workspace – type aliases, the error hierarchy, the
//! Observer/Observable pattern, the `Handle` wrapper, `LazyObject`, and
//! `Settings`.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

// ── Public modules ───────────────────────────────────────────────────────────

/// Compounding conventions.
pub mod compounding;

/// Error types and the `ensure!` / `fail!` macros.
pub mod errors;

/// Shared reference handle (`Handle<T>`, `RelinkableHandle<T>`).
pub mod handle;

/// Design patterns: observable, lazy_object, visitor.
pub mod patterns;

/// Global library settings (evaluation date, etc.).
pub mod settings;

/// Miscellaneous utilities.
pub mod utilities;

// ── Primitive type aliases ────────────────────────────────────────────────────

/// Floating-point type used throughout the library.
pub type Real = f64;

/// Integer type used for general-purpose counting.
pub type Integer = i32;

/// Non-negative integer type.
pub type Natural = u32;

/// Alias used for array sizes / indices.
pub type Size = usize;

/// A rate expressed as a decimal (e.g. 0.05 = 5 %).
pub type Rate = Real;

/// A spread over a reference rate.
pub type Spread = Real;

/// A discount factor in [0, 1].
pub type DiscountFactor = Real;

/// A price or value.
pub type Price = Real;

/// A volatility level expressed as a decimal.
pub type Volatility = Real;

/// A time measurement in years.
pub type Time = Real;

// ── Re-exports for convenience ────────────────────────────────────────────────

pub use compounding::Compounding;
pub use errors::{Error, Result};
pub use handle::{Handle, RelinkableHandle};
pub use settings::Settings;
