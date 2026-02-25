//! Integral Heston variance option engine.
//!
//! Translates `ql/experimental/varianceoption/integralhestonvarianceoptionengine.cpp`.
//!
//! Prices variance options (options on realized variance) under the Heston model
//! using the approach from Recchioni & Sun:
//! <http://www.econ.univpm.it/recchioni/finance/w4/>
//!
//! Two internal routines:
//! - `ivop_one_dim`: fast path for plain vanilla call on variance
//! - `ivop_two_dim`: general payoff (e.g. put)

use num_complex::Complex64;
use ql_core::{errors::Result, Real, Time};
use ql_instruments::OptionType;
use std::f64::consts::PI;

// ── Public engine ─────────────────────────────────────────────────────────────

/// Variance option arguments (simplified, analogous to C++ VarianceOption::arguments).
#[derive(Debug, Clone)]
pub struct VarianceOptionArguments {
    /// The payoff type and strike.
    pub option_type: OptionType,
    /// The variance strike.
    pub strike: Real,
    /// Notional amount.
    pub notional: Real,
    /// Time to maturity (year fraction).
    pub tau: Time,
}

/// Variance option pricing results.
#[derive(Debug, Clone)]
pub struct VarianceOptionResults {
    /// Net present value.
    pub npv: Real,
}

/// Integral Heston variance-option engine.
///
/// Corresponds to `QuantLib::IntegralHestonVarianceOptionEngine`.
#[derive(Debug, Clone)]
pub struct IntegralHestonVarianceOptionEngine {
    /// Initial variance (v₀).
    pub v0: Real,
    /// Mean-reversion speed (κ).
    pub kappa: Real,
    /// Long-run variance (θ).
    pub theta: Real,
    /// Vol of vol (σ).
    pub sigma: Real,
    /// Spot-vol correlation (ρ).
    pub rho: Real,
    /// Risk-free rate (continuous).
    pub r: Real,
}

impl IntegralHestonVarianceOptionEngine {
    /// Create engine from Heston model parameters.
    pub fn new(v0: Real, kappa: Real, theta: Real, sigma: Real, rho: Real, r: Real) -> Self {
        Self {
            v0,
            kappa,
            theta,
            sigma,
            rho,
            r,
        }
    }

    /// Price a variance option.
    pub fn calculate(&self, args: &VarianceOptionArguments) -> Result<VarianceOptionResults> {
        let epsilon = self.sigma;
        let chi = self.kappa;
        let theta = self.theta;
        let rho = self.rho;
        let v0 = self.v0;
        let tau = args.tau;
        let r = self.r;

        let npv = match args.option_type {
            OptionType::Call => {
                // Fast 1D path for plain vanilla call on variance
                ivop_one_dim(epsilon, chi, theta, rho, v0, args.strike, tau, r) * args.notional
            }
            OptionType::Put => {
                // General 2D path for non-call payoffs (including puts)
                let strike = args.strike;
                let payoff = move |v: Real| -> Real { (strike - v).max(0.0) };
                ivop_two_dim(epsilon, chi, theta, rho, v0, tau, r, &payoff) * args.notional
            }
        };

        Ok(VarianceOptionResults { npv })
    }
}

// ── IvopOneDim ────────────────────────────────────────────────────────────────

/// Price a variance call option under Heston using the 1D integral method.
///
/// Bailey-Swarztrauber approach for the characteristic function transform.
/// Parameters:
/// - eps (σ): vol of vol
/// - chi (κ): mean reversion speed
/// - theta (θ): long-run variance
/// - rho: correlation (unused in this method)
/// - v0: initial variance
/// - eprice: variance strike
/// - tau: time to maturity
/// - rtax: risk-free rate
fn ivop_one_dim(
    eps: Real,
    chi: Real,
    theta: Real,
    _rho: Real,
    v0: Real,
    eprice: Real,
    tau: Time,
    rtax: Real,
) -> Real {
    let ui = Complex64::new(0.0, 1.0);
    let i0: Real = 0.0; // initial integrated variance

    let pi2 = 2.0 * PI;
    let s = 2.0 * chi * theta / (eps * eps) - 1.0;

    assert!(s > 0.0, "Feller condition parameter s must be > 0, got {s}");

    let ss = s + 1.0;

    // Grid parameters
    let dstep: Real = 256.0;
    let nris = (pi2).sqrt() / dstep;
    let mm = (pi2 / (nris * nris)) as usize;

    // Build grid points
    let mut xiv = vec![0.0_f64; mm + 1];
    for j in 0..mm {
        xiv[j + 1] = (j as Real - mm as Real / 2.0) * nris;
    }

    // Compute characteristic function values
    let mut ff = vec![Complex64::new(0.0, 0.0); mm + 1];

    for j in 0..mm {
        let xi = xiv[j + 1];

        let caux_r = chi * chi;
        let caux1 = 2.0 * eps * eps * xi * ui;
        let caux2 = caux1 + Complex64::new(caux_r, 0.0);

        let zita = caux2.sqrt() * 0.5;
        let caux1_exp = (-2.0 * tau * zita).exp();

        let beta = Complex64::new(0.5 * chi, 0.0)
            + zita
            + caux1_exp * (zita - Complex64::new(0.5 * chi, 0.0));
        let gamma = Complex64::new(1.0, 0.0) - caux1_exp;

        let caux_log = Complex64::new(ss * tau, 0.0) * (zita - Complex64::new(0.5 * chi, 0.0));
        let caux_ss = Complex64::new(ss, 0.0) * (Complex64::new(2.0, 0.0) * (zita / beta)).ln();
        let caux3 = -Complex64::new(v0, 0.0) * ui * Complex64::new(xi, 0.0) * (gamma / beta);
        let caux_total = caux_ss + caux3 - caux_log;

        ff[j + 1] = caux_total.exp();

        // Payoff transform: for a call on realized variance
        let xi_c = Complex64::new(xi, 0.0);
        let contrib = if (xi_c.norm()) > 1e-6 {
            let t1 = -Complex64::new(eprice, 0.0) / (ui * xi_c);
            let eterm = (ui * xi_c * Complex64::new(eprice, 0.0)).exp() - Complex64::new(1.0, 0.0);
            t1 + eterm / (ui * xi_c * ui * xi_c)
        } else {
            Complex64::new(eprice * eprice * 0.5, 0.0)
        };

        ff[j + 1] *= contrib;
    }

    // Inverse FFT-like summation (Bailey-Swarztrauber)
    let mut csum = Complex64::new(0.0, 0.0);
    for j in 0..mm {
        let sign = (-1.0_f64).powi(j as i32);
        let exp_arg = ui
            * Complex64::new(
                -2.0 * PI * (mm as Real) * (j as Real) * 0.5 / (mm as Real),
                0.0,
            );
        csum += ff[j + 1] * Complex64::new(sign, 0.0) * exp_arg.exp();
    }

    let sign_mm = (-1.0_f64).powi(mm as i32);
    csum *= Complex64::new(sign_mm.sqrt() * nris / pi2, 0.0);

    // Add deterministic part
    let vero = i0 - eprice + theta * tau + (1.0 - (-chi * tau).exp()) * (v0 - theta) / chi;
    csum += Complex64::new(vero, 0.0);

    // Option value
    let option = (-rtax * tau).exp() * csum.re;

    let impart = csum.im.abs();
    assert!(
        impart <= 1e-3,
        "imaginary part of option must be near zero, got {impart}"
    );

    option
}

// ── IvopTwoDim ────────────────────────────────────────────────────────────────

/// Price a variance option under Heston using the 2D integral method.
///
/// General payoff version of the Bailey-Swarztrauber approach.
/// Uses `dstep=64` giving `mm=4096` grid points, so the double loop is
/// `4096² ≈ 16.7M` iterations — tractable in well under a second.
///
/// Parameters:
/// - eps (σ): vol of vol
/// - chi (κ): mean reversion speed
/// - theta (θ): long-run variance
/// - _rho: correlation (unused in this method)
/// - v0: initial variance
/// - tau: time to maturity
/// - rtax: risk-free rate
/// - payoff: payoff function f(V) where V = integrated variance
fn ivop_two_dim(
    eps: Real,
    chi: Real,
    theta: Real,
    _rho: Real,
    v0: Real,
    tau: Time,
    rtax: Real,
    payoff: &dyn Fn(Real) -> Real,
) -> Real {
    let ui = Complex64::new(0.0, 1.0);
    let i0: Real = 0.0;

    let pi2 = 2.0 * PI;
    let s = 2.0 * chi * theta / (eps * eps) - 1.0;

    assert!(s > 0.0, "Feller condition parameter s must be > 0, got {s}");

    let ss = s + 1.0;

    // Grid parameters — dstep=64 gives mm=4096
    let dstep: Real = 64.0;
    let nris = (pi2).sqrt() / dstep;
    let mm = (pi2 / (nris * nris)) as usize;

    // Build grid points (two grids: frequency and payoff evaluation)
    let mut xiv = vec![0.0_f64; mm + 1];
    let mut ivet = vec![0.0_f64; mm + 1];
    for j in 0..mm {
        xiv[j + 1] = (j as Real - mm as Real / 2.0) * nris;
        ivet[j + 1] = (j as Real - mm as Real / 2.0) * pi2 / (mm as Real * nris);
    }

    // Compute characteristic function values (NO payoff transform — that's in outer loop)
    let mut ff = vec![Complex64::new(0.0, 0.0); mm + 1];

    for j in 0..mm {
        let xi = xiv[j + 1];

        let caux_r = chi * chi;
        let caux1 = 2.0 * eps * eps * xi * ui;
        let caux2 = caux1 + Complex64::new(caux_r, 0.0);

        let zita = caux2.sqrt() * 0.5;
        let caux1_exp = (-2.0 * tau * zita).exp();

        let beta = Complex64::new(0.5 * chi, 0.0)
            + zita
            + caux1_exp * (zita - Complex64::new(0.5 * chi, 0.0));
        let gamma = Complex64::new(1.0, 0.0) - caux1_exp;

        let caux_log = Complex64::new(ss * tau, 0.0) * (zita - Complex64::new(0.5 * chi, 0.0));
        let caux_ss = Complex64::new(ss, 0.0) * (Complex64::new(2.0, 0.0) * (zita / beta)).ln();
        let caux3 = -Complex64::new(v0, 0.0) * ui * Complex64::new(xi, 0.0) * (gamma / beta);
        let caux_total = caux_ss + caux3 - caux_log;

        ff[j + 1] = caux_total.exp();
    }

    // Double sum: outer over payoff evaluation points, inner over CF
    let mut sumr: Real = 0.0;

    for k in 0..mm {
        let ip = i0 - ivet[k + 1];
        let payoffval = payoff(ip);

        let dxi = Complex64::new(0.0, 2.0 * PI * (k as Real) / (mm as Real));

        let mut csum = Complex64::new(0.0, 0.0);
        for j in 0..mm {
            let z = -Complex64::new(j as Real, 0.0) * dxi;
            let sign = (-1.0_f64).powi(j as i32);
            csum += ff[j + 1] * Complex64::new(sign, 0.0) * z.exp();
        }

        let sign_k = (-1.0_f64).powi(k as i32);
        csum *= Complex64::new(sign_k * nris / pi2, 0.0);

        sumr += payoffval * csum.re;
    }

    sumr *= nris;

    (-rtax * tau).exp() * sumr
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integral_heston_call() {
        // C++ test: testIntegralHeston — Call case
        // v0=2.0, kappa=2.0, theta=0.01, sigma=0.1, rho=-0.5, r=0.0
        // strike=0.05, nominal=1.0, T=1.5
        let engine = IntegralHestonVarianceOptionEngine::new(
            2.0,  // v0
            2.0,  // kappa
            0.01, // theta
            0.1,  // sigma
            -0.5, // rho
            0.0,  // r
        );

        let args = VarianceOptionArguments {
            option_type: OptionType::Call,
            strike: 0.05,
            notional: 1.0,
            tau: 1.5,
        };

        let result = engine.calculate(&args).unwrap();
        let expected = 0.9104619;
        let error = (result.npv - expected).abs();
        assert!(
            error < 1e-4,
            "Call: expected {expected}, got {}, error {error}",
            result.npv
        );
    }

    #[test]
    fn test_integral_heston_put() {
        // C++ test: testIntegralHeston — Put case
        // v0=1.5, kappa=2.0, theta=0.01, sigma=0.1, rho=-0.5, r=0.0
        // strike=0.7, nominal=1.0, T=1.0
        let engine = IntegralHestonVarianceOptionEngine::new(
            1.5,  // v0
            2.0,  // kappa
            0.01, // theta
            0.1,  // sigma
            -0.5, // rho
            0.0,  // r
        );

        let args = VarianceOptionArguments {
            option_type: OptionType::Put,
            strike: 0.7,
            notional: 1.0,
            tau: 1.0,
        };

        let result = engine.calculate(&args).unwrap();
        let expected = 0.0466796;
        let error = (result.npv - expected).abs();
        assert!(
            error < 1e-4,
            "Put: expected {expected}, got {}, error {error}",
            result.npv
        );
    }
}
