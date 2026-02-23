//! Probability distributions (translates `ql/math/distributions/`).
//!
//! Provides Normal, Chi-Square, Gamma, Student-t, Poisson, and Binomial
//! distributions, delegating to the `statrs` crate where appropriate.

pub mod binomial;
pub mod chi_square;
pub mod gamma;
pub mod normal;
pub mod poisson;
pub mod student_t;

pub use binomial::BinomialDistribution;
pub use chi_square::ChiSquareDistribution;
pub use gamma::GammaDistribution;
pub use normal::{bivariate_normal_cdf, normal_cdf, normal_cdf_inverse, normal_pdf};
pub use poisson::PoissonDistribution;
pub use student_t::StudentTDistribution;
