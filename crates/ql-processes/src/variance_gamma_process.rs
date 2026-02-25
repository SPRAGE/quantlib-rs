//! Variance Gamma process
//! (translates `ql/processes/variancegammaprocess.hpp`).
//!
//! The Variance Gamma process is a subordinated Brownian motion:
//!
//! ```text
//! X(t) = θ·G(t) + σ·W(G(t))
//! ```
//!
//! where `G(t)` is a Gamma process with unit mean rate and variance rate `ν`,
//! `θ` is the drift of the subordinated BM, and `σ` is its volatility.
//!
//! Applied to asset pricing:
//! ```text
//! ln S(t) = ln S(0) + (r − q + ω)·t + X(t)
//! ```
//! where `ω = ln(1 − θ·ν − σ²·ν/2)/ν` is the martingale correction.

use crate::stochastic_process::StochasticProcess1D;
use ql_core::{Real, Time};
use ql_termstructures::YieldTermStructure;
use std::sync::Arc;

/// Variance Gamma process for asset pricing.
///
/// Corresponds to `QuantLib::VarianceGammaProcess`.
#[derive(Debug)]
pub struct VarianceGammaProcess {
    s0: Real,
    /// Risk-free rate term structure.
    risk_free: Arc<dyn YieldTermStructure>,
    /// Dividend yield term structure.
    dividend: Arc<dyn YieldTermStructure>,
    /// Volatility of the subordinated Brownian motion.
    pub sigma: Real,
    /// Variance rate of the Gamma subordinator.
    pub nu: Real,
    /// Drift of the subordinated Brownian motion.
    pub theta: Real,
}

impl VarianceGammaProcess {
    /// Create a new Variance Gamma process.
    pub fn new(
        s0: Real,
        risk_free: Arc<dyn YieldTermStructure>,
        dividend: Arc<dyn YieldTermStructure>,
        sigma: Real,
        nu: Real,
        theta: Real,
    ) -> Self {
        Self {
            s0,
            risk_free,
            dividend,
            sigma,
            nu,
            theta,
        }
    }

    /// Martingale correction `ω = ln(1 − θν − σ²ν/2)/ν`.
    pub fn omega(&self) -> Real {
        (1.0 - self.theta * self.nu - 0.5 * self.sigma * self.sigma * self.nu).ln() / self.nu
    }

    /// Access the risk-free term structure.
    pub fn risk_free_rate(&self) -> &Arc<dyn YieldTermStructure> {
        &self.risk_free
    }

    /// Access the dividend term structure.
    pub fn dividend_yield(&self) -> &Arc<dyn YieldTermStructure> {
        &self.dividend
    }
}

impl StochasticProcess1D for VarianceGammaProcess {
    fn x0(&self) -> Real {
        self.s0.ln()
    }

    fn drift_1d(&self, t: Time, _x: Real) -> Real {
        let r = self.risk_free.forward_rate_impl(t);
        let q = self.dividend.forward_rate_impl(t);
        r - q + self.omega()
    }

    fn diffusion_1d(&self, _t: Time, _x: Real) -> Real {
        // In the VG model, the diffusion doesn't factorize neatly into
        // σ·√dt because the time-change is stochastic. For Euler-scheme
        // purposes, we return σ (the BM component).
        self.sigma
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
    fn vg_omega() {
        let vg = VarianceGammaProcess::new(
            100.0,
            flat_ts(0.05),
            flat_ts(0.02),
            0.2,   // sigma
            0.25,  // nu
            -0.14, // theta
        );
        let omega = vg.omega();
        // ω = ln(1 - (-0.14)*0.25 - 0.5*0.04*0.25)/0.25
        //   = ln(1 + 0.035 - 0.005)/0.25
        //   = ln(1.03)/0.25
        let expected = (1.0_f64 - (-0.14) * 0.25 - 0.5 * 0.04 * 0.25).ln() / 0.25;
        assert!((omega - expected).abs() < 1e-12);
    }

    #[test]
    fn vg_x0_is_log_spot() {
        let vg = VarianceGammaProcess::new(100.0, flat_ts(0.05), flat_ts(0.02), 0.2, 0.25, -0.14);
        assert!((vg.x0() - 100.0_f64.ln()).abs() < 1e-12);
    }

    #[test]
    fn vg_drift_includes_omega() {
        let vg = VarianceGammaProcess::new(100.0, flat_ts(0.05), flat_ts(0.0), 0.2, 0.25, -0.14);
        let d = vg.drift_1d(0.0, 0.0);
        // r - q + omega = 0.05 - 0.0 + omega
        let expected = 0.05 + vg.omega();
        assert!((d - expected).abs() < 1e-12);
    }
}
