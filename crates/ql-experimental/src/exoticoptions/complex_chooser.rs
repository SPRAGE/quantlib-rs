//! Analytic pricing engine for complex chooser options.
//!
//! A complex chooser option allows the holder to choose at time `t_c` between
//! a call with strike `Xc` and expiry `Tc`, or a put with strike `Xp` and
//! expiry `Tp`. (Different strikes and different maturities are allowed.)
//!
//! Uses the bivariate normal distribution and Newton-Raphson root finding.
//!
//! Reference: Rubinstein (1991); Haug "Complete Guide to Option Pricing Formulas".

use ql_core::Real;
use super::bivariate_normal::bivariate_normal_cdf_dr78;
use ql_math::distributions::normal_cdf;
use ql_processes::GeneralizedBlackScholesProcess;
use std::sync::Arc;

/// Arguments for a complex chooser option.
#[derive(Debug, Clone)]
pub struct ComplexChooserOptionArgs {
    /// The choosing date as year fraction from reference date.
    pub choosing_time: Real,
    /// Strike for the call path.
    pub strike_call: Real,
    /// Strike for the put path.
    pub strike_put: Real,
    /// Time to maturity for the call path (year fraction from reference date).
    pub call_maturity_time: Real,
    /// Time to maturity for the put path (year fraction from reference date).
    pub put_maturity_time: Real,
}

/// Analytic pricing engine for complex chooser options.
///
/// Corresponds to `QuantLib::AnalyticComplexChooserEngine`.
#[derive(Debug)]
pub struct AnalyticComplexChooserEngine {
    process: Arc<GeneralizedBlackScholesProcess>,
}

impl AnalyticComplexChooserEngine {
    /// Create a new engine with the given Black-Scholes process.
    pub fn new(process: Arc<GeneralizedBlackScholesProcess>) -> Self {
        Self { process }
    }

    /// Price the complex chooser option.
    pub fn calculate(&self, args: &ComplexChooserOptionArgs) -> Real {
        let s = self.process.spot();
        let xc = args.strike_call;
        let xp = args.strike_put;
        let t = args.choosing_time;
        let tc = args.call_maturity_time - t;
        let tp = args.put_maturity_time - t;

        // Find the critical value I* using Newton-Raphson
        let i_star = self.critical_value(args);

        // Compute with rates/vols at the appropriate tenors
        let b_t = self.risk_free_rate(t) - self.dividend_yield(t);
        let v_t = self.volatility(t);

        let d1 = ((s / i_star).ln() + (b_t + v_t * v_t / 2.0) * t) / (v_t * t.sqrt());
        let d2 = d1 - v_t * t.sqrt();

        let b_tc = self.risk_free_rate(t + tc) - self.dividend_yield(t + tc);
        let v_tc = self.volatility(tc);
        let y1 = ((s / xc).ln() + (b_tc + v_tc * v_tc / 2.0) * tc) / (v_tc * tc.sqrt());

        let b_tp = self.risk_free_rate(t + tp) - self.dividend_yield(t + tp);
        let v_tp = self.volatility(tp);
        let y2 = ((s / xp).ln() + (b_tp + v_tp * v_tp / 2.0) * tp) / (v_tp * tp.sqrt());

        let rho1 = (t / tc).sqrt();
        let rho2 = (t / tp).sqrt();

        // Call component
        let b = self.risk_free_rate(t + tc) - self.dividend_yield(t + tc);
        let r = self.risk_free_rate(t + tc);
        let v = self.volatility(tc);
        let mut result = s * ((b - r) * tc).exp() * bivariate_normal_cdf_dr78(d1, y1, rho1)
            - xc * (-r * tc).exp()
                * bivariate_normal_cdf_dr78(d2, y1 - v * tc.sqrt(), rho1);

        // Put component
        let b = self.risk_free_rate(t + tp) - self.dividend_yield(t + tp);
        let r = self.risk_free_rate(t + tp);
        let v = self.volatility(tp);
        result -= s * ((b - r) * tp).exp() * bivariate_normal_cdf_dr78(-d1, -y2, rho2);
        result +=
            xp * (-r * tp).exp() * bivariate_normal_cdf_dr78(-d2, -y2 + v * tp.sqrt(), rho2);

        result
    }

    /// Find the critical spot value I* where call(I*) = put(I*) at choosing time.
    fn critical_value(&self, args: &ComplexChooserOptionArgs) -> Real {
        let mut sv = self.process.spot();

        let epsilon = 0.001;

        loop {
            let (ci, dc) = self.bs_value_delta(sv, args, true);
            let (pi, dp) = self.bs_value_delta(sv, args, false);

            let yi = ci - pi;
            let di = dc - dp;

            if yi.abs() <= epsilon {
                break;
            }

            sv -= yi / di;
        }
        sv
    }

    /// Compute BS value and delta for the call or put component.
    fn bs_value_delta(&self, spot: Real, args: &ComplexChooserOptionArgs, is_call: bool) -> (Real, Real) {
        let t_choose = args.choosing_time;

        let (strike, mat_time) = if is_call {
            (args.strike_call, args.call_maturity_time - 2.0 * t_choose)
        } else {
            (args.strike_put, args.put_maturity_time - 2.0 * t_choose)
        };

        let vol = self.volatility(mat_time);
        let std_dev = vol * mat_time.sqrt();
        let growth = self.dividend_discount(mat_time);
        let discount = self.risk_free_discount(mat_time);

        // Black-Scholes calculator
        let forward = spot * growth / discount;
        let d1 = if std_dev > 1e-15 {
            (forward / strike).ln() / std_dev + 0.5 * std_dev
        } else {
            if forward > strike { 1e15 } else { -1e15 }
        };
        let d2 = d1 - std_dev;

        if is_call {
            let value = discount * (forward * normal_cdf(d1) - strike * normal_cdf(d2));
            let delta = growth * normal_cdf(d1);
            (value, delta)
        } else {
            let value = discount * (strike * normal_cdf(-d2) - forward * normal_cdf(-d1));
            let delta = -growth * normal_cdf(-d1);
            (value, delta)
        }
    }

    fn volatility(&self, t: Real) -> Real {
        self.process
            .black_volatility()
            .expect("process must have a black vol surface")
            .black_vol_time(t, 0.0)
    }

    fn risk_free_rate(&self, t: Real) -> Real {
        self.process.risk_free_rate().zero_rate_impl(t)
    }

    fn dividend_yield(&self, t: Real) -> Real {
        self.process.dividend_yield().zero_rate_impl(t)
    }

    fn risk_free_discount(&self, t: Real) -> Real {
        self.process.risk_free_rate().discount(t)
    }

    fn dividend_discount(&self, t: Real) -> Real {
        self.process.dividend_yield().discount(t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ql_termstructures::{BlackConstantVol, FlatForward, YieldTermStructure};
    use ql_time::{Actual360, Date, DayCounter};

    fn make_process(
        ref_date: Date,
        spot: Real,
        q: Real,
        r: Real,
        vol: Real,
    ) -> Arc<GeneralizedBlackScholesProcess> {
        let r_ts: Arc<dyn YieldTermStructure> =
            Arc::new(FlatForward::continuous(ref_date, r, Actual360));
        let q_ts: Arc<dyn YieldTermStructure> =
            Arc::new(FlatForward::continuous(ref_date, q, Actual360));
        let vol_ts = Arc::new(BlackConstantVol::new(ref_date, vol, Actual360));
        Arc::new(GeneralizedBlackScholesProcess::new(
            spot, r_ts, q_ts, vol_ts,
        ))
    }

    /// Test from Haug "Complete Guide to Option Pricing Formulas".
    /// Matches QuantLib C++ test `testAnalyticComplexChooserEngine`.
    #[test]
    fn test_complex_chooser() {
        let today = Date::from_ymd(2025, 1, 2).unwrap();
        let dc = Actual360;

        let process = make_process(today, 50.0, 0.05, 0.10, 0.35);

        let choosing_date = today + 90;
        let call_exercise_date = choosing_date + 180;
        let put_exercise_date = choosing_date + 210;

        let args = ComplexChooserOptionArgs {
            choosing_time: dc.year_fraction(today, choosing_date),
            strike_call: 55.0,
            strike_put: 48.0,
            call_maturity_time: dc.year_fraction(today, call_exercise_date),
            put_maturity_time: dc.year_fraction(today, put_exercise_date),
        };

        let engine = AnalyticComplexChooserEngine::new(process);
        let calculated = engine.calculate(&args);
        let expected = 6.0508;
        let tolerance = 1e-4;

        assert!(
            (calculated - expected).abs() < tolerance,
            "complex chooser: expected {expected}, got {calculated}, error {}",
            (calculated - expected).abs()
        );
    }
}
