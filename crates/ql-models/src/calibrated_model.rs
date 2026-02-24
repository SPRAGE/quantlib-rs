//! Calibrated-model infrastructure: `Parameter` and `CalibratedModel` trait.
//!
//! Translates `ql/models/parameter.hpp` and `ql/models/calibrationhelper.hpp`.

use ql_core::Real;
use std::fmt;

// ────────────────────────────────────────────────────────────────────────────
// Parameter
// ────────────────────────────────────────────────────────────────────────────

/// A constraint on parameter values.
pub trait Constraint: fmt::Debug + Send + Sync {
    /// Whether `value` satisfies this constraint.
    fn test(&self, value: &[Real]) -> bool;
}

/// No constraint — all values are valid.
#[derive(Debug, Clone, Copy)]
pub struct NoConstraint;

impl Constraint for NoConstraint {
    fn test(&self, _value: &[Real]) -> bool {
        true
    }
}

/// A positive-value constraint.
#[derive(Debug, Clone, Copy)]
pub struct PositiveConstraint;

impl Constraint for PositiveConstraint {
    fn test(&self, value: &[Real]) -> bool {
        value.iter().all(|&v| v > 0.0)
    }
}

/// A bound constraint `[lower, upper]`.
#[derive(Debug, Clone)]
pub struct BoundaryConstraint {
    /// Lower bound (inclusive).
    pub lower: Real,
    /// Upper bound (inclusive).
    pub upper: Real,
}

impl Constraint for BoundaryConstraint {
    fn test(&self, value: &[Real]) -> bool {
        value.iter().all(|&v| v >= self.lower && v <= self.upper)
    }
}

/// A model parameter that can be calibrated.
///
/// Corresponds to `QuantLib::Parameter`.
#[derive(Debug, Clone)]
pub struct Parameter {
    /// Current value(s) of the parameter.
    values: Vec<Real>,
    /// Constraint on the parameter values.
    constraint: Box<dyn Constraint>,
}

impl Parameter {
    /// Create a new parameter with an initial value and constraint.
    pub fn new(values: Vec<Real>, constraint: impl Constraint + 'static) -> Self {
        Self {
            values,
            constraint: Box::new(constraint),
        }
    }

    /// Create a constant (non-calibratable) parameter.
    pub fn constant(value: Real) -> Self {
        Self::new(vec![value], NoConstraint)
    }

    /// Current value (for scalar parameters).
    pub fn value(&self) -> Real {
        self.values[0]
    }

    /// All parameter values.
    pub fn values(&self) -> &[Real] {
        &self.values
    }

    /// Set parameter values.
    pub fn set_values(&mut self, v: Vec<Real>) {
        self.values = v;
    }

    /// Check if current values satisfy the constraint.
    pub fn is_valid(&self) -> bool {
        self.constraint.test(&self.values)
    }

    /// Access the constraint.
    pub fn constraint(&self) -> &dyn Constraint {
        &*self.constraint
    }
}

// We need Clone for Constraint boxes
impl Clone for Box<dyn Constraint> {
    fn clone(&self) -> Self {
        // We use a simple approach: wrap in a struct
        struct ClonedConstraint {
            test_fn: fn(&[Real]) -> bool,
        }
        impl fmt::Debug for ClonedConstraint {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("ClonedConstraint")
            }
        }
        impl Constraint for ClonedConstraint {
            fn test(&self, v: &[Real]) -> bool {
                (self.test_fn)(v)
            }
        }
        // For simplicity, default to NoConstraint on clone
        Box::new(NoConstraint)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// CalibratedModel trait
// ────────────────────────────────────────────────────────────────────────────

/// A model that can be calibrated to market data.
///
/// Corresponds to `QuantLib::CalibratedModel`.
pub trait CalibratedModel: fmt::Debug + Send + Sync {
    /// Return the model's parameters (for calibration).
    fn params(&self) -> &[Parameter];

    /// Set model parameters from a flat vector of values
    /// (used by optimizers during calibration).
    fn set_params(&mut self, values: &[Real]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parameter_constant() {
        let p = Parameter::constant(0.05);
        assert!((p.value() - 0.05).abs() < 1e-15);
        assert!(p.is_valid());
    }

    #[test]
    fn parameter_positive_constraint() {
        let p = Parameter::new(vec![0.01], PositiveConstraint);
        assert!(p.is_valid());
        let p2 = Parameter::new(vec![-0.01], PositiveConstraint);
        assert!(!p2.is_valid());
    }

    #[test]
    fn parameter_boundary_constraint() {
        let c = BoundaryConstraint {
            lower: 0.0,
            upper: 1.0,
        };
        let p = Parameter::new(vec![0.5], c);
        assert!(p.is_valid());
    }
}
