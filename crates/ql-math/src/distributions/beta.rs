//! Beta distribution (translates `ql/math/distributions/betadistribution.hpp`
//! and related gamma/beta function utilities).

use ql_core::Real;
use statrs::distribution::{Beta, Continuous, ContinuousCDF};

/// Beta distribution with shape parameters `α` and `β`.
///
/// Corresponds to `QuantLib::BetaDistribution` + `QuantLib::IncompleteBetaFunction`.
#[derive(Debug, Clone)]
pub struct BetaDistribution {
    inner: Beta,
    alpha: Real,
    beta: Real,
}

impl BetaDistribution {
    /// Create a Beta(α, β) distribution.
    pub fn new(alpha: Real, beta: Real) -> Self {
        let inner = Beta::new(alpha, beta).expect("invalid beta distribution parameters");
        Self { inner, alpha, beta }
    }

    /// Shape parameter α.
    pub fn alpha(&self) -> Real {
        self.alpha
    }

    /// Shape parameter β.
    pub fn beta(&self) -> Real {
        self.beta
    }

    /// Probability density function.
    pub fn pdf(&self, x: Real) -> Real {
        self.inner.pdf(x)
    }

    /// Cumulative distribution function (regularized incomplete beta function).
    pub fn cdf(&self, x: Real) -> Real {
        self.inner.cdf(x)
    }
}

/// The Gamma function Γ(z).
///
/// Uses the Lanczos approximation via `statrs`.
pub fn gamma_function(z: Real) -> Real {
    statrs::function::gamma::gamma(z)
}

/// The natural logarithm of the Gamma function: ln Γ(z).
pub fn log_gamma(z: Real) -> Real {
    statrs::function::gamma::ln_gamma(z)
}

/// The error function erf(x).
///
/// erf(x) = 2/√π ∫₀ˣ e^{-t²} dt
pub fn error_function(x: Real) -> Real {
    statrs::function::erf::erf(x)
}

/// The complementary error function erfc(x) = 1 − erf(x).
pub fn erfc(x: Real) -> Real {
    statrs::function::erf::erfc(x)
}

/// The inverse error function erf⁻¹(x).
pub fn inverse_error_function(x: Real) -> Real {
    statrs::function::erf::erf_inv(x)
}

/// The regularized incomplete beta function I_x(a, b).
pub fn incomplete_beta(a: Real, b: Real, x: Real) -> Real {
    statrs::function::beta::beta_reg(a, b, x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beta_pdf_uniform() {
        // Beta(1,1) is the uniform distribution
        let d = BetaDistribution::new(1.0, 1.0);
        assert!((d.pdf(0.5) - 1.0).abs() < 1e-10);
        assert!((d.cdf(0.5) - 0.5).abs() < 1e-10);
    }

    #[test]
    fn beta_cdf_boundary() {
        let d = BetaDistribution::new(2.0, 5.0);
        assert!((d.cdf(0.0)).abs() < 1e-10);
        assert!((d.cdf(1.0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn gamma_function_integers() {
        // Γ(n) = (n-1)! for positive integers
        assert!((gamma_function(1.0) - 1.0).abs() < 1e-10);
        assert!((gamma_function(2.0) - 1.0).abs() < 1e-10);
        assert!((gamma_function(5.0) - 24.0).abs() < 1e-10);
        assert!((gamma_function(6.0) - 120.0).abs() < 1e-8);
    }

    #[test]
    fn error_function_values() {
        assert!(error_function(0.0).abs() < 1e-15);
        assert!((error_function(1.0) - 0.8427007929).abs() < 1e-6);
        assert!((erfc(0.0) - 1.0).abs() < 1e-15);
    }

    #[test]
    fn inverse_erf_roundtrip() {
        for &x in &[0.1, 0.3, 0.5, 0.7, 0.9] {
            let y = error_function(x);
            let x_back = inverse_error_function(y);
            assert!(
                (x - x_back).abs() < 1e-10,
                "roundtrip failed for x={x}: got {x_back}"
            );
        }
    }
}
