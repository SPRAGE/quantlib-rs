//! Gaussian Short-Rate (GSR) process
//! (translates `ql/processes/gsrprocess.hpp`).
//!
//! The GSR process is a generalized Hull-White model with
//! **piecewise-constant** mean-reversion and volatility:
//!
//! ```text
//! dx = −a(t)·x dt + σ(t) dW
//! ```
//!
//! where `a(t)` and `σ(t)` are piecewise constant (step functions).
//! The short rate is `r(t) = x(t) + φ(t)` with an appropriate deterministic
//! shift.

use crate::stochastic_process::StochasticProcess1D;
use ql_core::{Real, Time};

/// Gaussian short-rate process with piecewise-constant parameters.
///
/// Corresponds to `QuantLib::GsrProcess`.
#[derive(Debug, Clone)]
pub struct GsrProcess {
    /// Breakpoints for piecewise-constant parameters (sorted).
    times: Vec<Time>,
    /// Mean-reversion `a(t)` for each interval.
    reversion: Vec<Real>,
    /// Volatility `σ(t)` for each interval.
    vols: Vec<Real>,
    /// Initial value.
    x0_val: Real,
}

impl GsrProcess {
    /// Create a new GSR process.
    ///
    /// `times` are interval boundaries (length N). `reversion` and `vols`
    /// have length N+1 (one value per interval, including the final
    /// open-ended interval).
    pub fn new(times: Vec<Time>, reversion: Vec<Real>, vols: Vec<Real>, x0: Real) -> Self {
        assert_eq!(
            reversion.len(),
            times.len() + 1,
            "reversion must have length times.len() + 1"
        );
        assert_eq!(
            vols.len(),
            times.len() + 1,
            "vols must have length times.len() + 1"
        );
        Self {
            times,
            reversion,
            vols,
            x0_val: x0,
        }
    }

    /// Look up the interval index containing time `t`.
    fn index(&self, t: Time) -> usize {
        match self.times.iter().position(|&ti| t < ti) {
            Some(i) => i,
            None => self.times.len(), // last interval
        }
    }

    /// Get mean-reversion at time `t`.
    pub fn a(&self, t: Time) -> Real {
        self.reversion[self.index(t)]
    }

    /// Get volatility at time `t`.
    pub fn sigma(&self, t: Time) -> Real {
        self.vols[self.index(t)]
    }
}

impl StochasticProcess1D for GsrProcess {
    fn x0(&self) -> Real {
        self.x0_val
    }

    fn drift_1d(&self, t: Time, x: Real) -> Real {
        -self.a(t) * x
    }

    fn diffusion_1d(&self, t: Time, _x: Real) -> Real {
        self.sigma(t)
    }

    fn expectation_1d(&self, t: Time, x: Real, dt: Time) -> Real {
        // Simple case: treat a and σ as constant within [t, t+dt]
        let a = self.a(t);
        x * (-a * dt).exp()
    }

    fn std_deviation_1d(&self, t: Time, _x: Real, dt: Time) -> Real {
        let a = self.a(t);
        let s = self.sigma(t);
        if a.abs() < 1e-12 {
            s * dt.sqrt()
        } else {
            s * ((1.0 - (-2.0 * a * dt).exp()) / (2.0 * a)).sqrt()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gsr_constant_parameters() {
        // Constant a=0.1, σ=0.01 everywhere
        let p = GsrProcess::new(vec![], vec![0.1], vec![0.01], 0.0);
        assert!((p.x0() - 0.0).abs() < 1e-15);
        assert!((p.a(5.0) - 0.1).abs() < 1e-15);
        assert!((p.sigma(5.0) - 0.01).abs() < 1e-15);
    }

    #[test]
    fn gsr_piecewise_lookup() {
        // Switch at t=2: a=[0.05, 0.10], σ=[0.01, 0.02]
        let p = GsrProcess::new(vec![2.0], vec![0.05, 0.10], vec![0.01, 0.02], 0.03);
        assert!((p.a(1.0) - 0.05).abs() < 1e-15);
        assert!((p.a(3.0) - 0.10).abs() < 1e-15);
        assert!((p.sigma(1.0) - 0.01).abs() < 1e-15);
        assert!((p.sigma(3.0) - 0.02).abs() < 1e-15);
    }

    #[test]
    fn gsr_drift_and_diffusion() {
        let p = GsrProcess::new(vec![], vec![0.1], vec![0.01], 0.0);
        assert!((p.drift_1d(0.0, 0.05) - (-0.005)).abs() < 1e-15);
        assert!((p.diffusion_1d(0.0, 0.05) - 0.01).abs() < 1e-15);
    }

    #[test]
    fn gsr_expectation_decay() {
        let p = GsrProcess::new(vec![], vec![0.1], vec![0.01], 0.05);
        let e = p.expectation_1d(0.0, 0.05, 1.0);
        let expected = 0.05 * (-0.1_f64).exp();
        assert!((e - expected).abs() < 1e-12);
    }
}
