//! Akima spline interpolation (translates `ql/math/interpolations/cubicinterpolation.hpp`,
//! Akima variant).
//!
//! Akima splines avoid the overshooting problems of natural cubic splines by
//! using a weighted average of neighbouring slopes to determine the tangent at
//! each point.

use ql_core::{errors::Result, Real};

use super::Interpolation1D;

/// Akima spline interpolation.
///
/// Corresponds to the Akima variant of `QuantLib::CubicInterpolation`.
#[derive(Debug, Clone)]
pub struct AkimaSpline {
    xs: Vec<Real>,
    ys: Vec<Real>,
    /// Hermite slopes at each knot
    ts: Vec<Real>,
}

impl AkimaSpline {
    /// Build an Akima spline through the given data points.
    ///
    /// Requires at least 5 data points for the full Akima formula; with fewer
    /// points it falls back to simpler slope estimation.
    pub fn new(xs: &[Real], ys: &[Real]) -> Result<Self> {
        let n = xs.len();
        ql_core::ensure!(n >= 2, "Akima spline requires at least 2 points");
        ql_core::ensure!(xs.len() == ys.len(), "xs and ys lengths must match");
        let xs = xs.to_vec();
        let ys = ys.to_vec();

        // Compute slopes between consecutive points
        let mut m = Vec::with_capacity(n + 3);
        for i in 0..n - 1 {
            m.push((ys[i + 1] - ys[i]) / (xs[i + 1] - xs[i]));
        }
        let nm = m.len(); // n-1

        // Extend with phantom slopes for boundary treatment
        // Use Akima's original boundary extension: linear extrapolation of slopes
        let m_neg2 = if nm >= 2 {
            3.0 * m[0] - 2.0 * m[1]
        } else {
            m[0]
        };
        let m_neg1 = if nm >= 2 { 2.0 * m[0] - m[1] } else { m[0] };
        let m_np1 = if nm >= 2 {
            2.0 * m[nm - 1] - m[nm - 2]
        } else {
            m[nm - 1]
        };
        let m_np2 = if nm >= 2 {
            3.0 * m[nm - 1] - 2.0 * m[nm - 2]
        } else {
            m[nm - 1]
        };

        // Build extended slope array: indices -2, -1, 0, ..., n-2, n-1, n
        let mut me = Vec::with_capacity(nm + 4);
        me.push(m_neg2);
        me.push(m_neg1);
        me.extend_from_slice(&m);
        me.push(m_np1);
        me.push(m_np2);

        // Compute Akima tangents
        let mut ts = Vec::with_capacity(n);
        for i in 0..n {
            let idx = i + 2; // offset into me
            let w1 = (me[idx + 1] - me[idx]).abs();
            let w2 = (me[idx - 1] - me[idx - 2]).abs();
            if (w1 + w2).abs() < 1e-30 {
                // Equal weights — average
                ts.push(0.5 * (me[idx - 1] + me[idx]));
            } else {
                ts.push((w1 * me[idx - 1] + w2 * me[idx]) / (w1 + w2));
            }
        }

        Ok(Self { xs, ys, ts })
    }
}

impl Interpolation1D for AkimaSpline {
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

        // Binary search for interval
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
    fn akima_exact_on_nodes() {
        let xs = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
        let ys = [0.0, 1.0, 0.5, 2.0, 1.5, 3.0];
        let s = AkimaSpline::new(&xs, &ys).unwrap();
        for (&x, &y) in xs.iter().zip(ys.iter()) {
            let v = s.operator(x);
            assert!((v - y).abs() < 1e-12, "at x={x}: expected {y}, got {v}");
        }
    }

    #[test]
    fn akima_monotone_section() {
        // On a monotone section the Akima spline should not overshoot
        let xs = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
        let ys = [0.0, 0.5, 1.0, 1.5, 2.0, 2.5]; // purely linear
        let s = AkimaSpline::new(&xs, &ys).unwrap();
        let v = s.operator(2.5);
        assert!((v - 1.25).abs() < 1e-10, "expected 1.25, got {v}");
    }
}
