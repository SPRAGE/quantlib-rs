//! Vasicek short-rate model.
//!
//! Translates `ql/models/shortrate/onefactormodels/vasicek.hpp`.
//!
//! ```text
//! dr = a(b − r) dt + σ dW
//! ```
//!
//! Discount bond price: `P(t,T) = A(t,T) exp(−B(t,T) r(t))`
//!
//! where
//! * `B(t,T) = (1 − e^{−a(T−t)}) / a`
//! * `A(t,T) = exp((B−(T−t))(ab²−σ²/2)/a² − σ²B²/(4a))`

use crate::calibrated_model::{CalibratedModel, Parameter, PositiveConstraint};
use crate::short_rate_model::{OneFactorModel, ShortRateModel};
use ql_core::{Real, Time};
use ql_processes::StochasticProcess1D;
use ql_termstructures::YieldTermStructure;
use std::sync::Arc;

/// Vasicek short-rate model.
///
/// Parameters: `a` (mean-reversion speed), `b` (long-run mean),
/// `σ` (volatility), `r₀` (initial short rate).
///
/// Corresponds to `QuantLib::Vasicek`.
#[derive(Debug)]
pub struct Vasicek {
    /// Mean-reversion speed.
    pub a: Real,
    /// Long-run mean level.
    pub b: Real,
    /// Volatility.
    pub sigma: Real,
    /// Initial short rate.
    pub r0: Real,
    /// The term structure used for fitting.
    term_structure: Arc<dyn YieldTermStructure>,
    /// Calibration parameters [a, b, sigma].
    params: Vec<Parameter>,
}

impl Vasicek {
    /// Create a new Vasicek model.
    pub fn new(
        a: Real,
        b: Real,
        sigma: Real,
        r0: Real,
        term_structure: Arc<dyn YieldTermStructure>,
    ) -> Self {
        let params = vec![
            Parameter::new(vec![a], PositiveConstraint),
            Parameter::constant(b),
            Parameter::new(vec![sigma], PositiveConstraint),
        ];
        Self {
            a,
            b,
            sigma,
            r0,
            term_structure,
            params,
        }
    }

    /// Bond duration function `B(t, T) = (1 − e^{−a(T−t)}) / a`.
    pub fn b_function(&self, t: Time, big_t: Time) -> Real {
        let tau = big_t - t;
        if self.a.abs() < 1e-12 {
            tau
        } else {
            (1.0 - (-self.a * tau).exp()) / self.a
        }
    }

    /// Log of the `A(t,T)` function.
    fn log_a(&self, t: Time, big_t: Time) -> Real {
        let b_val = self.b_function(t, big_t);
        let tau = big_t - t;
        let sigma2 = self.sigma * self.sigma;

        if self.a.abs() < 1e-12 {
            -sigma2 * tau * tau * tau / 6.0
        } else {
            let a2 = self.a * self.a;
            (b_val - tau) * (self.a * self.b - sigma2 / (2.0 * a2))
                - sigma2 * b_val * b_val / (4.0 * self.a)
        }
    }
}

impl CalibratedModel for Vasicek {
    fn params(&self) -> &[Parameter] {
        &self.params
    }

    fn set_params(&mut self, values: &[Real]) {
        if values.len() >= 3 {
            self.a = values[0];
            self.b = values[1];
            self.sigma = values[2];
            self.params[0].set_values(vec![values[0]]);
            self.params[1].set_values(vec![values[1]]);
            self.params[2].set_values(vec![values[2]]);
        }
    }
}

impl ShortRateModel for Vasicek {
    fn discount_bond(&self, t: Time, big_t: Time, rate: Real) -> Real {
        let b_val = self.b_function(t, big_t);
        (self.log_a(t, big_t) - b_val * rate).exp()
    }

    fn term_structure(&self) -> &Arc<dyn YieldTermStructure> {
        &self.term_structure
    }
}

/// A simple OU dynamics process for the Vasicek model.
#[derive(Debug)]
struct VasicekDynamics {
    a: Real,
    b: Real,
    sigma: Real,
    r0: Real,
}

impl StochasticProcess1D for VasicekDynamics {
    fn x0(&self) -> Real {
        self.r0
    }

    fn drift_1d(&self, _t: Time, x: Real) -> Real {
        self.a * (self.b - x)
    }

    fn diffusion_1d(&self, _t: Time, _x: Real) -> Real {
        self.sigma
    }

    fn expectation_1d(&self, _t: Time, x: Real, dt: Time) -> Real {
        let ema = (-self.a * dt).exp();
        x * ema + self.b * (1.0 - ema)
    }

    fn std_deviation_1d(&self, _t: Time, _x: Real, dt: Time) -> Real {
        if self.a.abs() < 1e-12 {
            self.sigma * dt.sqrt()
        } else {
            self.sigma * ((1.0 - (-2.0 * self.a * dt).exp()) / (2.0 * self.a)).sqrt()
        }
    }
}

impl OneFactorModel for Vasicek {
    fn short_rate_drift(&self, _t: Time, r: Real) -> Real {
        self.a * (self.b - r)
    }

    fn short_rate_diffusion(&self, _t: Time, _r: Real) -> Real {
        self.sigma
    }

    fn dynamics_process(&self) -> Box<dyn StochasticProcess1D> {
        Box::new(VasicekDynamics {
            a: self.a,
            b: self.b,
            sigma: self.sigma,
            r0: self.r0,
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
    fn vasicek_discount_bond_at_zero() {
        let v = Vasicek::new(0.1, 0.05, 0.01, 0.05, flat_ts(0.05));
        // P(0,0) = 1
        let p = v.discount_bond(0.0, 0.0, 0.05);
        assert!((p - 1.0).abs() < 1e-12);
    }

    #[test]
    fn vasicek_discount_bond_positive() {
        let v = Vasicek::new(0.1, 0.05, 0.01, 0.05, flat_ts(0.05));
        let p = v.discount_bond(0.0, 1.0, 0.05);
        assert!(p > 0.0);
        assert!(p < 1.0);
    }

    #[test]
    fn vasicek_b_function() {
        let v = Vasicek::new(0.1, 0.05, 0.01, 0.05, flat_ts(0.05));
        let b = v.b_function(0.0, 10.0);
        let expected = (1.0 - (-1.0_f64).exp()) / 0.1;
        assert!((b - expected).abs() < 1e-10);
    }

    #[test]
    fn vasicek_model_traits() {
        let v = Vasicek::new(0.1, 0.05, 0.01, 0.05, flat_ts(0.05));
        assert!((v.short_rate_drift(0.0, 0.05)).abs() < 1e-15);
        assert!((v.short_rate_diffusion(0.0, 0.05) - 0.01).abs() < 1e-15);
    }

    #[test]
    fn vasicek_dynamics() {
        let v = Vasicek::new(0.1, 0.05, 0.01, 0.05, flat_ts(0.05));
        let proc = v.dynamics_process();
        assert!((proc.x0() - 0.05).abs() < 1e-15);
        // At r=b, drift=0
        assert!((proc.drift_1d(0.0, 0.05)).abs() < 1e-15);
    }
}
