//! Analytic European option engine (Black-Scholes-Merton).
//!
//! Translates `ql/pricingengines/vanilla/analyticeuropeanengine.hpp`.
//!
//! Prices European vanilla options using the closed-form Black-Scholes-Merton
//! formula. Computes NPV and first/second-order Greeks.

use ql_core::{errors::Result, Real};
use ql_instruments::{OptionType, PricingEngine, PricingResults, VanillaOptionArguments};
use ql_math::distributions::{normal_cdf, normal_pdf};
use ql_processes::GeneralizedBlackScholesProcess;

use std::sync::Arc;

/// Analytic pricing engine for European vanilla options.
///
/// Implements the Black-Scholes-Merton closed-form solution:
///
/// $$C = S e^{-qT} N(d_1) - K e^{-rT} N(d_2)$$
/// $$P = K e^{-rT} N(-d_2) - S e^{-qT} N(-d_1)$$
///
/// where $d_{1,2} = \frac{\ln(S/K) + (r - q \pm \sigma^2/2)T}{\sigma\sqrt{T}}$
///
/// Corresponds to `QuantLib::AnalyticEuropeanEngine`.
#[derive(Debug)]
pub struct AnalyticEuropeanEngine {
    process: Arc<GeneralizedBlackScholesProcess>,
}

impl AnalyticEuropeanEngine {
    /// Create a new engine with the given Black-Scholes process.
    pub fn new(process: Arc<GeneralizedBlackScholesProcess>) -> Self {
        Self { process }
    }
}

/// Compute Black-Scholes price and Greeks for a European option.
///
/// Returns `(price, delta, gamma, vega, theta, rho)`.
pub fn black_scholes_merton(
    option_type: OptionType,
    spot: Real,
    strike: Real,
    risk_free_rate: Real,
    dividend_yield: Real,
    volatility: Real,
    time_to_expiry: Real,
) -> (Real, Real, Real, Real, Real, Real) {
    let phi = option_type.sign();
    let t = time_to_expiry;

    if t <= 0.0 {
        let intrinsic = (phi * (spot - strike)).max(0.0);
        return (intrinsic, 0.0, 0.0, 0.0, 0.0, 0.0);
    }

    let r = risk_free_rate;
    let q = dividend_yield;
    let sigma = volatility;
    let sqrt_t = t.sqrt();
    let std_dev = sigma * sqrt_t;
    let df_r = (-r * t).exp();
    let df_q = (-q * t).exp();
    let fwd = spot * ((r - q) * t).exp();

    let (d1, d2) = if std_dev > 1e-15 {
        let d1 = ((spot / strike).ln() + (r - q + 0.5 * sigma * sigma) * t) / std_dev;
        let d2 = d1 - std_dev;
        (d1, d2)
    } else {
        let big = if fwd > strike { 1e15 } else { -1e15 };
        (big, big)
    };

    let nd1 = normal_cdf(phi * d1);
    let nd2 = normal_cdf(phi * d2);
    let npd1 = normal_pdf(d1);

    // Price
    let price = phi * (spot * df_q * nd1 - strike * df_r * nd2);
    // Delta
    let delta = phi * df_q * nd1;
    // Gamma
    let gamma = df_q * npd1 / (spot * std_dev);
    // Vega (per 1.0 absolute vol, not per 1%)
    let vega = spot * df_q * npd1 * sqrt_t;
    // Theta (per year)
    let theta = {
        let term1 = -(spot * df_q * npd1 * sigma) / (2.0 * sqrt_t);
        let term2 = -phi * r * strike * df_r * nd2;
        let term3 = phi * q * spot * df_q * nd1;
        term1 + term2 + term3
    };
    // Rho (per 1.0 rate shift)
    let rho = phi * strike * t * df_r * nd2;

    (price, delta, gamma, vega, theta, rho)
}

impl PricingEngine<VanillaOptionArguments> for AnalyticEuropeanEngine {
    fn calculate(&self, args: &VanillaOptionArguments) -> Result<PricingResults> {
        let spot = self.process.spot();
        let strike = args.payoff.strike();
        let option_type = args.payoff.option_type();
        let expiry = args.exercise.last_date();

        let ref_date = self.process.risk_free_rate().reference_date();
        let dc = self.process.risk_free_rate().day_counter();
        let t = dc.year_fraction(ref_date, expiry);

        // Continuous rates
        let r = self.process.risk_free_rate().zero_rate_impl(t);
        let q = self.process.dividend_yield().zero_rate_impl(t);

        // Black vol
        let sigma = self
            .process
            .black_volatility()
            .expect("process must have a black vol surface")
            .black_vol_time(t, strike);

        let (price, delta, gamma, vega, theta, rho) =
            black_scholes_merton(option_type, spot, strike, r, q, sigma, t);

        Ok(PricingResults::from_npv(price)
            .with_result("delta", delta)
            .with_result("gamma", gamma)
            .with_result("vega", vega)
            .with_result("theta", theta)
            .with_result("rho", rho))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bs_call_price() {
        // S=100, K=100, r=5%, q=0%, σ=20%, T=1
        let (price, delta, gamma, vega, _theta, rho) =
            black_scholes_merton(OptionType::Call, 100.0, 100.0, 0.05, 0.0, 0.20, 1.0);
        // Expected ≈ 10.45
        assert!((price - 10.4506).abs() < 0.01, "price = {price}");
        assert!(delta > 0.5 && delta < 0.8, "delta = {delta}");
        assert!(gamma > 0.0, "gamma = {gamma}");
        assert!(vega > 0.0, "vega = {vega}");
        assert!(rho > 0.0, "rho = {rho}");
    }

    #[test]
    fn bs_put_price() {
        // Put via put-call parity: P = C - S*exp(-qT) + K*exp(-rT)
        let (call, ..) = black_scholes_merton(OptionType::Call, 100.0, 100.0, 0.05, 0.0, 0.20, 1.0);
        let (put, ..) = black_scholes_merton(OptionType::Put, 100.0, 100.0, 0.05, 0.0, 0.20, 1.0);
        let parity = call - 100.0 + 100.0 * (-0.05_f64).exp();
        assert!((put - parity).abs() < 1e-10, "put={put}, parity={parity}");
    }

    #[test]
    fn bs_deep_itm_call() {
        let (price, delta, ..) =
            black_scholes_merton(OptionType::Call, 200.0, 100.0, 0.05, 0.0, 0.20, 1.0);
        assert!(price > 100.0, "price = {price}");
        assert!(delta > 0.95, "delta = {delta}");
    }

    #[test]
    fn bs_deep_otm_put() {
        let (price, delta, ..) =
            black_scholes_merton(OptionType::Put, 200.0, 100.0, 0.05, 0.0, 0.20, 1.0);
        assert!(price < 1.0, "price = {price}");
        assert!(delta > -0.05, "delta = {delta}");
    }

    #[test]
    fn bs_put_call_parity_with_dividends() {
        let s = 100.0;
        let k = 105.0;
        let r = 0.08;
        let q = 0.03;
        let sigma = 0.25;
        let t = 0.5;
        let (call, ..) = black_scholes_merton(OptionType::Call, s, k, r, q, sigma, t);
        let (put, ..) = black_scholes_merton(OptionType::Put, s, k, r, q, sigma, t);
        let parity = call - s * (-q * t).exp() + k * (-r * t).exp();
        assert!((put - parity).abs() < 1e-10, "put={put}, parity={parity}");
    }

    #[test]
    fn bs_zero_vol_call() {
        // Zero vol → max(S*exp(-qT) - K*exp(-rT), 0)
        let (price, ..) = black_scholes_merton(OptionType::Call, 100.0, 95.0, 0.05, 0.0, 0.0, 1.0);
        let expected = 100.0 - 95.0 * (-0.05_f64).exp();
        assert!(
            (price - expected).abs() < 0.01,
            "price={price}, expected={expected}"
        );
    }

    #[test]
    fn engine_with_process() {
        use ql_termstructures::{BlackConstantVol, FlatForward};
        use ql_time::{Actual365Fixed, Date};

        let ref_date = Date::from_ymd(2025, 1, 15).unwrap();
        let rf = Arc::new(FlatForward::continuous(ref_date, 0.05, Actual365Fixed));
        let div = Arc::new(FlatForward::continuous(ref_date, 0.0, Actual365Fixed));
        let vol = Arc::new(BlackConstantVol::new(ref_date, 0.20, Actual365Fixed));

        let process = Arc::new(GeneralizedBlackScholesProcess::new(100.0, rf, div, vol));
        let engine = AnalyticEuropeanEngine::new(process);

        let expiry = Date::from_ymd(2026, 1, 15).unwrap();
        let args = VanillaOptionArguments {
            payoff: Arc::new(ql_instruments::PlainVanillaPayoff::new(
                OptionType::Call,
                100.0,
            )),
            exercise: ql_instruments::Exercise::european(expiry),
        };

        let result = engine.calculate(&args).unwrap();
        assert!((result.npv - 10.45).abs() < 0.1, "npv = {}", result.npv);
        assert!(result.additional_results.contains_key("delta"));
        assert!(result.additional_results.contains_key("gamma"));
        assert!(result.additional_results.contains_key("vega"));
    }
}
