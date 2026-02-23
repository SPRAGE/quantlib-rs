//! Student's t-distribution (translates `ql/math/distributions/studenttdistribution.hpp`).
//!
//! Wraps the `statrs` crate's Student-t implementation.

use ql_core::Real;
use statrs::distribution::{ContinuousCDF, Continuous, StudentsT};

/// Student's t-distribution with `df` degrees of freedom.
///
/// Corresponds to `QuantLib::StudentDistribution` /
/// `QuantLib::CumulativeStudentDistribution`.
#[derive(Debug, Clone)]
pub struct StudentTDistribution {
    dist: StudentsT,
    df: Real,
}

impl StudentTDistribution {
    /// Create a Student-t distribution with the given degrees of freedom.
    ///
    /// # Panics
    /// Panics if `df <= 0`.
    pub fn new(df: Real) -> Self {
        assert!(df > 0.0, "degrees of freedom must be positive");
        Self {
            // location=0, scale=1 (standard t-distribution)
            dist: StudentsT::new(0.0, 1.0, df).expect("invalid degrees of freedom"),
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

    /// Cumulative distribution function P(T ≤ x).
    pub fn cdf(&self, x: Real) -> Real {
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
    fn student_t_symmetry() {
        let d = StudentTDistribution::new(5.0);
        // CDF(0) should be 0.5 by symmetry
        assert!(
            (d.cdf(0.0) - 0.5).abs() < 1e-12,
            "CDF(0) = {}",
            d.cdf(0.0)
        );
        // PDF is symmetric: pdf(-x) == pdf(x)
        let x = 1.5;
        assert!(
            (d.pdf(x) - d.pdf(-x)).abs() < 1e-12,
            "pdf({x}) = {}, pdf({}) = {}",
            d.pdf(x),
            -x,
            d.pdf(-x)
        );
    }

    #[test]
    fn student_t_cdf_range() {
        let d = StudentTDistribution::new(10.0);
        assert!(d.cdf(-100.0) < 1e-10);
        assert!((d.cdf(100.0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn student_t_inverse_roundtrip() {
        let d = StudentTDistribution::new(4.0);
        for p in [0.01, 0.1, 0.25, 0.5, 0.75, 0.9, 0.99] {
            let x = d.inverse_cdf(p);
            let p2 = d.cdf(x);
            assert!(
                (p2 - p).abs() < 1e-6,
                "roundtrip failed for p={p}: got {p2}"
            );
        }
    }

    #[test]
    fn student_t_converges_to_normal() {
        // With very large df, Student-t ≈ Normal(0,1)
        let t = StudentTDistribution::new(1e6);
        let normal_cdf_175 = 0.959_940_843; // Φ(1.75) ≈ 0.9599
        assert!(
            (t.cdf(1.75) - normal_cdf_175).abs() < 1e-4,
            "t-CDF(1.75) = {} vs normal {}",
            t.cdf(1.75),
            normal_cdf_175
        );
    }
}
