//! Per-expiry smile calibration framework.
//!
//! Calibrates smile parameters (SABR, SVI, or custom) to market quotes on a
//! per-expiry basis, producing a collection of `SmileSection` objects that
//! together describe the full volatility surface.
//!
//! This module provides:
//! * `SmileCalibrationResult` — calibrated smile parameters for one expiry
//! * `SmileSurface` — a collection of per-expiry smile sections
//! * `calibrate_sabr_surface` — calibrate SABR smiles across multiple expiries
//! * `calibrate_svi_surface` — calibrate SVI smiles across multiple expiries

use ql_core::{Real, Time, Volatility};
use ql_math::interpolations::sabr::{calibrate_sabr, SabrParameters};

use crate::smile_section::{
    calibrate_svi, SabrSmileSection, SmileSection, SviParameters, SviSmileSection,
};

/// Market data for a single expiry.
#[derive(Debug, Clone)]
pub struct ExpirySmileData {
    /// Time to expiry in years.
    pub expiry: Time,
    /// Forward price at this expiry.
    pub forward: Real,
    /// Strike levels.
    pub strikes: Vec<Real>,
    /// Market implied Black volatilities.
    pub vols: Vec<Volatility>,
}

impl ExpirySmileData {
    /// Create new expiry smile data.
    pub fn new(expiry: Time, forward: Real, strikes: Vec<Real>, vols: Vec<Volatility>) -> Self {
        assert_eq!(strikes.len(), vols.len(), "strikes and vols must match");
        assert!(!strikes.is_empty(), "need at least one data point");
        Self {
            expiry,
            forward,
            strikes,
            vols,
        }
    }
}

/// Result of calibrating a smile for one expiry.
#[derive(Debug, Clone)]
pub struct SmileCalibrationResult<P: Clone + std::fmt::Debug> {
    /// Time to expiry.
    pub expiry: Time,
    /// Forward price.
    pub forward: Real,
    /// Calibrated parameters.
    pub params: P,
    /// RMS calibration error (in vol units).
    pub rms_error: Real,
    /// Maximum absolute calibration error (in vol units).
    pub max_error: Real,
}

/// A volatility surface described by a collection of per-expiry smile sections.
#[derive(Debug)]
pub struct SmileSurface {
    /// Per-expiry smile sections, sorted by expiry time.
    sections: Vec<(Time, Box<dyn SmileSection>)>,
}

impl SmileSurface {
    /// Create a new empty smile surface.
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
        }
    }

    /// Add a smile section for a given expiry.
    pub fn add_section(&mut self, expiry: Time, section: Box<dyn SmileSection>) {
        self.sections.push((expiry, section));
        self.sections.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    }

    /// Number of expiry slices.
    pub fn num_expiries(&self) -> usize {
        self.sections.len()
    }

    /// Get the smile section for the i-th expiry.
    pub fn section(&self, index: usize) -> Option<&dyn SmileSection> {
        self.sections.get(index).map(|(_, s)| s.as_ref())
    }

    /// Get the expiry time for the i-th slice.
    pub fn expiry(&self, index: usize) -> Option<Time> {
        self.sections.get(index).map(|(t, _)| *t)
    }

    /// Interpolate implied volatility at an arbitrary (time, strike).
    ///
    /// Uses flat extrapolation in time beyond the surface boundaries, and
    /// linear interpolation in variance between adjacent expiries.
    pub fn volatility(&self, t: Time, strike: Real) -> Volatility {
        if self.sections.is_empty() {
            return 0.0;
        }

        // Before the first expiry
        if t <= self.sections[0].0 {
            return self.sections[0].1.volatility(strike);
        }

        // After the last expiry
        let n = self.sections.len();
        if t >= self.sections[n - 1].0 {
            return self.sections[n - 1].1.volatility(strike);
        }

        // Find the bracketing expiries
        let mut i = 0;
        while i < n - 1 && self.sections[i + 1].0 < t {
            i += 1;
        }

        let t1 = self.sections[i].0;
        let t2 = self.sections[i + 1].0;
        let v1 = self.sections[i].1.volatility(strike);
        let v2 = self.sections[i + 1].1.volatility(strike);

        // Linear interpolation in total variance
        let w1 = v1 * v1 * t1;
        let w2 = v2 * v2 * t2;
        let alpha = (t - t1) / (t2 - t1);
        let w = w1 + alpha * (w2 - w1);

        if w <= 0.0 || t <= 0.0 {
            return 0.0;
        }
        (w / t).sqrt()
    }
}

impl Default for SmileSurface {
    fn default() -> Self {
        Self::new()
    }
}

// ── SABR surface calibration ─────────────────────────────────────────────────

/// Calibrate SABR smiles across multiple expiries.
///
/// # Arguments
/// * `market_data` — per-expiry market data
/// * `beta` — fixed CEV exponent (same for all expiries)
///
/// Returns a vector of calibration results and a `SmileSurface`.
pub fn calibrate_sabr_surface(
    market_data: &[ExpirySmileData],
    beta: Real,
) -> (Vec<SmileCalibrationResult<SabrParameters>>, SmileSurface) {
    let mut results = Vec::with_capacity(market_data.len());
    let mut surface = SmileSurface::new();

    for data in market_data {
        let params = calibrate_sabr(
            data.forward,
            data.expiry,
            &data.strikes,
            &data.vols,
            beta,
            0.04,  // initial alpha
            0.4,   // initial nu
            -0.3,  // initial rho
        );

        // Compute calibration errors
        let (rms, max_err) = calibration_errors(data, |k| {
            ql_math::interpolations::sabr::sabr_volatility(data.forward, k, data.expiry, &params)
        });

        results.push(SmileCalibrationResult {
            expiry: data.expiry,
            forward: data.forward,
            params,
            rms_error: rms,
            max_error: max_err,
        });

        let section = SabrSmileSection::new(data.expiry, data.forward, params);
        surface.add_section(data.expiry, Box::new(section));
    }

    (results, surface)
}

// ── SVI surface calibration ──────────────────────────────────────────────────

/// Calibrate SVI smiles across multiple expiries.
///
/// # Arguments
/// * `market_data` — per-expiry market data (must have ≥ 5 points per expiry)
///
/// Returns a vector of calibration results and a `SmileSurface`.
pub fn calibrate_svi_surface(
    market_data: &[ExpirySmileData],
) -> (Vec<SmileCalibrationResult<SviParameters>>, SmileSurface) {
    let mut results = Vec::with_capacity(market_data.len());
    let mut surface = SmileSurface::new();

    for data in market_data {
        let params = calibrate_svi(
            data.forward,
            data.expiry,
            &data.strikes,
            &data.vols,
            None,
        );

        // Compute calibration errors
        let (rms, max_err) = calibration_errors(data, |k| {
            let lk = (k / data.forward).ln();
            let w = crate::smile_section::svi_total_variance(&params, lk);
            (w.max(0.0) / data.expiry).sqrt()
        });

        results.push(SmileCalibrationResult {
            expiry: data.expiry,
            forward: data.forward,
            params,
            rms_error: rms,
            max_error: max_err,
        });

        let section = SviSmileSection::new(data.expiry, data.forward, params);
        surface.add_section(data.expiry, Box::new(section));
    }

    (results, surface)
}

/// Compute RMS and max abs calibration error.
fn calibration_errors(
    data: &ExpirySmileData,
    model_vol: impl Fn(Real) -> Volatility,
) -> (Real, Real) {
    let n = data.strikes.len() as Real;
    let mut sse = 0.0;
    let mut max_err = 0.0_f64;

    for i in 0..data.strikes.len() {
        let v_model = model_vol(data.strikes[i]);
        let err = (v_model - data.vols[i]).abs();
        sse += err * err;
        max_err = max_err.max(err);
    }

    ((sse / n).sqrt(), max_err)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    fn make_sabr_data(expiry: Real, forward: Real) -> ExpirySmileData {
        let params = SabrParameters {
            alpha: 0.04,
            beta: 0.5,
            nu: 0.3,
            rho: -0.25,
        };
        let strikes: Vec<Real> = (1..=9).map(|i| forward * (0.5 + 0.125 * i as Real)).collect();
        let vols: Vec<Real> = strikes
            .iter()
            .map(|&k| ql_math::interpolations::sabr::sabr_volatility(forward, k, expiry, &params))
            .collect();
        ExpirySmileData::new(expiry, forward, strikes, vols)
    }

    fn make_svi_data(expiry: Real, forward: Real) -> ExpirySmileData {
        let params = SviParameters {
            a: 0.04,
            b: 0.2,
            sigma: 0.1,
            rho: -0.3,
            m: 0.0,
        };
        let strikes: Vec<Real> = (1..=11)
            .map(|i| forward * (0.5 + 0.1 * i as Real))
            .collect();
        let vols: Vec<Real> = strikes
            .iter()
            .map(|&k| {
                let lk = (k / forward).ln();
                let w = crate::smile_section::svi_total_variance(&params, lk);
                (w.max(0.0) / expiry).sqrt()
            })
            .collect();
        ExpirySmileData::new(expiry, forward, strikes, vols)
    }

    #[test]
    fn sabr_surface_calibration() {
        let data = vec![
            make_sabr_data(0.25, 0.04),
            make_sabr_data(0.50, 0.04),
            make_sabr_data(1.00, 0.04),
        ];

        let (results, surface) = calibrate_sabr_surface(&data, 0.5);

        assert_eq!(results.len(), 3);
        assert_eq!(surface.num_expiries(), 3);

        for r in &results {
            assert!(
                r.rms_error < 0.005,
                "SABR calibration RMS error too large: {:.6} at T={}",
                r.rms_error,
                r.expiry
            );
        }
    }

    #[test]
    fn svi_surface_calibration() {
        let data = vec![
            make_svi_data(0.25, 100.0),
            make_svi_data(0.50, 100.0),
            make_svi_data(1.00, 100.0),
        ];

        let (results, surface) = calibrate_svi_surface(&data);

        assert_eq!(results.len(), 3);
        assert_eq!(surface.num_expiries(), 3);

        for r in &results {
            assert!(
                r.rms_error < 0.01,
                "SVI calibration RMS error too large: {:.6} at T={}",
                r.rms_error,
                r.expiry
            );
        }
    }

    #[test]
    fn smile_surface_interpolation() {
        let data = vec![
            make_sabr_data(0.5, 0.04),
            make_sabr_data(1.0, 0.04),
        ];

        let (_, surface) = calibrate_sabr_surface(&data, 0.5);

        // At exactly T=0.5 and T=1.0, should match the sections
        let v05 = surface.volatility(0.5, 0.04);
        let v10 = surface.volatility(1.0, 0.04);
        assert!(v05 > 0.0);
        assert!(v10 > 0.0);

        // At T=0.75, should interpolate in variance
        let v075 = surface.volatility(0.75, 0.04);
        assert!(v075 > 0.0);
    }

    #[test]
    fn smile_surface_flat_extrapolation() {
        let data = vec![
            make_sabr_data(0.5, 0.04),
            make_sabr_data(1.0, 0.04),
        ];

        let (_, surface) = calibrate_sabr_surface(&data, 0.5);

        // Before first expiry — should use first section
        let v_before = surface.volatility(0.1, 0.04);
        let v_first = surface.section(0).unwrap().volatility(0.04);
        assert_abs_diff_eq!(v_before, v_first, epsilon = 1e-10);

        // After last expiry — should use last section
        let v_after = surface.volatility(2.0, 0.04);
        let v_last = surface.section(1).unwrap().volatility(0.04);
        assert_abs_diff_eq!(v_after, v_last, epsilon = 1e-10);
    }
}
