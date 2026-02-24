//! G2++ two-factor Gaussian short-rate model.
//!
//! Translates `ql/models/shortrate/twofactormodels/g2.hpp`.
//!
//! ```text
//! r(t) = x(t) + y(t) + φ(t)
//! dx = −a·x dt + σ dW₁
//! dy = −b·y dt + η dW₂
//! dW₁·dW₂ = ρ dt
//! ```
//!
//! Discount bond: `P(t,T) = A(t,T) exp(−B_a(τ)·x − B_b(τ)·y)`

use crate::calibrated_model::{CalibratedModel, Parameter, PositiveConstraint, BoundaryConstraint};
use crate::short_rate_model::{ShortRateModel, TwoFactorModel};
use ql_core::{Real, Time};
use ql_termstructures::YieldTermStructure;
use std::sync::Arc;

/// G2++ two-factor model.
///
/// Corresponds to `QuantLib::G2`.
#[derive(Debug)]
pub struct G2Model {
    /// Mean-reversion speed of first factor.
    pub a: Real,
    /// Volatility of first factor.
    pub sigma: Real,
    /// Mean-reversion speed of second factor.
    pub b: Real,
    /// Volatility of second factor.
    pub eta: Real,
    /// Correlation between factors.
    pub rho: Real,
    term_structure: Arc<dyn YieldTermStructure>,
    params: Vec<Parameter>,
}

impl G2Model {
    /// Create a new G2++ model.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        term_structure: Arc<dyn YieldTermStructure>,
        a: Real,
        sigma: Real,
        b: Real,
        eta: Real,
        rho: Real,
    ) -> Self {
        let params = vec![
            Parameter::new(vec![a], PositiveConstraint),
            Parameter::new(vec![sigma], PositiveConstraint),
            Parameter::new(vec![b], PositiveConstraint),
            Parameter::new(vec![eta], PositiveConstraint),
            Parameter::new(
                vec![rho],
                BoundaryConstraint {
                    lower: -1.0,
                    upper: 1.0,
                },
            ),
        ];
        Self {
            a,
            sigma,
            b,
            eta,
            rho,
            term_structure,
            params,
        }
    }

    /// `B_a(τ) = (1 − e^{−aτ})/a`
    fn b_a(&self, tau: Real) -> Real {
        if self.a.abs() < 1e-12 {
            tau
        } else {
            (1.0 - (-self.a * tau).exp()) / self.a
        }
    }

    /// `B_b(τ) = (1 − e^{−bτ})/b`
    fn b_b(&self, tau: Real) -> Real {
        if self.b.abs() < 1e-12 {
            tau
        } else {
            (1.0 - (-self.b * tau).exp()) / self.b
        }
    }

    /// `V(t,T)` — the variance function used in the A(t,T) formula.
    fn v_function(&self, t: Time, big_t: Time) -> Real {
        let tau = big_t - t;
        let s2 = self.sigma * self.sigma;
        let e2 = self.eta * self.eta;
        let a = self.a;
        let b = self.b;

        let ba = self.b_a(tau);
        let bb = self.b_b(tau);

        let term1 = s2 / (a * a) * (tau - 2.0 * ba + self.b_a(2.0 * tau) / 2.0);
        let term2 = e2 / (b * b) * (tau - 2.0 * bb + self.b_b(2.0 * tau) / 2.0);
        let term3 = 2.0 * self.rho * self.sigma * self.eta / (a * b)
            * (tau - ba - bb
                + (1.0 - (-(a + b) * tau).exp()) / (a + b));

        term1 + term2 + term3
    }
}

impl CalibratedModel for G2Model {
    fn params(&self) -> &[Parameter] {
        &self.params
    }

    fn set_params(&mut self, values: &[Real]) {
        if values.len() >= 5 {
            self.a = values[0];
            self.sigma = values[1];
            self.b = values[2];
            self.eta = values[3];
            self.rho = values[4];
            for (i, val) in values[..5].iter().enumerate() {
                self.params[i].set_values(vec![*val]);
            }
        }
    }
}

impl ShortRateModel for G2Model {
    fn discount_bond(&self, t: Time, big_t: Time, _rate: Real) -> Real {
        // For the G2++ model, the discount bond depends on the two state
        // variables x and y. As a simplification with rate = x + y + φ,
        // we provide the initial-curve-fitted version:
        // P(0,T)/P(0,t) * exp(-0.5*(V(0,T) - V(0,t)))
        // This is the zero-x, zero-y bond price from the fitted model.
        let ts = &self.term_structure;
        let df_t = ts.discount(t);
        let df_t_big = ts.discount(big_t);
        if df_t.abs() < 1e-30 {
            return 0.0;
        }
        let ratio = df_t_big / df_t;
        let v_adj = 0.5 * (self.v_function(0.0, big_t) - self.v_function(0.0, t));
        ratio * (-v_adj).exp()
    }

    fn term_structure(&self) -> &Arc<dyn YieldTermStructure> {
        &self.term_structure
    }
}

impl TwoFactorModel for G2Model {
    fn correlation(&self) -> Real {
        self.rho
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
    fn g2_model_five_params() {
        let g = G2Model::new(flat_ts(0.05), 0.1, 0.01, 0.2, 0.015, -0.5);
        assert_eq!(g.params().len(), 5);
        assert!((g.correlation() - (-0.5)).abs() < 1e-15);
    }

    #[test]
    fn g2_discount_bond_at_zero() {
        let g = G2Model::new(flat_ts(0.05), 0.1, 0.01, 0.2, 0.015, -0.5);
        let p = g.discount_bond(0.0, 0.0, 0.05);
        assert!((p - 1.0).abs() < 1e-10);
    }

    #[test]
    fn g2_discount_bond_positive() {
        let g = G2Model::new(flat_ts(0.05), 0.1, 0.01, 0.2, 0.015, -0.5);
        let p = g.discount_bond(0.0, 5.0, 0.05);
        assert!(p > 0.0);
        assert!(p < 1.0);
    }

    #[test]
    fn g2_set_params() {
        let mut g = G2Model::new(flat_ts(0.05), 0.1, 0.01, 0.2, 0.015, -0.5);
        g.set_params(&[0.2, 0.02, 0.3, 0.025, 0.3]);
        assert!((g.a - 0.2).abs() < 1e-15);
        assert!((g.rho - 0.3).abs() < 1e-15);
    }
}
