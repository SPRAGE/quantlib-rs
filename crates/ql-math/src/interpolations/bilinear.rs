//! Bilinear 2D interpolation between discrete grid points
//! (translates `ql/math/interpolations/bilinearinterpolation.hpp`).
//!
//! Standard bilinear interpolation on a rectangular grid `(xs × ys → z)`.

use ql_core::{errors::Result, Real};

/// 2D interpolation trait.
///
/// Corresponds to `QuantLib::Interpolation2D`.
pub trait Interpolation2D: std::fmt::Debug + Send + Sync {
    /// Evaluate the surface at `(x, y)`.
    fn operator(&self, x: Real, y: Real) -> Real;
    /// Lower bound of the x domain.
    fn x_min(&self) -> Real;
    /// Upper bound of the x domain.
    fn x_max(&self) -> Real;
    /// Lower bound of the y domain.
    fn y_min(&self) -> Real;
    /// Upper bound of the y domain.
    fn y_max(&self) -> Real;
}

/// Bilinear interpolation on a rectangular grid.
///
/// `z` is stored in row-major order: `z[j][i]` = f(xs[i], ys[j]).
///
/// Corresponds to `QuantLib::BilinearInterpolation`.
#[derive(Debug, Clone)]
pub struct BilinearInterpolation {
    xs: Vec<Real>,
    ys: Vec<Real>,
    /// z values stored in row-major order: z[j * nx + i] = f(xs[i], ys[j])
    z: Vec<Real>,
    nx: usize,
}

impl BilinearInterpolation {
    /// Build a bilinear interpolation on the grid `(xs × ys → z)`.
    ///
    /// `z` is row-major: `z[j * nx + i]` = f(xs\[i\], ys\[j\]).
    ///
    /// Both `xs` and `ys` must be sorted in strictly increasing order.
    pub fn new(xs: &[Real], ys: &[Real], z: &[Real]) -> Result<Self> {
        let nx = xs.len();
        let ny = ys.len();
        ql_core::ensure!(nx >= 2, "need at least 2 x points");
        ql_core::ensure!(ny >= 2, "need at least 2 y points");
        ql_core::ensure!(
            z.len() == nx * ny,
            "z length ({}) must equal nx*ny ({}*{}={})",
            z.len(),
            nx,
            ny,
            nx * ny
        );
        Ok(Self {
            xs: xs.to_vec(),
            ys: ys.to_vec(),
            z: z.to_vec(),
            nx,
        })
    }

    fn locate_x(&self, x: Real) -> usize {
        locate(&self.xs, x)
    }

    fn locate_y(&self, y: Real) -> usize {
        locate(&self.ys, y)
    }

    /// Access z[j][i] (row j, column i).
    fn z_at(&self, i: usize, j: usize) -> Real {
        self.z[j * self.nx + i]
    }
}

/// Binary search: find `k` such that `vs[k] <= v < vs[k+1]`, clamped.
fn locate(vs: &[Real], v: Real) -> usize {
    let n = vs.len();
    if v <= vs[0] {
        return 0;
    }
    if v >= vs[n - 1] {
        return n - 2;
    }
    let mut lo = 0;
    let mut hi = n - 1;
    while hi - lo > 1 {
        let mid = (lo + hi) / 2;
        if vs[mid] <= v {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}

impl Interpolation2D for BilinearInterpolation {
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
        let i = self.locate_x(x);
        let j = self.locate_y(y);

        let z1 = self.z_at(i, j);
        let z2 = self.z_at(i + 1, j);
        let z3 = self.z_at(i, j + 1);
        let z4 = self.z_at(i + 1, j + 1);

        let t = (x - self.xs[i]) / (self.xs[i + 1] - self.xs[i]);
        let u = (y - self.ys[j]) / (self.ys[j + 1] - self.ys[j]);

        (1.0 - t) * (1.0 - u) * z1 + t * (1.0 - u) * z2 + (1.0 - t) * u * z3 + t * u * z4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bilinear_exact_on_grid() {
        let xs = vec![0.0, 1.0, 2.0];
        let ys = vec![0.0, 1.0];
        // z[j][i]:  j=0: [1, 2, 3],  j=1: [4, 5, 6]
        let z = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let interp = BilinearInterpolation::new(&xs, &ys, &z).unwrap();
        assert!((interp.operator(0.0, 0.0) - 1.0).abs() < 1e-12);
        assert!((interp.operator(1.0, 0.0) - 2.0).abs() < 1e-12);
        assert!((interp.operator(2.0, 0.0) - 3.0).abs() < 1e-12);
        assert!((interp.operator(0.0, 1.0) - 4.0).abs() < 1e-12);
        assert!((interp.operator(1.0, 1.0) - 5.0).abs() < 1e-12);
        assert!((interp.operator(2.0, 1.0) - 6.0).abs() < 1e-12);
    }

    #[test]
    fn bilinear_midpoint() {
        let xs = vec![0.0, 1.0];
        let ys = vec![0.0, 1.0];
        // z = [[0, 1], [2, 3]]
        let z = vec![0.0, 1.0, 2.0, 3.0];
        let interp = BilinearInterpolation::new(&xs, &ys, &z).unwrap();
        // Centre: average of all four corners = (0+1+2+3)/4 = 1.5
        let v = interp.operator(0.5, 0.5);
        assert!((v - 1.5).abs() < 1e-12, "expected 1.5, got {v}");
    }

    #[test]
    fn bilinear_reproduces_plane() {
        // z = x + 2y on a 3×3 grid
        let xs = vec![0.0, 1.0, 2.0];
        let ys = vec![0.0, 1.0, 2.0];
        let mut z = Vec::new();
        for &y in &ys {
            for &x in &xs {
                z.push(x + 2.0 * y);
            }
        }
        let interp = BilinearInterpolation::new(&xs, &ys, &z).unwrap();
        // Bilinear should reproduce any bilinear function exactly
        let v = interp.operator(0.5, 1.5);
        let expected = 0.5 + 2.0 * 1.5;
        assert!((v - expected).abs() < 1e-12, "expected {expected}, got {v}");
    }

    #[test]
    fn bilinear_edge_interpolation() {
        let xs = vec![0.0, 1.0, 2.0];
        let ys = vec![0.0, 1.0];
        let z = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let interp = BilinearInterpolation::new(&xs, &ys, &z).unwrap();
        // Along y=0 edge: linear interp between 1,2,3
        let v = interp.operator(0.5, 0.0);
        assert!((v - 1.5).abs() < 1e-12);
    }
}
