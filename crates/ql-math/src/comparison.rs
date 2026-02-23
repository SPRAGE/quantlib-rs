//! Comparison utilities (translates `ql/math/comparison.hpp`).

use ql_core::Real;

/// Default epsilon for close-enough comparisons.
pub const EPSILON: Real = 1e-10;

/// Return `true` if `|a - b| <= epsilon`.
#[inline]
pub fn close(a: Real, b: Real, epsilon: Real) -> bool {
    (a - b).abs() <= epsilon
}

/// Return `true` if `|a - b| <= n * epsilon` where `epsilon` is the
/// machine-epsilon relative to `max(|a|, |b|)`.
#[inline]
pub fn close_enough(a: Real, b: Real, n: u32) -> bool {
    if a == b {
        return true;
    }
    let eps = (a.abs().max(b.abs())) * f64::EPSILON * n as f64;
    (a - b).abs() <= eps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn close_basic() {
        assert!(close(1.0, 1.0 + 1e-11, 1e-10));
        assert!(!close(1.0, 1.0 + 1e-9, 1e-10));
    }

    #[test]
    fn close_enough_basic() {
        assert!(close_enough(1.0, 1.0, 10));
        assert!(close_enough(1.0, 1.0 + f64::EPSILON * 5.0, 10));
    }
}
