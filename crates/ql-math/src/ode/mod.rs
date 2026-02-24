//! ODE solvers (translates `ql/math/ode/`).
//!
//! Provides an adaptive Runge-Kutta 4th/5th order solver (Dormand-Prince)
//! for ordinary differential equations.

use ql_core::Real;

/// A function `f(t, y) → dy/dt` for an ODE system.
pub trait OdeFunction {
    /// Evaluate the right-hand side of `dy/dt = f(t, y)`.
    fn eval(&self, t: Real, y: &[Real]) -> Vec<Real>;
}

impl<F> OdeFunction for F
where
    F: Fn(Real, &[Real]) -> Vec<Real>,
{
    fn eval(&self, t: Real, y: &[Real]) -> Vec<Real> {
        (self)(t, y)
    }
}

/// Adaptive Runge-Kutta 4(5) ODE solver (Dormand-Prince method).
///
/// Integrates `dy/dt = f(t, y)` from `t0` to `t1` with automatic step-size
/// control to satisfy the specified tolerance.
///
/// Corresponds to `QuantLib::AdaptiveRungeKutta`.
pub struct AdaptiveRungeKutta {
    /// Absolute tolerance.
    pub abs_tol: Real,
    /// Relative tolerance.
    pub rel_tol: Real,
    /// Maximum allowed step size.
    pub max_step: Real,
}

impl AdaptiveRungeKutta {
    /// Create a new adaptive Runge-Kutta solver.
    pub fn new(abs_tol: Real, rel_tol: Real) -> Self {
        Self {
            abs_tol,
            rel_tol,
            max_step: f64::MAX,
        }
    }

    /// Set the maximum step size.
    pub fn with_max_step(mut self, max_step: Real) -> Self {
        self.max_step = max_step;
        self
    }

    /// Integrate `dy/dt = f(t, y)` from `t0` to `t1`, returning the final state `y(t1)`.
    pub fn integrate<F: OdeFunction>(
        &self,
        f: &F,
        t0: Real,
        y0: &[Real],
        t1: Real,
    ) -> Vec<Real> {
        let n = y0.len();
        let mut t = t0;
        let mut y = y0.to_vec();
        let mut h = (t1 - t0) * 0.01; // initial step
        h = h.abs().min(self.max_step);
        if t1 < t0 {
            h = -h;
        }

        let direction = if t1 >= t0 { 1.0 } else { -1.0 };
        let max_steps = 100_000;

        for _ in 0..max_steps {
            if direction * (t - t1) >= 0.0 {
                break;
            }

            // Don't overshoot
            if direction * (t + h - t1) > 0.0 {
                h = t1 - t;
            }

            let (y_new, err) = self.dormand_prince_step(f, t, &y, h);

            // Error estimate
            let mut err_norm = 0.0;
            for i in 0..n {
                let scale = self.abs_tol + self.rel_tol * y[i].abs().max(y_new[i].abs());
                err_norm += (err[i] / scale) * (err[i] / scale);
            }
            err_norm = (err_norm / n as Real).sqrt();

            if err_norm <= 1.0 {
                // Accept step
                t += h;
                y = y_new;

                // Increase step size
                if err_norm > 1e-15 {
                    h *= 0.9 * err_norm.powf(-0.2);
                } else {
                    h *= 2.0;
                }
                h = h.abs().min(self.max_step) * direction;
            } else {
                // Reject step, reduce step size
                h *= 0.9 * err_norm.powf(-0.25);
                h = h.abs().max(1e-15 * (t1 - t0).abs()) * direction;
            }
        }

        y
    }

    /// Single Dormand-Prince step. Returns `(y_new, error_estimate)`.
    fn dormand_prince_step<F: OdeFunction>(
        &self,
        f: &F,
        t: Real,
        y: &[Real],
        h: Real,
    ) -> (Vec<Real>, Vec<Real>) {
        // Dormand-Prince 4(5) Butcher tableau coefficients
        let n = y.len();

        let k1 = f.eval(t, y);

        let y2: Vec<Real> = (0..n).map(|i| y[i] + h * (1.0 / 5.0) * k1[i]).collect();
        let k2 = f.eval(t + h / 5.0, &y2);

        let y3: Vec<Real> = (0..n)
            .map(|i| y[i] + h * (3.0 / 40.0 * k1[i] + 9.0 / 40.0 * k2[i]))
            .collect();
        let k3 = f.eval(t + 3.0 / 10.0 * h, &y3);

        let y4: Vec<Real> = (0..n)
            .map(|i| {
                y[i] + h * (44.0 / 45.0 * k1[i] - 56.0 / 15.0 * k2[i] + 32.0 / 9.0 * k3[i])
            })
            .collect();
        let k4 = f.eval(t + 4.0 / 5.0 * h, &y4);

        let y5: Vec<Real> = (0..n)
            .map(|i| {
                y[i] + h
                    * (19372.0 / 6561.0 * k1[i] - 25360.0 / 2187.0 * k2[i]
                        + 64448.0 / 6561.0 * k3[i]
                        - 212.0 / 729.0 * k4[i])
            })
            .collect();
        let k5 = f.eval(t + 8.0 / 9.0 * h, &y5);

        let y6: Vec<Real> = (0..n)
            .map(|i| {
                y[i] + h
                    * (9017.0 / 3168.0 * k1[i] - 355.0 / 33.0 * k2[i]
                        + 46732.0 / 5247.0 * k3[i]
                        + 49.0 / 176.0 * k4[i]
                        - 5103.0 / 18656.0 * k5[i])
            })
            .collect();
        let k6 = f.eval(t + h, &y6);

        // 5th order solution (for advancing)
        let y_new: Vec<Real> = (0..n)
            .map(|i| {
                y[i] + h
                    * (35.0 / 384.0 * k1[i]
                        + 500.0 / 1113.0 * k3[i]
                        + 125.0 / 192.0 * k4[i]
                        - 2187.0 / 6784.0 * k5[i]
                        + 11.0 / 84.0 * k6[i])
            })
            .collect();

        // 7th stage (FSAL — evaluated at the new point)
        let k7 = f.eval(t + h, &y_new);

        // Error = 5th order - 4th order (using the standard DP error coefficients)
        // e_i = h * (71/57600*k1 - 71/16695*k3 + 71/1920*k4 - 17253/339200*k5 + 22/525*k6 - 1/40*k7)
        let err: Vec<Real> = (0..n)
            .map(|i| {
                h * (71.0 / 57600.0 * k1[i]
                    - 71.0 / 16695.0 * k3[i]
                    + 71.0 / 1920.0 * k4[i]
                    - 17253.0 / 339200.0 * k5[i]
                    + 22.0 / 525.0 * k6[i]
                    - 1.0 / 40.0 * k7[i])
            })
            .collect();

        (y_new, err)
    }
}

impl Default for AdaptiveRungeKutta {
    fn default() -> Self {
        Self::new(1e-10, 1e-10)
    }
}

/// Convenience function: integrate a scalar ODE `dy/dt = f(t, y)`.
pub fn integrate_scalar<F>(
    f: F,
    t0: Real,
    y0: Real,
    t1: Real,
    tol: Real,
) -> Real
where
    F: Fn(Real, Real) -> Real,
{
    let wrapper = |t: Real, y: &[Real]| -> Vec<Real> { vec![f(t, y[0])] };
    let solver = AdaptiveRungeKutta::new(tol, tol);
    let result = solver.integrate(&wrapper, t0, &[y0], t1);
    result[0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exponential_growth() {
        // dy/dt = y, y(0) = 1 → y(t) = e^t
        let result = integrate_scalar(|_t, y| y, 0.0, 1.0, 1.0, 1e-10);
        assert!(
            (result - std::f64::consts::E).abs() < 1e-6,
            "got {result}, expected e ≈ {}", std::f64::consts::E
        );
    }

    #[test]
    fn exponential_decay() {
        // dy/dt = -y, y(0) = 1 → y(1) = e^{-1}
        let result = integrate_scalar(|_t, y| -y, 0.0, 1.0, 1.0, 1e-10);
        let expected = (-1.0_f64).exp();
        assert!(
            (result - expected).abs() < 1e-6,
            "got {result}, expected {expected}"
        );
    }

    #[test]
    fn sine_cosine_system() {
        // dy₁/dt = y₂, dy₂/dt = -y₁
        // y₁(0) = 0, y₂(0) = 1
        // Solution: y₁(t) = sin(t), y₂(t) = cos(t)
        let f = |_t: Real, y: &[Real]| -> Vec<Real> { vec![y[1], -y[0]] };
        let solver = AdaptiveRungeKutta::new(1e-10, 1e-10);
        let t_end = std::f64::consts::PI;
        let result = solver.integrate(&f, 0.0, &[0.0, 1.0], t_end);

        // y₁(π) = sin(π) ≈ 0, y₂(π) = cos(π) ≈ -1
        assert!(
            result[0].abs() < 1e-5,
            "y1(π) = {}, expected ~0",
            result[0]
        );
        assert!(
            (result[1] + 1.0).abs() < 1e-5,
            "y2(π) = {}, expected ~-1",
            result[1]
        );
    }

    #[test]
    fn logistic_growth() {
        // dy/dt = y(1-y), y(0) = 0.1 → y(t) = 1 / (1 + 9·e^{-t})
        let result = integrate_scalar(|_t, y| y * (1.0 - y), 0.0, 0.1, 5.0, 1e-10);
        let expected = 1.0 / (1.0 + 9.0 * (-5.0_f64).exp());
        assert!(
            (result - expected).abs() < 1e-5,
            "got {result}, expected {expected}"
        );
    }

    #[test]
    fn backward_integration() {
        // Integrate from t=1 to t=0: dy/dt = y → going backward
        // y(1) = e → y(0) = 1
        let result = integrate_scalar(|_t, y| y, 1.0, std::f64::consts::E, 0.0, 1e-10);
        assert!(
            (result - 1.0).abs() < 1e-5,
            "got {result}, expected 1.0"
        );
    }
}
