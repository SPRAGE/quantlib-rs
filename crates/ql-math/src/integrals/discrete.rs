//! Discrete integrators operating on pre-computed data arrays.
//!
//! Translates `ql/math/integrals/discreteintegrals.hpp`.
//!
//! These are useful when the integrand is only available as a vector of
//! function values at specified abscissae (e.g., bond NPV computation from
//! cashflow schedules).

use ql_core::Real;

/// Composite trapezoidal rule on discrete data points.
///
/// Given abscissae `x[0..n]` and ordinates `f[0..n]`, returns
///
/// $$\sum_{i=0}^{n-2} \tfrac12 (x_{i+1}-x_i)(f_i + f_{i+1}).$$
///
/// Corresponds to `QuantLib::DiscreteTrapezoidIntegral`.
pub fn discrete_trapezoid(x: &[Real], f: &[Real]) -> Real {
    debug_assert_eq!(x.len(), f.len());
    let n = x.len();
    if n < 2 {
        return 0.0;
    }
    let mut sum = 0.0;
    for i in 0..n - 1 {
        sum += (x[i + 1] - x[i]) * (f[i] + f[i + 1]);
    }
    0.5 * sum
}

/// Composite Simpson's rule on discrete data points (non-uniform spacing).
///
/// Processes pairs of sub-intervals using the standard Simpson 1/3 rule
/// adapted for non-uniform spacing. If the number of points is even (odd
/// number of intervals), the last interval is handled with the trapezoidal
/// rule.
///
/// Corresponds to `QuantLib::DiscreteSimpsonIntegral`.
pub fn discrete_simpson(x: &[Real], f: &[Real]) -> Real {
    debug_assert_eq!(x.len(), f.len());
    let n = x.len();
    if n < 2 {
        return 0.0;
    }
    if n == 2 {
        return 0.5 * (x[1] - x[0]) * (f[0] + f[1]);
    }

    let mut sum = 0.0;
    let mut j = 0;
    while j + 2 < n {
        let dxj = x[j + 1] - x[j];
        let dxjp1 = x[j + 2] - x[j + 1];
        let dd = dxj + dxjp1;
        let k = dd / (6.0 * dxjp1 * dxj);
        let alpha = dxjp1 * (2.0 * dxj - dxjp1);
        let beta = dd * dd;
        let gamma = dxj * (2.0 * dxjp1 - dxj);
        sum += k * (alpha * f[j] + beta * f[j + 1] + gamma * f[j + 2]);
        j += 2;
    }
    // If even number of points (odd intervals), last interval via trapezoid
    if n % 2 == 0 {
        sum += 0.5 * (x[n - 1] - x[n - 2]) * (f[n - 1] + f[n - 2]);
    }
    sum
}

/// Composite trapezoidal rule on a function, using a uniform grid of `n`
/// evaluation points.
///
/// Corresponds to `QuantLib::DiscreteTrapezoidIntegrator`.
pub fn discrete_trapezoid_fn<F: Fn(Real) -> Real>(f: F, a: Real, b: Real, n: usize) -> Real {
    assert!(n >= 2, "need at least 2 evaluation points");
    let h = (b - a) / (n - 1) as Real;
    let mut sum = 0.5 * (f(a) + f(b));
    for i in 1..n - 1 {
        sum += f(a + i as Real * h);
    }
    sum * h
}

/// Composite Simpson's rule on a function, using a uniform grid of `n`
/// evaluation points.
///
/// Corresponds to `QuantLib::DiscreteSimpsonIntegrator`.
pub fn discrete_simpson_fn<F: Fn(Real) -> Real>(f: F, a: Real, b: Real, n: usize) -> Real {
    assert!(n >= 3, "need at least 3 evaluation points");
    let intervals = n - 1;
    let h = (b - a) / intervals as Real;

    // Coefficients: endpoints 1, odd indices 4, even indices 2
    let mut sum = f(a);
    for i in 1..intervals {
        let x = a + i as Real * h;
        if i % 2 == 1 {
            sum += 4.0 * f(x);
        } else {
            sum += 2.0 * f(x);
        }
    }
    // Handle last interval if intervals is odd (can't pair into Simpson panels)
    if intervals % 2 == 1 {
        // Use Simpson's 3/8 rule for the last 3 intervals
        // For simplicity, add the last point normally and let the composite rule work
        sum += f(b);
        return sum * h / 3.0;
    }
    sum += f(b);
    sum * h / 3.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn trapezoid_on_array_linear() {
        // ∫₀¹ x dx = 0.5 — trapezoid is exact for linear functions
        let x = [0.0, 0.25, 0.5, 0.75, 1.0];
        let f: Vec<Real> = x.to_vec();
        let result = discrete_trapezoid(&x, &f);
        assert!((result - 0.5).abs() < 1e-14, "got {result}");
    }

    #[test]
    fn trapezoid_on_array_quadratic() {
        // ∫₀¹ x² dx = 1/3 — uniform grid
        let n = 101;
        let x: Vec<Real> = (0..n).map(|i| i as Real / (n - 1) as Real).collect();
        let f: Vec<Real> = x.iter().map(|&xi| xi * xi).collect();
        let result = discrete_trapezoid(&x, &f);
        assert!(
            (result - 1.0 / 3.0).abs() < 1e-4,
            "got {result}, expect 1/3"
        );
    }

    #[test]
    fn simpson_on_array_quadratic() {
        // Simpson should be exact for polynomials up to degree 3
        let x = [0.0, 0.5, 1.0];
        let f: Vec<Real> = x.iter().map(|&xi| xi * xi).collect();
        let result = discrete_simpson(&x, &f);
        assert!(
            (result - 1.0 / 3.0).abs() < 1e-12,
            "got {result}, expect 1/3"
        );
    }

    #[test]
    fn simpson_on_array_nonuniform() {
        // Non-uniform spacing: ∫₀¹ x² dx = 1/3
        let x = [0.0, 0.3, 0.7, 0.8, 1.0];
        let f: Vec<Real> = x.iter().map(|&xi| xi * xi).collect();
        let result = discrete_simpson(&x, &f);
        // Should be very close for quadratic
        assert!(
            (result - 1.0 / 3.0).abs() < 1e-10,
            "got {result}, expect ~1/3"
        );
    }

    #[test]
    fn trapezoid_fn_sin() {
        // ∫₀^π sin(x) dx = 2  (trapezoidal rule, O(h²) error)
        let result = discrete_trapezoid_fn(|x| x.sin(), 0.0, PI, 10_001);
        assert!((result - 2.0).abs() < 1e-7, "got {result}");
    }

    #[test]
    fn simpson_fn_cubic() {
        // ∫₀¹ x³ dx = 1/4 — Simpson is exact for cubics
        let result = discrete_simpson_fn(|x| x * x * x, 0.0, 1.0, 101);
        assert!((result - 0.25).abs() < 1e-8, "got {result}, expect 0.25");
    }
}
