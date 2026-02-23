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
pub struct CompositeQuote<Q1: Quote, Q2: Quote, F> {
    q1: Q1,
    q2: Q2,
    func: F,
}

impl<Q1: Quote, Q2: Quote, F> std::fmt::Debug for CompositeQuote<Q1, Q2, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompositeQuote")
            .field("q1", &self.q1)
            .field("q2", &self.q2)
            .field("func", &std::any::type_name::<F>())
            .finish()
    }
}

impl<Q1: Quote, Q2: Quote, F> CompositeQuote<Q1, Q2, F>
where
    F: Fn(Real, Real) -> Result<Real> + Send + Sync,
{
    /// Create a composite quote.
    pub fn new(q1: Q1, q2: Q2, func: F) -> Self {
        Self { q1, q2, func }
    }
}

impl<Q1: Quote, Q2: Quote, F> Quote for CompositeQuote<Q1, Q2, F>
where
    F: Fn(Real, Real) -> Result<Real> + Send + Sync,
{
    fn value(&self) -> Option<Real> {
        let v1 = self.q1.value()?;
        let v2 = self.q2.value()?;
        (self.func)(v1, v2).ok()
    }
}

// ── DerivedQuote ──────────────────────────────────────────────────────────────

/// A quote whose value is derived by applying a unary function to another
/// quote.
///
/// Corresponds to `QuantLib::DerivedQuote`.
pub struct DerivedQuote<Q: Quote, F> {
    inner: Q,
    func: F,
}

impl<Q: Quote, F> std::fmt::Debug for DerivedQuote<Q, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DerivedQuote")
            .field("inner", &self.inner)
            .field("func", &std::any::type_name::<F>())
            .finish()
    }
}

impl<Q: Quote, F> DerivedQuote<Q, F>
where
    F: Fn(Real) -> Real + Send + Sync,
{
    /// Create a derived quote.
    pub fn new(inner: Q, func: F) -> Self {
        Self { inner, func }
    }
}

impl<Q: Quote, F> Quote for DerivedQuote<Q, F>
where
    F: Fn(Real) -> Real + Send + Sync,
{
    fn value(&self) -> Option<Real> {
        self.inner.value().map(&self.func)
    }
}

// ── ForwardValueQuote ─────────────────────────────────────────────────────────

/// A quote that wraps an inner quote and interprets it as a forward value
/// (i.e., divides by a discount factor).
///
/// Corresponds to `QuantLib::ForwardValueQuote` (simplified).
#[derive(Debug)]
pub struct ForwardValueQuote<Q: Quote> {
    inner: Q,
    discount: Real,
}

impl<Q: Quote> ForwardValueQuote<Q> {
    /// Create a forward-value quote. `discount` is the discount factor to
    /// the forward date.
    pub fn new(inner: Q, discount: Real) -> Self {
        Self { inner, discount }
    }
}

impl<Q: Quote> Quote for ForwardValueQuote<Q> {
    fn value(&self) -> Option<Real> {
        self.inner.value().map(|v| v / self.discount)
    }
}

// ── ImpliedStdDevQuote ────────────────────────────────────────────────────────

/// A quote that stores an implied standard-deviation level.
///
/// This is a pass-through wrapper that labels a quote as an implied vol ×
/// √t measure, commonly used in option pricing.
///
/// Corresponds to `QuantLib::ImpliedStdDevQuote` (simplified).
#[derive(Debug, Clone)]
pub struct ImpliedStdDevQuote {
    value: Option<Real>,
}

impl ImpliedStdDevQuote {
    /// Create from a known implied-stddev value.
    pub fn new(value: Real) -> Self {
        Self { value: Some(value) }
    }

    /// Create an empty quote.
    pub fn empty() -> Self {
        Self { value: None }
    }

    /// Set the value.
    pub fn set_value(&mut self, value: Real) {
        self.value = Some(value);
    }
}

impl Quote for ImpliedStdDevQuote {
    fn value(&self) -> Option<Real> {
        self.value
    }
}

// ── LastFixingQuote ───────────────────────────────────────────────────────────

/// A quote that always returns a fixed snapshot value.
///
/// Corresponds to `QuantLib::LastFixingQuote` (simplified — no index
/// dependency; just a frozen value).
#[derive(Debug, Clone)]
pub struct LastFixingQuote {
    value: Option<Real>,
}

impl LastFixingQuote {
    /// Create from a known fixing value.
    pub fn new(value: Real) -> Self {
        Self { value: Some(value) }
    }

    /// Create empty.
    pub fn empty() -> Self {
        Self { value: None }
    }

    /// Set the fixing.
    pub fn set_value(&mut self, value: Real) {
        self.value = Some(value);
    }
}

impl Quote for LastFixingQuote {
    fn value(&self) -> Option<Real> {
        self.value
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

    #[test]
    fn derived_quote() {
        let q = DerivedQuote::new(SimpleQuote::new(4.0), |v| v.sqrt());
        assert_eq!(q.value(), Some(2.0));
    }

    #[test]
    fn derived_quote_empty_inner() {
        let q = DerivedQuote::new(SimpleQuote::empty(), |v: Real| v * 2.0);
        assert_eq!(q.value(), None);
    }

    #[test]
    fn composite_quote() {
        let q = CompositeQuote::new(
            SimpleQuote::new(3.0),
            SimpleQuote::new(4.0),
            |a, b| Ok(a + b),
        );
        assert_eq!(q.value(), Some(7.0));
    }

    #[test]
    fn forward_value_quote() {
        let q = ForwardValueQuote::new(SimpleQuote::new(100.0), 0.95);
        let v = q.value().unwrap();
        assert!((v - 100.0 / 0.95).abs() < 1e-10);
    }

    #[test]
    fn implied_stddev_quote() {
        let q = ImpliedStdDevQuote::new(0.25);
        assert_eq!(q.value(), Some(0.25));
    }

    #[test]
    fn last_fixing_quote() {
        let mut q = LastFixingQuote::empty();
        assert!(!q.is_valid());
        q.set_value(42.0);
        assert_eq!(q.value(), Some(42.0));
    }
}
