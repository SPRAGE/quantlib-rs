//! # ql-indexes
//!
//! Interest-rate, inflation, equity, and FX index definitions.
//!
//! Translates `ql/indexes/` — the `Index`, `InterestRateIndex`, `IborIndex`,
//! and `OvernightIndex` class hierarchy.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

// ── Modules ───────────────────────────────────────────────────────────────────

/// `Index` trait — base trait for all market indexes.
pub mod index;

/// `InterestRateIndex` — base for interest-rate indexes.
pub mod interest_rate_index;

/// `IborIndex` — interbank offered-rate indexes (Euribor, LIBOR, etc.).
pub mod ibor_index;

/// `OvernightIndex` — overnight rate indexes (SOFR, ESTR, SONIA, etc.).
pub mod overnight_index;

/// Concrete IBOR index definitions (Euribor, USD LIBOR, etc.).
pub mod ibor;

/// Concrete overnight index definitions (SOFR, ESTR, SONIA, etc.).
pub mod overnight;

/// Inflation index stub types.
pub mod inflation;

/// Swap index stub types.
pub mod swap_index;

// ── Convenience re-exports ────────────────────────────────────────────────────

pub use ibor::{euribor, gbp_libor, jpy_libor, usd_libor};
pub use ibor_index::IborIndex;
pub use index::Index;
pub use interest_rate_index::InterestRateIndex;
pub use overnight::{estr, sofr, sonia};
pub use overnight_index::OvernightIndex;
pub use swap_index::SwapIndex;
