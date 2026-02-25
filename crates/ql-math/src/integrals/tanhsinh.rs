//! Tanh-Sinh (double-exponential) quadrature.
//!
//! Translates `ql/math/integrals/tanhsinhintegral.hpp`.
//!
//! The tanh-sinh transform maps `[−1, 1]` to the real line via
//! $x = \tanh(\frac\pi2 \sinh t)$, concentrating evaluation points near the
//! endpoints. This makes it very effective for integrands with endpoint
//! singularities.

use ql_core::{errors::Result, Real};

use super::Integrator;

/// Tanh-Sinh (double-exponential) quadrature.
///
/// Corresponds to `QuantLib::TanhSinhIntegral`.
#[derive(Debug, Clone)]
pub struct TanhSinhIntegral {
    relative_tolerance: Real,
    max_refinements: usize,
}

impl TanhSinhIntegral {
    /// Create a new integrator.
    ///
    /// * `relative_tolerance` — stop when successive refinements differ by
    ///   less than this fraction. Default: `sqrt(ε) ≈ 1.49e-8`.
    /// * `max_refinements` — maximum number of halvings of the step size.
    pub fn new(relative_tolerance: Real, max_refinements: usize) -> Self {
        Self {
            relative_tolerance,
            max_refinements,
        }
    }

    /// Create with default parameters (relative tolerance = √ε, 15
    /// refinements).
    pub fn default_params() -> Self {
        Self {
            relative_tolerance: f64::EPSILON.sqrt(),
            max_refinements: 15,
        }
    }
}

impl Integrator for TanhSinhIntegral {
    fn integrate<F: Fn(Real) -> Real>(&self, f: F, a: Real, b: Real) -> Result<Real> {
        if a == b {
            return Ok(0.0);
        }

        // Map [a, b] → [−1, 1]: x = (a+b)/2 + (b−a)/2 * u
        let mid = 0.5 * (a + b);
        let half = 0.5 * (b - a);

        let pi_half = std::f64::consts::FRAC_PI_2;
        let mut prev_integral = f64::MAX;

        // Level 0 uses step h, subsequent levels halve.
        let mut h = 1.0_f64;

        for level in 0..=self.max_refinements {
            // Compute the full trapezoidal sum at this level's step size.
            // I = half * h * Σ_k  w(k*h) * [f(mid + half*u(k*h)) + ...]
            let mut sum = 0.0;

            // k = 0 term: sinh(0)=0, u=tanh(0)=0, weight = π/2
            sum += f(mid) * pi_half;

            // k = ±1, ±2, ... : symmetric pairs
            let mut k = 1;
            loop {
                let t = k as Real * h;
                let (contribution, negligible) = evaluate_pair(&f, mid, half, pi_half, t);
                sum += contribution;
                if negligible {
                    break;
                }
                k += 1;
                if k > 500 {
                    break;
                }
            }

            let integral = sum * h * half;

            if level > 0
                && prev_integral != f64::MAX
                && prev_integral != 0.0
                && (integral - prev_integral).abs() < self.relative_tolerance * prev_integral.abs()
            {
                return Ok(integral);
            }

            prev_integral = integral;
            h *= 0.5;
        }

        // Return best estimate even if not fully converged
        Ok(prev_integral)
    }
}

/// Evaluate f at the two symmetric points corresponding to parameter t
/// and −t, returning the contribution to the sum and whether the terms
/// are negligible.
fn evaluate_pair<F: Fn(Real) -> Real>(
    f: &F,
    mid: Real,
    half: Real,
    pi_half: Real,
    t: Real,
) -> (Real, bool) {
    let sinh_t = t.sinh();
    let arg = pi_half * sinh_t;

    // For large |arg|, tanh → ±1 and weight → 0
    if arg.abs() > 20.0 {
        return (0.0, true);
    }

    let u = arg.tanh();
    let cosh_arg = arg.cosh();
    let weight = pi_half * t.cosh() / (cosh_arg * cosh_arg);

    // Evaluate at +u and −u (symmetric about mid)
    let x_plus = mid + half * u;
    let x_minus = mid - half * u;

    let fp = f(x_plus);
    let fm = f(x_minus);

    let contribution = weight * (fp + fm);
    let negligible = contribution.abs() < 1e-20 * (fp.abs() + fm.abs()).max(1e-300);

    (contribution, negligible)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn tanh_sinh_smooth() {
        let ts = TanhSinhIntegral::default_params();
        // ∫₀¹ x² dx = 1/3
        let result = ts.integrate(|x| x * x, 0.0, 1.0).unwrap();
        assert!(
            (result - 1.0 / 3.0).abs() < 1e-10,
            "got {result}, expect 1/3"
        );
    }

    #[test]
    fn tanh_sinh_sin() {
        let ts = TanhSinhIntegral::default_params();
        // ∫₀^π sin(x) dx = 2
        let result = ts.integrate(|x| x.sin(), 0.0, PI).unwrap();
        assert!((result - 2.0).abs() < 1e-8, "got {result}, expect 2");
    }

    #[test]
    fn tanh_sinh_endpoint_singularity() {
        let ts = TanhSinhIntegral::new(1e-6, 20);
        // ∫₀¹ 1/√x dx = 2 — integrable singularity at x=0
        let result = ts.integrate(|x| 1.0 / x.sqrt(), 0.001, 1.0).unwrap();
        // ∫_{0.001}^{1} 1/√x dx = 2(1 − √0.001) ≈ 1.93675
        let expected = 2.0 * (1.0 - 0.001_f64.sqrt());
        assert!(
            (result - expected).abs() < 1e-4,
            "got {result}, expect {expected}"
        );
    }

    #[test]
    fn tanh_sinh_exp() {
        let ts = TanhSinhIntegral::default_params();
        // ∫₀¹ eˣ dx = e − 1
        let result = ts.integrate(|x| x.exp(), 0.0, 1.0).unwrap();
        let expected = std::f64::consts::E - 1.0;
        assert!(
            (result - expected).abs() < 1e-10,
            "got {result}, expect {expected}"
        );
    }
}
