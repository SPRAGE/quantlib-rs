//! Copula functions (translates `ql/math/copulas/`).
//!
//! Copulas describe the dependence structure between random variables,
//! independent of their marginal distributions.

use crate::distributions;
use ql_core::Real;
use std::f64::consts::PI;

/// A copula function `C(u, v)` mapping `[0,1]² → [0,1]`.
pub trait Copula {
    /// Evaluate the copula at `(u, v)`.
    fn value(&self, u: Real, v: Real) -> Real;
}

/// Minimum copula: `C(u, v) = max(u + v − 1, 0)` (Fréchet–Hoeffding lower bound).
///
/// Represents perfect negative dependence.
#[derive(Debug, Clone, Copy, Default)]
pub struct MinCopula;

impl Copula for MinCopula {
    fn value(&self, u: Real, v: Real) -> Real {
        (u + v - 1.0).max(0.0)
    }
}

/// Maximum copula: `C(u, v) = min(u, v)` (Fréchet–Hoeffding upper bound).
///
/// Represents perfect positive dependence (comonotonicity).
#[derive(Debug, Clone, Copy, Default)]
pub struct MaxCopula;

impl Copula for MaxCopula {
    fn value(&self, u: Real, v: Real) -> Real {
        u.min(v)
    }
}

/// Independence copula: `C(u, v) = u · v`.
#[derive(Debug, Clone, Copy, Default)]
pub struct IndependenceCopula;

impl Copula for IndependenceCopula {
    fn value(&self, u: Real, v: Real) -> Real {
        u * v
    }
}

/// Gaussian (normal) copula.
///
/// `C(u, v; ρ) = Φ₂(Φ⁻¹(u), Φ⁻¹(v); ρ)`
///
/// where `Φ₂` is the bivariate standard normal CDF and `ρ` is the correlation.
///
/// Corresponds to `QuantLib::GaussianCopula`.
#[derive(Debug, Clone, Copy)]
pub struct GaussianCopula {
    rho: Real,
}

impl GaussianCopula {
    /// Create a Gaussian copula with the given correlation `ρ ∈ [-1, 1]`.
    pub fn new(rho: Real) -> Self {
        assert!(
            (-1.0..=1.0).contains(&rho),
            "correlation must be in [-1, 1], got {rho}"
        );
        Self { rho }
    }

    /// The correlation parameter.
    pub fn rho(&self) -> Real {
        self.rho
    }
}

impl Copula for GaussianCopula {
    fn value(&self, u: Real, v: Real) -> Real {
        let x = distributions::normal_cdf_inverse(u);
        let y = distributions::normal_cdf_inverse(v);
        distributions::bivariate_normal_cdf(x, y, self.rho)
    }
}

/// Clayton copula.
///
/// `C(u, v; θ) = (u^{-θ} + v^{-θ} − 1)^{-1/θ}` for `θ > 0`.
///
/// Models lower-tail dependence (extreme co-movements in downturns).
///
/// Corresponds to `QuantLib::ClaytonCopula`.
#[derive(Debug, Clone, Copy)]
pub struct ClaytonCopula {
    theta: Real,
}

impl ClaytonCopula {
    /// Create a Clayton copula with parameter `θ > 0`.
    pub fn new(theta: Real) -> Self {
        assert!(theta > 0.0, "Clayton theta must be positive, got {theta}");
        Self { theta }
    }
}

impl Copula for ClaytonCopula {
    fn value(&self, u: Real, v: Real) -> Real {
        let t = self.theta;
        let val = u.powf(-t) + v.powf(-t) - 1.0;
        if val <= 0.0 {
            0.0
        } else {
            val.powf(-1.0 / t)
        }
    }
}

/// Frank copula.
///
/// `C(u, v; θ) = −(1/θ) · ln(1 + (e^{−θu} − 1)(e^{−θv} − 1) / (e^{−θ} − 1))`
///
/// Symmetric dependence (no tail dependence).
///
/// Corresponds to `QuantLib::FrankCopula`.
#[derive(Debug, Clone, Copy)]
pub struct FrankCopula {
    theta: Real,
}

impl FrankCopula {
    /// Create a Frank copula with parameter `θ ≠ 0`.
    pub fn new(theta: Real) -> Self {
        assert!(theta.abs() > 1e-15, "Frank theta must be non-zero");
        Self { theta }
    }
}

impl Copula for FrankCopula {
    fn value(&self, u: Real, v: Real) -> Real {
        let t = self.theta;
        let num = ((-t * u).exp() - 1.0) * ((-t * v).exp() - 1.0);
        let den = (-t).exp() - 1.0;
        -(1.0 + num / den).ln() / t
    }
}

/// Gumbel copula.
///
/// `C(u, v; θ) = exp(−[(−ln u)^θ + (−ln v)^θ]^{1/θ})` for `θ ≥ 1`.
///
/// Models upper-tail dependence.
///
/// Corresponds to `QuantLib::GumbelCopula`.
#[derive(Debug, Clone, Copy)]
pub struct GumbelCopula {
    theta: Real,
}

impl GumbelCopula {
    /// Create a Gumbel copula with parameter `θ ≥ 1`.
    pub fn new(theta: Real) -> Self {
        assert!(theta >= 1.0, "Gumbel theta must be ≥ 1, got {theta}");
        Self { theta }
    }
}

impl Copula for GumbelCopula {
    fn value(&self, u: Real, v: Real) -> Real {
        let t = self.theta;
        let lu = (-u.ln()).powf(t);
        let lv = (-v.ln()).powf(t);
        (-(lu + lv).powf(1.0 / t)).exp()
    }
}

/// Marshall-Olkin copula.
///
/// `C(u, v; α₁, α₂) = min(u · v^{1−α₂}, u^{1−α₁} · v)`
///
/// where `α₁, α₂ ∈ [0, 1]`.
#[derive(Debug, Clone, Copy)]
pub struct MarshallOlkinCopula {
    alpha1: Real,
    alpha2: Real,
}

impl MarshallOlkinCopula {
    /// Create a Marshall-Olkin copula.
    pub fn new(alpha1: Real, alpha2: Real) -> Self {
        assert!(
            (0.0..=1.0).contains(&alpha1) && (0.0..=1.0).contains(&alpha2),
            "parameters must be in [0, 1]"
        );
        Self { alpha1, alpha2 }
    }
}

impl Copula for MarshallOlkinCopula {
    fn value(&self, u: Real, v: Real) -> Real {
        (u * v.powf(1.0 - self.alpha2)).min(u.powf(1.0 - self.alpha1) * v)
    }
}

/// Ali-Mikhail-Haq copula.
///
/// `C(u, v; θ) = u·v / (1 − θ·(1−u)·(1−v))` for `θ ∈ [-1, 1]`.
#[derive(Debug, Clone, Copy)]
pub struct AliMikhailHaqCopula {
    theta: Real,
}

impl AliMikhailHaqCopula {
    /// Create an Ali-Mikhail-Haq copula with `θ ∈ [-1, 1]`.
    pub fn new(theta: Real) -> Self {
        assert!(
            (-1.0..=1.0).contains(&theta),
            "theta must be in [-1, 1], got {theta}"
        );
        Self { theta }
    }
}

impl Copula for AliMikhailHaqCopula {
    fn value(&self, u: Real, v: Real) -> Real {
        u * v / (1.0 - self.theta * (1.0 - u) * (1.0 - v))
    }
}

/// Farlie-Gumbel-Morgenstern copula.
///
/// `C(u, v; θ) = u·v + θ·u·v·(1−u)·(1−v)` for `θ ∈ [-1, 1]`.
#[derive(Debug, Clone, Copy)]
pub struct FarlieCopula {
    theta: Real,
}

impl FarlieCopula {
    /// Create a Farlie-Gumbel-Morgenstern copula with `θ ∈ [-1, 1]`.
    pub fn new(theta: Real) -> Self {
        assert!(
            (-1.0..=1.0).contains(&theta),
            "theta must be in [-1, 1], got {theta}"
        );
        Self { theta }
    }
}

impl Copula for FarlieCopula {
    fn value(&self, u: Real, v: Real) -> Real {
        u * v + self.theta * u * v * (1.0 - u) * (1.0 - v)
    }
}

// Suppress the unused import warning when PI isn't used directly
const _: Real = PI;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn min_copula() {
        let c = MinCopula;
        assert!((c.value(0.3, 0.8) - 0.1).abs() < 1e-12);
        assert!((c.value(0.2, 0.5) - 0.0).abs() < 1e-12);
    }

    #[test]
    fn max_copula() {
        let c = MaxCopula;
        assert!((c.value(0.3, 0.8) - 0.3).abs() < 1e-12);
        assert!((c.value(0.5, 0.2) - 0.2).abs() < 1e-12);
    }

    #[test]
    fn independence_copula() {
        let c = IndependenceCopula;
        assert!((c.value(0.3, 0.8) - 0.24).abs() < 1e-12);
    }

    #[test]
    fn gaussian_copula_independence() {
        // With ρ=0, the Gaussian copula should equal the independence copula
        let c = GaussianCopula::new(0.0);
        let val = c.value(0.5, 0.5);
        assert!(
            (val - 0.25).abs() < 0.02,
            "Gaussian(ρ=0) at (0.5,0.5) = {val}, expected ~0.25"
        );
    }

    #[test]
    fn gaussian_copula_perfect_positive() {
        let c = GaussianCopula::new(0.999);
        let val = c.value(0.3, 0.3);
        // With near-perfect correlation, C(u,u) ≈ u
        assert!(
            (val - 0.3).abs() < 0.05,
            "Gaussian(ρ≈1) at (0.3,0.3) = {val}"
        );
    }

    #[test]
    fn clayton_copula() {
        let c = ClaytonCopula::new(2.0);
        let val = c.value(0.5, 0.5);
        // C(0.5, 0.5; 2) = (0.5^{-2} + 0.5^{-2} - 1)^{-1/2} = (4+4-1)^{-1/2} = 7^{-0.5}
        let expected = 7.0_f64.powf(-0.5);
        assert!(
            (val - expected).abs() < 1e-10,
            "Clayton at (0.5,0.5) = {val}, expected {expected}"
        );
    }

    #[test]
    fn frank_copula() {
        let c = FrankCopula::new(5.0);
        let val = c.value(0.5, 0.5);
        // Should be between min and max copula at this point
        assert!(val > 0.0 && val < 0.5, "Frank at (0.5,0.5) = {val}");
    }

    #[test]
    fn gumbel_copula_theta_1_is_independence() {
        let c = GumbelCopula::new(1.0);
        let val = c.value(0.3, 0.7);
        // θ=1 gives independence copula: u*v = 0.21
        assert!(
            (val - 0.21).abs() < 1e-10,
            "Gumbel(θ=1) at (0.3,0.7) = {val}"
        );
    }

    #[test]
    fn copula_boundary_conditions() {
        // All copulas should satisfy C(0, v) = 0, C(u, 0) = 0, C(1, v) = v, C(u, 1) = u
        let copulas: Vec<Box<dyn Copula>> = vec![
            Box::new(MinCopula),
            Box::new(MaxCopula),
            Box::new(IndependenceCopula),
            Box::new(ClaytonCopula::new(1.0)),
            Box::new(FrankCopula::new(2.0)),
            Box::new(GumbelCopula::new(2.0)),
            Box::new(FarlieCopula::new(0.5)),
        ];

        for (i, c) in copulas.iter().enumerate() {
            assert!(
                c.value(0.0001, 0.5) < 0.01,
                "copula {i}: C(~0, 0.5) should be ~0"
            );
            assert!(
                c.value(0.5, 0.0001) < 0.01,
                "copula {i}: C(0.5, ~0) should be ~0"
            );
            let v = c.value(0.999, 0.5);
            assert!(
                (v - 0.5).abs() < 0.05,
                "copula {i}: C(~1, 0.5) = {v}, expected ~0.5"
            );
        }
    }
}
