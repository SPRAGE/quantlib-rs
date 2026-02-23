//! # ql-currencies
//!
//! Currency and exchange-rate definitions.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

/// Currency data and definitions.
pub mod currency;

/// Exchange rate and money types.
pub mod exchange_rate;

/// Pre-defined world currencies.
pub mod currencies;

pub use currency::{Currency, Money};
pub use exchange_rate::{ExchangeRate, ExchangeRateManager};
