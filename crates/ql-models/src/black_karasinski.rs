//! Black-Karasinski short-rate model.
//!
//! Translates `ql/models/shortrate/onefactormodels/blackkarasinski.hpp`.
//!
//! ```text
//! d ln(r) = (θ(t) − a·ln(r)) dt + σ dW
//! ```
//!
//! This is a log-normal short-rate model — rates stay positive by construction.
//! Unlike Vasicek / Hull-White, there is no closed-form bond price;
//! pricing requires a tree or MC.

use crate::calibrated_model::{CalibratedModel, Parameter, PositiveConstraint};
use crate::short_rate_model::{OneFactorModel, ShortRateModel};
use ql_core::{Real, Time};
use ql_processes::StochasticProcess1D;
use ql_termstructures::YieldTermStructure;
use std::sync::Arc;

/// Black-Karasinski short-rate model.
///
/// Corresponds to `QuantLib::BlackKarasinski`.
#[derive(Debug)]
pub struct BlackKarasinski {
    /// Mean-reversion speed.
    pub a: Real,
    /// Volatility of ln(r).
    pub sigma: Real,
    /// Initial yield curve.
    term_structure: Arc<dyn YieldTermStructure>,
    params: Vec<Parameter>,
}

impl BlackKarasinski {
    /// Create a new Black-Karasinski model.
    pub fn new(
        term_structure: Arc<dyn YieldTermStructure>,
        a: Real,
        sigma: Real,
    ) -> Self {
        let params = vec![
            Parameter::new(vec![a], PositiveConstraint),
            Parameter::new(vec![sigma], PositiveConstraint),
        ];
        Self {
            a,
            sigma,
            term_structure,
            params,
        }
    }
}

impl CalibratedModel for BlackKarasinski {
    fn params(&self) -> &[Parameter] {
        &self.params
    }

    fn set_params(&mut self, values: &[Real]) {
        if values.len() >= 2 {
            self.a = values[0];
            self.sigma = values[1];
            self.params[0].set_values(vec![values[0]]);
            self.params[1].set_values(vec![values[1]]);
        }
    }
}

impl ShortRateModel for BlackKarasinski {
    fn discount_bond(&self, _t: Time, big_t: Time, _rate: Real) -> Real {
        // No analytical solution — use the initial term structure as
        // a first-order approximation.
        self.term_structure.discount(big_t)
    }

    fn term_structure(&self) -> &Arc<dyn YieldTermStructure> {
        &self.term_structure
    }
}

/// Dynamics for BK: state y = ln(r), dr = r·(θ − a·y) dt + r·σ dW.
/// More precisely, dy = (θ(t) − a·y) dt + σ dW where y = ln(r).
#[derive(Debug)]
struct BkDynamics {
    a: Real,
    sigma: Real,
    y0: Real, // ln(r0)
}

impl StochasticProcess1D for BkDynamics {
    fn x0(&self) -> Real {
        self.y0
    }

    fn drift_1d(&self, _t: Time, y: Real) -> Real {
        // θ(t) − a·y; we approximate θ with a·y0 so at y=y0 drift=0
        -self.a * (y - self.y0)
    }

    fn diffusion_1d(&self, _t: Time, _y: Real) -> Real {
        self.sigma
    }

    fn expectation_1d(&self, _t: Time, y: Real, dt: Time) -> Real {
        let ema = (-self.a * dt).exp();
        y * ema + self.y0 * (1.0 - ema)
    }

    fn std_deviation_1d(&self, _t: Time, _y: Real, dt: Time) -> Real {
        if self.a.abs() < 1e-12 {
            self.sigma * dt.sqrt()
        } else {
            self.sigma * ((1.0 - (-2.0 * self.a * dt).exp()) / (2.0 * self.a)).sqrt()
        }
    }
}

impl OneFactorModel for BlackKarasinski {
    fn short_rate_drift(&self, t: Time, r: Real) -> Real {
        // In the log space: d ln(r) = (θ(t) − a ln(r)) dt
        // For the rate itself: dr = r (θ(t) − a ln(r)) dt
        let ln_r = r.ln();
        let f0 = self.term_structure.forward_rate_impl(t);
        let theta_approx = self.a * f0.ln();
        r * (theta_approx - self.a * ln_r)
    }

    fn short_rate_diffusion(&self, _t: Time, r: Real) -> Real {
        r * self.sigma
    }

    fn dynamics_process(&self) -> Box<dyn StochasticProcess1D> {
        let r0 = self.term_structure.forward_rate_impl(0.0).max(1e-10);
        Box::new(BkDynamics {
            a: self.a,
            sigma: self.sigma,
            y0: r0.ln(),
        })
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
    fn bk_dynamics_log_space() {
        let bk = BlackKarasinski::new(flat_ts(0.05), 0.1, 0.15);
        let proc = bk.dynamics_process();
        let y0 = proc.x0();
        assert!((y0 - 0.05_f64.ln()).abs() < 0.01);
    }

    #[test]
    fn bk_diffusion_proportional_to_r() {
        let bk = BlackKarasinski::new(flat_ts(0.05), 0.1, 0.15);
        let d = bk.short_rate_diffusion(0.0, 0.05);
        assert!((d - 0.05 * 0.15).abs() < 1e-15);
    }

    #[test]
    fn bk_params_calibratable() {
        let mut bk = BlackKarasinski::new(flat_ts(0.05), 0.1, 0.15);
        assert_eq!(bk.params().len(), 2);
        bk.set_params(&[0.2, 0.25]);
        assert!((bk.a - 0.2).abs() < 1e-15);
        assert!((bk.sigma - 0.25).abs() < 1e-15);
    }
}
