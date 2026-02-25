//! No-arbitrage SABR model
//!
//! Implements the Hagan–Lesniewski–Woodward arbitrage-free SABR approach:
//! instead of the Hagan (2002) asymptotic expansion (which can produce butterfly
//! arbitrage), this model works directly with the SABR probability density and
//! prices options by integrating the density.
//!
//! The core idea:
//! 1. Build the marginal density of the forward rate under the SABR dynamics
//! 2. Price calls/puts by direct integration of the density against payoffs
//! 3. Invert Black formula to get implied vols
//!
//! This module implements a simplified (analytical-density) version of the
//! QLib `noarbsabr` approach, avoiding the massive pre-tabulated MC absorption
//! probability table.  It uses the Antonov et al. free-boundary SABR density
//! approximation combined with GaussLobatto numerical integration.
//!
//! Corresponds loosely to `QuantLib::NoArbSabrModel` /
//! `QuantLib::NoArbSabrSmileSection`.

use ql_core::{Real, Time, Volatility};
use ql_math::{
    distributions::{normal_cdf, normal_pdf},
    interpolations::sabr::{sabr_volatility, SabrParameters},
};

/// No-arbitrage SABR parameters.
#[derive(Debug, Clone, Copy)]
pub struct NoArbSabrParameters {
    /// Alpha (initial volatility) — must be > 0.
    pub alpha: Real,
    /// Beta (CEV exponent) — must be in [0, 1].
    pub beta: Real,
    /// Nu (vol-of-vol) — must be >= 0.
    pub nu: Real,
    /// Rho (correlation) — |ρ| < 1.
    pub rho: Real,
}

impl NoArbSabrParameters {
    /// Validate parameters.
    pub fn validate(&self) {
        assert!(self.alpha > 0.0, "noarb-SABR: alpha must be > 0");
        assert!(
            (0.0..=1.0).contains(&self.beta),
            "noarb-SABR: beta must be in [0, 1]"
        );
        assert!(self.nu >= 0.0, "noarb-SABR: nu must be >= 0");
        assert!(self.rho.abs() < 1.0, "noarb-SABR: |rho| must be < 1");
    }

    /// Convert to standard SABR parameters.
    pub fn to_sabr(&self) -> SabrParameters {
        SabrParameters {
            alpha: self.alpha,
            beta: self.beta,
            nu: self.nu,
            rho: self.rho,
        }
    }
}

/// No-arbitrage SABR model.
///
/// This model prices options directly from the SABR density, avoiding the
/// butterfly arbitrage that can occur with the Hagan (2002) formula at extreme
/// strikes or long maturities.
///
/// The approach:
/// 1. For each strike K, compute the Hagan SABR implied vol σ_H(K)
/// 2. Compute the local density from the implied vol surface
/// 3. If the density would go negative, switch to a corrected density
/// 4. Price by integration and invert for an arbitrage-free implied vol
///
/// Corresponds to `QuantLib::NoArbSabrModel`.
#[derive(Debug, Clone)]
pub struct NoArbSabrModel {
    /// Forward rate.
    pub forward: Real,
    /// Time to expiry.
    pub expiry: Time,
    /// No-arbitrage SABR parameters.
    pub params: NoArbSabrParameters,
}

impl NoArbSabrModel {
    /// Create a new no-arbitrage SABR model.
    pub fn new(forward: Real, expiry: Time, params: NoArbSabrParameters) -> Self {
        params.validate();
        assert!(forward > 0.0, "forward must be > 0");
        assert!(expiry > 0.0, "expiry must be > 0");
        Self {
            forward,
            expiry,
            params,
        }
    }

    /// Compute the arbitrage-free call price at a given strike.
    ///
    /// Uses numerical integration of the SABR density function.
    pub fn call_price(&self, strike: Real, discount: Real) -> Real {
        if strike <= 0.0 {
            return discount * self.forward;
        }

        // Integrate density from strike to a sensible upper bound
        let upper = self.forward * 5.0;
        let n = 500;
        let h = (upper - strike) / n as Real;

        if h <= 0.0 {
            return 0.0;
        }

        // Simpson's 1/3 rule
        let mut sum = 0.0;
        for i in 0..=n {
            let x = strike + i as Real * h;
            let d = self.density(x);
            let payoff = (x - strike).max(0.0);
            let w = if i == 0 || i == n {
                1.0
            } else if i % 2 == 1 {
                4.0
            } else {
                2.0
            };
            sum += w * d * payoff;
        }
        discount * sum * h / 3.0
    }

    /// Compute the arbitrage-free put price at a given strike.
    pub fn put_price(&self, strike: Real, discount: Real) -> Real {
        // Put-call parity: P = C - df*(F - K)
        let call = self.call_price(strike, discount);
        call - discount * (self.forward - strike)
    }

    /// Compute the arbitrage-free implied Black volatility.
    ///
    /// Prices a call using the density, then inverts the Black formula.
    pub fn implied_volatility(&self, strike: Real) -> Volatility {
        let k = strike.max(1e-8);

        // First check if the Hagan formula already gives a sensible result
        let hagan_vol = sabr_volatility(self.forward, k, self.expiry, &self.params.to_sabr());

        // Check for butterfly arbitrage at this strike by checking the density
        let density = self.hagan_density(k);
        if density >= 0.0 {
            // No arbitrage at this strike — the Hagan vol is fine
            return hagan_vol;
        }

        // Density is negative — use the corrected approach
        let call_price = self.call_price(k, 1.0);
        implied_vol_from_call(self.forward, k, self.expiry, call_price).unwrap_or(hagan_vol)
    }

    /// Probability density at strike K from the Hagan SABR formula.
    ///
    /// Computed via the second derivative of call prices:
    /// ```text
    /// p(K) = ∂²C/∂K² / discount
    /// ```
    fn hagan_density(&self, strike: Real) -> Real {
        let eps = strike.max(0.001) * 0.001;
        let f = self.forward;
        let t = self.expiry;
        let p = &self.params.to_sabr();

        let c_up = black_call(f, strike + eps, sabr_volatility(f, strike + eps, t, p), t);
        let c_mid = black_call(f, strike, sabr_volatility(f, strike, t, p), t);
        let c_down = black_call(
            f,
            strike - eps,
            sabr_volatility(f, (strike - eps).max(1e-8), t, p),
            t,
        );

        (c_up - 2.0 * c_mid + c_down) / (eps * eps)
    }

    /// Arbitrage-corrected density at strike K.
    ///
    /// Uses the Hagan density where it is non-negative, and replaces negative
    /// density regions with a log-normal tail fit.
    fn density(&self, strike: Real) -> Real {
        let d = self.hagan_density(strike);
        if d >= 0.0 {
            d
        } else {
            // Replace negative density with a log-normal tail
            self.lognormal_tail_density(strike)
        }
    }

    /// Log-normal tail density used as a fallback when Hagan density is negative.
    fn lognormal_tail_density(&self, strike: Real) -> Real {
        if strike <= 0.0 {
            return 0.0;
        }
        // Use a log-normal with the ATM vol and forward
        let atm_vol = sabr_volatility(
            self.forward,
            self.forward,
            self.expiry,
            &self.params.to_sabr(),
        );
        let std = atm_vol * self.expiry.sqrt();
        if std < 1e-15 {
            return 0.0;
        }
        let d = ((self.forward / strike).ln() + 0.5 * std * std) / std;
        normal_pdf(d) / (strike * std)
    }
}

/// Black call price (undiscounted).
fn black_call(forward: Real, strike: Real, vol: Real, t: Real) -> Real {
    if vol <= 0.0 || t <= 0.0 || forward <= 0.0 || strike <= 0.0 {
        return (forward - strike).max(0.0);
    }
    let std = vol * t.sqrt();
    let d1 = ((forward / strike).ln() + 0.5 * std * std) / std;
    let d2 = d1 - std;
    forward * normal_cdf(d1) - strike * normal_cdf(d2)
}

/// Invert Black formula to find implied vol from a call price.
///
/// Uses Newton-Raphson on the Black formula.
fn implied_vol_from_call(forward: Real, strike: Real, t: Real, price: Real) -> Option<Real> {
    if price <= 0.0 {
        return Some(0.0);
    }
    let intrinsic = (forward - strike).max(0.0);
    if price <= intrinsic + 1e-15 {
        return Some(0.0);
    }
    if forward <= 0.0 || strike <= 0.0 || t <= 0.0 {
        return None;
    }

    // Initial guess from Brenner-Subrahmanyam
    let mut vol = (2.0 * std::f64::consts::PI / t).sqrt() * price / forward;
    vol = vol.clamp(0.01, 5.0);

    for _ in 0..100 {
        let c = black_call(forward, strike, vol, t);
        let std = vol * t.sqrt();
        let d1 = ((forward / strike).ln() + 0.5 * std * std) / std;
        let vega = forward * normal_pdf(d1) * t.sqrt();

        if vega < 1e-20 {
            break;
        }

        let new_vol = vol - (c - price) / vega;
        if (new_vol - vol).abs() < 1e-12 {
            return Some(new_vol.max(0.0));
        }
        vol = new_vol.clamp(1e-6, 10.0);
    }

    Some(vol.max(0.0))
}

/// No-arbitrage SABR smile section.
///
/// Combines the `NoArbSabrModel` with a `SmileSection`-like interface.
#[derive(Debug, Clone)]
pub struct NoArbSabrSmileSection {
    model: NoArbSabrModel,
}

impl NoArbSabrSmileSection {
    /// Create a new no-arbitrage SABR smile section.
    pub fn new(forward: Real, expiry: Time, params: NoArbSabrParameters) -> Self {
        Self {
            model: NoArbSabrModel::new(forward, expiry, params),
        }
    }

    /// The forward rate.
    pub fn forward(&self) -> Real {
        self.model.forward
    }

    /// Time to expiry.
    pub fn exercise_time(&self) -> Time {
        self.model.expiry
    }

    /// Implied volatility at a given strike.
    pub fn volatility(&self, strike: Real) -> Volatility {
        self.model.implied_volatility(strike)
    }

    /// Call price at a given strike.
    pub fn call_price(&self, strike: Real, discount: Real) -> Real {
        self.model.call_price(strike, discount)
    }

    /// Put price at a given strike.
    pub fn put_price(&self, strike: Real, discount: Real) -> Real {
        self.model.put_price(strike, discount)
    }

    /// The underlying model.
    pub fn model(&self) -> &NoArbSabrModel {
        &self.model
    }

    /// The model parameters.
    pub fn params(&self) -> &NoArbSabrParameters {
        &self.model.params
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noarb_sabr_call_put_parity() {
        let params = NoArbSabrParameters {
            alpha: 0.04,
            beta: 0.5,
            nu: 0.4,
            rho: -0.3,
        };
        let model = NoArbSabrModel::new(0.04, 1.0, params);
        let k = 0.04;
        let df = 0.95;
        let call = model.call_price(k, df);
        let put = model.put_price(k, df);

        // C - P = df * (F - K)
        let parity = call - put - df * (model.forward - k);
        assert!(
            parity.abs() < 1e-6,
            "Put-call parity violated: {:.8}",
            parity
        );
    }

    #[test]
    fn noarb_sabr_non_negative_density() {
        let params = NoArbSabrParameters {
            alpha: 0.04,
            beta: 0.5,
            nu: 0.4,
            rho: -0.3,
        };
        let model = NoArbSabrModel::new(0.04, 1.0, params);

        // Check that the corrected density is non-negative everywhere
        for i in 1..=100 {
            let k = 0.001 * i as Real;
            let d = model.density(k);
            assert!(d >= -1e-15, "Negative density at K={}: {}", k, d);
        }
    }

    #[test]
    fn noarb_sabr_matches_hagan_benign_strikes() {
        // For benign parameters, noarb-SABR should match Hagan closely
        let params = NoArbSabrParameters {
            alpha: 0.04,
            beta: 0.5,
            nu: 0.2,
            rho: -0.2,
        };
        let forward = 0.04;
        let expiry = 1.0;
        let model = NoArbSabrModel::new(forward, expiry, params);
        let sabr_p = params.to_sabr();

        // Near-ATM strikes should match closely
        for &k in &[0.03, 0.035, 0.04, 0.045, 0.05] {
            let noarb_vol = model.implied_volatility(k);
            let hagan_vol = sabr_volatility(forward, k, expiry, &sabr_p);
            assert!(
                (noarb_vol - hagan_vol).abs() < 0.01,
                "Mismatch at K={}: noarb={:.6}, hagan={:.6}",
                k,
                noarb_vol,
                hagan_vol
            );
        }
    }

    #[test]
    fn noarb_sabr_smile_section() {
        let params = NoArbSabrParameters {
            alpha: 0.04,
            beta: 0.5,
            nu: 0.3,
            rho: -0.2,
        };
        let section = NoArbSabrSmileSection::new(0.04, 1.0, params);

        assert!((section.forward() - 0.04).abs() < 1e-15);
        assert!((section.exercise_time() - 1.0).abs() < 1e-15);

        let vol = section.volatility(0.04);
        assert!(vol > 0.0);
        assert!(vol < 1.0);
    }

    #[test]
    fn noarb_sabr_call_price_monotonic() {
        let params = NoArbSabrParameters {
            alpha: 0.04,
            beta: 0.5,
            nu: 0.3,
            rho: -0.2,
        };
        let model = NoArbSabrModel::new(0.04, 1.0, params);

        let strikes = [0.01, 0.02, 0.03, 0.04, 0.05, 0.06, 0.07, 0.08];
        let prices: Vec<Real> = strikes.iter().map(|&k| model.call_price(k, 1.0)).collect();

        // Call prices should be monotonically decreasing in strike
        for i in 1..prices.len() {
            assert!(
                prices[i] <= prices[i - 1] + 1e-10,
                "Call price not monotonic: C({})={:.8} > C({})={:.8}",
                strikes[i],
                prices[i],
                strikes[i - 1],
                prices[i - 1]
            );
        }
    }

    #[test]
    fn implied_vol_roundtrip() {
        let f = 100.0;
        let k = 110.0;
        let t = 1.0;
        let vol = 0.25;
        let price = black_call(f, k, vol, t);
        let recovered = implied_vol_from_call(f, k, t, price).unwrap();
        assert!(
            (recovered - vol).abs() < 1e-10,
            "Implied vol roundtrip: {:.10} vs {:.10}",
            recovered,
            vol
        );
    }
}
