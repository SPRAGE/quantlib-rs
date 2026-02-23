//! Matrix decomposition and utility functions (translates
//! `ql/math/matrixutilities/`).
//!
//! Wraps nalgebra's decompositions into QuantLib-style APIs.

use crate::array::Array;
use crate::matrix::Matrix;
use ql_core::{
    errors::{Error, Result},
    Real,
};
use nalgebra::DMatrix;

/// Cholesky decomposition of a symmetric positive-definite matrix.
///
/// Returns the lower-triangular factor `L` such that `A = L * Lᵀ`.
///
/// Corresponds to `QuantLib::CholeskyDecomposition`.
pub fn cholesky_decomposition(m: &Matrix) -> Result<Matrix> {
    let inner = m.inner();
    if inner.nrows() != inner.ncols() {
        return Err(Error::InvalidArgument("matrix must be square".into()));
    }
    match inner.clone().cholesky() {
        Some(chol) => Ok(Matrix::from(chol.l())),
        None => Err(Error::Runtime(
            "Cholesky decomposition failed — matrix is not positive-definite".into(),
        )),
    }
}

/// Singular value decomposition.
///
/// Returns `(U, S, Vt)` where `U * diag(S) * Vt = A`.
///
/// Corresponds to `QuantLib::SVD`.
pub struct SVD {
    /// Left singular vectors (column unitary matrix).
    pub u: Matrix,
    /// Singular values (as an array, in descending order).
    pub singular_values: Array,
    /// Right singular vectors transposed.
    pub v_t: Matrix,
}

impl SVD {
    /// Compute the SVD of `m`.
    pub fn new(m: &Matrix) -> Self {
        let svd = m.inner().clone().svd(true, true);
        let u = svd.u.expect("U computed");
        let v_t = svd.v_t.expect("Vt computed");
        let sv = svd.singular_values;
        Self {
            u: Matrix::from(u),
            singular_values: Array::from(sv.clone_owned()),
            v_t: Matrix::from(v_t),
        }
    }
}

/// QR decomposition.
///
/// Returns `(Q, R)` where `Q` is orthogonal and `R` is upper-triangular.
///
/// Corresponds to `QuantLib::qrDecomposition`.
pub fn qr_decomposition(m: &Matrix) -> (Matrix, Matrix) {
    let qr = m.inner().clone().qr();
    let q = qr.q();
    let r = qr.r();
    (Matrix::from(q), Matrix::from(r))
}

/// LU decomposition with partial pivoting.
///
/// Returns `(L, U)` such that `P * A = L * U` for some permutation.
pub fn lu_decomposition(m: &Matrix) -> (Matrix, Matrix) {
    let lu = m.inner().clone().lu();
    let l = lu.l();
    let u = lu.u();
    (
        Matrix::from(l),
        Matrix::from(u),
    )
}

/// Eigenvalue decomposition of a symmetric real matrix.
///
/// Returns `(eigenvalues, eigenvectors)` where the eigenvalues are sorted in
/// ascending order and each column of the eigenvectors matrix is the
/// corresponding eigenvector.
///
/// Corresponds to `QuantLib::SymmetricSchurDecomposition`.
pub fn symmetric_eigen(m: &Matrix) -> Result<(Array, Matrix)> {
    let inner = m.inner();
    if inner.nrows() != inner.ncols() {
        return Err(Error::InvalidArgument("matrix must be square".into()));
    }
    let eigen = inner.clone().symmetric_eigen();
    // nalgebra returns eigenvalues in ascending order
    Ok((
        Array::from(eigen.eigenvalues.clone_owned()),
        Matrix::from(eigen.eigenvectors),
    ))
}

/// Pseudo square-root of a symmetric positive-semidefinite matrix.
///
/// Computes `S` such that `S * Sᵀ ≈ M` using the eigenvalue decomposition,
/// zeroing out negative eigenvalues.
///
/// Corresponds to `QuantLib::pseudoSqrt`.
pub fn pseudo_sqrt(m: &Matrix) -> Result<Matrix> {
    let (eigenvalues, eigenvectors) = symmetric_eigen(m)?;
    let n = eigenvalues.size();
    let mut diag = DMatrix::<Real>::zeros(n, n);
    for i in 0..n {
        let ev = eigenvalues[i];
        diag[(i, i)] = if ev > 0.0 { ev.sqrt() } else { 0.0 };
    }
    let evec = eigenvectors.into_inner();
    let result = &evec * &diag;
    Ok(Matrix::from(result))
}

/// Rank of a matrix (number of singular values above `tolerance`).
pub fn rank(m: &Matrix, tolerance: Real) -> usize {
    let svd = SVD::new(m);
    svd.singular_values
        .iter()
        .filter(|&&s| s > tolerance)
        .count()
}

/// Moore–Penrose pseudo-inverse.
pub fn pseudo_inverse(m: &Matrix, tolerance: Real) -> Matrix {
    let svd = m.inner().clone().svd(true, true);
    let u = svd.u.expect("U");
    let v_t = svd.v_t.expect("Vt");
    let s = &svd.singular_values;

    let n = s.len();
    let mut s_inv = DMatrix::<Real>::zeros(m.cols(), m.rows());
    for i in 0..n {
        if s[i] > tolerance {
            s_inv[(i, i)] = 1.0 / s[i];
        }
    }
    Matrix::from(v_t.transpose() * s_inv * u.transpose())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cholesky_2x2() {
        // [[4, 2], [2, 10]]
        let m = Matrix::from_row_slice(2, 2, &[4.0, 2.0, 2.0, 10.0]);
        let l = cholesky_decomposition(&m).unwrap();
        // Verify L * L^T == M
        let reconstructed = &l * &l.transpose();
        for i in 0..2 {
            for j in 0..2 {
                assert!(
                    (reconstructed[(i, j)] - m[(i, j)]).abs() < 1e-10,
                    "mismatch at ({i},{j})"
                );
            }
        }
    }

    #[test]
    fn cholesky_not_positive_definite() {
        let m = Matrix::from_row_slice(2, 2, &[-1.0, 0.0, 0.0, 1.0]);
        assert!(cholesky_decomposition(&m).is_err());
    }

    #[test]
    fn svd_basic() {
        let m = Matrix::from_row_slice(2, 2, &[3.0, 0.0, 0.0, 4.0]);
        let svd = SVD::new(&m);
        // Singular values of diag(3,4) are 4 and 3 (descending)
        let sv = &svd.singular_values;
        assert!((sv[0] - 4.0).abs() < 1e-10);
        assert!((sv[1] - 3.0).abs() < 1e-10);
    }

    #[test]
    fn qr_basic() {
        let m = Matrix::from_row_slice(3, 3, &[1.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 1.0]);
        let (q, r) = qr_decomposition(&m);
        // Q * R should reconstruct M
        let recon = &q * &r;
        for i in 0..3 {
            for j in 0..3 {
                assert!(
                    (recon[(i, j)] - m[(i, j)]).abs() < 1e-10,
                    "mismatch at ({i},{j}): {} vs {}",
                    recon[(i, j)],
                    m[(i, j)]
                );
            }
        }
    }

    #[test]
    fn symmetric_eigen_diagonal() {
        let m = Matrix::from_row_slice(3, 3, &[2.0, 0.0, 0.0, 0.0, 5.0, 0.0, 0.0, 0.0, 1.0]);
        let (vals, _vecs) = symmetric_eigen(&m).unwrap();
        let mut sorted: Vec<Real> = vals.iter().copied().collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert!((sorted[0] - 1.0).abs() < 1e-10);
        assert!((sorted[1] - 2.0).abs() < 1e-10);
        assert!((sorted[2] - 5.0).abs() < 1e-10);
    }

    #[test]
    fn pseudo_sqrt_identity() {
        let m = Matrix::identity(3);
        let s = pseudo_sqrt(&m).unwrap();
        // sqrt(I) should be close to identity (up to sign/column permutation)
        // The product S * S^T should be I
        let prod = &s * &s.transpose();
        for i in 0..3 {
            for j in 0..3 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!(
                    (prod[(i, j)] - expected).abs() < 1e-10,
                    "mismatch at ({i},{j})"
                );
            }
        }
    }

    #[test]
    fn rank_test() {
        // Rank-1 matrix
        let m = Matrix::from_row_slice(2, 2, &[1.0, 2.0, 2.0, 4.0]);
        assert_eq!(rank(&m, 1e-10), 1);
        // Full-rank matrix
        let m2 = Matrix::from_row_slice(2, 2, &[1.0, 0.0, 0.0, 1.0]);
        assert_eq!(rank(&m2, 1e-10), 2);
    }

    #[test]
    fn pseudo_inverse_test() {
        let m = Matrix::from_row_slice(2, 2, &[1.0, 2.0, 3.0, 4.0]);
        let pinv = pseudo_inverse(&m, 1e-10);
        // M * M+ * M should equal M
        let recon = &(&m * &pinv) * &m;
        for i in 0..2 {
            for j in 0..2 {
                assert!(
                    (recon[(i, j)] - m[(i, j)]).abs() < 1e-10,
                    "mismatch at ({i},{j})"
                );
            }
        }
    }
}
