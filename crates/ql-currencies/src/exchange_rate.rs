//! Exchange rate and money types.

use crate::currency::Currency;
use ql_core::{errors::Result, Real};

/// An exchange rate between two currencies.
///
/// Corresponds to `QuantLib::ExchangeRate`.
#[derive(Debug, Clone)]
pub struct ExchangeRate {
    /// The source currency.
    pub source: &'static Currency,
    /// The target currency.
    pub target: &'static Currency,
    /// Rate: how many units of `target` one unit of `source` buys.
    pub rate: Real,
}

impl ExchangeRate {
    /// Create a new exchange rate.
    pub fn new(source: &'static Currency, target: &'static Currency, rate: Real) -> Self {
        Self {
            source,
            target,
            rate,
        }
    }

    /// Convert a monetary amount from `source` to `target` currency.
    pub fn exchange(&self, amount: Real) -> Result<Real> {
        Ok(amount * self.rate)
    }

    /// Return the inverse rate (target → source).
    pub fn inverse(&self) -> Self {
        Self {
            source: self.target,
            target: self.source,
            rate: 1.0 / self.rate,
        }
    }
}

/// Re-export `Money` here for convenience.
pub use crate::currency::Money;
