//! Heston stochastic volatility model.
//!
//! Translates `ql/models/equity/hestonmodel.hpp`.
//!
//! ```text
//! dS = (r − q)·S dt + √v·S dW₁
//! dv = κ(θ − v) dt + σ_v √v dW₂
//! dW₁·dW₂ = ρ dt
//! ```
//!
//! This model wraps a `HestonProcess` and provides calibration infrastructure.

use crate::calibrated_model::{
    BoundaryConstraint, CalibratedModel, Parameter, PositiveConstraint,
};
use ql_core::Real;
use ql_processes::HestonProcess;
use ql_termstructures::YieldTermStructure;
use std::sync::Arc;

/// Heston stochastic volatility model with calibration support.
///
/// Corresponds to `QuantLib::HestonModel`.
#[derive(Debug)]
pub struct HestonModel {
    /// The underlying Heston process.
    process: HestonProcess,
    /// Calibration parameters: [κ, θ, σ_v, ρ, v0].
    params: Vec<Parameter>,
}

impl HestonModel {
    /// Create a new Heston model from its process.
    pub fn new(process: HestonProcess) -> Self {
        let params = vec![
            Parameter::new(vec![process.kappa()], PositiveConstraint),     // κ
            Parameter::new(vec![process.theta()], PositiveConstraint),     // θ
            Parameter::new(vec![process.sigma()], PositiveConstraint),   // σ_v
            Parameter::new(
                vec![process.rho()],
                BoundaryConstraint {
                    lower: -1.0,
                    upper: 1.0,
                },
            ),
            Parameter::new(vec![process.v0()], PositiveConstraint),       // v0
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
    ) -> Self {
        let process = HestonProcess::new(s0, v0, risk_free, dividend, kappa, theta, sigma_v, rho);
        Self::new(process)
    }

    /// Access the underlying process.
    pub fn process(&self) -> &HestonProcess {
        &self.process
    }

    /// Initial spot.
    pub fn s0(&self) -> Real {
        self.process.s0()
    }

    /// Initial variance.
    pub fn v0(&self) -> Real {
        self.process.v0()
    }

    /// Mean-reversion speed.
    pub fn kappa(&self) -> Real {
        self.process.kappa()
    }

    /// Long-run variance.
    pub fn theta(&self) -> Real {
        self.process.theta()
    }

    /// Vol-of-vol.
    pub fn sigma(&self) -> Real {
        self.process.sigma()
    }

    /// Spot-vol correlation.
    pub fn rho(&self) -> Real {
        self.process.rho()
    }

    /// Feller condition: `2κθ > σ²`.
    pub fn feller_satisfied(&self) -> bool {
        2.0 * self.process.kappa() * self.process.theta() > self.process.sigma() * self.process.sigma()
    }
}

impl CalibratedModel for HestonModel {
    fn params(&self) -> &[Parameter] {
        &self.params
    }

    fn set_params(&mut self, values: &[Real]) {
        if values.len() >= 5 {
            // HestonProcess fields are private — we need to reconstruct.
            // For now, update our parameter records for calibration readback.
            for (i, val) in values[..5].iter().enumerate() {
                self.params[i].set_values(vec![*val]);
            }
            // Reconstruct process with new parameters
            let risk_free = self.process.risk_free_rate_arc();
            let dividend = self.process.dividend_yield_arc();
            self.process = HestonProcess::new(
                self.process.s0(),
                values[4],  // v0
                risk_free,
                dividend,
                values[0],  // kappa
                values[1],  // theta
                values[2],  // sigma
                values[3],  // rho
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ql_termstructures::FlatForward;
    use ql_time::{Actual365Fixed, Date};

    fn make_heston_model() -> HestonModel {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let r: Arc<dyn YieldTermStructure> =
            Arc::new(FlatForward::continuous(ref_date, 0.05, Actual365Fixed));
        let q: Arc<dyn YieldTermStructure> =
            Arc::new(FlatForward::continuous(ref_date, 0.02, Actual365Fixed));
        HestonModel::from_params(100.0, 0.04, r, q, 1.5, 0.04, 0.3, -0.7)
    }

    #[test]
    fn heston_model_params() {
        let hm = make_heston_model();
        assert_eq!(hm.params().len(), 5);
        assert!((hm.kappa() - 1.5).abs() < 1e-15);
        assert!((hm.rho() - (-0.7)).abs() < 1e-15);
    }

    #[test]
    fn heston_model_feller() {
        let hm = make_heston_model();
        // 2*1.5*0.04 = 0.12 > 0.09 = 0.3² => Feller OK
        assert!(hm.feller_satisfied());
    }

    #[test]
    fn heston_model_set_params() {
        let mut hm = make_heston_model();
        hm.set_params(&[2.0, 0.05, 0.4, -0.8, 0.06]);
        assert!((hm.kappa() - 2.0).abs() < 1e-15);
        assert!((hm.theta() - 0.05).abs() < 1e-15);
        assert!((hm.sigma() - 0.4).abs() < 1e-15);
        assert!((hm.rho() - (-0.8)).abs() < 1e-15);
        assert!((hm.v0() - 0.06).abs() < 1e-15);
    }

    #[test]
    fn heston_model_process_access() {
        let hm = make_heston_model();
        assert!((hm.s0() - 100.0).abs() < 1e-15);
        assert!((hm.process().s0() - 100.0).abs() < 1e-15);
    }
}
