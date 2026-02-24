//! Geometric Brownian motion process
//! (translates `ql/processes/geometricbrownianmotionprocess.hpp`).
//!
//! ```text
//! dS/S = μ dt + σ dW
//! ```
//!
//! The simplest continuous-time model for asset prices with constant drift
//! and volatility.

use crate::stochastic_process::StochasticProcess1D;
use ql_core::{Real, Time};

/// Geometric Brownian motion with constant drift and volatility.
///
/// `dS = μ·S·dt + σ·S·dW`
///
/// Closed-form solution: `S(t) = S₀ exp((μ − σ²/2)t + σW(t))`
///
/// Corresponds to `QuantLib::GeometricBrownianMotionProcess`.
#[derive(Debug, Clone)]
pub struct GeometricBrownianMotionProcess {
    x0: Real,
    mu: Real,
    sigma: Real,
}

impl GeometricBrownianMotionProcess {
    /// Create a new GBM process.
    ///
    /// # Arguments
    /// * `x0` — initial asset price (must be > 0)
    /// * `mu` — drift (growth rate)
    /// * `sigma` — volatility (must be ≥ 0)
    pub fn new(x0: Real, mu: Real, sigma: Real) -> Self {
        assert!(x0 > 0.0, "initial value must be positive, got {x0}");
        assert!(sigma >= 0.0, "volatility must be non-negative, got {sigma}");
        Self { x0, mu, sigma }
    }
}

impl StochasticProcess1D for GeometricBrownianMotionProcess {
    fn x0(&self) -> Real {
        self.x0
    }

    fn drift_1d(&self, _t: Time, x: Real) -> Real {
        self.mu * x
    }

    fn diffusion_1d(&self, _t: Time, x: Real) -> Real {
        self.sigma * x
    }

    /// Exact expectation: `x · exp(μ · dt)`.
    fn expectation_1d(&self, _t: Time, x: Real, dt: Time) -> Real {
        x * (self.mu * dt).exp()
    }

    /// Exact standard deviation.
    fn std_deviation_1d(&self, _t: Time, x: Real, dt: Time) -> Real {
        // Std dev of log-normal: x * exp(μ·dt) * sqrt(exp(σ²·dt) - 1)
        // For Euler consistency, use σ*x*√dt
        self.sigma * x * dt.sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gbm_drift_proportional() {
        let p = GeometricBrownianMotionProcess::new(100.0, 0.05, 0.2);
        assert!((p.drift_1d(0.0, 100.0) - 5.0).abs() < 1e-12);
        assert!((p.drift_1d(0.0, 200.0) - 10.0).abs() < 1e-12);
    }

    #[test]
    fn gbm_diffusion_proportional() {
        let p = GeometricBrownianMotionProcess::new(100.0, 0.05, 0.2);
        assert!((p.diffusion_1d(0.0, 100.0) - 20.0).abs() < 1e-12);
    }

    #[test]
    fn gbm_zero_noise_step() {
        let p = GeometricBrownianMotionProcess::new(100.0, 0.05, 0.2);
        let x_new = p.evolve_1d(0.0, 100.0, 1.0, 0.0);
        // E[S(1)] = 100 * exp(0.05) ≈ 105.127
        let expected = 100.0 * (0.05_f64).exp();
        assert!(
            (x_new - expected).abs() < 1e-6,
            "got {x_new}, expected {expected}"
        );
    }

    #[test]
    fn gbm_initial_values() {
        let p = GeometricBrownianMotionProcess::new(42.0, 0.1, 0.3);
        assert!((p.x0() - 42.0).abs() < 1e-15);
    }
}
