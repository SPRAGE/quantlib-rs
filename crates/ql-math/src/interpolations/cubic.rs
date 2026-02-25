//! Generic cubic Hermite interpolation with configurable slope algorithms
//! (translates `ql/math/interpolations/cubicinterpolation.hpp`, local schemes).
//!
//! The C++ `CubicInterpolation` class supports many derivative-approximation
//! algorithms.  The *local* schemes compute `f'(xᵢ)` using only nearby data:
//!
//! - **Parabolic** — weighted average of adjacent secant slopes.
//! - **FritschButland** — weighted harmonic mean giving monotone-preserving cubics.
//! - **Kruger** — harmonic mean with sign check, also monotone-preserving.
//!
//! All three share the same cubic polynomial evaluation once slopes are known.

use ql_core::{errors::Result, Real};

use super::Interpolation1D;

// ── Shared helpers ────────────────────────────────────────────────────────────

/// Binary search: find `i` such that `xs[i] <= x < xs[i+1]`, clamped.
fn locate(xs: &[Real], x: Real) -> usize {
    let n = xs.len();
    if x <= xs[0] {
        return 0;
    }
    if x >= xs[n - 1] {
        return n - 2;
    }
    let mut lo = 0;
    let mut hi = n - 1;
    while hi - lo > 1 {
        let mid = (lo + hi) / 2;
        if xs[mid] <= x {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}

/// Convert slopes (`ts`) + data (`xs`, `ys`) into polynomial coefficients.
///
/// For each interval `[x_i, x_{i+1}]`:
///
///   `f(x) = y_i + dx*(a_i + dx*(b_i + dx*c_i))`
///
/// where `dx = x - x_i`.
fn compute_coefficients(
    xs: &[Real],
    ys: &[Real],
    ts: &[Real],
) -> (Vec<Real>, Vec<Real>, Vec<Real>) {
    let n = xs.len();
    let mut a = Vec::with_capacity(n - 1);
    let mut b = Vec::with_capacity(n - 1);
    let mut c = Vec::with_capacity(n - 1);

    for i in 0..n - 1 {
        let dx = xs[i + 1] - xs[i];
        let s = (ys[i + 1] - ys[i]) / dx;
        a.push(ts[i]);
        b.push((3.0 * s - ts[i + 1] - 2.0 * ts[i]) / dx);
        c.push((ts[i + 1] + ts[i] - 2.0 * s) / (dx * dx));
    }

    (a, b, c)
}

/// Evaluate `y_i + dx*(a_i + dx*(b_i + dx*c_i))`.
fn poly_eval(xs: &[Real], ys: &[Real], a: &[Real], b: &[Real], c: &[Real], x: Real) -> Real {
    let i = locate(xs, x);
    let dx = x - xs[i];
    ys[i] + dx * (a[i] + dx * (b[i] + dx * c[i]))
}

/// Hyman monotonicity correction on boundary slopes.
///
/// Ensures the first/last derivative has the correct sign relative to the
/// adjacent secant and does not exceed `3 |S|` in magnitude (standard Hyman
/// one-sided constraint).
fn hyman_boundary_correction(ts: &mut [Real], s: &[Real]) {
    let n = ts.len();
    // Left boundary: correct vs S[0]
    if ts[0] * s[0] <= 0.0 {
        ts[0] = 0.0;
    } else if ts[0].abs() > 3.0 * s[0].abs() {
        ts[0] = ts[0].signum() * 3.0 * s[0].abs();
    }
    // Right boundary: correct vs S[n-2]
    let last_s = s[n - 2];
    if ts[n - 1] * last_s <= 0.0 {
        ts[n - 1] = 0.0;
    } else if ts[n - 1].abs() > 3.0 * last_s.abs() {
        ts[n - 1] = ts[n - 1].signum() * 3.0 * last_s.abs();
    }
}

// ── FritschButland ────────────────────────────────────────────────────────────

/// Fritsch-Butland cubic interpolation (local, monotone-preserving).
///
/// Uses a weighted harmonic mean of adjacent secant slopes, producing a
/// C¹ interpolation that cannot introduce new extrema.
///
/// Corresponds to `QuantLib::FritschButlandCubic`.
#[derive(Debug, Clone)]
pub struct FritschButlandCubic {
    xs: Vec<Real>,
    ys: Vec<Real>,
    a: Vec<Real>,
    b: Vec<Real>,
    c: Vec<Real>,
}

impl FritschButlandCubic {
    /// Build a Fritsch-Butland cubic interpolation.
    ///
    /// Requires at least 3 points (uses Parabolic-style boundary formulas).
    pub fn new(xs: &[Real], ys: &[Real]) -> Result<Self> {
        let n = xs.len();
        ql_core::ensure!(n >= 3, "FritschButland requires at least 3 points");
        ql_core::ensure!(xs.len() == ys.len(), "xs and ys must have the same length");

        let xs = xs.to_vec();
        let ys = ys.to_vec();

        // Secant slopes and interval widths
        let mut s = Vec::with_capacity(n - 1);
        let mut dx = Vec::with_capacity(n - 1);
        for i in 0..n - 1 {
            dx.push(xs[i + 1] - xs[i]);
            s.push((ys[i + 1] - ys[i]) / dx[i]);
        }

        // Interior slopes: Fritsch-Butland weighted harmonic mean
        let mut ts = vec![0.0; n];
        for i in 1..n - 1 {
            let s_min = s[i - 1].min(s[i]);
            let s_max = s[i - 1].max(s[i]);
            let denom = s_max + 2.0 * s_min;
            if denom.abs() < 1e-30 {
                ts[i] = 0.0;
            } else {
                ts[i] = 3.0 * s_min * s_max / denom;
            }
        }

        // Boundary slopes: Parabolic-style end formulas
        ts[0] = ((2.0 * dx[0] + dx[1]) * s[0] - dx[0] * s[1]) / (dx[0] + dx[1]);
        ts[n - 1] = ((2.0 * dx[n - 2] + dx[n - 3]) * s[n - 2] - dx[n - 2] * s[n - 3])
            / (dx[n - 2] + dx[n - 3]);

        // Hyman monotonicity correction on boundary slopes:
        // clip sign and magnitude to be consistent with the adjacent secant.
        hyman_boundary_correction(&mut ts, &s);

        let (a, b, c) = compute_coefficients(&xs, &ys, &ts);
        Ok(Self { xs, ys, a, b, c })
    }
}

impl Interpolation1D for FritschButlandCubic {
    fn x_min(&self) -> Real {
        self.xs[0]
    }

    fn x_max(&self) -> Real {
        *self.xs.last().unwrap()
    }

    fn operator(&self, x: Real) -> Real {
        poly_eval(&self.xs, &self.ys, &self.a, &self.b, &self.c, x)
    }
}

// ── Kruger ────────────────────────────────────────────────────────────────────

/// Kruger cubic interpolation (local, monotone-preserving).
///
/// Uses the harmonic mean of adjacent secant slopes with a sign check:
/// if the secants have opposite signs the derivative is set to zero.
///
/// Corresponds to `QuantLib::KrugerCubic`.
#[derive(Debug, Clone)]
pub struct KrugerCubic {
    xs: Vec<Real>,
    ys: Vec<Real>,
    a: Vec<Real>,
    b: Vec<Real>,
    c: Vec<Real>,
}

impl KrugerCubic {
    /// Build a Kruger cubic interpolation.
    ///
    /// Requires at least 3 points.
    pub fn new(xs: &[Real], ys: &[Real]) -> Result<Self> {
        let n = xs.len();
        ql_core::ensure!(n >= 3, "Kruger requires at least 3 points");
        ql_core::ensure!(xs.len() == ys.len(), "xs and ys must have the same length");

        let xs = xs.to_vec();
        let ys = ys.to_vec();

        // Secant slopes
        let mut s = Vec::with_capacity(n - 1);
        for i in 0..n - 1 {
            s.push((ys[i + 1] - ys[i]) / (xs[i + 1] - xs[i]));
        }

        // Interior slopes: harmonic mean with sign check
        let mut ts = vec![0.0; n];
        for i in 1..n - 1 {
            if s[i - 1] * s[i] < 0.0 {
                // Opposite signs → zero slope (monotonicity requirement)
                ts[i] = 0.0;
            } else {
                ts[i] = 2.0 / (1.0 / s[i - 1] + 1.0 / s[i]);
            }
        }

        // Boundary slopes: Kruger end formulas
        ts[0] = (3.0 * s[0] - ts[1]) / 2.0;
        ts[n - 1] = (3.0 * s[n - 2] - ts[n - 2]) / 2.0;

        let (a, b, c) = compute_coefficients(&xs, &ys, &ts);
        Ok(Self { xs, ys, a, b, c })
    }
}

impl Interpolation1D for KrugerCubic {
    fn x_min(&self) -> Real {
        self.xs[0]
    }

    fn x_max(&self) -> Real {
        *self.xs.last().unwrap()
    }

    fn operator(&self, x: Real) -> Real {
        poly_eval(&self.xs, &self.ys, &self.a, &self.b, &self.c, x)
    }
}

// ── Parabolic ─────────────────────────────────────────────────────────────────

/// Parabolic cubic interpolation (local, non-monotonic).
///
/// Uses a distance-weighted average of adjacent secant slopes. Produces a
/// smooth C¹ interpolation but does not guarantee monotonicity.
///
/// Corresponds to `QuantLib::Parabolic`.
#[derive(Debug, Clone)]
pub struct ParabolicCubic {
    xs: Vec<Real>,
    ys: Vec<Real>,
    a: Vec<Real>,
    b: Vec<Real>,
    c: Vec<Real>,
}

impl ParabolicCubic {
    /// Build a Parabolic cubic interpolation.
    ///
    /// Requires at least 3 points.
    pub fn new(xs: &[Real], ys: &[Real]) -> Result<Self> {
        let n = xs.len();
        ql_core::ensure!(n >= 3, "Parabolic requires at least 3 points");
        ql_core::ensure!(xs.len() == ys.len(), "xs and ys must have the same length");

        let xs = xs.to_vec();
        let ys = ys.to_vec();

        // Secant slopes and interval widths
        let mut s = Vec::with_capacity(n - 1);
        let mut dx = Vec::with_capacity(n - 1);
        for i in 0..n - 1 {
            dx.push(xs[i + 1] - xs[i]);
            s.push((ys[i + 1] - ys[i]) / dx[i]);
        }

        // Interior slopes: distance-weighted average of adjacent secants
        let mut ts = vec![0.0; n];
        for i in 1..n - 1 {
            ts[i] = (dx[i - 1] * s[i] + dx[i] * s[i - 1]) / (dx[i - 1] + dx[i]);
        }

        // Boundary slopes: Parabolic end formulas
        ts[0] = ((2.0 * dx[0] + dx[1]) * s[0] - dx[0] * s[1]) / (dx[0] + dx[1]);
        ts[n - 1] = ((2.0 * dx[n - 2] + dx[n - 3]) * s[n - 2] - dx[n - 2] * s[n - 3])
            / (dx[n - 2] + dx[n - 3]);

        let (a, b, c) = compute_coefficients(&xs, &ys, &ts);
        Ok(Self { xs, ys, a, b, c })
    }
}

impl Interpolation1D for ParabolicCubic {
    fn x_min(&self) -> Real {
        self.xs[0]
    }

    fn x_max(&self) -> Real {
        *self.xs.last().unwrap()
    }

    fn operator(&self, x: Real) -> Real {
        poly_eval(&self.xs, &self.ys, &self.a, &self.b, &self.c, x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── FritschButland ────────────────────────────────────────────────────────

    #[test]
    fn fritsch_butland_passes_through_nodes() {
        let xs = [0.0, 1.0, 2.0, 3.0, 4.0];
        let ys = [0.0, 1.0, 0.5, 2.0, 1.5];
        let f = FritschButlandCubic::new(&xs, &ys).unwrap();
        for (&x, &y) in xs.iter().zip(ys.iter()) {
            let v = f.operator(x);
            assert!((v - y).abs() < 1e-12, "at x={x}: expected {y}, got {v}");
        }
    }

    #[test]
    fn fritsch_butland_monotone_data() {
        // From C++ test-suite/interpolations.cpp testFritschButland
        let xs = [0.0, 1.0, 2.0, 3.0, 4.0];
        let ys = [1.0, 2.0, 1.0, 1.0, 2.0];
        let f = FritschButlandCubic::new(&xs, &ys).unwrap();

        // On [2,3] where y=1→1 (flat), interpolant should stay near 1
        for j in 0..=10 {
            let x = 2.0 + (j as f64) / 10.0;
            let v = f.operator(x);
            // Should be close to 1 on flat segment, allow small overshoot
            assert!((v - 1.0).abs() < 0.25, "at x={x}: expected ~1.0, got {v}");
        }
    }

    #[test]
    fn fritsch_butland_preserves_monotonicity() {
        // Monotone increasing data
        let xs = [0.0, 1.0, 2.0, 3.0, 4.0];
        let ys = [0.0, 0.1, 0.5, 2.0, 4.0];
        let f = FritschButlandCubic::new(&xs, &ys).unwrap();
        let mut prev = -1e30;
        for i in 0..=100 {
            let x = 4.0 * (i as f64) / 100.0;
            let v = f.operator(x);
            assert!(v >= prev - 1e-12, "not monotone at x={x}: {v} < {prev}");
            prev = v;
        }
    }

    // ── Kruger ────────────────────────────────────────────────────────────────

    #[test]
    fn kruger_passes_through_nodes() {
        let xs = [0.0, 1.0, 2.0, 3.0, 4.0];
        let ys = [0.0, 1.0, 0.5, 2.0, 1.5];
        let f = KrugerCubic::new(&xs, &ys).unwrap();
        for (&x, &y) in xs.iter().zip(ys.iter()) {
            let v = f.operator(x);
            assert!((v - y).abs() < 1e-12, "at x={x}: expected {y}, got {v}");
        }
    }

    #[test]
    fn kruger_monotone_section() {
        // On monotone data, Kruger should not introduce new extrema
        let xs = [0.0, 1.0, 2.0, 3.0, 4.0];
        let ys = [0.0, 0.5, 1.0, 1.5, 2.0]; // linear
        let f = KrugerCubic::new(&xs, &ys).unwrap();
        let mut prev = -1e30;
        for i in 0..=100 {
            let x = 4.0 * (i as f64) / 100.0;
            let v = f.operator(x);
            assert!(v >= prev - 1e-12, "not monotone at x={x}: {v} < {prev}");
            prev = v;
        }
    }

    #[test]
    fn kruger_reproduces_linear() {
        // Linear data → should reproduce exactly
        let xs = [0.0, 1.0, 2.0, 3.0, 4.0];
        let ys = [1.0, 2.0, 3.0, 4.0, 5.0];
        let f = KrugerCubic::new(&xs, &ys).unwrap();
        for i in 0..=40 {
            let x = 4.0 * (i as f64) / 40.0;
            let expected = 1.0 + x;
            let v = f.operator(x);
            assert!(
                (v - expected).abs() < 1e-10,
                "at x={x}: expected {expected}, got {v}"
            );
        }
    }

    // ── Parabolic ─────────────────────────────────────────────────────────────

    #[test]
    fn parabolic_passes_through_nodes() {
        let xs = [0.0, 1.0, 2.0, 3.0, 4.0];
        let ys = [0.0, 1.0, 0.5, 2.0, 1.5];
        let f = ParabolicCubic::new(&xs, &ys).unwrap();
        for (&x, &y) in xs.iter().zip(ys.iter()) {
            let v = f.operator(x);
            assert!((v - y).abs() < 1e-12, "at x={x}: expected {y}, got {v}");
        }
    }

    #[test]
    fn parabolic_reproduces_linear() {
        // Linear data → parabolic slopes = slope = 1, should reproduce exactly
        let xs = [0.0, 1.0, 2.0, 3.0, 4.0];
        let ys = [0.0, 1.0, 2.0, 3.0, 4.0];
        let f = ParabolicCubic::new(&xs, &ys).unwrap();
        for i in 0..=40 {
            let x = 4.0 * (i as f64) / 40.0;
            let v = f.operator(x);
            assert!((v - x).abs() < 1e-10, "at x={x}: expected {x}, got {v}");
        }
    }

    #[test]
    fn parabolic_smooth_quadratic() {
        // On dense enough x² data, parabolic should give a good approximation
        let xs: Vec<f64> = (0..=10).map(|i| i as f64).collect();
        let ys: Vec<f64> = xs.iter().map(|&x| x * x).collect();
        let f = ParabolicCubic::new(&xs, &ys).unwrap();
        // Check at midpoints
        for i in 1..10 {
            let x = i as f64 + 0.5;
            let expected = x * x;
            let v = f.operator(x);
            assert!(
                (v - expected).abs() < 0.5,
                "at x={x}: expected {expected}, got {v}"
            );
        }
    }
}
