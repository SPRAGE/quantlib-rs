//! Gamma distribution (translates `ql/math/distributions/gammadistribution.hpp`).
//!
//! Wraps the `statrs` crate's gamma implementation.

use ql_core::Real;
use statrs::distribution::{Continuous, ContinuousCDF, Gamma};

/// Gamma distribution with shape `a` and rate `b` (scale = 1/b).
///
/// Corresponds to `QuantLib::GammaDistribution` / `QuantLib::CumulativeGammaDistribution`.
#[derive(Debug, Clone)]
pub struct GammaDistribution {
    dist: Gamma,
    shape: Real,
    rate: Real,
}

impl GammaDistribution {
    /// Create a gamma distribution with given shape `a` and rate `b`.
    ///
    /// The scale parameter is `1/b`. For the standard gamma distribution use
    /// `b = 1.0`.
    ///
    /// # Panics
    /// Panics if `a <= 0` or `b <= 0`.
    pub fn new(shape: Real, rate: Real) -> Self {
        assert!(shape > 0.0 && rate > 0.0, "shape and rate must be positive");
        Self {
            dist: Gamma::new(shape, 1.0 / rate).expect("invalid gamma parameters"),
            shape,
            rate,
        }
    }

    /// Shape parameter.
    pub fn shape(&self) -> Real {
        self.shape
    }

    /// Rate parameter.
    pub fn rate(&self) -> Real {
        self.rate
    }

    /// Probability density function.
    pub fn pdf(&self, x: Real) -> Real {
        if x < 0.0 {
            return 0.0;
        }
        self.dist.pdf(x)
    }

    /// Cumulative distribution function P(X ≤ x).
    pub fn cdf(&self, x: Real) -> Real {
        if x <= 0.0 {
            return 0.0;
        }
        self.dist.cdf(x)
    }

    /// Inverse CDF (quantile function).
    pub fn inverse_cdf(&self, p: Real) -> Real {
        assert!((0.0..=1.0).contains(&p), "p must be in [0, 1]");
        self.dist.inverse_cdf(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gamma_cdf() {
        // Gamma(1, 1) = Exponential(1), CDF = 1 - e^(-x)
        let d = GammaDistribution::new(1.0, 1.0);
        let x: Real = 2.0;
        let expected = 1.0 - (-x).exp();
        assert!(
            (d.cdf(x) - expected).abs() < 1e-10,
            "got {}, expected {}",
            d.cdf(x),
            expected
        );
    }

    #[test]
    fn gamma_pdf() {
        // Gamma(1, 1) = Exponential(1), pdf = e^(-x)
        let d = GammaDistribution::new(1.0, 1.0);
        let x: Real = 1.5;
        let expected = (-x).exp();
        assert!(
            (d.pdf(x) - expected).abs() < 1e-10,
            "got {}, expected {}",
            d.pdf(x),
            expected
        );
    }

    #[test]
    fn gamma_inverse_cdf() {
        let d = GammaDistribution::new(3.0, 2.0);
        for p in [0.1, 0.25, 0.5, 0.75, 0.9] {
            let x = d.inverse_cdf(p);
            let p2 = d.cdf(x);
            assert!(
                (p2 - p).abs() < 1e-6,
                "roundtrip failed for p={p}: got {p2}"
            );
        }
    }
}
