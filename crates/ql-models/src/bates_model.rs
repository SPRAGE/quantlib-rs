//! Bates stochastic volatility with jumps model.
//!
//! Translates `ql/models/equity/batesmodel.hpp`.
//!
//! Extends the Heston model with log-normal jumps in the spot process:
//!
//! ```text
//! dS/S = (r − q − λ·k) dt + √v dW₁ + J·dN(λ)
//! dv = κ(θ − v) dt + σ_v √v dW₂
//! ```
//!
//! where `J ~ N(ν, δ²)` is the log-jump size and `k = E[e^J − 1]`.

use crate::calibrated_model::{
    BoundaryConstraint, CalibratedModel, Parameter, PositiveConstraint,
};
use ql_core::Real;
use ql_processes::BatesProcess;
use ql_termstructures::YieldTermStructure;
use std::sync::Arc;

/// Bates stochastic-volatility jump-diffusion model.
///
/// Corresponds to `QuantLib::BatesModel`.
#[derive(Debug)]
pub struct BatesModel {
    /// The underlying Bates process.
    process: BatesProcess,
    /// Calibration parameters: [κ, θ, σ_v, ρ, v0, λ, ν, δ].
    params: Vec<Parameter>,
}

impl BatesModel {
    /// Create a new Bates model from its process.
    pub fn new(process: BatesProcess) -> Self {
        let h = process.heston();
        let params = vec![
            Parameter::new(vec![h.kappa()], PositiveConstraint),
            Parameter::new(vec![h.theta()], PositiveConstraint),
            Parameter::new(vec![h.sigma()], PositiveConstraint),
            Parameter::new(
                vec![h.rho()],
                BoundaryConstraint {
                    lower: -1.0,
                    upper: 1.0,
                },
            ),
            Parameter::new(vec![h.v0()], PositiveConstraint),
            Parameter::new(vec![process.lambda], PositiveConstraint),
            Parameter::constant(process.nu),
            Parameter::new(vec![process.delta], PositiveConstraint),
        ];
        Self { process, params }
    }

    /// Create from individual parameters.
    #[allow(clippy::too_many_arguments)]
    pub fn from_params(
        s0: Real,
        v0: Real,
        risk_free: Arc<dyn YieldTermStructure>,
        dividend: Arc<dyn YieldTermStructure>,
        kappa: Real,
        theta: Real,
        sigma_v: Real,
        rho: Real,
        lambda: Real,
        delta: Real,
        nu: Real,
    ) -> Self {
        let process = BatesProcess::new(
            s0, v0, risk_free, dividend, kappa, theta, sigma_v, rho, lambda, delta, nu,
        );
        Self::new(process)
    }

    /// Access the underlying process.
    pub fn process(&self) -> &BatesProcess {
        &self.process
    }

    /// Jump intensity λ.
    pub fn lambda(&self) -> Real {
        self.process.lambda
    }

    /// Mean of log-jump size ν.
    pub fn nu(&self) -> Real {
        self.process.nu
    }

    /// Volatility of log-jump size δ.
    pub fn delta(&self) -> Real {
        self.process.delta
    }
}

impl CalibratedModel for BatesModel {
    fn params(&self) -> &[Parameter] {
        &self.params
    }

    fn set_params(&mut self, values: &[Real]) {
        if values.len() >= 8 {
            // Update underlying Heston params via the process
            // Since BatesProcess wraps HestonProcess, we'd need mutable access.
            // For now, store in our params for calibration readback.
            for (i, val) in values[..8].iter().enumerate() {
                self.params[i].set_values(vec![*val]);
            }
            self.process.lambda = values[5];
            self.process.nu = values[6];
            self.process.delta = values[7];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ql_termstructures::FlatForward;
    use ql_time::{Actual365Fixed, Date};

    fn make_bates() -> BatesModel {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let r: Arc<dyn YieldTermStructure> =
            Arc::new(FlatForward::continuous(ref_date, 0.05, Actual365Fixed));
        let q: Arc<dyn YieldTermStructure> =
            Arc::new(FlatForward::continuous(ref_date, 0.02, Actual365Fixed));
        BatesModel::from_params(100.0, 0.04, r, q, 1.5, 0.04, 0.3, -0.7, 0.5, -0.1, 0.15)
    }

    #[test]
    fn bates_model_params() {
        let bm = make_bates();
        assert_eq!(bm.params().len(), 8);
        assert!((bm.lambda() - 0.5).abs() < 1e-15);
        // from_params(lambda=0.5, delta=-0.1, nu=0.15)
        assert!((bm.delta() - (-0.1)).abs() < 1e-15);
        assert!((bm.nu() - 0.15).abs() < 1e-15);
    }

    #[test]
    fn bates_model_set_jump_params() {
        let mut bm = make_bates();
        bm.set_params(&[1.5, 0.04, 0.3, -0.7, 0.04, 1.0, -0.2, 0.3]);
        assert!((bm.lambda() - 1.0).abs() < 1e-15);
        assert!((bm.nu() - (-0.2)).abs() < 1e-15);
    }
}
