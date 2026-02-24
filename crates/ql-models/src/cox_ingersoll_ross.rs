//! Cox–Ingersoll–Ross (CIR) short-rate model.
//!
//! Translates `ql/models/shortrate/onefactormodels/coxingersollross.hpp`.
//!
//! ```text
//! dr = a(b − r) dt + σ √r dW
//! ```
//!
//! Discount bond: `P(t,T) = A(t,T) exp(−B(t,T) r(t))`
//! where `γ = √(a² + 2σ²)`.

use crate::calibrated_model::{CalibratedModel, Parameter, PositiveConstraint};
use crate::short_rate_model::{OneFactorModel, ShortRateModel};
use ql_core::{Real, Time};
use ql_processes::StochasticProcess1D;
use ql_termstructures::YieldTermStructure;
use std::sync::Arc;

/// Cox–Ingersoll–Ross model.
///
/// Corresponds to `QuantLib::CoxIngersollRoss`.
#[derive(Debug)]
pub struct CoxIngersollRoss {
    /// Mean-reversion speed.
    pub a: Real,
    /// Long-run mean.
    pub b: Real,
    /// Volatility.
    pub sigma: Real,
    /// Initial short rate.
    pub r0: Real,
    term_structure: Arc<dyn YieldTermStructure>,
    params: Vec<Parameter>,
}

impl CoxIngersollRoss {
    /// Create a new CIR model.
    ///
    /// Feller condition requires `2ab > σ²` for the rate to stay positive.
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

    /// `γ = √(a² + 2σ²)`
    fn gamma(&self) -> Real {
        (self.a * self.a + 2.0 * self.sigma * self.sigma).sqrt()
    }

    /// `B(t,T)` for CIR model.
    pub fn b_function(&self, t: Time, big_t: Time) -> Real {
        let tau = big_t - t;
        let g = self.gamma();
        2.0 * ((g * tau).exp() - 1.0)
            / ((g + self.a) * ((g * tau).exp() - 1.0) + 2.0 * g)
    }

    /// `ln A(t,T)` for CIR model.
    fn log_a(&self, t: Time, big_t: Time) -> Real {
        let tau = big_t - t;
        let g = self.gamma();
        let exponent = 2.0 * self.a * self.b / (self.sigma * self.sigma);
        let numerator = 2.0 * g * (-(g + self.a) * tau / 2.0).exp();
        let denominator = (g + self.a) * ((g * tau).exp() - 1.0) + 2.0 * g;
        // Avoid issues when tau is very small
        if tau.abs() < 1e-14 {
            0.0
        } else {
            exponent * (numerator / denominator).ln()
        }
    }

    /// Check the Feller condition: `2ab > σ²`.
    pub fn feller_satisfied(&self) -> bool {
        2.0 * self.a * self.b > self.sigma * self.sigma
    }
}

impl CalibratedModel for CoxIngersollRoss {
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

impl ShortRateModel for CoxIngersollRoss {
    fn discount_bond(&self, t: Time, big_t: Time, rate: Real) -> Real {
        if (big_t - t).abs() < 1e-14 {
            return 1.0;
        }
        let b_val = self.b_function(t, big_t);
        (self.log_a(t, big_t) - b_val * rate).exp()
    }

    fn term_structure(&self) -> &Arc<dyn YieldTermStructure> {
        &self.term_structure
    }
}

/// CIR dynamics process.
#[derive(Debug)]
struct CirDynamics {
    a: Real,
    b: Real,
    sigma: Real,
    r0: Real,
}

impl StochasticProcess1D for CirDynamics {
    fn x0(&self) -> Real {
        self.r0
    }

    fn drift_1d(&self, _t: Time, x: Real) -> Real {
        self.a * (self.b - x)
    }

    fn diffusion_1d(&self, _t: Time, x: Real) -> Real {
        self.sigma * x.max(0.0).sqrt()
    }
}

impl OneFactorModel for CoxIngersollRoss {
    fn short_rate_drift(&self, _t: Time, r: Real) -> Real {
        self.a * (self.b - r)
    }

    fn short_rate_diffusion(&self, _t: Time, r: Real) -> Real {
        self.sigma * r.max(0.0).sqrt()
    }

    fn dynamics_process(&self) -> Box<dyn StochasticProcess1D> {
        Box::new(CirDynamics {
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
    fn cir_discount_bond_at_zero() {
        let m = CoxIngersollRoss::new(0.3, 0.05, 0.1, 0.05, flat_ts(0.05));
        let p = m.discount_bond(0.0, 0.0, 0.05);
        assert!((p - 1.0).abs() < 1e-12);
    }

    #[test]
    fn cir_discount_bond_positive() {
        let m = CoxIngersollRoss::new(0.3, 0.05, 0.1, 0.05, flat_ts(0.05));
        let p = m.discount_bond(0.0, 5.0, 0.05);
        assert!(p > 0.0);
        assert!(p < 1.0);
    }

    #[test]
    fn cir_feller_condition() {
        // 2ab = 2*0.3*0.05 = 0.03, σ² = 0.01 => Feller OK
        let m = CoxIngersollRoss::new(0.3, 0.05, 0.1, 0.05, flat_ts(0.05));
        assert!(m.feller_satisfied());

        // σ = 0.5 => σ² = 0.25 > 2ab = 0.03 => Feller violated
        let m2 = CoxIngersollRoss::new(0.3, 0.05, 0.5, 0.05, flat_ts(0.05));
        assert!(!m2.feller_satisfied());
    }

    #[test]
    fn cir_diffusion_sqrt() {
        let m = CoxIngersollRoss::new(0.3, 0.05, 0.1, 0.04, flat_ts(0.05));
        let d = m.short_rate_diffusion(0.0, 0.04);
        let expected = 0.1 * 0.04_f64.sqrt();
        assert!((d - expected).abs() < 1e-12);
    }
}
