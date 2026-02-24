//! `SmileSection` — abstract interface for volatility smile sections.
//!
//! A smile section represents the implied volatility smile at a single expiry
//! as a function of strike.  The base trait provides default implementations
//! for option prices, digital prices, vega and density via Black or
//! Bachelier formulas.
//!
//! Corresponds to `QuantLib::SmileSection`.

use ql_core::{Real, Time, Volatility};

/// Volatility type indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolatilityType {
    /// Shifted log-normal (Black) volatility.
    ShiftedLognormal,
    /// Normal (Bachelier) volatility.
    Normal,
}

/// Option type for smile section pricing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmileOptionType {
    /// Call option.
    Call,
    /// Put option.
    Put,
}

/// A volatility smile at a single expiry.
///
/// Provides implied volatility as a function of strike, plus convenience
/// methods for option pricing, vega, and density.
///
/// Corresponds to `QuantLib::SmileSection`.
pub trait SmileSection: std::fmt::Debug + Send + Sync {
    // ── Required methods ──────────────────────────────────────────────────

    /// Minimum valid strike for this smile section.
    fn min_strike(&self) -> Real;

    /// Maximum valid strike for this smile section.
    fn max_strike(&self) -> Real;

    /// ATM level (forward price).
    fn atm_level(&self) -> Real;

    /// Implied volatility at a given strike.
    ///
    /// For ShiftedLognormal type, this is σ_B(K).
    /// For Normal type, this is σ_N(K).
    fn volatility_impl(&self, strike: Real) -> Volatility;

    // ── Provided accessors ────────────────────────────────────────────────

    /// Time to expiry in years.
    fn exercise_time(&self) -> Time;

    /// Volatility type (default: ShiftedLognormal).
    fn volatility_type(&self) -> VolatilityType {
        VolatilityType::ShiftedLognormal
    }

    /// Shift for shifted log-normal (default: 0).
    fn shift(&self) -> Real {
        0.0
    }

    // ── Derived methods ───────────────────────────────────────────────────

    /// Implied volatility at a given strike (public access).
    fn volatility(&self, strike: Real) -> Volatility {
        self.volatility_impl(strike)
    }

    /// Total variance σ²·T at a given strike.
    fn variance(&self, strike: Real) -> Real {
        let vol = self.volatility_impl(strike);
        vol * vol * self.exercise_time()
    }

    /// Option price using Black formula (ShiftedLognormal).
    ///
    /// # Arguments
    /// * `strike` — option strike
    /// * `option_type` — Call or Put
    /// * `discount` — discount factor to expiry
    fn option_price(
        &self,
        strike: Real,
        option_type: SmileOptionType,
        discount: Real,
    ) -> Real {
        let f = self.atm_level();
        let t = self.exercise_time();
        let vol = self.volatility_impl(strike);
        let shift_val = self.shift();

        match self.volatility_type() {
            VolatilityType::ShiftedLognormal => {
                black_formula(f + shift_val, strike + shift_val, vol, t, discount, option_type)
            }
            VolatilityType::Normal => {
                bachelier_formula(f, strike, vol, t, discount, option_type)
            }
        }
    }

    /// Digital option price via finite difference of option prices.
    fn digital_option_price(
        &self,
        strike: Real,
        option_type: SmileOptionType,
        discount: Real,
        gap: Real,
    ) -> Real {
        let kl = strike - gap / 2.0;
        let kr = strike + gap / 2.0;
        let pl = self.option_price(kl, option_type, discount);
        let pr = self.option_price(kr, option_type, discount);
        match option_type {
            SmileOptionType::Call => (pl - pr) / gap,
            SmileOptionType::Put => (pr - pl) / gap,
        }
    }

    /// Density derived from second finite difference of digital option prices.
    fn density(&self, strike: Real, discount: Real, gap: Real) -> Real {
        let kl = strike - gap / 2.0;
        let kr = strike + gap / 2.0;
        let dl = self.digital_option_price(kl, SmileOptionType::Call, discount, gap);
        let dr = self.digital_option_price(kr, SmileOptionType::Call, discount, gap);
        (dl - dr) / gap
    }

    /// Vega (vol sensitivity) via Black formula.
    fn vega(&self, strike: Real, discount: Real) -> Real {
        let f = self.atm_level();
        let t = self.exercise_time();
        let vol = self.volatility_impl(strike);
        let shift_val = self.shift();

        match self.volatility_type() {
            VolatilityType::ShiftedLognormal => {
                let std_dev = vol * t.sqrt();
                if std_dev < 1e-15 {
                    return 0.0;
                }
                let fwd = f + shift_val;
                let k = strike + shift_val;
                let d1 = ((fwd / k).ln() + 0.5 * std_dev * std_dev) / std_dev;
                discount * fwd * normal_pdf(d1) * t.sqrt() * 0.01
            }
            VolatilityType::Normal => {
                discount * normal_pdf((f - strike) / (vol * t.sqrt())) * t.sqrt() * 0.01
            }
        }
    }
}

// ── Black and Bachelier formulas ──────────────────────────────────────────────

use ql_math::distributions::{normal_cdf, normal_pdf};

/// Black formula for a call or put.
fn black_formula(
    forward: Real,
    strike: Real,
    vol: Real,
    t: Real,
    discount: Real,
    option_type: SmileOptionType,
) -> Real {
    if vol <= 0.0 || t <= 0.0 {
        let intrinsic = match option_type {
            SmileOptionType::Call => (forward - strike).max(0.0),
            SmileOptionType::Put => (strike - forward).max(0.0),
        };
        return discount * intrinsic;
    }
    let std_dev = vol * t.sqrt();
    if forward <= 0.0 || strike <= 0.0 {
        let intrinsic = match option_type {
            SmileOptionType::Call => (forward - strike).max(0.0),
            SmileOptionType::Put => (strike - forward).max(0.0),
        };
        return discount * intrinsic;
    }
    let d1 = ((forward / strike).ln() + 0.5 * std_dev * std_dev) / std_dev;
    let d2 = d1 - std_dev;
    match option_type {
        SmileOptionType::Call => discount * (forward * normal_cdf(d1) - strike * normal_cdf(d2)),
        SmileOptionType::Put => discount * (strike * normal_cdf(-d2) - forward * normal_cdf(-d1)),
    }
}

/// Bachelier (normal) formula for a call or put.
fn bachelier_formula(
    forward: Real,
    strike: Real,
    vol: Real,
    t: Real,
    discount: Real,
    option_type: SmileOptionType,
) -> Real {
    if vol <= 0.0 || t <= 0.0 {
        let intrinsic = match option_type {
            SmileOptionType::Call => (forward - strike).max(0.0),
            SmileOptionType::Put => (strike - forward).max(0.0),
        };
        return discount * intrinsic;
    }
    let std_dev = vol * t.sqrt();
    let d = (forward - strike) / std_dev;
    match option_type {
        SmileOptionType::Call => discount * (std_dev * normal_pdf(d) + (forward - strike) * normal_cdf(d)),
        SmileOptionType::Put => discount * (std_dev * normal_pdf(d) - (forward - strike) * normal_cdf(-d)),
    }
}

// ── FlatSmileSection ──────────────────────────────────────────────────────────

/// A constant-volatility smile section.
///
/// `σ(K) = constant` for all strikes K.
///
/// Corresponds to `QuantLib::FlatSmileSection`.
#[derive(Debug, Clone)]
pub struct FlatSmileSection {
    exercise_time: Time,
    vol: Volatility,
    atm_level: Real,
    vol_type: VolatilityType,
    shift: Real,
}

impl FlatSmileSection {
    /// Create a flat smile section.
    pub fn new(exercise_time: Time, vol: Volatility, atm_level: Real) -> Self {
        Self {
            exercise_time,
            vol,
            atm_level,
            vol_type: VolatilityType::ShiftedLognormal,
            shift: 0.0,
        }
    }

    /// Create with volatility type and shift.
    pub fn with_type(mut self, vol_type: VolatilityType, shift: Real) -> Self {
        self.vol_type = vol_type;
        self.shift = shift;
        self
    }
}

impl SmileSection for FlatSmileSection {
    fn min_strike(&self) -> Real {
        f64::NEG_INFINITY
    }

    fn max_strike(&self) -> Real {
        f64::INFINITY
    }

    fn atm_level(&self) -> Real {
        self.atm_level
    }

    fn volatility_impl(&self, _strike: Real) -> Volatility {
        self.vol
    }

    fn exercise_time(&self) -> Time {
        self.exercise_time
    }

    fn volatility_type(&self) -> VolatilityType {
        self.vol_type
    }

    fn shift(&self) -> Real {
        self.shift
    }
}

// ── SabrSmileSection ──────────────────────────────────────────────────────────

use ql_math::interpolations::sabr::{sabr_volatility, SabrParameters};

/// A SABR-based smile section.
///
/// Wraps SABR parameters and a forward price to produce implied vols via the
/// Hagan et al. (2002) formula.
///
/// Corresponds to `QuantLib::SabrSmileSection`.
#[derive(Debug, Clone)]
pub struct SabrSmileSection {
    exercise_time: Time,
    forward: Real,
    params: SabrParameters,
    shift: Real,
    vol_type: VolatilityType,
}

impl SabrSmileSection {
    /// Create a new SABR smile section.
    pub fn new(exercise_time: Time, forward: Real, params: SabrParameters) -> Self {
        assert!(params.alpha > 0.0, "alpha must be positive");
        assert!(
            (0.0..=1.0).contains(&params.beta),
            "beta must be in [0, 1]"
        );
        assert!(params.nu >= 0.0, "nu must be non-negative");
        assert!(
            params.rho * params.rho < 1.0,
            "rho must satisfy |rho| < 1"
        );
        Self {
            exercise_time,
            forward,
            params,
            shift: 0.0,
            vol_type: VolatilityType::ShiftedLognormal,
        }
    }

    /// Create with shift and volatility type.
    pub fn with_shift(mut self, shift: Real, vol_type: VolatilityType) -> Self {
        self.shift = shift;
        self.vol_type = vol_type;
        self
    }

    /// The SABR model parameters.
    pub fn params(&self) -> &SabrParameters {
        &self.params
    }

    /// The forward rate.
    pub fn forward(&self) -> Real {
        self.forward
    }
}

impl SmileSection for SabrSmileSection {
    fn min_strike(&self) -> Real {
        -self.shift
    }

    fn max_strike(&self) -> Real {
        f64::INFINITY
    }

    fn atm_level(&self) -> Real {
        self.forward
    }

    fn volatility_impl(&self, strike: Real) -> Volatility {
        let k = strike.max(1e-5 - self.shift);
        sabr_volatility(self.forward, k, self.exercise_time, &self.params)
    }

    fn exercise_time(&self) -> Time {
        self.exercise_time
    }

    fn volatility_type(&self) -> VolatilityType {
        self.vol_type
    }

    fn shift(&self) -> Real {
        self.shift
    }
}

// ── SviSmileSection ───────────────────────────────────────────────────────────

/// SVI (Stochastic Volatility Inspired) parameters.
///
/// The SVI parameterization of total variance is:
///
/// $w(k) = a + b \bigl(\rho \cdot (k - m) + \sqrt{(k - m)^2 + \sigma^2}\bigr)$
///
/// where $k = \ln(K / F)$ is log-moneyness.
///
/// Reference: Gatheral (2004).
#[derive(Debug, Clone, Copy)]
pub struct SviParameters {
    /// Level parameter.
    pub a: Real,
    /// Slope parameter (must be ≥ 0).
    pub b: Real,
    /// Smoothness parameter (must be > 0).
    pub sigma: Real,
    /// Tilt parameter (|ρ| < 1).
    pub rho: Real,
    /// Translation parameter.
    pub m: Real,
}

impl SviParameters {
    /// Validate the SVI parameters.
    ///
    /// QuantLib conditions:
    /// 1. b ≥ 0
    /// 2. |ρ| < 1
    /// 3. σ > 0
    /// 4. a + b·σ·√(1 - ρ²) ≥ 0 (no-calendar-spread arbitrage)
    /// 5. b·(1 + |ρ|) ≤ 4 (Roger Lee's moment formula bound, optional)
    pub fn validate(&self) {
        assert!(self.b >= 0.0, "SVI: b must be >= 0, got {}", self.b);
        assert!(
            self.rho.abs() < 1.0,
            "SVI: |rho| must be < 1, got {}",
            self.rho
        );
        assert!(
            self.sigma > 0.0,
            "SVI: sigma must be > 0, got {}",
            self.sigma
        );
        let min_var = self.a + self.b * self.sigma * (1.0 - self.rho * self.rho).sqrt();
        assert!(
            min_var >= -1e-10,
            "SVI: a + b*sigma*sqrt(1-rho^2) must be >= 0, got {}",
            min_var
        );
    }
}

/// Compute SVI total variance at log-moneyness k.
///
/// $w(k) = a + b \bigl(\rho \cdot (k - m) + \sqrt{(k - m)^2 + \sigma^2}\bigr)$
pub fn svi_total_variance(p: &SviParameters, k: Real) -> Real {
    let km = k - p.m;
    p.a + p.b * (p.rho * km + (km * km + p.sigma * p.sigma).sqrt())
}

/// An SVI-based smile section.
///
/// Produces implied Black volatilities from SVI parameters.
///
/// Corresponds to `QuantLib::SviSmileSection`.
#[derive(Debug, Clone)]
pub struct SviSmileSection {
    exercise_time: Time,
    forward: Real,
    params: SviParameters,
}

impl SviSmileSection {
    /// Create a new SVI smile section.
    pub fn new(exercise_time: Time, forward: Real, params: SviParameters) -> Self {
        assert!(exercise_time > 0.0, "exercise time must be > 0");
        params.validate();
        Self {
            exercise_time,
            forward,
            params,
        }
    }

    /// The SVI parameters.
    pub fn params(&self) -> &SviParameters {
        &self.params
    }

    /// The forward price.
    pub fn forward(&self) -> Real {
        self.forward
    }
}

impl SmileSection for SviSmileSection {
    fn min_strike(&self) -> Real {
        0.0
    }

    fn max_strike(&self) -> Real {
        f64::INFINITY
    }

    fn atm_level(&self) -> Real {
        self.forward
    }

    fn volatility_impl(&self, strike: Real) -> Volatility {
        let k = (strike.max(1e-6) / self.forward).ln();
        let total_var = svi_total_variance(&self.params, k);
        (total_var.max(0.0) / self.exercise_time).sqrt()
    }

    fn exercise_time(&self) -> Time {
        self.exercise_time
    }
}

// ── SVI Calibration ───────────────────────────────────────────────────────────

/// Calibrate SVI parameters to market quotes.
///
/// # Arguments
/// * `forward` — forward price
/// * `expiry` — time to expiry
/// * `strikes` — market strikes
/// * `vols` — market implied Black vols
/// * `initial` — initial guess for SVI parameters (if `None`, uses defaults)
///
/// Returns calibrated `SviParameters`.
pub fn calibrate_svi(
    forward: Real,
    expiry: Real,
    strikes: &[Real],
    vols: &[Real],
    initial: Option<SviParameters>,
) -> SviParameters {
    let n = strikes.len();
    assert_eq!(n, vols.len());
    assert!(n >= 5, "need at least 5 points to calibrate 5 SVI parameters");

    // Convert vols to total variances
    let total_vars: Vec<Real> = vols.iter().map(|v| v * v * expiry).collect();
    let log_moneyness: Vec<Real> = strikes.iter().map(|k| (k / forward).ln()).collect();

    let init = initial.unwrap_or_else(|| {
        let rho: Real = -0.4;
        let sigma: Real = 0.1;
        let b = 2.0 / (1.0 + rho.abs());
        let avg_var: Real = total_vars.iter().sum::<Real>() / n as Real;
        let a = (avg_var - b * sigma * (1.0 - rho * rho).sqrt()).max(1e-6);
        SviParameters {
            a,
            b,
            sigma,
            rho,
            m: 0.0,
        }
    });

    // Nelder-Mead in 5D: [a, b, sigma, rho, m]
    let objective = |x: &[Real; 5]| -> Real {
        let p = SviParameters {
            a: x[0],
            b: x[1].max(0.0),
            sigma: x[2].max(1e-6),
            rho: x[3].max(-0.999).min(0.999),
            m: x[4],
        };
        let min_var = p.a + p.b * p.sigma * (1.0 - p.rho * p.rho).sqrt();
        let penalty = if min_var < 0.0 { 1e6 * min_var * min_var } else { 0.0 };

        let mut sse = 0.0;
        for i in 0..n {
            let model_var = svi_total_variance(&p, log_moneyness[i]);
            let diff = model_var - total_vars[i];
            sse += diff * diff;
        }
        sse + penalty
    };

    let x0 = [init.a, init.b, init.sigma, init.rho, init.m];
    let result = nelder_mead_5d(objective, x0, 10000, 1e-14);

    SviParameters {
        a: result[0],
        b: result[1].max(0.0),
        sigma: result[2].max(1e-6),
        rho: result[3].max(-0.999).min(0.999),
        m: result[4],
    }
}

/// 5-dimensional Nelder-Mead simplex optimizer.
fn nelder_mead_5d(
    f: impl Fn(&[Real; 5]) -> Real,
    x0: [Real; 5],
    max_iter: usize,
    tol: Real,
) -> [Real; 5] {
    const N: usize = 5;
    let mut simplex: Vec<([Real; N], Real)> = Vec::with_capacity(N + 1);
    simplex.push((x0, f(&x0)));

    for i in 0..N {
        let mut xi = x0;
        let pert = if xi[i].abs() > 1e-8 {
            xi[i] * 0.1
        } else {
            0.01
        };
        xi[i] += pert;
        simplex.push((xi, f(&xi)));
    }

    for _ in 0..max_iter {
        simplex.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        if simplex[N].1 - simplex[0].1 < tol {
            break;
        }

        // Centroid (excluding worst)
        let mut centroid = [0.0; N];
        for j in 0..N {
            for i in 0..N {
                centroid[i] += simplex[j].0[i];
            }
        }
        for c in &mut centroid {
            *c /= N as f64;
        }

        // Reflection
        let worst = simplex[N].0;
        let mut reflected = [0.0; N];
        for i in 0..N {
            reflected[i] = 2.0 * centroid[i] - worst[i];
        }
        let fr = f(&reflected);

        if fr < simplex[0].1 {
            // Expansion
            let mut expanded = [0.0; N];
            for i in 0..N {
                expanded[i] = 2.0 * reflected[i] - centroid[i];
            }
            let fe = f(&expanded);
            if fe < fr {
                simplex[N] = (expanded, fe);
            } else {
                simplex[N] = (reflected, fr);
            }
        } else if fr < simplex[N - 1].1 {
            simplex[N] = (reflected, fr);
        } else {
            // Contraction
            let mut contracted = [0.0; N];
            if fr < simplex[N].1 {
                for i in 0..N {
                    contracted[i] = 0.5 * (reflected[i] + centroid[i]);
                }
            } else {
                for i in 0..N {
                    contracted[i] = 0.5 * (worst[i] + centroid[i]);
                }
            }
            let fc = f(&contracted);
            if fc < simplex[N].1 {
                simplex[N] = (contracted, fc);
            } else {
                // Shrink
                let best = simplex[0].0;
                for j in 1..=N {
                    let mut xi = simplex[j].0;
                    for i in 0..N {
                        xi[i] = 0.5 * (xi[i] + best[i]);
                    }
                    simplex[j] = (xi, f(&xi));
                }
            }
        }
    }

    simplex.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    simplex[0].0
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn flat_smile_section_basic() {
        let smile = FlatSmileSection::new(1.0, 0.20, 100.0);
        assert_abs_diff_eq!(smile.volatility(90.0), 0.20, epsilon = 1e-15);
        assert_abs_diff_eq!(smile.volatility(110.0), 0.20, epsilon = 1e-15);
        assert_abs_diff_eq!(smile.variance(100.0), 0.04, epsilon = 1e-15);
        assert_abs_diff_eq!(smile.atm_level(), 100.0, epsilon = 1e-15);
    }

    #[test]
    fn flat_smile_section_option_price_call_put_parity() {
        let smile = FlatSmileSection::new(0.5, 0.25, 100.0);
        let k = 100.0;
        let df = 0.98;
        let call = smile.option_price(k, SmileOptionType::Call, df);
        let put = smile.option_price(k, SmileOptionType::Put, df);
        // C - P = df * (F - K)
        assert_abs_diff_eq!(call - put, df * (100.0 - k), epsilon = 1e-10);
    }

    #[test]
    fn sabr_smile_section_matches_direct() {
        let params = SabrParameters {
            alpha: 0.04,
            beta: 0.5,
            nu: 0.3,
            rho: -0.2,
        };
        let section = SabrSmileSection::new(1.0, 0.03, params);
        let direct = sabr_volatility(0.03, 0.035, 1.0, &params);
        assert_abs_diff_eq!(
            section.volatility(0.035),
            direct,
            epsilon = 1e-10
        );
    }

    #[test]
    fn svi_total_variance_at_m() {
        // At k = m: w(m) = a + b * sigma
        let params = SviParameters {
            a: -0.0666,
            b: 0.229,
            sigma: 0.337,
            rho: 0.439,
            m: 0.193,
        };
        let w = svi_total_variance(&params, params.m);
        let expected = params.a + params.b * params.sigma;
        assert_abs_diff_eq!(w, expected, epsilon = 1e-10);
    }

    #[test]
    fn svi_smile_section_basic() {
        // From QuantLib test: tte = 11/365, forward = 123.45
        let tte = 11.0 / 365.0;
        let forward = 123.45;
        let params = SviParameters {
            a: -0.0666,
            b: 0.229,
            sigma: 0.337,
            rho: 0.439,
            m: 0.193,
        };
        let section = SviSmileSection::new(tte, forward, params);

        // At k = forward * exp(m): log-moneyness = m, so w = a + b*sigma
        let strike = forward * params.m.exp();
        let expected_var = params.a + params.b * params.sigma;
        let expected_vol = (expected_var / tte).sqrt();
        assert_abs_diff_eq!(section.volatility(strike), expected_vol, epsilon = 1e-6);
    }

    #[test]
    fn svi_calibration_roundtrip() {
        let forward = 100.0;
        let expiry = 1.0;
        let true_params = SviParameters {
            a: 0.04,
            b: 0.2,
            sigma: 0.1,
            rho: -0.3,
            m: 0.0,
        };

        // Generate synthetic smile
        let strikes: Vec<Real> = (80..=120).map(|k| k as Real).collect();
        let vols: Vec<Real> = strikes
            .iter()
            .map(|&k| {
                let lk = (k / forward).ln();
                let w = svi_total_variance(&true_params, lk);
                (w / expiry).sqrt()
            })
            .collect();

        let calibrated = calibrate_svi(forward, expiry, &strikes, &vols, None);

        // Check that the calibrated model reproduces the input vols
        for i in 0..strikes.len() {
            let lk = (strikes[i] / forward).ln();
            let model_var = svi_total_variance(&calibrated, lk);
            let model_vol = (model_var / expiry).sqrt();
            assert!(
                (model_vol - vols[i]).abs() < 0.005,
                "SVI calibration: strike {}, expected {:.6}, got {:.6}",
                strikes[i],
                vols[i],
                model_vol
            );
        }
    }
}
