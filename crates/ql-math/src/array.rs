//! `Array` — a one-dimensional vector of reals (translates `ql/math/array.hpp`).
//!
//! This is a thin newtype around `nalgebra::DVector<f64>` that exposes the
//! same API as QuantLib's `Array`: indexing, element-wise arithmetic, dot
//! product, and norms.

use nalgebra::DVector;
use ql_core::Real;
use std::ops::{Add, Div, Index, IndexMut, Mul, Neg, Sub};

/// A dynamically-sized 1D vector of `Real` values.
///
/// Corresponds to `QuantLib::Array`.
#[derive(Debug, Clone, PartialEq)]
pub struct Array(DVector<Real>);

impl Array {
    /// Create a zero-filled array of length `n`.
    pub fn zeros(n: usize) -> Self {
        Self(DVector::zeros(n))
    }

    /// Create an array filled with `value`.
    pub fn from_element(n: usize, value: Real) -> Self {
        Self(DVector::from_element(n, value))
    }

    /// Create an array from a slice.
    pub fn from_slice(data: &[Real]) -> Self {
        Self(DVector::from_column_slice(data))
    }

    /// Create an array from a `Vec`.
    pub fn from_vec(data: Vec<Real>) -> Self {
        Self(DVector::from_vec(data))
    }

    /// Number of elements.
    pub fn size(&self) -> usize {
        self.0.len()
    }

    /// Return `true` if empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Return the elements as a slice.
    pub fn as_slice(&self) -> &[Real] {
        self.0.as_slice()
    }

    /// Return the elements as a mutable slice.
    pub fn as_mut_slice(&mut self) -> &mut [Real] {
        self.0.as_mut_slice()
    }

    /// Borrow the inner `DVector`.
    pub fn inner(&self) -> &DVector<Real> {
        &self.0
    }

    /// Consume and return the inner `DVector`.
    pub fn into_inner(self) -> DVector<Real> {
        self.0
    }

    /// Dot product with another array.
    pub fn dot(&self, other: &Array) -> Real {
        self.0.dot(&other.0)
    }

    /// Euclidean (L2) norm.
    pub fn norm(&self) -> Real {
        self.0.norm()
    }

    /// Squared Euclidean norm.
    pub fn norm_squared(&self) -> Real {
        self.0.norm_squared()
    }

    /// Sum of all elements.
    pub fn sum(&self) -> Real {
        self.0.sum()
    }

    /// Minimum element.
    pub fn min(&self) -> Real {
        self.0.min()
    }

    /// Maximum element.
    pub fn max(&self) -> Real {
        self.0.max()
    }

    /// Apply a function element-wise, returning a new array.
    pub fn map<F: Fn(Real) -> Real>(&self, f: F) -> Self {
        Self(self.0.map(f))
    }

    /// Multiply every element by `scalar`.
    pub fn scale(&self, scalar: Real) -> Self {
        Self(&self.0 * scalar)
    }

    /// Element-wise absolute value.
    pub fn abs(&self) -> Self {
        self.map(|x| x.abs())
    }

    /// Iterator over elements.
    pub fn iter(&self) -> impl Iterator<Item = &Real> {
        self.0.iter()
    }
}

// ── From / Into conversions ───────────────────────────────────────────────────

impl From<DVector<Real>> for Array {
    fn from(v: DVector<Real>) -> Self {
        Self(v)
    }
}

impl From<Array> for DVector<Real> {
    fn from(a: Array) -> Self {
        a.0
    }
}

impl From<Vec<Real>> for Array {
    fn from(v: Vec<Real>) -> Self {
        Self::from_vec(v)
    }
}

impl From<&[Real]> for Array {
    fn from(s: &[Real]) -> Self {
        Self::from_slice(s)
    }
}

// ── Index ─────────────────────────────────────────────────────────────────────

impl Index<usize> for Array {
    type Output = Real;
    fn index(&self, i: usize) -> &Real {
        &self.0[i]
    }
}

impl IndexMut<usize> for Array {
    fn index_mut(&mut self, i: usize) -> &mut Real {
        &mut self.0[i]
    }
}

// ── Element-wise arithmetic ───────────────────────────────────────────────────

impl Add for &Array {
    type Output = Array;
    fn add(self, rhs: &Array) -> Array {
        Array(&self.0 + &rhs.0)
    }
}

impl Add for Array {
    type Output = Array;
    fn add(self, rhs: Array) -> Array {
        Array(self.0 + rhs.0)
    }
}

impl Sub for &Array {
    type Output = Array;
    fn sub(self, rhs: &Array) -> Array {
        Array(&self.0 - &rhs.0)
    }
}

impl Sub for Array {
    type Output = Array;
    fn sub(self, rhs: Array) -> Array {
        Array(self.0 - rhs.0)
    }
}

impl Mul<Real> for &Array {
    type Output = Array;
    fn mul(self, rhs: Real) -> Array {
        Array(&self.0 * rhs)
    }
}

impl Mul<Real> for Array {
    type Output = Array;
    fn mul(self, rhs: Real) -> Array {
        Array(self.0 * rhs)
    }
}

impl Mul<&Array> for Real {
    type Output = Array;
    fn mul(self, rhs: &Array) -> Array {
        Array(&rhs.0 * self)
    }
}

impl Div<Real> for &Array {
    type Output = Array;
    fn div(self, rhs: Real) -> Array {
        Array(&self.0 / rhs)
    }
}

impl Div<Real> for Array {
    type Output = Array;
    fn div(self, rhs: Real) -> Array {
        Array(self.0 / rhs)
    }
}

impl Neg for &Array {
    type Output = Array;
    fn neg(self) -> Array {
        Array(-&self.0)
    }
}

impl Neg for Array {
    type Output = Array;
    fn neg(self) -> Array {
        Array(-self.0)
    }
}

// ── Display ───────────────────────────────────────────────────────────────────

impl std::fmt::Display for Array {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (i, v) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{v}")?;
        }
        write!(f, "]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zeros() {
        let a = Array::zeros(5);
        assert_eq!(a.size(), 5);
        assert_eq!(a[0], 0.0);
    }

    #[test]
    fn from_slice() {
        let a = Array::from_slice(&[1.0, 2.0, 3.0]);
        assert_eq!(a.size(), 3);
        assert_eq!(a[1], 2.0);
    }

    #[test]
    fn dot_product() {
        let a = Array::from_slice(&[1.0, 2.0, 3.0]);
        let b = Array::from_slice(&[4.0, 5.0, 6.0]);
        assert!((a.dot(&b) - 32.0).abs() < 1e-12);
    }

    #[test]
    fn norm() {
        let a = Array::from_slice(&[3.0, 4.0]);
        assert!((a.norm() - 5.0).abs() < 1e-12);
    }

    #[test]
    fn element_wise_ops() {
        let a = Array::from_slice(&[1.0, 2.0, 3.0]);
        let b = Array::from_slice(&[4.0, 5.0, 6.0]);
        let sum = &a + &b;
        assert_eq!(sum[0], 5.0);
        assert_eq!(sum[1], 7.0);
        assert_eq!(sum[2], 9.0);

        let diff = &b - &a;
        assert_eq!(diff[0], 3.0);

        let scaled = &a * 2.0;
        assert_eq!(scaled[0], 2.0);
        assert_eq!(scaled[2], 6.0);

        let neg = -&a;
        assert_eq!(neg[0], -1.0);
    }

    #[test]
    fn sum_min_max() {
        let a = Array::from_slice(&[1.0, 5.0, 3.0, 2.0]);
        assert!((a.sum() - 11.0).abs() < 1e-12);
        assert!((a.min() - 1.0).abs() < 1e-12);
        assert!((a.max() - 5.0).abs() < 1e-12);
    }

    #[test]
    fn map_abs() {
        let a = Array::from_slice(&[-1.0, 2.0, -3.0]);
        let b = a.abs();
        assert_eq!(b[0], 1.0);
        assert_eq!(b[2], 3.0);
    }
}
