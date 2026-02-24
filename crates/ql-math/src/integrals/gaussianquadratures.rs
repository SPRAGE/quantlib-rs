//! Gaussian quadrature rules (translates `ql/math/integrals/gaussianquadratures.hpp`).
//!
//! Provides classical Gauss quadrature rules with pre-computed nodes and weights
//! for common families: Legendre, Hermite, Laguerre, Chebyshev, and Jacobi.

use ql_core::Real;
use std::f64::consts::PI;

/// A Gauss quadrature rule defined by nodes and weights.
///
/// Corresponds to `QuantLib::GaussianQuadrature`.
#[derive(Debug, Clone)]
pub struct GaussianQuadrature {
    x: Vec<Real>,
    w: Vec<Real>,
}

impl GaussianQuadrature {
    /// Quadrature nodes.
    pub fn x(&self) -> &[Real] {
        &self.x
    }

    /// Quadrature weights.
    pub fn w(&self) -> &[Real] {
        &self.w
    }

    /// Number of quadrature points.
    pub fn order(&self) -> usize {
        self.x.len()
    }

    /// Evaluate ∫ f(x) w(x) dx ≈ Σ wᵢ f(xᵢ).
    pub fn integrate<F: Fn(Real) -> Real>(&self, f: F) -> Real {
        self.x.iter().zip(self.w.iter()).map(|(&xi, &wi)| wi * f(xi)).sum()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Gauss-Legendre quadrature on [-1, 1]
// ═══════════════════════════════════════════════════════════════════════════════

/// Gauss-Legendre quadrature on [−1, 1].
///
/// Corresponds to `QuantLib::GaussLegendreIntegration`.
pub struct GaussLegendreIntegration;

impl GaussLegendreIntegration {
    /// Build a Gauss-Legendre quadrature of given `order`.
    pub fn new(order: usize) -> GaussianQuadrature {
        gauss_jacobi_nodes_weights(order, 0.0, 0.0)
    }

    /// Integrate `f` on [a, b] by mapping to [−1, 1].
    pub fn integrate<F: Fn(Real) -> Real>(order: usize, f: F, a: Real, b: Real) -> Real {
        let q = Self::new(order);
        let half = 0.5 * (b - a);
        let mid = 0.5 * (a + b);
        q.x()
            .iter()
            .zip(q.w().iter())
            .map(|(&xi, &wi)| wi * f(mid + half * xi))
            .sum::<Real>()
            * half
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Gauss-Hermite quadrature (probabilist: weight e^{-x²/2})
// ═══════════════════════════════════════════════════════════════════════════════

/// Gauss-Hermite quadrature (physicists' convention: weight e^{-x²}).
///
/// Corresponds to `QuantLib::GaussHermiteIntegration`.
pub struct GaussHermiteIntegration;

impl GaussHermiteIntegration {
    /// Build a Gauss-Hermite quadrature of given `order`.
    ///
    /// Uses the Golub-Welsch algorithm with the recurrence relation for
    /// Hermite polynomials: Hₙ₊₁(x) = x Hₙ(x) − n Hₙ₋₁(x).
    pub fn new(order: usize) -> GaussianQuadrature {
        let n = order;
        // Tridiagonal: α_i = 0, β_i = i  (H_{i+1}(x) = x H_i(x) - i H_{i-1}(x))
        let alpha: Vec<Real> = vec![0.0; n];
        let beta: Vec<Real> = (0..n).map(|i| if i == 0 { PI.sqrt() } else { (i as Real) / 2.0 }).collect();
        golub_welsch(&alpha, &beta)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Gauss-Laguerre quadrature (weight e^{-x} on [0, ∞))
// ═══════════════════════════════════════════════════════════════════════════════

/// Gauss-Laguerre quadrature (weight e^{-x} on [0, ∞)).
///
/// Corresponds to `QuantLib::GaussLaguerreIntegration`.
pub struct GaussLaguerreIntegration;

impl GaussLaguerreIntegration {
    /// Build a Gauss-Laguerre quadrature with generalized parameter `s` (default 0).
    ///
    /// For the generalized Laguerre weight x^s e^{-x}.
    pub fn new(order: usize, s: Real) -> GaussianQuadrature {
        let n = order;
        // Recurrence: α_i = 2i + 1 + s, β_i = i(i + s)
        let alpha: Vec<Real> = (0..n).map(|i| 2.0 * (i as Real) + 1.0 + s).collect();
        let beta: Vec<Real> = (0..n)
            .map(|i| {
                if i == 0 {
                    statrs::function::gamma::gamma(s + 1.0)
                } else {
                    (i as Real) * ((i as Real) + s)
                }
            })
            .collect();
        golub_welsch(&alpha, &beta)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Gauss-Chebyshev quadrature (weight 1/√(1−x²) on [-1, 1])
// ═══════════════════════════════════════════════════════════════════════════════

/// Gauss-Chebyshev quadrature of the first kind.
///
/// weight: 1/√(1−x²) on [−1, 1].
/// Corresponds to `QuantLib::GaussChebyshevIntegration`.
pub struct GaussChebyshevIntegration;

impl GaussChebyshevIntegration {
    /// Build a Gauss-Chebyshev (first-kind) quadrature of given `order`.
    ///
    /// Nodes are xᵢ = cos((2i+1)π / (2n)), weights are wᵢ = π/n.
    pub fn new(order: usize) -> GaussianQuadrature {
        let n = order;
        let w_val = PI / (n as Real);
        let x: Vec<Real> = (0..n)
            .map(|i| ((2 * i + 1) as Real * PI / (2.0 * n as Real)).cos())
            .collect();
        let w: Vec<Real> = vec![w_val; n];
        GaussianQuadrature { x, w }
    }
}

/// Gauss-Chebyshev quadrature of the second kind.
///
/// weight: √(1−x²) on [−1, 1].
/// Corresponds to `QuantLib::GaussChebyshev2ndIntegration`.
pub struct GaussChebyshev2ndIntegration;

impl GaussChebyshev2ndIntegration {
    /// Build a Gauss-Chebyshev-2nd quadrature of given `order`.
    pub fn new(order: usize) -> GaussianQuadrature {
        let n = order;
        let x: Vec<Real> = (0..n)
            .map(|i| ((i + 1) as Real * PI / ((n + 1) as Real)).cos())
            .collect();
        let w: Vec<Real> = (0..n)
            .map(|i| {
                let theta = (i + 1) as Real * PI / ((n + 1) as Real);
                PI / ((n + 1) as Real) * theta.sin().powi(2)
            })
            .collect();
        GaussianQuadrature { x, w }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Golub-Welsch algorithm for general orthogonal-polynomial quadrature
// ═══════════════════════════════════════════════════════════════════════════════

/// Compute Gauss-Jacobi quadrature nodes and weights for the weight function
/// (1 − x)^α (1 + x)^β on [−1, 1].
///
/// Uses the Golub-Welsch algorithm.
pub fn gauss_jacobi_nodes_weights(n: usize, alpha: Real, beta: Real) -> GaussianQuadrature {
    if n == 0 {
        return GaussianQuadrature {
            x: vec![],
            w: vec![],
        };
    }

    // Three-term recurrence coefficients for Jacobi polynomials
    let mut a_diag = vec![0.0_f64; n];
    let mut b_sub = vec![0.0_f64; n]; // b_sub[0] = μ₀ = ∫ w(x) dx

    let ab = alpha + beta;

    // μ₀ = ∫_{-1}^{1} (1-x)^α (1+x)^β dx = 2^(α+β+1) B(α+1, β+1)
    let mu0 = 2.0_f64.powf(ab + 1.0)
        * statrs::function::gamma::gamma(alpha + 1.0)
        * statrs::function::gamma::gamma(beta + 1.0)
        / statrs::function::gamma::gamma(ab + 2.0);

    if n == 1 {
        let x0 = (beta - alpha) / (ab + 2.0);
        return GaussianQuadrature {
            x: vec![x0],
            w: vec![mu0],
        };
    }

    // a_0
    a_diag[0] = (beta - alpha) / (ab + 2.0);
    // b_1 (used as sub-diagonal)
    b_sub[0] = mu0; // this will be sqrt'd later, used as μ₀ here

    for i in 1..n {
        let ii = i as Real;
        let two_i = 2.0 * ii;
        let denom1 = two_i + ab;
        // aₙ = (β² - α²) / ((2n+α+β)(2n+α+β+2))
        a_diag[i] = (beta * beta - alpha * alpha) / (denom1 * (denom1 + 2.0));
        // bₙ = 4n(n+α)(n+β)(n+α+β) / ((2n+α+β)²(2n+α+β+1)(2n+α+β-1))
        let num = 4.0 * ii * (ii + alpha) * (ii + beta) * (ii + ab);
        let d = denom1 * denom1;
        b_sub[i] = num / (d * (denom1 + 1.0) * (denom1 - 1.0));
    }

    // Build symmetric tridiagonal matrix, eigensolve via QR
    let mut diag = a_diag;
    let mut off = vec![0.0; n - 1];
    for i in 0..(n - 1) {
        off[i] = b_sub[i + 1].sqrt();
    }

    let (eigenvalues, eigenvectors) = symmetric_tridiagonal_qr(&mut diag, &mut off, n);

    let x = eigenvalues;
    let w: Vec<Real> = (0..n)
        .map(|i| mu0 * eigenvectors[i][0] * eigenvectors[i][0])
        .collect();

    GaussianQuadrature { x, w }
}

/// Golub-Welsch for general three-term recurrence.
///
/// `alpha[i]` = diagonal coefficients, `beta[i]` where `beta[0]` = μ₀ and
/// `beta[i]` for i>0 are the squared sub-diagonal elements.
fn golub_welsch(alpha: &[Real], beta: &[Real]) -> GaussianQuadrature {
    let n = alpha.len();
    if n == 0 {
        return GaussianQuadrature {
            x: vec![],
            w: vec![],
        };
    }
    if n == 1 {
        return GaussianQuadrature {
            x: vec![alpha[0]],
            w: vec![beta[0]],
        };
    }

    let mut diag = alpha.to_vec();
    let mut off: Vec<Real> = (1..n).map(|i| beta[i].abs().sqrt()).collect();

    let (eigenvalues, eigenvectors) = symmetric_tridiagonal_qr(&mut diag, &mut off, n);

    let mu0 = beta[0];
    let w: Vec<Real> = (0..n)
        .map(|i| mu0 * eigenvectors[i][0] * eigenvectors[i][0])
        .collect();

    GaussianQuadrature {
        x: eigenvalues,
        w,
    }
}

/// Implicit QR algorithm for a symmetric tridiagonal matrix.
///
/// Returns (eigenvalues, eigenvectors) where eigenvectors[i] is the i-th
/// eigenvector.
fn symmetric_tridiagonal_qr(
    diag: &mut [Real],
    off: &mut [Real],
    n: usize,
) -> (Vec<Real>, Vec<Vec<Real>>) {
    // Initialize eigenvector matrix to identity
    let mut z: Vec<Vec<Real>> = (0..n)
        .map(|i| {
            let mut row = vec![0.0; n];
            row[i] = 1.0;
            row
        })
        .collect();

    let max_iter = 100 * n;
    let mut m = n;

    for _iteration in 0..max_iter {
        if m <= 1 {
            break;
        }

        // Find the largest unreduced submatrix
        let mut l = m - 1;
        while l > 0 && off[l - 1].abs() > 1e-15 * (diag[l - 1].abs() + diag[l].abs()) {
            l -= 1;
        }

        if l == m - 1 {
            m -= 1;
            continue;
        }

        // Wilkinson shift
        let d = (diag[m - 2] - diag[m - 1]) / 2.0;
        let mu = if d.abs() < 1e-300 {
            diag[m - 1] - off[m - 2].abs()
        } else {
            diag[m - 1] - off[m - 2] * off[m - 2] / (d + d.signum() * (d * d + off[m - 2] * off[m - 2]).sqrt())
        };

        // QR step with implicit shift
        let mut x = diag[l] - mu;
        let mut y = off[l];

        for k in l..(m - 1) {
            // Givens rotation
            let (c, s) = if x.abs() > y.abs() {
                let t = -y / x;
                let c_ = 1.0 / (1.0 + t * t).sqrt();
                (c_, c_ * t)
            } else if y.abs() > 1e-300 {
                let t = -x / y;
                let s_ = 1.0 / (1.0 + t * t).sqrt();
                (s_ * t, s_)
            } else {
                (1.0, 0.0)
            };

            let w_val = c * x - s * y;
            let d_val = diag[k] - diag[k + 1];
            let z_val = (2.0 * c * off[k] + d_val * s) * s;
            diag[k] -= z_val;
            diag[k + 1] += z_val;
            if k > l {
                off[k - 1] = w_val;
            }
            off[k] = d_val * c * s + (c * c - s * s) * off[k];

            // Update eigenvectors
            for j in 0..n {
                let t0 = z[j][k];
                let t1 = z[j][k + 1];
                z[j][k] = c * t0 - s * t1;
                z[j][k + 1] = s * t0 + c * t1;
            }

            x = off[k];
            if k < m - 2 {
                y = -s * off[k + 1];
                off[k + 1] *= c;
            }
        }
    }

    // Sort eigenvalues and corresponding eigenvectors
    let mut idx: Vec<usize> = (0..n).collect();
    idx.sort_by(|&a, &b| diag[a].partial_cmp(&diag[b]).unwrap_or(std::cmp::Ordering::Equal));

    let eigenvalues: Vec<Real> = idx.iter().map(|&i| diag[i]).collect();
    let eigenvectors: Vec<Vec<Real>> = idx
        .iter()
        .map(|&i| (0..n).map(|j| z[j][i]).collect())
        .collect();

    (eigenvalues, eigenvectors)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_near(a: Real, b: Real, tol: Real) {
        assert!(
            (a - b).abs() < tol,
            "expected {b}, got {a}, diff = {}",
            (a - b).abs()
        );
    }

    #[test]
    fn gauss_legendre_exact_for_polynomials() {
        // ∫_{-1}^{1} x^4 dx = 2/5 — should be exact with order >= 3
        let q = GaussLegendreIntegration::new(5);
        let result = q.integrate(|x| x.powi(4));
        assert_near(result, 0.4, 1e-12);
    }

    #[test]
    fn gauss_legendre_integrate_interval() {
        // ∫_0^1 x^2 dx = 1/3
        let result = GaussLegendreIntegration::integrate(5, |x| x * x, 0.0, 1.0);
        assert_near(result, 1.0 / 3.0, 1e-12);
    }

    #[test]
    fn gauss_legendre_sin() {
        // ∫_0^π sin(x) dx = 2
        let result = GaussLegendreIntegration::integrate(10, |x| x.sin(), 0.0, PI);
        assert_near(result, 2.0, 1e-10);
    }

    #[test]
    fn gauss_chebyshev_first_kind() {
        // ∫_{-1}^{1} 1/√(1-x²) dx = π  (the weight function)
        // The quadrature integrates f(x)/w(x) · w(x) so:
        // ∫ 1 · w(x) dx = Σ wᵢ · 1 = π
        let q = GaussChebyshevIntegration::new(10);
        let sum: Real = q.w().iter().sum();
        assert_near(sum, PI, 1e-12);
    }

    #[test]
    fn gauss_chebyshev_second_kind() {
        // ∫_{-1}^{1} √(1-x²) dx = π/2
        let q = GaussChebyshev2ndIntegration::new(10);
        let sum: Real = q.w().iter().sum();
        assert_near(sum, PI / 2.0, 1e-10);
    }

    #[test]
    fn gauss_hermite_gaussian_integral() {
        // ∫_{-∞}^{∞} e^{-x²} dx = √π
        // Hermite quadrature with weight e^{-x²}: ∫ f(x)·e^{-x²} dx ≈ Σ wᵢ f(xᵢ)
        // For f(x)=1: sum of weights = √π
        let q = GaussHermiteIntegration::new(10);
        let sum: Real = q.w().iter().sum();
        assert_near(sum, PI.sqrt(), 1e-10);
    }

    #[test]
    fn gauss_laguerre_exponential_integral() {
        // ∫_0^∞ e^{-x} dx = 1  (weight function itself)
        // For f(x) = 1, Σ wᵢ = 1
        let q = GaussLaguerreIntegration::new(10, 0.0);
        let sum: Real = q.w().iter().sum();
        assert_near(sum, 1.0, 1e-10);
    }

    #[test]
    fn gauss_laguerre_polynomial() {
        // ∫_0^∞ x^2 e^{-x} dx = Γ(3) = 2
        let q = GaussLaguerreIntegration::new(10, 0.0);
        let result = q.integrate(|x| x * x);
        assert_near(result, 2.0, 1e-10);
    }

    #[test]
    fn gauss_jacobi_reduces_to_legendre() {
        // Jacobi(α=0, β=0) is Legendre
        let q = gauss_jacobi_nodes_weights(5, 0.0, 0.0);
        let result: Real = q.x().iter().zip(q.w().iter()).map(|(&xi, &wi)| wi * xi.powi(4)).sum();
        assert_near(result, 0.4, 1e-12);
    }
}
