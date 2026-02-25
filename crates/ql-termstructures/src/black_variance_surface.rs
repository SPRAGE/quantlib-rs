//! `BlackVarianceSurface` — a Black-variance surface interpolated from a
//! grid of (expiry × strike) implied volatilities
//! (translates `ql/termstructures/volatility/equityfx/blackvariancesurface.hpp`).
//!
//! The surface stores a 2D grid of Black volatilities and performs bilinear
//! interpolation on **variance** (`σ²·t`) to ensure calendar-time consistency.

use crate::black_vol_term_structure::BlackVolTermStructure;
use crate::term_structure::TermStructure;
use crate::volatility_term_structure::VolatilityTermStructure;
use crate::yield_term_structure::YieldTermStructureData;
use ql_core::{errors::Result, Real, Time, Volatility};
use ql_time::{Calendar, Date, DayCounter, NullCalendar};
use std::sync::Arc;

/// Extrapolation mode for out-of-range strikes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Extrapolation {
    /// Clamp to the nearest boundary value.
    ConstantExtrapolation,
    /// No extrapolation — panic on out-of-range queries (for debugging).
    None,
}

/// A Black-variance surface built from a grid of implied volatilities.
///
/// Interpolation is performed on the total variance `v(t,K) = σ²(t,K) · t`
/// and then converted back to volatility or variance.
///
/// Corresponds to `QuantLib::BlackVarianceSurface`.
#[derive(Debug)]
pub struct BlackVarianceSurface {
    data: YieldTermStructureData,
    /// Expiry dates.
    dates: Vec<Date>,
    /// Times corresponding to expiry dates.
    times: Vec<Time>,
    /// Strikes (ascending).
    strikes: Vec<Real>,
    /// Total variances: `variances[i][j] = σ²(t_i, K_j) * t_i`.
    /// Row i = expiry i, Col j = strike j.
    variances: Vec<Vec<Real>>,
    /// Extrapolation mode.
    extrapolation: Extrapolation,
}

impl BlackVarianceSurface {
    /// Build a Black variance surface from dates, strikes, and a volatility grid.
    ///
    /// # Arguments
    /// * `reference_date` — the valuation date
    /// * `dates` — option expiry dates (ascending, all after reference_date)
    /// * `strikes` — strike grid (ascending)
    /// * `vols` — `vols[i][j]` = implied vol for expiry `dates[i]`, strike `strikes[j]`
    /// * `day_counter` — used for date → time conversion
    /// * `extrapolation` — how to handle out-of-range strikes
    pub fn new(
        reference_date: Date,
        dates: &[Date],
        strikes: &[Real],
        vols: &[Vec<Volatility>],
        day_counter: impl DayCounter + 'static,
        extrapolation: Extrapolation,
    ) -> Result<Self> {
        ql_core::ensure!(!dates.is_empty(), "need at least 1 expiry date");
        ql_core::ensure!(!strikes.is_empty(), "need at least 1 strike");
        ql_core::ensure!(
            vols.len() == dates.len(),
            "vols rows must match dates length"
        );
        for (i, row) in vols.iter().enumerate() {
            ql_core::ensure!(
                row.len() == strikes.len(),
                "vols row {i} length ({}) must match strikes length ({})",
                row.len(),
                strikes.len()
            );
        }

        let dc: Arc<dyn DayCounter> = Arc::new(day_counter);

        let times: Vec<Time> = dates
            .iter()
            .map(|&d| dc.year_fraction(reference_date, d))
            .collect();

        // Pre-compute total variances: σ² × t
        let variances: Vec<Vec<Real>> = vols
            .iter()
            .zip(times.iter())
            .map(|(row, &t)| row.iter().map(|&v| v * v * t).collect())
            .collect();

        Ok(Self {
            data: YieldTermStructureData {
                reference_date,
                calendar: Box::new(NullCalendar),
                day_counter: dc,
            },
            dates: dates.to_vec(),
            times,
            strikes: strikes.to_vec(),
            variances,
            extrapolation,
        })
    }

    /// Set a custom calendar.
    pub fn with_calendar(mut self, calendar: impl Calendar + 'static) -> Self {
        self.data.calendar = Box::new(calendar);
        self
    }

    /// Bilinear interpolation on the total variance grid.
    fn interpolate_variance(&self, t: Time, strike: Real) -> Real {
        let n_t = self.times.len();
        let n_k = self.strikes.len();

        // Clamp or restrict strike
        let k = match self.extrapolation {
            Extrapolation::ConstantExtrapolation => {
                strike.clamp(self.strikes[0], self.strikes[n_k - 1])
            }
            Extrapolation::None => strike,
        };

        // Clamp time to surface range
        let t_clamped = t.clamp(self.times[0], self.times[n_t - 1]);

        // Find time interval
        let ti = find_interval(&self.times, t_clamped);
        // Find strike interval
        let ki = find_interval(&self.strikes, k);

        // Bilinear interpolation on total variance
        let t_frac = if self.times[ti + 1] - self.times[ti] > 0.0 {
            (t_clamped - self.times[ti]) / (self.times[ti + 1] - self.times[ti])
        } else {
            0.0
        };

        let k_frac = if self.strikes[ki + 1] - self.strikes[ki] > 0.0 {
            (k - self.strikes[ki]) / (self.strikes[ki + 1] - self.strikes[ki])
        } else {
            0.0
        };

        let v00 = self.variances[ti][ki];
        let v01 = self.variances[ti][ki + 1];
        let v10 = self.variances[ti + 1][ki];
        let v11 = self.variances[ti + 1][ki + 1];

        let v0 = v00 + k_frac * (v01 - v00);
        let v1 = v10 + k_frac * (v11 - v10);

        v0 + t_frac * (v1 - v0)
    }
}

/// Find the index `i` such that `xs[i] <= x < xs[i+1]`.
/// Clamps to `[0, n-2]`.
fn find_interval(xs: &[Real], x: Real) -> usize {
    let n = xs.len();
    if n < 2 {
        return 0;
    }
    if x <= xs[0] {
        return 0;
    }
    if x >= xs[n - 1] {
        return n - 2;
    }
    // Binary search
    let mut lo = 0;
    let mut hi = n - 1;
    while hi - lo > 1 {
        let mid = (lo + hi) / 2;
        if xs[mid] <= x {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}

impl TermStructure for BlackVarianceSurface {
    fn reference_date(&self) -> Date {
        self.data.reference_date
    }

    fn day_counter(&self) -> &dyn DayCounter {
        &*self.data.day_counter
    }

    fn calendar(&self) -> &dyn Calendar {
        &*self.data.calendar
    }

    fn max_date(&self) -> Date {
        *self.dates.last().unwrap()
    }
}

impl VolatilityTermStructure for BlackVarianceSurface {
    fn min_strike(&self) -> Real {
        self.strikes[0]
    }

    fn max_strike(&self) -> Real {
        *self.strikes.last().unwrap()
    }
}

impl BlackVolTermStructure for BlackVarianceSurface {
    fn black_variance_impl(&self, t: Time, strike: Real) -> Real {
        if t <= 0.0 {
            return 0.0;
        }
        self.interpolate_variance(t, strike)
    }

    fn black_vol_impl(&self, t: Time, strike: Real) -> Volatility {
        if t <= 0.0 {
            // Return the vol at the smallest expiry
            return if self.variances.is_empty() {
                0.0
            } else {
                let var = self.interpolate_variance(self.times[0], strike);
                (var / self.times[0]).sqrt()
            };
        }
        let var = self.interpolate_variance(t, strike);
        (var / t).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use ql_time::Actual365Fixed;

    fn sample_surface() -> BlackVarianceSurface {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let dates = vec![
            Date::from_ymd(2025, 4, 2).unwrap(), // ~0.25y
            Date::from_ymd(2025, 7, 2).unwrap(), // ~0.5y
            Date::from_ymd(2026, 1, 2).unwrap(), // ~1.0y
        ];
        let strikes = vec![80.0, 100.0, 120.0];
        // Simple smile: higher vol at wings
        let vols = vec![
            vec![0.25, 0.20, 0.22], // 3M
            vec![0.24, 0.19, 0.21], // 6M
            vec![0.23, 0.18, 0.20], // 1Y
        ];
        BlackVarianceSurface::new(
            ref_date,
            &dates,
            &strikes,
            &vols,
            Actual365Fixed,
            Extrapolation::ConstantExtrapolation,
        )
        .unwrap()
    }

    #[test]
    fn surface_at_pillar_points() {
        let surface = sample_surface();
        // At the ATM strike (100) and first expiry time
        let t1 = surface.times[0];
        let vol = surface.black_vol_impl(t1, 100.0);
        assert_abs_diff_eq!(vol, 0.20, epsilon = 1e-10);
    }

    #[test]
    fn surface_interpolation_strike() {
        let surface = sample_surface();
        let t1 = surface.times[0];
        // Between 80 (vol=0.25) and 100 (vol=0.20) — variance interpolation
        let vol = surface.black_vol_impl(t1, 90.0);
        // Should be between the two wing vols
        assert!(vol > 0.19 && vol < 0.26, "vol = {vol}");
    }

    #[test]
    fn surface_interpolation_time() {
        let surface = sample_surface();
        // At ATM between 6M and 1Y
        let t_mid = (surface.times[1] + surface.times[2]) / 2.0;
        let vol = surface.black_vol_impl(t_mid, 100.0);
        // Should be between 0.19 and 0.18
        assert!(vol > 0.17 && vol < 0.20, "vol = {vol}");
    }

    #[test]
    fn surface_extrapolation_clamp() {
        let surface = sample_surface();
        let t1 = surface.times[0];
        // Strike below min → clamp to 80 (vol = 0.25 at 3M)
        let vol_low = surface.black_vol_impl(t1, 50.0);
        assert_abs_diff_eq!(vol_low, 0.25, epsilon = 1e-10);

        // Strike above max → clamp to 120 (vol = 0.22 at 3M)
        let vol_high = surface.black_vol_impl(t1, 200.0);
        assert_abs_diff_eq!(vol_high, 0.22, epsilon = 1e-10);
    }

    #[test]
    fn surface_variance_at_zero() {
        let surface = sample_surface();
        let var = surface.black_variance_impl(0.0, 100.0);
        assert_abs_diff_eq!(var, 0.0, epsilon = 1e-15);
    }
}
