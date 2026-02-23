//! Observer / Observable pattern (translates `ql/patterns/observable.hpp`).
//!
//! QuantLib's core notification mechanism:
//! * An **Observable** object notifies registered **Observer**s whenever it
//!   changes state.
//! * Observers react by calling `update()`.
//!
//! In Rust we express this with traits; concrete implementations can use
//! `Arc<Mutex<…>>` for thread-safe scenarios or `Rc<RefCell<…>>` for
//! single-threaded use.

use std::sync::{Arc, Mutex, Weak};

/// An object that can notify interested parties when it changes.
///
/// Implementors hold a list of `Weak` references to registered [`Observer`]s
/// and call `notify_observers()` whenever their state changes.
pub trait Observable {
    /// Register an observer to receive future change notifications.
    fn register_observer(&mut self, observer: Weak<dyn Observer>);

    /// Remove a previously registered observer.
    fn unregister_observer(&mut self, observer: &Weak<dyn Observer>);

    /// Notify all currently registered observers that this object has changed.
    fn notify_observers(&mut self);
}

/// An object that reacts to changes in [`Observable`]s it has subscribed to.
pub trait Observer: Send + Sync {
    /// Called by every observable this observer is registered with when that
    /// observable changes state.
    fn update(&self);
}

/// A helper struct that can be embedded in any type to provide the standard
/// observer-list management (equivalent to `Observable::Impl` in QuantLib).
#[derive(Default)]
pub struct ObservableImpl {
    observers: Vec<Weak<dyn Observer>>,
}

impl ObservableImpl {
    /// Create a new, empty observable implementation.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an observer.
    pub fn register(&mut self, observer: Weak<dyn Observer>) {
        self.observers.push(observer);
    }

    /// Remove an observer (by pointer equality of the `Weak`).
    pub fn unregister(&mut self, observer: &Weak<dyn Observer>) {
        self.observers
            .retain(|o| !Weak::ptr_eq(o, observer));
    }

    /// Notify all live observers, removing dead `Weak` references as we go.
    pub fn notify(&mut self) {
        self.observers.retain(|weak| {
            if let Some(obs) = weak.upgrade() {
                obs.update();
                true
            } else {
                false // prune dead reference
            }
        });
    }
}

/// A simple observable that wraps a value and notifies observers on mutation.
///
/// This is the Rust analogue of QuantLib's `Observable` base class when used
/// standalone (e.g., in `SimpleQuote`).
pub struct NotifyingValue<T> {
    value: T,
    inner: ObservableImpl,
}

impl<T: Clone> NotifyingValue<T> {
    /// Create a new `NotifyingValue` with the given initial value.
    pub fn new(value: T) -> Self {
        Self {
            value,
            inner: ObservableImpl::new(),
        }
    }

    /// Return a reference to the current value without triggering a notification.
    pub fn get(&self) -> &T {
        &self.value
    }

    /// Set a new value and notify all registered observers.
    pub fn set(&mut self, value: T) {
        self.value = value;
        self.inner.notify();
    }

    /// Register an observer.
    pub fn register_observer(&mut self, observer: Weak<dyn Observer>) {
        self.inner.register(observer);
    }

    /// Unregister an observer.
    pub fn unregister_observer(&mut self, observer: &Weak<dyn Observer>) {
        self.inner.unregister(observer);
    }
}

/// A thread-safe, shared handle to an [`Observable`] value.
///
/// Observers can subscribe to change notifications via `Arc<Mutex<T>>` handles.
pub type SharedObservable<T> = Arc<Mutex<NotifyingValue<T>>>;
