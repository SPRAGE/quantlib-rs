//! Barone-Adesi-Whaley American option approximation.
//!
//! Translates `ql/pricingengines/vanilla/baroneadesiwhaleyengine.hpp`.
//!
//! Provides a fast quadratic approximation for American vanilla option prices.
//! This is one of the most popular analytic approximations for American options.

use std::sync::Arc;

use ql_core::{errors::Result, Real};
use ql_instruments::{OptionType, PricingEngine, PricingResults, VanillaOptionArguments};
use ql_math::distributions::{normal_cdf, normal_pdf};
use ql_processes::GeneralizedBlackScholesProcess;

use crate::analytic_european_engine::black_scholes_merton;

/// Barone-Adesi-Whaley American option pricing engine.
///
/// Uses the quadratic approximation method from Barone-Adesi & Whaley (1987)
/// which extends the Black-Scholes formula to American-style exercise.
///
/// Corresponds to `QuantLib::BaroneAdesiWhaleyApproximationEngine`.
#[derive(Debug)]
pub struct BaroneAdesiWhaleyEngine {
    process: Arc<GeneralizedBlackScholesProcess>,
}

impl BaroneAdesiWhaleyEngine {
    /// Create a new engine with the given Black-Scholes process.
    pub fn new(process: Arc<GeneralizedBlackScholesProcess>) -> Self {
        Self { process }
    }
}

fn d1(s: Real, k: Real, r: Real, q: Real, sigma: Real, t: Real) -> Real {
    ((s / k).ln() + (r - q + 0.5 * sigma * sigma) * t) / (sigma * t.sqrt())
}

/// Barone-Adesi-Whaley American option price.
pub fn barone_adesi_whaley(
    option_type: OptionType,
    spot: Real,
    strike: Real,
    r: Real,
    q: Real,
    sigma: Real,
    t: Real,
) -> Real {
    if t <= 0.0 {
        let phi = option_type.sign();
        return (phi * (spot - strike)).max(0.0);
    }

    let (european, ..) = black_scholes_merton(option_type, spot, strike, r, q, sigma, t);

    let sigma2 = sigma * sigma;
    let m = 2.0 * r / sigma2;
    let n = 2.0 * (r - q) / sigma2;
    let big_k = 1.0 - (-r * t).exp();

    if big_k.abs() < 1e-15 {
        return european;
    }

    match option_type {
        OptionType::Call => baw_call(spot, strike, r, q, sigma, t, european, m, n, big_k),
        OptionType::Put => baw_put(spot, strike, r, q, sigma, t, european, m, n, big_k),
    }
}

fn baw_call(
    spot: Real,
    strike: Real,
    r: Real,
    q: Real,
    sigma: Real,
    t: Real,
    european: Real,
    m: Real,
    n: Real,
    big_k: Real,
) -> Real {
    let q2 = (-(n - 1.0) + ((n - 1.0) * (n - 1.0) + 4.0 * m / big_k).sqrt()) / 2.0;

    if q2 <= 1.0 {
        return european;
    }

    let s_star = find_critical_call(strike, r, q, sigma, t, q2);

    if spot >= s_star {
        spot - strike
    } else {
        let d1_val = d1(s_star, strike, r, q, sigma, t);
        let a2 = (s_star / q2) * (1.0 - (-q * t).exp() * normal_cdf(d1_val));
        european + a2 * (spot / s_star).powf(q2)
    }
}

fn baw_put(
    spot: Real,
    strike: Real,
    r: Real,
    q: Real,
    sigma: Real,
    t: Real,
    european: Real,
    m: Real,
    n: Real,
    big_k: Real,
) -> Real {
    let q1 = (-(n - 1.0) - ((n - 1.0) * (n - 1.0) + 4.0 * m / big_k).sqrt()) / 2.0;

    let s_star = find_critical_put(strike, r, q, sigma, t, q1);

    if spot <= s_star {
        strike - spot
    } else {
        let d1_val = d1(s_star, strike, r, q, sigma, t);
        let a1 = -(s_star / q1) * (1.0 - (-q * t).exp() * normal_cdf(-d1_val));
        european + a1 * (spot / s_star).powf(q1)
    }
}

/// Find critical call exercise price S* via Newton's method.
/// Solves g(S) = (S - K) - C_BS(S) - (S/q₂)(1 - e^{-qT}N(d₁(S))) = 0.
fn find_critical_call(
    strike: Real,
    r: Real,
    q: Real,
    sigma: Real,
    t: Real,
    q2: Real,
) -> Real {
    let s_inf = strike / (1.0 - 2.0 / q2);
    let h2 = -((r - q) * t + 2.0 * sigma * t.sqrt()) * strike / (s_inf - strike);
    let mut si = s_inf + (strike - s_inf) * (-h2).exp();
    si = si.max(strike * 1.001);

    for _ in 0..200 {
        let (bs, ..) = black_scholes_merton(OptionType::Call, si, strike, r, q, sigma, t);
        let d1v = d1(si, strike, r, q, sigma, t);
        let eq = (-q * t).exp();
        let nd1 = normal_cdf(d1v);
        let npd1 = normal_pdf(d1v);
        let sst = sigma * t.sqrt();

        let a2 = (si / q2) * (1.0 - eq * nd1);
        let gv = (si - strike) - bs - a2;

        if gv.abs() < 1e-8 * strike {
            return si;
        }

        // g'(S) = 1 - e^{-qT}N(d₁) - (1/q₂)(1 - e^{-qT}N(d₁)) + e^{-qT}n(d₁)/(q₂σ√T)
        let delta = eq * nd1;
        let da2 = (1.0 / q2) * (1.0 - eq * nd1) - eq * npd1 / (q2 * sst);
        let gp = 1.0 - delta - da2;

        if gp.abs() < 1e-15 {
            break;
        }

        si -= gv / gp;
        si = si.max(strike * 1.001).min(strike * 100.0);
    }

    si
}

/// Find critical put exercise price S* via Newton's method.
/// Solves g(S) = (K - S) - P_BS(S) + (S/q₁)(1 - e^{-qT}N(-d₁(S))) = 0.
fn find_critical_put(
    strike: Real,
    r: Real,
    q: Real,
    sigma: Real,
    t: Real,
    q1: Real,
) -> Real {
    let s_zero = strike / (1.0 - 2.0 / q1);
    let h1 = ((r - q) * t - 2.0 * sigma * t.sqrt()) * strike / (strike - s_zero);
    let mut si = s_zero + (strike - s_zero) * (-h1).exp();
    si = si.max(1e-10).min(strike * 0.999);

    for _ in 0..200 {
        let (bs, ..) = black_scholes_merton(OptionType::Put, si, strike, r, q, sigma, t);
        let d1v = d1(si, strike, r, q, sigma, t);
        let eq = (-q * t).exp();
        let nmd1 = normal_cdf(-d1v);
        let npd1 = normal_pdf(d1v);
        let sst = sigma * t.sqrt();

        let a1 = -(si / q1) * (1.0 - eq * nmd1);
        let gv = (strike - si) - bs - a1;

        if gv.abs() < 1e-8 * strike {
            return si;
        }

        // g'(S) = -1 + e^{-qT}N(-d₁) + (1/q₁)(1 - e^{-qT}N(-d₁)) + e^{-qT}n(d₁)/(q₁σ√T)
        let delta_put = -eq * nmd1;
        let da1 = -(1.0 / q1) * (1.0 - eq * nmd1) - eq * npd1 / (q1 * sst);
        let gp = -1.0 - delta_put - da1;

        if gp.abs() < 1e-15 {
            break;
        }

        si -= gv / gp;
        si = si.max(1e-10).min(strike * 0.999);
    }

    si
}

impl PricingEngine<VanillaOptionArguments> for BaroneAdesiWhaleyEngine {
    fn calculate(&self, args: &VanillaOptionArguments) -> Result<PricingResults> {
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

        let price = barone_adesi_whaley(option_type, spot, strike, r, q, sigma, t);

        Ok(PricingResults::from_npv(price))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn american_call_geq_european() {
        let spot = 100.0;
        let strike = 100.0;
        let r = 0.05;
        let q = 0.02;
        let sigma = 0.25;
        let t = 1.0;

        let american = barone_adesi_whaley(OptionType::Call, spot, strike, r, q, sigma, t);
        let (european, ..) = black_scholes_merton(OptionType::Call, spot, strike, r, q, sigma, t);

        assert!(
            american >= european - 0.01,
            "american={american}, european={european}"
        );
    }

    #[test]
    fn american_put_geq_european() {
        let spot = 100.0;
        let strike = 100.0;
        let r = 0.05;
        let q = 0.0;
        let sigma = 0.25;
        let t = 1.0;

        let american = barone_adesi_whaley(OptionType::Put, spot, strike, r, q, sigma, t);
        let (european, ..) = black_scholes_merton(OptionType::Put, spot, strike, r, q, sigma, t);

        assert!(
            american >= european - 0.01,
            "american={american}, european={european}"
        );
    }

    #[test]
    fn deep_itm_put_near_intrinsic() {
        // Deep ITM American put should be close to intrinsic value with early exercise
        let spot = 50.0;
        let strike = 100.0;
        let r = 0.10;
        let q = 0.0;
        let sigma = 0.25;
        let t = 1.0;

        let price = barone_adesi_whaley(OptionType::Put, spot, strike, r, q, sigma, t);
        let intrinsic = strike - spot; // 50.0

        assert!(price >= intrinsic - 0.01, "price={price}, intrinsic={intrinsic}");
    }

    #[test]
    fn american_call_no_dividend_equals_european() {
        // Without dividends, American call = European call
        let spot = 100.0;
        let strike = 100.0;
        let r = 0.05;
        let q = 0.0; // no dividends
        let sigma = 0.20;
        let t = 1.0;

        let american = barone_adesi_whaley(OptionType::Call, spot, strike, r, q, sigma, t);
        let (european, ..) = black_scholes_merton(OptionType::Call, spot, strike, r, q, sigma, t);

        assert!(
            (american - european).abs() < 0.5,
            "american={american}, european={european}"
        );
    }

    #[test]
    fn american_put_positive() {
        let price = barone_adesi_whaley(OptionType::Put, 100.0, 100.0, 0.05, 0.0, 0.20, 1.0);
        assert!(price > 0.0, "price = {price}");
    }
}
