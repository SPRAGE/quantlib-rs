//! `Handle<T>` — a shared, observable reference to a value (translates
//! `ql/handle.hpp`).
//!
//! A `Handle<T>` is a reference-counted pointer to a value.  Multiple handles
//! can share ownership of the same underlying value.
//!
//! A [`RelinkableHandle<T>`] can be repointed to a different value at runtime;
//! when this happens all registered observers are notified.
//!
//! | C++ | Rust |
//! |-----|------|
//! | `Handle<T>` (shared_ptr, non-owning) | `Handle<T>` (Arc, optionally null) |
//! | `RelinkableHandle<T>` | `RelinkableHandle<T>` (wraps `Arc<Mutex<…>>` + observer list) |

use crate::patterns::observable::{Observable, ObservableImpl, Observer};
use std::sync::{Arc, Mutex, Weak};

/// A shared, optionally-null reference to a value of type `T`.
///
/// Equivalent to QuantLib's `Handle<T>`.  The handle is *read-only* — to
/// replace the contained value use a [`RelinkableHandle`].
#[derive(Clone)]
pub struct Handle<T> {
    inner: Option<Arc<T>>,
}

impl<T> Handle<T> {
    /// Create a non-null handle wrapping `value`.
    pub fn new(value: T) -> Self {
        Self {
            inner: Some(Arc::new(value)),
        }
    }

    /// Create a handle from an existing `Arc`.
    pub fn from_arc(arc: Arc<T>) -> Self {
        Self { inner: Some(arc) }
    }

    /// Create a null (empty) handle.
    pub fn null() -> Self {
        Self { inner: None }
    }

    /// Return `true` if the handle is null (contains no value).
    pub fn is_empty(&self) -> bool {
        self.inner.is_none()
    }

    /// Return a reference to the inner `Arc<T>`, or `None` if this handle is
    /// null.
    pub fn as_arc(&self) -> Option<&Arc<T>> {
        self.inner.as_ref()
    }

    /// Attempt to borrow the contained value.
    ///
    /// Returns `None` if the handle is null.
    pub fn get(&self) -> Option<&T> {
        self.inner.as_deref()
    }

    /// Dereference the handle, panicking if it is null.
    ///
    /// Use only when you know the handle is non-null (e.g., after validation).
    pub fn unwrap(&self) -> &T {
        self.inner.as_deref().expect("dereferenced a null Handle")
    }
}

impl<T> Default for Handle<T> {
    fn default() -> Self {
        Self::null()
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            Some(v) => write!(f, "Handle({:?})", v),
            None => write!(f, "Handle(null)"),
        }
    }
}

/// A [`Handle`] whose contained value can be relinked at runtime.
///
/// Equivalent to QuantLib's `RelinkableHandle<T>`.  When the handle is
/// relinked via [`link_to`][Self::link_to] or [`link_to_arc`][Self::link_to_arc],
/// all registered observers are notified.
#[derive(Clone)]
pub struct RelinkableHandle<T> {
    inner: Arc<Mutex<Option<Arc<T>>>>,
    observable: Arc<ObservableImpl>,
}

#[allow(clippy::arc_with_non_send_sync)]
impl<T> RelinkableHandle<T> {
    /// Create a new relinkable handle, initially null.
    pub fn null() -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
            observable: Arc::new(ObservableImpl::new()),
        }
    }

    /// Create a new relinkable handle wrapping `value`.
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Some(Arc::new(value)))),
            observable: Arc::new(ObservableImpl::new()),
        }
    }

    /// Replace the contained value with `value`, notifying any observers.
    pub fn link_to(&self, value: T) {
        {
            let mut guard = self.inner.lock().expect("RelinkableHandle mutex poisoned");
            *guard = Some(Arc::new(value));
        }
        self.observable.notify();
    }

    /// Replace the contained value with an existing `Arc`, notifying observers.
    pub fn link_to_arc(&self, arc: Arc<T>) {
        {
            let mut guard = self.inner.lock().expect("RelinkableHandle mutex poisoned");
            *guard = Some(arc);
        }
        self.observable.notify();
    }

    /// Detach the handle from any value (make it null), notifying observers.
    pub fn unlink(&self) {
        {
            let mut guard = self.inner.lock().expect("RelinkableHandle mutex poisoned");
            *guard = None;
        }
        self.observable.notify();
    }

    /// Return `true` if the handle currently contains no value.
    pub fn is_empty(&self) -> bool {
        self.inner
            .lock()
            .expect("RelinkableHandle mutex poisoned")
            .is_none()
    }

    /// Execute a closure with a reference to the contained value.
    ///
    /// Returns `None` if the handle is null.
    pub fn with<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&T) -> R,
    {
        let guard = self.inner.lock().expect("RelinkableHandle mutex poisoned");
        guard.as_deref().map(f)
    }

    /// Obtain a snapshot `Arc<T>` of the current value.
    ///
    /// Returns `None` if the handle is null.
    pub fn current(&self) -> Option<Arc<T>> {
        let guard = self.inner.lock().expect("RelinkableHandle mutex poisoned");
        guard.clone()
    }
}

impl<T> Observable for RelinkableHandle<T> {
    fn register_observer(&self, observer: Weak<dyn Observer>) {
        self.observable.register(observer);
    }

    fn unregister_observer(&self, observer: &Weak<dyn Observer>) {
        self.observable.unregister(observer);
    }

    fn notify_observers(&self) {
        self.observable.notify();
    }
}

impl<T> Default for RelinkableHandle<T> {
    fn default() -> Self {
        Self::null()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    struct Counter(AtomicU32);
    impl Observer for Counter {
        fn update(&self) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[test]
    fn relinkable_handle_notifies_on_link() {
        let obs = Arc::new(Counter(AtomicU32::new(0)));
        let h = RelinkableHandle::new(42i32);
        h.register_observer(Arc::downgrade(&obs) as Weak<dyn Observer>);
        h.link_to(99);
        assert_eq!(obs.0.load(Ordering::Relaxed), 1);
        h.unlink();
        assert_eq!(obs.0.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn handle_get() {
        let h = Handle::new(3.14_f64);
        assert!((h.unwrap() - 3.14).abs() < f64::EPSILON);
        let null: Handle<f64> = Handle::null();
        assert!(null.get().is_none());
    }
}
