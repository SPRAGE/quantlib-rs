//! Compounding conventions (translates `ql/compounding.hpp`).

/// How interest is compounded.
///
/// Mirrors `QuantLib::Compounding`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Compounding {
    /// Simple interest: `1 + r·t`
    Simple,
    /// Compounded interest: `(1 + r)^t`
    Compounded,
    /// Continuously compounded: `e^(r·t)`
    Continuous,
    /// Simple interest **up to** the first coupon, compounded thereafter.
    SimpleThenCompounded,
    /// Compounded up to the last coupon, simple thereafter.
    CompoundedThenSimple,
}
