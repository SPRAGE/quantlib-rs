//! Poisson distribution (translates `ql/math/distributions/poissondistribution.hpp`).
//!
//! Wraps the `statrs` crate's Poisson implementation.

use ql_core::Real;
use statrs::distribution::{Discrete, DiscreteCDF, Poisson};

/// Poisson distribution with mean `lambda`.
///
/// Corresponds to `QuantLib::CumulativePoissonDistribution` /
/// `QuantLib::PoissonDistribution`.
#[derive(Debug, Clone)]
pub struct PoissonDistribution {
    dist: Poisson,
    lambda: Real,
}

impl PoissonDistribution {
    /// Create a Poisson distribution with the given mean `lambda`.
    ///
    /// # Panics
    /// Panics if `lambda <= 0`.
    pub fn new(lambda: Real) -> Self {
        assert!(lambda > 0.0, "lambda must be positive");
        Self {
            dist: Poisson::new(lambda).expect("invalid lambda"),
            lambda,
        }
    }

    /// Mean parameter λ.
    pub fn lambda(&self) -> Real {
        self.lambda
    }

    /// Probability mass function P(X = k).
    pub fn pmf(&self, k: u64) -> Real {
        self.dist.pmf(k)
    }

    /// Cumulative distribution function P(X ≤ k).
    pub fn cdf(&self, k: u64) -> Real {
        self.dist.cdf(k)
    }

    /// Mean of the distribution (= λ).
    pub fn mean(&self) -> Real {
        self.lambda
    }

    /// Variance of the distribution (= λ).
    pub fn variance(&self) -> Real {
        self.lambda
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poisson_pmf() {
        let d = PoissonDistribution::new(3.0);
        // P(X=0) = e^{-3}
        let expected = (-3.0_f64).exp();
        assert!(
            (d.pmf(0) - expected).abs() < 1e-10,
            "got {}, expected {}",
            d.pmf(0),
            expected
        );
        // P(X=3) = e^{-3} * 3^3 / 3! = e^{-3} * 27 / 6
        let expected3 = expected * 27.0 / 6.0;
        assert!(
            (d.pmf(3) - expected3).abs() < 1e-10,
            "got {}, expected {}",
            d.pmf(3),
            expected3
        );
    }

    #[test]
    fn poisson_cdf_sums_to_one() {
        let d = PoissonDistribution::new(5.0);
        // For large k, CDF should approach 1
        assert!((d.cdf(50) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn poisson_cdf_monotone() {
        let d = PoissonDistribution::new(2.0);
        let mut prev = 0.0;
        for k in 0..20 {
            let c = d.cdf(k);
            assert!(c >= prev, "CDF not monotone at k={k}");
            prev = c;
        }
    }
}
