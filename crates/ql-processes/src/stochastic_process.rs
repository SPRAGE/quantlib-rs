//! `StochasticProcess` — base traits for stochastic processes
//! (translates `ql/stochasticprocess.hpp`).
//!
//! A stochastic process `dX = μ(t,X) dt + σ(t,X) dW` is described by its
//! drift (`μ`), diffusion (`σ`), and an apply method that advances the state.

use ql_core::{Real, Time};
use ql_math::{Array, Matrix};

/// A general multi-dimensional stochastic process.
///
/// Corresponds to `QuantLib::StochasticProcess`.
pub trait StochasticProcess: std::fmt::Debug + Send + Sync {
    /// Number of dimensions.
    fn size(&self) -> usize;

    /// Number of independent Brownian motions driving the process.
    fn factors(&self) -> usize {
        self.size()
    }

    /// Initial value(s) of the process.
    fn initial_values(&self) -> Array;

    /// Drift vector `μ(t, x)`.
    fn drift(&self, t: Time, x: &Array) -> Array;

    /// Diffusion matrix `σ(t, x)`, dimensioned `size() × factors()`.
    fn diffusion(&self, t: Time, x: &Array) -> Matrix;

    /// Expectation `E[x(t+Δt) | x(t)]`.
    ///
    /// Default: first-order Euler `x + μ(t,x)·Δt`.
    fn expectation(&self, t: Time, x: &Array, dt: Time) -> Array {
        let mu = self.drift(t, x);
        let mut result = x.clone();
        for i in 0..self.size() {
            result[i] += mu[i] * dt;
        }
        result
    }

    /// Standard deviation: `σ(t,x) · √Δt`.
    ///
    /// Returns a `size() × factors()` matrix.
    fn std_deviation(&self, t: Time, x: &Array, dt: Time) -> Matrix {
        let sigma = self.diffusion(t, x);
        let sqrt_dt = dt.sqrt();
        let mut result = sigma;
        let (rows, cols) = (result.nrows(), result.ncols());
        for r in 0..rows {
            for c in 0..cols {
                result[(r, c)] *= sqrt_dt;
            }
        }
        result
    }

    /// Apply a change: advance the state by an Euler step.
    ///
    /// `x(t+Δt) = E[x(t+Δt)|x(t)] + σ·√Δt · dw`
    fn evolve(&self, t: Time, x: &Array, dt: Time, dw: &Array) -> Array {
        let e = self.expectation(t, x, dt);
        let s = self.std_deviation(t, x, dt);
        let mut result = e;
        for i in 0..self.size() {
            let mut diffusion_contribution = 0.0;
            for j in 0..self.factors() {
                diffusion_contribution += s[(i, j)] * dw[j];
            }
            result[i] += diffusion_contribution;
        }
        result
    }
}

/// A 1-dimensional stochastic process `dX = μ(t,X) dt + σ(t,X) dW`.
///
/// Provides scalar versions of drift, diffusion, etc.
///
/// Corresponds to `QuantLib::StochasticProcess1D`.
pub trait StochasticProcess1D: StochasticProcess {
    /// Initial value of the process.
    fn x0(&self) -> Real;

    /// 1D drift `μ(t, x)`.
    fn drift_1d(&self, t: Time, x: Real) -> Real;

    /// 1D diffusion `σ(t, x)`.
    fn diffusion_1d(&self, t: Time, x: Real) -> Real;

    /// Expected value `E[x(t+Δt) | x(t) = x]`.
    fn expectation_1d(&self, t: Time, x: Real, dt: Time) -> Real {
        x + self.drift_1d(t, x) * dt
    }

    /// Standard deviation `σ(t,x) · √Δt`.
    fn std_deviation_1d(&self, t: Time, x: Real, dt: Time) -> Real {
        self.diffusion_1d(t, x) * dt.sqrt()
    }

    /// Euler step: `E + σ·√Δt · dw`.
    fn evolve_1d(&self, t: Time, x: Real, dt: Time, dw: Real) -> Real {
        self.expectation_1d(t, x, dt) + self.std_deviation_1d(t, x, dt) * dw
    }

    /// Variance of the process over `Δt`.
    fn variance_1d(&self, t: Time, x: Real, dt: Time) -> Real {
        let s = self.diffusion_1d(t, x);
        s * s * dt
    }
}

/// Blanket implementation: any 1D process is also a multi-dimensional process
/// of size 1.
impl<T: StochasticProcess1D> StochasticProcess for T {
    fn size(&self) -> usize {
        1
    }

    fn factors(&self) -> usize {
        1
    }

    fn initial_values(&self) -> Array {
        Array::from_vec(vec![self.x0()])
    }

    fn drift(&self, t: Time, x: &Array) -> Array {
        Array::from_vec(vec![self.drift_1d(t, x[0])])
    }

    fn diffusion(&self, t: Time, x: &Array) -> Matrix {
        let mut m = Matrix::zeros(1, 1);
        m[(0, 0)] = self.diffusion_1d(t, x[0]);
        m
    }

    fn expectation(&self, t: Time, x: &Array, dt: Time) -> Array {
        Array::from_vec(vec![self.expectation_1d(t, x[0], dt)])
    }

    fn std_deviation(&self, t: Time, x: &Array, dt: Time) -> Matrix {
        let mut m = Matrix::zeros(1, 1);
        m[(0, 0)] = self.std_deviation_1d(t, x[0], dt);
        m
    }

    fn evolve(&self, t: Time, x: &Array, dt: Time, dw: &Array) -> Array {
        Array::from_vec(vec![self.evolve_1d(t, x[0], dt, dw[0])])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A simple test process: dX = 0.05·dt + 0.20·dW  (constant drift & vol)
    #[derive(Debug)]
    struct ConstantProcess {
        x0: Real,
        mu: Real,
        sigma: Real,
    }

    impl StochasticProcess1D for ConstantProcess {
        fn x0(&self) -> Real {
            self.x0
        }

        fn drift_1d(&self, _t: Time, _x: Real) -> Real {
            self.mu
        }

        fn diffusion_1d(&self, _t: Time, _x: Real) -> Real {
            self.sigma
        }
    }

    #[test]
    fn process_1d_size() {
        let p = ConstantProcess { x0: 100.0, mu: 0.05, sigma: 0.20 };
        assert_eq!(p.size(), 1);
        assert_eq!(p.factors(), 1);
    }

    #[test]
    fn process_1d_initial_values() {
        let p = ConstantProcess { x0: 100.0, mu: 0.05, sigma: 0.20 };
        let iv = p.initial_values();
        assert_eq!(iv.len(), 1);
        assert!((iv[0] - 100.0).abs() < 1e-15);
    }

    #[test]
    fn process_1d_euler_step() {
        let p = ConstantProcess { x0: 100.0, mu: 0.05, sigma: 0.20 };
        let dt = 1.0;
        let dw = 0.0; // zero noise
        let x_new = p.evolve_1d(0.0, 100.0, dt, dw);
        // x + μ·Δt + σ·√Δt·0 = 100 + 0.05 = 100.05
        assert!((x_new - 100.05).abs() < 1e-12);
    }

    #[test]
    fn process_1d_evolve_via_array() {
        let p = ConstantProcess { x0: 100.0, mu: 0.05, sigma: 0.20 };
        let x = Array::from_vec(vec![100.0]);
        let dw = Array::from_vec(vec![1.0]); // 1 std dev
        let x_new = p.evolve(0.0, &x, 1.0, &dw);
        // 100 + 0.05*1 + 0.20*1*1 = 100.25
        assert!((x_new[0] - 100.25).abs() < 1e-12);
    }

    #[test]
    fn process_1d_variance() {
        let p = ConstantProcess { x0: 100.0, mu: 0.05, sigma: 0.20 };
        let v = p.variance_1d(0.0, 100.0, 0.25);
        // σ² · Δt = 0.04 * 0.25 = 0.01
        assert!((v - 0.01).abs() < 1e-15);
    }
}
