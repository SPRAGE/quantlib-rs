//! Clone helpers (translates `ql/utilities/clone.hpp`).
//!
//! QuantLib provides a `clone()` function that deep-copies a
//! `shared_ptr<T>`.  In Rust this is simply the `Clone` trait, which is
//! already built-in.  This module provides a convenience trait for cloning
//! trait objects into `Box<dyn Trait>`.

/// Extension trait that allows cloning a trait object into a `Box`.
///
/// Implement this for any trait whose concrete types are `Clone`:
///
/// ```
/// use ql_core::utilities::clone::CloneBox;
///
/// trait MyTrait: CloneBox {}
///
/// // Auto-implemented for any T: Clone + MyTrait
/// ```
pub trait CloneBox {
    /// Clone this value into a heap-allocated `Box`.
    fn clone_box(&self) -> Box<dyn CloneBox>;
}

impl<T: Clone + 'static> CloneBox for T {
    fn clone_box(&self) -> Box<dyn CloneBox> {
        Box::new(self.clone())
    }
}
