//! 1D interpolation trait and implementations (translates
//! `ql/math/interpolation.hpp` and `ql/math/interpolations/`).

use ql_core::{errors::Result, Real};

/// A 1D interpolation function `f: R → R` defined by a set of known points.
///
/// Corresponds to `QuantLib::Interpolation`.
pub trait Interpolation1D: std::fmt::Debug {
    /// Evaluate the interpolation at `x`.
    fn operator(&self, x: Real) -> Real;

    /// Return the lower bound of the interpolation domain.
    fn x_min(&self) -> Real;

    /// Return the upper bound of the interpolation domain.
    fn x_max(&self) -> Real;

    /// Return `true` if `x` is within the interpolation range.
    fn is_in_range(&self, x: Real) -> bool {
        x >= self.x_min() && x <= self.x_max()
    }
}

// ── Linear ────────────────────────────────────────────────────────────────────

/// Linear interpolation.
///
/// `f(x) = y[i] + (y[i+1] - y[i]) * (x - x[i]) / (x[i+1] - x[i])`
#[derive(Debug, Clone)]
pub struct LinearInterpolation {
    xs: Vec<Real>,
    ys: Vec<Real>,
}

impl LinearInterpolation {
    /// Construct a linear interpolation from sorted `xs` and corresponding `ys`.
    ///
    /// # Errors
    /// Returns an error if the slices have different lengths or fewer than 2 points.
    pub fn new(xs: &[Real], ys: &[Real]) -> Result<Self> {
        ql_core::ensure!(xs.len() >= 2, "need at least 2 points for interpolation");
        ql_core::ensure!(
            xs.len() == ys.len(),
            "xs and ys must have the same length"
        );
        Ok(Self {
            xs: xs.to_vec(),
            ys: ys.to_vec(),
        })
    }

    fn locate(&self, x: Real) -> usize {
        // Binary search for the interval containing x
        let n = self.xs.len();
        if x <= self.xs[0] {
            return 0;
        }
        if x >= self.xs[n - 1] {
            return n - 2;
        }
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
        lo
    }
}

impl Interpolation1D for LinearInterpolation {
    fn x_min(&self) -> Real {
        self.xs[0]
    }

    fn x_max(&self) -> Real {
        *self.xs.last().unwrap()
    }

    fn operator(&self, x: Real) -> Real {
        let i = self.locate(x);
        let dx = self.xs[i + 1] - self.xs[i];
        if dx.abs() < f64::EPSILON {
            return self.ys[i];
        }
        self.ys[i] + (x - self.xs[i]) * (self.ys[i + 1] - self.ys[i]) / dx
    }
}

// ── Log-linear ────────────────────────────────────────────────────────────────

/// Log-linear interpolation.
///
/// Interpolates `log(y)` linearly and exponentiates the result.
#[derive(Debug, Clone)]
pub struct LogLinearInterpolation {
    inner: LinearInterpolation,
}

impl LogLinearInterpolation {
    /// Construct a log-linear interpolation.
    ///
    /// All `ys` values must be strictly positive.
    pub fn new(xs: &[Real], ys: &[Real]) -> Result<Self> {
        ql_core::ensure!(
            ys.iter().all(|&y| y > 0.0),
            "all y values must be positive for log-linear interpolation"
        );
        let log_ys: Vec<Real> = ys.iter().map(|&y| y.ln()).collect();
        Ok(Self {
            inner: LinearInterpolation::new(xs, &log_ys)?,
        })
    }
}

impl Interpolation1D for LogLinearInterpolation {
    fn x_min(&self) -> Real {
        self.inner.x_min()
    }

    fn x_max(&self) -> Real {
        self.inner.x_max()
    }

    fn operator(&self, x: Real) -> Real {
        self.inner.operator(x).exp()
    }
}

// ── Flat (constant / nearest-neighbour) ──────────────────────────────────────

/// Flat (step-function) interpolation — uses the value at the lower node.
#[derive(Debug, Clone)]
pub struct FlatInterpolation {
    xs: Vec<Real>,
    ys: Vec<Real>,
}

impl FlatInterpolation {
    /// Construct a flat interpolation.
    pub fn new(xs: &[Real], ys: &[Real]) -> Result<Self> {
        ql_core::ensure!(xs.len() >= 1, "need at least 1 point");
        ql_core::ensure!(xs.len() == ys.len(), "xs and ys lengths must match");
        Ok(Self {
            xs: xs.to_vec(),
            ys: ys.to_vec(),
        })
    }
}

impl Interpolation1D for FlatInterpolation {
    fn x_min(&self) -> Real {
        self.xs[0]
    }

    fn x_max(&self) -> Real {
        *self.xs.last().unwrap()
    }

    fn operator(&self, x: Real) -> Real {
        if x <= self.xs[0] {
            return self.ys[0];
        }
        // Return the value at the last node with xs[i] <= x
        let n = self.xs.len();
        for i in (0..n).rev() {
            if self.xs[i] <= x {
                return self.ys[i];
            }
        }
        self.ys[0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_interpolation() {
        let xs = [0.0, 1.0, 2.0];
        let ys = [0.0, 1.0, 4.0];
        let interp = LinearInterpolation::new(&xs, &ys).unwrap();
        assert!((interp.operator(0.5) - 0.5).abs() < 1e-12);
        assert!((interp.operator(1.5) - 2.5).abs() < 1e-12);
    }

    #[test]
    fn log_linear_interpolation() {
        let xs = [0.0, 1.0];
        let ys = [1.0, std::f64::consts::E];
        let interp = LogLinearInterpolation::new(&xs, &ys).unwrap();
        // At x=0.5, log(y)=0.5 → y = e^0.5
        let expected = std::f64::consts::E.sqrt();
        assert!((interp.operator(0.5) - expected).abs() < 1e-12);
    }

    #[test]
    fn flat_interpolation() {
        let xs = [0.0, 1.0, 2.0];
        let ys = [1.0, 2.0, 3.0];
        let interp = FlatInterpolation::new(&xs, &ys).unwrap();
        assert!((interp.operator(0.5) - 1.0).abs() < 1e-12);
        assert!((interp.operator(1.5) - 2.0).abs() < 1e-12);
        assert!((interp.operator(2.0) - 3.0).abs() < 1e-12);
    }
}
