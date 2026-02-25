//! BFGS and Steepest-Descent optimizers
//! (translates `ql/math/optimization/bfgs.hpp` and
//!  `ql/math/optimization/steepestdescent.hpp`).
//!
//! Also includes Differential Evolution for global optimization.

use crate::array::Array;
use crate::optimization::{
    Constraint, CostFunction, EndCriteria, EndCriteriaType, OptimizationResult,
};
use ql_core::{errors::Result, Real};

// ── BFGS ──────────────────────────────────────────────────────────────────────

/// BFGS (Broyden–Fletcher–Goldfarb–Shanno) quasi-Newton optimizer.
///
/// Maintains an approximation to the inverse Hessian that is updated at each
/// step, giving super-linear convergence near a minimum.
///
/// Corresponds to `QuantLib::BFGS`.
pub struct Bfgs;

impl Bfgs {
    /// Create a new BFGS optimizer.
    pub fn new() -> Self {
        Self
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
        let mut x = initial_values.clone();
        let mut value = cost_fn.value(&x);
        let mut grad = cost_fn.gradient(&x);

        // Initialize inverse Hessian approximation as identity
        let mut h_inv = identity_matrix(n);

        let mut prev_value = value;
        let mut stationary_count = 0;

        for iteration in 0..end_criteria.max_iterations {
            // Check convergence
            if value < end_criteria.root_epsilon {
                return Ok(OptimizationResult {
                    x,
                    value,
                    iterations: iteration,
                    end_type: EndCriteriaType::RootEpsilon,
                });
            }

            let grad_norm = array_norm(&grad);
            if grad_norm < end_criteria.gradient_norm_epsilon {
                return Ok(OptimizationResult {
                    x,
                    value,
                    iterations: iteration,
                    end_type: EndCriteriaType::GradientNormEpsilon,
                });
            }

            if (prev_value - value).abs() < end_criteria.function_epsilon {
                stationary_count += 1;
                if stationary_count >= end_criteria.max_stationary_state_iterations {
                    return Ok(OptimizationResult {
                        x,
                        value,
                        iterations: iteration,
                        end_type: EndCriteriaType::StationaryPoint,
                    });
                }
            } else {
                stationary_count = 0;
            }
            prev_value = value;

            // Search direction: p = -H⁻¹ · ∇f
            let direction = mat_vec_mul(&h_inv, &grad, n);
            let direction = negate(&direction);

            // Backtracking line search (Armijo condition)
            let directional_deriv = array_dot(&grad, &direction);
            let mut alpha = 1.0;
            let mut x_new = array_add(&x, &array_scale(&direction, alpha));

            for _ in 0..50 {
                if constraint.test(&x_new) {
                    let new_val = cost_fn.value(&x_new);
                    if new_val <= value + 1e-4 * alpha * directional_deriv {
                        break;
                    }
                }
                alpha *= 0.5;
                x_new = array_add(&x, &array_scale(&direction, alpha));
            }

            let new_grad = cost_fn.gradient(&x_new);

            // BFGS update of the inverse Hessian approximation
            let s = array_sub(&x_new, &x); // s = x_new - x
            let y = array_sub(&new_grad, &grad); // y = ∇f_new - ∇f

            let sy = array_dot(&s, &y);
            if sy > 1e-30 {
                // H⁻¹_new = (I - s·yᵀ/sᵀy) · H⁻¹ · (I - y·sᵀ/sᵀy) + s·sᵀ/sᵀy
                let rho = 1.0 / sy;
                update_inverse_hessian(&mut h_inv, &s, &y, rho, n);
            }

            x = x_new;
            value = cost_fn.value(&x);
            grad = new_grad;
        }

        Ok(OptimizationResult {
            x,
            value,
            iterations: end_criteria.max_iterations,
            end_type: EndCriteriaType::MaxIterations,
        })
    }
}

impl Default for Bfgs {
    fn default() -> Self {
        Self::new()
    }
}

// ── Steepest Descent ──────────────────────────────────────────────────────────

/// Steepest descent (gradient descent) optimizer.
///
/// Corresponds to `QuantLib::SteepestDescent`.
pub struct SteepestDescent;

impl SteepestDescent {
    /// Create a new steepest descent optimizer.
    pub fn new() -> Self {
        Self
    }

    /// Minimize `cost_fn` subject to `constraint`, starting from `initial_values`.
    pub fn minimize<C: CostFunction, K: Constraint>(
        &self,
        cost_fn: &C,
        constraint: &K,
        initial_values: &Array,
        end_criteria: &EndCriteria,
    ) -> Result<OptimizationResult> {
        let mut x = initial_values.clone();
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
            let grad_norm = array_norm(&grad);
            if grad_norm < end_criteria.gradient_norm_epsilon {
                return Ok(OptimizationResult {
                    x,
                    value,
                    iterations: iteration,
                    end_type: EndCriteriaType::GradientNormEpsilon,
                });
            }

            // Normalized steepest descent direction
            let direction = array_scale(&grad, -1.0 / grad_norm);

            // Backtracking line search
            let mut alpha = 1.0;
            for _ in 0..50 {
                let x_new = array_add(&x, &array_scale(&direction, alpha));
                if constraint.test(&x_new) {
                    let new_val = cost_fn.value(&x_new);
                    if new_val < value - 1e-4 * alpha * grad_norm {
                        x = x_new;
                        value = new_val;
                        break;
                    }
                }
                alpha *= 0.5;
            }
        }

        Ok(OptimizationResult {
            x,
            value,
            iterations: end_criteria.max_iterations,
            end_type: EndCriteriaType::MaxIterations,
        })
    }
}

impl Default for SteepestDescent {
    fn default() -> Self {
        Self::new()
    }
}

// ── Differential Evolution ────────────────────────────────────────────────────

/// Differential Evolution global optimizer.
///
/// A population-based, derivative-free optimizer that works well for
/// non-smooth or multi-modal cost functions.
///
/// Corresponds to `QuantLib::DifferentialEvolution`.
pub struct DifferentialEvolution {
    population_size: usize,
    crossover_prob: Real,
    differential_weight: Real,
    seed: u64,
}

impl DifferentialEvolution {
    /// Create a new Differential Evolution optimizer.
    ///
    /// * `population_size` — size of the candidate pool (typically 5–10× dimensionality)
    /// * `crossover_prob` — probability of crossover (typically 0.5–0.9)
    /// * `differential_weight` — scale factor for difference vectors (typically 0.5–1.0)
    pub fn new(population_size: usize, crossover_prob: Real, differential_weight: Real) -> Self {
        Self {
            population_size,
            crossover_prob,
            differential_weight,
            seed: 42,
        }
    }

    /// Set the random seed.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
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
        let np = self.population_size.max(4); // need at least 4 for DE

        // Simple LCG for reproducibility
        let mut rng_state = self.seed;

        // Inline RNG helpers to avoid borrow conflicts
        #[inline]
        fn lcg_f64(state: &mut u64) -> f64 {
            *state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (*state >> 33) as f64 / (1u64 << 31) as f64
        }
        #[inline]
        fn lcg_usize(state: &mut u64, max: usize) -> usize {
            (lcg_f64(state) * max as f64) as usize % max
        }

        // Initialize population around initial_values
        let mut population: Vec<Array> = Vec::with_capacity(np);
        population.push(initial_values.clone());
        for _ in 1..np {
            let mut candidate = initial_values.clone();
            for j in 0..n {
                candidate[j] += (lcg_f64(&mut rng_state) - 0.5) * 2.0;
            }
            population.push(candidate);
        }

        let mut costs: Vec<Real> = population.iter().map(|p| cost_fn.value(p)).collect();

        for iteration in 0..end_criteria.max_iterations {
            let best_idx = costs
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .unwrap()
                .0;

            if costs[best_idx] < end_criteria.root_epsilon {
                return Ok(OptimizationResult {
                    x: population[best_idx].clone(),
                    value: costs[best_idx],
                    iterations: iteration,
                    end_type: EndCriteriaType::RootEpsilon,
                });
            }

            for i in 0..np {
                // Pick 3 distinct random members (different from i)
                let mut r1 = lcg_usize(&mut rng_state, np);
                while r1 == i {
                    r1 = lcg_usize(&mut rng_state, np);
                }
                let mut r2 = lcg_usize(&mut rng_state, np);
                while r2 == i || r2 == r1 {
                    r2 = lcg_usize(&mut rng_state, np);
                }
                let mut r3 = lcg_usize(&mut rng_state, np);
                while r3 == i || r3 == r1 || r3 == r2 {
                    r3 = lcg_usize(&mut rng_state, np);
                }

                // Mutant vector
                let mut trial = population[i].clone();
                let j_rand = lcg_usize(&mut rng_state, n);

                for j in 0..n {
                    if lcg_f64(&mut rng_state) < self.crossover_prob || j == j_rand {
                        trial[j] = population[r1][j]
                            + self.differential_weight * (population[r2][j] - population[r3][j]);
                    }
                }

                if constraint.test(&trial) {
                    let trial_cost = cost_fn.value(&trial);
                    if trial_cost <= costs[i] {
                        population[i] = trial;
                        costs[i] = trial_cost;
                    }
                }
            }
        }

        let best_idx = costs
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap()
            .0;

        Ok(OptimizationResult {
            x: population[best_idx].clone(),
            value: costs[best_idx],
            iterations: end_criteria.max_iterations,
            end_type: EndCriteriaType::MaxIterations,
        })
    }
}

// ── Helper functions for dense vector/matrix ops on flat Vec<Real> ────────────

fn identity_matrix(n: usize) -> Vec<Real> {
    let mut m = vec![0.0; n * n];
    for i in 0..n {
        m[i * n + i] = 1.0;
    }
    m
}

fn mat_vec_mul(m: &[Real], v: &Array, n: usize) -> Array {
    let mut result = Array::zeros(n);
    for i in 0..n {
        let mut sum = 0.0;
        for j in 0..n {
            sum += m[i * n + j] * v[j];
        }
        result[i] = sum;
    }
    result
}

fn array_dot(a: &Array, b: &Array) -> Real {
    a.dot(b)
}

fn array_norm(a: &Array) -> Real {
    a.norm()
}

fn array_add(a: &Array, b: &Array) -> Array {
    a + b
}

fn array_sub(a: &Array, b: &Array) -> Array {
    let mut r = a.clone();
    for i in 0..r.size() {
        r[i] -= b[i];
    }
    r
}

fn array_scale(a: &Array, s: Real) -> Array {
    a * s
}

fn negate(a: &Array) -> Array {
    -a.clone()
}

fn update_inverse_hessian(h: &mut [Real], s: &Array, y: &Array, rho: Real, n: usize) {
    // BFGS formula: H = (I - ρ·s·yᵀ) · H · (I - ρ·y·sᵀ) + ρ·s·sᵀ
    // More efficient to do step by step.

    // temp = H · y
    let mut hy = vec![0.0; n];
    for i in 0..n {
        for j in 0..n {
            hy[i] += h[i * n + j] * y[j];
        }
    }

    // yᵀ · H · y
    let mut yhy = 0.0;
    for i in 0..n {
        yhy += y[i] * hy[i];
    }

    // Sherman-Morrison-Woodbury style update:
    // H_new = H − (H·y·sᵀ + s·yᵀ·H) / (sᵀy) + (1 + yᵀHy/sᵀy) · s·sᵀ / (sᵀy)
    let sy = 1.0 / rho;
    let factor = 1.0 + yhy / sy;

    for i in 0..n {
        for j in 0..n {
            // temp2 = Σ_k H[i,k]*y[k] = hy[i], and s·yᵀ·H row j = Σ_k y[k]*H[k,j]
            let mut yh_j = 0.0;
            for k in 0..n {
                yh_j += y[k] * h[k * n + j];
            }
            h[i * n + j] += rho * (factor * s[i] * s[j] - hy[i] * s[j] - s[i] * yh_j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimization::NoConstraint;

    struct SimpleQuadratic;
    impl CostFunction for SimpleQuadratic {
        fn values(&self, x: &Array) -> Array {
            Array::from_slice(&[x[0] - 3.0])
        }
    }

    struct Rosenbrock;
    impl CostFunction for Rosenbrock {
        fn values(&self, x: &Array) -> Array {
            let a = 1.0 - x[0];
            let b = 10.0 * (x[1] - x[0] * x[0]);
            Array::from_slice(&[a, b])
        }
    }

    #[test]
    fn bfgs_simple_quadratic() {
        let opt = Bfgs::new();
        let ec = EndCriteria::new(1000, 100, 1e-12, 1e-12, 1e-12);
        let result = opt
            .minimize(
                &SimpleQuadratic,
                &NoConstraint,
                &Array::from_slice(&[0.0]),
                &ec,
            )
            .unwrap();
        assert!((result.x[0] - 3.0).abs() < 1e-4, "got x = {}", result.x[0]);
    }

    #[test]
    fn bfgs_rosenbrock() {
        let opt = Bfgs::new();
        let ec = EndCriteria::new(5000, 500, 1e-12, 1e-14, 1e-10);
        let result = opt
            .minimize(
                &Rosenbrock,
                &NoConstraint,
                &Array::from_slice(&[-1.0, 1.0]),
                &ec,
            )
            .unwrap();
        assert!((result.x[0] - 1.0).abs() < 0.1, "x[0] = {}", result.x[0]);
        assert!((result.x[1] - 1.0).abs() < 0.1, "x[1] = {}", result.x[1]);
    }

    #[test]
    fn steepest_descent_simple() {
        let opt = SteepestDescent::new();
        let ec = EndCriteria::new(5000, 100, 1e-12, 1e-12, 1e-12);
        let result = opt
            .minimize(
                &SimpleQuadratic,
                &NoConstraint,
                &Array::from_slice(&[0.0]),
                &ec,
            )
            .unwrap();
        assert!((result.x[0] - 3.0).abs() < 0.1, "got x = {}", result.x[0]);
    }

    #[test]
    fn differential_evolution_simple() {
        let opt = DifferentialEvolution::new(20, 0.7, 0.8).with_seed(42);
        let ec = EndCriteria::new(500, 100, 1e-8, 1e-8, 1e-8);
        let result = opt
            .minimize(
                &SimpleQuadratic,
                &NoConstraint,
                &Array::from_slice(&[0.0]),
                &ec,
            )
            .unwrap();
        assert!((result.x[0] - 3.0).abs() < 0.5, "got x = {}", result.x[0]);
    }
}
