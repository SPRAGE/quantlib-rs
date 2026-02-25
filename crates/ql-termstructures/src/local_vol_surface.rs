//! `LocalVolSurface` — Dupire local volatility surface.
//!
//! Computes local volatilities from an implied (Black) volatility surface
//! using Dupire's formula.
//!
//! Corresponds to `QuantLib::LocalVolSurface`.

use crate::black_vol_term_structure::BlackVolTermStructure;
use crate::local_vol_term_structure::LocalVolTermStructure;
use crate::term_structure::TermStructure;
use crate::volatility_term_structure::VolatilityTermStructure;
use crate::yield_term_structure::{YieldTermStructure, YieldTermStructureData};
use ql_core::{Real, Time, Volatility};
use ql_time::{Calendar, Date, DayCounter, NullCalendar};
use std::sync::Arc;

/// A local volatility surface derived from a Black volatility surface via
/// Dupire's formula.
///
/// Given a Black volatility surface `σ_B(T, K)`, the local variance is:
///
/// $$\sigma^2_\text{loc}(T, K) = \frac{\frac{\partial w}{\partial T}}
///     {1 - \frac{y}{w}\frac{\partial w}{\partial y}
///       + \frac14\left(-\frac14 - \frac{1}{w} + \frac{y^2}{w^2}\right)
///              \left(\frac{\partial w}{\partial y}\right)^2
///       + \frac12 \frac{\partial^2 w}{\partial y^2}}$$
///
/// where `w = σ²·T` is the total implied variance and `y = ln(K/F)` is
/// the log-moneyness.
///
/// Corresponds to `QuantLib::LocalVolSurface`.
#[derive(Debug)]
pub struct LocalVolSurface {
    data: YieldTermStructureData,
    /// The implied Black vol surface.
    black_vol: Arc<dyn BlackVolTermStructure>,
    /// The risk-free yield curve.
    risk_free_rate: Arc<dyn YieldTermStructure>,
    /// The dividend yield curve.
    dividend_yield: Arc<dyn YieldTermStructure>,
    /// The current underlying spot price.
    underlying: Real,
}

impl LocalVolSurface {
    /// Create a new LocalVolSurface from a Black vol surface.
    ///
    /// # Arguments
    /// * `black_vol` — the implied Black volatility surface
    /// * `risk_free_rate` — the risk-free yield curve
    /// * `dividend_yield` — the dividend yield curve
    /// * `underlying` — the current underlying spot price
    /// * `day_counter` — the day counter for the local vol surface
    pub fn new(
        black_vol: Arc<dyn BlackVolTermStructure>,
        risk_free_rate: Arc<dyn YieldTermStructure>,
        dividend_yield: Arc<dyn YieldTermStructure>,
        underlying: Real,
        day_counter: impl DayCounter + 'static,
    ) -> Self {
        let reference_date = black_vol.reference_date();
        Self {
            data: YieldTermStructureData {
                reference_date,
                calendar: Box::new(NullCalendar),
                day_counter: Arc::new(day_counter),
            },
            black_vol,
            risk_free_rate,
            dividend_yield,
            underlying,
        }
    }

    /// Create with a specific calendar.
    pub fn with_calendar(mut self, calendar: impl Calendar + 'static) -> Self {
        self.data.calendar = Box::new(calendar);
        self
    }

    /// Forward price for time `t`.
    #[allow(dead_code)]
    fn forward(&self, t: Time) -> Real {
        let df_r = self.risk_free_rate.discount(t);
        let df_q = self.dividend_yield.discount(t);
        self.underlying * df_q / df_r
    }
}

impl TermStructure for LocalVolSurface {
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
        self.black_vol.max_date()
    }
}

impl VolatilityTermStructure for LocalVolSurface {
    fn min_strike(&self) -> Real {
        self.black_vol.min_strike()
    }

    fn max_strike(&self) -> Real {
        self.black_vol.max_strike()
    }
}

impl LocalVolTermStructure for LocalVolSurface {
    fn local_vol_impl(&self, t: Time, strike: Real) -> Volatility {
        dupire_local_vol(
            t,
            strike,
            &*self.black_vol,
            &*self.risk_free_rate,
            &*self.dividend_yield,
            self.underlying,
        )
    }
}

/// Dupire's formula for local volatility.
///
/// Computes the local volatility at time `t` and strike `strike` using
/// finite differences on the Black total variance surface.
fn dupire_local_vol(
    t: Time,
    strike: Real,
    black_vol: &dyn BlackVolTermStructure,
    risk_free_rate: &dyn YieldTermStructure,
    dividend_yield: &dyn YieldTermStructure,
    spot: Real,
) -> Volatility {
    let eps_t = 1e-4_f64;
    let eps_k = strike.max(0.001) * 0.001;

    // Forward price at time t
    let df_r = risk_free_rate.discount(t.max(eps_t));
    let df_q = dividend_yield.discount(t.max(eps_t));
    let forward = spot * df_q / df_r;

    // Strike clamped
    let k = strike.max(1e-8);

    // Total implied variance w(t, k) = σ²(t, k) × t
    let w_fn = |tt: Time, kk: Real| -> Real {
        if tt <= 0.0 {
            return 0.0;
        }
        let vol = black_vol.black_vol_impl(tt, kk);
        vol * vol * tt
    };

    let w = w_fn(t, k);
    if w <= 0.0 {
        return 0.0;
    }

    // ── Time derivative: ∂w/∂t ──────────────────────────────────────────
    let dwdt = if t < eps_t {
        // Forward difference at small t
        let w_plus = w_fn(t + eps_t, k);
        w_plus / eps_t
    } else {
        // Central difference
        let w_plus = w_fn(t + eps_t, k);
        let w_minus = w_fn((t - eps_t).max(0.0), k);
        let dt = (t + eps_t) - (t - eps_t).max(0.0);
        (w_plus - w_minus) / dt
    };

    // Ensure non-decreasing total variance (no calendar spread arbitrage)
    let dwdt = dwdt.max(1e-15);

    // ── Log-moneyness y = ln(k / F) ────────────────────────────────────
    let y = (k / forward).ln();

    // ── Strike derivatives via log-moneyness ────────────────────────────
    let ln_eps = (eps_k / k).abs().max(1e-6);

    // ∂w/∂y via central differences on log-moneyness
    let k_up = k * (ln_eps).exp();
    let k_down = k * (-ln_eps).exp();
    let w_up = w_fn(t, k_up);
    let w_down = w_fn(t, k_down);
    let dwdy = (w_up - w_down) / (2.0 * ln_eps);

    // ∂²w/∂y² via second central difference
    let d2wdy2 = (w_up - 2.0 * w + w_down) / (ln_eps * ln_eps);

    // ── Dupire formula ──────────────────────────────────────────────────
    // numerator = ∂w/∂t
    // denominator = 1 - (y/w)*(∂w/∂y)
    //   + (1/4)*(-1/4 - 1/w + y²/w²)*(∂w/∂y)²
    //   + (1/2)*(∂²w/∂y²)
    let den1 = 1.0 - y / w * dwdy;
    let den2 = 0.25 * (-0.25 - 1.0 / w + y * y / (w * w)) * dwdy * dwdy;
    let den3 = 0.5 * d2wdy2;

    let denominator = den1 + den2 + den3;

    if denominator <= 1e-15 {
        // Degenerate case — return the Black vol as fallback
        return black_vol.black_vol_impl(t, k);
    }

    let local_var = dwdt / denominator;
    if local_var <= 0.0 {
        return 0.0;
    }
    local_var.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::black_vol_term_structure::BlackConstantVol;
    use crate::flat_forward::FlatForward;
    use approx::assert_abs_diff_eq;
    use ql_core::Compounding;
    use ql_time::{Actual365Fixed, Frequency};

    fn make_flat_surface() -> (
        Arc<dyn BlackVolTermStructure>,
        Arc<dyn YieldTermStructure>,
        Arc<dyn YieldTermStructure>,
    ) {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let vol_surface: Arc<dyn BlackVolTermStructure> =
            Arc::new(BlackConstantVol::new(ref_date, 0.20, Actual365Fixed));
        let rf: Arc<dyn YieldTermStructure> = Arc::new(FlatForward::new(
            ref_date,
            0.05,
            Actual365Fixed,
            Compounding::Continuous,
            Frequency::Annual,
        ));
        let dy: Arc<dyn YieldTermStructure> = Arc::new(FlatForward::new(
            ref_date,
            0.02,
            Actual365Fixed,
            Compounding::Continuous,
            Frequency::Annual,
        ));
        (vol_surface, rf, dy)
    }

    #[test]
    fn local_vol_surface_constant_vol() {
        // For a constant Black vol surface, local vol ≈ Black vol everywhere
        let (vol_surface, rf, dy) = make_flat_surface();
        let surface = LocalVolSurface::new(vol_surface, rf, dy, 100.0, Actual365Fixed);

        // Away from t=0, local vol should be ≈ 0.20
        assert_abs_diff_eq!(surface.local_vol_impl(1.0, 100.0), 0.20, epsilon = 0.01);
        assert_abs_diff_eq!(surface.local_vol_impl(2.0, 80.0), 0.20, epsilon = 0.01);
        assert_abs_diff_eq!(surface.local_vol_impl(0.5, 120.0), 0.20, epsilon = 0.01);
    }

    #[test]
    fn local_vol_surface_term_structure() {
        let (vol_surface, rf, dy) = make_flat_surface();
        let surface = LocalVolSurface::new(vol_surface, rf, dy, 100.0, Actual365Fixed);

        assert_eq!(
            surface.reference_date(),
            Date::from_ymd(2025, 1, 2).unwrap()
        );
    }

    #[test]
    fn local_vol_surface_strike_range() {
        let (vol_surface, rf, dy) = make_flat_surface();
        let surface = LocalVolSurface::new(vol_surface, rf, dy, 100.0, Actual365Fixed);

        assert!(surface.min_strike() < 0.0);
        assert!(surface.max_strike() > 1e10);
    }
}
