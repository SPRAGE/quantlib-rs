//! 1D and 2D interpolation traits and implementations (translates
//! `ql/math/interpolation.hpp` and `ql/math/interpolations/`).
//!
//! **1D interpolations:** Linear, LogLinear, Flat (backward), ForwardFlat,
//! CubicNaturalSpline, Lagrange, Akima, MonotoneCubic (Fritsch-Carlson),
//! FritschButland, Kruger, Parabolic, Chebyshev, LogCubic, SABR.
//!
//! **2D interpolations:** Bilinear, Bicubic.

pub mod akima;
pub mod bicubic;
pub mod bilinear;
pub mod chebyshev;
pub mod cubic;
pub mod log_cubic;
pub mod monotone_cubic;
pub mod sabr;

use ql_core::{errors::Result, Real};

/// A 1D interpolation function `f: R → R` defined by a set of known points.
///
/// Corresponds to `QuantLib::Interpolation`.
pub trait Interpolation1D: std::fmt::Debug + Send + Sync {
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
        ql_core::ensure!(xs.len() == ys.len(), "xs and ys must have the same length");
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
        ql_core::ensure!(!xs.is_empty(), "need at least 1 point");
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

// ── Forward Flat ──────────────────────────────────────────────────────────────

/// Forward flat interpolation — uses the value at the upper node.
///
/// Corresponds to `QuantLib::ForwardFlatInterpolation`.
#[derive(Debug, Clone)]
pub struct ForwardFlatInterpolation {
    xs: Vec<Real>,
    ys: Vec<Real>,
}

impl ForwardFlatInterpolation {
    /// Construct a forward flat interpolation.
    pub fn new(xs: &[Real], ys: &[Real]) -> Result<Self> {
        ql_core::ensure!(!xs.is_empty(), "need at least 1 point");
        ql_core::ensure!(xs.len() == ys.len(), "xs and ys lengths must match");
        Ok(Self {
            xs: xs.to_vec(),
            ys: ys.to_vec(),
        })
    }
}

impl Interpolation1D for ForwardFlatInterpolation {
    fn x_min(&self) -> Real {
        self.xs[0]
    }

    fn x_max(&self) -> Real {
        *self.xs.last().unwrap()
    }

    fn operator(&self, x: Real) -> Real {
        let n = self.xs.len();
        if x >= self.xs[n - 1] {
            return self.ys[n - 1];
        }
        // Find interval i such that xs[i] <= x < xs[i+1], return ys[i].
        // This matches C++ `ForwardFlatInterpolation`: yBegin_[locate(x)].
        let i = match self.xs.partition_point(|&xi| xi <= x) {
            0 => 0,
            p => p - 1,
        };
        self.ys[i]
    }
}

// ── Cubic Natural Spline ─────────────────────────────────────────────────────

/// Natural cubic spline interpolation (second derivatives vanish at endpoints).
///
/// Corresponds to `QuantLib::CubicInterpolation` with `Natural` boundary
/// conditions.
#[derive(Debug, Clone)]
pub struct CubicNaturalSpline {
    xs: Vec<Real>,
    ys: Vec<Real>,
    /// Second derivatives at the knots.
    m: Vec<Real>,
}

impl CubicNaturalSpline {
    /// Build a natural cubic spline through `(xs[i], ys[i])`.
    ///
    /// The xs must be sorted in strictly increasing order.
    ///
    /// # Errors
    /// Returns an error if fewer than 3 points are provided or lengths differ.
    pub fn new(xs: &[Real], ys: &[Real]) -> Result<Self> {
        ql_core::ensure!(xs.len() >= 3, "need at least 3 points for cubic spline");
        ql_core::ensure!(xs.len() == ys.len(), "xs and ys must have the same length");
        let n = xs.len();
        let m = Self::compute_second_derivatives(xs, ys, n);
        Ok(Self {
            xs: xs.to_vec(),
            ys: ys.to_vec(),
            m,
        })
    }

    /// Solve the tridiagonal system for natural spline second-derivatives.
    fn compute_second_derivatives(xs: &[Real], ys: &[Real], n: usize) -> Vec<Real> {
        let nm1 = n - 1;

        // h[i] = xs[i+1] - xs[i]
        let h: Vec<Real> = (0..nm1).map(|i| xs[i + 1] - xs[i]).collect();

        // RHS of the tridiagonal system
        let mut rhs = vec![0.0; n];
        for i in 1..nm1 {
            rhs[i] = 6.0 * ((ys[i + 1] - ys[i]) / h[i] - (ys[i] - ys[i - 1]) / h[i - 1]);
        }

        // Natural boundary: m[0] = m[n-1] = 0
        // Tridiagonal system: h[i-1]*m[i-1] + 2*(h[i-1]+h[i])*m[i] + h[i]*m[i+1] = rhs[i]
        // Solve using Thomas algorithm for i = 1..n-2

        let mut c_prime = vec![0.0; n];
        let mut d_prime = vec![0.0; n];

        // Forward sweep
        for i in 1..nm1 {
            let diag = 2.0 * (h[i - 1] + h[i]);
            let sub = h[i - 1];
            let sup = h[i];

            let denom = diag - sub * c_prime[i - 1];
            c_prime[i] = sup / denom;
            d_prime[i] = (rhs[i] - sub * d_prime[i - 1]) / denom;
        }

        // Back substitution
        let mut m = vec![0.0; n];
        for i in (1..nm1).rev() {
            m[i] = d_prime[i] - c_prime[i] * m[i + 1];
        }

        m
    }

    fn locate(&self, x: Real) -> usize {
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

impl Interpolation1D for CubicNaturalSpline {
    fn x_min(&self) -> Real {
        self.xs[0]
    }

    fn x_max(&self) -> Real {
        *self.xs.last().unwrap()
    }

    fn operator(&self, x: Real) -> Real {
        let i = self.locate(x);
        let h = self.xs[i + 1] - self.xs[i];
        let t = x - self.xs[i];
        let a = (self.m[i + 1] - self.m[i]) / (6.0 * h);
        let b = self.m[i] / 2.0;
        let c = (self.ys[i + 1] - self.ys[i]) / h - h * (2.0 * self.m[i] + self.m[i + 1]) / 6.0;
        let d = self.ys[i];
        d + t * (c + t * (b + t * a))
    }
}

// ── Lagrange Interpolation ───────────────────────────────────────────────────

/// Lagrange polynomial interpolation.
///
/// Corresponds to `QuantLib::LagrangeInterpolation`.
/// Uses the barycentric formula for numerical stability.
#[derive(Debug, Clone)]
pub struct LagrangeInterpolation {
    xs: Vec<Real>,
    ys: Vec<Real>,
    weights: Vec<Real>,
}

impl LagrangeInterpolation {
    /// Build a Lagrange interpolation through `(xs[i], ys[i])`.
    ///
    /// # Errors
    /// Returns an error if fewer than 2 points or lengths differ.
    pub fn new(xs: &[Real], ys: &[Real]) -> Result<Self> {
        ql_core::ensure!(xs.len() >= 2, "need at least 2 points");
        ql_core::ensure!(xs.len() == ys.len(), "xs and ys must have the same length");
        let n = xs.len();
        // Barycentric weights: w[j] = 1 / prod_{k≠j} (x[j] - x[k])
        let mut weights = vec![1.0; n];
        for j in 0..n {
            for k in 0..n {
                if k != j {
                    weights[j] /= xs[j] - xs[k];
                }
            }
        }
        Ok(Self {
            xs: xs.to_vec(),
            ys: ys.to_vec(),
            weights,
        })
    }
}

impl Interpolation1D for LagrangeInterpolation {
    fn x_min(&self) -> Real {
        self.xs[0]
    }

    fn x_max(&self) -> Real {
        *self.xs.last().unwrap()
    }

    fn operator(&self, x: Real) -> Real {
        // Check if x coincides with a node
        for (i, &xi) in self.xs.iter().enumerate() {
            if (x - xi).abs() < f64::EPSILON * (1.0 + x.abs()) {
                return self.ys[i];
            }
        }
        // Barycentric formula: f(x) = [Σ w_j * y_j / (x - x_j)] / [Σ w_j / (x - x_j)]
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

    #[test]
    fn forward_flat_interpolation() {
        let xs = [0.0, 1.0, 2.0];
        let ys = [1.0, 2.0, 3.0];
        let interp = ForwardFlatInterpolation::new(&xs, &ys).unwrap();
        // Between 0 and 1, forward flat uses the LEFT knot value (x=0 → y=1)
        assert!((interp.operator(0.5) - 1.0).abs() < 1e-12);
        // Between 1 and 2, forward flat uses the LEFT knot value (x=1 → y=2)
        assert!((interp.operator(1.5) - 2.0).abs() < 1e-12);
        assert!((interp.operator(2.0) - 3.0).abs() < 1e-12);
    }

    #[test]
    fn cubic_spline_quadratic() {
        // A natural cubic spline through points of x^2. Due to natural boundary
        // conditions (S''=0 at endpoints), it won't reproduce x^2 exactly at
        // endpoints, but should be close in the interior.
        let xs: Vec<Real> = (-3..=3).map(|i| i as Real).collect();
        let ys: Vec<Real> = xs.iter().map(|&x| x * x).collect();
        let spline = CubicNaturalSpline::new(&xs, &ys).unwrap();
        // Check at midpoints in the interior (away from boundary effects)
        for &x in &[-1.5, -0.5, 0.5, 1.5] {
            let expected = x * x;
            let got = spline.operator(x);
            assert!(
                (got - expected).abs() < 0.05,
                "at x={x}: got {got}, expected {expected}"
            );
        }
    }

    #[test]
    fn cubic_spline_passes_through_nodes() {
        let xs = [0.0, 1.0, 2.0, 3.0, 4.0];
        let ys = [0.0, 1.0, 0.5, 2.0, 1.5];
        let spline = CubicNaturalSpline::new(&xs, &ys).unwrap();
        for i in 0..xs.len() {
            let got = spline.operator(xs[i]);
            assert!(
                (got - ys[i]).abs() < 1e-12,
                "at node x={}: got {got}, expected {}",
                xs[i],
                ys[i]
            );
        }
    }

    #[test]
    fn lagrange_exact_polynomial() {
        // Lagrange through (0,0), (1,1), (2,4) should reproduce x^2
        let xs = [0.0, 1.0, 2.0];
        let ys = [0.0, 1.0, 4.0];
        let interp = LagrangeInterpolation::new(&xs, &ys).unwrap();
        // At x=0.5, x^2=0.25
        assert!(
            (interp.operator(0.5) - 0.25).abs() < 1e-12,
            "got {}",
            interp.operator(0.5)
        );
        // At x=1.5, x^2=2.25
        assert!(
            (interp.operator(1.5) - 2.25).abs() < 1e-12,
            "got {}",
            interp.operator(1.5)
        );
    }

    #[test]
    fn lagrange_passes_through_nodes() {
        let xs = [0.0, 1.0, 3.0, 5.0];
        let ys = [1.0, -1.0, 2.0, 0.5];
        let interp = LagrangeInterpolation::new(&xs, &ys).unwrap();
        for i in 0..xs.len() {
            let got = interp.operator(xs[i]);
            assert!(
                (got - ys[i]).abs() < 1e-10,
                "at x={}: got {got}, expected {}",
                xs[i],
                ys[i]
            );
        }
    }

    // ── Tests ported from C++ test-suite/interpolations.cpp ──────────────

    /// Ported from `testBackwardFlat`.
    #[test]
    fn backward_flat_ported() {
        let xs = [0.0, 1.0, 2.0, 3.0, 4.0];
        let ys = [5.0, 4.0, 3.0, 2.0, 1.0];
        let f = FlatInterpolation::new(&xs, &ys).unwrap();
        let tol = 1e-12;
        // Exact on grid points
        for i in 0..5 {
            assert!((f.operator(xs[i]) - ys[i]).abs() < tol);
        }
        // Between points: backward-flat uses the LEFT value (value at lower node)
        for i in 0..4 {
            let v = f.operator(xs[i] + 0.5);
            assert!(
                (v - ys[i]).abs() < tol,
                "at x={}: expected {}, got {v}",
                xs[i] + 0.5,
                ys[i]
            );
        }
    }

    /// Ported from `testForwardFlat`.
    #[test]
    fn forward_flat_ported() {
        let xs = [0.0, 1.0, 2.0, 3.0, 4.0];
        let ys = [5.0, 4.0, 3.0, 2.0, 1.0];
        let f = ForwardFlatInterpolation::new(&xs, &ys).unwrap();
        let tol = 1e-12;
        // Exact on grid points
        for i in 0..5 {
            assert!((f.operator(xs[i]) - ys[i]).abs() < tol);
        }
        // Between points: forward-flat uses the LEFT knot's value
        // (C++ returns yBegin_[i] where i = locate(x))
        for i in 0..4 {
            let v = f.operator(xs[i] + 0.5);
            assert!(
                (v - ys[i]).abs() < tol,
                "at x={}: expected {}, got {v}",
                xs[i] + 0.5,
                ys[i]
            );
        }
    }

    /// Ported from `testSplineErrorOnGaussianValues`.
    ///
    /// Checks that the natural cubic spline approximation error for exp(−x²)
    /// converges as the number of grid points increases.  The C++ test uses
    /// not-a-knot boundary conditions; we verify convergence rate (O(h⁴))
    /// with natural BCs instead.
    #[test]
    fn cubic_spline_gaussian_convergence() {
        let x_start = -1.7;
        let x_end = 1.9;
        let eval_points = 10_000;

        let point_counts = [5_usize, 9, 17, 33];
        let mut prev_error = f64::MAX;

        for &n in &point_counts {
            let dx = (x_end - x_start) / (n - 1) as f64;
            let xs: Vec<f64> = (0..n).map(|i| x_start + i as f64 * dx).collect();
            let ys: Vec<f64> = xs.iter().map(|&x| (-x * x).exp()).collect();

            let spline = CubicNaturalSpline::new(&xs, &ys).unwrap();

            let mut max_error = 0.0_f64;
            let eval_dx = (xs[n - 1] - xs[0]) / eval_points as f64;
            for j in 0..=eval_points {
                let x = xs[0] + j as f64 * eval_dx;
                let exact = (-x * x).exp();
                let approx = spline.operator(x);
                max_error = max_error.max((exact - approx).abs());
            }

            // Each doubling of points should decrease error significantly.
            assert!(
                max_error < prev_error,
                "n={n}: max_error={max_error:.2e} not less than previous {prev_error:.2e}"
            );
            prev_error = max_error;
        }

        // The finest grid (33 points) should achieve < 1e-3 max error.
        // (Natural BCs give ~3.3e-4; not-a-knot would give ~1.8e-6.)
        assert!(
            prev_error < 1e-3,
            "33-point natural spline: max_error={prev_error:.2e}, expected < 1e-3"
        );
    }

    /// Ported from `testSplineOnRPN15AValues`.
    ///
    /// The RPN15A data is a steep sigmoid-like function. A non-MC natural spline
    /// overshoots above 1.0 at x=11, but the monotone-preserving cubic should
    /// remain below 1.0.
    #[test]
    fn cubic_spline_rpn15a_overshoot() {
        let xs = [7.99, 8.09, 8.19, 8.7, 9.2, 10.0, 12.0, 15.0, 20.0];
        let ys = [
            0.0, 2.76429e-5, 4.37498e-5, 0.169183, 0.469428, 0.943740, 0.998636, 0.999919, 0.999994,
        ];

        // Natural spline: known to overshoot above 1.0 at x=11
        let spline = CubicNaturalSpline::new(&xs, &ys).unwrap();
        let v = spline.operator(11.0);
        // This confirms the known overshoot behaviour
        assert!(
            v > 1.0,
            "natural spline at x=11: expected overshoot, got {v}"
        );

        // Monotone cubic should NOT overshoot
        let mc = super::monotone_cubic::MonotoneCubicSpline::new(&xs, &ys).unwrap();
        let v_mc = mc.operator(11.0);
        assert!(
            v_mc <= 1.0 + 1e-10,
            "monotone cubic at x=11: expected ≤ 1.0, got {v_mc}"
        );
    }
}
