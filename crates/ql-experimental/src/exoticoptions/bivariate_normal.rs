//! Bivariate cumulative normal distribution — Drezner (1978) algorithm.
//!
//! This is a direct port of `QuantLib::BivariateCumulativeNormalDistributionDr78`
//! which uses a 5×5 product Gauss-Hermite quadrature rule for the case
//! `a ≤ 0, b ≤ 0, ρ ≤ 0`, and reduces all other sign combinations to that case
//! via symmetry relations.
//!
//! This implementation matches the QuantLib Dr78 output exactly, which is needed
//! for accurate pricing of exotic options using bivariate normal CDF.

use ql_math::distributions::normal_cdf;

const X: [f64; 5] = [
    0.24840615,
    0.39233107,
    0.21141819,
    0.03324666,
    0.00082485334,
];
const Y: [f64; 5] = [0.10024215, 0.48281397, 1.06094980, 1.77972940, 2.66976040];

/// Bivariate cumulative normal distribution using the Drezner (1978) algorithm.
///
/// Computes `P(X ≤ a, Y ≤ b)` where `(X, Y)` is standard bivariate normal
/// with correlation `rho`.
///
/// This matches `QuantLib::BivariateCumulativeNormalDistributionDr78`.
pub fn bivariate_normal_cdf_dr78(a: f64, b: f64, rho: f64) -> f64 {
    let cum_a = normal_cdf(a);
    let cum_b = normal_cdf(b);
    let max_cum = cum_a.max(cum_b);
    let min_cum = cum_a.min(cum_b);

    if 1.0 - max_cum < 1e-15 {
        return min_cum;
    }
    if min_cum < 1e-15 {
        return min_cum;
    }

    let rho2 = rho * rho;
    let a1 = a / (2.0 * (1.0 - rho2)).sqrt();
    let b1 = b / (2.0 * (1.0 - rho2)).sqrt();

    if a <= 0.0 && b <= 0.0 && rho <= 0.0 {
        // Direct 5×5 product quadrature
        let mut sum = 0.0;
        for i in 0..5 {
            for j in 0..5 {
                sum += X[i]
                    * X[j]
                    * (a1 * (2.0 * Y[i] - a1)
                        + b1 * (2.0 * Y[j] - b1)
                        + 2.0 * rho * (Y[i] - a1) * (Y[j] - b1))
                        .exp();
            }
        }
        (1.0 - rho2).sqrt() / std::f64::consts::PI * sum
    } else if a <= 0.0 && b >= 0.0 && rho >= 0.0 {
        cum_a - bivariate_normal_cdf_dr78(a, -b, -rho)
    } else if a >= 0.0 && b <= 0.0 && rho >= 0.0 {
        cum_b - bivariate_normal_cdf_dr78(-a, b, -rho)
    } else if a >= 0.0 && b >= 0.0 && rho <= 0.0 {
        cum_a + cum_b - 1.0 + bivariate_normal_cdf_dr78(-a, -b, rho)
    } else if a * b * rho > 0.0 {
        let denom = (a * a - 2.0 * rho * a * b + b * b).sqrt();
        let rho1 = (rho * a - b) * (if a > 0.0 { 1.0 } else { -1.0 }) / denom;
        let rho2_val = (rho * b - a) * (if b > 0.0 { 1.0 } else { -1.0 }) / denom;
        let delta =
            (1.0 - (if a > 0.0 { 1.0 } else { -1.0 }) * (if b > 0.0 { 1.0 } else { -1.0 })) / 4.0;
        bivariate_normal_cdf_dr78(a, 0.0, rho1) + bivariate_normal_cdf_dr78(b, 0.0, rho2_val)
            - delta
    } else {
        // Fall through — shouldn't happen for valid inputs
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dr78_zero_correlation() {
        // Independent normals: P(X≤0, Y≤0) = 0.25
        let result = bivariate_normal_cdf_dr78(0.0, 0.0, 0.0);
        assert!(
            (result - 0.25).abs() < 1e-6,
            "Dr78(0,0,0) = {result}, expected 0.25"
        );
    }

    #[test]
    fn test_dr78_identity_formula() {
        // P(X≤0, Y≤0; ρ) = 0.25 + arcsin(ρ)/(2π)
        for rho in [-0.5, 0.0, 0.3, 0.5, 0.75, 0.9] {
            let result = bivariate_normal_cdf_dr78(0.0, 0.0, rho);
            let expected = 0.25 + rho.asin() / (2.0 * std::f64::consts::PI);
            assert!(
                (result - expected).abs() < 1e-4,
                "Dr78(0,0,{rho}) = {result}, expected {expected}"
            );
        }
    }
}
