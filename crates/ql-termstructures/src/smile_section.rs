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
    fn option_price(&self, strike: Real, option_type: SmileOptionType, discount: Real) -> Real {
        let f = self.atm_level();
        let t = self.exercise_time();
        let vol = self.volatility_impl(strike);
        let shift_val = self.shift();

        match self.volatility_type() {
            VolatilityType::ShiftedLognormal => black_formula(
                f + shift_val,
                strike + shift_val,
                vol,
                t,
                discount,
                option_type,
            ),
            VolatilityType::Normal => bachelier_formula(f, strike, vol, t, discount, option_type),
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
        SmileOptionType::Call => {
            discount * (std_dev * normal_pdf(d) + (forward - strike) * normal_cdf(d))
        }
        SmileOptionType::Put => {
            discount * (std_dev * normal_pdf(d) - (forward - strike) * normal_cdf(-d))
        }
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
        assert!((0.0..=1.0).contains(&params.beta), "beta must be in [0, 1]");
        assert!(params.nu >= 0.0, "nu must be non-negative");
        assert!(params.rho * params.rho < 1.0, "rho must satisfy |rho| < 1");
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

    /// Convert Raw SVI parameters to Jump-Wing (JW) parameters.
    ///
    /// The inverse of [`SviJwParameters::to_raw`].  All returned JW quantities
    /// are expressed as **total variance** (same units as the Raw `a` and `w(k)`).
    ///
    /// # Arguments
    /// * `t` — time to expiry (validated > 0; not used in the algebraic
    ///   conversion because all quantities are in total-variance units)
    pub fn to_jw(&self, t: Real) -> SviJwParameters {
        assert!(t > 0.0, "to_jw: t must be > 0");
        let delta = (self.m * self.m + self.sigma * self.sigma).sqrt();
        SviJwParameters {
            v_t: svi_total_variance(self, 0.0),
            psi_t: self.b * (self.rho - self.m / delta),
            p_t: self.b * (1.0 - self.rho),
            c_t: self.b * (1.0 + self.rho),
            v_tilde_t: self.a + self.b * self.sigma * (1.0 - self.rho * self.rho).sqrt(),
        }
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
    assert!(
        n >= 5,
        "need at least 5 points to calibrate 5 SVI parameters"
    );

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
            rho: x[3].clamp(-0.999, 0.999),
            m: x[4],
        };
        let min_var = p.a + p.b * p.sigma * (1.0 - p.rho * p.rho).sqrt();
        let penalty = if min_var < 0.0 {
            1e6 * min_var * min_var
        } else {
            0.0
        };

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
        rho: result[3].clamp(-0.999, 0.999),
        m: result[4],
    }
}

/// 5-dimensional Nelder-Mead simplex optimizer.
#[allow(clippy::needless_range_loop)]
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

// ── SVI Jump-Wing (JW) Parameterization ──────────────────────────────────────

/// SVI Jump-Wing (JW) parameters (Gatheral & Jacquier, 2014).
///
/// An equivalent reparameterization of SVI Raw that provides intuitive,
/// trader-friendly parameters with direct market interpretation.
///
/// All quantities are expressed as **total variance** (same units as the Raw
/// SVI output `w(k) = a + b(…)`), not as annualized quantities.
///
/// The mapping between Raw `{a, b, σ, ρ, m}` and these JW parameters is:
///
/// - `v_t   = w(0) = a + b(−ρm + √(m²+σ²))`   (ATM total variance)
/// - `ψ_t   = ∂w/∂k|_{k=0} = b(ρ − m/Δ)`     (ATM skew; `Δ = √(m²+σ²)`)
/// - `p_t   = b(1 − ρ)`                         (left / put wing slope)
/// - `c_t   = b(1 + ρ)`                         (right / call wing slope)
/// - `ṽ_t   = a + bσ√(1−ρ²)`                   (minimum total variance)
///
/// Reference: Gatheral & Jacquier (2014), *Arbitrage-free SVI volatility
/// surfaces*, Quantitative Finance 14(1), 59–71.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SviJwParameters {
    /// ATM total variance: $v_t = w(k=0)$.
    pub v_t: Real,
    /// ATM skew: $\psi_t = \partial_k w|_{k=0} = b(\rho - m/\Delta)$.
    ///
    /// Can be negative (negative skew is common in equity markets).
    pub psi_t: Real,
    /// Left (put) wing slope: $p_t = b(1 - \rho) > 0$.
    pub p_t: Real,
    /// Right (call) wing slope: $c_t = b(1 + \rho) > 0$.
    pub c_t: Real,
    /// Minimum total variance: $\tilde{v}_t = a + b\sigma\sqrt{1-\rho^2}$.
    pub v_tilde_t: Real,
}

impl SviJwParameters {
    /// Validate no-arbitrage and well-posedness constraints.
    ///
    /// Enforces:
    /// - `v_t > 0`
    /// - `v_tilde_t >= 0` and `v_tilde_t <= v_t`
    /// - `p_t > 0` and `c_t > 0` (required so that `|ρ| < 1` in Raw)
    /// - `−p_t ≤ ψ_t ≤ c_t` (required so that `|β| ≤ 1`, keeping σ real)
    pub fn validate(&self) {
        assert!(self.v_t > 0.0, "SVI-JW: v_t must be > 0, got {}", self.v_t);
        assert!(
            self.v_tilde_t >= 0.0,
            "SVI-JW: v_tilde_t must be >= 0, got {}",
            self.v_tilde_t
        );
        assert!(
            self.v_tilde_t <= self.v_t + 1e-10,
            "SVI-JW: v_tilde_t ({}) must be <= v_t ({})",
            self.v_tilde_t,
            self.v_t
        );
        assert!(
            self.p_t > 0.0,
            "SVI-JW: p_t must be > 0 (ensures |rho| < 1), got {}",
            self.p_t
        );
        assert!(
            self.c_t > 0.0,
            "SVI-JW: c_t must be > 0 (ensures |rho| < 1), got {}",
            self.c_t
        );
        assert!(
            self.psi_t >= -self.p_t - 1e-10,
            "SVI-JW: psi_t ({}) must be >= -p_t ({}) to keep |beta| <= 1",
            self.psi_t,
            -self.p_t
        );
        assert!(
            self.psi_t <= self.c_t + 1e-10,
            "SVI-JW: psi_t ({}) must be <= c_t ({}) to keep |beta| <= 1",
            self.psi_t,
            self.c_t
        );
    }

    /// Convert JW parameters to Raw SVI parameters.
    ///
    /// Uses the closed-form algebraic mapping from Gatheral & Jacquier (2014).
    ///
    /// # Critical β = 0 edge case
    ///
    /// The auxiliary variable β = ρ − ψ_t/b equals m/Δ in Raw space.
    /// When |β| < 1e-12 the smile's minimum lies exactly at the money
    /// (`m = 0`), and the general Δ-formula would divide by an expression
    /// that collapses to zero.  The closed-form shortcut used instead is:
    ///
    /// - `m = 0`
    /// - `σ = (v_t − ṽ_t) / (b · (1 − √(1 − ρ²)))`
    ///
    /// # Arguments
    /// * `t` — time to expiry (validated > 0; not consumed in the algebraic
    ///   conversion because all JW parameters are already in total-variance units)
    pub fn to_raw(&self, t: Real) -> SviParameters {
        assert!(t > 0.0, "to_raw: t must be > 0");

        let b = 0.5 * (self.p_t + self.c_t);
        let rho = 1.0 - self.p_t / b;
        // β = ρ − ψ_t/b = m/Δ in Raw space.
        let beta = rho - self.psi_t / b;

        let (m, sigma) = if beta.abs() < 1e-12 {
            // ── β = 0 edge case: minimum is at the money (m = 0) ──────────
            // σ = (v_t − ṽ_t) / (b · (1 − √(1−ρ²)))
            let one_minus_sqrt_one_minus_rho_sq = 1.0 - (1.0 - rho * rho).sqrt();
            let sigma = (self.v_t - self.v_tilde_t) / (b * one_minus_sqrt_one_minus_rho_sq);
            (0.0_f64, sigma)
        } else {
            // ── General case: β ≠ 0 ───────────────────────────────────────
            // Δ = (v_t − ṽ_t) / (b · (1 − ρβ − √((1−β²)(1−ρ²))))
            let denom =
                1.0 - rho * beta - ((1.0 - beta * beta) * (1.0 - rho * rho)).sqrt();
            let big_delta = (self.v_t - self.v_tilde_t) / (b * denom);
            let m = beta * big_delta;
            let sigma = big_delta * (1.0 - beta * beta).sqrt();
            (m, sigma)
        };

        let a = self.v_tilde_t - b * sigma * (1.0 - rho * rho).sqrt();
        SviParameters { a, b, sigma, rho, m }
    }
}

/// An SVI Jump-Wing smile section.
///
/// Stores JW parameters and delegates to [`SviSmileSection`] internally after
/// converting to Raw at construction time.
#[derive(Debug, Clone)]
pub struct SviJwSmileSection {
    jw_params: SviJwParameters,
    inner: SviSmileSection,
}

impl SviJwSmileSection {
    /// Create a new SVI Jump-Wing smile section.
    ///
    /// Validates the JW parameters, converts them to Raw, and constructs an
    /// inner [`SviSmileSection`] for all volatility computations.
    pub fn new(exercise_time: Time, forward: Real, params: SviJwParameters) -> Self {
        assert!(exercise_time > 0.0, "exercise time must be > 0");
        params.validate();
        let raw = params.to_raw(exercise_time);
        let inner = SviSmileSection::new(exercise_time, forward, raw);
        Self { jw_params: params, inner }
    }

    /// The Jump-Wing parameters.
    pub fn jw_params(&self) -> &SviJwParameters {
        &self.jw_params
    }

    /// The equivalent Raw SVI parameters.
    pub fn raw_params(&self) -> &SviParameters {
        self.inner.params()
    }

    /// The forward price.
    pub fn forward(&self) -> Real {
        self.inner.forward()
    }
}

impl SmileSection for SviJwSmileSection {
    fn min_strike(&self) -> Real {
        self.inner.min_strike()
    }

    fn max_strike(&self) -> Real {
        self.inner.max_strike()
    }

    fn atm_level(&self) -> Real {
        self.inner.atm_level()
    }

    fn volatility_impl(&self, strike: Real) -> Volatility {
        self.inner.volatility_impl(strike)
    }

    fn exercise_time(&self) -> Time {
        self.inner.exercise_time()
    }
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
        assert_abs_diff_eq!(section.volatility(0.035), direct, epsilon = 1e-10);
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

    // ── SVI Jump-Wing Tests ───────────────────────────────────────────────────

    /// Roundtrip: Raw → JW → Raw.  Parameters must be recovered exactly.
    #[test]
    fn svi_jw_roundtrip_raw_to_jw_to_raw() {
        let t = 1.0;
        let raw = SviParameters {
            a: -0.0666,
            b: 0.229,
            sigma: 0.337,
            rho: 0.439,
            m: 0.193,
        };
        let jw = raw.to_jw(t);
        let raw2 = jw.to_raw(t);

        assert_abs_diff_eq!(raw2.a, raw.a, epsilon = 1e-10);
        assert_abs_diff_eq!(raw2.b, raw.b, epsilon = 1e-10);
        assert_abs_diff_eq!(raw2.sigma, raw.sigma, epsilon = 1e-10);
        assert_abs_diff_eq!(raw2.rho, raw.rho, epsilon = 1e-10);
        assert_abs_diff_eq!(raw2.m, raw.m, epsilon = 1e-10);
    }

    /// Roundtrip: JW → Raw → JW.  JW parameters must be recovered exactly.
    #[test]
    fn svi_jw_roundtrip_jw_to_raw_to_jw() {
        let t = 0.5;
        let raw = SviParameters {
            a: 0.04,
            b: 0.2,
            sigma: 0.1,
            rho: -0.3,
            m: 0.05,
        };
        let jw = raw.to_jw(t);
        let raw2 = jw.to_raw(t);
        let jw2 = raw2.to_jw(t);

        assert_abs_diff_eq!(jw2.v_t, jw.v_t, epsilon = 1e-10);
        assert_abs_diff_eq!(jw2.psi_t, jw.psi_t, epsilon = 1e-10);
        assert_abs_diff_eq!(jw2.p_t, jw.p_t, epsilon = 1e-10);
        assert_abs_diff_eq!(jw2.c_t, jw.c_t, epsilon = 1e-10);
        assert_abs_diff_eq!(jw2.v_tilde_t, jw.v_tilde_t, epsilon = 1e-10);
    }

    /// β = 0 edge case: ψ_t = ρ·b forces m = 0 in Raw.
    ///
    /// With p_t = 0.1, c_t = 0.3: b = 0.2, ρ = 0.5.
    /// β = ρ − ψ_t/b = 0  ⟺  ψ_t = ρ·b = 0.1.
    #[test]
    fn svi_jw_beta_zero_no_panic_and_correct_raw() {
        let jw = SviJwParameters {
            v_t: 0.04,
            psi_t: 0.1, // = ρ·b = 0.5 · 0.2, so β = 0
            p_t: 0.1,
            c_t: 0.3,
            v_tilde_t: 0.02,
        };
        let t = 1.0;
        let raw = jw.to_raw(t); // must not panic

        // β = 0 ⟹ m = 0
        assert_abs_diff_eq!(raw.m, 0.0, epsilon = 1e-12);
        assert_abs_diff_eq!(raw.b, 0.2, epsilon = 1e-10);
        assert_abs_diff_eq!(raw.rho, 0.5, epsilon = 1e-10);

        // σ = (v_t − ṽ_t) / (b · (1 − √(1−ρ²)))
        let expected_sigma =
            0.02 / (0.2 * (1.0 - (1.0_f64 - 0.25_f64).sqrt()));
        assert_abs_diff_eq!(raw.sigma, expected_sigma, epsilon = 1e-10);

        // Full roundtrip back to JW
        let jw2 = raw.to_jw(t);
        assert_abs_diff_eq!(jw2.v_t, jw.v_t, epsilon = 1e-10);
        assert_abs_diff_eq!(jw2.psi_t, jw.psi_t, epsilon = 1e-10);
        assert_abs_diff_eq!(jw2.p_t, jw.p_t, epsilon = 1e-10);
        assert_abs_diff_eq!(jw2.c_t, jw.c_t, epsilon = 1e-10);
        assert_abs_diff_eq!(jw2.v_tilde_t, jw.v_tilde_t, epsilon = 1e-10);
    }

    /// SviJwSmileSection must produce the same implied vols as an
    /// SviSmileSection built from the equivalent Raw params.
    #[test]
    fn svi_jw_smile_section_matches_raw() {
        let t = 11.0 / 365.0;
        let forward = 100.0;
        let raw_params = SviParameters {
            a: 0.04,
            b: 0.2,
            sigma: 0.1,
            rho: -0.3,
            m: 0.05,
        };
        let jw_params = raw_params.to_jw(t);

        let raw_section = SviSmileSection::new(t, forward, raw_params);
        let jw_section = SviJwSmileSection::new(t, forward, jw_params);

        for &strike in &[80.0, 90.0, 95.0, 100.0, 105.0, 110.0, 120.0] {
            let vol_raw = raw_section.volatility(strike);
            let vol_jw = jw_section.volatility(strike);
            assert_abs_diff_eq!(vol_jw, vol_raw, epsilon = 1e-12);
        }
    }

    /// Invalid JW params (v_t ≤ 0) must be rejected.
    #[test]
    #[should_panic(expected = "v_t must be > 0")]
    fn svi_jw_validate_rejects_non_positive_v_t() {
        SviJwParameters {
            v_t: -0.01,
            psi_t: 0.0,
            p_t: 0.1,
            c_t: 0.2,
            v_tilde_t: 0.0,
        }
        .validate();
    }

    /// Invalid JW params (v_tilde_t > v_t) must be rejected.
    #[test]
    #[should_panic(expected = "v_tilde_t")]
    fn svi_jw_validate_rejects_v_tilde_exceeds_v_t() {
        SviJwParameters {
            v_t: 0.04,
            psi_t: 0.0,
            p_t: 0.1,
            c_t: 0.2,
            v_tilde_t: 0.05, // exceeds v_t
        }
        .validate();
    }

    /// Invalid JW params (p_t ≤ 0) must be rejected.
    #[test]
    #[should_panic(expected = "p_t must be > 0")]
    fn svi_jw_validate_rejects_non_positive_p_t() {
        SviJwParameters {
            v_t: 0.04,
            psi_t: 0.0,
            p_t: 0.0,
            c_t: 0.2,
            v_tilde_t: 0.02,
        }
        .validate();
    }

    /// Invalid JW params (ψ_t > c_t, i.e. β < −1) must be rejected.
    #[test]
    #[should_panic(expected = "psi_t")]
    fn svi_jw_validate_rejects_psi_t_out_of_range() {
        SviJwParameters {
            v_t: 0.04,
            psi_t: 0.5, // exceeds c_t = 0.2
            p_t: 0.1,
            c_t: 0.2,
            v_tilde_t: 0.02,
        }
        .validate();
    }
}
