//! Hull-White (extended Vasicek) model.
//!
//! Translates `ql/models/shortrate/onefactormodels/hullwhite.hpp`.
//!
//! ```text
//! dr = (θ(t) − a·r) dt + σ dW
//! ```
//!
//! The function `θ(t)` is chosen to exactly fit the initial yield curve.
//!
//! Discount bond price:
//! `P(t,T) = A(t,T) exp(−B(t,T) r(t))`
//!
//! where `B` is the same as Vasicek and `A` is adjusted to fit the
//! initial curve.

use crate::calibrated_model::{CalibratedModel, Parameter, PositiveConstraint};
use crate::short_rate_model::{OneFactorModel, ShortRateModel};
use ql_core::{Real, Time};
use ql_processes::StochasticProcess1D;
use ql_termstructures::YieldTermStructure;
use std::sync::Arc;

/// Hull-White one-factor model.
///
/// Corresponds to `QuantLib::HullWhite`.
#[derive(Debug)]
pub struct HullWhite {
    /// Mean-reversion speed.
    pub a: Real,
    /// Volatility.
    pub sigma: Real,
    /// Initial yield curve.
    term_structure: Arc<dyn YieldTermStructure>,
    params: Vec<Parameter>,
}

impl HullWhite {
    /// Create a new Hull-White model.
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

    /// `B(t,T) = (1 - exp(-a(T-t)))/a`
    pub fn b_function(&self, t: Time, big_t: Time) -> Real {
        let tau = big_t - t;
        if self.a.abs() < 1e-12 {
            tau
        } else {
            (1.0 - (-self.a * tau).exp()) / self.a
        }
    }

    /// `A(t,T)` using the initial yield curve for exact fitting.
    ///
    /// `ln A(t,T) = ln(P(0,T)/P(0,t)) − B(t,T)·f(0,t) − σ²/(4a)·B²·(1−e^{-2at})`
    fn log_a(&self, t: Time, big_t: Time) -> Real {
        let b_val = self.b_function(t, big_t);
        let ts = &self.term_structure;

        let ln_pt = (-ts.zero_rate_impl(big_t) * big_t).min(0.0).max(-500.0);
        let ln_p0 = if t > 1e-12 {
            (-ts.zero_rate_impl(t) * t).min(0.0).max(-500.0)
        } else {
            0.0
        };

        let f0t = ts.forward_rate_impl(t);
        let sigma2 = self.sigma * self.sigma;

        (ln_pt - ln_p0) - b_val * f0t
            - sigma2 / (4.0 * self.a) * b_val * b_val * (1.0 - (-2.0 * self.a * t).exp())
    }

    /// Compute `θ(t)` from the initial yield curve.
    fn theta(&self, t: Time) -> Real {
        let dt = 1e-4;
        let f_t = self.term_structure.forward_rate_impl(t);
        let f_tdt = self.term_structure.forward_rate_impl(t + dt);
        let df_dt = (f_tdt - f_t) / dt;
        df_dt + self.a * f_t
            + self.sigma * self.sigma / (2.0 * self.a) * (1.0 - (-2.0 * self.a * t).exp())
    }
}

impl CalibratedModel for HullWhite {
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

impl ShortRateModel for HullWhite {
    fn discount_bond(&self, t: Time, big_t: Time, rate: Real) -> Real {
        let b_val = self.b_function(t, big_t);
        (self.log_a(t, big_t) - b_val * rate).exp()
    }

    fn term_structure(&self) -> &Arc<dyn YieldTermStructure> {
        &self.term_structure
    }
}

/// Dynamics process for the Hull-White model.
#[derive(Debug)]
struct HullWhiteDynamics {
    a: Real,
    sigma: Real,
    r0: Real,
    term_structure: Arc<dyn YieldTermStructure>,
}

impl StochasticProcess1D for HullWhiteDynamics {
    fn x0(&self) -> Real {
        self.r0
    }

    fn drift_1d(&self, t: Time, x: Real) -> Real {
        let dt = 1e-4;
        let f_t = self.term_structure.forward_rate_impl(t);
        let f_tdt = self.term_structure.forward_rate_impl(t + dt);
        let df_dt = (f_tdt - f_t) / dt;
        let theta = df_dt + self.a * f_t
            + self.sigma * self.sigma / (2.0 * self.a) * (1.0 - (-2.0 * self.a * t).exp());
        theta - self.a * x
    }

    fn diffusion_1d(&self, _t: Time, _x: Real) -> Real {
        self.sigma
    }

    fn expectation_1d(&self, t: Time, x: Real, dt: Time) -> Real {
        let ema = (-self.a * dt).exp();
        let theta_mid = {
            let eps = 1e-4;
            let mid = t + 0.5 * dt;
            let f_t = self.term_structure.forward_rate_impl(mid);
            let f_tdt = self.term_structure.forward_rate_impl(mid + eps);
            let df_dt = (f_tdt - f_t) / eps;
            df_dt + self.a * f_t
                + self.sigma * self.sigma / (2.0 * self.a)
                    * (1.0 - (-2.0 * self.a * mid).exp())
        };
        x * ema + (theta_mid / self.a) * (1.0 - ema)
    }

    fn std_deviation_1d(&self, _t: Time, _x: Real, dt: Time) -> Real {
        if self.a.abs() < 1e-12 {
            self.sigma * dt.sqrt()
        } else {
            self.sigma * ((1.0 - (-2.0 * self.a * dt).exp()) / (2.0 * self.a)).sqrt()
        }
    }
}

impl OneFactorModel for HullWhite {
    fn short_rate_drift(&self, t: Time, r: Real) -> Real {
        self.theta(t) - self.a * r
    }

    fn short_rate_diffusion(&self, _t: Time, _r: Real) -> Real {
        self.sigma
    }

    fn dynamics_process(&self) -> Box<dyn StochasticProcess1D> {
        let r0 = self.term_structure.forward_rate_impl(0.0);
        Box::new(HullWhiteDynamics {
            a: self.a,
            sigma: self.sigma,
            r0,
            term_structure: Arc::clone(&self.term_structure),
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
    fn hw_discount_bond_at_zero() {
        let hw = HullWhite::new(flat_ts(0.05), 0.1, 0.01);
        let p = hw.discount_bond(0.0, 0.0, 0.05);
        assert!((p - 1.0).abs() < 1e-10);
    }

    #[test]
    fn hw_discount_bond_positive() {
        let hw = HullWhite::new(flat_ts(0.05), 0.1, 0.01);
        let p = hw.discount_bond(0.0, 5.0, 0.05);
        assert!(p > 0.0);
        assert!(p < 1.0);
    }

    #[test]
    fn hw_b_function() {
        let hw = HullWhite::new(flat_ts(0.05), 0.1, 0.01);
        let b = hw.b_function(0.0, 10.0);
        let expected = (1.0 - (-1.0_f64).exp()) / 0.1;
        assert!((b - expected).abs() < 1e-10);
    }

    #[test]
    fn hw_dynamics_initial_rate() {
        let hw = HullWhite::new(flat_ts(0.05), 0.1, 0.01);
        let proc = hw.dynamics_process();
        assert!((proc.x0() - 0.05).abs() < 0.01);
    }

    #[test]
    fn hw_diffusion_constant() {
        let hw = HullWhite::new(flat_ts(0.05), 0.1, 0.01);
        assert!((hw.short_rate_diffusion(0.0, 0.05) - 0.01).abs() < 1e-15);
    }
}
