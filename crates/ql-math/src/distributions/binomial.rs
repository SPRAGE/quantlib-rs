//! Binomial distribution (translates `ql/math/distributions/binomialdistribution.hpp`).
//!
//! Wraps the `statrs` crate's binomial implementation.

use ql_core::Real;
use statrs::distribution::{Binomial, Discrete, DiscreteCDF};

/// Binomial distribution with `n` trials and success probability `p`.
///
/// Corresponds to `QuantLib::CumulativeBinomialDistribution` /
/// `QuantLib::BinomialDistribution`.
#[derive(Debug, Clone)]
pub struct BinomialDistribution {
    dist: Binomial,
    n: u64,
    p: Real,
}

impl BinomialDistribution {
    /// Create a binomial distribution with `n` trials and probability `p`.
    ///
    /// # Panics
    /// Panics if `p` is not in `[0, 1]` or `n` is 0.
    pub fn new(p: Real, n: u64) -> Self {
        assert!((0.0..=1.0).contains(&p), "p must be in [0, 1]");
        assert!(n > 0, "n must be positive");
        Self {
            dist: Binomial::new(p, n).expect("invalid binomial parameters"),
            n,
            p,
        }
    }

    /// Number of trials.
    pub fn n(&self) -> u64 {
        self.n
    }

    /// Success probability.
    pub fn p(&self) -> Real {
        self.p
    }

    /// Probability mass function P(X = k).
    pub fn pmf(&self, k: u64) -> Real {
        self.dist.pmf(k)
    }

    /// Cumulative distribution function P(X ≤ k).
    pub fn cdf(&self, k: u64) -> Real {
        self.dist.cdf(k)
    }

    /// Mean of the distribution (= np).
    pub fn mean(&self) -> Real {
        self.n as Real * self.p
    }

    /// Variance of the distribution (= np(1-p)).
    pub fn variance(&self) -> Real {
        self.n as Real * self.p * (1.0 - self.p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binomial_fair_coin() {
        let d = BinomialDistribution::new(0.5, 10);
        assert!((d.mean() - 5.0).abs() < 1e-12);
        assert!((d.variance() - 2.5).abs() < 1e-12);
    }

    #[test]
    fn binomial_pmf_sums_to_one() {
        let d = BinomialDistribution::new(0.3, 20);
        let total: Real = (0..=20).map(|k| d.pmf(k)).sum();
        assert!(
            (total - 1.0).abs() < 1e-10,
            "sum of PMF = {total}"
        );
    }

    #[test]
    fn binomial_cdf_boundary() {
        let d = BinomialDistribution::new(0.7, 5);
        // CDF(5) should be 1.0 (all outcomes covered)
        assert!((d.cdf(5) - 1.0).abs() < 1e-10);
        // CDF(0) = (1-p)^n = 0.3^5
        let expected = 0.3_f64.powi(5);
        assert!(
            (d.cdf(0) - expected).abs() < 1e-10,
            "CDF(0) = {}, expected {}",
            d.cdf(0),
            expected
        );
    }

    #[test]
    fn binomial_cdf_monotone() {
        let d = BinomialDistribution::new(0.4, 15);
        let mut prev = 0.0;
        for k in 0..=15 {
            let c = d.cdf(k);
            assert!(c >= prev, "CDF not monotone at k={k}");
            prev = c;
        }
    }
}
