//! `Quote` trait and `SimpleQuote` implementation.
//!
//! Translates `ql/quote.hpp` and `ql/quotes/simplequote.hpp`.

use ql_core::{errors::Result, Real};

/// A market-observable value.
///
/// Corresponds to `QuantLib::Quote`.
pub trait Quote: std::fmt::Debug + Send + Sync {
    /// Return the current value.
    ///
    /// Returns `None` if the quote is not currently valid / set.
    fn value(&self) -> Option<Real>;

    /// Return `true` if the quote is currently valid.
    fn is_valid(&self) -> bool {
        self.value().is_some()
    }
}

/// A simple, mutable market quote.
///
/// Corresponds to `QuantLib::SimpleQuote`.
#[derive(Debug, Clone)]
pub struct SimpleQuote {
    value: Option<Real>,
}

impl SimpleQuote {
    /// Create a new quote with the given value.
    pub fn new(value: Real) -> Self {
        Self { value: Some(value) }
    }

    /// Create an empty (invalid) quote.
    pub fn empty() -> Self {
        Self { value: None }
    }

    /// Set a new value.
    pub fn set_value(&mut self, value: Real) {
        self.value = Some(value);
    }

    /// Clear the value, making the quote invalid.
    pub fn reset(&mut self) {
        self.value = None;
    }
}

impl Quote for SimpleQuote {
    fn value(&self) -> Option<Real> {
        self.value
    }
}

/// A quote derived as the negative of another quote.
#[derive(Debug)]
pub struct NegativeQuote<Q: Quote> {
    inner: Q,
}

impl<Q: Quote> NegativeQuote<Q> {
    /// Wrap a quote, negating its value.
    pub fn new(inner: Q) -> Self {
        Self { inner }
    }
}

impl<Q: Quote> Quote for NegativeQuote<Q> {
    fn value(&self) -> Option<Real> {
        self.inner.value().map(|v| -v)
    }
}

/// A quote derived as the composite of two quotes via an arithmetic operation.
#[derive(Debug)]
pub struct CompositeQuote<Q1: Quote, Q2: Quote, F> {
    q1: Q1,
    q2: Q2,
    func: F,
}

impl<Q1: Quote, Q2: Quote, F> CompositeQuote<Q1, Q2, F>
where
    F: Fn(Real, Real) -> Result<Real> + std::fmt::Debug + Send + Sync,
{
    /// Create a composite quote.
    pub fn new(q1: Q1, q2: Q2, func: F) -> Self {
        Self { q1, q2, func }
    }
}

impl<Q1: Quote, Q2: Quote, F> Quote for CompositeQuote<Q1, Q2, F>
where
    F: Fn(Real, Real) -> Result<Real> + std::fmt::Debug + Send + Sync,
{
    fn value(&self) -> Option<Real> {
        let v1 = self.q1.value()?;
        let v2 = self.q2.value()?;
        (self.func)(v1, v2).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_quote() {
        let q = SimpleQuote::new(1.05);
        assert!(q.is_valid());
        assert_eq!(q.value(), Some(1.05));
    }

    #[test]
    fn empty_quote() {
        let q = SimpleQuote::empty();
        assert!(!q.is_valid());
        assert_eq!(q.value(), None);
    }

    #[test]
    fn negative_quote() {
        let q = NegativeQuote::new(SimpleQuote::new(2.0));
        assert_eq!(q.value(), Some(-2.0));
    }
}
