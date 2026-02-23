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

impl std::ops::Add for Money {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        assert_eq!(
            self.currency, rhs.currency,
            "cannot add amounts in different currencies ({} vs {})",
            self.currency.code, rhs.currency.code
        );
        Self::new(self.value + rhs.value, self.currency)
    }
}

impl std::ops::Sub for Money {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        assert_eq!(
            self.currency, rhs.currency,
            "cannot subtract amounts in different currencies ({} vs {})",
            self.currency.code, rhs.currency.code
        );
        Self::new(self.value - rhs.value, self.currency)
    }
}

impl std::ops::Neg for Money {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.value, self.currency)
    }
}

impl std::ops::Mul<Real> for Money {
    type Output = Self;
    fn mul(self, rhs: Real) -> Self {
        Self::new(self.value * rhs, self.currency)
    }
}

impl std::ops::Mul<Money> for Real {
    type Output = Money;
    fn mul(self, rhs: Money) -> Money {
        Money::new(self * rhs.value, rhs.currency)
    }
}

impl std::ops::Div<Real> for Money {
    type Output = Self;
    fn div(self, rhs: Real) -> Self {
        Self::new(self.value / rhs, self.currency)
    }
}

impl std::fmt::Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2} {}", self.value, self.currency.code)
    }
}
