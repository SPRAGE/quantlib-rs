//! Finite difference methods for PDE-based option pricing.
//!
//! Translates the core of `ql/methods/finitedifferences/` — the 1-D
//! Black-Scholes PDE solver using implicit, explicit, or Crank-Nicolson schemes.
//!
//! # Overview
//!
//! * [`TridiagonalOperator`] — tridiagonal matrix with Thomas-algorithm solver
//! * [`Fdm1dSolver`] — 1-D finite difference solver for the BS PDE
//! * [`FdmScheme`] — explicit, implicit, or Crank-Nicolson

use ql_core::Real;

// ─── FDM scheme selection ─────────────────────────────────────────────────────

/// Finite difference time-stepping scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FdmScheme {
    /// Explicit: `V^{n} = A · V^{n+1}` — simple but conditionally stable.
    Explicit,
    /// Fully implicit: `A · V^{n} = V^{n+1}` — unconditionally stable.
    Implicit,
    /// Crank-Nicolson: θ-average of explicit and implicit — second-order in time.
    CrankNicolson,
}

// ─── Tridiagonal operator ─────────────────────────────────────────────────────

/// A tridiagonal matrix operator.
///
/// Stores the lower, diagonal, and upper bands. Used for 1-D finite difference
/// discretisations of second-order PDEs.
///
/// Corresponds to the core of `QuantLib::TridiagonalOperator`.
#[derive(Debug, Clone)]
pub struct TridiagonalOperator {
    /// Lower diagonal (index 0 unused — starts from row 1).
    pub lower: Vec<Real>,
    /// Main diagonal.
    pub diag: Vec<Real>,
    /// Upper diagonal (last index unused — ends at row n−2).
    pub upper: Vec<Real>,
}

impl TridiagonalOperator {
    /// Create a zero tridiagonal operator of size `n`.
    pub fn new(n: usize) -> Self {
        Self {
            lower: vec![0.0; n],
            diag: vec![0.0; n],
            upper: vec![0.0; n],
        }
    }

    /// Size (number of rows/columns).
    pub fn size(&self) -> usize {
        self.diag.len()
    }

    /// Apply the operator: `y = A · x`.
    pub fn apply(&self, x: &[Real]) -> Vec<Real> {
        let n = self.size();
        assert_eq!(x.len(), n);
        let mut y = vec![0.0; n];
        y[0] = self.diag[0] * x[0] + self.upper[0] * x[1];
        for i in 1..n - 1 {
            y[i] = self.lower[i] * x[i - 1] + self.diag[i] * x[i] + self.upper[i] * x[i + 1];
        }
        y[n - 1] = self.lower[n - 1] * x[n - 2] + self.diag[n - 1] * x[n - 1];
        y
    }

    /// Solve `A · x = rhs` using the Thomas algorithm (LU decomposition
    /// for tridiagonal systems).
    ///
    /// Returns the solution vector `x`.
    pub fn solve(&self, rhs: &[Real]) -> Vec<Real> {
        let n = self.size();
        assert_eq!(rhs.len(), n);

        // Forward sweep
        let mut c_prime = vec![0.0; n];
        let mut d_prime = vec![0.0; n];

        c_prime[0] = self.upper[0] / self.diag[0];
        d_prime[0] = rhs[0] / self.diag[0];

        for i in 1..n {
            let m = self.diag[i] - self.lower[i] * c_prime[i - 1];
            if i < n - 1 {
                c_prime[i] = self.upper[i] / m;
            }
            d_prime[i] = (rhs[i] - self.lower[i] * d_prime[i - 1]) / m;
        }

        // Back substitution
        let mut x = vec![0.0; n];
        x[n - 1] = d_prime[n - 1];
        for i in (0..n - 1).rev() {
            x[i] = d_prime[i] - c_prime[i] * x[i + 1];
        }

        x
    }

    /// Scale all entries by a scalar.
    pub fn scale(&mut self, factor: Real) {
        for v in &mut self.lower {
            *v *= factor;
        }
        for v in &mut self.diag {
            *v *= factor;
        }
        for v in &mut self.upper {
            *v *= factor;
        }
    }

    /// Add the identity matrix scaled by `factor`: `A ← A + factor · I`.
    pub fn add_identity(&mut self, factor: Real) {
        for d in &mut self.diag {
            *d += factor;
        }
    }
}

// ─── 1-D Black-Scholes FDM solver ────────────────────────────────────────────

/// A 1-D finite difference solver for the Black-Scholes PDE.
///
/// Solves `∂V/∂t + ½σ²S²·∂²V/∂S² + (r−q)S·∂V/∂S − rV = 0`
/// backward in time from the terminal payoff.
///
/// The spatial grid is in log-space: `x = ln(S)`, with uniform spacing.
///
/// Corresponds to a simplified `QuantLib::Fdm1DimSolver`.
pub struct Fdm1dSolver {
    /// Risk-free rate.
    r: Real,
    /// Dividend yield.
    q: Real,
    /// Volatility.
    sigma: Real,
    /// Maturity.
    maturity: Real,
    /// Number of spatial grid points.
    nx: usize,
    /// Number of time steps.
    nt: usize,
    /// Finite difference scheme.
    scheme: FdmScheme,
}

impl Fdm1dSolver {
    /// Create a new 1-D FDM solver.
    ///
    /// # Arguments
    /// * `r` — risk-free rate
    /// * `q` — continuous dividend yield
    /// * `sigma` — volatility
    /// * `maturity` — time to expiry
    /// * `nx` — number of spatial (log-price) grid points
    /// * `nt` — number of time steps
    /// * `scheme` — time-stepping scheme
    pub fn new(
        r: Real,
        q: Real,
        sigma: Real,
        maturity: Real,
        nx: usize,
        nt: usize,
        scheme: FdmScheme,
    ) -> Self {
        Self {
            r,
            q,
            sigma,
            maturity,
            nx,
            nt,
            scheme,
        }
    }

    /// Solve and return the option value at `spot`.
    ///
    /// `payoff` takes a stock price `S` and returns the terminal payoff.
    pub fn price(&self, spot: Real, payoff: &dyn Fn(Real) -> Real) -> Real {
        let sigma2 = self.sigma * self.sigma;
        let dt = self.maturity / self.nt as Real;
        let n = self.nx;

        // Log-space grid: x ∈ [x_min, x_max]
        let x_center = spot.ln();
        let x_range = 4.0 * self.sigma * self.maturity.sqrt(); // ±4σ√T
        let x_min = x_center - x_range;
        let x_max = x_center + x_range;
        let dx = (x_max - x_min) / (n - 1) as Real;

        // Grid values
        let x_grid: Vec<Real> = (0..n).map(|i| x_min + i as Real * dx).collect();
        let s_grid: Vec<Real> = x_grid.iter().map(|&x| x.exp()).collect();

        // Terminal condition
        let mut values: Vec<Real> = s_grid.iter().map(|&s| payoff(s)).collect();

        // Coefficients of the PDE in log-space (constant coefficients):
        // ∂V/∂t + α·∂²V/∂x² + β·∂V/∂x − r·V = 0
        // where α = σ²/2, β = r − q − σ²/2
        let alpha = 0.5 * sigma2;
        let beta = self.r - self.q - 0.5 * sigma2;

        // Build the spatial operator L such that LV ≈ α·V_xx + β·V_x − r·V
        // Using central differences:
        // V_xx ≈ (V[i+1] - 2V[i] + V[i-1]) / dx²
        // V_x  ≈ (V[i+1] - V[i-1]) / (2dx)
        let a = alpha / (dx * dx) - beta / (2.0 * dx); // lower
        let b = -2.0 * alpha / (dx * dx) - self.r; // diag
        let c = alpha / (dx * dx) + beta / (2.0 * dx); // upper

        // Time stepping: V^{n} from V^{n+1}
        // PDE: dV/dt = -L·V  (backward in time)
        // Explicit: V^n = V^{n+1} + dt·L·V^{n+1} = (I + dt·L)·V^{n+1}
        // Implicit: (I - dt·L)·V^n = V^{n+1}
        // CN: (I - 0.5·dt·L)·V^n = (I + 0.5·dt·L)·V^{n+1}
        for _step in 0..self.nt {
            match self.scheme {
                FdmScheme::Explicit => {
                    let mut new_values = values.clone();
                    for i in 1..n - 1 {
                        new_values[i] = values[i]
                            + dt * (a * values[i - 1] + b * values[i] + c * values[i + 1]);
                    }
                    // Boundary conditions: linearity in log-space
                    new_values[0] = 2.0 * new_values[1] - new_values[2];
                    new_values[n - 1] = 2.0 * new_values[n - 2] - new_values[n - 3];
                    values = new_values;
                }
                FdmScheme::Implicit => {
                    let mut op = TridiagonalOperator::new(n);
                    for i in 1..n - 1 {
                        op.lower[i] = -dt * a;
                        op.diag[i] = 1.0 - dt * b;
                        op.upper[i] = -dt * c;
                    }
                    // Boundaries: identity (Dirichlet-like)
                    op.diag[0] = 1.0;
                    op.upper[0] = 0.0;
                    op.diag[n - 1] = 1.0;
                    op.lower[n - 1] = 0.0;
                    values = op.solve(&values);
                }
                FdmScheme::CrankNicolson => {
                    // RHS: (I + 0.5·dt·L) · V^{n+1}
                    let mut rhs = values.clone();
                    for i in 1..n - 1 {
                        rhs[i] = values[i]
                            + 0.5
                                * dt
                                * (a * values[i - 1] + b * values[i] + c * values[i + 1]);
                    }

                    // LHS: (I - 0.5·dt·L) · V^n = rhs
                    let mut op = TridiagonalOperator::new(n);
                    for i in 1..n - 1 {
                        op.lower[i] = -0.5 * dt * a;
                        op.diag[i] = 1.0 - 0.5 * dt * b;
                        op.upper[i] = -0.5 * dt * c;
                    }
                    op.diag[0] = 1.0;
                    op.upper[0] = 0.0;
                    op.diag[n - 1] = 1.0;
                    op.lower[n - 1] = 0.0;
                    values = op.solve(&rhs);
                }
            }
        }

        // Interpolate at `spot`
        let x_spot = spot.ln();
        // Find the grid index
        let idx = ((x_spot - x_min) / dx).floor() as usize;
        let idx = idx.min(n - 2);
        let frac = (x_spot - x_grid[idx]) / dx;
        values[idx] * (1.0 - frac) + values[idx + 1] * frac
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// BS call price for reference: S=100, K=100, r=5%, q=0%, σ=20%, T=1.
    fn bs_call_ref() -> Real {
        use ql_instruments::OptionType;
        use ql_pricingengines::analytic_european_engine::black_scholes_merton;
        let (price, ..) = black_scholes_merton(OptionType::Call, 100.0, 100.0, 0.05, 0.0, 0.20, 1.0);
        price
    }

    #[test]
    fn thomas_algorithm_solves_identity() {
        let mut op = TridiagonalOperator::new(4);
        for i in 0..4 {
            op.diag[i] = 1.0;
        }
        let rhs = vec![1.0, 2.0, 3.0, 4.0];
        let x = op.solve(&rhs);
        for i in 0..4 {
            assert!((x[i] - rhs[i]).abs() < 1e-12);
        }
    }

    #[test]
    fn thomas_algorithm_solves_tridiagonal() {
        // A = [[2, -1, 0], [-1, 2, -1], [0, -1, 2]]
        // x = [1, 2, 3]
        // Ax = [0, 0, 4]
        let mut op = TridiagonalOperator::new(3);
        op.diag = vec![2.0, 2.0, 2.0];
        op.lower = vec![0.0, -1.0, -1.0];
        op.upper = vec![-1.0, -1.0, 0.0];
        let rhs = vec![0.0, 0.0, 4.0];
        let x = op.solve(&rhs);
        assert!((x[0] - 1.0).abs() < 1e-12);
        assert!((x[1] - 2.0).abs() < 1e-12);
        assert!((x[2] - 3.0).abs() < 1e-12);
    }

    #[test]
    fn fdm_cn_european_call_converges_to_bs() {
        let bs = bs_call_ref();
        let solver = Fdm1dSolver::new(0.05, 0.0, 0.20, 1.0, 200, 200, FdmScheme::CrankNicolson);
        let price = solver.price(100.0, &|s| (s - 100.0).max(0.0));
        assert!(
            (price - bs).abs() < 0.20,
            "FDM CN call = {price:.4}, BS = {bs:.4}"
        );
    }

    #[test]
    fn fdm_implicit_european_call_converges_to_bs() {
        let bs = bs_call_ref();
        let solver = Fdm1dSolver::new(0.05, 0.0, 0.20, 1.0, 200, 200, FdmScheme::Implicit);
        let price = solver.price(100.0, &|s| (s - 100.0).max(0.0));
        assert!(
            (price - bs).abs() < 0.30,
            "FDM Implicit call = {price:.4}, BS = {bs:.4}"
        );
    }

    #[test]
    fn fdm_cn_european_put_converges() {
        let solver = Fdm1dSolver::new(0.05, 0.0, 0.20, 1.0, 200, 200, FdmScheme::CrankNicolson);
        let call = solver.price(100.0, &|s| (s - 100.0).max(0.0));
        let put = solver.price(100.0, &|s| (100.0 - s).max(0.0));

        // Put-call parity: C - P = S - K·exp(-rT)
        let parity = call - put;
        let expected = 100.0 - 100.0 * (-0.05_f64).exp();
        assert!(
            (parity - expected).abs() < 0.50,
            "parity: {parity:.4} vs {expected:.4}"
        );
    }
}
