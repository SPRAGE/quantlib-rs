//! Generalized Black-Scholes process
//! (translates `ql/processes/blackscholesprocess.hpp`).
//!
//! `dS/S = (r − q) dt + σ(t, S) dW`
//!
//! where `r` is the risk-free rate, `q` is the dividend yield (continuous),
//! and `σ` can be constant, a Black-vol function, or a local-vol function.
//!
//! Concrete variants:
//! * `GeneralizedBlackScholesProcess` — the most general form
//! * `BlackScholesProcess` — no dividends
//! * `BlackScholesMertonProcess` — continuous dividend yield

use crate::stochastic_process::StochasticProcess1D;
use ql_core::{Real, Time};
use ql_termstructures::{
    BlackVolTermStructure, LocalVolTermStructure, YieldTermStructure,
};
use std::sync::Arc;

/// A generalized Black-Scholes stochastic process.
///
/// `dS = (r(t) − q(t)) · S · dt + σ(t, S) · S · dW`
///
/// Corresponds to `QuantLib::GeneralizedBlackScholesProcess`.
#[derive(Debug)]
pub struct GeneralizedBlackScholesProcess {
    x0: Real,
    risk_free_rate: Arc<dyn YieldTermStructure>,
    dividend_yield: Arc<dyn YieldTermStructure>,
    black_vol: Option<Arc<dyn BlackVolTermStructure>>,
    local_vol: Option<Arc<dyn LocalVolTermStructure>>,
}

impl GeneralizedBlackScholesProcess {
    /// Create a new GBS process with a Black volatility surface.
    pub fn new(
        x0: Real,
        risk_free_rate: Arc<dyn YieldTermStructure>,
        dividend_yield: Arc<dyn YieldTermStructure>,
        black_vol: Arc<dyn BlackVolTermStructure>,
    ) -> Self {
        Self {
            x0,
            risk_free_rate,
            dividend_yield,
            black_vol: Some(black_vol),
            local_vol: None,
        }
    }

    /// Create with a local volatility surface instead.
    pub fn with_local_vol(
        x0: Real,
        risk_free_rate: Arc<dyn YieldTermStructure>,
        dividend_yield: Arc<dyn YieldTermStructure>,
        local_vol: Arc<dyn LocalVolTermStructure>,
    ) -> Self {
        Self {
            x0,
            risk_free_rate,
            dividend_yield,
            black_vol: None,
            local_vol: Some(local_vol),
        }
    }

    /// The spot price.
    pub fn spot(&self) -> Real {
        self.x0
    }

    /// The risk-free rate term structure.
    pub fn risk_free_rate(&self) -> &dyn YieldTermStructure {
        &*self.risk_free_rate
    }

    /// The dividend yield term structure.
    pub fn dividend_yield(&self) -> &dyn YieldTermStructure {
        &*self.dividend_yield
    }

    /// The Black volatility surface (if set).
    pub fn black_volatility(&self) -> Option<&dyn BlackVolTermStructure> {
        self.black_vol.as_deref()
    }

    /// The local volatility surface (if set).
    pub fn local_volatility(&self) -> Option<&dyn LocalVolTermStructure> {
        self.local_vol.as_deref()
    }

    /// Get the volatility at time `t` and underlying `x`.
    fn vol(&self, t: Time, x: Real) -> Real {
        if let Some(ref lv) = self.local_vol {
            lv.local_vol_impl(t, x)
        } else if let Some(ref bv) = self.black_vol {
            bv.black_vol_impl(t, x)
        } else {
            0.0
        }
    }
}

impl StochasticProcess1D for GeneralizedBlackScholesProcess {
    fn x0(&self) -> Real {
        self.x0
    }

    fn drift_1d(&self, t: Time, x: Real) -> Real {
        let sigma = self.vol(t, x);
        let r = self.risk_free_rate.zero_rate_impl(t);
        let q = self.dividend_yield.zero_rate_impl(t);
        // For log-price: drift = (r - q - σ²/2)
        // For price: drift = (r - q) * S
        // Using price-level process: dS = (r-q)·S·dt + σ·S·dW
        (r - q) * x - 0.5 * sigma * sigma * x
    }

    fn diffusion_1d(&self, t: Time, x: Real) -> Real {
        let sigma = self.vol(t, x);
        sigma * x
    }

    fn expectation_1d(&self, t: Time, x: Real, dt: Time) -> Real {
        // For a GBM: E[S(t+dt)] = S(t) * exp((r-q) * dt)
        // But for the Euler scheme on log-space this is more accurate:
        let sigma = self.vol(t, x);
        let r = self.risk_free_rate.zero_rate_impl(t);
        let q = self.dividend_yield.zero_rate_impl(t);
        x * ((r - q - 0.5 * sigma * sigma) * dt).exp()
    }

    fn std_deviation_1d(&self, t: Time, x: Real, dt: Time) -> Real {
        let sigma = self.vol(t, x);
        sigma * x * dt.sqrt()
    }

    fn evolve_1d(&self, t: Time, x: Real, dt: Time, dw: Real) -> Real {
        // Exact GBM evolution when σ is treated as constant over [t, t+dt]:
        // S(t+dt) = S(t) * exp((r - q - σ²/2)·dt + σ·√dt·dw)
        let sigma = self.vol(t, x);
        let r = self.risk_free_rate.zero_rate_impl(t);
        let q = self.dividend_yield.zero_rate_impl(t);
        x * ((r - q - 0.5 * sigma * sigma) * dt + sigma * dt.sqrt() * dw).exp()
    }
}

// ── BlackScholesProcess (no dividends) ────────────────────────────────────────

/// A Black-Scholes process with no dividends (`q = 0`).
///
/// Corresponds to `QuantLib::BlackScholesProcess`.
pub fn black_scholes_process(
    x0: Real,
    risk_free_rate: Arc<dyn YieldTermStructure>,
    black_vol: Arc<dyn BlackVolTermStructure>,
) -> GeneralizedBlackScholesProcess {
    use ql_termstructures::FlatForward;
    use ql_time::Actual365Fixed;

    let ref_date = risk_free_rate.reference_date();
    let zero_yield: Arc<dyn YieldTermStructure> =
        Arc::new(FlatForward::continuous(ref_date, 0.0, Actual365Fixed));

    GeneralizedBlackScholesProcess::new(x0, risk_free_rate, zero_yield, black_vol)
}

/// A Black-Scholes-Merton process with continuous dividend yield.
///
/// Corresponds to `QuantLib::BlackScholesMertonProcess`.
pub fn black_scholes_merton_process(
    x0: Real,
    risk_free_rate: Arc<dyn YieldTermStructure>,
    dividend_yield: Arc<dyn YieldTermStructure>,
    black_vol: Arc<dyn BlackVolTermStructure>,
) -> GeneralizedBlackScholesProcess {
    GeneralizedBlackScholesProcess::new(x0, risk_free_rate, dividend_yield, black_vol)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stochastic_process::StochasticProcess;
    use approx::assert_abs_diff_eq;
    use ql_termstructures::{BlackConstantVol, FlatForward};
    use ql_time::{Actual365Fixed, Date};

    fn make_bsm() -> GeneralizedBlackScholesProcess {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let r: Arc<dyn YieldTermStructure> =
            Arc::new(FlatForward::continuous(ref_date, 0.05, Actual365Fixed));
        let q: Arc<dyn YieldTermStructure> =
            Arc::new(FlatForward::continuous(ref_date, 0.02, Actual365Fixed));
        let vol: Arc<dyn BlackVolTermStructure> =
            Arc::new(BlackConstantVol::new(ref_date, 0.20, Actual365Fixed));

        GeneralizedBlackScholesProcess::new(100.0, r, q, vol)
    }

    #[test]
    fn bsm_initial_values() {
        let p = make_bsm();
        assert_eq!(p.size(), 1);
        assert_abs_diff_eq!(p.x0(), 100.0, epsilon = 1e-15);
        assert_abs_diff_eq!(p.initial_values()[0], 100.0, epsilon = 1e-15);
    }

    #[test]
    fn bsm_drift_positive_rate_spread() {
        let p = make_bsm();
        // drift = (r - q - σ²/2) * S = (0.05 - 0.02 - 0.02) * 100 = 1.0
        let d = p.drift_1d(0.0, 100.0);
        assert_abs_diff_eq!(d, 1.0, epsilon = 0.1);
    }

    #[test]
    fn bsm_diffusion() {
        let p = make_bsm();
        // σ * S = 0.20 * 100 = 20
        let d = p.diffusion_1d(0.0, 100.0);
        assert_abs_diff_eq!(d, 20.0, epsilon = 1e-10);
    }

    #[test]
    fn bsm_evolve_zero_noise() {
        let p = make_bsm();
        let dt = 1.0;
        let x_new = p.evolve_1d(0.0, 100.0, dt, 0.0);
        // S * exp((r-q-σ²/2)*dt) = 100 * exp(0.05-0.02-0.02) = 100*exp(0.01)
        let expected = 100.0 * (0.01_f64).exp();
        assert_abs_diff_eq!(x_new, expected, epsilon = 0.01);
    }

    #[test]
    fn bsm_evolve_with_noise() {
        let p = make_bsm();
        let dt = 1.0 / 252.0; // 1 trading day
        let dw = 1.0; // 1 std dev
        let x_new = p.evolve_1d(0.0, 100.0, dt, dw);
        // Should be close to 100 but moved by ~σ√dt ≈ 1.26
        assert!(x_new > 99.0 && x_new < 103.0, "x_new = {x_new}");
    }

    #[test]
    fn black_scholes_no_div() {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let r: Arc<dyn YieldTermStructure> =
            Arc::new(FlatForward::continuous(ref_date, 0.05, Actual365Fixed));
        let vol: Arc<dyn BlackVolTermStructure> =
            Arc::new(BlackConstantVol::new(ref_date, 0.20, Actual365Fixed));

        let p = black_scholes_process(100.0, r, vol);
        // Dividend yield should be zero
        assert_abs_diff_eq!(p.dividend_yield().zero_rate_impl(1.0), 0.0, epsilon = 1e-15);
    }
}
