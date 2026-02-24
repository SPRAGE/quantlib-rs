//! `YieldTermStructure` — yield / interest-rate term structures
//! (translates `ql/termstructures/yieldtermstructure.hpp`).
//!
//! This module defines the `YieldTermStructure` trait together with the three
//! fundamental quantities any yield curve must provide:
//!
//! * **discount factor** — `P(0,t)`
//! * **zero rate** — the continuously-compounded (or other convention) zero
//!   rate for maturity *t*
//! * **forward rate** — the instantaneous or simple forward rate between two
//!   times

use crate::term_structure::TermStructure;
use ql_core::{Compounding, DiscountFactor, Rate, Real, Time};
use ql_time::{Date, DayCounter, Frequency, InterestRate};
use std::sync::Arc;

/// A yield (interest-rate) term structure.
///
/// Implementors must provide **exactly one** of the three low-level methods:
///
/// * [`discount_impl`](YieldTermStructure::discount_impl)
/// * [`zero_rate_impl`](YieldTermStructure::zero_rate_impl)
/// * [`forward_rate_impl`](YieldTermStructure::forward_rate_impl)
///
/// Default implementations of the other two are provided via the
/// mathematical relationships that connect them.
///
/// Corresponds to `QuantLib::YieldTermStructure`.
pub trait YieldTermStructure: TermStructure {
    // ── Low-level impl hooks (override exactly one) ──────────────────────

    /// Return the discount factor for a given time `t`.
    ///
    /// Default: computed from `zero_rate_impl`.
    fn discount_impl(&self, t: Time) -> DiscountFactor {
        if t == 0.0 {
            return 1.0;
        }
        let r = self.zero_rate_impl(t);
        (-r * t).exp()
    }

    /// Return the continuously-compounded zero rate for time `t`.
    ///
    /// Default: computed from `discount_impl`.
    fn zero_rate_impl(&self, t: Time) -> Rate {
        if t == 0.0 {
            // Use the instantaneous forward rate at t=0 as the limit
            return self.forward_rate_impl(0.0);
        }
        let df = self.discount_impl(t);
        -df.ln() / t
    }

    /// Return the instantaneous forward rate at time `t`.
    ///
    /// Default: computed via the negative derivative of log discount.
    /// Uses a central difference approximation `∂ ln P / ∂t`.
    fn forward_rate_impl(&self, t: Time) -> Rate {
        let dt = 1.0e-4_f64;
        let t1 = (t - dt / 2.0).max(0.0);
        let t2 = t + dt / 2.0;
        let df1 = self.discount_impl(t1);
        let df2 = self.discount_impl(t2);
        // -d(ln P)/dt ≈ (ln P(t1) - ln P(t2)) / (t2 - t1)
        (df1.ln() - df2.ln()) / (t2 - t1)
    }

    // ── Public interface ─────────────────────────────────────────────────

    /// Discount factor for a date.
    fn discount_date(&self, date: Date) -> DiscountFactor {
        self.discount_impl(self.time_from_reference(date))
    }

    /// Discount factor for a time.
    fn discount(&self, t: Time) -> DiscountFactor {
        self.discount_impl(t)
    }

    /// Zero rate between the reference date and `date`, expressed under the
    /// given compounding and frequency conventions.
    fn zero_rate(
        &self,
        date: Date,
        dc: &dyn DayCounter,
        comp: Compounding,
        freq: Frequency,
    ) -> InterestRate {
        let t = dc.year_fraction(self.reference_date(), date);
        self.zero_rate_time(t, comp, freq)
    }

    /// Zero rate for time `t`, expressed under the given conventions.
    fn zero_rate_time(
        &self,
        t: Time,
        comp: Compounding,
        freq: Frequency,
    ) -> InterestRate {
        let df = self.discount_impl(t);
        InterestRate::implied_rate_time(if df > 0.0 { 1.0 / df } else { 1.0 }, comp, freq, t)
    }

    /// Forward rate between two dates, expressed under the given conventions.
    fn forward_rate(
        &self,
        d1: Date,
        d2: Date,
        dc: &dyn DayCounter,
        comp: Compounding,
        freq: Frequency,
    ) -> InterestRate {
        let t1 = dc.year_fraction(self.reference_date(), d1);
        let t2 = dc.year_fraction(self.reference_date(), d2);
        self.forward_rate_time(t1, t2, comp, freq)
    }

    /// Forward rate between two times, expressed under the given conventions.
    fn forward_rate_time(
        &self,
        t1: Time,
        t2: Time,
        comp: Compounding,
        freq: Frequency,
    ) -> InterestRate {
        let compound = if t2 == t1 {
            // instantaneous forward
            let r = self.forward_rate_impl(t1);
            (r * DT).exp()
        } else {
            let df1 = self.discount_impl(t1);
            let df2 = self.discount_impl(t2);
            df1 / df2
        };
        InterestRate::implied_rate_time(compound, comp, freq, if t2 == t1 { DT } else { t2 - t1 })
    }
}

/// Small time step used for instantaneous forward rate computations.
const DT: Real = 1.0e-4;

// ── Helpers for concrete term structures ──────────────────────────────────────

/// Common data shared by most yield-curve implementations.
#[derive(Debug)]
pub struct YieldTermStructureData {
    /// Reference date.
    pub reference_date: Date,
    /// Calendar for date adjustments.
    pub calendar: Box<dyn Calendar>,
    /// Day counter for time calculations.
    pub day_counter: Arc<dyn DayCounter>,
}

use ql_time::Calendar;

impl YieldTermStructureData {
    /// Create a new data bundle.
    pub fn new(
        reference_date: Date,
        calendar: impl Calendar + 'static,
        day_counter: impl DayCounter + 'static,
    ) -> Self {
        Self {
            reference_date,
            calendar: Box::new(calendar),
            day_counter: Arc::new(day_counter),
        }
    }
}
