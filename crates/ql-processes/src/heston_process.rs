//! Heston stochastic volatility process
//! (translates `ql/processes/hestonprocess.hpp`).
//!
//! The Heston model describes two coupled SDEs:
//!
//! ```text
//! dS = (r − q) S dt + √v S dW₁
//! dv = κ(θ − v) dt + σ √v dW₂
//! dW₁ dW₂ = ρ dt
//! ```
//!
//! State vector: `x = [S, v]`

use crate::stochastic_process::StochasticProcess;
use ql_core::{Real, Time};
use ql_math::{Array, Matrix};
use ql_termstructures::YieldTermStructure;
use std::sync::Arc;

/// The Heston stochastic volatility process.
///
/// * `v0`    — initial variance
/// * `kappa` — mean-reversion speed of variance
/// * `theta` — long-run variance level
/// * `sigma` — vol-of-vol
/// * `rho`   — correlation between the two Brownian motions
///
/// Corresponds to `QuantLib::HestonProcess`.
#[derive(Debug)]
pub struct HestonProcess {
    s0: Real,
    v0: Real,
    kappa: Real,
    theta: Real,
    sigma: Real,
    rho: Real,
    risk_free_rate: Arc<dyn YieldTermStructure>,
    dividend_yield: Arc<dyn YieldTermStructure>,
}

impl HestonProcess {
    /// Create a new Heston process.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        s0: Real,
        v0: Real,
        risk_free_rate: Arc<dyn YieldTermStructure>,
        dividend_yield: Arc<dyn YieldTermStructure>,
        kappa: Real,
        theta: Real,
        sigma: Real,
        rho: Real,
    ) -> Self {
        assert!(
            (-1.0..=1.0).contains(&rho),
            "correlation ρ must be in [-1, 1], got {rho}"
        );
        assert!(v0 >= 0.0, "initial variance must be non-negative, got {v0}");
        assert!(kappa >= 0.0, "mean reversion speed must be non-negative");
        assert!(theta >= 0.0, "long-run variance must be non-negative");
        assert!(sigma >= 0.0, "vol-of-vol must be non-negative");

        Self {
            s0,
            v0,
            risk_free_rate,
            dividend_yield,
            kappa,
            theta,
            sigma,
            rho,
        }
    }

    /// Spot price.
    pub fn s0(&self) -> Real {
        self.s0
    }

    /// Initial variance.
    pub fn v0(&self) -> Real {
        self.v0
    }

    /// Mean-reversion speed.
    pub fn kappa(&self) -> Real {
        self.kappa
    }

    /// Long-run variance.
    pub fn theta(&self) -> Real {
        self.theta
    }

    /// Vol-of-vol.
    pub fn sigma(&self) -> Real {
        self.sigma
    }

    /// Correlation.
    pub fn rho(&self) -> Real {
        self.rho
    }

    /// Risk-free rate.
    pub fn risk_free_rate(&self) -> &dyn YieldTermStructure {
        &*self.risk_free_rate
    }

    /// Risk-free rate as `Arc`.
    pub fn risk_free_rate_arc(&self) -> Arc<dyn YieldTermStructure> {
        Arc::clone(&self.risk_free_rate)
    }

    /// Dividend yield.
    pub fn dividend_yield(&self) -> &dyn YieldTermStructure {
        &*self.dividend_yield
    }

    /// Dividend yield as `Arc`.
    pub fn dividend_yield_arc(&self) -> Arc<dyn YieldTermStructure> {
        Arc::clone(&self.dividend_yield)
    }
}

impl StochasticProcess for HestonProcess {
    fn size(&self) -> usize {
        2
    }

    fn factors(&self) -> usize {
        2
    }

    fn initial_values(&self) -> Array {
        Array::from_vec(vec![self.s0, self.v0])
    }

    fn drift(&self, t: Time, x: &Array) -> Array {
        let s = x[0];
        let v = x[1].max(0.0); // floor variance at 0 for numerical stability

        let r = self.risk_free_rate.zero_rate_impl(t);
        let q = self.dividend_yield.zero_rate_impl(t);

        Array::from_vec(vec![(r - q) * s, self.kappa * (self.theta - v)])
    }

    fn diffusion(&self, _t: Time, x: &Array) -> Matrix {
        let s = x[0];
        let v = x[1].max(0.0);
        let sqrt_v = v.sqrt();

        // Cholesky decomposition of correlation matrix:
        // L = [[1, 0], [ρ, √(1-ρ²)]]
        // diffusion = diag(σ_S, σ_v) * L
        // σ_S = √v * S,  σ_v = sigma_v * √v (vol-of-vol)
        let mut m = Matrix::zeros(2, 2);
        m[(0, 0)] = sqrt_v * s;
        m[(0, 1)] = 0.0;
        m[(1, 0)] = self.sigma * sqrt_v * self.rho;
        m[(1, 1)] = self.sigma * sqrt_v * (1.0 - self.rho * self.rho).sqrt();
        m
    }

    fn evolve(&self, t: Time, x: &Array, dt: Time, dw: &Array) -> Array {
        // Euler-Maruyama step with full-truncation scheme for variance
        let s = x[0];
        let v = x[1].max(0.0);
        let sqrt_v = v.sqrt();
        let sqrt_dt = dt.sqrt();

        let r = self.risk_free_rate.zero_rate_impl(t);
        let q = self.dividend_yield.zero_rate_impl(t);

        // Evolve log-spot for better accuracy
        let new_s = s * ((r - q - 0.5 * v) * dt + sqrt_v * sqrt_dt * dw[0]).exp();

        // Evolve variance (full truncation)
        let dw_v = self.rho * dw[0] + (1.0 - self.rho * self.rho).sqrt() * dw[1];
        let new_v = v + self.kappa * (self.theta - v) * dt + self.sigma * sqrt_v * sqrt_dt * dw_v;

        Array::from_vec(vec![new_s, new_v.max(0.0)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use ql_termstructures::FlatForward;
    use ql_time::{Actual365Fixed, Date};

    fn make_heston() -> HestonProcess {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let r: Arc<dyn YieldTermStructure> =
            Arc::new(FlatForward::continuous(ref_date, 0.05, Actual365Fixed));
        let q: Arc<dyn YieldTermStructure> =
            Arc::new(FlatForward::continuous(ref_date, 0.02, Actual365Fixed));

        HestonProcess::new(
            100.0, // s0
            0.04,  // v0 (σ=0.2 => v=0.04)
            r, q, 1.5,  // κ
            0.04, // θ
            0.3,  // σ_v
            -0.7, // ρ
        )
    }

    #[test]
    fn heston_size_and_factors() {
        let p = make_heston();
        assert_eq!(p.size(), 2);
        assert_eq!(p.factors(), 2);
    }

    #[test]
    fn heston_initial_values() {
        let p = make_heston();
        let iv = p.initial_values();
        assert_abs_diff_eq!(iv[0], 100.0, epsilon = 1e-15);
        assert_abs_diff_eq!(iv[1], 0.04, epsilon = 1e-15);
    }

    #[test]
    fn heston_drift() {
        let p = make_heston();
        let x = Array::from_vec(vec![100.0, 0.04]);
        let d = p.drift(0.0, &x);
        // drift_s = (r-q)*S = 0.03*100 = 3.0
        assert_abs_diff_eq!(d[0], 3.0, epsilon = 0.01);
        // drift_v = κ(θ-v) = 1.5*(0.04-0.04) = 0.0
        assert_abs_diff_eq!(d[1], 0.0, epsilon = 1e-12);
    }

    #[test]
    fn heston_diffusion_at_mean() {
        let p = make_heston();
        let x = Array::from_vec(vec![100.0, 0.04]);
        let m = p.diffusion(0.0, &x);
        // (0,0) = √v * S = 0.2 * 100 = 20
        assert_abs_diff_eq!(m[(0, 0)], 20.0, epsilon = 1e-10);
        // (0,1) = 0
        assert_abs_diff_eq!(m[(0, 1)], 0.0, epsilon = 1e-15);
        // (1,0) = σ_v * √v * ρ = 0.3 * 0.2 * (-0.7) = -0.042
        assert_abs_diff_eq!(m[(1, 0)], -0.042, epsilon = 1e-10);
        // (1,1) = σ_v * √v * √(1-ρ²) = 0.3 * 0.2 * √(0.51) ≈ 0.04285
        let expected = 0.3 * 0.2 * (1.0 - 0.49_f64).sqrt();
        assert_abs_diff_eq!(m[(1, 1)], expected, epsilon = 1e-10);
    }

    #[test]
    fn heston_evolve_zero_noise() {
        let p = make_heston();
        let x = Array::from_vec(vec![100.0, 0.04]);
        let dt = 1.0 / 252.0;
        let dw = Array::from_vec(vec![0.0, 0.0]);
        let x_new = p.evolve(0.0, &x, dt, &dw);
        // spot should increase slightly: (r-q-v/2)*dt ~ (0.03-0.02)/252 > 0
        assert!(x_new[0] > 99.9 && x_new[0] < 100.1, "s = {}", x_new[0]);
        // variance should stay near 0.04 (at mean, so drift ≈ 0)
        assert_abs_diff_eq!(x_new[1], 0.04, epsilon = 1e-4);
    }

    #[test]
    fn heston_variance_stays_positive() {
        let p = make_heston();
        // Very low variance — should never go negative after evolve
        let x = Array::from_vec(vec![100.0, 0.001]);
        let dt = 1.0 / 252.0;
        let dw = Array::from_vec(vec![-3.0, -3.0]); // extreme negative shock
        let x_new = p.evolve(0.0, &x, dt, &dw);
        assert!(x_new[1] >= 0.0, "variance went negative: {}", x_new[1]);
    }
}
