//! Normal (Gaussian) distribution (translates `ql/math/distributions/normaldistribution.hpp`).

use ql_core::Real;
use std::f64::consts::PI;

/// The standard normal probability density function.
///
/// `φ(x) = exp(-x²/2) / √(2π)`
#[inline]
pub fn normal_pdf(x: Real) -> Real {
    (-0.5 * x * x).exp() / (2.0 * PI).sqrt()
}

/// The standard normal cumulative distribution function Φ(x).
///
/// Uses a high-accuracy rational Chebyshev approximation.
/// Maximum absolute error < 7.5×10⁻⁸.
pub fn normal_cdf(x: Real) -> Real {
    // Abramowitz & Stegun 26.2.17 — maximum |error| < 7.5e-8
    // but special-case x = 0 for exact 0.5
    if x == 0.0 {
        return 0.5;
    }
    let sign = if x < 0.0 { -1.0_f64 } else { 1.0_f64 };
    let t = 1.0 / (1.0 + 0.2316419 * x.abs());
    let poly = t
        * (0.319_381_530
            + t * (-0.356_563_782
                + t * (1.781_477_937
                    + t * (-1.821_255_978 + t * 1.330_274_429))));
    let pdf = normal_pdf(x);
    0.5 + sign * (0.5 - poly * pdf)
}

/// The inverse standard normal CDF (probit function).
///
/// Translates QuantLib's `InverseCumulativeNormal` (Moro / Acklam algorithm).
/// Uses a rational approximation from Peter J. Acklam.
pub fn normal_cdf_inverse(p: Real) -> Real {
    assert!(p > 0.0 && p < 1.0, "p must be in (0, 1)");
    acklam_inverse(p)
}

/// Peter J. Acklam's rational approximation to the inverse normal CDF.
///
/// Maximum absolute error < 1.15e-9.
fn acklam_inverse(p: Real) -> Real {
    const A: [f64; 6] = [
        -3.969_683_028_665_376e+01,
        2.209_460_984_245_205e+02,
        -2.759_285_104_469_687e+02,
        1.383_577_518_672_69e2,
        -3.066_479_806_614_716e+01,
        2.506_628_277_459_239e+00,
    ];
    const B: [f64; 5] = [
        -5.447_609_879_822_406e+01,
        1.615_858_368_580_409e+02,
        -1.556_989_798_598_866e+02,
        6.680_131_188_771_972e+01,
        -1.328_068_155_288_572e+01,
    ];
    const C: [f64; 6] = [
        -7.784_894_002_430_293e-03,
        -3.223_964_580_411_365e-01,
        -2.400_758_277_161_838e+00,
        -2.549_732_539_343_734e+00,
        4.374_664_141_464_968e+00,
        2.938_163_982_698_783e+00,
    ];
    const D: [f64; 4] = [
        7.784_695_709_041_462e-03,
        3.224_671_290_700_398e-01,
        2.445_134_137_142_996e+00,
        3.754_408_661_907_416e+00,
    ];

    const P_LOW: f64 = 0.02425;
    const P_HIGH: f64 = 1.0 - P_LOW;

    if p < P_LOW {
        let q = (-2.0 * p.ln()).sqrt();
        (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    } else if p <= P_HIGH {
        let q = p - 0.5;
        let r = q * q;
        (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5]) * q
            / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    }
}

/// Bivariate normal CDF using the method of Drezner and Wesolowsky.
///
/// Computes `P(X ≤ a, Y ≤ b)` where `(X, Y)` is standard bivariate normal
/// with correlation `rho`.
pub fn bivariate_normal_cdf(a: Real, b: Real, rho: Real) -> Real {
    // Abramowitz & Stegun method (Drezner approximation)
    const X: [f64; 5] = [0.24840615, 0.39233107, 0.21141819, 0.03324666, 0.00082485334];
    const Y: [f64; 5] = [0.10024215, 0.48281397, 1.06094980, 1.77972940, 2.66976040];

    let a = a / (2.0_f64).sqrt();
    let b = b / (2.0_f64).sqrt();

    let tp = 2.0 * PI;

    if rho.abs() < 0.7 {
        let hs = -(a * a + b * b) / 2.0;
        let asr = rho.asin();
        let mut sum = 0.0;
        for i in 0..5 {
            for sn in [-1.0_f64, 1.0] {
                let xs = (asr * (sn * Y[i] + 1.0) / 2.0).sin();
                sum += X[i]
                    * ((xs * (a * a + b * b) - 2.0 * a * b * xs)
                        / (1.0 - xs * xs) + hs)
                        .exp();
            }
        }
        return (asr * sum / (2.0 * tp)) + normal_cdf(a * std::f64::consts::SQRT_2) * normal_cdf(b * std::f64::consts::SQRT_2);
    }

    // |rho| >= 0.7
    let _hs = -(a * a + b * b);
    let asr = if rho > 0.0 {
        ((a - b) / (2.0 * (1.0 - rho)).sqrt()).atan()
    } else {
        -(-rho).sqrt().atan2(1.0 - rho)
    };
    let mut sum = (0..5)
        .map(|i| {
            X[i] * {
                let mut s = 0.0;
                for sn in [-1.0_f64, 1.0] {
                    let xs = (asr * (sn * Y[i] + 1.0) / 2.0).sin();
                    s += ((xs * (a * a + b * b) - 2.0 * a * b * xs)
                        / (2.0 * (1.0 - xs * xs)))
                        .exp();
                }
                s
            }
        })
        .sum::<f64>();
    sum = asr * sum / (2.0 * tp);
    if rho > 0.0 {
        sum + normal_cdf(a.min(b) * std::f64::consts::SQRT_2)
    } else {
        let tmp = -normal_cdf(a * std::f64::consts::SQRT_2) + normal_cdf(b * std::f64::consts::SQRT_2);
        (-sum + tmp).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_pdf_at_zero() {
        let expected = 1.0 / (2.0 * PI).sqrt();
        assert!((normal_pdf(0.0) - expected).abs() < 1e-12);
    }

    #[test]
    fn normal_cdf_at_zero() {
        assert!((normal_cdf(0.0) - 0.5).abs() < 1e-12);
    }

    #[test]
    fn normal_cdf_tails() {
        assert!((normal_cdf(10.0) - 1.0).abs() < 1e-10);
        assert!(normal_cdf(-10.0) < 1e-10);
    }

    #[test]
    fn inverse_cdf_roundtrip() {
        for p in [0.01, 0.1, 0.25, 0.5, 0.75, 0.9, 0.99] {
            let x = normal_cdf_inverse(p);
            let p2 = normal_cdf(x);
            assert!(
                (p2 - p).abs() < 1e-6,
                "roundtrip failed for p={p}: got {p2}"
            );
        }
    }
}
