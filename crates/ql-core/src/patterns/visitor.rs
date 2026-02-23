//! Visitor pattern (translates `ql/patterns/visitor.hpp`).
//!
//! QuantLib uses the visitor pattern primarily for instrument / payoff
//! dispatch.  In Rust this is expressed as a trait with a single `visit`
//! method parameterised over the visited type.

/// A visitor that can inspect objects of type `T`.
///
/// Corresponds to `QuantLib::Visitor<T>`.
pub trait Visitor<T> {
    /// Visit an object of type `T`.
    fn visit(&mut self, visitable: &T);
}

/// An object that can be visited by any [`Visitor`].
///
/// Corresponds to `QuantLib::AcyclicVisitable`.
pub trait AcyclicVisitable {
    /// Accept a type-erased visitor.
    ///
    /// The default implementation does nothing (graceful no-op, matching
    /// QuantLib's `AcyclicVisitable::accept` default body).
    fn accept(&self, visitor: &mut dyn std::any::Any) {
        let _ = visitor; // default: ignore
    }
}
