//! `Matrix` — a two-dimensional matrix of reals (translates `ql/math/matrix.hpp`).
//!
//! This is a thin newtype around `nalgebra::DMatrix<f64>` that exposes the
//! same API as QuantLib's `Matrix`: indexing, transpose, multiplication,
//! determinant, inverse, etc.

use crate::array::Array;
use nalgebra::DMatrix;
use ql_core::Real;
use std::ops::{Add, Index, IndexMut, Mul, Neg, Sub};

/// A dynamically-sized 2D matrix of `Real` values (row-major access).
///
/// Corresponds to `QuantLib::Matrix`.
#[derive(Debug, Clone, PartialEq)]
pub struct Matrix(DMatrix<Real>);

impl Matrix {
    /// Create a zero-filled `rows × cols` matrix.
    pub fn zeros(rows: usize, cols: usize) -> Self {
        Self(DMatrix::zeros(rows, cols))
    }

    /// Create a matrix filled with `value`.
    pub fn from_element(rows: usize, cols: usize, value: Real) -> Self {
        Self(DMatrix::from_element(rows, cols, value))
    }

    /// Create an identity matrix of size `n × n`.
    pub fn identity(n: usize) -> Self {
        Self(DMatrix::identity(n, n))
    }

    /// Create from a row-major data slice.
    pub fn from_row_slice(rows: usize, cols: usize, data: &[Real]) -> Self {
        Self(DMatrix::from_row_slice(rows, cols, data))
    }

    /// Create from column-major data slice (nalgebra's native layout).
    pub fn from_column_slice(rows: usize, cols: usize, data: &[Real]) -> Self {
        Self(DMatrix::from_column_slice(rows, cols, data))
    }

    /// Number of rows.
    pub fn rows(&self) -> usize {
        self.0.nrows()
    }

    /// Number of columns.
    pub fn cols(&self) -> usize {
        self.0.ncols()
    }

    /// Return `true` if the matrix is square.
    pub fn is_square(&self) -> bool {
        self.0.nrows() == self.0.ncols()
    }

    /// Return `true` if all elements are zero.
    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&x| x == 0.0)
    }

    /// Borrow the inner `DMatrix`.
    pub fn inner(&self) -> &DMatrix<Real> {
        &self.0
    }

    /// Consume and return the inner `DMatrix`.
    pub fn into_inner(self) -> DMatrix<Real> {
        self.0
    }

    /// Transpose.
    pub fn transpose(&self) -> Self {
        Self(self.0.transpose())
    }

    /// Determinant (only for square matrices).
    pub fn determinant(&self) -> Real {
        self.0.determinant()
    }

    /// Inverse (returns `None` if the matrix is singular or not square).
    pub fn try_inverse(&self) -> Option<Self> {
        self.0.clone().try_inverse().map(Self)
    }

    /// Trace (sum of diagonal elements).
    pub fn trace(&self) -> Real {
        self.0.trace()
    }

    /// Frobenius norm.
    pub fn norm(&self) -> Real {
        self.0.norm()
    }

    /// Diagonal elements as an `Array`.
    pub fn diagonal(&self) -> Array {
        let n = self.0.nrows().min(self.0.ncols());
        let data: Vec<Real> = (0..n).map(|i| self.0[(i, i)]).collect();
        Array::from_vec(data)
    }

    /// Extract a row as an `Array`.
    pub fn row(&self, i: usize) -> Array {
        let data: Vec<Real> = self.0.row(i).iter().copied().collect();
        Array::from_vec(data)
    }

    /// Extract a column as an `Array`.
    pub fn column(&self, j: usize) -> Array {
        let data: Vec<Real> = self.0.column(j).iter().copied().collect();
        Array::from_vec(data)
    }

    /// Matrix-vector product `M * v`.
    pub fn mul_vec(&self, v: &Array) -> Array {
        Array::from((&self.0 * v.inner()).clone_owned())
    }

    /// Element-wise apply.
    pub fn map<F: Fn(Real) -> Real>(&self, f: F) -> Self {
        Self(self.0.map(f))
    }

    /// Multiply every element by `scalar`.
    pub fn scale(&self, scalar: Real) -> Self {
        Self(&self.0 * scalar)
    }

    /// Sum of all elements.
    pub fn sum(&self) -> Real {
        self.0.sum()
    }
}

// ── From / Into ───────────────────────────────────────────────────────────────

impl From<DMatrix<Real>> for Matrix {
    fn from(m: DMatrix<Real>) -> Self {
        Self(m)
    }
}

impl From<Matrix> for DMatrix<Real> {
    fn from(m: Matrix) -> Self {
        m.0
    }
}

// ── Indexing ──────────────────────────────────────────────────────────────────

impl Index<(usize, usize)> for Matrix {
    type Output = Real;
    fn index(&self, (i, j): (usize, usize)) -> &Real {
        &self.0[(i, j)]
    }
}

impl IndexMut<(usize, usize)> for Matrix {
    fn index_mut(&mut self, (i, j): (usize, usize)) -> &mut Real {
        &mut self.0[(i, j)]
    }
}

// ── Arithmetic ────────────────────────────────────────────────────────────────

impl Add for &Matrix {
    type Output = Matrix;
    fn add(self, rhs: &Matrix) -> Matrix {
        Matrix(&self.0 + &rhs.0)
    }
}

impl Add for Matrix {
    type Output = Matrix;
    fn add(self, rhs: Matrix) -> Matrix {
        Matrix(self.0 + rhs.0)
    }
}

impl Sub for &Matrix {
    type Output = Matrix;
    fn sub(self, rhs: &Matrix) -> Matrix {
        Matrix(&self.0 - &rhs.0)
    }
}

impl Sub for Matrix {
    type Output = Matrix;
    fn sub(self, rhs: Matrix) -> Matrix {
        Matrix(self.0 - rhs.0)
    }
}

impl Mul for &Matrix {
    type Output = Matrix;
    fn mul(self, rhs: &Matrix) -> Matrix {
        Matrix(&self.0 * &rhs.0)
    }
}

impl Mul for Matrix {
    type Output = Matrix;
    fn mul(self, rhs: Matrix) -> Matrix {
        Matrix(self.0 * rhs.0)
    }
}

impl Mul<Real> for &Matrix {
    type Output = Matrix;
    fn mul(self, rhs: Real) -> Matrix {
        Matrix(&self.0 * rhs)
    }
}

impl Mul<Real> for Matrix {
    type Output = Matrix;
    fn mul(self, rhs: Real) -> Matrix {
        Matrix(self.0 * rhs)
    }
}

impl Neg for &Matrix {
    type Output = Matrix;
    fn neg(self) -> Matrix {
        Matrix(-&self.0)
    }
}

impl Neg for Matrix {
    type Output = Matrix;
    fn neg(self) -> Matrix {
        Matrix(-self.0)
    }
}

// ── Display ───────────────────────────────────────────────────────────────────

impl std::fmt::Display for Matrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.0.nrows() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "[")?;
            for j in 0..self.0.ncols() {
                if j > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", self.0[(i, j)])?;
            }
            write!(f, "]")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity() {
        let m = Matrix::identity(3);
        assert_eq!(m[(0, 0)], 1.0);
        assert_eq!(m[(0, 1)], 0.0);
        assert_eq!(m[(1, 1)], 1.0);
        assert_eq!(m.rows(), 3);
        assert_eq!(m.cols(), 3);
    }

    #[test]
    fn transpose() {
        let m = Matrix::from_row_slice(2, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        let mt = m.transpose();
        assert_eq!(mt.rows(), 3);
        assert_eq!(mt.cols(), 2);
        assert_eq!(mt[(0, 0)], 1.0);
        assert_eq!(mt[(0, 1)], 4.0);
        assert_eq!(mt[(2, 0)], 3.0);
    }

    #[test]
    fn multiply() {
        let a = Matrix::from_row_slice(2, 2, &[1.0, 2.0, 3.0, 4.0]);
        let b = Matrix::from_row_slice(2, 2, &[5.0, 6.0, 7.0, 8.0]);
        let c = &a * &b;
        assert_eq!(c[(0, 0)], 19.0);
        assert_eq!(c[(0, 1)], 22.0);
        assert_eq!(c[(1, 0)], 43.0);
        assert_eq!(c[(1, 1)], 50.0);
    }

    #[test]
    fn determinant_and_inverse() {
        let m = Matrix::from_row_slice(2, 2, &[1.0, 2.0, 3.0, 4.0]);
        assert!((m.determinant() - (-2.0)).abs() < 1e-12);
        let inv = m.try_inverse().unwrap();
        let prod = &m * &inv;
        assert!((prod[(0, 0)] - 1.0).abs() < 1e-12);
        assert!((prod[(0, 1)]).abs() < 1e-12);
        assert!((prod[(1, 0)]).abs() < 1e-12);
        assert!((prod[(1, 1)] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn singular_no_inverse() {
        let m = Matrix::from_row_slice(2, 2, &[1.0, 2.0, 2.0, 4.0]);
        assert!(m.try_inverse().is_none());
    }

    #[test]
    fn trace_and_diagonal() {
        let m = Matrix::from_row_slice(3, 3, &[1.0, 0.0, 0.0, 0.0, 5.0, 0.0, 0.0, 0.0, 9.0]);
        assert!((m.trace() - 15.0).abs() < 1e-12);
        let d = m.diagonal();
        assert_eq!(d[0], 1.0);
        assert_eq!(d[1], 5.0);
        assert_eq!(d[2], 9.0);
    }

    #[test]
    fn matrix_vector_mul() {
        let m = Matrix::from_row_slice(2, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        let v = Array::from_slice(&[1.0, 1.0, 1.0]);
        let result = m.mul_vec(&v);
        assert_eq!(result.size(), 2);
        assert!((result[0] - 6.0).abs() < 1e-12);
        assert!((result[1] - 15.0).abs() < 1e-12);
    }

    #[test]
    fn add_sub_scale() {
        let a = Matrix::from_row_slice(2, 2, &[1.0, 2.0, 3.0, 4.0]);
        let b = Matrix::from_row_slice(2, 2, &[5.0, 6.0, 7.0, 8.0]);
        let s = &a + &b;
        assert_eq!(s[(0, 0)], 6.0);
        let d = &b - &a;
        assert_eq!(d[(0, 0)], 4.0);
        let sc = a.scale(2.0);
        assert_eq!(sc[(1, 1)], 8.0);
    }
}
