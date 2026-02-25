//! Chebyshev interpolation on Chebyshev nodes
//! (translates `ql/math/interpolations/chebyshevinterpolation.hpp`).
//!
//! Uses Lagrange interpolation (barycentric formula) at either first-kind
//! or second-kind Chebyshev nodes on `[−1, 1]`.  Chebyshev nodes minimise
//! the Runge phenomenon and provide near-optimal polynomial approximation.

use std::f64::consts::PI;

use ql_core::{errors::Result, Real};

use super::Interpolation1D;

/// Which Chebyshev node set to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChebyshevPointsType {
    /// Roots of `T_n(x)`: `x_i = −cos((i + ½)π / n)`.
    FirstKind,
    /// Extrema of `T_{n−1}(x)`: `x_i = −cos(iπ / (n − 1))`.
    SecondKind,
}

/// Chebyshev interpolation on `[−1, 1]`.
///
/// Corresponds to `QuantLib::ChebyshevInterpolation`.
#[derive(Debug, Clone)]
pub struct ChebyshevInterpolation {
    xs: Vec<Real>,
    ys: Vec<Real>,
    /// Barycentric weights
    weights: Vec<Real>,
}

impl ChebyshevInterpolation {
    /// Build a Chebyshev interpolation from `n` pre-evaluated function values
    /// at the specified Chebyshev nodes.
    ///
    /// `ys[i]` = f(node_i) where node_i is the i-th Chebyshev node.
    pub fn new(ys: &[Real], points_type: ChebyshevPointsType) -> Result<Self> {
        let n = ys.len();
        ql_core::ensure!(n >= 2, "Chebyshev interpolation requires at least 2 points");
        let xs = chebyshev_nodes(n, points_type);
        let weights = barycentric_weights(&xs);
        Ok(Self {
            xs,
            ys: ys.to_vec(),
            weights,
        })
    }

    /// Build a Chebyshev interpolation by evaluating `f` at `n` Chebyshev nodes.
    pub fn from_function(
        n: usize,
        f: &dyn Fn(Real) -> Real,
        points_type: ChebyshevPointsType,
    ) -> Result<Self> {
        ql_core::ensure!(n >= 2, "Chebyshev interpolation requires at least 2 points");
        let xs = chebyshev_nodes(n, points_type);
        let ys: Vec<Real> = xs.iter().map(|&x| f(x)).collect();
        let weights = barycentric_weights(&xs);
        Ok(Self { xs, ys, weights })
    }

    /// Return the Chebyshev nodes used by this interpolation.
    pub fn nodes(&self) -> &[Real] {
        &self.xs
    }
}

/// Compute the `n` Chebyshev nodes of the given type on `[−1, 1]`.
pub fn chebyshev_nodes(n: usize, points_type: ChebyshevPointsType) -> Vec<Real> {
    let mut t = Vec::with_capacity(n);
    match points_type {
        ChebyshevPointsType::FirstKind => {
            for i in 0..n {
                t.push(-((i as f64 + 0.5) * PI / n as f64).cos());
            }
        }
        ChebyshevPointsType::SecondKind => {
            for i in 0..n {
                t.push(-(i as f64 * PI / (n - 1) as f64).cos());
            }
        }
    }
    t
}

/// Compute barycentric weights for the given nodes.
fn barycentric_weights(xs: &[Real]) -> Vec<Real> {
    let n = xs.len();
    let mut weights = vec![1.0; n];
    for j in 0..n {
        for k in 0..n {
            if k != j {
                weights[j] /= xs[j] - xs[k];
            }
        }
    }
    weights
}

impl Interpolation1D for ChebyshevInterpolation {
    fn x_min(&self) -> Real {
        // Chebyshev nodes are on [-1, 1]; first node is the smallest
        self.xs.iter().copied().reduce(f64::min).unwrap_or(-1.0)
    }

    fn x_max(&self) -> Real {
        self.xs.iter().copied().reduce(f64::max).unwrap_or(1.0)
    }

    fn operator(&self, x: Real) -> Real {
        // Check if x coincides with a node
        for (i, &xi) in self.xs.iter().enumerate() {
            if (x - xi).abs() < f64::EPSILON * (1.0 + x.abs()) {
                return self.ys[i];
            }
        }
        // Barycentric formula: f(x) = [Σ w_j y_j / (x − x_j)] / [Σ w_j / (x − x_j)]
        let mut numer = 0.0;
        let mut denom = 0.0;
        for j in 0..self.xs.len() {
            let t = self.weights[j] / (x - self.xs[j]);
            numer += t * self.ys[j];
            denom += t;
        }
        numer / denom
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chebyshev_nodes_second_kind() {
        let nodes = chebyshev_nodes(5, ChebyshevPointsType::SecondKind);
        assert_eq!(nodes.len(), 5);
        // First node = -cos(0) = -1
        assert!((nodes[0] - (-1.0)).abs() < 1e-12);
        // Last node = -cos(π) = 1
        assert!((nodes[4] - 1.0).abs() < 1e-12);
        // Middle node = -cos(π/2) = 0
        assert!(nodes[2].abs() < 1e-12);
    }

    #[test]
    fn chebyshev_nodes_first_kind() {
        let nodes = chebyshev_nodes(4, ChebyshevPointsType::FirstKind);
        assert_eq!(nodes.len(), 4);
        // All nodes should be in (-1, 1)
        for &x in &nodes {
            assert!(x > -1.0 && x < 1.0);
        }
    }

    #[test]
    fn chebyshev_approximates_cos() {
        // cos(x) on [-1, 1] should be well-approximated by Chebyshev
        let f = |x: Real| x.cos();
        let interp =
            ChebyshevInterpolation::from_function(10, &f, ChebyshevPointsType::SecondKind).unwrap();
        for i in 0..=20 {
            let x = -1.0 + 2.0 * (i as f64) / 20.0;
            let expected = x.cos();
            let v = interp.operator(x);
            assert!(
                (v - expected).abs() < 1e-8,
                "at x={x}: expected {expected}, got {v}"
            );
        }
    }

    #[test]
    fn chebyshev_from_values() {
        // Build from pre-evaluated values at second-kind nodes
        let n = 8;
        let nodes = chebyshev_nodes(n, ChebyshevPointsType::SecondKind);
        let ys: Vec<Real> = nodes.iter().map(|&x| x * x).collect(); // x²
        let interp = ChebyshevInterpolation::new(&ys, ChebyshevPointsType::SecondKind).unwrap();
        // Should reproduce x² very well
        let v = interp.operator(0.5);
        assert!((v - 0.25).abs() < 1e-10, "expected 0.25, got {v}");
    }

    #[test]
    fn chebyshev_first_kind_polynomial() {
        // Degree-3 polynomial should be exactly reproduced with 4+ nodes
        let f = |x: Real| x * x * x - 2.0 * x + 1.0;
        let interp =
            ChebyshevInterpolation::from_function(5, &f, ChebyshevPointsType::FirstKind).unwrap();
        for i in 0..=10 {
            let x = -1.0 + 2.0 * (i as f64) / 10.0;
            let expected = f(x);
            let v = interp.operator(x);
            assert!(
                (v - expected).abs() < 1e-10,
                "at x={x}: expected {expected}, got {v}"
            );
        }
    }
}
