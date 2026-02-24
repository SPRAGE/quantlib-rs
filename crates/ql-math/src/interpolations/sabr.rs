//! SABR (Stochastic Alpha Beta Rho) interpolation
//! (translates `ql/math/interpolations/sabrinterpolation.hpp`).
//!
//! This module provides the Hagan et al. (2002) SABR formula for implied
//! volatility as a function of strike, along with a `SabrInterpolation` that
//! can be used as `Interpolation1D` once calibrated to a smile.

use ql_core::Real;

/// SABR model parameters.
#[derive(Debug, Clone, Copy)]
pub struct SabrParameters {
    /// Alpha (initial vol-of-vol backbone)
    pub alpha: Real,
    /// Beta (CEV exponent, usually fixed: 0 = normal, 1 = log-normal)
    pub beta: Real,
    /// Nu (vol-of-vol)
    pub nu: Real,
    /// Rho (correlation between asset and vol increments)
    pub rho: Real,
}

/// Compute the SABR implied (Black) volatility using the Hagan et al. (2002) formula.
///
/// # Arguments
/// * `f` — forward rate
/// * `k` — strike
/// * `t` — time to expiry (years)
/// * `p` — SABR parameters
///
/// Returns implied Black volatility σ_B(K).
pub fn sabr_volatility(f: Real, k: Real, t: Real, p: &SabrParameters) -> Real {
    let alpha = p.alpha;
    let beta = p.beta;
    let nu = p.nu;
    let rho = p.rho;

    // Handle ATM case (f ≈ K)
    let fk = f * k;
    if (f - k).abs() < 1e-12 * f.abs().max(1e-30) {
        return sabr_volatility_atm(f, t, p);
    }

    let one_minus_beta = 1.0 - beta;

    let fk_beta = fk.powf(one_minus_beta);
    let log_fk = (f / k).ln();
    let fk_half_beta = fk.powf(one_minus_beta / 2.0);

    // z = (nu / alpha) * (f*k)^((1-β)/2) * ln(f/k)
    let z = (nu / alpha) * fk_half_beta * log_fk;

    // x(z) = ln((√(1 - 2ρz + z²) + z - ρ) / (1 - ρ))
    let sqrt_arg = 1.0 - 2.0 * rho * z + z * z;
    let sqrt_val = sqrt_arg.max(0.0).sqrt();
    let xz = ((sqrt_val + z - rho) / (1.0 - rho)).ln();

    if xz.abs() < 1e-15 {
        return sabr_volatility_atm(f, t, p);
    }

    let a = one_minus_beta * one_minus_beta;

    // Leading term
    let numer = alpha;
    let denom = fk_half_beta * (1.0 + a / 24.0 * log_fk * log_fk + a * a / 1920.0 * log_fk.powi(4));

    // z/x(z) ratio
    let ratio = z / xz;

    // Correction factor
    let correction = 1.0
        + (a / 24.0 * alpha * alpha / fk_beta
            + 0.25 * rho * beta * nu * alpha / fk_half_beta
            + (2.0 - 3.0 * rho * rho) / 24.0 * nu * nu)
            * t;

    numer / denom * ratio * correction
}

/// SABR ATM volatility (f = K).
fn sabr_volatility_atm(f: Real, t: Real, p: &SabrParameters) -> Real {
    let alpha = p.alpha;
    let beta = p.beta;
    let nu = p.nu;
    let rho = p.rho;

    let one_minus_beta = 1.0 - beta;
    let f_beta = f.powf(one_minus_beta);

    let term1 = one_minus_beta * one_minus_beta / 24.0 * alpha * alpha / (f_beta * f_beta);
    let term2 = 0.25 * rho * beta * nu * alpha / f_beta;
    let term3 = (2.0 - 3.0 * rho * rho) / 24.0 * nu * nu;

    alpha / f_beta * (1.0 + (term1 + term2 + term3) * t)
}

/// A SABR smile interpolation.
///
/// Given calibrated SABR parameters (α, β, ν, ρ), produces implied Black
/// volatilities for arbitrary strikes via the Hagan formula.
///
/// Corresponds to `QuantLib::SABRInterpolation`.
#[derive(Debug, Clone)]
pub struct SabrSmile {
    /// Forward rate
    pub forward: Real,
    /// Time to expiry
    pub expiry: Real,
    /// Calibrated SABR parameters
    pub params: SabrParameters,
}

impl SabrSmile {
    /// Create a new SABR smile.
    pub fn new(forward: Real, expiry: Real, params: SabrParameters) -> Self {
        Self {
            forward,
            expiry,
            params,
        }
    }

    /// Compute implied vol at the given strike.
    pub fn volatility(&self, strike: Real) -> Real {
        sabr_volatility(self.forward, strike, self.expiry, &self.params)
    }
}

/// Calibrate SABR parameters (α, ν, ρ) to market implied vols, with β fixed.
///
/// Uses a simple least-squares minimizer (Nelder-Mead / Simplex-like iteration).
///
/// # Arguments
/// * `forward` — current forward rate
/// * `expiry` — time to expiry
/// * `strikes` — market strike levels
/// * `vols` — market implied Black vols
/// * `beta` — fixed CEV exponent
/// * `initial_alpha`, `initial_nu`, `initial_rho` — starting guess
///
/// Returns calibrated `SabrParameters`.
pub fn calibrate_sabr(
    forward: Real,
    expiry: Real,
    strikes: &[Real],
    vols: &[Real],
    beta: Real,
    initial_alpha: Real,
    initial_nu: Real,
    initial_rho: Real,
) -> SabrParameters {
    // Simple Nelder-Mead-style calibration
    let n = strikes.len();
    assert_eq!(n, vols.len(), "strikes and vols must have equal length");
    assert!(n >= 3, "need at least 3 points to calibrate 3 parameters");

    // Objective function: sum of squared errors
    let objective = |params: &[Real; 3]| -> Real {
        let p = SabrParameters {
            alpha: params[0].max(1e-10),
            beta,
            nu: params[1].max(1e-10),
            rho: params[2].max(-0.999).min(0.999),
        };
        let mut sse = 0.0;
        for i in 0..n {
            let model_vol = sabr_volatility(forward, strikes[i], expiry, &p);
            let diff = model_vol - vols[i];
            sse += diff * diff;
        }
        sse
    };

    // Nelder-Mead simplex in 3D
    let mut simplex: Vec<([Real; 3], Real)> = Vec::with_capacity(4);
    let x0 = [initial_alpha, initial_nu, initial_rho];
    simplex.push((x0, objective(&x0)));

    let perturbations = [
        [initial_alpha * 0.1, 0.0, 0.0],
        [0.0, initial_nu * 0.1, 0.0],
        [0.0, 0.0, 0.05],
    ];
    for pert in &perturbations {
        let xi = [x0[0] + pert[0], x0[1] + pert[1], x0[2] + pert[2]];
        simplex.push((xi, objective(&xi)));
    }

    let max_iter = 5000;
    let tol = 1e-12;

    for _iter in 0..max_iter {
        // Sort by function value
        simplex.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        // Check convergence
        let range = simplex[3].1 - simplex[0].1;
        if range < tol {
            break;
        }

        // Centroid (excluding worst)
        let mut centroid = [0.0; 3];
        for j in 0..3 {
            for k in 0..3 {
                centroid[k] += simplex[j].0[k];
            }
        }
        for k in 0..3 {
            centroid[k] /= 3.0;
        }

        let worst = simplex[3].0;

        // Reflection
        let mut reflected = [0.0; 3];
        for k in 0..3 {
            reflected[k] = 2.0 * centroid[k] - worst[k];
        }
        let f_reflected = objective(&reflected);

        if f_reflected < simplex[0].1 {
            // Expansion
            let mut expanded = [0.0; 3];
            for k in 0..3 {
                expanded[k] = 3.0 * centroid[k] - 2.0 * worst[k];
            }
            let f_expanded = objective(&expanded);
            if f_expanded < f_reflected {
                simplex[3] = (expanded, f_expanded);
            } else {
                simplex[3] = (reflected, f_reflected);
            }
        } else if f_reflected < simplex[2].1 {
            simplex[3] = (reflected, f_reflected);
        } else {
            // Contraction
            let mut contracted = [0.0; 3];
            if f_reflected < simplex[3].1 {
                // Outside contraction
                for k in 0..3 {
                    contracted[k] = 0.5 * (centroid[k] + reflected[k]);
                }
            } else {
                // Inside contraction
                for k in 0..3 {
                    contracted[k] = 0.5 * (centroid[k] + worst[k]);
                }
            }
            let f_contracted = objective(&contracted);
            if f_contracted < simplex[3].1 {
                simplex[3] = (contracted, f_contracted);
            } else {
                // Shrink
                let best = simplex[0].0;
                for j in 1..4 {
                    for k in 0..3 {
                        simplex[j].0[k] = 0.5 * (best[k] + simplex[j].0[k]);
                    }
                    simplex[j].1 = objective(&simplex[j].0);
                }
            }
        }
    }

    simplex.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    let best = simplex[0].0;

    SabrParameters {
        alpha: best[0].max(1e-10),
        beta,
        nu: best[1].max(1e-10),
        rho: best[2].max(-0.999).min(0.999),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sabr_atm_consistency() {
        let f = 0.04;
        let t = 1.0;
        let p = SabrParameters {
            alpha: 0.04,
            beta: 0.5,
            nu: 0.4,
            rho: -0.3,
        };
        // ATM vol from the full formula should match the dedicated ATM formula
        let v1 = sabr_volatility(f, f * (1.0 + 1e-10), t, &p);
        let v2 = sabr_volatility_atm(f, t, &p);
        assert!(
            (v1 - v2).abs() < 1e-6,
            "ATM vol mismatch: {v1} vs {v2}"
        );
    }

    #[test]
    fn sabr_smile_shape() {
        // SABR with negative rho should produce a downward-skewed smile
        let f = 0.04;
        let t = 1.0;
        let p = SabrParameters {
            alpha: 0.04,
            beta: 0.5,
            nu: 0.4,
            rho: -0.5,
        };
        let v_low = sabr_volatility(f, 0.02, t, &p);
        let v_atm = sabr_volatility(f, f, t, &p);
        let v_high = sabr_volatility(f, 0.08, t, &p);

        // With positive nu and negative rho, low strikes should have higher vol
        assert!(v_low > v_atm, "expected v_low > v_atm");
        assert!(v_atm > 0.0, "ATM vol should be positive");
        assert!(v_high > 0.0, "high-strike vol should be positive");
    }

    #[test]
    fn sabr_calibration_roundtrip() {
        let f = 0.04;
        let t = 1.0;
        let true_params = SabrParameters {
            alpha: 0.04,
            beta: 0.5,
            nu: 0.4,
            rho: -0.3,
        };

        // Generate "market" vols
        let strikes: Vec<Real> = (1..=9).map(|i| 0.01 * (i as Real)).collect();
        let vols: Vec<Real> = strikes
            .iter()
            .map(|&k| sabr_volatility(f, k, t, &true_params))
            .collect();

        // Calibrate from a perturbed starting point
        let cal = calibrate_sabr(f, t, &strikes, &vols, 0.5, 0.05, 0.5, -0.2);

        assert!(
            (cal.alpha - true_params.alpha).abs() < 0.005,
            "alpha: expected ~{}, got {}",
            true_params.alpha,
            cal.alpha
        );
        assert!(
            (cal.nu - true_params.nu).abs() < 0.05,
            "nu: expected ~{}, got {}",
            true_params.nu,
            cal.nu
        );
        assert!(
            (cal.rho - true_params.rho).abs() < 0.05,
            "rho: expected ~{}, got {}",
            true_params.rho,
            cal.rho
        );
    }
}
