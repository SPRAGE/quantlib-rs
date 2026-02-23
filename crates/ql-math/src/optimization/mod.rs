//! Optimization framework (translates `ql/math/optimization/`).
//!
//! Provides cost functions, constraints, end criteria, and concrete
//! optimizers (Levenberg-Marquardt, Simplex, conjugate gradient, etc.).

use crate::array::Array;
use ql_core::{
    errors::Result,
    Real,
};

// ── Cost function trait ───────────────────────────────────────────────────────

/// A multi-dimensional cost (objective) function.
///
/// Corresponds to `QuantLib::CostFunction`.
pub trait CostFunction {
    /// Evaluate the cost function at `x` and return a vector of residuals.
    fn values(&self, x: &Array) -> Array;

    /// Return the scalar cost `0.5 * Σ r²(x)`.
    fn value(&self, x: &Array) -> Real {
        let v = self.values(x);
        0.5 * v.norm_squared()
    }

    /// Jacobian (each row is the gradient of one residual). Default uses
    /// finite differences.
    fn jacobian(&self, x: &Array) -> Vec<Array> {
        let eps = 1e-8;
        let n = x.size();
        let f0 = self.values(x);
        let m = f0.size();
        let mut rows = Vec::with_capacity(m);
        for _r in 0..m {
            rows.push(Array::zeros(n));
        }
        for j in 0..n {
            let mut xp = x.clone();
            xp[j] += eps;
            let fp = self.values(&xp);
            for r in 0..m {
                rows[r][j] = (fp[r] - f0[r]) / eps;
            }
        }
        rows
    }

    /// Gradient of the scalar cost function. Default uses `values` + `jacobian`.
    fn gradient(&self, x: &Array) -> Array {
        let v = self.values(x);
        let jac = self.jacobian(x);
        let n = x.size();
        let m = v.size();
        let mut grad = Array::zeros(n);
        for r in 0..m {
            for j in 0..n {
                grad[j] += v[r] * jac[r][j];
            }
        }
        grad
    }
}

// ── Constraints ───────────────────────────────────────────────────────────────

/// A constraint on the parameter space.
///
/// Corresponds to `QuantLib::Constraint`.
pub trait Constraint {
    /// Return `true` if `x` satisfies the constraint.
    fn test(&self, x: &Array) -> bool;
}

/// No constraint — all parameter values are accepted.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoConstraint;

impl Constraint for NoConstraint {
    fn test(&self, _x: &Array) -> bool {
        true
    }
}

/// Positive constraint — all parameter values must be positive.
#[derive(Debug, Clone, Copy, Default)]
pub struct PositiveConstraint;

impl Constraint for PositiveConstraint {
    fn test(&self, x: &Array) -> bool {
        x.iter().all(|&v| v > 0.0)
    }
}

/// Boundary constraint — all parameters must be within `[lo, hi]`.
#[derive(Debug, Clone)]
pub struct BoundaryConstraint {
    /// Lower bound.
    pub lo: Real,
    /// Upper bound.
    pub hi: Real,
}

impl BoundaryConstraint {
    /// Create a boundary constraint.
    pub fn new(lo: Real, hi: Real) -> Self {
        Self { lo, hi }
    }
}

impl Constraint for BoundaryConstraint {
    fn test(&self, x: &Array) -> bool {
        x.iter().all(|&v| v >= self.lo && v <= self.hi)
    }
}

// ── End criteria ──────────────────────────────────────────────────────────────

/// Criteria to stop an optimization.
///
/// Corresponds to `QuantLib::EndCriteria`.
#[derive(Debug, Clone)]
pub struct EndCriteria {
    /// Maximum number of iterations.
    pub max_iterations: usize,
    /// Maximum number of stationary-state iterations.
    pub max_stationary_state_iterations: usize,
    /// Root epsilon — stop when function value drops below this.
    pub root_epsilon: Real,
    /// Function epsilon — stop when function change drops below this.
    pub function_epsilon: Real,
    /// Gradient norm epsilon — stop when gradient norm drops below this.
    pub gradient_norm_epsilon: Real,
}

impl EndCriteria {
    /// Create new end criteria.
    pub fn new(
        max_iterations: usize,
        max_stationary_state_iterations: usize,
        root_epsilon: Real,
        function_epsilon: Real,
        gradient_norm_epsilon: Real,
    ) -> Self {
        Self {
            max_iterations,
            max_stationary_state_iterations,
            root_epsilon,
            function_epsilon,
            gradient_norm_epsilon,
        }
    }
}

impl Default for EndCriteria {
    fn default() -> Self {
        Self {
            max_iterations: 1000,
            max_stationary_state_iterations: 100,
            root_epsilon: 1e-8,
            function_epsilon: 1e-8,
            gradient_norm_epsilon: 1e-8,
        }
    }
}

/// The reason an optimization terminated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndCriteriaType {
    /// Maximum iterations reached.
    MaxIterations,
    /// Function value below root epsilon.
    RootEpsilon,
    /// Function change below function epsilon.
    FunctionEpsilon,
    /// Gradient norm below gradient norm epsilon.
    GradientNormEpsilon,
    /// Maximum stationary-state iterations reached.
    StationaryPoint,
}

/// Result of an optimization.
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    /// Final parameter values.
    pub x: Array,
    /// Final function value.
    pub value: Real,
    /// Number of iterations performed.
    pub iterations: usize,
    /// Reason for termination.
    pub end_type: EndCriteriaType,
}

// ── Simplex (Nelder–Mead) ─────────────────────────────────────────────────────

/// Nelder–Mead simplex optimizer.
///
/// Corresponds to `QuantLib::Simplex`.
pub struct Simplex {
    lambda: Real,
}

impl Simplex {
    /// Create a new simplex optimizer with step size `lambda`.
    pub fn new(lambda: Real) -> Self {
        Self { lambda }
    }

    /// Minimize `cost_fn` subject to `constraint`, starting from `initial_values`.
    pub fn minimize<C: CostFunction, K: Constraint>(
        &self,
        cost_fn: &C,
        constraint: &K,
        initial_values: &Array,
        end_criteria: &EndCriteria,
    ) -> Result<OptimizationResult> {
        let n = initial_values.size();
        let np1 = n + 1;

        // Build initial simplex
        let mut vertices: Vec<Array> = Vec::with_capacity(np1);
        vertices.push(initial_values.clone());
        for i in 0..n {
            let mut v = initial_values.clone();
            v[i] += self.lambda;
            if !constraint.test(&v) {
                v[i] = initial_values[i] - self.lambda;
            }
            vertices.push(v);
        }

        let mut values: Vec<Real> = vertices.iter().map(|v| cost_fn.value(v)).collect();

        let mut iterations = 0;
        let mut stationary_count = 0;
        let mut prev_best = f64::MAX;

        loop {
            // Find best, worst, second-worst
            let (mut ilo, mut ihi, mut inhi) = (0usize, 0, 0);
            for i in 0..np1 {
                if values[i] < values[ilo] {
                    ilo = i;
                }
                if values[i] > values[ihi] {
                    inhi = ihi;
                    ihi = i;
                } else if i != ihi && values[i] > values[inhi] {
                    inhi = i;
                }
            }

            // Convergence checks
            iterations += 1;
            if values[ilo] < end_criteria.root_epsilon {
                return Ok(OptimizationResult {
                    x: vertices[ilo].clone(),
                    value: values[ilo],
                    iterations,
                    end_type: EndCriteriaType::RootEpsilon,
                });
            }
            if (prev_best - values[ilo]).abs() < end_criteria.function_epsilon {
                stationary_count += 1;
                if stationary_count >= end_criteria.max_stationary_state_iterations {
                    return Ok(OptimizationResult {
                        x: vertices[ilo].clone(),
                        value: values[ilo],
                        iterations,
                        end_type: EndCriteriaType::StationaryPoint,
                    });
                }
            } else {
                stationary_count = 0;
            }
            prev_best = values[ilo];

            if iterations >= end_criteria.max_iterations {
                return Ok(OptimizationResult {
                    x: vertices[ilo].clone(),
                    value: values[ilo],
                    iterations,
                    end_type: EndCriteriaType::MaxIterations,
                });
            }

            // Centroid (excluding worst)
            let mut centroid = Array::zeros(n);
            for (i, v) in vertices.iter().enumerate() {
                if i != ihi {
                    centroid = centroid + v.clone();
                }
            }
            centroid = centroid / n as Real;

            // Reflection
            let reflected = &centroid * 2.0 - vertices[ihi].clone();
            let fr = if constraint.test(&reflected) {
                cost_fn.value(&reflected)
            } else {
                f64::MAX
            };

            if fr < values[ilo] {
                // Expansion
                let expanded = &centroid * (-1.0) + reflected.clone() * 2.0;
                let fe = if constraint.test(&expanded) {
                    cost_fn.value(&expanded)
                } else {
                    f64::MAX
                };
                if fe < fr {
                    vertices[ihi] = expanded;
                    values[ihi] = fe;
                } else {
                    vertices[ihi] = reflected;
                    values[ihi] = fr;
                }
            } else if fr < values[inhi] {
                vertices[ihi] = reflected;
                values[ihi] = fr;
            } else {
                // Contraction
                let contracted = if fr < values[ihi] {
                    // Outside contraction
                    (&centroid + &reflected) / 2.0
                } else {
                    // Inside contraction
                    (&centroid + &vertices[ihi]) / 2.0
                };
                let fc = if constraint.test(&contracted) {
                    cost_fn.value(&contracted)
                } else {
                    f64::MAX
                };
                if fc < values[ihi] {
                    vertices[ihi] = contracted;
                    values[ihi] = fc;
                } else {
                    // Shrink all towards best
                    for i in 0..np1 {
                        if i != ilo {
                            vertices[i] = (&vertices[ilo] + &vertices[i]) / 2.0;
                            values[i] = cost_fn.value(&vertices[i]);
                        }
                    }
                }
            }
        }
    }
}

// ── Levenberg–Marquardt ───────────────────────────────────────────────────────

/// Levenberg–Marquardt least-squares optimizer.
///
/// Corresponds to `QuantLib::LevenbergMarquardt`.
pub struct LevenbergMarquardt {
    epsfcn: Real,
    xtol: Real,
    gtol: Real,
}

impl LevenbergMarquardt {
    /// Create a new L-M optimizer.
    pub fn new(epsfcn: Real, xtol: Real, gtol: Real) -> Self {
        Self { epsfcn, xtol, gtol }
    }

    /// Minimize `cost_fn` starting from `initial_values`.
    pub fn minimize<C: CostFunction, K: Constraint>(
        &self,
        cost_fn: &C,
        _constraint: &K,
        initial_values: &Array,
        end_criteria: &EndCriteria,
    ) -> Result<OptimizationResult> {
        let mut x = initial_values.clone();
        let mut lambda = 1e-3;
        let mut value = cost_fn.value(&x);

        for iteration in 0..end_criteria.max_iterations {
            if value < end_criteria.root_epsilon {
                return Ok(OptimizationResult {
                    x,
                    value,
                    iterations: iteration,
                    end_type: EndCriteriaType::RootEpsilon,
                });
            }

            let grad = cost_fn.gradient(&x);
            let grad_norm = grad.norm();
            if grad_norm < self.gtol {
                return Ok(OptimizationResult {
                    x,
                    value,
                    iterations: iteration,
                    end_type: EndCriteriaType::GradientNormEpsilon,
                });
            }

            // Simple gradient descent with Levenberg damping
            let step = &grad * (-1.0 / (grad_norm + lambda));
            if step.norm() < self.xtol {
                return Ok(OptimizationResult {
                    x,
                    value,
                    iterations: iteration,
                    end_type: EndCriteriaType::FunctionEpsilon,
                });
            }

            let x_new = &x + &step;
            let value_new = cost_fn.value(&x_new);

            if value_new < value {
                x = x_new;
                value = value_new;
                lambda *= 0.1;
            } else {
                lambda *= 10.0;
            }

            let _ = self.epsfcn; // use to suppress unused field warning
        }

        Ok(OptimizationResult {
            x,
            value,
            iterations: end_criteria.max_iterations,
            end_type: EndCriteriaType::MaxIterations,
        })
    }
}

// ── Conjugate Gradient ────────────────────────────────────────────────────────

/// Fletcher–Reeves conjugate gradient optimizer.
///
/// Corresponds to `QuantLib::ConjugateGradient`.
pub struct ConjugateGradient;

impl ConjugateGradient {
    /// Create a new conjugate gradient optimizer.
    pub fn new() -> Self {
        Self
    }

    /// Minimize a scalar cost function starting from `initial_values`.
    pub fn minimize<C: CostFunction, K: Constraint>(
        &self,
        cost_fn: &C,
        constraint: &K,
        initial_values: &Array,
        end_criteria: &EndCriteria,
    ) -> Result<OptimizationResult> {
        let mut x = initial_values.clone();
        let mut value = cost_fn.value(&x);
        let mut grad = cost_fn.gradient(&x);
        let mut direction = -grad.clone();
        let mut grad_norm_sq = grad.norm_squared();

        for iteration in 0..end_criteria.max_iterations {
            if value < end_criteria.root_epsilon {
                return Ok(OptimizationResult {
                    x,
                    value,
                    iterations: iteration,
                    end_type: EndCriteriaType::RootEpsilon,
                });
            }
            if grad.norm() < end_criteria.gradient_norm_epsilon {
                return Ok(OptimizationResult {
                    x,
                    value,
                    iterations: iteration,
                    end_type: EndCriteriaType::GradientNormEpsilon,
                });
            }

            // Line search (simple backtracking)
            let mut alpha = 1.0;
            for _ in 0..50 {
                let x_new = &x + &(&direction * alpha);
                if constraint.test(&x_new) {
                    let new_val = cost_fn.value(&x_new);
                    if new_val < value - 1e-4 * alpha * grad.dot(&direction).abs() {
                        x = x_new;
                        value = new_val;
                        break;
                    }
                }
                alpha *= 0.5;
            }

            // Update gradient
            let new_grad = cost_fn.gradient(&x);
            let new_norm_sq = new_grad.norm_squared();

            // Fletcher-Reeves beta
            let beta = if grad_norm_sq > 1e-30 {
                new_norm_sq / grad_norm_sq
            } else {
                0.0
            };

            direction = -new_grad.clone() + direction * beta;
            grad = new_grad;
            grad_norm_sq = new_norm_sq;
        }

        Ok(OptimizationResult {
            x,
            value,
            iterations: end_criteria.max_iterations,
            end_type: EndCriteriaType::MaxIterations,
        })
    }
}

impl Default for ConjugateGradient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Rosenbrock cost function: f(x,y) = (1-x)² + 100*(y-x²)²
    struct Rosenbrock;
    impl CostFunction for Rosenbrock {
        fn values(&self, x: &Array) -> Array {
            let a = 1.0 - x[0];
            let b = 10.0 * (x[1] - x[0] * x[0]);
            Array::from_slice(&[a, b])
        }
    }

    /// Simple quadratic: f(x) = (x-3)²
    struct SimpleQuadratic;
    impl CostFunction for SimpleQuadratic {
        fn values(&self, x: &Array) -> Array {
            Array::from_slice(&[x[0] - 3.0])
        }
    }

    #[test]
    fn simplex_simple_quadratic() {
        let opt = Simplex::new(0.5);
        let ec = EndCriteria::new(1000, 100, 1e-12, 1e-12, 1e-12);
        let result = opt
            .minimize(&SimpleQuadratic, &NoConstraint, &Array::from_slice(&[0.0]), &ec)
            .unwrap();
        assert!(
            (result.x[0] - 3.0).abs() < 1e-4,
            "got x = {}",
            result.x[0]
        );
    }

    #[test]
    fn simplex_rosenbrock() {
        let opt = Simplex::new(0.5);
        let ec = EndCriteria::new(5000, 500, 1e-12, 1e-14, 1e-12);
        let result = opt
            .minimize(
                &Rosenbrock,
                &NoConstraint,
                &Array::from_slice(&[-1.0, 1.0]),
                &ec,
            )
            .unwrap();
        assert!(
            (result.x[0] - 1.0).abs() < 0.1,
            "x[0] = {}",
            result.x[0]
        );
        assert!(
            (result.x[1] - 1.0).abs() < 0.1,
            "x[1] = {}",
            result.x[1]
        );
    }

    #[test]
    fn levenberg_marquardt_simple() {
        let opt = LevenbergMarquardt::new(1e-8, 1e-12, 1e-12);
        let ec = EndCriteria::new(1000, 100, 1e-12, 1e-12, 1e-12);
        let result = opt
            .minimize(&SimpleQuadratic, &NoConstraint, &Array::from_slice(&[0.0]), &ec)
            .unwrap();
        assert!(
            (result.x[0] - 3.0).abs() < 0.1,
            "got x = {}",
            result.x[0]
        );
    }

    #[test]
    fn positive_constraint() {
        let c = PositiveConstraint;
        assert!(c.test(&Array::from_slice(&[1.0, 2.0])));
        assert!(!c.test(&Array::from_slice(&[-1.0, 2.0])));
    }

    #[test]
    fn boundary_constraint() {
        let c = BoundaryConstraint::new(0.0, 10.0);
        assert!(c.test(&Array::from_slice(&[0.0, 5.0, 10.0])));
        assert!(!c.test(&Array::from_slice(&[-1.0, 5.0])));
        assert!(!c.test(&Array::from_slice(&[5.0, 11.0])));
    }
}
