//! Null / unset value utilities (translates `ql/utilities/null.hpp`).
//!
//! In QuantLib many optional numeric values are represented as a special
//! "null" sentinel (e.g., `Null<Rate>()` returns the largest finite `double`).
//! Rust idiomatically uses `Option<T>` for this, but we expose a `Null` trait
//! for interoperability with code that tests `x == Null::<T>::value()`.

/// A type that has a distinguished "null" sentinel value.
///
/// Implementors provide a `value()` associated function returning the sentinel.
/// By convention the sentinel is the **maximum** value of the type, mirroring
/// QuantLib's `Null<Real>() == std::numeric_limits<double>::max()`.
pub trait Null: Sized + PartialEq + Copy {
    /// The null / unset sentinel value for this type.
    fn null() -> Self;

    /// Return `true` if `self` equals the null sentinel.
    fn is_null(&self) -> bool {
        *self == Self::null()
    }
}

impl Null for f64 {
    fn null() -> Self {
        f64::MAX
    }
}

impl Null for f32 {
    fn null() -> Self {
        f32::MAX
    }
}

impl Null for i32 {
    fn null() -> Self {
        i32::MIN
    }
}

impl Null for i64 {
    fn null() -> Self {
        i64::MIN
    }
}

impl Null for u32 {
    fn null() -> Self {
        u32::MAX
    }
}

impl Null for usize {
    fn null() -> Self {
        usize::MAX
    }
}
