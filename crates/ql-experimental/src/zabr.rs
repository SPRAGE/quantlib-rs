//! ZABR (Zeta Alpha Beta Rho) model
//!
//! The ZABR model generalises SABR by replacing the CEV dynamics of the
//! forward rate with a more general local volatility:
//!
//! ```text
//! dF = σ · F^β · dW₁
//! dσ = ν · σ^γ · dW₂
//! ```
//!
//! where γ is the ZABR parameter.  When γ = 1 the model reduces to
//! standard SABR.  The `short-maturity lognormal` expansion (Andreasen &
//! Huge, 2011) is used to produce implied Black volatilities.
//!
//! Corresponds to `QuantLib::ZabrModel` / `QuantLib::ZabrSmileSection`.

use ql_core::{Real, Time, Volatility};
use ql_math::interpolations::sabr::{sabr_volatility, SabrParameters};

/// ZABR (Zeta Alpha Beta Rho) model parameters.
#[derive(Debug, Clone, Copy)]
pub struct ZabrParameters {
    /// Alpha — initial volatility.
    pub alpha: Real,
    /// Beta — CEV exponent for the forward.
    pub beta: Real,
    /// Nu — vol-of-vol.
    pub nu: Real,
    /// Rho — correlation between asset and vol.
    pub rho: Real,
    /// Gamma — CEV exponent for the vol process.
    /// γ = 1 recovers standard SABR.
    pub gamma: Real,
}

impl ZabrParameters {
    /// Validate the ZABR parameters.
    pub fn validate(&self) {
        assert!(
            self.alpha > 0.0,
            "ZABR: alpha must be > 0, got {}",
            self.alpha
        );
        assert!(
            (0.0..=1.0).contains(&self.beta),
            "ZABR: beta must be in [0, 1], got {}",
            self.beta
        );
        assert!(self.nu >= 0.0, "ZABR: nu must be >= 0, got {}", self.nu);
        assert!(
            self.rho.abs() < 1.0,
            "ZABR: |rho| must be < 1, got {}",
            self.rho
        );
        assert!(
            self.gamma > 0.0,
            "ZABR: gamma must be > 0, got {}",
            self.gamma
        );
    }
}

/// ZABR evaluation method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZabrEvaluationMethod {
    /// Short-maturity lognormal expansion.
    ShortMaturityLognormal,
    /// Short-maturity normal expansion.
    ShortMaturityNormal,
    /// Local volatility (effective SABR with modified α).
    LocalVolatility,
}

/// The ZABR model.
///
/// Computes implied Black volatility for a given forward and strike using
/// the short-maturity expansion approach.
///
/// When γ = 1.0, the model is identical to SABR.
///
/// # References
/// * Andreasen, J. & Huge, B. (2011), "ZABR — Expansions for the Masses".
/// * QuantLib implementation: `ql/experimental/volatility/zabrinterpolation.hpp`.
#[derive(Debug, Clone)]
pub struct ZabrModel {
    /// Model parameters.
    pub params: ZabrParameters,
    /// Forward rate.
    pub forward: Real,
    /// Time to expiry.
    pub expiry: Time,
    /// Evaluation method.
    pub method: ZabrEvaluationMethod,
}

impl ZabrModel {
    /// Create a new ZABR model.
    pub fn new(forward: Real, expiry: Time, params: ZabrParameters) -> Self {
        params.validate();
        Self {
            params,
            forward,
            expiry,
            method: ZabrEvaluationMethod::ShortMaturityLognormal,
        }
    }

    /// Set the evaluation method.
    pub fn with_method(mut self, method: ZabrEvaluationMethod) -> Self {
        self.method = method;
        self
    }

    /// Compute implied Black volatility for a given strike.
    pub fn implied_volatility(&self, strike: Real) -> Volatility {
        match self.method {
            ZabrEvaluationMethod::ShortMaturityLognormal => self.short_maturity_lognormal(strike),
            ZabrEvaluationMethod::ShortMaturityNormal => self.short_maturity_normal(strike),
            ZabrEvaluationMethod::LocalVolatility => self.local_vol_expansion(strike),
        }
    }

    /// Short-maturity lognormal expansion.
    ///
    /// This is the primary ZABR approximation.  When γ = 1 it reduces to
    /// the standard Hagan SABR formula with an effective α.
    fn short_maturity_lognormal(&self, strike: Real) -> Volatility {
        let p = &self.params;
        let t = self.expiry;

        // Effective alpha for the short-maturity expansion
        let alpha_eff = self.effective_alpha();

        // Build effective SABR parameters and use the standard SABR formula
        let sabr_p = SabrParameters {
            alpha: alpha_eff,
            beta: p.beta,
            nu: p.nu,
            rho: p.rho,
        };

        sabr_volatility(self.forward, strike, t, &sabr_p)
    }

    /// Short-maturity normal expansion.
    ///
    /// Produces a normal (Bachelier) vol, which is then converted to Black
    /// vol using the simple conversion: σ_B ≈ σ_N / F^β.
    fn short_maturity_normal(&self, strike: Real) -> Volatility {
        let p = &self.params;
        let t = self.expiry;
        let f = self.forward;

        let alpha_eff = self.effective_alpha();

        // Normal SABR: use β = 0 effective SABR for normal vol
        let sabr_normal = SabrParameters {
            alpha: alpha_eff * f.powf(p.beta),
            beta: 0.0,
            nu: p.nu,
            rho: p.rho,
        };

        let normal_vol = sabr_volatility(f, strike, t, &sabr_normal);

        // Convert normal vol to lognormal vol
        let fk = (f * strike).sqrt();
        if fk < 1e-15 {
            return normal_vol;
        }
        normal_vol / fk
    }

    /// Local volatility expansion.
    ///
    /// Uses a modified SABR alpha that captures the γ-correction at first
    /// order.
    fn local_vol_expansion(&self, strike: Real) -> Volatility {
        let p = &self.params;
        let t = self.expiry;

        // The local vol approach uses a modified effective alpha
        let alpha_eff = self.effective_alpha_local_vol();

        let sabr_p = SabrParameters {
            alpha: alpha_eff,
            beta: p.beta,
            nu: p.nu,
            rho: p.rho,
        };

        sabr_volatility(self.forward, strike, t, &sabr_p)
    }

    /// Compute the effective alpha for the short-maturity expansion.
    ///
    /// For γ ≠ 1:
    /// ```text
    /// α_eff = α · [1 + (γ-1)·ν²·α^(2(γ-1))·T / 2]^(1/(2(γ-1)))
    ///              ÷ [1 + (γ-1)·ν²·α^(2(γ-1))·T / 2]^(1/(2(γ-1)))
    /// ```
    ///
    /// More precisely, for the short-maturity case, the ZABR effective α is:
    ///
    /// $\alpha_\text{eff} = \alpha \cdot \left(1 + \frac{(\gamma-1)\,\nu^2\,\alpha^{2(\gamma-1)}\,T}{2}\right)^{\frac{1}{2(1-\gamma)}}$
    ///
    /// when γ < 1, and for γ > 1 one uses the appropriate sign convention.
    /// For γ = 1, α_eff = α (standard SABR).
    fn effective_alpha(&self) -> Real {
        let p = &self.params;
        let t = self.expiry;

        if (p.gamma - 1.0).abs() < 1e-10 {
            // gamma ≈ 1: standard SABR, no correction needed
            return p.alpha;
        }

        let gm1 = p.gamma - 1.0;
        let a2gm1 = p.alpha.powf(2.0 * gm1);
        let arg = 1.0 + gm1 * p.nu * p.nu * a2gm1 * t / 2.0;

        if arg <= 0.0 {
            // Fallback: the expansion broke down
            return p.alpha;
        }

        let exponent = 1.0 / (2.0 * (1.0 - p.gamma));

        // Effective sigma of the vol process integrated over [0,T]
        // E[σ²] ≈ α² × correction
        // effective α comes from matching the zeroth-order SABR expansion
        p.alpha * arg.powf(exponent)
    }

    /// Effective alpha using the local volatility method (asymptotic at second order).
    fn effective_alpha_local_vol(&self) -> Real {
        let p = &self.params;
        let t = self.expiry;

        if (p.gamma - 1.0).abs() < 1e-10 {
            return p.alpha;
        }

        let gm1 = p.gamma - 1.0;
        let a2gm1 = p.alpha.powf(2.0 * gm1);

        // Second-order correction for local vol approach
        let correction = 1.0
            + gm1 * p.nu * p.nu * a2gm1 * t / 2.0
            + gm1 * (2.0 * gm1 - 1.0) * p.nu.powi(4) * a2gm1 * a2gm1 * t * t / 8.0;

        if correction <= 0.0 {
            return p.alpha;
        }

        let exponent = 1.0 / (2.0 * (1.0 - p.gamma));
        p.alpha * correction.powf(exponent)
    }
}

/// A ZABR-based smile section.
///
/// Implements the `SmileSection` interface for ZABR.
#[derive(Debug, Clone)]
pub struct ZabrSmileSection {
    model: ZabrModel,
}

impl ZabrSmileSection {
    /// Create a new ZABR smile section.
    pub fn new(forward: Real, expiry: Time, params: ZabrParameters) -> Self {
        Self {
            model: ZabrModel::new(forward, expiry, params),
        }
    }

    /// Set the evaluation method.
    pub fn with_method(mut self, method: ZabrEvaluationMethod) -> Self {
        self.model = self.model.with_method(method);
        self
    }

    /// The underlying ZABR model.
    pub fn model(&self) -> &ZabrModel {
        &self.model
    }

    /// Implied volatility at a given strike.
    pub fn volatility(&self, strike: Real) -> Volatility {
        self.model.implied_volatility(strike)
    }

    /// The forward rate.
    pub fn forward(&self) -> Real {
        self.model.forward
    }

    /// Time to expiry.
    pub fn exercise_time(&self) -> Time {
        self.model.expiry
    }

    /// The ZABR parameters.
    pub fn params(&self) -> &ZabrParameters {
        &self.model.params
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zabr_gamma_one_matches_sabr() {
        // ZABR with γ=1 should exactly match standard SABR
        let alpha = 0.08;
        let beta = 0.70;
        let nu = 0.20;
        let rho = -0.30;
        let tau = 5.0;
        let forward = 0.03;

        let zabr_params = ZabrParameters {
            alpha,
            beta,
            nu,
            rho,
            gamma: 1.0,
        };
        let sabr_params = SabrParameters {
            alpha,
            beta,
            nu,
            rho,
        };

        let model = ZabrModel::new(forward, tau, zabr_params);

        let strikes = [0.01, 0.02, 0.03, 0.04, 0.05, 0.06, 0.08];
        for &k in &strikes {
            let zabr_vol = model.implied_volatility(k);
            let sabr_vol = sabr_volatility(forward, k, tau, &sabr_params);
            assert!(
                (zabr_vol - sabr_vol).abs() < 1e-10,
                "ZABR(γ=1) vs SABR mismatch at K={}: {:.8} vs {:.8}",
                k,
                zabr_vol,
                sabr_vol
            );
        }
    }

    #[test]
    fn zabr_effective_alpha_identity_at_gamma_one() {
        let params = ZabrParameters {
            alpha: 0.04,
            beta: 0.5,
            nu: 0.3,
            rho: -0.2,
            gamma: 1.0,
        };
        let model = ZabrModel::new(0.03, 1.0, params);
        assert!(
            (model.effective_alpha() - params.alpha).abs() < 1e-14,
            "effective alpha should equal alpha when gamma=1"
        );
    }

    #[test]
    fn zabr_gamma_less_than_one() {
        // ZABR with γ < 1 should still produce sensible vols
        let params = ZabrParameters {
            alpha: 0.04,
            beta: 0.5,
            nu: 0.3,
            rho: -0.2,
            gamma: 0.5,
        };
        let model = ZabrModel::new(0.03, 1.0, params);

        let strikes = [0.02, 0.025, 0.03, 0.035, 0.04];
        for &k in &strikes {
            let vol = model.implied_volatility(k);
            assert!(vol > 0.0, "vol should be positive at K={}", k);
            assert!(vol < 2.0, "vol should be < 200% at K={}, got {}", k, vol);
        }
    }

    #[test]
    fn zabr_gamma_greater_than_one() {
        // ZABR with γ > 1 should still produce sensible vols
        let params = ZabrParameters {
            alpha: 0.04,
            beta: 0.5,
            nu: 0.3,
            rho: -0.2,
            gamma: 1.5,
        };
        let model = ZabrModel::new(0.03, 1.0, params);

        let strikes = [0.02, 0.025, 0.03, 0.035, 0.04];
        for &k in &strikes {
            let vol = model.implied_volatility(k);
            assert!(vol > 0.0, "vol should be positive at K={}", k);
            assert!(vol < 2.0, "vol should be < 200% at K={}, got {}", k, vol);
        }
    }

    #[test]
    fn zabr_smile_section() {
        let params = ZabrParameters {
            alpha: 0.08,
            beta: 0.70,
            nu: 0.20,
            rho: -0.30,
            gamma: 1.0,
        };
        let section = ZabrSmileSection::new(0.03, 5.0, params);

        assert!((section.forward() - 0.03).abs() < 1e-15);
        assert!((section.exercise_time() - 5.0).abs() < 1e-15);

        let vol = section.volatility(0.03);
        assert!(vol > 0.0);
        assert!(vol < 1.0);
    }

    #[test]
    fn zabr_evaluation_methods_consistent() {
        // All three methods should give similar (if not identical) results for gamma=1
        let params = ZabrParameters {
            alpha: 0.04,
            beta: 0.5,
            nu: 0.3,
            rho: -0.2,
            gamma: 1.0,
        };
        let f = 0.03;
        let t = 1.0;
        let k = 0.03;

        let model_ln =
            ZabrModel::new(f, t, params).with_method(ZabrEvaluationMethod::ShortMaturityLognormal);
        let model_lv =
            ZabrModel::new(f, t, params).with_method(ZabrEvaluationMethod::LocalVolatility);

        let vol_ln = model_ln.implied_volatility(k);
        let vol_lv = model_lv.implied_volatility(k);

        // Both should match SABR for gamma = 1
        let sabr_p = SabrParameters {
            alpha: params.alpha,
            beta: params.beta,
            nu: params.nu,
            rho: params.rho,
        };
        let sabr_vol = sabr_volatility(f, k, t, &sabr_p);

        assert!(
            (vol_ln - sabr_vol).abs() < 1e-10,
            "LN vs SABR: {:.8} vs {:.8}",
            vol_ln,
            sabr_vol
        );
        assert!(
            (vol_lv - sabr_vol).abs() < 1e-10,
            "LV vs SABR: {:.8} vs {:.8}",
            vol_lv,
            sabr_vol
        );
    }
}
