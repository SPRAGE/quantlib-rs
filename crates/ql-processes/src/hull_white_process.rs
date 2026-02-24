//! Hull-White (extended Vasicek) one-factor short-rate process
//! (translates `ql/processes/hullwhiteprocess.hpp`).
//!
//! The Hull-White model is:
//!
//! ```text
//! dr = (θ(t) − a·r) dt + σ dW
//! ```
//!
//! where `a` is the mean-reversion speed, `σ` is the volatility, and
//! `θ(t)` is a time-dependent drift chosen to exactly fit the initial
//! yield curve.
//!
//! The state variable is `r` (the short rate).

use crate::stochastic_process::StochasticProcess1D;
use ql_core::{Real, Time};
use ql_termstructures::YieldTermStructure;
use std::sync::Arc;

/// Hull-White one-factor short-rate process.
///
/// `dr = (θ(t) − a·r) dt + σ dW`
///
/// Corresponds to `QuantLib::HullWhiteProcess`.
#[derive(Debug)]
pub struct HullWhiteProcess {
    r0: Real,
    a: Real,
    sigma: Real,
    yield_curve: Arc<dyn YieldTermStructure>,
}

impl HullWhiteProcess {
    /// Create a new Hull-White process.
    ///
    /// # Arguments
    /// * `yield_curve` — the initial yield curve (for θ(t) calibration)
    /// * `a` — mean-reversion speed
    /// * `sigma` — short-rate volatility
    pub fn new(yield_curve: Arc<dyn YieldTermStructure>, a: Real, sigma: Real) -> Self {
        let r0 = yield_curve.forward_rate_impl(0.0);
        Self {
            r0,
            a,
            sigma,
            yield_curve,
        }
    }

    /// Mean-reversion speed.
    pub fn a(&self) -> Real {
        self.a
    }

    /// Short-rate volatility.
    pub fn sigma(&self) -> Real {
        self.sigma
    }

    /// Compute θ(t) from the initial yield curve.
    ///
    /// θ(t) = f'(0,t) + a·f(0,t) + σ²/(2a)·(1 − e^{−2at})
    ///
    /// where f(0,t) is the instantaneous forward rate at time t.
    fn theta(&self, t: Time) -> Real {
        let dt = 0.0001;
        let f_t = self.yield_curve.forward_rate_impl(t);
        // Finite-difference approximation of df/dt
        let f_t_plus = self.yield_curve.forward_rate_impl(t + dt);
        let df_dt = (f_t_plus - f_t) / dt;

        df_dt + self.a * f_t + self.sigma * self.sigma / (2.0 * self.a)
            * (1.0 - (-2.0 * self.a * t).exp())
    }
}

impl StochasticProcess1D for HullWhiteProcess {
    fn x0(&self) -> Real {
        self.r0
    }

    fn drift_1d(&self, t: Time, x: Real) -> Real {
        self.theta(t) - self.a * x
    }

    fn diffusion_1d(&self, _t: Time, _x: Real) -> Real {
        self.sigma
    }

    /// Exact conditional expectation:
    /// `E[r(t+dt)|r(t)] = r(t)·e^{-a·dt} + α(t+dt) - α(t)·e^{-a·dt}`
    ///
    /// We approximate using the Euler step for simplicity (the exact
    /// formula requires pre-computing α from the yield curve).
    fn expectation_1d(&self, t: Time, x: Real, dt: Time) -> Real {
        let ema = (-self.a * dt).exp();
        // For constant θ, exact conditional mean:
        // x * e^{-a·dt} + (θ/a)(1 - e^{-a·dt})
        // We use the numerical θ(t) evaluated at the midpoint
        let theta_mid = self.theta(t + 0.5 * dt);
        x * ema + (theta_mid / self.a) * (1.0 - ema)
    }

    /// Exact conditional standard deviation:
    /// `σ · √((1 − e^{-2a·dt}) / (2a))`
    fn std_deviation_1d(&self, _t: Time, _x: Real, dt: Time) -> Real {
        if self.a.abs() < 1e-15 {
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
    fn hull_white_initial_rate() {
        let p = HullWhiteProcess::new(flat_ts(0.05), 0.1, 0.01);
        // r(0) should be close to the flat rate
        assert!((p.x0() - 0.05).abs() < 0.01);
    }

    #[test]
    fn hull_white_diffusion_constant() {
        let p = HullWhiteProcess::new(flat_ts(0.05), 0.1, 0.01);
        assert!((p.diffusion_1d(0.0, 0.05) - 0.01).abs() < 1e-15);
        assert!((p.diffusion_1d(1.0, 0.10) - 0.01).abs() < 1e-15);
    }

    #[test]
    fn hull_white_exact_variance() {
        let a = 0.1;
        let sigma = 0.01;
        let p = HullWhiteProcess::new(flat_ts(0.05), a, sigma);
        let dt = 0.5;
        let sd = p.std_deviation_1d(0.0, 0.05, dt);
        let expected = sigma * ((1.0 - (-2.0 * a * dt).exp()) / (2.0 * a)).sqrt();
        assert!((sd - expected).abs() < 1e-12);
    }

    #[test]
    fn hull_white_zero_speed_limit() {
        // As a → 0, std dev → σ√dt
        let sigma = 0.01;
        let p = HullWhiteProcess::new(flat_ts(0.05), 1e-18, sigma);
        let dt = 1.0;
        let sd = p.std_deviation_1d(0.0, 0.05, dt);
        assert!((sd - sigma * dt.sqrt()).abs() < 1e-6);
    }
}
