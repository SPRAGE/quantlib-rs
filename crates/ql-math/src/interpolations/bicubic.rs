//! Bicubic spline interpolation on a 2D grid
//! (translates `ql/math/interpolations/bicubicsplineinterpolation.hpp`).
//!
//! For each row of the grid, a 1D cubic natural spline is built along `x`.
//! Then, for any query `(x, y)`, the row splines are evaluated at `x` to
//! produce an intermediate column of values, which is itself interpolated
//! along `y` with another cubic natural spline.

use ql_core::{errors::Result, Real};

use super::{bilinear::Interpolation2D, CubicNaturalSpline, Interpolation1D};

/// Bicubic spline interpolation on a rectangular grid.
///
/// `z` is stored in row-major order: `z[j * nx + i]` = f(xs\[i\], ys\[j\]).
///
/// Corresponds to `QuantLib::BicubicSpline`.
#[derive(Debug, Clone)]
pub struct BicubicSpline {
    xs: Vec<Real>,
    ys: Vec<Real>,
    /// One cubic spline per y-row, interpolating along x.
    row_splines: Vec<CubicNaturalSpline>,
}

impl BicubicSpline {
    /// Build a bicubic spline on the grid `(xs × ys → z)`.
    ///
    /// Both `xs` and `ys` must be sorted and have at least 3 elements (needed
    /// for cubic splines).  `z` is row-major: `z[j * nx + i]` = f(xs\[i\], ys\[j\]).
    pub fn new(xs: &[Real], ys: &[Real], z: &[Real]) -> Result<Self> {
        let nx = xs.len();
        let ny = ys.len();
        ql_core::ensure!(nx >= 3, "bicubic spline needs at least 3 x grid points");
        ql_core::ensure!(ny >= 3, "bicubic spline needs at least 3 y grid points");
        ql_core::ensure!(
            z.len() == nx * ny,
            "z length ({}) must equal nx*ny ({}*{}={})",
            z.len(),
            nx,
            ny,
            nx * ny
        );

        // Build one spline per y-row
        let mut row_splines = Vec::with_capacity(ny);
        for j in 0..ny {
            let row = &z[j * nx..(j + 1) * nx];
            row_splines.push(CubicNaturalSpline::new(xs, row)?);
        }

        Ok(Self {
            xs: xs.to_vec(),
            ys: ys.to_vec(),
            row_splines,
        })
    }
}

impl Interpolation2D for BicubicSpline {
    fn x_min(&self) -> Real {
        self.xs[0]
    }

    fn x_max(&self) -> Real {
        *self.xs.last().unwrap()
    }

    fn y_min(&self) -> Real {
        self.ys[0]
    }

    fn y_max(&self) -> Real {
        *self.ys.last().unwrap()
    }

    fn operator(&self, x: Real, y: Real) -> Real {
        // Evaluate each row spline at x → column of values
        let column: Vec<Real> = self.row_splines.iter().map(|s| s.operator(x)).collect();
        // Interpolate column along y
        let col_spline = CubicNaturalSpline::new(&self.ys, &column)
            .expect("column spline construction should not fail for valid grid");
        col_spline.operator(y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bicubic_exact_on_grid() {
        let xs = vec![0.0, 1.0, 2.0, 3.0];
        let ys = vec![0.0, 1.0, 2.0, 3.0];
        let mut z = Vec::new();
        for &y in &ys {
            for &x in &xs {
                z.push(x + y);
            }
        }
        let interp = BicubicSpline::new(&xs, &ys, &z).unwrap();
        // Should pass through all grid points
        for &y in &ys {
            for &x in &xs {
                let v = interp.operator(x, y);
                let expected = x + y;
                assert!(
                    (v - expected).abs() < 1e-10,
                    "at ({x},{y}): expected {expected}, got {v}"
                );
            }
        }
    }

    #[test]
    fn bicubic_smooth_interior() {
        // z = x*y over a 4×4 grid
        let xs = vec![0.0, 1.0, 2.0, 3.0];
        let ys = vec![0.0, 1.0, 2.0, 3.0];
        let mut z = Vec::new();
        for &y in &ys {
            for &x in &xs {
                z.push(x * y);
            }
        }
        let interp = BicubicSpline::new(&xs, &ys, &z).unwrap();
        // Check interior point
        let v = interp.operator(1.5, 1.5);
        let expected = 1.5 * 1.5;
        assert!(
            (v - expected).abs() < 0.3,
            "at (1.5,1.5): expected {expected}, got {v}"
        );
    }

    #[test]
    fn bicubic_reproduces_bilinear_function() {
        // z = 2x + 3y + 1: cubic spline should reproduce any linear function exactly
        let xs = vec![0.0, 1.0, 2.0, 3.0];
        let ys = vec![0.0, 1.0, 2.0, 3.0];
        let mut z = Vec::new();
        for &y in &ys {
            for &x in &xs {
                z.push(2.0 * x + 3.0 * y + 1.0);
            }
        }
        let interp = BicubicSpline::new(&xs, &ys, &z).unwrap();
        let v = interp.operator(1.5, 2.5);
        let expected = 2.0 * 1.5 + 3.0 * 2.5 + 1.0;
        assert!(
            (v - expected).abs() < 1e-10,
            "expected {expected}, got {v}"
        );
    }
}
