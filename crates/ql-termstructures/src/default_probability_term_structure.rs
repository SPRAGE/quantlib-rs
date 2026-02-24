//! `DefaultProbabilityTermStructure` — credit / default-probability term
//! structures (translates
//! `ql/termstructures/defaulttermstructure.hpp` and related files).
//!
//! Provides the `DefaultProbabilityTermStructure` trait plus:
//! * `FlatHazardRate` — constant hazard-rate curve
//! * `InterpolatedHazardRateCurve` — piecewise hazard-rate curve

use crate::term_structure::TermStructure;
use crate::yield_term_structure::YieldTermStructureData;
use crate::interpolated_zero_curve::InterpolationBuilder;
use ql_core::{errors::Result, Real, Time};
use ql_math::Interpolation1D;
use ql_time::{Calendar, Date, DayCounter, NullCalendar};
use std::sync::Arc;

/// Probability type alias.
pub type Probability = Real;

/// Hazard rate type alias.
pub type HazardRate = Real;

/// A default-probability term structure.
///
/// Implementors must provide **exactly one** of:
/// * [`survival_probability_impl`](DefaultProbabilityTermStructure::survival_probability_impl)
/// * [`hazard_rate_impl`](DefaultProbabilityTermStructure::hazard_rate_impl)
/// * [`default_density_impl`](DefaultProbabilityTermStructure::default_density_impl)
///
/// The others are derived.
///
/// Corresponds to `QuantLib::DefaultProbabilityTermStructure`.
pub trait DefaultProbabilityTermStructure: TermStructure {
    /// Survival probability `S(t) = P(τ > t)`.
    ///
    /// Default: `exp(−∫₀ᵗ h(s) ds)` where `h` is the hazard rate.
    fn survival_probability_impl(&self, t: Time) -> Probability {
        if t == 0.0 {
            return 1.0;
        }
        let h = self.hazard_rate_impl(t);
        (-h * t).exp()
    }

    /// Default (cumulative) probability `F(t) = 1 − S(t)`.
    fn default_probability_impl(&self, t: Time) -> Probability {
        1.0 - self.survival_probability_impl(t)
    }

    /// Hazard rate `h(t)`.
    ///
    /// Default: `-d ln S(t) / dt`.
    fn hazard_rate_impl(&self, t: Time) -> HazardRate {
        let dt = 1.0e-4_f64;
        let t1 = (t - dt / 2.0).max(0.0);
        let t2 = t + dt / 2.0;
        let s1 = self.survival_probability_impl(t1);
        let s2 = self.survival_probability_impl(t2);
        if s2 <= 0.0 {
            return 0.0;
        }
        (s1.ln() - s2.ln()) / (t2 - t1)
    }

    /// Default density `f(t) = h(t) · S(t)`.
    fn default_density_impl(&self, t: Time) -> Real {
        self.hazard_rate_impl(t) * self.survival_probability_impl(t)
    }

    // ── Public interface ─────────────────────────────────────────────────

    /// Survival probability for a date.
    fn survival_probability(&self, date: Date) -> Probability {
        self.survival_probability_impl(self.time_from_reference(date))
    }

    /// Survival probability for a time.
    fn survival_probability_time(&self, t: Time) -> Probability {
        self.survival_probability_impl(t)
    }

    /// Default probability for a date.
    fn default_probability(&self, date: Date) -> Probability {
        self.default_probability_impl(self.time_from_reference(date))
    }

    /// Default probability for a time.
    fn default_probability_time(&self, t: Time) -> Probability {
        self.default_probability_impl(t)
    }

    /// Hazard rate for a date.
    fn hazard_rate(&self, date: Date) -> HazardRate {
        self.hazard_rate_impl(self.time_from_reference(date))
    }

    /// Hazard rate for a time.
    fn hazard_rate_time(&self, t: Time) -> HazardRate {
        self.hazard_rate_impl(t)
    }
}

// ── FlatHazardRate ────────────────────────────────────────────────────────────

/// A constant hazard-rate default-probability term structure.
///
/// `S(t) = exp(-h·t)` where `h` is a constant hazard rate.
///
/// Corresponds to `QuantLib::FlatHazardRate`.
#[derive(Debug)]
pub struct FlatHazardRate {
    data: YieldTermStructureData,
    hazard_rate: HazardRate,
}

impl FlatHazardRate {
    /// Create a flat hazard-rate curve.
    pub fn new(
        reference_date: Date,
        hazard_rate: HazardRate,
        day_counter: impl DayCounter + 'static,
    ) -> Self {
        Self {
            data: YieldTermStructureData {
                reference_date,
                calendar: Box::new(NullCalendar),
                day_counter: Arc::new(day_counter),
            },
            hazard_rate,
        }
    }

    /// Create with a specific calendar.
    pub fn with_calendar(mut self, calendar: impl Calendar + 'static) -> Self {
        self.data.calendar = Box::new(calendar);
        self
    }

    /// The constant hazard rate.
    pub fn hazard_rate_value(&self) -> HazardRate {
        self.hazard_rate
    }
}

impl TermStructure for FlatHazardRate {
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
        Date::MAX
    }
}

impl DefaultProbabilityTermStructure for FlatHazardRate {
    fn survival_probability_impl(&self, t: Time) -> Probability {
        if t == 0.0 {
            return 1.0;
        }
        (-self.hazard_rate * t).exp()
    }

    fn hazard_rate_impl(&self, _t: Time) -> HazardRate {
        self.hazard_rate
    }

    fn default_density_impl(&self, t: Time) -> Real {
        self.hazard_rate * self.survival_probability_impl(t)
    }
}

// ── InterpolatedHazardRateCurve ───────────────────────────────────────────────

/// A default-probability curve defined by hazard rates at known dates.
///
/// Hazard rates are interpolated as a function of time using a pluggable
/// `InterpolationBuilder`. Survival probability is computed numerically
/// via the trapezoidal rule: `S(t) = exp(-∫₀ᵗ h(s) ds)`.
///
/// Corresponds to `QuantLib::InterpolatedHazardRateCurve<Interpolator>`.
#[derive(Debug)]
pub struct InterpolatedHazardRateCurve {
    data: YieldTermStructureData,
    dates: Vec<Date>,
    times: Vec<Real>,
    hazard_rates: Vec<HazardRate>,
    interp: Box<dyn Interpolation1D>,
    max_date: Date,
}

impl InterpolatedHazardRateCurve {
    /// Build a hazard-rate curve from dates and corresponding hazard rates.
    ///
    /// The first date must be the reference date.
    pub fn new(
        dates: &[Date],
        hazard_rates: &[HazardRate],
        day_counter: impl DayCounter + 'static,
        builder: &dyn InterpolationBuilder,
    ) -> Result<Self> {
        ql_core::ensure!(
            dates.len() >= 2,
            "need at least 2 dates (reference + 1 pillar)"
        );
        ql_core::ensure!(
            dates.len() == hazard_rates.len(),
            "dates and hazard_rates must have the same length"
        );

        let reference_date = dates[0];
        let dc: Arc<dyn DayCounter> = Arc::new(day_counter);

        let times: Vec<Real> = dates
            .iter()
            .map(|&d| dc.year_fraction(reference_date, d))
            .collect();

        let interp = builder.build(&times, hazard_rates)?;
        let max_date = *dates.last().unwrap();

        Ok(Self {
            data: YieldTermStructureData {
                reference_date,
                calendar: Box::new(NullCalendar),
                day_counter: dc,
            },
            dates: dates.to_vec(),
            times,
            hazard_rates: hazard_rates.to_vec(),
            interp,
            max_date,
        })
    }

    /// Set a custom calendar.
    pub fn with_calendar(mut self, calendar: impl Calendar + 'static) -> Self {
        self.data.calendar = Box::new(calendar);
        self
    }

    /// Return the pillar dates.
    pub fn dates(&self) -> &[Date] {
        &self.dates
    }

    /// Return the pillar hazard rates.
    pub fn hazard_rates(&self) -> &[HazardRate] {
        &self.hazard_rates
    }

    /// Integrate the hazard rate from 0 to `t` using the trapezoidal rule.
    fn integrate_hazard(&self, t: Time) -> Real {
        if t <= 0.0 {
            return 0.0;
        }

        let mut integral = 0.0;
        let mut prev_t = 0.0;
        let mut prev_h = self.interp.operator(0.0);

        for &ti in &self.times {
            if ti <= 0.0 {
                continue;
            }
            if ti >= t {
                break;
            }
            let hi = self.interp.operator(ti);
            integral += 0.5 * (prev_h + hi) * (ti - prev_t);
            prev_t = ti;
            prev_h = hi;
        }

        // Final segment
        let ht = self.interp.operator(t);
        integral += 0.5 * (prev_h + ht) * (t - prev_t);

        integral
    }
}

impl TermStructure for InterpolatedHazardRateCurve {
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
        self.max_date
    }
}

impl DefaultProbabilityTermStructure for InterpolatedHazardRateCurve {
    fn hazard_rate_impl(&self, t: Time) -> HazardRate {
        self.interp.operator(t)
    }

    fn survival_probability_impl(&self, t: Time) -> Probability {
        if t == 0.0 {
            return 1.0;
        }
        (-self.integrate_hazard(t)).exp()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpolated_zero_curve::Linear;
    use approx::assert_abs_diff_eq;
    use ql_time::Actual365Fixed;

    #[test]
    fn flat_hazard_survival_at_zero() {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let curve = FlatHazardRate::new(ref_date, 0.02, Actual365Fixed);

        assert_abs_diff_eq!(
            curve.survival_probability_impl(0.0),
            1.0,
            epsilon = 1e-15
        );
    }

    #[test]
    fn flat_hazard_survival() {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let curve = FlatHazardRate::new(ref_date, 0.02, Actual365Fixed);

        // S(5) = exp(-0.02 * 5) = exp(-0.1)
        assert_abs_diff_eq!(
            curve.survival_probability_impl(5.0),
            (-0.1_f64).exp(),
            epsilon = 1e-12
        );
    }

    #[test]
    fn flat_hazard_default_prob() {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let curve = FlatHazardRate::new(ref_date, 0.02, Actual365Fixed);

        let s = curve.survival_probability_impl(5.0);
        let d = curve.default_probability_impl(5.0);
        assert_abs_diff_eq!(s + d, 1.0, epsilon = 1e-15);
    }

    #[test]
    fn flat_hazard_rate_constant() {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let curve = FlatHazardRate::new(ref_date, 0.03, Actual365Fixed);

        assert_abs_diff_eq!(curve.hazard_rate_impl(1.0), 0.03, epsilon = 1e-15);
        assert_abs_diff_eq!(curve.hazard_rate_impl(10.0), 0.03, epsilon = 1e-15);
    }

    #[test]
    fn flat_hazard_density() {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let curve = FlatHazardRate::new(ref_date, 0.02, Actual365Fixed);

        // f(t) = h · S(t) = 0.02 * exp(-0.02 * 3)
        let expected = 0.02 * (-0.06_f64).exp();
        assert_abs_diff_eq!(
            curve.default_density_impl(3.0),
            expected,
            epsilon = 1e-12
        );
    }

    #[test]
    fn interpolated_hazard_at_pillars() {
        let dates = vec![
            Date::from_ymd(2025, 1, 2).unwrap(),
            Date::from_ymd(2026, 1, 2).unwrap(),
            Date::from_ymd(2027, 1, 2).unwrap(),
            Date::from_ymd(2030, 1, 2).unwrap(),
        ];
        let rates = vec![0.01, 0.02, 0.025, 0.03];
        let curve = InterpolatedHazardRateCurve::new(
            &dates,
            &rates,
            Actual365Fixed,
            &Linear,
        )
        .unwrap();

        // At each pillar, the hazard rate should match
        for (i, &d) in dates.iter().enumerate() {
            let t = curve.time_from_reference(d);
            assert_abs_diff_eq!(curve.hazard_rate_impl(t), rates[i], epsilon = 1e-10);
        }
    }

    #[test]
    fn interpolated_hazard_survival_decreases() {
        let dates = vec![
            Date::from_ymd(2025, 1, 2).unwrap(),
            Date::from_ymd(2026, 1, 2).unwrap(),
            Date::from_ymd(2030, 1, 2).unwrap(),
        ];
        let rates = vec![0.02, 0.02, 0.02]; // constant 2%
        let curve = InterpolatedHazardRateCurve::new(
            &dates,
            &rates,
            Actual365Fixed,
            &Linear,
        )
        .unwrap();

        let s1 = curve.survival_probability_impl(1.0);
        let s3 = curve.survival_probability_impl(3.0);
        let s5 = curve.survival_probability_impl(5.0);

        assert!(s1 > s3, "survival should decrease");
        assert!(s3 > s5, "survival should decrease");

        // For flat hazard, should match exp(-h*t)
        assert_abs_diff_eq!(s1, (-0.02_f64).exp(), epsilon = 1e-6);
        assert_abs_diff_eq!(s5, (-0.10_f64).exp(), epsilon = 1e-6);
    }
}
