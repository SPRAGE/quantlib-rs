//! Error types for quantlib-rs.
//!
//! This module translates QuantLib's exception hierarchy (rooted in
//! `std::exception`) to a single `thiserror`-derived enum.  The C++ macros
//! `QL_REQUIRE`, `QL_ENSURE`, and `QL_FAIL` map to the `ensure!` and `fail!`
//! convenience macros defined here.

use thiserror::Error;

/// The top-level error type used throughout quantlib-rs.
///
/// Mirrors QuantLib's exception hierarchy.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum Error {
    /// General runtime error (maps to `QL_FAIL`).
    #[error("{0}")]
    Runtime(String),

    /// Precondition violated (maps to `QL_REQUIRE`).
    #[error("precondition not satisfied: {0}")]
    Precondition(String),

    /// Postcondition violated (maps to `QL_ENSURE`).
    #[error("postcondition not satisfied: {0}")]
    Postcondition(String),

    /// An operation was requested on a null / unset value.
    #[error("null value")]
    NullValue,

    /// Date-related error.
    #[error("date error: {0}")]
    Date(String),

    /// Index out of range.
    #[error("index ({index}) out of range [0, {size})")]
    IndexOutOfRange {
        /// The index that was out of range.
        index: usize,
        /// The size of the container.
        size: usize,
    },

    /// Invalid argument.
    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    /// Not implemented.
    #[error("not implemented: {0}")]
    NotImplemented(String),
}

/// Shorthand `Result` type used throughout quantlib-rs.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Equivalent to C++ `QL_REQUIRE(condition, message)`.
///
/// Returns `Err(Error::Precondition(...))` if `$cond` is false.
///
/// # Example
/// ```
/// use ql_core::{ensure, errors::Error};
/// fn positive(x: f64) -> ql_core::errors::Result<f64> {
///     ensure!(x > 0.0, "x must be positive, got {x}");
///     Ok(x)
/// }
/// assert!(positive(1.0).is_ok());
/// assert!(positive(-1.0).is_err());
/// ```
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $($msg:tt)*) => {
        if !$cond {
            return Err($crate::errors::Error::Precondition(
                format!($($msg)*)
            ));
        }
    };
}

/// Equivalent to C++ `QL_ENSURE(condition, message)`.
///
/// Returns `Err(Error::Postcondition(...))` if `$cond` is false.
///
/// # Example
/// ```
/// use ql_core::{ensure_post, errors::Error};
/// fn compute(x: f64) -> ql_core::errors::Result<f64> {
///     let result = x * 2.0;
///     ensure_post!(result > 0.0, "result must be positive, got {result}");
///     Ok(result)
/// }
/// assert!(compute(1.0).is_ok());
/// assert!(compute(-1.0).is_err());
/// ```
#[macro_export]
macro_rules! ensure_post {
    ($cond:expr, $($msg:tt)*) => {
        if !$cond {
            return Err($crate::errors::Error::Postcondition(
                format!($($msg)*)
            ));
        }
    };
}

/// Equivalent to C++ `QL_FAIL(message)`.
///
/// Returns `Err(Error::Runtime(...))` immediately.
///
/// # Example
/// ```
/// use ql_core::{fail, errors::Error};
/// fn always_err() -> ql_core::errors::Result<()> {
///     fail!("something went wrong");
/// }
/// assert!(always_err().is_err());
/// ```
#[macro_export]
macro_rules! fail {
    ($($msg:tt)*) => {
        return Err($crate::errors::Error::Runtime(format!($($msg)*)))
    };
}
