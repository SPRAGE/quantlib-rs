//! 1D root-finding solvers (translates `ql/math/solvers1d/`).

use ql_core::{
    errors::{Error, Result},
    Real,
};

const MAX_ITERATIONS: u32 = 100;
const DEFAULT_ACCURACY: Real = 1.0e-11;

// ── Brent ─────────────────────────────────────────────────────────────────────

/// Brent's method for finding a root of `f(x)` in `[x_min, x_max]`.
///
/// Combines bisection, secant, and inverse quadratic interpolation.
pub fn brent<F>(f: F, x_min: Real, x_max: Real, accuracy: Real) -> Result<Real>
where
    F: Fn(Real) -> Real,
{
    let acc = if accuracy > 0.0 {
        accuracy
    } else {
        DEFAULT_ACCURACY
    };
    let mut a = x_min;
    let mut b = x_max;
    let mut fa = f(a);
    let mut fb = f(b);

    if fa * fb > 0.0 {
        return Err(Error::Precondition(format!(
            "Brent: f({a}) and f({b}) must have opposite signs"
        )));
    }
    if fa == 0.0 {
        return Ok(a);
    }
    if fb == 0.0 {
        return Ok(b);
    }

    let mut c = b;
    let mut fc = fb;
    let mut d = b - a;
    let mut e = d;

    for _ in 0..MAX_ITERATIONS {
        if fb * fc > 0.0 {
            c = a;
            fc = fa;
            d = b - a;
            e = d;
        }
        if fc.abs() < fb.abs() {
            a = b;
            b = c;
            c = a;
            fa = fb;
            fb = fc;
            fc = fa;
        }
        let tol = 2.0 * f64::EPSILON * b.abs() + 0.5 * acc;
        let xm = 0.5 * (c - b);
        if xm.abs() <= tol || fb == 0.0 {
            return Ok(b);
        }
        if e.abs() >= tol && fa.abs() > fb.abs() {
            let s = fb / fa;
            let (p, q) = if a == c {
                let p = 2.0 * xm * s;
                let q = 1.0 - s;
                (p, q)
            } else {
                let q = fa / fc;
                let r = fb / fc;
                let p = s * (2.0 * xm * q * (q - r) - (b - a) * (r - 1.0));
                let q = (q - 1.0) * (r - 1.0) * (s - 1.0);
                (p, q)
            };
            let (p, q) = if p > 0.0 { (p, -q) } else { (-p, q) };
            if 2.0 * p < (3.0 * xm * q - (tol * q).abs()) && 2.0 * p < (e * q).abs() {
                e = d;
                d = p / q;
            } else {
                d = xm;
                e = d;
            }
        } else {
            d = xm;
            e = d;
        }
        a = b;
        fa = fb;
        b += if d.abs() > tol {
            d
        } else if xm > 0.0 {
            tol
        } else {
            -tol
        };
        fb = f(b);
    }
    Err(Error::Runtime(
        "Brent solver: maximum iterations reached".into(),
    ))
}

// ── Bisection ────────────────────────────────────────────────────────────────

/// Simple bisection method.
pub fn bisection<F>(f: F, x_min: Real, x_max: Real, accuracy: Real) -> Result<Real>
where
    F: Fn(Real) -> Real,
{
    let acc = if accuracy > 0.0 {
        accuracy
    } else {
        DEFAULT_ACCURACY
    };
    let mut a = x_min;
    let mut b = x_max;
    let fa = f(a);
    let fb = f(b);

    if fa * fb > 0.0 {
        return Err(Error::Precondition(
            "Bisection: f(x_min) and f(x_max) must have opposite signs".into(),
        ));
    }
    if fa == 0.0 {
        return Ok(a);
    }
    if fb == 0.0 {
        return Ok(b);
    }

    for _ in 0..MAX_ITERATIONS {
        let mid = 0.5 * (a + b);
        let fm = f(mid);
        if fm == 0.0 || (b - a) * 0.5 < acc {
            return Ok(mid);
        }
        if fm * fa > 0.0 {
            a = mid;
        } else {
            b = mid;
        }
    }
    Err(Error::Runtime(
        "Bisection solver: maximum iterations reached".into(),
    ))
}

// ── Newton-Raphson ────────────────────────────────────────────────────────────

/// Newton-Raphson method using function value and its derivative.
///
/// Falls back to bisection when the Newton step would leave the bracket.
pub fn newton<F>(f_df: F, x0: Real, x_min: Real, x_max: Real, accuracy: Real) -> Result<Real>
where
    F: Fn(Real) -> (Real, Real),
{
    let acc = if accuracy > 0.0 {
        accuracy
    } else {
        DEFAULT_ACCURACY
    };
    let mut x = x0.clamp(x_min, x_max);

    for _ in 0..MAX_ITERATIONS {
        let (fx, dfx) = f_df(x);
        if fx.abs() < acc {
            return Ok(x);
        }
        if dfx.abs() > f64::EPSILON {
            let step = fx / dfx;
            let x_new = x - step;
            if x_new >= x_min && x_new <= x_max {
                x = x_new;
                continue;
            }
        }
        // Fall back: bisection step
        let dx = 0.5 * (x_max - x_min);
        x = x_min + dx;
        if dx < acc {
            return Ok(x);
        }
    }
    Err(Error::Runtime(
        "Newton solver: maximum iterations reached".into(),
    ))
}

// ── Secant ────────────────────────────────────────────────────────────────────

/// Secant method for root finding.
///
/// Uses two initial points `x_min` and `x_max` and iteratively refines.
/// Corresponds to `QuantLib::Secant`.
pub fn secant<F>(f: F, x_min: Real, x_max: Real, accuracy: Real) -> Result<Real>
where
    F: Fn(Real) -> Real,
{
    let acc = if accuracy > 0.0 {
        accuracy
    } else {
        DEFAULT_ACCURACY
    };
    let mut x0 = x_min;
    let mut x1 = x_max;
    let mut f0 = f(x0);
    let mut f1 = f(x1);

    if f0.abs() < acc {
        return Ok(x0);
    }
    if f1.abs() < acc {
        return Ok(x1);
    }

    for _ in 0..MAX_ITERATIONS {
        let denom = f1 - f0;
        if denom.abs() < f64::EPSILON {
            return Err(Error::Runtime(
                "Secant: derivative vanishes (f(x0) ≈ f(x1))".into(),
            ));
        }
        let x2 = x1 - f1 * (x1 - x0) / denom;
        let f2 = f(x2);

        if f2.abs() < acc || (x2 - x1).abs() < acc {
            return Ok(x2);
        }

        x0 = x1;
        f0 = f1;
        x1 = x2;
        f1 = f2;
    }

    Err(Error::Runtime(
        "Secant solver: maximum iterations reached".into(),
    ))
}

// ── Ridder ────────────────────────────────────────────────────────────────────

/// Ridder's method for root finding.
///
/// Requires that `f(x_min)` and `f(x_max)` have opposite signs.
/// Corresponds to `QuantLib::Ridder`.
pub fn ridder<F>(f: F, x_min: Real, x_max: Real, accuracy: Real) -> Result<Real>
where
    F: Fn(Real) -> Real,
{
    let acc = if accuracy > 0.0 {
        accuracy
    } else {
        DEFAULT_ACCURACY
    };
    let mut a = x_min;
    let mut b = x_max;
    let mut fa = f(a);
    let mut fb = f(b);

    if fa * fb > 0.0 {
        return Err(Error::Precondition(
            "Ridder: f(x_min) and f(x_max) must have opposite signs".into(),
        ));
    }
    if fa == 0.0 {
        return Ok(a);
    }
    if fb == 0.0 {
        return Ok(b);
    }

    for _ in 0..MAX_ITERATIONS {
        let mid = 0.5 * (a + b);
        let fm = f(mid);

        let s = (fm * fm - fa * fb).sqrt();
        if s == 0.0 {
            return Ok(mid);
        }

        let sign = if fa < fb { 1.0 } else { -1.0 };
        let x_new = mid + (mid - a) * sign * fm / s;
        let f_new = f(x_new);

        if f_new.abs() < acc || (b - a).abs() < acc {
            return Ok(x_new);
        }

        // Bisect: update bracket
        if fm * f_new < 0.0 {
            a = mid;
            fa = fm;
            b = x_new;
            fb = f_new;
        } else if fa * f_new < 0.0 {
            b = x_new;
            fb = f_new;
        } else {
            a = x_new;
            fa = f_new;
        }

        if (b - a).abs() < acc {
            return Ok(0.5 * (a + b));
        }
    }

    Err(Error::Runtime(
        "Ridder solver: maximum iterations reached".into(),
    ))
}

// ── False Position ────────────────────────────────────────────────────────────

/// False position (regula falsi) method.
///
/// Requires that `f(x_min)` and `f(x_max)` have opposite signs.
/// Corresponds to `QuantLib::FalsePosition`.
pub fn false_position<F>(f: F, x_min: Real, x_max: Real, accuracy: Real) -> Result<Real>
where
    F: Fn(Real) -> Real,
{
    let acc = if accuracy > 0.0 {
        accuracy
    } else {
        DEFAULT_ACCURACY
    };
    let mut a = x_min;
    let mut b = x_max;
    let fa = f(a);
    let fb_val = f(b);

    if fa * fb_val > 0.0 {
        return Err(Error::Precondition(
            "FalsePosition: f(x_min) and f(x_max) must have opposite signs".into(),
        ));
    }

    // Ensure fa < 0 and fb > 0
    if fa > 0.0 {
        std::mem::swap(&mut a, &mut b);
    }

    for _ in 0..MAX_ITERATIONS {
        let fa_val = f(a);
        let fb_val = f(b);
        let denom = fb_val - fa_val;
        if denom.abs() < f64::EPSILON {
            return Ok(0.5 * (a + b));
        }
        let c = a - fa_val * (b - a) / denom;
        let fc = f(c);

        if fc.abs() < acc || (b - a).abs() < acc {
            return Ok(c);
        }

        if fc < 0.0 {
            a = c;
        } else {
            b = c;
        }
    }

    Err(Error::Runtime(
        "FalsePosition solver: maximum iterations reached".into(),
    ))
}

// ── Newton-Safe ──────────────────────────────────────────────────────────────

/// A safe Newton-Raphson method that falls back to bisection when the Newton
/// step would leave the bracket `[x_min, x_max]`.
///
/// Requires the function and its derivative, plus a bracket `[x_min, x_max]`
/// where `f(x_min)` and `f(x_max)` have opposite signs.
///
/// Corresponds to `QuantLib::NewtonSafe`.
pub fn newton_safe<F>(f_df: F, x_min: Real, x_max: Real, accuracy: Real) -> Result<Real>
where
    F: Fn(Real) -> (Real, Real),
{
    let acc = if accuracy > 0.0 {
        accuracy
    } else {
        DEFAULT_ACCURACY
    };
    let (flo, _) = f_df(x_min);
    let (fhi, _) = f_df(x_max);

    if flo * fhi > 0.0 {
        return Err(Error::Precondition(
            "NewtonSafe: f(x_min) and f(x_max) must have opposite signs".into(),
        ));
    }

    // Orient so that f(xl) < 0
    let (mut xl, mut xh) = if flo < 0.0 {
        (x_min, x_max)
    } else {
        (x_max, x_min)
    };

    let mut x = 0.5 * (xl + xh);
    let mut dx_old = (xh - xl).abs();
    let mut dx = dx_old;

    let (mut fx, mut dfx) = f_df(x);

    for _ in 0..MAX_ITERATIONS {
        // Use Newton step if it stays in bracket and is converging
        let newton_out_of_range = ((x - xh) * dfx - fx) * ((x - xl) * dfx - fx) > 0.0;
        let bisection_faster = (2.0 * fx).abs() > (dx_old * dfx).abs();

        if newton_out_of_range || bisection_faster {
            // Bisection step
            dx_old = dx;
            dx = 0.5 * (xh - xl);
            x = xl + dx;
        } else {
            // Newton step
            dx_old = dx;
            dx = fx / dfx;
            x -= dx;
        }

        if dx.abs() < acc {
            return Ok(x);
        }

        let result = f_df(x);
        fx = result.0;
        dfx = result.1;

        if fx.abs() < acc {
            return Ok(x);
        }

        if fx < 0.0 {
            xl = x;
        } else {
            xh = x;
        }
    }

    Err(Error::Runtime(
        "NewtonSafe solver: maximum iterations reached".into(),
    ))
}

// ── Finite-Difference Newton-Safe ─────────────────────────────────────────────

/// A safe Newton-Raphson method that estimates the derivative via finite
/// differences from the farthest bracket point, falling back to bisection when
/// the Newton step would leave the bracket.
///
/// Unlike [`newton_safe`], this only requires `f(x)`, not `f(x)` **and**
/// `f'(x)`, making it ideal when the derivative is unavailable or expensive.
///
/// Corresponds to `QuantLib::FiniteDifferenceNewtonSafe`.
pub fn fd_newton_safe<F>(f: F, x_min: Real, x_max: Real, accuracy: Real) -> Result<Real>
where
    F: Fn(Real) -> Real,
{
    let acc = if accuracy > 0.0 {
        accuracy
    } else {
        DEFAULT_ACCURACY
    };
    let flo = f(x_min);
    let fhi = f(x_max);

    if flo * fhi > 0.0 {
        return Err(Error::Precondition(
            "FDNewtonSafe: f(x_min) and f(x_max) must have opposite signs".into(),
        ));
    }
    if flo == 0.0 {
        return Ok(x_min);
    }
    if fhi == 0.0 {
        return Ok(x_max);
    }

    // Orient so that f(xl) < 0
    let (mut xl, mut xh) = if flo < 0.0 {
        (x_min, x_max)
    } else {
        (x_max, x_min)
    };

    let mut x = 0.5 * (xl + xh);
    let mut dx_old = (xh - xl).abs();
    let mut dx = dx_old;

    let mut fx = f(x);

    for _ in 0..MAX_ITERATIONS {
        // Estimate derivative using finite difference from the farthest
        // bracket endpoint (this maximises the base, improving stability).
        let (x_far, f_far) = if (x - xl).abs() >= (x - xh).abs() {
            (xl, f(xl))
        } else {
            (xh, f(xh))
        };

        let dfx = if (x - x_far).abs() > f64::EPSILON {
            (fx - f_far) / (x - x_far)
        } else {
            // Fall back to bisection if points coincide
            0.0
        };

        // Decide: Newton or bisection
        let newton_step_valid = dfx.abs() > f64::EPSILON;
        let newton_x = if newton_step_valid {
            x - fx / dfx
        } else {
            f64::NAN
        };

        let use_newton = newton_step_valid
            && newton_x > xl
            && newton_x < xh
            && (2.0 * fx.abs()) <= (dx_old * dfx).abs();

        if use_newton {
            dx_old = dx;
            dx = fx / dfx;
            x -= dx;
        } else {
            // Bisection step
            dx_old = dx;
            dx = 0.5 * (xh - xl);
            x = xl + dx;
        }

        if dx.abs() < acc {
            return Ok(x);
        }

        fx = f(x);

        if fx.abs() < acc {
            return Ok(x);
        }

        if fx < 0.0 {
            xl = x;
        } else {
            xh = x;
        }
    }

    Err(Error::Runtime(
        "FDNewtonSafe solver: maximum iterations reached".into(),
    ))
}

// ── Halley ────────────────────────────────────────────────────────────────────

/// Halley's method — uses function value, derivative, and second derivative.
///
/// Convergence is cubic: each step roughly triples correct digits.
///
/// Corresponds to `QuantLib::Halley`.
pub fn halley<F>(f_df_d2f: F, x0: Real, x_min: Real, x_max: Real, accuracy: Real) -> Result<Real>
where
    F: Fn(Real) -> (Real, Real, Real),
{
    let acc = if accuracy > 0.0 {
        accuracy
    } else {
        DEFAULT_ACCURACY
    };
    let mut x = x0.clamp(x_min, x_max);

    for _ in 0..MAX_ITERATIONS {
        let (fx, dfx, d2fx) = f_df_d2f(x);
        if fx.abs() < acc {
            return Ok(x);
        }
        if dfx.abs() < f64::EPSILON {
            return Err(Error::Runtime("Halley: derivative is zero".into()));
        }

        let denom = 2.0 * dfx * dfx - fx * d2fx;
        if denom.abs() < f64::EPSILON {
            // Fall back to Newton step
            let x_new = x - fx / dfx;
            x = x_new.clamp(x_min, x_max);
        } else {
            let step = 2.0 * fx * dfx / denom;
            let x_new = x - step;
            x = x_new.clamp(x_min, x_max);
        }
    }

    Err(Error::Runtime(
        "Halley solver: maximum iterations reached".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brent_sqrt2() {
        let root = brent(|x| x * x - 2.0, 0.0, 2.0, 1e-12).unwrap();
        assert!((root - 2.0_f64.sqrt()).abs() < 1e-10);
    }

    #[test]
    fn bisection_sqrt2() {
        let root = bisection(|x| x * x - 2.0, 0.0, 2.0, 1e-12).unwrap();
        assert!((root - 2.0_f64.sqrt()).abs() < 1e-9);
    }

    #[test]
    fn newton_sqrt2() {
        let root = newton(|x| (x * x - 2.0, 2.0 * x), 1.5, 0.0, 2.0, 1e-12).unwrap();
        assert!((root - 2.0_f64.sqrt()).abs() < 1e-10);
    }

    #[test]
    fn brent_opposite_signs_required() {
        assert!(brent(|x| x, 1.0, 2.0, 1e-10).is_err());
    }

    #[test]
    fn secant_sqrt2() {
        let root = secant(|x| x * x - 2.0, 1.0, 2.0, 1e-12).unwrap();
        assert!((root - 2.0_f64.sqrt()).abs() < 1e-10, "got {root}");
    }

    #[test]
    fn ridder_sqrt2() {
        let root = ridder(|x| x * x - 2.0, 0.0, 2.0, 1e-12).unwrap();
        assert!((root - 2.0_f64.sqrt()).abs() < 1e-10, "got {root}");
    }

    #[test]
    fn false_position_sqrt2() {
        let root = false_position(|x| x * x - 2.0, 0.0, 2.0, 1e-12).unwrap();
        assert!((root - 2.0_f64.sqrt()).abs() < 1e-8, "got {root}");
    }

    #[test]
    fn newton_safe_sqrt2() {
        let root = newton_safe(|x| (x * x - 2.0, 2.0 * x), 0.0, 2.0, 1e-12).unwrap();
        assert!((root - 2.0_f64.sqrt()).abs() < 1e-10, "got {root}");
    }

    #[test]
    fn halley_cube_root_27() {
        // f(x) = x^3 - 27, f'(x) = 3x^2, f''(x) = 6x
        let root = halley(
            |x| (x * x * x - 27.0, 3.0 * x * x, 6.0 * x),
            2.0,
            0.0,
            10.0,
            1e-12,
        )
        .unwrap();
        assert!((root - 3.0).abs() < 1e-10, "got {root}");
    }

    #[test]
    fn newton_safe_opposite_signs_required() {
        assert!(newton_safe(|x| (x * x - 2.0, 2.0 * x), 3.0, 5.0, 1e-10).is_err());
    }

    #[test]
    fn fd_newton_safe_sqrt2() {
        let root = fd_newton_safe(|x| x * x - 2.0, 0.0, 2.0, 1e-12).unwrap();
        assert!((root - 2.0_f64.sqrt()).abs() < 1e-8, "got {root}");
    }

    #[test]
    fn fd_newton_safe_sin() {
        let root = fd_newton_safe(|x| x.sin(), 2.0, 4.0, 1e-12).unwrap();
        assert!((root - std::f64::consts::PI).abs() < 1e-8, "got {root}");
    }
}
