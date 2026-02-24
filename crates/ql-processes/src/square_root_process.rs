//! Square-root (CIR) process (translates `ql/processes/squarerootprocess.hpp`).
//!
//! ```text
//! dX = a(b − X) dt + σ √X dW
//! ```
//!
//! This is the Cox-Ingersoll-Ross process, used in short-rate models and as
//! the variance process in the Heston model.

use crate::stochastic_process::StochasticProcess1D;
use ql_core::{Real, Time};

/// A square-root (CIR) process.
///
/// `dX = speed · (mean − X) dt + volatility · √X · dW`
///
/// Corresponds to `QuantLib::SquareRootProcess`.
#[derive(Debug, Clone)]
pub struct SquareRootProcess {
    x0: Real,
    speed: Real,
    mean: Real,
    volatility: Real,
}

impl SquareRootProcess {
    /// Create a new square-root (CIR) process.
    ///
    /// # Arguments
    /// * `speed` — mean-reversion speed `a`
    /// * `mean` — long-run level `b`
    /// * `volatility` — volatility `σ`
    /// * `x0` — initial value (must be ≥ 0)
    pub fn new(speed: Real, mean: Real, volatility: Real, x0: Real) -> Self {
        assert!(x0 >= 0.0, "initial value must be non-negative, got {x0}");
        Self {
            x0,
            speed,
            mean,
            volatility,
        }
    }

    /// Mean-reversion speed.
    pub fn speed(&self) -> Real {
        self.speed
    }

    /// Long-run mean level.
    pub fn mean(&self) -> Real {
        self.mean
    }

    /// Volatility.
    pub fn volatility(&self) -> Real {
        self.volatility
    }
}

impl StochasticProcess1D for SquareRootProcess {
    fn x0(&self) -> Real {
        self.x0
    }

    fn drift_1d(&self, _t: Time, x: Real) -> Real {
        self.speed * (self.mean - x)
    }

    fn diffusion_1d(&self, _t: Time, x: Real) -> Real {
        self.volatility * x.max(0.0).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_root_drift_at_mean() {
        let p = SquareRootProcess::new(1.0, 0.04, 0.3, 0.04);
        // At mean, drift should be zero
        assert!((p.drift_1d(0.0, 0.04)).abs() < 1e-15);
    }

    #[test]
    fn square_root_diffusion_at_zero() {
        let p = SquareRootProcess::new(1.0, 0.04, 0.3, 0.0);
        // At x=0, diffusion should be zero (√0 = 0)
        assert!((p.diffusion_1d(0.0, 0.0)).abs() < 1e-15);
    }

    #[test]
    fn square_root_drift_mean_reverting() {
        let p = SquareRootProcess::new(2.0, 0.04, 0.3, 0.01);
        // Below mean: drift > 0 (pushes up)
        assert!(p.drift_1d(0.0, 0.01) > 0.0);
        // Above mean: drift < 0 (pushes down)
        assert!(p.drift_1d(0.0, 0.10) < 0.0);
    }

    #[test]
    fn square_root_euler_step() {
        let p = SquareRootProcess::new(1.0, 0.04, 0.1, 0.04);
        let dt = 0.01;
        let dw = 0.0;
        let x_new = p.evolve_1d(0.0, 0.04, dt, dw);
        // At mean with zero noise: x should stay at 0.04
        assert!((x_new - 0.04).abs() < 1e-10);
    }
}
