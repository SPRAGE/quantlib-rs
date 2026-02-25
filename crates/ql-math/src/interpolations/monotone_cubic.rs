//! Monotone-preserving cubic Hermite interpolation
//! (translates `ql/math/interpolations/cubicinterpolation.hpp`, monotone variant).
//!
//! Implements the Fritsch-Carlson algorithm that modifies cubic Hermite slopes
//! to guarantee monotonicity on each sub-interval where the data is monotone.

use ql_core::{errors::Result, Real};

use super::Interpolation1D;

/// Monotone-preserving cubic Hermite spline.
///
/// Corresponds to `QuantLib::MonotonicCubicNaturalSpline`.
#[derive(Debug, Clone)]
pub struct MonotoneCubicSpline {
    xs: Vec<Real>,
    ys: Vec<Real>,
    /// Adjusted tangent at each knot
    ts: Vec<Real>,
}

impl MonotoneCubicSpline {
    /// Build a monotone cubic spline through the given data.
    pub fn new(xs: &[Real], ys: &[Real]) -> Result<Self> {
        let n = xs.len();
        ql_core::ensure!(n >= 2, "need at least 2 points");
        ql_core::ensure!(xs.len() == ys.len(), "xs and ys must match in length");

        let xs = xs.to_vec();
        let ys = ys.to_vec();

        // Step 1: compute secant slopes δ_i
        let mut delta = Vec::with_capacity(n - 1);
        for i in 0..n - 1 {
            delta.push((ys[i + 1] - ys[i]) / (xs[i + 1] - xs[i]));
        }

        // Step 2: initial tangent estimates (three-point formula)
        let mut ts = vec![0.0; n];
        ts[0] = delta[0];
        if n > 2 {
            ts[n - 1] = delta[n - 2];
        } else {
            ts[1] = delta[0];
            return Ok(Self { xs, ys, ts });
        }
        for i in 1..n - 1 {
            ts[i] = 0.5 * (delta[i - 1] + delta[i]);
        }

        // Step 3: Fritsch-Carlson monotonicity corrections
        for i in 0..n - 1 {
            if delta[i].abs() < 1e-30 {
                // Flat segment — force both tangents to zero
                ts[i] = 0.0;
                ts[i + 1] = 0.0;
            } else {
                let alpha = ts[i] / delta[i];
                let beta = ts[i + 1] / delta[i];
                // Ensure we're inside the monotone region: α² + β² ≤ 9
                let r2 = alpha * alpha + beta * beta;
                if r2 > 9.0 {
                    let tau = 3.0 / r2.sqrt();
                    ts[i] = tau * alpha * delta[i];
                    ts[i + 1] = tau * beta * delta[i];
                }
            }
        }

        Ok(Self { xs, ys, ts })
    }
}

impl Interpolation1D for MonotoneCubicSpline {
    fn x_min(&self) -> Real {
        self.xs[0]
    }

    fn x_max(&self) -> Real {
        *self.xs.last().unwrap()
    }

    fn operator(&self, x: Real) -> Real {
        let n = self.xs.len();
        if x <= self.xs[0] {
            return self.ys[0];
        }
        if x >= self.xs[n - 1] {
            return self.ys[n - 1];
        }

        // Binary search
        let mut lo = 0;
        let mut hi = n - 1;
        while hi - lo > 1 {
            let mid = (lo + hi) / 2;
            if self.xs[mid] <= x {
                lo = mid;
            } else {
                hi = mid;
            }
        }

        let h = self.xs[hi] - self.xs[lo];
        let t = (x - self.xs[lo]) / h;
        // Hermite basis
        let h00 = (1.0 + 2.0 * t) * (1.0 - t) * (1.0 - t);
        let h10 = t * (1.0 - t) * (1.0 - t);
        let h01 = t * t * (3.0 - 2.0 * t);
        let h11 = t * t * (t - 1.0);

        h00 * self.ys[lo] + h10 * h * self.ts[lo] + h01 * self.ys[hi] + h11 * h * self.ts[hi]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monotone_exact_on_nodes() {
        let xs = [0.0, 1.0, 2.0, 3.0, 4.0];
        let ys = [0.0, 1.0, 1.5, 3.0, 5.0];
        let s = MonotoneCubicSpline::new(&xs, &ys).unwrap();
        for (&x, &y) in xs.iter().zip(ys.iter()) {
            let v = s.operator(x);
            assert!((v - y).abs() < 1e-12, "at x={x}: expected {y}, got {v}");
        }
    }

    #[test]
    fn monotone_preserves_monotonicity() {
        // Monotone increasing data — interpolant should not decrease
        let xs = [0.0, 1.0, 2.0, 3.0, 4.0];
        let ys = [0.0, 0.1, 0.5, 2.0, 4.0];
        let s = MonotoneCubicSpline::new(&xs, &ys).unwrap();
        let mut prev = -1e30;
        for i in 0..=100 {
            let x = 4.0 * (i as f64) / 100.0;
            let v = s.operator(x);
            assert!(v >= prev - 1e-12, "not monotone at x={x}: {v} < {prev}");
            prev = v;
        }
    }

    #[test]
    fn monotone_step_function() {
        // Step: 0,0,1,1 — should stay in [0,1]
        let xs = [0.0, 1.0, 2.0, 3.0];
        let ys = [0.0, 0.0, 1.0, 1.0];
        let s = MonotoneCubicSpline::new(&xs, &ys).unwrap();
        for i in 0..=100 {
            let x = 3.0 * (i as f64) / 100.0;
            let v = s.operator(x);
            assert!(
                (-1e-10..=1.0 + 1e-10).contains(&v),
                "out of range at x={x}: {v}"
            );
        }
    }
}
