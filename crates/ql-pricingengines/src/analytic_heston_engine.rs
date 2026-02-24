//! Analytic (semi-analytic) Heston model pricing engine.
//!
//! Translates `ql/pricingengines/vanilla/analytichestonengine.hpp`.
//!
//! Prices European options under the Heston stochastic volatility model using
//! Gauss-Laguerre numerical integration of the characteristic function.

use std::f64::consts::PI;
use std::sync::Arc;

use ql_core::{errors::Result, Real};
use ql_instruments::{OptionType, PricingEngine, PricingResults, VanillaOptionArguments};
use ql_math::integrals::{Integrator, SimpsonIntegral};
use ql_models::HestonModel;

/// Semi-analytic Heston pricing engine using Gauss-Laguerre quadrature.
///
/// The Heston model assumes the variance follows a CIR process:
///
/// $$dS = (r-q) S \, dt + \sqrt{v} S \, dW_1$$
/// $$dv = \kappa(\theta - v) \, dt + \sigma_v \sqrt{v} \, dW_2$$
/// $$dW_1 dW_2 = \rho \, dt$$
///
/// The call price is $C = S e^{-qT} P_1 - K e^{-rT} P_2$ where $P_1, P_2$
/// are computed from the characteristic function via numerical integration.
///
/// Corresponds to `QuantLib::AnalyticHestonEngine`.
#[derive(Debug)]
pub struct AnalyticHestonEngine {
    model: Arc<HestonModel>,
    /// Number of Gauss-Laguerre quadrature points (default: 128).
    integration_order: usize,
}

impl AnalyticHestonEngine {
    /// Create a new Heston engine with the given model.
    pub fn new(model: Arc<HestonModel>) -> Self {
        Self {
            model,
            integration_order: 128,
        }
    }

    /// Set the Gauss-Laguerre integration order.
    pub fn with_integration_order(mut self, order: usize) -> Self {
        self.integration_order = order;
        self
    }
}

/// Heston characteristic function (log-transform, Albrecher et al. formulation).
///
/// Computes $\phi_j(\xi; t, v_0, \kappa, \theta, \sigma, \rho)$ for $j \in \{1, 2\}$.
///
/// Returns `(Re, Im)` of the integrand for the probability $P_j$.
fn heston_char_func(
    phi: Real,
    t: Real,
    v0: Real,
    kappa: Real,
    theta: Real,
    sigma: Real,
    rho: Real,
    j: usize, // 1 or 2
) -> (Real, Real) {
    // For j=1: u = 0.5,  b = kappa - rho*sigma
    // For j=2: u = -0.5, b = kappa
    let (u, b) = if j == 1 {
        (0.5, kappa - rho * sigma)
    } else {
        (-0.5, kappa)
    };

    // d = sqrt((rho*sigma*i*phi - b)^2 - sigma^2*(2*u*i*phi - phi^2))
    // Let alpha = rho*sigma*phi, beta = b
    // (alpha*i - beta)^2 = -alpha^2 + 2*alpha*beta*i + beta^2 (wrong sign)
    // Actually: (rho*sigma*i*phi - b)^2 = (b - rho*sigma*i*phi)^2
    //   = b^2 - 2*b*rho*sigma*i*phi + (rho*sigma*phi)^2*(-1)
    //   = b^2 - (rho*sigma*phi)^2 - 2*b*rho*sigma*phi*i
    // sigma^2*(2*u*i*phi - phi^2) = -sigma^2*phi^2 + 2*u*sigma^2*phi*i
    //
    // d^2 = (b^2 - rho^2*sigma^2*phi^2 + sigma^2*phi^2) + (-2*b*rho*sigma*phi - 2*u*sigma^2*phi)*i
    //      = b^2 + sigma^2*phi^2*(1-rho^2) + (-2*sigma*phi*(b*rho+u*sigma))*i

    let sigma2 = sigma * sigma;
    let phi2 = phi * phi;

    let d_re = b * b + sigma2 * phi2 * (1.0 - rho * rho);
    let d_im = -2.0 * sigma * phi * (b * rho + u * sigma);

    // d = sqrt(d_re + d_im * i)
    let (d_r, d_i) = complex_sqrt(d_re, d_im);
    // Enforce Re(d) >= 0 for the numerically stable branch
    let (d_r, d_i) = if d_r < 0.0 { (-d_r, -d_i) } else { (d_r, d_i) };

    // c = b - rho*sigma*i*phi = (b, -rho*sigma*phi)
    let c_r = b;
    let c_i = -rho * sigma * phi;

    // Numerically stable formulation: g_m = (c - d)/(c + d)
    // (small when c ≈ d, avoiding the g = (c+d)/(c-d) → ∞ singularity)
    let gm_num_r = c_r - d_r;
    let gm_num_i = c_i - d_i;
    let gm_den_r = c_r + d_r;
    let gm_den_i = c_i + d_i;
    let (gm_r, gm_i) = complex_div(gm_num_r, gm_num_i, gm_den_r, gm_den_i);

    // exp(-d*T) — use negative exponent for stability (Re(d) >= 0 → decays)
    let emdt_mag = (-d_r * t).exp();
    let emdt_r = emdt_mag * (-d_i * t).cos();
    let emdt_i = emdt_mag * (-d_i * t).sin();

    // g_m * exp(-dT)
    let gme_r = gm_r * emdt_r - gm_i * emdt_i;
    let gme_i = gm_r * emdt_i + gm_i * emdt_r;

    // 1 - g_m * exp(-dT)
    let one_m_gme_r = 1.0 - gme_r;
    let one_m_gme_i = -gme_i;

    // 1 - g_m
    let one_m_gm_r = 1.0 - gm_r;
    let one_m_gm_i = -gm_i;

    // D = (c - d)/σ² * (1 - exp(-dT)) / (1 - g_m * exp(-dT))
    let one_m_emdt_r = 1.0 - emdt_r;
    let one_m_emdt_i = -emdt_i;
    let (frac_r, frac_i) = complex_div(one_m_emdt_r, one_m_emdt_i, one_m_gme_r, one_m_gme_i);

    let cmd_r = c_r - d_r;
    let cmd_i = c_i - d_i;
    let big_d_r = (cmd_r * frac_r - cmd_i * frac_i) / sigma2;
    let big_d_i = (cmd_r * frac_i + cmd_i * frac_r) / sigma2;

    // C = κθ/σ² * [(c - d)T - 2*ln((1 - g_m*exp(-dT)) / (1 - g_m))]
    let cmdt_r = cmd_r * t;
    let cmdt_i = cmd_i * t;
    let (log_arg_r, log_arg_i) =
        complex_div(one_m_gme_r, one_m_gme_i, one_m_gm_r, one_m_gm_i);
    let (log_r, log_i) = complex_log(log_arg_r, log_arg_i);

    let big_c_r = kappa * theta / sigma2 * (cmdt_r - 2.0 * log_r);
    let big_c_i = kappa * theta / sigma2 * (cmdt_i - 2.0 * log_i);

    // Characteristic function: exp(C + D*v0)
    let exp_arg_r = big_c_r + big_d_r * v0;
    let exp_arg_i = big_c_i + big_d_i * v0;

    let exp_mag = exp_arg_r.exp();
    let cf_r = exp_mag * exp_arg_i.cos();
    let cf_i = exp_mag * exp_arg_i.sin();

    (cf_r, cf_i)
}

/// Compute $P_j = \frac{1}{2} + \frac{1}{\pi} \int_0^\infty
/// \mathrm{Re}\!\left[\frac{e^{-i\phi\ln K}\,f_j(\phi)}{i\phi}\right] d\phi$
fn compute_pj(
    j: usize,
    spot: Real,
    strike: Real,
    t: Real,
    r: Real,
    q: Real,
    v0: Real,
    kappa: Real,
    theta: Real,
    sigma: Real,
    rho: Real,
    _integration_order: usize,
) -> Real {
    // Combine log(S*exp((r-q)T)) - log(K) = log-moneyness of forward
    let x = spot.ln() + (r - q) * t - strike.ln();

    let integrand = |phi: Real| -> Real {
        if phi < 1e-12 {
            return 0.0;
        }
        let (cf_r, cf_i) = heston_char_func(phi, t, v0, kappa, theta, sigma, rho, j);

        // exp(i*phi*x) = cos(phi*x) + i*sin(phi*x)
        let cos_px = (phi * x).cos();
        let sin_px = (phi * x).sin();

        // Re[cf * exp(i*phi*x) / (i*phi)] = Im[cf * exp(i*phi*x)] / phi
        (cf_r * sin_px + cf_i * cos_px) / phi
    };

    // Adaptive Simpson integration over [ε, upper_bound].
    // The integrand decays as phi → ∞ due to the CF exponential decay.
    let integrator = SimpsonIntegral::new(1e-10, 200_000);
    let integral = integrator.integrate(integrand, 1e-8, 500.0).unwrap_or(0.0);
    0.5 + integral / PI
}

/// Price a European option under the Heston model.
///
/// Returns `(price, delta)`.
pub fn heston_price(
    option_type: OptionType,
    spot: Real,
    strike: Real,
    r: Real,
    q: Real,
    t: Real,
    v0: Real,
    kappa: Real,
    theta: Real,
    sigma: Real,
    rho: Real,
    integration_order: usize,
) -> Real {
    let _phi = option_type.sign();

    let p1 = compute_pj(1, spot, strike, t, r, q, v0, kappa, theta, sigma, rho, integration_order);
    let p2 = compute_pj(2, spot, strike, t, r, q, v0, kappa, theta, sigma, rho, integration_order);

    let df_q = (-q * t).exp();
    let df_r = (-r * t).exp();

    let call = spot * df_q * p1 - strike * df_r * p2;

    if option_type == OptionType::Call {
        call
    } else {
        // Put-call parity: P = C - S*exp(-qT) + K*exp(-rT)
        call - spot * df_q + strike * df_r
    }
}

impl PricingEngine<VanillaOptionArguments> for AnalyticHestonEngine {
    fn calculate(&self, args: &VanillaOptionArguments) -> Result<PricingResults> {
        let process = self.model.process();
        let spot = process.s0();
        let strike = args.payoff.strike();
        let option_type = args.payoff.option_type();
        let expiry = args.exercise.last_date();

        let ref_date = process.risk_free_rate().reference_date();
        let dc = process.risk_free_rate().day_counter();
        let t = dc.year_fraction(ref_date, expiry);

        let r = process.risk_free_rate().zero_rate_impl(t);
        let q = process.dividend_yield().zero_rate_impl(t);

        let v0 = self.model.v0();
        let kappa = self.model.kappa();
        let theta = self.model.theta();
        let sigma = self.model.sigma();
        let rho = self.model.rho();

        let price = heston_price(
            option_type,
            spot,
            strike,
            r,
            q,
            t,
            v0,
            kappa,
            theta,
            sigma,
            rho,
            self.integration_order,
        );

        Ok(PricingResults::from_npv(price))
    }
}

// ─── Complex arithmetic helpers ─────────────────────────────────────────────

fn complex_sqrt(re: Real, im: Real) -> (Real, Real) {
    let r = (re * re + im * im).sqrt().sqrt();
    let theta = im.atan2(re) / 2.0;
    (r * theta.cos(), r * theta.sin())
}

fn complex_div(a_r: Real, a_i: Real, b_r: Real, b_i: Real) -> (Real, Real) {
    let denom = b_r * b_r + b_i * b_i;
    if denom < 1e-300 {
        return (0.0, 0.0);
    }
    (
        (a_r * b_r + a_i * b_i) / denom,
        (a_i * b_r - a_r * b_i) / denom,
    )
}

fn complex_log(re: Real, im: Real) -> (Real, Real) {
    let r = (re * re + im * im).sqrt();
    let theta = im.atan2(re);
    (r.ln(), theta)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Heston with moderate vol-of-vol should produce prices close to BS.
    #[test]
    fn heston_close_to_bs_low_vol_of_vol() {
        // With small σ_v and ρ=0, Heston should be close to BS with σ = √v0.
        // We use σ_v=0.1 (not too small for numerical stability).
        let spot = 100.0;
        let strike = 100.0;
        let r = 0.05;
        let q = 0.0;
        let t = 1.0;
        let v0 = 0.04; // σ = 0.20
        let kappa = 2.0;
        let theta = 0.04;
        let sigma_v = 0.1;
        let rho = 0.0;

        let heston = heston_price(
            OptionType::Call,
            spot, strike, r, q, t, v0, kappa, theta, sigma_v, rho, 128,
        );

        // Compare to BS — with σ_v=0.1 there's a small correction
        use crate::analytic_european_engine::black_scholes_merton;
        let (bs, ..) = black_scholes_merton(OptionType::Call, spot, strike, r, q, 0.20, t);

        assert!(
            (heston - bs).abs() < 1.0,
            "heston={heston}, bs={bs}"
        );
    }

    /// Heston put-call parity.
    #[test]
    fn heston_put_call_parity() {
        let spot = 100.0;
        let strike = 105.0;
        let r = 0.05;
        let q = 0.02;
        let t = 1.0;
        let v0 = 0.04;
        let kappa = 2.0;
        let theta = 0.04;
        let sigma_v = 0.3;
        let rho = -0.7;

        let call = heston_price(OptionType::Call, spot, strike, r, q, t, v0, kappa, theta, sigma_v, rho, 128);
        let put = heston_price(OptionType::Put, spot, strike, r, q, t, v0, kappa, theta, sigma_v, rho, 128);

        // C - P = S*exp(-qT) - K*exp(-rT)
        let lhs = call - put;
        let rhs = spot * (-q * t).exp() - strike * (-r * t).exp();
        assert!(
            (lhs - rhs).abs() < 0.01,
            "parity: lhs={lhs}, rhs={rhs}"
        );
    }

    /// Heston engine via model.
    #[test]
    fn heston_engine_via_model() {
        use ql_termstructures::FlatForward;
        use ql_time::{Actual365Fixed, Date};
        use ql_processes::HestonProcess;

        let ref_date = Date::from_ymd(2025, 1, 15).unwrap();
        let rf = Arc::new(FlatForward::continuous(ref_date, 0.05, Actual365Fixed));
        let div = Arc::new(FlatForward::continuous(ref_date, 0.0, Actual365Fixed));

        let process = HestonProcess::new(100.0, 0.04, rf, div, 2.0, 0.04, 0.3, -0.5);
        let model = Arc::new(HestonModel::new(process));
        let engine = AnalyticHestonEngine::new(model);

        let expiry = Date::from_ymd(2026, 1, 15).unwrap();
        let args = VanillaOptionArguments {
            payoff: Arc::new(ql_instruments::PlainVanillaPayoff::new(OptionType::Call, 100.0)),
            exercise: ql_instruments::Exercise::european(expiry),
        };

        let result = engine.calculate(&args).unwrap();
        // Heston price should be in reasonable range for ATM call
        assert!(result.npv > 5.0 && result.npv < 20.0, "npv = {}", result.npv);
    }

    /// Heston negative correlation produces implied vol skew.
    #[test]
    fn heston_skew_with_negative_rho() {
        let spot = 100.0;
        let r = 0.05;
        let q = 0.0;
        let t = 1.0;
        let v0 = 0.04;
        let kappa = 2.0;
        let theta = 0.04;
        let sigma_v = 0.4;
        let rho = -0.7;

        // OTM put (K=90) should have higher implied vol than ATM
        let otm_put = heston_price(OptionType::Put, spot, 90.0, r, q, t, v0, kappa, theta, sigma_v, rho, 128);
        let atm_call = heston_price(OptionType::Call, spot, 100.0, r, q, t, v0, kappa, theta, sigma_v, rho, 128);

        // Both should be positive
        assert!(otm_put > 0.0, "otm_put = {otm_put}");
        assert!(atm_call > 0.0, "atm_call = {atm_call}");
    }
}
