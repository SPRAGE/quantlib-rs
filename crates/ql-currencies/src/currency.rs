//! `Currency` — definition and metadata for a financial currency.
//!
//! Translates `ql/currency.hpp`.

use ql_core::{Integer, Real};

/// Data describing a single currency.
///
/// Corresponds to `QuantLib::Currency`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Currency {
    /// Full name (e.g. "United States Dollar").
    pub name: &'static str,
    /// ISO 4217 alphabetic code (e.g. "USD").
    pub code: &'static str,
    /// ISO 4217 numeric code (e.g. 840).
    pub numeric_code: u16,
    /// Symbol used in financial notation (e.g. "$").
    pub symbol: &'static str,
    /// Fraction symbol (e.g. "¢").
    pub fraction_symbol: &'static str,
    /// Number of fractional units per whole unit (e.g. 100 for cents).
    pub fractions_per_unit: Integer,
    /// Rounding precision (decimal places for display).
    pub rounding: u8,
}

impl Currency {
    /// Return `true` if this is the null / empty currency sentinel.
    pub fn is_empty(&self) -> bool {
        self.code.is_empty()
    }
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code)
    }
}

/// A monetary amount with an associated currency.
#[derive(Debug, Clone, PartialEq)]
pub struct Money {
    /// Numeric value.
    pub value: Real,
    /// The currency.
    pub currency: &'static Currency,
}

impl Money {
    /// Create a new monetary amount.
    pub fn new(value: Real, currency: &'static Currency) -> Self {
        Self { value, currency }
    }
}

impl std::fmt::Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2} {}", self.value, self.currency.code)
    }
}
