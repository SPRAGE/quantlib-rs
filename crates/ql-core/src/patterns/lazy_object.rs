//! LazyObject pattern (translates `ql/patterns/lazyobject.hpp`).
//!
//! `LazyObject` combines the `Observable` and `Observer` traits: it caches
//! an expensive computation and recalculates only when its inputs change.
//!
//! In Rust the caching uses interior mutability (`Cell<bool>`) so that the
//! calculation can be triggered via an `&self` reference (matching QuantLib's
//! use of `mutable`).

use std::cell::Cell;

/// Trait for objects that lazily compute and cache their results.
///
/// Implementors must provide [`perform_calculations`][Self::perform_calculations].
/// The base machinery in this trait handles the `calculated_` flag and
/// `freeze_count` logic from QuantLib's `LazyObject`.
pub trait LazyObject {
    /// Perform the actual (expensive) calculation.
    ///
    /// Called automatically by [`calculate`][Self::calculate] when the cached
    /// result is stale.
    fn perform_calculations(&self) -> crate::errors::Result<()>;

    /// Trigger recalculation on next call to [`calculate`][Self::calculate].
    ///
    /// This is equivalent to QuantLib's `LazyObject::update()` – it clears the
    /// `calculated_` flag.
    fn recalculate_flag(&self) -> &Cell<bool>;

    /// If `false`, the object will defer recalculation until unfrozen.
    fn freeze_count(&self) -> &Cell<u32>;

    /// Ensure results are up-to-date.
    ///
    /// If the cache is stale (and not frozen), calls
    /// [`perform_calculations`][Self::perform_calculations] and marks the cache
    /// as valid.
    fn calculate(&self) -> crate::errors::Result<()> {
        if !self.recalculate_flag().get() && self.freeze_count().get() == 0 {
            self.recalculate_flag().set(true);
            self.perform_calculations()?;
        }
        Ok(())
    }

    /// Mark the cached result as stale without triggering a recalculation.
    fn update(&self) {
        self.recalculate_flag().set(false);
    }

    /// Prevent automatic recalculation until [`unfreeze`][Self::unfreeze] is
    /// called.
    fn freeze(&self) {
        self.freeze_count().set(self.freeze_count().get() + 1);
    }

    /// Undo one call to [`freeze`][Self::freeze].
    ///
    /// When the freeze count reaches zero the object recalculates on the next
    /// [`calculate`][Self::calculate] call.
    fn unfreeze(&self) {
        let count = self.freeze_count().get();
        if count > 0 {
            self.freeze_count().set(count - 1);
        }
    }

    /// Return `true` if the cache is currently valid.
    fn is_calculated(&self) -> bool {
        self.recalculate_flag().get()
    }

    /// Return `true` if recalculation is currently deferred.
    fn is_frozen(&self) -> bool {
        self.freeze_count().get() > 0
    }
}

/// Convenience struct that holds the bookkeeping fields required by
/// [`LazyObject`].
///
/// Embed this in your struct and delegate the accessor methods to it.
///
/// # Example
/// ```
/// use std::cell::Cell;
/// use ql_core::patterns::lazy_object::{LazyObject, LazyState};
///
/// struct MyLazy {
///     state: LazyState,
///     result: Cell<f64>,
/// }
///
/// impl LazyObject for MyLazy {
///     fn perform_calculations(&self) -> ql_core::errors::Result<()> {
///         self.result.set(42.0);
///         Ok(())
///     }
///     fn recalculate_flag(&self) -> &Cell<bool> { &self.state.calculated }
///     fn freeze_count(&self) -> &Cell<u32> { &self.state.freeze_count }
/// }
///
/// let obj = MyLazy { state: LazyState::new(), result: Cell::new(0.0) };
/// obj.calculate().unwrap();
/// assert_eq!(obj.result.get(), 42.0);
/// ```
pub struct LazyState {
    /// `true` when the cached result is valid.
    pub calculated: Cell<bool>,
    /// Number of times the object has been frozen without a matching unfreeze.
    pub freeze_count: Cell<u32>,
}

impl LazyState {
    /// Create a new `LazyState` where the cache is initially stale.
    pub fn new() -> Self {
        Self {
            calculated: Cell::new(false),
            freeze_count: Cell::new(0),
        }
    }
}

impl Default for LazyState {
    fn default() -> Self {
        Self::new()
    }
}
