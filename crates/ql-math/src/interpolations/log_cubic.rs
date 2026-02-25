//! Log-transformed cubic interpolation
//! (translates `ql/math/interpolations/loginterpolation.hpp`, cubic variants).
//!
//! Interpolates `log(y)` using a cubic scheme, then exponentiates.
//! This is the standard approach for discount-factor / survival-probability
//! curves where `y > 0` and log-space interpolation is natural.

use ql_core::{errors::Result, Real};

use super::{
    cubic::{FritschButlandCubic, KrugerCubic, ParabolicCubic},
    monotone_cubic::MonotoneCubicSpline,
    CubicNaturalSpline, Interpolation1D,
};

/// Which cubic scheme to use in log-space.
#[derive(Debug, Clone, Copy)]
pub enum LogCubicScheme {
    /// Natural cubic spline of log(y).
    Natural,
    /// Monotone cubic (Fritsch-Carlson) of log(y).
    MonotoneFritschCarlson,
    /// Fritsch-Butland cubic of log(y).
    FritschButland,
    /// Kruger cubic of log(y).
    Kruger,
    /// Parabolic cubic of log(y).
    Parabolic,
}

/// Log-cubic interpolation: cubic interpolation in log-space.
///
/// All `y` values must be strictly positive.
///
/// Corresponds to `QuantLib::LogCubicInterpolation` / `LogCubicNaturalSpline` / etc.
pub struct LogCubicInterpolation {
    inner: Box<dyn Interpolation1D>,
}

impl LogCubicInterpolation {
    /// Build a log-cubic interpolation.
    ///
    /// All `ys` must be strictly positive.
    pub fn new(xs: &[Real], ys: &[Real], scheme: LogCubicScheme) -> Result<Self> {
        ql_core::ensure!(
            ys.iter().all(|&y| y > 0.0),
            "all y values must be positive for log-cubic interpolation"
        );
        let log_ys: Vec<Real> = ys.iter().map(|&y| y.ln()).collect();
        let inner: Box<dyn Interpolation1D> = match scheme {
            LogCubicScheme::Natural => Box::new(CubicNaturalSpline::new(xs, &log_ys)?),
            LogCubicScheme::MonotoneFritschCarlson => {
                Box::new(MonotoneCubicSpline::new(xs, &log_ys)?)
            }
            LogCubicScheme::FritschButland => Box::new(FritschButlandCubic::new(xs, &log_ys)?),
            LogCubicScheme::Kruger => Box::new(KrugerCubic::new(xs, &log_ys)?),
            LogCubicScheme::Parabolic => Box::new(ParabolicCubic::new(xs, &log_ys)?),
        };
        Ok(Self { inner })
    }
}

impl Interpolation1D for LogCubicInterpolation {
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

impl std::fmt::Debug for LogCubicInterpolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LogCubicInterpolation")
            .field("inner", &"<cubic in log-space>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_cubic_passes_through_nodes() {
        // Discount factors: strictly positive, decreasing
        let xs = [0.0, 0.5, 1.0, 2.0, 3.0];
        let ys = [1.0, 0.98, 0.95, 0.88, 0.80];
        let interp = LogCubicInterpolation::new(&xs, &ys, LogCubicScheme::Natural).unwrap();
        for (&x, &y) in xs.iter().zip(ys.iter()) {
            let v = interp.operator(x);
            assert!(
                (v - y).abs() < 1e-10,
                "at x={x}: expected {y}, got {v}"
            );
        }
    }

    #[test]
    fn log_cubic_positive() {
        // Interpolated values in log-space are finite → exp should be positive
        let xs = [0.0, 1.0, 2.0, 3.0, 4.0];
        let ys = [1.0, 0.95, 0.88, 0.80, 0.70];
        let interp =
            LogCubicInterpolation::new(&xs, &ys, LogCubicScheme::FritschButland).unwrap();
        for i in 0..=40 {
            let x = 4.0 * (i as f64) / 40.0;
            let v = interp.operator(x);
            assert!(v > 0.0, "at x={x}: got non-positive {v}");
        }
    }

    #[test]
    fn log_cubic_exponential_exact() {
        // y = e^(-0.05*x): ln(y) = -0.05*x is linear → any cubic scheme
        // should reproduce it exactly
        let xs = [0.0, 1.0, 2.0, 3.0, 4.0];
        let ys: Vec<f64> = xs.iter().map(|&x| (-0.05 * x as f64).exp()).collect();
        let interp = LogCubicInterpolation::new(&xs, &ys, LogCubicScheme::Kruger).unwrap();
        // Check midpoints
        for &x in &[0.5, 1.5, 2.5, 3.5] {
            let expected = (-0.05_f64 * x).exp();
            let v = interp.operator(x);
            assert!(
                (v - expected).abs() < 1e-10,
                "at x={x}: expected {expected}, got {v}"
            );
        }
    }

    #[test]
    fn log_cubic_monotone_scheme() {
        // With Kruger (monotone), log-cubic should preserve decay ordering
        let xs = [0.0, 1.0, 2.0, 3.0, 4.0];
        let ys = [1.0, 0.95, 0.88, 0.80, 0.70];
        let interp = LogCubicInterpolation::new(&xs, &ys, LogCubicScheme::Kruger).unwrap();
        let mut prev = 2.0;
        for i in 0..=100 {
            let x = 4.0 * (i as f64) / 100.0;
            let v = interp.operator(x);
            assert!(v <= prev + 1e-12, "not decreasing at x={x}: {v} > {prev}");
            prev = v;
        }
    }

    #[test]
    fn log_cubic_parabolic() {
        // Basic test for the Parabolic scheme
        let xs = [0.0, 1.0, 2.0, 3.0, 4.0];
        let ys = [1.0, 0.98, 0.95, 0.90, 0.82];
        let interp = LogCubicInterpolation::new(&xs, &ys, LogCubicScheme::Parabolic).unwrap();
        let v = interp.operator(1.5);
        // Should be between 0.95 and 0.98
        assert!(
            (0.94..=0.99).contains(&v),
            "unexpected value at x=1.5: {v}"
        );
    }
}
