//! Observer / Observable pattern (translates `ql/patterns/observable.hpp`).
//!
//! QuantLib's core notification mechanism:
//! * An **Observable** object notifies registered **Observer**s whenever it
//!   changes state.
//! * Observers react by calling `update()`.
//!
//! In Rust we express this with traits using interior mutability (`RefCell`)
//! so that registration and notification work through `&self` references,
//! matching QuantLib's usage where observables are shared via `shared_ptr`.

use std::cell::RefCell;
use std::sync::{Arc, Mutex, Weak};

/// An object that can notify interested parties when it changes.
///
/// Implementors hold a list of `Weak` references to registered [`Observer`]s
/// and call `notify_observers()` whenever their state changes.
///
/// All methods take `&self` (not `&mut self`) to support shared ownership
/// patterns — interior mutability is used for the observer list.
pub trait Observable {
    /// Register an observer to receive future change notifications.
    fn register_observer(&self, observer: Weak<dyn Observer>);

    /// Remove a previously registered observer.
    fn unregister_observer(&self, observer: &Weak<dyn Observer>);

    /// Notify all currently registered observers that this object has changed.
    fn notify_observers(&self);
}

/// An object that reacts to changes in [`Observable`]s it has subscribed to.
pub trait Observer: Send + Sync {
    /// Called by every observable this observer is registered with when that
    /// observable changes state.
    fn update(&self);
}

/// A helper struct that can be embedded in any type to provide the standard
/// observer-list management (equivalent to `Observable::Impl` in QuantLib).
///
/// Uses interior mutability via `RefCell` so that `register`, `unregister`,
/// and `notify` all work through `&self` references.
pub struct ObservableImpl {
    observers: RefCell<Vec<Weak<dyn Observer>>>,
}

impl Default for ObservableImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl ObservableImpl {
    /// Create a new, empty observable implementation.
    pub fn new() -> Self {
        Self {
            observers: RefCell::new(Vec::new()),
        }
    }

    /// Register an observer.
    pub fn register(&self, observer: Weak<dyn Observer>) {
        self.observers.borrow_mut().push(observer);
    }

    /// Remove an observer (by pointer equality of the `Weak`).
    pub fn unregister(&self, observer: &Weak<dyn Observer>) {
        self.observers
            .borrow_mut()
            .retain(|o| !Weak::ptr_eq(o, observer));
    }

    /// Notify all live observers, removing dead `Weak` references as we go.
    pub fn notify(&self) {
        // Collect live observers first, then call update outside borrow
        let observers: Vec<Arc<dyn Observer>> = self
            .observers
            .borrow()
            .iter()
            .filter_map(|w| w.upgrade())
            .collect();
        // Prune dead references
        self.observers
            .borrow_mut()
            .retain(|w| w.upgrade().is_some());
        // Notify outside the borrow
        for obs in observers {
            obs.update();
        }
    }
}

/// A simple observable that wraps a value and notifies observers on mutation.
///
/// This is the Rust analogue of QuantLib's `Observable` base class when used
/// standalone (e.g., in `SimpleQuote`).
pub struct NotifyingValue<T> {
    value: RefCell<T>,
    inner: ObservableImpl,
}

impl<T: Clone> NotifyingValue<T> {
    /// Create a new `NotifyingValue` with the given initial value.
    pub fn new(value: T) -> Self {
        Self {
            value: RefCell::new(value),
            inner: ObservableImpl::new(),
        }
    }

    /// Return a clone of the current value without triggering a notification.
    pub fn get(&self) -> T {
        self.value.borrow().clone()
    }

    /// Set a new value and notify all registered observers.
    pub fn set(&self, value: T) {
        *self.value.borrow_mut() = value;
        self.inner.notify();
    }
}

impl<T: Clone> Observable for NotifyingValue<T> {
    fn register_observer(&self, observer: Weak<dyn Observer>) {
        self.inner.register(observer);
    }

    fn unregister_observer(&self, observer: &Weak<dyn Observer>) {
        self.inner.unregister(observer);
    }

    fn notify_observers(&self) {
        self.inner.notify();
    }
}

/// A thread-safe, shared handle to an [`Observable`] value.
///
/// Observers can subscribe to change notifications via `Arc<Mutex<T>>` handles.
pub type SharedObservable<T> = Arc<Mutex<NotifyingValue<T>>>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    struct CountingObserver {
        count: AtomicU32,
    }

    impl Observer for CountingObserver {
        fn update(&self) {
            self.count.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[test]
    fn register_and_notify() {
        let obs = Arc::new(CountingObserver {
            count: AtomicU32::new(0),
        });
        let observable = ObservableImpl::new();
        observable.register(Arc::downgrade(&obs) as Weak<dyn Observer>);
        observable.notify();
        assert_eq!(obs.count.load(Ordering::Relaxed), 1);
        observable.notify();
        assert_eq!(obs.count.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn dead_observer_pruned() {
        let observable = ObservableImpl::new();
        {
            let obs = Arc::new(CountingObserver {
                count: AtomicU32::new(0),
            });
            observable.register(Arc::downgrade(&obs) as Weak<dyn Observer>);
        }
        // obs dropped — notify should prune it
        observable.notify();
        assert_eq!(observable.observers.borrow().len(), 0);
    }

    #[test]
    fn unregister() {
        let obs = Arc::new(CountingObserver {
            count: AtomicU32::new(0),
        });
        let weak = Arc::downgrade(&obs) as Weak<dyn Observer>;
        let observable = ObservableImpl::new();
        observable.register(weak.clone());
        observable.unregister(&weak);
        observable.notify();
        assert_eq!(obs.count.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn notifying_value() {
        let obs = Arc::new(CountingObserver {
            count: AtomicU32::new(0),
        });
        let nv = NotifyingValue::new(42.0_f64);
        nv.register_observer(Arc::downgrade(&obs) as Weak<dyn Observer>);
        nv.set(100.0);
        assert_eq!(obs.count.load(Ordering::Relaxed), 1);
        assert!((nv.get() - 100.0).abs() < f64::EPSILON);
    }
}
