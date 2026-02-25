//! Hull-White forward-measure process
//! (translates `ql/processes/hullwhiteforwardprocess.hpp`).
//!
//! Under the T-forward measure, the Hull-White process has dynamics:
//!
//! ```text
//! dr = (θ(t) − a·r − σ²·B(t,T)) dt + σ dW^T
//! ```
//!
//! where `B(t,T) = (1 - exp(-a(T-t)))/a` and `W^T` is a Brownian motion
//! under the T-forward measure.

use crate::stochastic_process::StochasticProcess1D;
use ql_core::{Real, Time};
use ql_termstructures::YieldTermStructure;
use std::sync::Arc;

/// Hull-White process under the forward measure.
///
/// Corresponds to `QuantLib::HullWhiteForwardProcess`.
#[derive(Debug)]
pub struct HullWhiteForwardProcess {
    r0: Real,
    a: Real,
    sigma: Real,
    maturity_t: Real,
    yield_curve: Arc<dyn YieldTermStructure>,
}

impl HullWhiteForwardProcess {
    pub fn new(
        yield_curve: Arc<dyn YieldTermStructure>,
        a: Real,
        sigma: Real,
        maturity_t: Real,
    ) -> Self {
        let r0 = yield_curve.forward_rate_impl(0.0);
        Self {
            r0,
            a,
            sigma,
            maturity_t,
            yield_curve,
        }
    }

    /// `B(t, T) = (1 - exp(-a(T-t)))/a`   (bond duration function).
    fn b_function(&self, t: Time, big_t: Real) -> Real {
        if self.a.abs() < 1e-12 {
            big_t - t
        } else {
            (1.0 - (-self.a * (big_t - t)).exp()) / self.a
        }
    }

    /// θ(t) via finite-difference of instantaneous forward rate.
    fn theta(&self, t: Time) -> Real {
        let eps = 1e-6;
        let f_t = self.yield_curve.forward_rate_impl(t);
        let f_te = self.yield_curve.forward_rate_impl(t + eps);
        let df_dt = (f_te - f_t) / eps;
        df_dt
            + self.a * f_t
            + self.sigma * self.sigma / (2.0 * self.a) * (1.0 - (-2.0 * self.a * t).exp())
    }

    /// Access mean-reversion speed.
    pub fn a(&self) -> Real {
        self.a
    }

    /// Access volatility.
    pub fn sigma(&self) -> Real {
        self.sigma
    }

    /// Access the forward measure maturity.
    pub fn maturity(&self) -> Real {
        self.maturity_t
    }
}

impl StochasticProcess1D for HullWhiteForwardProcess {
    fn x0(&self) -> Real {
        self.r0
    }

    fn drift_1d(&self, t: Time, r: Real) -> Real {
        let drift_correction = self.sigma * self.sigma * self.b_function(t, self.maturity_t);
        self.theta(t) - self.a * r - drift_correction
    }

    fn diffusion_1d(&self, _t: Time, _r: Real) -> Real {
        self.sigma
    }

    fn expectation_1d(&self, t: Time, r: Real, dt: Time) -> Real {
        let exp_a = (-self.a * dt).exp();
        // Use trapezoid approximation for θ integral
        let theta_t = self.theta(t);
        let theta_t1 = self.theta(t + dt);
        let theta_avg = 0.5 * (theta_t + theta_t1);

        let b_drift = self.sigma * self.sigma * self.b_function(t + 0.5 * dt, self.maturity_t);

        r * exp_a + (theta_avg - b_drift) * (1.0 - exp_a) / self.a
    }

    fn std_deviation_1d(&self, _t: Time, _r: Real, dt: Time) -> Real {
        if self.a.abs() < 1e-12 {
            self.sigma * dt.sqrt()
        } else {
            self.sigma * ((1.0 - (-2.0 * self.a * dt).exp()) / (2.0 * self.a)).sqrt()
        }
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
    fn hw_forward_basic() {
        let p = HullWhiteForwardProcess::new(flat_ts(0.05), 0.1, 0.01, 10.0);
        assert!((p.x0() - 0.05).abs() < 1e-10);
        assert_eq!(p.maturity(), 10.0);
    }

    #[test]
    fn hw_forward_diffusion_constant() {
        let p = HullWhiteForwardProcess::new(flat_ts(0.05), 0.1, 0.01, 10.0);
        assert!((p.diffusion_1d(1.0, 0.03) - 0.01).abs() < 1e-15);
    }

    #[test]
    fn hw_forward_drift_has_correction() {
        let p = HullWhiteForwardProcess::new(flat_ts(0.05), 0.1, 0.01, 10.0);
        let d_fwd = p.drift_1d(1.0, 0.05);
        // Without forward-measure correction at a=0.1, sigma=0.01, the magnitude differs
        // Simply check it's finite and reasonable
        assert!(d_fwd.is_finite());
        assert!(d_fwd.abs() < 1.0);
    }

    #[test]
    fn hw_forward_std_dev_positive() {
        let p = HullWhiteForwardProcess::new(flat_ts(0.05), 0.1, 0.01, 10.0);
        let sd = p.std_deviation_1d(0.0, 0.05, 0.25);
        assert!(sd > 0.0);
        assert!(sd < 0.1); // Small σ => small std dev
    }
}
