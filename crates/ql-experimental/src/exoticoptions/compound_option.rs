//! Analytic pricing engine for compound options (option on an option).
//!
//! A compound option has a "mother" option whose underlying is a "daughter"
//! European option. All four type combinations are supported (call-on-call,
//! call-on-put, put-on-call, put-on-put).
//!
//! Implements the Geske (1979) closed-form formula using the bivariate normal
//! distribution.
//!
//! Reference: Haug (2007); Wystup "Foreign Exchange Risk" p. 81.

use super::bivariate_normal::bivariate_normal_cdf_dr78;
use ql_core::Real;
use ql_math::distributions::{normal_cdf, normal_pdf};
use ql_math::solvers1d::brent;
use ql_processes::GeneralizedBlackScholesProcess;
use std::sync::Arc;

/// Arguments for a compound option.
#[derive(Debug, Clone)]
pub struct CompoundOptionArgs {
    /// Mother option type sign: +1 for call, -1 for put.
    pub type_mother: Real,
    /// Daughter option type sign: +1 for call, -1 for put.
    pub type_daughter: Real,
    /// Strike of the mother option.
    pub strike_mother: Real,
    /// Strike of the daughter option.
    pub strike_daughter: Real,
    /// Time to maturity of the mother option (year fraction).
    pub t_mother: Real,
    /// Time to maturity of the daughter option (year fraction).
    pub t_daughter: Real,
}

/// Analytic pricing engine for compound options.
///
/// Returns NPV and Greeks (delta, gamma, vega, theta) via the `calculate` method.
///
/// Corresponds to `QuantLib::AnalyticCompoundOptionEngine`.
#[derive(Debug)]
pub struct AnalyticCompoundOptionEngine {
    process: Arc<GeneralizedBlackScholesProcess>,
}

/// Result of compound option pricing.
#[derive(Debug, Clone)]
pub struct CompoundOptionResult {
    /// Net present value.
    pub npv: Real,
    /// Delta (∂V/∂S).
    pub delta: Real,
    /// Gamma (∂²V/∂S²).
    pub gamma: Real,
    /// Vega (∂V/∂σ).
    pub vega: Real,
    /// Theta (∂V/∂t), annualized.
    pub theta: Real,
}

impl AnalyticCompoundOptionEngine {
    /// Create a new engine with the given Black-Scholes process.
    pub fn new(process: Arc<GeneralizedBlackScholesProcess>) -> Self {
        Self { process }
    }

    /// Price the compound option and compute Greeks.
    pub fn calculate(&self, args: &CompoundOptionArgs) -> CompoundOptionResult {
        let spot = self.process.spot();
        let phi = args.type_daughter; // +1 call, -1 put
        let w = args.type_mother; // +1 call, -1 put
        let strike_m = args.strike_mother;
        let strike_d = args.strike_daughter;
        let t_m = args.t_mother;
        let t_d = args.t_daughter;

        assert!(spot > 0.0, "spot must be positive");
        assert!(strike_m > 0.0, "mother strike must be positive");
        assert!(strike_d > 0.0, "daughter strike must be positive");

        // Volatilities
        let vol_d = self.vol_at(t_d, strike_d);
        let vol_m = self.vol_at(t_m, strike_d);

        // Standard deviations
        let sd_d = vol_d * t_d.sqrt();
        let sd_m = vol_m * t_m.sqrt();

        // Discount factors
        let dd_d = self.dividend_discount(t_d);
        let rd_d = self.risk_free_discount(t_d);
        let rd_m = self.risk_free_discount(t_m);

        // Continuous rates
        let r_d = self.risk_free_rate(t_d);
        let d_d = self.dividend_rate(t_d);

        // Solve for the critical underlying price S* where daughter option value = strike_mother.
        let s_solved = self.solve_for_critical_spot(args, vol_d, t_d, t_m);

        // Transform S* to the standardized variable X
        let x = self.transform_x(s_solved, sd_m, dd_d, rd_d, t_m);

        // Correlation
        let rho = (t_m / t_d).sqrt();

        // d+ and d-
        let forward_d = spot * dd_d / rd_d;
        let d_plus = (forward_d / strike_d).ln() / sd_d + 0.5 * sd_d;
        let d_minus = d_plus - sd_d;

        // d+ for daughter from T1 to T2 evaluated at S*
        let dd_md = self.dividend_discount_between(t_m, t_d);
        let rd_md = self.risk_free_discount_between(t_m, t_d);
        let t_md = t_d - t_m;
        let sd_md = vol_d * t_md.sqrt();
        let forward_md = s_solved * dd_md / rd_md;
        let d_plus_t12 = (forward_md / strike_d).ln() / sd_md + 0.5 * sd_md;

        let x_m_sm = x - sd_m;

        // Bivariate normal evaluations
        let n2_xmsm = bivariate_normal_cdf_dr78(-phi * w * x_m_sm, phi * d_plus, w * rho);
        let n2_x = bivariate_normal_cdf_dr78(-phi * w * x, phi * d_minus, w * rho);

        // Helper function e(X)
        let e_x = (x * t_d.sqrt() + t_m.sqrt() * d_minus) / t_md.sqrt();

        let ne_x = normal_cdf(-phi * w * e_x);
        let nx = normal_cdf(-phi * w * x);
        let nt12 = normal_cdf(phi * d_plus_t12);
        let nd_plus = normal_pdf(d_plus);
        let n_xm = normal_pdf(x_m_sm);

        let inv_m_time = 1.0 / t_m.sqrt();
        let inv_d_time = 1.0 / t_d.sqrt();

        // Value
        let value = phi * w * spot * dd_d * n2_xmsm
            - phi * w * strike_d * rd_d * n2_x
            - w * strike_m * rd_m * nx;

        // Delta
        let delta = phi * w * dd_d * n2_xmsm;

        // Gamma
        let gamma =
            (dd_d / (vol_d * spot)) * (inv_m_time * n_xm * nt12 + w * inv_d_time * nd_plus * ne_x);

        // Vega
        let vega = dd_d * spot * (t_m.sqrt() * n_xm * nt12 + w * t_d.sqrt() * nd_plus * ne_x);

        // Theta
        let mut theta = phi * w * d_d * spot * dd_d * n2_xmsm
            - phi * w * r_d * strike_d * rd_d * n2_x
            - w * r_d * strike_m * rd_m * nx;
        theta -= 0.5
            * vol_d
            * spot
            * dd_d
            * (inv_m_time * n_xm * nt12 + w * inv_d_time * nd_plus * ne_x);

        CompoundOptionResult {
            npv: value,
            delta,
            gamma,
            vega,
            theta,
        }
    }

    /// Solve for the critical spot S* where the daughter option value equals the mother strike.
    fn solve_for_critical_spot(
        &self,
        args: &CompoundOptionArgs,
        vol_d: Real,
        t_d: Real,
        t_m: Real,
    ) -> Real {
        let t_md = t_d - t_m;
        let sd_md = vol_d * t_md.sqrt();
        let dd_md = self.dividend_discount_between(t_m, t_d);
        let rd_md = self.risk_free_discount_between(t_m, t_d);
        let phi = args.type_daughter;
        let strike_d = args.strike_daughter;
        let strike_m = args.strike_mother;

        let f = |s: Real| -> Real {
            let forward = s * dd_md / rd_md;
            let d1 = if sd_md > 1e-15 {
                (forward / strike_d).ln() / sd_md + 0.5 * sd_md
            } else if forward > strike_d {
                1e15
            } else {
                -1e15
            };
            let d2 = d1 - sd_md;
            let val = phi
                * (forward * rd_md * normal_cdf(phi * d1)
                    - strike_d * rd_md * normal_cdf(phi * d2));
            val - strike_m
        };

        // Use Brent solver
        brent(f, 1.0e-6, strike_d * 1000.0, 1.0e-6).unwrap_or(strike_d)
    }

    /// Transform S* to the standardized variable X (as in Wystup's book).
    fn transform_x(&self, s_solved: Real, sd_m: Real, _dd_d: Real, _rd_d: Real, t_m: Real) -> Real {
        let spot = self.process.spot();
        let dd_m = self.dividend_discount(t_m);
        let rd_m = self.risk_free_discount(t_m);

        let res = rd_m * s_solved / (spot * dd_m);
        let res = res * (0.5 * sd_m * sd_m).exp();
        res.ln() / sd_m
    }

    fn vol_at(&self, t: Real, strike: Real) -> Real {
        self.process
            .black_volatility()
            .expect("process must have a black vol surface")
            .black_vol_time(t, strike)
    }

    fn risk_free_rate(&self, t: Real) -> Real {
        self.process.risk_free_rate().zero_rate_impl(t)
    }

    fn dividend_rate(&self, t: Real) -> Real {
        self.process.dividend_yield().zero_rate_impl(t)
    }

    fn risk_free_discount(&self, t: Real) -> Real {
        self.process.risk_free_rate().discount(t)
    }

    fn dividend_discount(&self, t: Real) -> Real {
        self.process.dividend_yield().discount(t)
    }

    fn risk_free_discount_between(&self, t1: Real, t2: Real) -> Real {
        self.risk_free_discount(t2) / self.risk_free_discount(t1)
    }

    fn dividend_discount_between(&self, t1: Real, t2: Real) -> Real {
        self.dividend_discount(t2) / self.dividend_discount(t1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ql_instruments::OptionType;
    use ql_termstructures::{BlackConstantVol, FlatForward, YieldTermStructure};
    use ql_time::{Actual360, Date};

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

    fn _time_to_days(t: Real) -> i32 {
        (t * 360.0 + 0.5) as i32
    }

    /// Test compound option values and Greeks from Haug (2007) / sitmo.com / mathfinance VBA.
    /// Matches QuantLib C++ test `testValues`.
    #[test]
    fn test_compound_option_values() {
        struct CompoundOptionData {
            type_mother: OptionType,
            type_daughter: OptionType,
            strike_mother: Real,
            strike_daughter: Real,
            s: Real,
            q: Real,
            r: Real,
            t_mother: Real,
            t_daughter: Real,
            v: Real,
            npv: Real,
            tol: Real,
            delta: Real,
            gamma: Real,
            vega: Real,
            theta: Real,
        }

        let values = [
            CompoundOptionData {
                type_mother: OptionType::Put,
                type_daughter: OptionType::Call,
                strike_mother: 50.0,
                strike_daughter: 520.0,
                s: 500.0,
                q: 0.03,
                r: 0.08,
                t_mother: 0.25,
                t_daughter: 0.5,
                v: 0.35,
                npv: 21.1965,
                tol: 1.0e-3,
                delta: -0.1966,
                gamma: 0.0007,
                vega: -32.1241,
                theta: -3.3837,
            },
            CompoundOptionData {
                type_mother: OptionType::Call,
                type_daughter: OptionType::Call,
                strike_mother: 50.0,
                strike_daughter: 520.0,
                s: 500.0,
                q: 0.03,
                r: 0.08,
                t_mother: 0.25,
                t_daughter: 0.5,
                v: 0.35,
                npv: 17.5945,
                tol: 1.0e-3,
                delta: 0.3219,
                gamma: 0.0038,
                vega: 106.5185,
                theta: -65.1614,
            },
            CompoundOptionData {
                type_mother: OptionType::Call,
                type_daughter: OptionType::Put,
                strike_mother: 50.0,
                strike_daughter: 520.0,
                s: 500.0,
                q: 0.03,
                r: 0.08,
                t_mother: 0.25,
                t_daughter: 0.5,
                v: 0.35,
                npv: 18.7128,
                tol: 1.0e-3,
                delta: -0.2906,
                gamma: 0.0036,
                vega: 103.3856,
                theta: -46.6982,
            },
            CompoundOptionData {
                type_mother: OptionType::Put,
                type_daughter: OptionType::Put,
                strike_mother: 50.0,
                strike_daughter: 520.0,
                s: 500.0,
                q: 0.03,
                r: 0.08,
                t_mother: 0.25,
                t_daughter: 0.5,
                v: 0.35,
                npv: 15.2601,
                tol: 1.0e-3,
                delta: 0.1760,
                gamma: 0.0005,
                vega: -35.2570,
                theta: -10.1126,
            },
            CompoundOptionData {
                type_mother: OptionType::Call,
                type_daughter: OptionType::Call,
                strike_mother: 0.05,
                strike_daughter: 1.14,
                s: 1.20,
                q: 0.0,
                r: 0.01,
                t_mother: 0.5,
                t_daughter: 2.0,
                v: 0.11,
                npv: 0.0729,
                tol: 1.0e-3,
                delta: 0.6614,
                gamma: 2.5762,
                vega: 0.5812,
                theta: -0.0297,
            },
            CompoundOptionData {
                type_mother: OptionType::Call,
                type_daughter: OptionType::Put,
                strike_mother: 0.05,
                strike_daughter: 1.14,
                s: 1.20,
                q: 0.0,
                r: 0.01,
                t_mother: 0.5,
                t_daughter: 2.0,
                v: 0.11,
                npv: 0.0074,
                tol: 1.0e-3,
                delta: -0.1334,
                gamma: 1.9681,
                vega: 0.2933,
                theta: -0.0155,
            },
            CompoundOptionData {
                type_mother: OptionType::Put,
                type_daughter: OptionType::Call,
                strike_mother: 0.05,
                strike_daughter: 1.14,
                s: 1.20,
                q: 0.0,
                r: 0.01,
                t_mother: 0.5,
                t_daughter: 2.0,
                v: 0.11,
                npv: 0.0021,
                tol: 1.0e-3,
                delta: -0.0426,
                gamma: 0.7252,
                vega: -0.0052,
                theta: -0.0058,
            },
            CompoundOptionData {
                type_mother: OptionType::Put,
                type_daughter: OptionType::Put,
                strike_mother: 0.05,
                strike_daughter: 1.14,
                s: 1.20,
                q: 0.0,
                r: 0.01,
                t_mother: 0.5,
                t_daughter: 2.0,
                v: 0.11,
                npv: 0.0192,
                tol: 1.0e-3,
                delta: 0.1626,
                gamma: 0.1171,
                vega: -0.2931,
                theta: -0.0028,
            },
            CompoundOptionData {
                type_mother: OptionType::Call,
                type_daughter: OptionType::Call,
                strike_mother: 10.0,
                strike_daughter: 122.0,
                s: 120.0,
                q: 0.06,
                r: 0.02,
                t_mother: 0.1,
                t_daughter: 0.7,
                v: 0.22,
                npv: 0.4419,
                tol: 1.0e-3,
                delta: 0.1049,
                gamma: 0.0195,
                vega: 11.3368,
                theta: -6.2871,
            },
            CompoundOptionData {
                type_mother: OptionType::Call,
                type_daughter: OptionType::Put,
                strike_mother: 10.0,
                strike_daughter: 122.0,
                s: 120.0,
                q: 0.06,
                r: 0.02,
                t_mother: 0.1,
                t_daughter: 0.7,
                v: 0.22,
                npv: 2.6112,
                tol: 1.0e-3,
                delta: -0.3618,
                gamma: 0.0337,
                vega: 28.4843,
                theta: -13.4124,
            },
            CompoundOptionData {
                type_mother: OptionType::Put,
                type_daughter: OptionType::Call,
                strike_mother: 10.0,
                strike_daughter: 122.0,
                s: 120.0,
                q: 0.06,
                r: 0.02,
                t_mother: 0.1,
                t_daughter: 0.7,
                v: 0.22,
                npv: 4.1616,
                tol: 1.0e-3,
                delta: -0.3174,
                gamma: 0.0024,
                vega: -26.6403,
                theta: -2.2720,
            },
            CompoundOptionData {
                type_mother: OptionType::Put,
                type_daughter: OptionType::Put,
                strike_mother: 10.0,
                strike_daughter: 122.0,
                s: 120.0,
                q: 0.06,
                r: 0.02,
                t_mother: 0.1,
                t_daughter: 0.7,
                v: 0.22,
                npv: 1.0914,
                tol: 1.0e-3,
                delta: 0.1748,
                gamma: 0.0165,
                vega: -9.4928,
                theta: -4.8995,
            },
            CompoundOptionData {
                type_mother: OptionType::Call,
                type_daughter: OptionType::Call,
                strike_mother: 0.4,
                strike_daughter: 8.2,
                s: 8.0,
                q: 0.05,
                r: 0.00,
                t_mother: 2.0,
                t_daughter: 3.0,
                v: 0.08,
                npv: 0.0099,
                tol: 1.0e-3,
                delta: 0.0285,
                gamma: 0.0688,
                vega: 0.7764,
                theta: -0.0027,
            },
            CompoundOptionData {
                type_mother: OptionType::Call,
                type_daughter: OptionType::Put,
                strike_mother: 0.4,
                strike_daughter: 8.2,
                s: 8.0,
                q: 0.05,
                r: 0.00,
                t_mother: 2.0,
                t_daughter: 3.0,
                v: 0.08,
                npv: 0.9826,
                tol: 1.0e-3,
                delta: -0.7224,
                gamma: 0.2158,
                vega: 2.7279,
                theta: -0.3332,
            },
            CompoundOptionData {
                type_mother: OptionType::Put,
                type_daughter: OptionType::Call,
                strike_mother: 0.4,
                strike_daughter: 8.2,
                s: 8.0,
                q: 0.05,
                r: 0.00,
                t_mother: 2.0,
                t_daughter: 3.0,
                v: 0.08,
                npv: 0.3585,
                tol: 1.0e-3,
                delta: -0.0720,
                gamma: -0.0835,
                vega: -1.5633,
                theta: -0.0117,
            },
            CompoundOptionData {
                type_mother: OptionType::Put,
                type_daughter: OptionType::Put,
                strike_mother: 0.4,
                strike_daughter: 8.2,
                s: 8.0,
                q: 0.05,
                r: 0.00,
                t_mother: 2.0,
                t_daughter: 3.0,
                v: 0.08,
                npv: 0.0168,
                tol: 1.0e-3,
                delta: 0.0378,
                gamma: 0.0635,
                vega: 0.3882,
                theta: 0.0021,
            },
            CompoundOptionData {
                type_mother: OptionType::Call,
                type_daughter: OptionType::Call,
                strike_mother: 0.02,
                strike_daughter: 1.6,
                s: 1.6,
                q: 0.013,
                r: 0.022,
                t_mother: 0.45,
                t_daughter: 0.5,
                v: 0.17,
                npv: 0.0680,
                tol: 1.0e-3,
                delta: 0.4937,
                gamma: 2.1271,
                vega: 0.4418,
                theta: -0.0843,
            },
            CompoundOptionData {
                type_mother: OptionType::Call,
                type_daughter: OptionType::Put,
                strike_mother: 0.02,
                strike_daughter: 1.6,
                s: 1.6,
                q: 0.013,
                r: 0.022,
                t_mother: 0.45,
                t_daughter: 0.5,
                v: 0.17,
                npv: 0.0605,
                tol: 1.0e-3,
                delta: -0.4169,
                gamma: 2.0836,
                vega: 0.4330,
                theta: -0.0697,
            },
            CompoundOptionData {
                type_mother: OptionType::Put,
                type_daughter: OptionType::Call,
                strike_mother: 0.02,
                strike_daughter: 1.6,
                s: 1.6,
                q: 0.013,
                r: 0.022,
                t_mother: 0.45,
                t_daughter: 0.5,
                v: 0.17,
                npv: 0.0081,
                tol: 1.0e-3,
                delta: -0.0417,
                gamma: 0.0761,
                vega: -0.0045,
                theta: -0.0020,
            },
            CompoundOptionData {
                type_mother: OptionType::Put,
                type_daughter: OptionType::Put,
                strike_mother: 0.02,
                strike_daughter: 1.6,
                s: 1.6,
                q: 0.013,
                r: 0.022,
                t_mother: 0.45,
                t_daughter: 0.5,
                v: 0.17,
                npv: 0.0078,
                tol: 1.0e-3,
                delta: 0.0413,
                gamma: 0.0326,
                vega: -0.0133,
                theta: -0.0016,
            },
        ];

        let today = Date::from_ymd(2025, 1, 2).unwrap();
        let _dc = Actual360;

        for (i, v) in values.iter().enumerate() {
            let process = make_process(today, v.s, v.q, v.r, v.v);

            let args = CompoundOptionArgs {
                type_mother: v.type_mother.sign(),
                type_daughter: v.type_daughter.sign(),
                strike_mother: v.strike_mother,
                strike_daughter: v.strike_daughter,
                t_mother: v.t_mother,
                t_daughter: v.t_daughter,
            };

            let engine = AnalyticCompoundOptionEngine::new(process);
            let result = engine.calculate(&args);

            let check = |name: &str, calc: Real, exp: Real| {
                assert!(
                    (calc - exp).abs() < v.tol,
                    "case {i}: {name}: expected {exp}, got {calc}, error {}",
                    (calc - exp).abs()
                );
            };

            check("npv", result.npv, v.npv);
            check("delta", result.delta, v.delta);
            check("gamma", result.gamma, v.gamma);
            check("vega", result.vega, v.vega);
            check("theta", result.theta, v.theta);
        }
    }
}
