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
    let acc = if accuracy > 0.0 { accuracy } else { DEFAULT_ACCURACY };
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
        b += if d.abs() > tol { d } else if xm > 0.0 { tol } else { -tol };
        fb = f(b);
    }
    Err(Error::Runtime("Brent solver: maximum iterations reached".into()))
}

// ── Bisection ────────────────────────────────────────────────────────────────

/// Simple bisection method.
pub fn bisection<F>(f: F, x_min: Real, x_max: Real, accuracy: Real) -> Result<Real>
where
    F: Fn(Real) -> Real,
{
    let acc = if accuracy > 0.0 { accuracy } else { DEFAULT_ACCURACY };
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
    let acc = if accuracy > 0.0 { accuracy } else { DEFAULT_ACCURACY };
    let mut x = x0.clamp(x_min, x_max);
    let mut dx = x_max - x_min;

    for _ in 0..MAX_ITERATIONS {
        let (fx, dfx) = f_df(x);
        if fx.abs() < acc {
            return Ok(x);
        }
        if dfx.abs() > f64::EPSILON {
            let step = fx / dfx;
            let x_new = x - step;
            if x_new >= x_min && x_new <= x_max {
                dx = step.abs();
                x = x_new;
                continue;
            }
        }
        // Fall back: bisection step
        dx = 0.5 * (x_max - x_min);
        x = x_min + dx;
        if dx < acc {
            return Ok(x);
        }
    }
    Err(Error::Runtime(
        "Newton solver: maximum iterations reached".into(),
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
}
