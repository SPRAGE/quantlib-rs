//! Bates stochastic-volatility jump-diffusion process
//! (translates `ql/processes/batesprocess.hpp`).
//!
//! Extends the Heston model with Merton-style log-normal jumps in the asset:
//!
//! ```text
//! dS/S = (r − q − λ·k) dt + √v dW₁ + J dN
//! dv   = κ(θ − v) dt + σ √v dW₂
//! ```
//!
//! where `λ` is jump intensity, `J` is the log-jump (normal with mean `δ`,
//! std dev `ν`), and `k = exp(δ + ν²/2) − 1`.

use crate::heston_process::HestonProcess;
use crate::stochastic_process::StochasticProcess;
use ql_core::{Real, Time};
use ql_math::{Array, Matrix};
use ql_termstructures::YieldTermStructure;
use std::sync::Arc;

/// A Bates jump-diffusion stochastic volatility process.
///
/// This is a Heston process plus Merton-style jumps in the asset price.
///
/// Corresponds to `QuantLib::BatesProcess`.
#[derive(Debug)]
pub struct BatesProcess {
    /// The underlying Heston process (handles the stochastic vol part).
    heston: HestonProcess,
    /// Jump intensity λ.
    pub lambda: Real,
    /// Mean of log-jump size δ.
    pub delta: Real,
    /// Vol of log-jump size ν.
    pub nu: Real,
}

impl BatesProcess {
    /// Create a new Bates process.
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
        lambda: Real,
        delta: Real,
        nu: Real,
    ) -> Self {
        let heston = HestonProcess::new(
            s0,
            v0,
            risk_free_rate,
            dividend_yield,
            kappa,
            theta,
            sigma,
            rho,
        );
        Self {
            heston,
            lambda,
            delta,
            nu,
        }
    }

    /// Jump compensator: k = exp(δ + ν²/2) − 1.
    pub fn jump_compensator(&self) -> Real {
        (self.delta + 0.5 * self.nu * self.nu).exp() - 1.0
    }

    /// Access the underlying Heston process.
    pub fn heston(&self) -> &HestonProcess {
        &self.heston
    }

    /// Spot price.
    pub fn s0(&self) -> Real {
        self.heston.s0()
    }

    /// Initial variance.
    pub fn v0(&self) -> Real {
        self.heston.v0()
    }
}

impl StochasticProcess for BatesProcess {
    fn size(&self) -> usize {
        2
    }

    fn factors(&self) -> usize {
        2
    }

    fn initial_values(&self) -> Array {
        self.heston.initial_values()
    }

    fn drift(&self, t: Time, x: &Array) -> Array {
        // The drift is the same as Heston's except the asset drift is reduced
        // by the jump compensator: (r - q - λ·k) · S
        let mut d = self.heston.drift(t, x);
        let s = x[0];
        d[0] -= self.lambda * self.jump_compensator() * s;
        d
    }

    fn diffusion(&self, t: Time, x: &Array) -> Matrix {
        // Diffusion is the same as Heston.
        self.heston.diffusion(t, x)
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
    fn bates_reduces_to_heston() {
        // With λ=0, Bates should behave exactly like Heston
        let bp = BatesProcess::new(
            100.0,
            0.04,
            flat_ts(0.05),
            flat_ts(0.02),
            1.0,
            0.04,
            0.3,
            -0.5,
            0.0,
            0.0,
            0.0, // no jumps
        );
        let x = Array::from_vec(vec![100.0, 0.04]);
        let drift = bp.drift(0.0, &x);
        // With λ=0, Bates drift[0] = Heston drift[0]
        let h_drift = bp.heston().drift(0.0, &x);
        assert!((drift[0] - h_drift[0]).abs() < 1e-12);
    }

    #[test]
    fn bates_compensator() {
        let bp = BatesProcess::new(
            100.0,
            0.04,
            flat_ts(0.05),
            flat_ts(0.02),
            1.0,
            0.04,
            0.3,
            -0.5,
            1.0,
            -0.1,
            0.15,
        );
        let k = bp.jump_compensator();
        let expected = (-0.1 + 0.5 * 0.0225_f64).exp() - 1.0;
        assert!((k - expected).abs() < 1e-10);
    }

    #[test]
    fn bates_size_and_factors() {
        let bp = BatesProcess::new(
            100.0,
            0.04,
            flat_ts(0.05),
            flat_ts(0.02),
            1.0,
            0.04,
            0.3,
            -0.5,
            1.0,
            0.0,
            0.1,
        );
        assert_eq!(bp.size(), 2);
        assert_eq!(bp.factors(), 2);
    }
}
