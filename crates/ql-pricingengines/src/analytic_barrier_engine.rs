//! Analytic barrier option pricing engine.
//!
//! Translates `ql/pricingengines/barrier/analyticbarrierengine.hpp`.
//!
//! Prices European single-barrier options using the closed-form solution
//! from Reiner & Rubinstein (1991).

use std::sync::Arc;

use ql_core::{errors::Result, Real};
use ql_instruments::{
    BarrierOptionArguments, BarrierType, OptionType, PricingEngine, PricingResults,
};
use ql_math::distributions::normal_cdf;
use ql_processes::GeneralizedBlackScholesProcess;

/// Analytic barrier option engine (Reiner-Rubinstein).
///
/// Prices single-barrier European vanilla options (knock-in and knock-out,
/// up and down) using a closed-form solution.
///
/// The price of a down-and-out call, for example, is:
///
/// $$C_{\text{do}} = C_{\text{BS}} - C_{\text{di}}$$
///
/// Corresponds to `QuantLib::AnalyticBarrierEngine`.
#[derive(Debug)]
pub struct AnalyticBarrierEngine {
    process: Arc<GeneralizedBlackScholesProcess>,
}

impl AnalyticBarrierEngine {
    /// Create a new engine with the given Black-Scholes process.
    pub fn new(process: Arc<GeneralizedBlackScholesProcess>) -> Self {
        Self { process }
    }
}

/// Closed-form barrier option price (Reiner-Rubinstein).
///
/// Handles all 8 barrier types: up/down × in/out × call/put.
pub fn analytic_barrier_price(
    option_type: OptionType,
    barrier_type: BarrierType,
    spot: Real,
    strike: Real,
    barrier: Real,
    rebate: Real,
    r: Real,
    q: Real,
    sigma: Real,
    t: Real,
) -> Real {
    if t <= 0.0 {
        let phi = option_type.sign();
        let intrinsic = (phi * (spot - strike)).max(0.0);
        // Check if barrier was hit
        match barrier_type {
            BarrierType::DownOut | BarrierType::UpOut => {
                return intrinsic; // survived
            }
            BarrierType::DownIn | BarrierType::UpIn => {
                return 0.0; // never knocked in
            }
        }
    }

    let sigma2 = sigma * sigma;
    let sqrt_t = t.sqrt();
    let mu = (r - q - 0.5 * sigma2) / sigma2;
    let lambda = (mu * mu * sigma2 + 2.0 * r).sqrt() / sigma;
    let z = (barrier / spot).ln() / (sigma * sqrt_t) + lambda * sigma * sqrt_t;

    let phi = option_type.sign(); // +1 call, -1 put
    let eta = match barrier_type {
        BarrierType::DownIn | BarrierType::DownOut => 1.0,
        BarrierType::UpIn | BarrierType::UpOut => -1.0,
    };

    let x1 = (spot / strike).ln() / (sigma * sqrt_t) + (1.0 + mu) * sigma * sqrt_t;
    let x2 = (spot / barrier).ln() / (sigma * sqrt_t) + (1.0 + mu) * sigma * sqrt_t;
    let y1 =
        (barrier * barrier / (spot * strike)).ln() / (sigma * sqrt_t) + (1.0 + mu) * sigma * sqrt_t;
    let y2 = (barrier / spot).ln() / (sigma * sqrt_t) + (1.0 + mu) * sigma * sqrt_t;

    let df_r = (-r * t).exp();
    let df_q = (-q * t).exp();

    // Components A through F from Reiner-Rubinstein
    let a = phi * spot * df_q * normal_cdf(phi * x1)
        - phi * strike * df_r * normal_cdf(phi * x1 - phi * sigma * sqrt_t);

    let b = phi * spot * df_q * normal_cdf(phi * x2)
        - phi * strike * df_r * normal_cdf(phi * x2 - phi * sigma * sqrt_t);

    let c = phi * spot * df_q * (barrier / spot).powf(2.0 * (mu + 1.0)) * normal_cdf(eta * y1)
        - phi
            * strike
            * df_r
            * (barrier / spot).powf(2.0 * mu)
            * normal_cdf(eta * y1 - eta * sigma * sqrt_t);

    let d = phi * spot * df_q * (barrier / spot).powf(2.0 * (mu + 1.0)) * normal_cdf(eta * y2)
        - phi
            * strike
            * df_r
            * (barrier / spot).powf(2.0 * mu)
            * normal_cdf(eta * y2 - eta * sigma * sqrt_t);

    let e = rebate
        * df_r
        * (normal_cdf(eta * x2 - eta * sigma * sqrt_t)
            - (barrier / spot).powf(2.0 * mu) * normal_cdf(eta * y2 - eta * sigma * sqrt_t));

    let f = rebate
        * ((barrier / spot).powf(mu + lambda) * normal_cdf(eta * z)
            + (barrier / spot).powf(mu - lambda)
                * normal_cdf(eta * z - 2.0 * eta * lambda * sigma * sqrt_t));

    // Combine based on barrier type and option type
    match (barrier_type, option_type) {
        // Down-and-in
        (BarrierType::DownIn, OptionType::Call) if strike >= barrier => c + e,
        (BarrierType::DownIn, OptionType::Call) => a - b + d + e,
        (BarrierType::DownIn, OptionType::Put) if strike >= barrier => b - c + d + e,
        (BarrierType::DownIn, OptionType::Put) => a + e,

        // Up-and-in
        (BarrierType::UpIn, OptionType::Call) if strike >= barrier => a + e,
        (BarrierType::UpIn, OptionType::Call) => b - c + d + e,
        (BarrierType::UpIn, OptionType::Put) if strike >= barrier => a - b + d + e,
        (BarrierType::UpIn, OptionType::Put) => c + e,

        // Down-and-out (= vanilla - down-and-in)
        (BarrierType::DownOut, OptionType::Call) if strike >= barrier => a - c + f,
        (BarrierType::DownOut, OptionType::Call) => b - d + f,
        (BarrierType::DownOut, OptionType::Put) if strike >= barrier => a - b + c - d + f,
        (BarrierType::DownOut, OptionType::Put) => f,

        // Up-and-out
        (BarrierType::UpOut, OptionType::Call) if strike >= barrier => f,
        (BarrierType::UpOut, OptionType::Call) => a - b + c - d + f,
        (BarrierType::UpOut, OptionType::Put) if strike >= barrier => b - d + f,
        (BarrierType::UpOut, OptionType::Put) => a - c + f,
    }
}

impl PricingEngine<BarrierOptionArguments> for AnalyticBarrierEngine {
    fn calculate(&self, args: &BarrierOptionArguments) -> Result<PricingResults> {
        let spot = self.process.spot();
        let strike = args.payoff.strike();
        let option_type = args.payoff.option_type();
        let expiry = args.exercise.last_date();

        let ref_date = self.process.risk_free_rate().reference_date();
        let dc = self.process.risk_free_rate().day_counter();
        let t = dc.year_fraction(ref_date, expiry);

        let r = self.process.risk_free_rate().zero_rate_impl(t);
        let q = self.process.dividend_yield().zero_rate_impl(t);
        let sigma = self
            .process
            .black_volatility()
            .expect("process must have a black vol surface")
            .black_vol_time(t, strike);

        let price = analytic_barrier_price(
            option_type,
            args.barrier_type,
            spot,
            strike,
            args.barrier,
            args.rebate,
            r,
            q,
            sigma,
            t,
        );

        Ok(PricingResults::from_npv(price))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytic_european_engine::black_scholes_merton;

    fn params() -> (Real, Real, Real, Real, Real, Real) {
        // spot, strike, r, q, sigma, t
        (100.0, 100.0, 0.05, 0.02, 0.20, 1.0)
    }

    #[test]
    fn in_plus_out_equals_vanilla_call() {
        let (s, k, r, q, sigma, t) = params();
        let barrier = 90.0; // down barrier
        let rebate = 0.0;

        let di = analytic_barrier_price(
            OptionType::Call,
            BarrierType::DownIn,
            s,
            k,
            barrier,
            rebate,
            r,
            q,
            sigma,
            t,
        );
        let do_ = analytic_barrier_price(
            OptionType::Call,
            BarrierType::DownOut,
            s,
            k,
            barrier,
            rebate,
            r,
            q,
            sigma,
            t,
        );
        let (vanilla, ..) = black_scholes_merton(OptionType::Call, s, k, r, q, sigma, t);

        assert!(
            (di + do_ - vanilla).abs() < 0.01,
            "di={di}, do={do_}, vanilla={vanilla}, sum={}",
            di + do_
        );
    }

    #[test]
    fn in_plus_out_equals_vanilla_put() {
        let (s, k, r, q, sigma, t) = params();
        let barrier = 110.0; // up barrier
        let rebate = 0.0;

        let ui = analytic_barrier_price(
            OptionType::Put,
            BarrierType::UpIn,
            s,
            k,
            barrier,
            rebate,
            r,
            q,
            sigma,
            t,
        );
        let uo = analytic_barrier_price(
            OptionType::Put,
            BarrierType::UpOut,
            s,
            k,
            barrier,
            rebate,
            r,
            q,
            sigma,
            t,
        );
        let (vanilla, ..) = black_scholes_merton(OptionType::Put, s, k, r, q, sigma, t);

        assert!(
            (ui + uo - vanilla).abs() < 0.01,
            "ui={ui}, uo={uo}, vanilla={vanilla}, sum={}",
            ui + uo
        );
    }

    #[test]
    fn down_and_out_call_less_than_vanilla() {
        let (s, k, r, q, sigma, t) = params();
        let barrier = 90.0;

        let do_ = analytic_barrier_price(
            OptionType::Call,
            BarrierType::DownOut,
            s,
            k,
            barrier,
            0.0,
            r,
            q,
            sigma,
            t,
        );
        let (vanilla, ..) = black_scholes_merton(OptionType::Call, s, k, r, q, sigma, t);

        assert!(do_ < vanilla, "do={do_}, vanilla={vanilla}");
        assert!(do_ > 0.0, "do={do_} should be positive");
    }

    #[test]
    fn up_and_out_call_with_rebate() {
        let (s, k, r, q, sigma, t) = params();
        let barrier = 120.0;
        let rebate = 3.0;

        let with_rebate = analytic_barrier_price(
            OptionType::Call,
            BarrierType::UpOut,
            s,
            k,
            barrier,
            rebate,
            r,
            q,
            sigma,
            t,
        );
        let without = analytic_barrier_price(
            OptionType::Call,
            BarrierType::UpOut,
            s,
            k,
            barrier,
            0.0,
            r,
            q,
            sigma,
            t,
        );

        assert!(
            with_rebate > without,
            "with_rebate={with_rebate}, without={without}"
        );
    }

    #[test]
    fn barrier_prices_positive() {
        let (s, k, r, q, sigma, t) = params();

        for &bt in &[BarrierType::DownIn, BarrierType::DownOut] {
            let p = analytic_barrier_price(OptionType::Call, bt, s, k, 80.0, 0.0, r, q, sigma, t);
            assert!(p >= 0.0, "barrier={bt:?}, price={p}");
        }
        for &bt in &[BarrierType::UpIn, BarrierType::UpOut] {
            let p = analytic_barrier_price(OptionType::Put, bt, s, k, 120.0, 0.0, r, q, sigma, t);
            assert!(p >= 0.0, "barrier={bt:?}, price={p}");
        }
    }
}
