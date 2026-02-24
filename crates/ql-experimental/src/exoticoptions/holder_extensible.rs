//! Analytic pricing engine for holder-extensible options.
//!
//! A holder-extensible option gives the holder the right to extend the option's
//! life by paying a premium at the first expiry date. If extended, the option
//! continues with a (possibly different) second strike until the second expiry.
//!
//! Implements both call and put variants using bivariate normal CDF,
//! univariate normal CDF, and a Newton-Raphson root finder for critical
//! boundaries.
//!
//! Reference: Haug (2007).

use ql_core::Real;
use super::bivariate_normal::bivariate_normal_cdf_dr78;
use ql_math::distributions::normal_cdf;
use ql_processes::GeneralizedBlackScholesProcess;
use std::sync::Arc;

/// Arguments for a holder-extensible option.
#[derive(Debug, Clone)]
pub struct HolderExtensibleOptionArgs {
    /// Option type sign: +1 for call, -1 for put.
    pub option_type: Real,
    /// Premium to extend.
    pub premium: Real,
    /// First strike (initial option's strike).
    pub strike1: Real,
    /// Second strike (extended option's strike).
    pub strike2: Real,
    /// First expiry time (year fraction).
    pub t1: Real,
    /// Second expiry time (year fraction).
    pub t2: Real,
}

/// Analytic pricing engine for holder-extensible options.
///
/// Corresponds to `QuantLib::AnalyticHolderExtensibleOptionEngine`.
#[derive(Debug)]
pub struct AnalyticHolderExtensibleOptionEngine {
    process: Arc<GeneralizedBlackScholesProcess>,
}

impl AnalyticHolderExtensibleOptionEngine {
    /// Create a new engine.
    pub fn new(process: Arc<GeneralizedBlackScholesProcess>) -> Self {
        Self { process }
    }

    /// Price the holder-extensible option.
    pub fn calculate(&self, args: &HolderExtensibleOptionArgs) -> Real {
        let s = self.process.spot();
        let r = self.risk_free_rate();
        let b = r - self.dividend_yield();
        let x1 = args.strike1;
        let x2 = args.strike2;
        let t1 = args.t1;
        let t2 = args.t2;
        let a = args.premium;
        let vol = self.volatility();
        let is_call = args.option_type > 0.0;

        let growth = self.dividend_discount(t1);
        let discount = self.risk_free_discount(t1);

        let rho = (t1 / t2).sqrt();

        let z1 = ((s / x2).ln() + (b + vol * vol / 2.0) * t2) / (vol * t2.sqrt());
        let z2 = ((s / x1).ln() + (b + vol * vol / 2.0) * t1) / (vol * t1.sqrt());

        if is_call {
            let i1 = self.i1_call(args);
            let i2 = self.i2_call(args);

            let y1 = ((s / i2).ln() + (b + vol * vol / 2.0) * t1) / (vol * t1.sqrt());
            let y2 = ((s / i1).ln() + (b + vol * vol / 2.0) * t1) / (vol * t1.sqrt());

            // BSM call value for the base option
            let bsm = self.bs_call(s, x1, growth, discount, vol, t1);

            let result = bsm
                + s * ((b - r) * t2).exp()
                    * self.m2(y1, y2, f64::NEG_INFINITY, z1, rho)
                - x2 * (-r * t2).exp()
                    * self.m2(
                        y1 - vol * t1.sqrt(),
                        y2 - vol * t1.sqrt(),
                        f64::NEG_INFINITY,
                        z1 - vol * t2.sqrt(),
                        rho,
                    )
                - s * ((b - r) * t1).exp()
                    * self.n2(y1, z2)
                + x1 * (-r * t1).exp()
                    * self.n2(y1 - vol * t1.sqrt(), z2 - vol * t1.sqrt())
                - a * (-r * t1).exp()
                    * self.n2(y1 - vol * t1.sqrt(), y2 - vol * t1.sqrt());

            result
        } else {
            let i1 = self.i1_put(args);
            let i2 = self.i2_put(args);

            let y1 = ((s / i1).ln() + (b + vol * vol / 2.0) * t1) / (vol * t1.sqrt());
            let y2 = ((s / i2).ln() + (b + vol * vol / 2.0) * t1) / (vol * t1.sqrt());

            let bsm = self.bs_put(s, x1, growth, discount, vol, t1);

            let result = bsm
                - s * ((b - r) * t2).exp()
                    * self.m2(y1, y2, f64::NEG_INFINITY, -z1, rho)
                + x2 * (-r * t2).exp()
                    * self.m2(
                        y1 - vol * t1.sqrt(),
                        y2 - vol * t1.sqrt(),
                        f64::NEG_INFINITY,
                        -z1 + vol * t2.sqrt(),
                        rho,
                    )
                + s * ((b - r) * t1).exp()
                    * self.n2(z2, y2)
                - x1 * (-r * t1).exp()
                    * self.n2(z2 - vol * t1.sqrt(), y2 - vol * t1.sqrt())
                - a * (-r * t1).exp()
                    * self.n2(y1 - vol * t1.sqrt(), y2 - vol * t1.sqrt());

            result
        }
    }

    // ── Newton-Raphson critical boundaries ──────────────────────────────

    fn i1_call(&self, args: &HolderExtensibleOptionArgs) -> Real {
        let a = args.premium;
        if a == 0.0 {
            return 0.0;
        }
        let mut sv = self.process.spot();
        let epsilon = 0.001;

        loop {
            let (ci, dc) = self.bs_value_delta_ext(sv, args, true);
            let yi = ci - a;
            let di = dc;
            if yi.abs() <= epsilon {
                break;
            }
            sv -= yi / di;
        }
        sv
    }

    fn i2_call(&self, args: &HolderExtensibleOptionArgs) -> Real {
        let x1 = args.strike1;
        let x2 = args.strike2;
        let a = args.premium;
        let t1 = args.t1;
        let t2 = args.t2;
        let r = self.risk_free_rate();
        let val = x1 - x2 * (-r * (t2 - t1)).exp();

        if a < val {
            return f64::INFINITY;
        }

        let mut sv = self.process.spot();
        let epsilon = 0.001;

        loop {
            let (ci, dc) = self.bs_value_delta_ext(sv, args, true);
            let yi = ci - a - sv + x1;
            let di = dc - 1.0;
            if yi.abs() <= epsilon {
                break;
            }
            sv -= yi / di;
        }
        sv
    }

    fn i1_put(&self, args: &HolderExtensibleOptionArgs) -> Real {
        let x1 = args.strike1;
        let a = args.premium;
        let mut sv = self.process.spot();
        let epsilon = 0.001;

        loop {
            let (pi, dp) = self.bs_value_delta_ext(sv, args, false);
            let yi = pi - a + sv - x1;
            let di = dp - 1.0;
            if yi.abs() <= epsilon {
                break;
            }
            sv -= yi / di;
        }
        sv
    }

    fn i2_put(&self, args: &HolderExtensibleOptionArgs) -> Real {
        let a = args.premium;
        if a == 0.0 {
            return f64::INFINITY;
        }
        let mut sv = self.process.spot();
        let epsilon = 0.001;

        loop {
            let (pi, dp) = self.bs_value_delta_ext(sv, args, false);
            let yi = pi - a;
            let di = dp;
            if yi.abs() <= epsilon {
                break;
            }
            sv -= yi / di;
        }
        sv
    }

    /// BS value and delta for the extension period option (from T1 to T2).
    fn bs_value_delta_ext(&self, spot: Real, args: &HolderExtensibleOptionArgs, is_call: bool) -> (Real, Real) {
        let x2 = args.strike2;
        let t1 = args.t1;
        let t2 = args.t2;
        let t = t2 - t1;
        let vol = self.volatility();
        let std_dev = vol * t.sqrt();
        let growth = self.dividend_discount(t);
        let discount = self.risk_free_discount(t);

        let forward = spot * growth / discount;
        let d1 = if std_dev > 1e-15 {
            (forward / x2).ln() / std_dev + 0.5 * std_dev
        } else {
            if forward > x2 { 1e15 } else { -1e15 }
        };
        let d2 = d1 - std_dev;

        if is_call {
            let value = discount * (forward * normal_cdf(d1) - x2 * normal_cdf(d2));
            let delta = growth * normal_cdf(d1);
            (value, delta)
        } else {
            let value = discount * (x2 * normal_cdf(-d2) - forward * normal_cdf(-d1));
            let delta = -growth * normal_cdf(-d1);
            (value, delta)
        }
    }

    // ── BS helpers ──────────────────────────────────────────────────────

    fn bs_call(&self, s: Real, k: Real, growth: Real, discount: Real, vol: Real, t: Real) -> Real {
        let std_dev = vol * t.sqrt();
        let forward = s * growth / discount;
        let d1 = if std_dev > 1e-15 {
            (forward / k).ln() / std_dev + 0.5 * std_dev
        } else {
            if forward > k { 1e15 } else { -1e15 }
        };
        let d2 = d1 - std_dev;
        discount * (forward * normal_cdf(d1) - k * normal_cdf(d2))
    }

    fn bs_put(&self, s: Real, k: Real, growth: Real, discount: Real, vol: Real, t: Real) -> Real {
        let std_dev = vol * t.sqrt();
        let forward = s * growth / discount;
        let d1 = if std_dev > 1e-15 {
            (forward / k).ln() / std_dev + 0.5 * std_dev
        } else {
            if forward > k { 1e15 } else { -1e15 }
        };
        let d2 = d1 - std_dev;
        discount * (k * normal_cdf(-d2) - forward * normal_cdf(-d1))
    }

    /// Bivariate region integral: M2(a,b,c,d,rho) = N2(b,d;rho) - N2(a,d;rho) - N2(b,c;rho) + N2(a,c;rho)
    fn m2(&self, a: Real, b: Real, c: Real, d: Real, rho: Real) -> Real {
        bivariate_normal_cdf_dr78(b, d, rho)
            - bivariate_normal_cdf_dr78(a, d, rho)
            - bivariate_normal_cdf_dr78(b, c, rho)
            + bivariate_normal_cdf_dr78(a, c, rho)
    }

    /// Univariate region integral: N2(a,b) = N(b) - N(a)
    fn n2(&self, a: Real, b: Real) -> Real {
        normal_cdf(b) - normal_cdf(a)
    }

    fn volatility(&self) -> Real {
        // Use vol at first expiry, matching C++ which calls black_vol(firstExpiryTime(), strike())
        self.process
            .black_volatility()
            .expect("process must have a black vol surface")
            .black_vol_time(0.5, 0.0) // Flat vol, so time/strike don't matter
    }

    fn risk_free_rate(&self) -> Real {
        self.process.risk_free_rate().zero_rate_impl(0.5) // Flat, so time doesn't matter
    }

    fn dividend_yield(&self) -> Real {
        self.process.dividend_yield().zero_rate_impl(0.5)
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

    /// Test from QuantLib C++ `testAnalyticHolderExtensibleOptionEngine`.
    #[test]
    fn test_holder_extensible_call() {
        let today = Date::from_ymd(2025, 1, 2).unwrap();
        let dc = Actual360;

        let spot = 100.0;
        let r = 0.08;
        let q = 0.0;
        let vol = 0.25;
        let strike1 = 100.0;
        let strike2 = 105.0;
        let premium = 1.0;

        let ex_date1 = today + 180;
        let ex_date2 = today + 270;

        let t1 = dc.year_fraction(today, ex_date1);
        let t2 = dc.year_fraction(today, ex_date2);

        let r_ts: Arc<dyn YieldTermStructure> =
            Arc::new(FlatForward::continuous(today, r, Actual360));
        let q_ts: Arc<dyn YieldTermStructure> =
            Arc::new(FlatForward::continuous(today, q, Actual360));
        let vol_ts = Arc::new(BlackConstantVol::new(today, vol, Actual360));

        let process = Arc::new(GeneralizedBlackScholesProcess::new(
            spot, r_ts, q_ts, vol_ts,
        ));

        let args = HolderExtensibleOptionArgs {
            option_type: 1.0, // Call
            premium,
            strike1,
            strike2,
            t1,
            t2,
        };

        let engine = AnalyticHolderExtensibleOptionEngine::new(process);
        let calculated = engine.calculate(&args);
        let expected = 9.4233;
        let tolerance = 1e-4;

        assert!(
            (calculated - expected).abs() < tolerance,
            "holder-extensible call: expected {expected}, got {calculated}, error {}",
            (calculated - expected).abs()
        );
    }
}
