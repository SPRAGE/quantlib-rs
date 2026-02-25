//! Ornstein-Uhlenbeck mean-reverting process
//! (translates `ql/processes/ornsteinuhlenbeckprocess.hpp`).
//!
//! ```text
//! dX = a(b − X) dt + σ dW
//! ```
//!
//! where `a` is the speed of mean reversion, `b` is the long-run level,
//! and `σ` is the constant volatility.

use crate::stochastic_process::StochasticProcess1D;
use ql_core::{Real, Time};

/// An Ornstein-Uhlenbeck mean-reverting process.
///
/// `dX = speed · (level − X) dt + volatility · dW`
///
/// Closed-form expectation and variance:
/// ```text
/// E[X(t+dt) | X(t)] = level + (X(t) − level) · exp(−speed · dt)
/// Var[X(t+dt) | X(t)] = σ² / (2·speed) · (1 − exp(−2·speed·dt))
/// ```
///
/// Corresponds to `QuantLib::OrnsteinUhlenbeckProcess`.
#[derive(Debug, Clone)]
pub struct OrnsteinUhlenbeckProcess {
    x0: Real,
    speed: Real,
    level: Real,
    volatility: Real,
}

impl OrnsteinUhlenbeckProcess {
    /// Create a new Ornstein-Uhlenbeck process.
    ///
    /// # Arguments
    /// * `speed` — mean-reversion speed `a` (must be ≥ 0)
    /// * `volatility` — diffusion coefficient `σ` (must be ≥ 0)
    /// * `x0` — initial value
    /// * `level` — long-run mean level `b` (default 0 in QuantLib)
    pub fn new(speed: Real, volatility: Real, x0: Real, level: Real) -> Self {
        assert!(
            speed >= 0.0,
            "mean-reversion speed must be non-negative, got {speed}"
        );
        assert!(
            volatility >= 0.0,
            "volatility must be non-negative, got {volatility}"
        );
        Self {
            x0,
            speed,
            level,
            volatility,
        }
    }

    /// Create with default level = 0.
    pub fn new_zero_level(speed: Real, volatility: Real, x0: Real) -> Self {
        Self::new(speed, volatility, x0, 0.0)
    }

    /// Speed of mean reversion.
    pub fn speed(&self) -> Real {
        self.speed
    }

    /// Long-run level.
    pub fn level(&self) -> Real {
        self.level
    }

    /// Volatility.
    pub fn volatility(&self) -> Real {
        self.volatility
    }
}

impl StochasticProcess1D for OrnsteinUhlenbeckProcess {
    fn x0(&self) -> Real {
        self.x0
    }

    fn drift_1d(&self, _t: Time, x: Real) -> Real {
        self.speed * (self.level - x)
    }

    fn diffusion_1d(&self, _t: Time, _x: Real) -> Real {
        self.volatility
    }

    fn expectation_1d(&self, _t: Time, x: Real, dt: Time) -> Real {
        // Exact conditional mean:
        // E[X(t+dt)] = level + (x - level) * exp(-speed * dt)
        self.level + (x - self.level) * (-self.speed * dt).exp()
    }

    fn std_deviation_1d(&self, _t: Time, _x: Real, dt: Time) -> Real {
        // Exact conditional std dev:
        // std = σ * sqrt((1 - exp(-2·a·dt)) / (2·a))
        if self.speed < 1e-15 {
            // Degenerate case: no mean reversion => pure Brownian motion
            self.volatility * dt.sqrt()
        } else {
            self.volatility * ((1.0 - (-2.0 * self.speed * dt).exp()) / (2.0 * self.speed)).sqrt()
        }
    }

    fn variance_1d(&self, _t: Time, _x: Real, dt: Time) -> Real {
        // Exact conditional variance:
        // Var = σ² · (1 - exp(-2·a·dt)) / (2·a)
        if self.speed < 1e-15 {
            self.volatility * self.volatility * dt
        } else {
            self.volatility * self.volatility * (1.0 - (-2.0 * self.speed * dt).exp())
                / (2.0 * self.speed)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn ou_initial_value() {
        let p = OrnsteinUhlenbeckProcess::new(1.0, 0.2, 0.5, 0.0);
        assert_abs_diff_eq!(p.x0(), 0.5, epsilon = 1e-15);
    }

    #[test]
    fn ou_drift() {
        let p = OrnsteinUhlenbeckProcess::new(2.0, 0.3, 0.5, 1.0);
        // drift = speed * (level - x) = 2 * (1.0 - 0.5) = 1.0
        assert_abs_diff_eq!(p.drift_1d(0.0, 0.5), 1.0, epsilon = 1e-15);
    }

    #[test]
    fn ou_diffusion_constant() {
        let p = OrnsteinUhlenbeckProcess::new(1.0, 0.3, 0.5, 0.0);
        // Diffusion is constant σ regardless of state
        assert_abs_diff_eq!(p.diffusion_1d(0.0, 0.5), 0.3, epsilon = 1e-15);
        assert_abs_diff_eq!(p.diffusion_1d(0.0, 100.0), 0.3, epsilon = 1e-15);
    }

    #[test]
    fn ou_expectation_mean_reversion() {
        let p = OrnsteinUhlenbeckProcess::new(1.0, 0.2, 0.5, 0.0);
        // x₀ = 0.5, level = 0 → should revert toward 0
        let dt = 1.0;
        let e = p.expectation_1d(0.0, 0.5, dt);
        // E = 0 + (0.5 - 0) * exp(-1*1) = 0.5 * exp(-1) ≈ 0.1839
        let expected = 0.5 * (-1.0_f64).exp();
        assert_abs_diff_eq!(e, expected, epsilon = 1e-12);
    }

    #[test]
    fn ou_expectation_at_level() {
        let p = OrnsteinUhlenbeckProcess::new(2.0, 0.3, 1.0, 1.0);
        // When x = level, expectation stays at level
        let e = p.expectation_1d(0.0, 1.0, 0.25);
        assert_abs_diff_eq!(e, 1.0, epsilon = 1e-15);
    }

    #[test]
    fn ou_variance_formula() {
        let speed = 2.0;
        let sigma = 0.3;
        let p = OrnsteinUhlenbeckProcess::new(speed, sigma, 0.5, 0.0);
        let dt = 0.5;
        let var = p.variance_1d(0.0, 0.5, dt);
        // Var = σ² * (1 - exp(-2a·dt)) / (2a)
        //     = 0.09 * (1 - exp(-2)) / 4
        let expected = sigma * sigma * (1.0 - (-2.0 * speed * dt).exp()) / (2.0 * speed);
        assert_abs_diff_eq!(var, expected, epsilon = 1e-15);
    }

    #[test]
    fn ou_std_deviation_consistency() {
        let p = OrnsteinUhlenbeckProcess::new(1.5, 0.25, 0.0, 0.0);
        let dt = 0.1;
        let var = p.variance_1d(0.0, 0.0, dt);
        let std = p.std_deviation_1d(0.0, 0.0, dt);
        assert_abs_diff_eq!(std * std, var, epsilon = 1e-14);
    }

    #[test]
    fn ou_zero_speed_degenerates_to_brownian() {
        // When speed = 0, it's just dX = σ dW
        let sigma = 0.3;
        let p = OrnsteinUhlenbeckProcess::new(0.0, sigma, 1.0, 0.0);
        let dt = 0.25;
        // Expectation: x + 0 = x
        let e = p.expectation_1d(0.0, 1.0, dt);
        assert_abs_diff_eq!(e, 1.0, epsilon = 1e-15);
        // Variance: σ² · dt
        let var = p.variance_1d(0.0, 1.0, dt);
        assert_abs_diff_eq!(var, sigma * sigma * dt, epsilon = 1e-15);
    }

    #[test]
    fn ou_evolve_roundtrip() {
        use crate::stochastic_process::StochasticProcess;
        use ql_math::Array;

        let p = OrnsteinUhlenbeckProcess::new(1.0, 0.2, 0.5, 0.0);
        let x = Array::from_vec(vec![0.5]);
        let dw = Array::from_vec(vec![0.0]); // zero noise
        let dt = 1.0;
        let x_new = p.evolve(0.0, &x, dt, &dw);
        let expected = p.expectation_1d(0.0, 0.5, dt);
        assert_abs_diff_eq!(x_new[0], expected, epsilon = 1e-12);
    }
}
