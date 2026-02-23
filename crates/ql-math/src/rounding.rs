//! Rounding utilities (translates `ql/math/rounding.hpp`).

use ql_core::Real;

/// Rounding convention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rounding {
    /// No rounding — return the value unchanged.
    None,
    /// Round up (ceiling towards positive infinity).
    Up,
    /// Round down (floor towards negative infinity).
    Down,
    /// Round to nearest, ties away from zero (standard mathematical rounding).
    Closest,
    /// Round towards zero (truncation).
    Floor,
    /// Round away from zero (ceiling of absolute value).
    Ceiling,
}

/// Round `value` to `precision` decimal places using the given convention.
pub fn round(value: Real, precision: i32, convention: Rounding) -> Real {
    if matches!(convention, Rounding::None) {
        return value;
    }
    let mult = 10_f64.powi(precision);
    match convention {
        Rounding::None => value,
        Rounding::Up => (value * mult).ceil() / mult,
        Rounding::Down => (value * mult).floor() / mult,
        Rounding::Closest => (value * mult).round() / mult,
        Rounding::Floor => (value * mult).floor() / mult,
        Rounding::Ceiling => (value * mult).ceil() / mult,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closest_rounding() {
        assert!((round(1.2345, 2, Rounding::Closest) - 1.23).abs() < 1e-10);
        assert!((round(1.2355, 2, Rounding::Closest) - 1.24).abs() < 1e-10);
    }

    #[test]
    fn up_rounding() {
        assert!((round(1.2301, 2, Rounding::Up) - 1.24).abs() < 1e-10);
    }
}
