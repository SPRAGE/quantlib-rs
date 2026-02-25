//! Chi-square distribution (translates `ql/math/distributions/chisquaredistribution.hpp`).
//!
//! Wraps the `statrs` crate's chi-squared implementation to match the QuantLib API.

use ql_core::Real;
use statrs::distribution::{ChiSquared, Continuous, ContinuousCDF};

/// Chi-square distribution with `df` degrees of freedom.
///
/// Corresponds to `QuantLib::CumulativeChiSquareDistribution` /
/// `QuantLib::NonCentralChiSquareDistribution`.
#[derive(Debug, Clone)]
pub struct ChiSquareDistribution {
    dist: ChiSquared,
    df: Real,
}

impl ChiSquareDistribution {
    /// Create a chi-square distribution with the given degrees of freedom.
    ///
    /// # Panics
    /// Panics if `df <= 0`.
    pub fn new(df: Real) -> Self {
        assert!(df > 0.0, "degrees of freedom must be positive");
        Self {
            dist: ChiSquared::new(df).expect("invalid degrees of freedom"),
            df,
        }
    }

    /// Degrees of freedom.
    pub fn df(&self) -> Real {
        self.df
    }

    /// Probability density function.
    pub fn pdf(&self, x: Real) -> Real {
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
    fn chi_square_cdf() {
        let d = ChiSquareDistribution::new(2.0);
        // P(X ≤ 0) = 0 for chi-square
        assert!((d.cdf(0.0)).abs() < 1e-12);
        // P(X ≤ ∞) → 1
        assert!((d.cdf(100.0) - 1.0).abs() < 1e-6);
        // For df=2, CDF(x) = 1 - e^(-x/2)
        let x: Real = 4.0;
        let expected = 1.0 - (-x / 2.0).exp();
        assert!(
            (d.cdf(x) - expected).abs() < 1e-10,
            "got {}, expected {}",
            d.cdf(x),
            expected
        );
    }

    #[test]
    fn chi_square_pdf() {
        let d = ChiSquareDistribution::new(2.0);
        // For df=2, pdf(x) = 0.5 * e^(-x/2)
        let x: Real = 3.0;
        let expected = 0.5 * (-x / 2.0).exp();
        assert!(
            (d.pdf(x) - expected).abs() < 1e-10,
            "got {}, expected {}",
            d.pdf(x),
            expected
        );
    }

    #[test]
    fn chi_square_inverse_cdf() {
        let d = ChiSquareDistribution::new(5.0);
        for p in [0.1, 0.25, 0.5, 0.75, 0.9] {
            let x = d.inverse_cdf(p);
            let p2 = d.cdf(x);
            assert!(
                (p2 - p).abs() < 1e-4,
                "roundtrip failed for p={p}: got {p2}"
            );
        }
    }
}
