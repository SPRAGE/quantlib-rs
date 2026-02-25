//! General linear least-squares regression (translates
//! `ql/math/generallinearleastsquares.hpp`).
//!
//! Uses SVD to solve the linear regression problem
//! $\mathbf{y} = \mathbf{A}\,\boldsymbol{\beta} + \boldsymbol{\varepsilon}$,
//! where $\mathbf{A}$ is the design matrix built from user-supplied basis
//! functions.
//!
//! This is essential for Longstaff-Schwartz–style Monte Carlo methods where
//! regression on basis function values is used to approximate continuation
//! values.

use crate::array::Array;
use crate::matrix::Matrix;
use crate::matrix_utilities::SVD;
use ql_core::{
    errors::{Error, Result},
    Real,
};

/// Result of a general linear least-squares regression.
///
/// Corresponds to `QuantLib::GeneralLinearLeastSquares`.
#[derive(Debug, Clone)]
pub struct LinearLeastSquaresRegression {
    /// Fitted coefficients β (one per basis function).
    coefficients: Array,
    /// Standard errors of the coefficients.
    standard_errors: Array,
    /// Residuals (y − A β).
    residuals: Array,
}

impl LinearLeastSquaresRegression {
    /// Fit the model using the given data and basis functions.
    ///
    /// * `x` — independent variable observations (length *n*).
    /// * `y` — dependent variable observations (length *n*).
    /// * `basis` — a slice of basis functions $\phi_j(x)$, $j = 0, \ldots, m-1$.
    ///
    /// Builds the *n × m* design matrix $A_{ij} = \phi_j(x_i)$ and solves via
    /// SVD, thresholding small singular values.
    pub fn new<F>(x: &[Real], y: &[Real], basis: &[F]) -> Result<Self>
    where
        F: Fn(Real) -> Real,
    {
        let n = x.len();
        let m = basis.len();
        if n != y.len() {
            return Err(Error::InvalidArgument(
                "x and y must have the same length".into(),
            ));
        }
        if n < m {
            return Err(Error::InvalidArgument(
                "more basis functions than data points".into(),
            ));
        }

        // Build design matrix A (n × m) in column-major order as nalgebra expects
        let mut a_data = vec![0.0; n * m];
        for j in 0..m {
            for i in 0..n {
                // column-major: index = i + j * n
                a_data[i + j * n] = basis[j](x[i]);
            }
        }
        let a = Matrix::from_column_slice(n, m, &a_data);

        Self::from_design_matrix(&a, y)
    }

    /// Fit the model given a pre-built design matrix.
    ///
    /// * `a` — the *n × m* design matrix.
    /// * `y` — dependent variable observations (length *n*).
    pub fn from_design_matrix(a: &Matrix, y: &[Real]) -> Result<Self> {
        let n = a.rows();
        let m = a.cols();
        if y.len() != n {
            return Err(Error::InvalidArgument(
                "y length must equal number of rows of A".into(),
            ));
        }

        let svd = SVD::new(a);
        let sv = &svd.singular_values;
        let u = svd.u.inner();
        let v_t = &svd.v_t;

        // Threshold: max(n,m) * eps * max(singular_values)
        let sv_max = sv.iter().copied().fold(0.0_f64, f64::max);
        let threshold = n.max(m) as Real * f64::EPSILON * sv_max;

        // β = V * diag(1/sᵢ) * Uᵀ * y  (only for sᵢ > threshold)
        let y_vec = nalgebra::DVector::from_column_slice(y);
        let ut_y = u.transpose() * &y_vec; // vector of length min(n,m)

        let num_sv = sv.size();
        let mut coefficients = vec![0.0; m];
        let mut var_diag = vec![0.0; m]; // for standard errors

        for k in 0..num_sv {
            let s = sv[k];
            if s > threshold {
                let ratio = ut_y[k] / s;
                for j in 0..m {
                    coefficients[j] += ratio * v_t[(k, j)];
                }
                // Contribution to variance: (1/s²) * v_kj²
                let s2_inv = 1.0 / (s * s);
                for j in 0..m {
                    var_diag[j] += s2_inv * v_t[(k, j)] * v_t[(k, j)];
                }
            }
        }

        let coeff_array = Array::from_vec(coefficients);

        // Residuals = y − A * β
        let a_beta = a.inner() * coeff_array.inner();
        let resid = &y_vec - &a_beta;
        let residuals = Array::from(resid);

        // Estimate σ² = ||residuals||² / (n − m)
        let resid_ss: Real = residuals.iter().map(|r| r * r).sum();
        let sigma2 = if n > m {
            resid_ss / (n - m) as Real
        } else {
            0.0
        };

        let standard_errors =
            Array::from_vec(var_diag.iter().map(|v| (v * sigma2).sqrt()).collect());

        Ok(Self {
            coefficients: coeff_array,
            standard_errors,
            residuals,
        })
    }

    /// Fitted coefficients β.
    pub fn coefficients(&self) -> &Array {
        &self.coefficients
    }

    /// Standard errors of the coefficients.
    pub fn standard_errors(&self) -> &Array {
        &self.standard_errors
    }

    /// Residuals (y − A β).
    pub fn residuals(&self) -> &Array {
        &self.residuals
    }

    /// R² statistic (coefficient of determination).
    pub fn r_squared(&self, y: &[Real]) -> Real {
        let n = y.len();
        if n == 0 {
            return 0.0;
        }
        let y_mean: Real = y.iter().sum::<Real>() / n as Real;
        let ss_tot: Real = y.iter().map(|&yi| (yi - y_mean).powi(2)).sum();
        let ss_res: Real = self.residuals.iter().map(|r| r * r).sum();
        if ss_tot == 0.0 {
            1.0 // perfect fit if all y are equal and residuals are zero
        } else {
            1.0 - ss_res / ss_tot
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_fit() {
        // y = 2 + 3x
        let x: Vec<Real> = (0..20).map(|i| i as Real).collect();
        let y: Vec<Real> = x.iter().map(|&xi| 2.0 + 3.0 * xi).collect();

        let basis: Vec<Box<dyn Fn(Real) -> Real>> = vec![Box::new(|_| 1.0), Box::new(|x| x)];

        let reg = LinearLeastSquaresRegression::new(&x, &y, &basis).unwrap();
        let c = reg.coefficients();
        assert!((c[0] - 2.0).abs() < 1e-10, "intercept = {}", c[0]);
        assert!((c[1] - 3.0).abs() < 1e-10, "slope = {}", c[1]);

        let r2 = reg.r_squared(&y);
        assert!((r2 - 1.0).abs() < 1e-10, "R² = {r2}");
    }

    #[test]
    fn quadratic_fit() {
        // y = 1 - 2x + 0.5x²
        let x: Vec<Real> = (0..30).map(|i| -5.0 + i as Real * 0.5).collect();
        let y: Vec<Real> = x.iter().map(|&xi| 1.0 - 2.0 * xi + 0.5 * xi * xi).collect();

        let basis: Vec<Box<dyn Fn(Real) -> Real>> =
            vec![Box::new(|_| 1.0), Box::new(|x| x), Box::new(|x| x * x)];

        let reg = LinearLeastSquaresRegression::new(&x, &y, &basis).unwrap();
        let c = reg.coefficients();
        assert!((c[0] - 1.0).abs() < 1e-8, "c0 = {}", c[0]);
        assert!((c[1] + 2.0).abs() < 1e-8, "c1 = {}", c[1]);
        assert!((c[2] - 0.5).abs() < 1e-8, "c2 = {}", c[2]);
    }

    #[test]
    fn noisy_linear_fit() {
        // y ≈ 1 + 2x with small noise
        let x: Vec<Real> = (0..100).map(|i| i as Real * 0.1).collect();
        let noise = [
            0.01, -0.02, 0.015, -0.005, 0.03, -0.01, 0.02, -0.03, 0.005, 0.01,
        ];
        let y: Vec<Real> = x
            .iter()
            .enumerate()
            .map(|(i, &xi)| 1.0 + 2.0 * xi + noise[i % noise.len()])
            .collect();

        let basis: Vec<Box<dyn Fn(Real) -> Real>> = vec![Box::new(|_| 1.0), Box::new(|x| x)];

        let reg = LinearLeastSquaresRegression::new(&x, &y, &basis).unwrap();
        let c = reg.coefficients();
        assert!((c[0] - 1.0).abs() < 0.1, "intercept = {}", c[0]);
        assert!((c[1] - 2.0).abs() < 0.01, "slope = {}", c[1]);
    }

    #[test]
    fn too_few_observations() {
        let x = [1.0];
        let y = [2.0];
        let basis: Vec<Box<dyn Fn(Real) -> Real>> = vec![Box::new(|_| 1.0), Box::new(|x| x)];
        assert!(LinearLeastSquaresRegression::new(&x, &y, &basis).is_err());
    }

    #[test]
    fn design_matrix_interface() {
        // 3 × 2 design matrix for y = a + bx
        let _x = [1.0, 2.0, 3.0];
        let y = [3.0, 5.0, 7.0]; // y = 1 + 2x exactly

        let a = Matrix::from_row_slice(3, 2, &[1.0, 1.0, 1.0, 2.0, 1.0, 3.0]);
        let reg = LinearLeastSquaresRegression::from_design_matrix(&a, &y).unwrap();
        let c = reg.coefficients();
        assert!((c[0] - 1.0).abs() < 1e-10, "intercept = {}", c[0]);
        assert!((c[1] - 2.0).abs() < 1e-10, "slope = {}", c[1]);
    }
}
