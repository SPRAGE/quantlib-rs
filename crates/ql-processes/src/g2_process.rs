//! G2++ two-factor Gaussian short-rate process
//! (translates `ql/processes/g2process.hpp`).
//!
//! The G2++ model describes the short rate as:
//!
//! ```text
//! r(t) = x(t) + y(t) + φ(t)
//! dx = −a·x dt + σ dW₁
//! dy = −b·y dt + η dW₂
//! dW₁·dW₂ = ρ dt
//! ```
//!
//! where `φ(t)` is a deterministic shift chosen to fit the initial yield curve.
//!
//! State vector: `[x, y]`

use crate::stochastic_process::StochasticProcess;
use ql_core::{Real, Time};
use ql_math::{Array, Matrix};
use ql_termstructures::YieldTermStructure;
use std::sync::Arc;

/// G2++ two-factor Gaussian short-rate process.
///
/// State: `[x, y]` where `r(t) = x(t) + y(t) + φ(t)`.
///
/// Corresponds to `QuantLib::G2Process`.
#[derive(Debug)]
pub struct G2Process {
    /// Mean-reversion speed of first factor.
    pub a: Real,
    /// Volatility of first factor.
    pub sigma: Real,
    /// Mean-reversion speed of second factor.
    pub b: Real,
    /// Volatility of second factor.
    pub eta: Real,
    /// Correlation between the two factors.
    pub rho: Real,
    #[allow(dead_code)]
    yield_curve: Arc<dyn YieldTermStructure>,
}

impl G2Process {
    /// Create a new G2++ process.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        a: Real,
        sigma: Real,
        b: Real,
        eta: Real,
        rho: Real,
        yield_curve: Arc<dyn YieldTermStructure>,
    ) -> Self {
        assert!(
            (-1.0..=1.0).contains(&rho),
            "correlation must be in [-1, 1], got {rho}"
        );
        Self {
            a,
            sigma,
            b,
            eta,
            rho,
            yield_curve,
        }
    }
}

impl StochasticProcess for G2Process {
    fn size(&self) -> usize {
        2
    }

    fn factors(&self) -> usize {
        2
    }

    fn initial_values(&self) -> Array {
        Array::from_vec(vec![0.0, 0.0])
    }

    fn drift(&self, _t: Time, x: &Array) -> Array {
        Array::from_vec(vec![-self.a * x[0], -self.b * x[1]])
    }

    fn diffusion(&self, _t: Time, _x: &Array) -> Matrix {
        // Cholesky-style factorization for correlated Brownians:
        // [ σ       0           ]
        // [ η·ρ     η·√(1-ρ²)  ]
        let mut m = Matrix::zeros(2, 2);
        m[(0, 0)] = self.sigma;
        m[(1, 0)] = self.eta * self.rho;
        m[(1, 1)] = self.eta * (1.0 - self.rho * self.rho).max(0.0).sqrt();
        m
    }

    /// Exact conditional expectation:
    /// `E[x(t+dt)] = x(t) · exp(-a·dt)`
    /// `E[y(t+dt)] = y(t) · exp(-b·dt)`
    fn expectation(&self, _t: Time, x: &Array, dt: Time) -> Array {
        Array::from_vec(vec![
            x[0] * (-self.a * dt).exp(),
            x[1] * (-self.b * dt).exp(),
        ])
    }

    fn std_deviation(&self, _t: Time, _x: &Array, dt: Time) -> Matrix {
        let sqrt_dt = dt.sqrt();
        let mut m = Matrix::zeros(2, 2);
        m[(0, 0)] = self.sigma * sqrt_dt;
        m[(1, 0)] = self.eta * self.rho * sqrt_dt;
        m[(1, 1)] = self.eta * (1.0 - self.rho * self.rho).max(0.0).sqrt() * sqrt_dt;
        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ql_termstructures::FlatForward;
    use ql_time::{Actual365Fixed, Date};

    fn flat_ts(rate: Real) -> Arc<dyn YieldTermStructure> {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        Arc::new(FlatForward::continuous(ref_date, rate, Actual365Fixed))
    }

    #[test]
    fn g2_initial_values() {
        let p = G2Process::new(0.1, 0.01, 0.2, 0.015, -0.5, flat_ts(0.05));
        let iv = p.initial_values();
        assert!((iv[0]).abs() < 1e-15);
        assert!((iv[1]).abs() < 1e-15);
    }

    #[test]
    fn g2_drift_mean_reverting() {
        let p = G2Process::new(0.1, 0.01, 0.2, 0.015, -0.5, flat_ts(0.05));
        let x = Array::from_vec(vec![0.01, 0.02]);
        let d = p.drift(0.0, &x);
        // dx/dt = -a*x = -0.001
        assert!((d[0] - (-0.001)).abs() < 1e-15);
        // dy/dt = -b*y = -0.004
        assert!((d[1] - (-0.004)).abs() < 1e-15);
    }

    #[test]
    fn g2_diffusion_correlation() {
        let p = G2Process::new(0.1, 0.01, 0.2, 0.015, -0.5, flat_ts(0.05));
        let x = Array::from_vec(vec![0.0, 0.0]);
        let d = p.diffusion(0.0, &x);
        assert!((d[(0, 0)] - 0.01).abs() < 1e-15);
        assert!((d[(1, 0)] - 0.015 * (-0.5)).abs() < 1e-15);
    }

    #[test]
    fn g2_zero_correlation() {
        let rho = 0.0;
        let eta = 0.015;
        let p = G2Process::new(0.1, 0.01, 0.2, eta, rho, flat_ts(0.05));
        let x = Array::from_vec(vec![0.0, 0.0]);
        let d = p.diffusion(0.0, &x);
        // Off-diagonal should be zero
        assert!((d[(1, 0)]).abs() < 1e-15);
        assert!((d[(1, 1)] - eta).abs() < 1e-15);
    }
}
