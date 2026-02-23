//! Singleton pattern (translates `ql/patterns/singleton.hpp`).
//!
//! In QuantLib the `Singleton<T>` template ensures a single global instance
//! of `T`.  In Rust the idiomatic equivalent is `std::sync::LazyLock<T>`.
//!
//! This module re-exports `LazyLock` and provides a convenience macro
//! [`define_singleton!`] for declaring singletons.

/// Re-export `LazyLock` as the canonical singleton container.
pub use std::sync::LazyLock;

/// Define a singleton instance of type `$ty`.
///
/// The instance is lazily initialised on first access via `LazyLock`.
///
/// # Example
/// ```
/// use ql_core::define_singleton;
///
/// struct Registry { data: Vec<String> }
/// define_singleton!(REGISTRY, Registry, Registry { data: Vec::new() });
///
/// assert!(REGISTRY.data.is_empty());
/// ```
#[macro_export]
macro_rules! define_singleton {
    ($name:ident, $ty:ty, $init:expr) => {
        /// Lazily-initialised global singleton.
        pub static $name: std::sync::LazyLock<$ty> = std::sync::LazyLock::new(|| $init);
    };
}
