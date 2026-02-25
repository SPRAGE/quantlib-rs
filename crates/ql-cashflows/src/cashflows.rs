//! Cash-flow analysis functions.
//!
//! Translates `ql/cashflows/cashflows.hpp` / `cashflows.cpp`.
//!
//! Static utility functions that operate on a `Leg`:
//! - `npv` — present value
//! - `bps` — basis-point sensitivity
//! - `duration` — Macaulay / modified / simple
//! - `convexity`
//! - `yield_rate` — internal rate of return (solver-based)
//! - `z_spread` — Z-spread over a yield curve
//! - `maturity_date`, `previous_cashflow_date`, `next_cashflow_date`

use crate::cashflow::Leg;
use ql_core::{Compounding, Real};
use ql_math::solvers1d::brent;
use ql_termstructures::YieldTermStructure;
use ql_time::{Actual365Fixed, Date, DayCounter, Frequency, InterestRate};

/// Duration type (Macaulay, Modified, or Simple).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Duration {
    /// Macaulay duration.
    Macaulay,
    /// Modified duration.
    Modified,
    /// Simple (dollar) duration.
    Simple,
}

// ── Leg queries ──────────────────────────────────────────────────────────────

/// The maturity (last payment) date of a leg.
pub fn maturity_date(leg: &Leg) -> Option<Date> {
    leg.iter().map(|cf| cf.date()).max()
}

/// The date of the last cash flow on or before `ref_date`.
pub fn previous_cashflow_date(leg: &Leg, ref_date: Date) -> Option<Date> {
    leg.iter()
        .filter(|cf| cf.date() <= ref_date)
        .map(|cf| cf.date())
        .max()
}

/// The date of the next cash flow strictly after `ref_date`.
pub fn next_cashflow_date(leg: &Leg, ref_date: Date) -> Option<Date> {
    leg.iter()
        .filter(|cf| cf.date() > ref_date)
        .map(|cf| cf.date())
        .min()
}

// ── NPV with a yield curve ──────────────────────────────────────────────────

/// Net present value of a leg using a yield-term-structure.
///
/// Only cash flows after `settlement_date` are discounted.
pub fn npv_curve(leg: &Leg, yield_curve: &dyn YieldTermStructure, settlement_date: Date) -> Real {
    let ref_date = yield_curve.reference_date();
    let dc = Actual365Fixed;
    let mut result = 0.0;
    for cf in leg {
        if cf.date() <= settlement_date {
            continue;
        }
        let t = dc.year_fraction(ref_date, cf.date());
        result += cf.amount() * yield_curve.discount(t);
    }
    result
}

/// Basis-point sensitivity (BPS) using a yield curve.
///
/// This is the change in NPV for a 1bp parallel shift in the discount curve,
/// approximated as: `sum_i(t_i * df_i * amount_i)  * 0.0001`
pub fn bps_curve(leg: &Leg, yield_curve: &dyn YieldTermStructure, settlement_date: Date) -> Real {
    let ref_date = yield_curve.reference_date();
    let dc = Actual365Fixed;
    let mut result = 0.0;
    for cf in leg {
        if cf.date() <= settlement_date {
            continue;
        }
        let t = dc.year_fraction(ref_date, cf.date());
        result += t * yield_curve.discount(t) * cf.amount();
    }
    result * 0.0001
}

// ── NPV with a flat yield ───────────────────────────────────────────────────

/// Net present value of a leg at a flat yield (as an `InterestRate`).
///
/// Cash flows on or before `settlement_date` are excluded.
pub fn npv_yield(leg: &Leg, yield_rate: &InterestRate, settlement_date: Date) -> Real {
    let dc = Actual365Fixed;
    let mut result = 0.0;
    for cf in leg {
        if cf.date() <= settlement_date {
            continue;
        }
        let t = dc.year_fraction(settlement_date, cf.date());
        result += cf.amount() * yield_rate.discount_factor_time(t);
    }
    result
}

/// BPS at a flat yield.
pub fn bps_yield(leg: &Leg, yield_rate: &InterestRate, settlement_date: Date) -> Real {
    let dc = Actual365Fixed;
    let mut result = 0.0;
    for cf in leg {
        if cf.date() <= settlement_date {
            continue;
        }
        let t = dc.year_fraction(settlement_date, cf.date());
        result += t * yield_rate.discount_factor_time(t) * cf.amount();
    }
    result * 0.0001
}

// ── Duration ─────────────────────────────────────────────────────────────────

/// Duration of a leg at a flat yield.
pub fn duration(
    leg: &Leg,
    yield_rate: &InterestRate,
    duration_type: Duration,
    settlement_date: Date,
) -> Real {
    let dc = Actual365Fixed;
    let y = yield_rate.rate();
    let npv = npv_yield(leg, yield_rate, settlement_date);
    if npv.abs() < 1e-30 {
        return 0.0;
    }

    match duration_type {
        Duration::Simple => {
            let mut sum_t_pv = 0.0;
            for cf in leg {
                if cf.date() <= settlement_date {
                    continue;
                }
                let t = dc.year_fraction(settlement_date, cf.date());
                sum_t_pv += t * cf.amount() * yield_rate.discount_factor_time(t);
            }
            sum_t_pv / npv
        }
        Duration::Macaulay => {
            let mut sum_t_pv = 0.0;
            for cf in leg {
                if cf.date() <= settlement_date {
                    continue;
                }
                let t = dc.year_fraction(settlement_date, cf.date());
                sum_t_pv += t * cf.amount() * yield_rate.discount_factor_time(t);
            }
            sum_t_pv / npv
        }
        Duration::Modified => {
            // Modified = Macaulay / (1 + y/k)
            let mac = duration(leg, yield_rate, Duration::Macaulay, settlement_date);
            let k = match yield_rate.frequency() {
                Frequency::NoFrequency | Frequency::Once => 1.0,
                f => f as i32 as f64,
            };
            mac / (1.0 + y / k)
        }
    }
}

// ── Convexity ────────────────────────────────────────────────────────────────

/// Convexity of a leg at a flat yield.
pub fn convexity(leg: &Leg, yield_rate: &InterestRate, settlement_date: Date) -> Real {
    let dc = Actual365Fixed;
    let y = yield_rate.rate();
    let npv = npv_yield(leg, yield_rate, settlement_date);
    if npv.abs() < 1e-30 {
        return 0.0;
    }
    let k = match yield_rate.frequency() {
        Frequency::NoFrequency | Frequency::Once => 1.0,
        f => f as i32 as f64,
    };
    let mut sum = 0.0;
    for cf in leg {
        if cf.date() <= settlement_date {
            continue;
        }
        let t = dc.year_fraction(settlement_date, cf.date());
        let df = yield_rate.discount_factor_time(t);
        sum += t * (t + 1.0 / k) * cf.amount() * df;
    }
    sum / (npv * (1.0 + y / k).powi(2))
}

// ── Yield (IRR) ──────────────────────────────────────────────────────────────

/// Find the yield (internal rate of return) of a leg given its NPV.
///
/// Uses Brent's method to find `r` such that `npv_yield(leg, r) == target_npv`.
pub fn yield_rate(
    leg: &Leg,
    target_npv: Real,
    comp: Compounding,
    freq: Frequency,
    settlement_date: Date,
    accuracy: Real,
) -> ql_core::errors::Result<Real> {
    let f = |r: f64| -> f64 {
        let ir = InterestRate::new(r, Actual365Fixed, comp, freq);
        npv_yield(leg, &ir, settlement_date) - target_npv
    };
    brent(f, -0.10, 2.0, accuracy)
}

// ── Z-spread ─────────────────────────────────────────────────────────────────

/// NPV of a leg discounted with `yield_curve` plus a parallel spread `z`.
pub fn npv_z_spread(
    leg: &Leg,
    yield_curve: &dyn YieldTermStructure,
    z_spread: Real,
    comp: Compounding,
    freq: Frequency,
    settlement_date: Date,
) -> Real {
    let ref_date = yield_curve.reference_date();
    let dc = Actual365Fixed;
    let spread_ir = InterestRate::new(z_spread, Actual365Fixed, comp, freq);
    let mut result = 0.0;
    for cf in leg {
        if cf.date() <= settlement_date {
            continue;
        }
        let t = dc.year_fraction(ref_date, cf.date());
        let base_df = yield_curve.discount(t);
        let spread_df = spread_ir.discount_factor_time(t);
        result += cf.amount() * base_df * spread_df;
    }
    result
}

/// Find the Z-spread such that the NPV equals `target_npv`.
pub fn z_spread(
    leg: &Leg,
    target_npv: Real,
    yield_curve: &dyn YieldTermStructure,
    comp: Compounding,
    freq: Frequency,
    settlement_date: Date,
    accuracy: Real,
) -> ql_core::errors::Result<Real> {
    let f = |z: f64| -> f64 {
        npv_z_spread(leg, yield_curve, z, comp, freq, settlement_date) - target_npv
    };
    brent(f, -0.20, 5.0, accuracy)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixed_rate_coupon::FixedRateLegBuilder;
    use ql_time::{NullCalendar, Period, ScheduleBuilder, TimeUnit};

    fn make_fixed_leg(coupon_rate: Real) -> Leg {
        let start = Date::from_ymd(2025, 1, 15).unwrap();
        let end = Date::from_ymd(2030, 1, 15).unwrap();
        let tenor = Period::new(1, TimeUnit::Years);
        let cal = NullCalendar;
        let schedule = ScheduleBuilder::new(start, end, tenor, &cal)
            .build()
            .unwrap();
        FixedRateLegBuilder::new(&schedule)
            .with_notionals(vec![100.0])
            .with_coupon_rate(coupon_rate)
            .with_compounding(Compounding::Simple)
            .with_frequency(Frequency::Annual)
            .with_redemption(100.0)
            .build()
    }

    #[test]
    fn npv_at_par() {
        let leg = make_fixed_leg(0.05);
        let settlement = Date::from_ymd(2025, 1, 15).unwrap();
        let ir = InterestRate::new(0.05, Actual365Fixed, Compounding::Simple, Frequency::Annual);
        let npv = npv_yield(&leg, &ir, settlement);
        // At par rate, bond should be near 100 (small day-count drift with
        // simple compounding and Actual/365 is normal)
        assert!((npv - 100.0).abs() < 3.0, "npv = {npv}");
    }

    #[test]
    fn npv_above_par_at_lower_yield() {
        let leg = make_fixed_leg(0.05);
        let settlement = Date::from_ymd(2025, 1, 15).unwrap();
        let ir = InterestRate::new(0.03, Actual365Fixed, Compounding::Simple, Frequency::Annual);
        let npv = npv_yield(&leg, &ir, settlement);
        // Lower yield → higher NPV (premium)
        assert!(npv > 100.0, "npv = {npv}");
    }

    #[test]
    fn npv_below_par_at_higher_yield() {
        let leg = make_fixed_leg(0.05);
        let settlement = Date::from_ymd(2025, 1, 15).unwrap();
        let ir = InterestRate::new(0.08, Actual365Fixed, Compounding::Simple, Frequency::Annual);
        let npv = npv_yield(&leg, &ir, settlement);
        // Higher yield → lower NPV (discount)
        assert!(npv < 100.0, "npv = {npv}");
    }

    #[test]
    fn yield_rate_roundtrip() {
        let leg = make_fixed_leg(0.05);
        let settlement = Date::from_ymd(2025, 1, 15).unwrap();
        let ir = InterestRate::new(0.05, Actual365Fixed, Compounding::Simple, Frequency::Annual);
        let target = npv_yield(&leg, &ir, settlement);
        let found = yield_rate(
            &leg,
            target,
            Compounding::Simple,
            Frequency::Annual,
            settlement,
            1e-10,
        )
        .unwrap();
        assert!((found - 0.05).abs() < 1e-6, "found yield = {found}");
    }

    #[test]
    fn duration_positive() {
        let leg = make_fixed_leg(0.05);
        let settlement = Date::from_ymd(2025, 1, 15).unwrap();
        let ir = InterestRate::new(0.05, Actual365Fixed, Compounding::Simple, Frequency::Annual);
        let mac = duration(&leg, &ir, Duration::Macaulay, settlement);
        assert!(mac > 0.0, "Macaulay duration = {mac}");
        assert!(mac < 5.5, "Macaulay duration = {mac}");
        let modif = duration(&leg, &ir, Duration::Modified, settlement);
        assert!(modif > 0.0);
        assert!(modif <= mac, "modified = {modif}, macaulay = {mac}");
    }

    #[test]
    fn convexity_positive() {
        let leg = make_fixed_leg(0.05);
        let settlement = Date::from_ymd(2025, 1, 15).unwrap();
        let ir = InterestRate::new(0.05, Actual365Fixed, Compounding::Simple, Frequency::Annual);
        let c = convexity(&leg, &ir, settlement);
        assert!(c > 0.0, "convexity = {c}");
    }

    #[test]
    fn maturity_date_test() {
        let leg = make_fixed_leg(0.05);
        let mat = maturity_date(&leg).unwrap();
        assert_eq!(mat, Date::from_ymd(2030, 1, 15).unwrap());
    }

    #[test]
    fn next_prev_cashflow() {
        let leg = make_fixed_leg(0.05);
        let ref_date = Date::from_ymd(2027, 6, 1).unwrap();
        let prev = previous_cashflow_date(&leg, ref_date).unwrap();
        let next = next_cashflow_date(&leg, ref_date).unwrap();
        assert!(prev <= ref_date);
        assert!(next > ref_date);
    }
}
