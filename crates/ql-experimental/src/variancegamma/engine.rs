//! Analytic Variance Gamma pricing engine for European vanilla options.
//!
//! Prices by integrating the Black-Scholes price over a Gamma distribution of
//! the subordinated time variable:
//!
//! ```text
//! V = ∫₀^∞ BS(s0 · exp(θx + ω·t + σ²x/2), σ√(x/t)·√t) · gamma_pdf(x; t/ν, ν) dx
//! ```
//!
//! where `ω = ln(1 − θν − σ²ν/2)/ν` is the martingale correction.

use ql_core::{errors::Result, Real};
use ql_instruments::{
    instrument::PricingResults, OptionType, PricingEngine, VanillaOptionArguments,
};
use ql_math::distributions::log_gamma;
use ql_math::integrals::{GaussKronrodAdaptive, Integrator, SimpsonIntegral};
use ql_processes::{StochasticProcess1D, VarianceGammaProcess};
use std::sync::Arc;

/// Analytic pricing engine for European options under the Variance Gamma model.
///
/// Corresponds to `QuantLib::VarianceGammaEngine`.
#[derive(Debug)]
pub struct VarianceGammaEngine {
    process: Arc<VarianceGammaProcess>,
    abs_err: Real,
}

impl VarianceGammaEngine {
    /// Create a new VG engine with the given absolute error target.
    pub fn new(process: Arc<VarianceGammaProcess>, absolute_error: Real) -> Self {
        Self {
            process,
            abs_err: absolute_error,
        }
    }

    /// Create with default tolerance of 1e-5.
    pub fn with_default_tolerance(process: Arc<VarianceGammaProcess>) -> Self {
        Self::new(process, 1e-5)
    }
}

impl PricingEngine<VanillaOptionArguments> for VarianceGammaEngine {
    fn calculate(&self, args: &VanillaOptionArguments) -> Result<PricingResults> {
        let exercise_date = args.exercise.last_date();
        let option_type = args.payoff.option_type();
        let strike = args.payoff.strike();

        let risk_free = self.process.risk_free_rate();
        let dividend = self.process.dividend_yield();

        let rf_dc = risk_free.day_counter();
        let t = rf_dc.year_fraction(risk_free.reference_date(), exercise_date);

        let risk_free_discount = risk_free.discount_impl(t);
        let dividend_discount = dividend.discount_impl(t);

        let s0 = self.process.x0().exp(); // process x0 is log(spot)
        let sigma = self.process.sigma;
        let nu = self.process.nu;
        let theta = self.process.theta;

        // Martingale correction
        let omega = (1.0 - theta * nu - 0.5 * sigma * sigma * nu).ln() / nu;

        // Precompute gamma PDF denominator:
        // gamma_pdf(x; shape=t/nu, scale=nu) = x^(t/nu-1) * exp(-x/nu) / (nu^(t/nu) * Gamma(t/nu))
        let shape = t / nu;
        let gamma_denom = (log_gamma(shape) + shape * nu.ln()).exp();

        // The integrand: BS price at adjusted spot/vol × gamma pdf
        let integrand = move |x: Real| -> Real {
            if x <= 0.0 {
                return 0.0;
            }

            // Adjusted spot: S0 * exp(theta*x + omega*t + sigma^2*x/2)
            let s0_adj = s0 * (theta * x + omega * t + sigma * sigma * x / 2.0).exp();
            // Adjusted volatility: sigma * sqrt(x/t) * sqrt(t) = sigma * sqrt(x)
            let vol_adj = sigma * x.sqrt();

            if vol_adj < 1e-20 || s0_adj < 1e-20 {
                return 0.0;
            }

            // Black-Scholes price using the adjusted parameters
            let bs_price = black_price(
                option_type,
                s0_adj,
                strike,
                risk_free_discount,
                dividend_discount,
                vol_adj,
            );

            // Gamma PDF (unnormalized shape, then divide by denominator)
            let gamp = (x.powf(shape - 1.0) * (-x / nu).exp()) / gamma_denom;

            bs_price * gamp
        };

        // Find a good upper limit: extend until integrand is negligibly small
        let mut infinity = 15.0 * (nu * t).sqrt();
        let target = self.abs_err * 1e-4;
        let mut val = integrand(infinity);
        let mut attempts = 0;
        while val.abs() > target && attempts < 50 {
            infinity *= 1.5;
            val = integrand(infinity);
            attempts += 1;
        }

        // Integrate using SimpsonIntegral (matching C++ QuantLib), with
        // GaussKronrod fallback for short-dated options with singularity at zero.
        let price = {
            let integrator = SimpsonIntegral::new(self.abs_err, 10000);
            match integrator.integrate(&integrand, 0.0, infinity) {
                Ok(v) => v,
                Err(_) => {
                    // Simpson failed (likely singularity near zero) — use adaptive Gauss-Kronrod
                    let gk = GaussKronrodAdaptive::new(self.abs_err, 100_000);
                    gk.integrate(&integrand, 0.0, infinity)?
                }
            }
        };

        Ok(PricingResults::from_npv(price))
    }
}

/// Simplified Black-Scholes price given adjusted spot, strike, discount factors,
/// and total standard deviation.
///
/// `vol` here is the *total* standard deviation (σ√t adjusted).
fn black_price(
    option_type: OptionType,
    spot_adj: Real,
    strike: Real,
    risk_free_discount: Real,
    dividend_discount: Real,
    vol: Real,
) -> Real {
    use ql_math::distributions::normal_cdf;

    let phi = option_type.sign();
    let forward = spot_adj * dividend_discount / risk_free_discount;

    if vol < 1e-15 {
        return risk_free_discount * (phi * (forward - strike)).max(0.0);
    }

    let d1 = ((forward / strike).ln() + 0.5 * vol * vol) / vol;
    let d2 = d1 - vol;

    risk_free_discount * phi * (forward * normal_cdf(phi * d1) - strike * normal_cdf(phi * d2))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ql_instruments::{Exercise, PlainVanillaPayoff};
    use ql_termstructures::{FlatForward, YieldTermStructure};
    use ql_time::{Actual360, Date};

    fn flat_ts(rate: Real, ref_date: Date) -> Arc<dyn YieldTermStructure> {
        Arc::new(FlatForward::continuous(ref_date, rate, Actual360))
    }

    /// Expected results from QuantLib C++ test suite for two VG process
    /// parameter sets × 22 vanilla options each.
    #[test]
    fn test_variance_gamma_analytic() {
        let today = Date::from_ymd(2025, 1, 2).unwrap();

        // Process set 1: spot=6000, q=0, r=0.05, sigma=0.2, nu=0.05, theta=-0.5
        // Process set 2: spot=6000, q=0.02, r=0.05, sigma=0.15, nu=0.01, theta=-0.5
        struct ProcessData {
            s: Real,
            q: Real,
            r: Real,
            sigma: Real,
            nu: Real,
            theta: Real,
        }
        let processes = [
            ProcessData {
                s: 6000.0,
                q: 0.00,
                r: 0.05,
                sigma: 0.20,
                nu: 0.05,
                theta: -0.50,
            },
            ProcessData {
                s: 6000.0,
                q: 0.02,
                r: 0.05,
                sigma: 0.15,
                nu: 0.01,
                theta: -0.50,
            },
        ];

        struct OptionData {
            opt_type: OptionType,
            strike: Real,
        }
        let options = [
            OptionData {
                opt_type: OptionType::Call,
                strike: 5550.0,
            },
            OptionData {
                opt_type: OptionType::Call,
                strike: 5600.0,
            },
            OptionData {
                opt_type: OptionType::Call,
                strike: 5650.0,
            },
            OptionData {
                opt_type: OptionType::Call,
                strike: 5700.0,
            },
            OptionData {
                opt_type: OptionType::Call,
                strike: 5750.0,
            },
            OptionData {
                opt_type: OptionType::Call,
                strike: 5800.0,
            },
            OptionData {
                opt_type: OptionType::Call,
                strike: 5850.0,
            },
            OptionData {
                opt_type: OptionType::Call,
                strike: 5900.0,
            },
            OptionData {
                opt_type: OptionType::Call,
                strike: 5950.0,
            },
            OptionData {
                opt_type: OptionType::Call,
                strike: 6000.0,
            },
            OptionData {
                opt_type: OptionType::Call,
                strike: 6050.0,
            },
            OptionData {
                opt_type: OptionType::Call,
                strike: 6100.0,
            },
            OptionData {
                opt_type: OptionType::Call,
                strike: 6150.0,
            },
            OptionData {
                opt_type: OptionType::Call,
                strike: 6200.0,
            },
            OptionData {
                opt_type: OptionType::Call,
                strike: 6250.0,
            },
            OptionData {
                opt_type: OptionType::Call,
                strike: 6300.0,
            },
            OptionData {
                opt_type: OptionType::Call,
                strike: 6350.0,
            },
            OptionData {
                opt_type: OptionType::Call,
                strike: 6400.0,
            },
            OptionData {
                opt_type: OptionType::Call,
                strike: 6450.0,
            },
            OptionData {
                opt_type: OptionType::Call,
                strike: 6500.0,
            },
            OptionData {
                opt_type: OptionType::Call,
                strike: 6550.0,
            },
            OptionData {
                opt_type: OptionType::Put,
                strike: 5550.0,
            },
        ];

        // Expected values from the C++ QuantLib test suite
        let expected: [[Real; 22]; 2] = [
            [
                955.1637, 922.7529, 890.9872, 859.8739, 829.4197, 799.6303, 770.5104, 742.0640,
                714.2943, 687.2032, 660.7921, 635.0613, 610.0103, 585.6379, 561.9416, 538.9186,
                516.5649, 494.8760, 473.8464, 453.4700, 433.7400, 234.4870,
            ],
            [
                732.8705, 698.5542, 665.1404, 632.6498, 601.1002, 570.5068, 540.8824, 512.2367,
                484.5766, 457.9064, 432.2273, 407.5381, 383.8346, 361.1102, 339.3559, 318.5599,
                298.7087, 279.7864, 261.7751, 244.6552, 228.4057, 130.9974,
            ],
        ];

        let tol = 0.01;

        for (i, pd) in processes.iter().enumerate() {
            let spot_ts = flat_ts(pd.q, today);
            let risk_free_ts = flat_ts(pd.r, today);

            let process = Arc::new(VarianceGammaProcess::new(
                pd.s,
                risk_free_ts,
                spot_ts,
                pd.sigma,
                pd.nu,
                pd.theta,
            ));

            let engine = VarianceGammaEngine::new(process, 1e-5);

            for (j, od) in options.iter().enumerate() {
                // Create exercise date ~ 1 year from today
                let ex_date = today.advance(360, ql_time::TimeUnit::Days).unwrap();
                let exercise = Exercise::european(ex_date);
                let payoff: Arc<dyn ql_instruments::StrikedPayoff> =
                    Arc::new(PlainVanillaPayoff::new(od.opt_type, od.strike));

                let args = VanillaOptionArguments { payoff, exercise };
                let result = engine.calculate(&args).unwrap();
                let calculated = result.npv;
                let exp = expected[i][j];
                let error = (calculated - exp).abs();
                assert!(
                    error <= tol,
                    "Process {i}, Option {j}: expected {exp}, got {calculated}, error {error} > tol {tol}"
                );
            }
        }
    }

    /// Test that the engine doesn't hang on a very short-dated option
    /// (singularity-at-zero test from C++ suite).
    #[test]
    fn test_singularity_at_zero() {
        let today = Date::from_ymd(2017, 1, 1).unwrap();
        let maturity = Date::from_ymd(2017, 1, 10).unwrap();

        let risk_free = flat_ts(0.05, today);
        let dividend = flat_ts(0.0, today);

        let process = Arc::new(VarianceGammaProcess::new(
            100.0, risk_free, dividend, 0.12,  // sigma
            0.2,   // nu (kappa in C++ test)
            -0.14, // theta (mu in C++ test)
        ));

        let engine = VarianceGammaEngine::with_default_tolerance(process);
        let exercise = Exercise::european(maturity);
        let payoff: Arc<dyn ql_instruments::StrikedPayoff> =
            Arc::new(PlainVanillaPayoff::new(OptionType::Call, 98.0));

        let args = VanillaOptionArguments { payoff, exercise };
        // Just verify it completes without hanging
        let result = engine.calculate(&args).unwrap();
        assert!(result.npv >= 0.0);
        // It should be a small positive number for a near-ATM short-dated call
        assert!(result.npv > 0.0 && result.npv < 50.0);
    }
}
