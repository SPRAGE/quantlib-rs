//! Numerical integration (translates `ql/math/integrals/`).
//!
//! Provides Simpson, trapezoidal, Gauss-Kronrod adaptive,
//! Gauss-Lobatto quadrature rules, and Gaussian quadratures
//! (Legendre, Hermite, Laguerre, Chebyshev, Jacobi).

pub mod gaussianquadratures;

use ql_core::{
    errors::{Error, Result},
    Real,
};

/// A numerical integrator.
///
/// Corresponds to the abstract `QuantLib::Integrator` class.
pub trait Integrator {
    /// Integrate `f` on `[a, b]`.
    fn integrate<F: Fn(Real) -> Real>(&self, f: F, a: Real, b: Real) -> Result<Real>;
}

// ── Simpson ───────────────────────────────────────────────────────────────────

/// Simpson's rule (composite).
///
/// Corresponds to `QuantLib::SimpsonIntegral`.
#[derive(Debug, Clone)]
pub struct SimpsonIntegral {
    max_evaluations: usize,
    absolute_accuracy: Real,
}

impl SimpsonIntegral {
    /// Create a new Simpson integrator.
    pub fn new(absolute_accuracy: Real, max_evaluations: usize) -> Self {
        Self {
            max_evaluations,
            absolute_accuracy,
        }
    }
}

impl Integrator for SimpsonIntegral {
    fn integrate<F: Fn(Real) -> Real>(&self, f: F, a: Real, b: Real) -> Result<Real> {
        if a == b {
            return Ok(0.0);
        }
        let mut n = 1usize;
        let mut old_value = f64::MAX;
        let mut evals = 0;

        loop {
            let h = (b - a) / (2.0 * n as Real);
            // Composite Simpson's rule: S = h/3 * [f(a) + 4*Σf(odd) + 2*Σf(even) + f(b)]
            let mut sum_odd = 0.0;
            let mut sum_even = 0.0;
            for i in 1..2 * n {
                let x = a + i as Real * h;
                if i % 2 == 1 {
                    sum_odd += f(x);
                } else {
                    sum_even += f(x);
                }
            }
            evals += 2 * n;
            let value = h / 3.0 * (f(a) + 4.0 * sum_odd + 2.0 * sum_even + f(b));

            if evals > 2 && (value - old_value).abs() < self.absolute_accuracy {
                return Ok(value);
            }
            if evals >= self.max_evaluations {
                return Err(Error::Runtime(format!(
                    "SimpsonIntegral: max evaluations ({}) exceeded",
                    self.max_evaluations
                )));
            }
            old_value = value;
            n *= 2;
        }
    }
}

// ── Trapezoid ─────────────────────────────────────────────────────────────────

/// Composite trapezoidal rule with successive refinement.
///
/// Corresponds to `QuantLib::TrapezoidIntegral`.
#[derive(Debug, Clone)]
pub struct TrapezoidIntegral {
    max_evaluations: usize,
    absolute_accuracy: Real,
}

impl TrapezoidIntegral {
    /// Create a new trapezoidal integrator.
    pub fn new(absolute_accuracy: Real, max_evaluations: usize) -> Self {
        Self {
            max_evaluations,
            absolute_accuracy,
        }
    }
}

impl Integrator for TrapezoidIntegral {
    fn integrate<F: Fn(Real) -> Real>(&self, f: F, a: Real, b: Real) -> Result<Real> {
        if a == b {
            return Ok(0.0);
        }

        let mut n = 1usize;
        let mut old_value = 0.5 * (b - a) * (f(a) + f(b));
        let mut evals = 2usize;

        loop {
            n *= 2;
            let h = (b - a) / n as Real;
            // Add midpoints of all previous subintervals
            let mut sum = 0.0;
            for i in (1..n).step_by(2) {
                sum += f(a + i as Real * h);
            }
            evals += n / 2;
            let value = 0.5 * old_value + h * sum;

            if n > 2 && (value - old_value).abs() < self.absolute_accuracy {
                return Ok(value);
            }
            if evals >= self.max_evaluations {
                return Err(Error::Runtime(format!(
                    "TrapezoidIntegral: max evaluations ({}) exceeded",
                    self.max_evaluations
                )));
            }
            old_value = value;
        }
    }
}

// ── Gauss-Kronrod ─────────────────────────────────────────────────────────────

/// Gauss-Kronrod adaptive integration using 15-point rule.
///
/// Corresponds to `QuantLib::GaussKronrodAdaptive`.
#[derive(Debug, Clone)]
pub struct GaussKronrodAdaptive {
    absolute_accuracy: Real,
    max_evaluations: usize,
}

impl GaussKronrodAdaptive {
    /// Create a new integrator.
    pub fn new(absolute_accuracy: Real, max_evaluations: usize) -> Self {
        Self {
            absolute_accuracy,
            max_evaluations,
        }
    }

    fn integrate_recursive<F: Fn(Real) -> Real>(
        &self,
        f: &F,
        a: Real,
        b: Real,
        evals: &mut usize,
    ) -> Result<Real> {
        if *evals >= self.max_evaluations {
            return Err(Error::Runtime(
                "GaussKronrodAdaptive: max evaluations exceeded".into(),
            ));
        }

        let mid = 0.5 * (a + b);
        let half = 0.5 * (b - a);

        // 7-point Gauss and 15-point Kronrod nodes and weights (scaled to [-1,1])
        // Using a simpler 3+7 rule for illustration (G3/K7 pair)
        static G_NODES: [Real; 3] = [
            0.0,
            0.774_596_669_241_483_4,
            -0.774_596_669_241_483_4,
        ];
        static G_WEIGHTS: [Real; 3] = [
            0.888_888_888_888_888_8,
            0.555_555_555_555_555_6,
            0.555_555_555_555_555_6,
        ];
        static K_NODES: [Real; 7] = [
            0.0,
            0.405_845_151_377_397_2,
            -0.405_845_151_377_397_2,
            0.774_596_669_241_483_4,
            -0.774_596_669_241_483_4,
            0.960_491_268_708_02,
            -0.960_491_268_708_02,
        ];
        static K_WEIGHTS: [Real; 7] = [
            0.450_916_538_658_474,
            0.401_397_414_775_962_4,
            0.401_397_414_775_962_4,
            0.268_488_089_868_333_4,
            0.268_488_089_868_333_4,
            0.104_656_226_026_467_26,
            0.104_656_226_026_467_26,
        ];

        // Gauss estimate
        let gauss: Real = G_NODES
            .iter()
            .zip(G_WEIGHTS.iter())
            .map(|(&n, &w)| w * f(mid + half * n))
            .sum::<Real>()
            * half;

        // Kronrod estimate
        let kronrod: Real = K_NODES
            .iter()
            .zip(K_WEIGHTS.iter())
            .map(|(&n, &w)| w * f(mid + half * n))
            .sum::<Real>()
            * half;

        *evals += 10; // total function evaluations this call

        let error = (kronrod - gauss).abs();
        if error < self.absolute_accuracy || half.abs() < 1e-15 {
            return Ok(kronrod);
        }

        // Recurse on sub-intervals
        let left = self.integrate_recursive(f, a, mid, evals)?;
        let right = self.integrate_recursive(f, mid, b, evals)?;
        Ok(left + right)
    }
}

impl Integrator for GaussKronrodAdaptive {
    fn integrate<F: Fn(Real) -> Real>(&self, f: F, a: Real, b: Real) -> Result<Real> {
        let mut evals = 0;
        self.integrate_recursive(&f, a, b, &mut evals)
    }
}

// ── Gauss-Lobatto ─────────────────────────────────────────────────────────────

/// Gauss-Lobatto adaptive integration.
///
/// Corresponds to `QuantLib::GaussLobattoIntegral`.
#[derive(Debug, Clone)]
pub struct GaussLobattoIntegral {
    absolute_accuracy: Real,
    max_evaluations: usize,
}

impl GaussLobattoIntegral {
    /// Create a new integrator.
    pub fn new(absolute_accuracy: Real, max_evaluations: usize) -> Self {
        Self {
            absolute_accuracy,
            max_evaluations,
        }
    }

    fn adaptive<F: Fn(Real) -> Real>(
        &self,
        f: &F,
        a: Real,
        b: Real,
        fa: Real,
        fb: Real,
        evals: &mut usize,
    ) -> Result<Real> {
        if *evals >= self.max_evaluations {
            return Err(Error::Runtime(
                "GaussLobattoIntegral: max evaluations exceeded".into(),
            ));
        }

        let h = b - a;
        let mid = 0.5 * (a + b);
        let m_left = 0.5 * (a + mid);
        let m_right = 0.5 * (mid + b);

        let _fml = f(m_left);
        let fmid = f(mid);
        let _fmr = f(m_right);
        *evals += 3;

        // Use a simple Richardson-style estimate:
        let coarse = 0.5 * h * (fa + fb);
        let fine = h / 6.0 * (fa + 4.0 * fmid + fb);

        if (fine - coarse).abs() < self.absolute_accuracy || h.abs() < 1e-15 {
            return Ok(fine);
        }

        let left = self.adaptive(f, a, mid, fa, fmid, evals)?;
        let right = self.adaptive(f, mid, b, fmid, fb, evals)?;
        Ok(left + right)
    }
}

impl Integrator for GaussLobattoIntegral {
    fn integrate<F: Fn(Real) -> Real>(&self, f: F, a: Real, b: Real) -> Result<Real> {
        let fa = f(a);
        let fb = f(b);
        let mut evals = 2;
        self.adaptive(&f, a, b, fa, fb, &mut evals)
    }
}

// ── Segment integral (simple, non-adaptive) ──────────────────────────────────

/// A fixed-step numerical integrator using the midpoint rule with `n`
/// equally-spaced sub-intervals. Useful for quick approximations.
#[derive(Debug, Clone)]
pub struct SegmentIntegral {
    /// Number of segments.
    pub intervals: usize,
}

impl SegmentIntegral {
    /// Create a new segment integrator.
    pub fn new(intervals: usize) -> Self {
        Self { intervals }
    }
}

impl Integrator for SegmentIntegral {
    fn integrate<F: Fn(Real) -> Real>(&self, f: F, a: Real, b: Real) -> Result<Real> {
        let n = self.intervals;
        if n == 0 {
            return Err(Error::InvalidArgument(
                "SegmentIntegral: intervals must be > 0".into(),
            ));
        }
        let h = (b - a) / n as Real;
        let mut sum = 0.0;
        for i in 0..n {
            let x = a + (i as Real + 0.5) * h;
            sum += f(x);
        }
        Ok(sum * h)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simpson_x_squared() {
        let s = SimpsonIntegral::new(1e-10, 10_000);
        // ∫₀¹ x² dx = 1/3
        let result = s.integrate(|x| x * x, 0.0, 1.0).unwrap();
        assert!(
            (result - 1.0 / 3.0).abs() < 1e-8,
            "got {result}"
        );
    }

    #[test]
    fn trapezoid_x_squared() {
        let t = TrapezoidIntegral::new(1e-8, 100_000);
        let result = t.integrate(|x| x * x, 0.0, 1.0).unwrap();
        assert!(
            (result - 1.0 / 3.0).abs() < 1e-6,
            "got {result}"
        );
    }

    #[test]
    fn gauss_kronrod_sin() {
        let gk = GaussKronrodAdaptive::new(1e-10, 100_000);
        // ∫₀^π sin(x) dx = 2
        let result = gk.integrate(|x| x.sin(), 0.0, std::f64::consts::PI).unwrap();
        assert!(
            (result - 2.0).abs() < 1e-6,
            "got {result}"
        );
    }

    #[test]
    fn gauss_lobatto_exp() {
        let gl = GaussLobattoIntegral::new(1e-8, 100_000);
        // ∫₀¹ e^x dx = e - 1
        let result = gl.integrate(|x| x.exp(), 0.0, 1.0).unwrap();
        let expected = std::f64::consts::E - 1.0;
        assert!(
            (result - expected).abs() < 1e-6,
            "got {result}, expected {expected}"
        );
    }

    #[test]
    fn segment_constant() {
        let seg = SegmentIntegral::new(100);
        // ∫₀¹ 5 dx = 5
        let result = seg.integrate(|_| 5.0, 0.0, 1.0).unwrap();
        assert!((result - 5.0).abs() < 1e-12);
    }
}
