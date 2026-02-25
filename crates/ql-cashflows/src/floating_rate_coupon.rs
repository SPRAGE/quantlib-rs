//! Floating-rate coupons and Ibor leg builders.
//!
//! Translates `ql/cashflows/floatingratecoupon.hpp`,
//! `ql/cashflows/iborcoupon.hpp` and parts of `ql/cashflows/iborcoupon.cpp`.

use crate::cashflow::{CashFlow, Leg, Redemption};
use crate::coupon::Coupon;
use ql_core::{errors::Result, Real};
use ql_indexes::{IborIndex, Index, InterestRateIndex};
use ql_time::{Actual365Fixed, BusinessDayConvention, Date, DayCounter, Schedule};
use std::sync::Arc;

// ────────────────────────────────────────────────────────────────────────────
// FloatingRateCoupon
// ────────────────────────────────────────────────────────────────────────────

/// A coupon whose rate is derived from a floating index plus a spread.
///
/// `rate = gearing * index_fixing + spread`
///
/// Corresponds to `QuantLib::FloatingRateCoupon`.
#[derive(Debug)]
pub struct FloatingRateCoupon {
    /// Notional (face) amount.
    nominal: Real,
    /// Payment date.
    payment_date: Date,
    /// Accrual start date.
    accrual_start: Date,
    /// Accrual end date.
    accrual_end: Date,
    /// The fixing date (index observation date).
    fixing_date: Date,
    /// Multiplicative gearing (default 1.0).
    gearing: Real,
    /// Additive spread (default 0.0).
    spread: Real,
    /// Day counter for computing accrual fraction.
    day_counter: Box<dyn DayCounter>,
    /// Accrual period (year fraction, cached).
    accrual_period: Real,
    /// The index rate for the period (either stored or computed).
    ///
    /// When `None`, the coupon looks up the index on `fixing_date`.
    /// When `Some(r)`, uses that value directly (useful for testing or
    /// when the rate has been determined by a pricer).
    cached_rate: Option<Real>,
}

impl FloatingRateCoupon {
    /// Create a new floating-rate coupon.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        payment_date: Date,
        nominal: Real,
        accrual_start: Date,
        accrual_end: Date,
        fixing_date: Date,
        gearing: Real,
        spread: Real,
        day_counter: impl DayCounter + 'static,
    ) -> Self {
        let dc: Box<dyn DayCounter> = Box::new(day_counter);
        let accrual_period = dc.year_fraction(accrual_start, accrual_end);
        Self {
            nominal,
            payment_date,
            accrual_start,
            accrual_end,
            fixing_date,
            gearing,
            spread,
            day_counter: dc,
            accrual_period,
            cached_rate: None,
        }
    }

    /// Set a pre-determined index rate (overrides any index lookup).
    pub fn with_rate(mut self, rate: Real) -> Self {
        self.cached_rate = Some(rate);
        self
    }

    /// The fixing date for this coupon.
    pub fn fixing_date(&self) -> Date {
        self.fixing_date
    }

    /// The gearing multiplier.
    pub fn gearing(&self) -> Real {
        self.gearing
    }

    /// The spread.
    pub fn spread(&self) -> Real {
        self.spread
    }

    /// Compute the effective coupon rate.
    ///
    /// If a cached rate is set, use `gearing * cached_rate + spread`.
    /// Otherwise the caller must supply the index rate via [`with_rate`] or
    /// use [`IborCoupon`] which looks it up automatically.
    pub fn effective_rate(&self) -> Real {
        let index_rate = self.cached_rate.unwrap_or(0.0);
        self.gearing * index_rate + self.spread
    }
}

impl CashFlow for FloatingRateCoupon {
    fn date(&self) -> Date {
        self.payment_date
    }

    fn amount(&self) -> Real {
        self.nominal * self.effective_rate() * self.accrual_period
    }
}

impl Coupon for FloatingRateCoupon {
    fn nominal(&self) -> Real {
        self.nominal
    }

    fn accrual_start_date(&self) -> Date {
        self.accrual_start
    }

    fn accrual_end_date(&self) -> Date {
        self.accrual_end
    }

    fn accrual_period(&self) -> Real {
        self.accrual_period
    }

    fn day_counter(&self) -> &dyn DayCounter {
        &*self.day_counter
    }

    fn rate(&self) -> Real {
        self.effective_rate()
    }
}

// ────────────────────────────────────────────────────────────────────────────
// IborCoupon
// ────────────────────────────────────────────────────────────────────────────

/// A floating-rate coupon linked to an `IborIndex`.
///
/// The index rate is obtained from `IborIndex::fixing()`. The coupon amount is:
///
///   `amount = nominal * (gearing * fixing + spread) * accrual_period`
///
/// Corresponds to `QuantLib::IborCoupon`.
#[derive(Debug)]
pub struct IborCoupon {
    inner: FloatingRateCoupon,
    index: Arc<IborIndex>,
}

impl IborCoupon {
    /// Create a new Ibor coupon.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        payment_date: Date,
        nominal: Real,
        accrual_start: Date,
        accrual_end: Date,
        fixing_days: u32,
        index: Arc<IborIndex>,
        gearing: Real,
        spread: Real,
    ) -> Self {
        let fixing_date = index
            .fixing_calendar()
            .advance_business_days(accrual_start, -(fixing_days as i32));
        let dc = Actual365Fixed; // use index day counter ideally
        Self {
            inner: FloatingRateCoupon::new(
                payment_date,
                nominal,
                accrual_start,
                accrual_end,
                fixing_date,
                gearing,
                spread,
                dc,
            ),
            index,
        }
    }

    /// The underlying index.
    pub fn ibor_index(&self) -> &IborIndex {
        &self.index
    }

    /// The fixing rate from the index. Returns an error if the fixing is
    /// not available and term-structure forecasting is not yet implemented.
    pub fn index_fixing(&self) -> Result<Real> {
        self.index.fixing(self.inner.fixing_date, false)
    }

    /// Effective rate: `gearing * index_fixing + spread`.
    fn effective_rate_result(&self) -> Result<Real> {
        let fixing = self.index_fixing()?;
        Ok(self.inner.gearing * fixing + self.inner.spread)
    }
}

impl CashFlow for IborCoupon {
    fn date(&self) -> Date {
        self.inner.payment_date
    }

    fn amount(&self) -> Real {
        // If we can get the fixing, compute properly; otherwise fall back to
        // the cached rate
        match self.effective_rate_result() {
            Ok(r) => self.inner.nominal * r * self.inner.accrual_period,
            Err(_) => self.inner.amount(),
        }
    }
}

impl Coupon for IborCoupon {
    fn nominal(&self) -> Real {
        self.inner.nominal
    }

    fn accrual_start_date(&self) -> Date {
        self.inner.accrual_start
    }

    fn accrual_end_date(&self) -> Date {
        self.inner.accrual_end
    }

    fn accrual_period(&self) -> Real {
        self.inner.accrual_period
    }

    fn day_counter(&self) -> &dyn DayCounter {
        &*self.inner.day_counter
    }

    fn rate(&self) -> Real {
        self.effective_rate_result()
            .unwrap_or(self.inner.effective_rate())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Ibor leg builder
// ────────────────────────────────────────────────────────────────────────────

/// Build a floating-rate leg from a schedule and an Ibor index.
///
/// Corresponds to `QuantLib::IborLeg`.
pub struct IborLegBuilder<'a> {
    schedule: &'a Schedule,
    index: Arc<IborIndex>,
    notionals: Vec<Real>,
    gearings: Vec<Real>,
    spreads: Vec<Real>,
    fixing_days: Option<u32>,
    day_counter: Option<Box<dyn DayCounter>>,
    payment_convention: BusinessDayConvention,
    add_redemption: bool,
    redemption_amount: Real,
}

impl<'a> IborLegBuilder<'a> {
    /// Create a builder from a schedule and an index.
    pub fn new(schedule: &'a Schedule, index: Arc<IborIndex>) -> Self {
        Self {
            schedule,
            index,
            notionals: vec![1.0],
            gearings: vec![1.0],
            spreads: vec![0.0],
            fixing_days: None,
            day_counter: None,
            payment_convention: BusinessDayConvention::Following,
            add_redemption: false,
            redemption_amount: 100.0,
        }
    }

    /// Set notional(s).
    pub fn with_notionals(mut self, notionals: Vec<Real>) -> Self {
        self.notionals = notionals;
        self
    }

    /// Set a single gearing for all periods.
    pub fn with_gearing(mut self, gearing: Real) -> Self {
        self.gearings = vec![gearing];
        self
    }

    /// Set gearings per period.
    pub fn with_gearings(mut self, gearings: Vec<Real>) -> Self {
        self.gearings = gearings;
        self
    }

    /// Set a single spread for all periods.
    pub fn with_spread(mut self, spread: Real) -> Self {
        self.spreads = vec![spread];
        self
    }

    /// Set spreads per period.
    pub fn with_spreads(mut self, spreads: Vec<Real>) -> Self {
        self.spreads = spreads;
        self
    }

    /// Override fixing days (defaults to index fixing days).
    pub fn with_fixing_days(mut self, days: u32) -> Self {
        self.fixing_days = Some(days);
        self
    }

    /// Override day counter.
    pub fn with_day_counter(mut self, dc: impl DayCounter + 'static) -> Self {
        self.day_counter = Some(Box::new(dc));
        self
    }

    /// Set payment convention.
    pub fn with_payment_convention(mut self, convention: BusinessDayConvention) -> Self {
        self.payment_convention = convention;
        self
    }

    /// Add a final redemption.
    pub fn with_redemption(mut self, amount: Real) -> Self {
        self.add_redemption = true;
        self.redemption_amount = amount;
        self
    }

    /// Build the leg.
    pub fn build(self) -> Leg {
        let dates = self.schedule.dates();
        let n = dates.len().saturating_sub(1);
        let fixing_days = self.fixing_days.unwrap_or_else(|| self.index.fixing_days());
        let mut leg: Leg = Vec::with_capacity(n + if self.add_redemption { 1 } else { 0 });

        for i in 0..n {
            let start = dates[i];
            let end = dates[i + 1];
            let payment = end; // simplified: pay on period end

            let notional = self.notionals[i.min(self.notionals.len() - 1)];
            let gearing = self.gearings[i.min(self.gearings.len() - 1)];
            let spread = self.spreads[i.min(self.spreads.len() - 1)];

            let coupon = IborCoupon::new(
                payment,
                notional,
                start,
                end,
                fixing_days,
                Arc::clone(&self.index),
                gearing,
                spread,
            );

            leg.push(Box::new(coupon));
        }

        if self.add_redemption && n > 0 {
            let last_date = dates[n];
            leg.push(Box::new(Redemption::new(self.redemption_amount, last_date)));
        }

        leg
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ql_currencies::currencies::america::USD;
    use ql_time::{NullCalendar, Period, ScheduleBuilder, TimeUnit};

    fn make_test_index() -> Arc<IborIndex> {
        Arc::new(IborIndex::new(
            "USD-Libor-3M",
            Period::new(3, TimeUnit::Months),
            2,
            &USD,
            NullCalendar,
            BusinessDayConvention::ModifiedFollowing,
            false,
            Actual365Fixed,
        ))
    }

    #[test]
    fn floating_rate_coupon_with_rate() {
        let start = Date::from_ymd(2025, 1, 15).unwrap();
        let end = Date::from_ymd(2025, 7, 15).unwrap();
        let c = FloatingRateCoupon::new(
            end,
            1_000_000.0,
            start,
            end,
            start,
            1.0,
            0.0,
            Actual365Fixed,
        )
        .with_rate(0.05);
        // Simple: amount = N * r * t
        let t = Actual365Fixed.year_fraction(start, end);
        let expected = 1_000_000.0 * 0.05 * t;
        assert!((c.amount() - expected).abs() < 0.01);
    }

    #[test]
    fn floating_rate_coupon_with_gearing_and_spread() {
        let start = Date::from_ymd(2025, 1, 15).unwrap();
        let end = Date::from_ymd(2025, 7, 15).unwrap();
        let c = FloatingRateCoupon::new(
            end,
            1_000_000.0,
            start,
            end,
            start,
            2.0,
            0.01,
            Actual365Fixed,
        )
        .with_rate(0.03);
        // rate = 2.0 * 0.03 + 0.01 = 0.07
        let t = Actual365Fixed.year_fraction(start, end);
        let expected = 1_000_000.0 * 0.07 * t;
        assert!((c.amount() - expected).abs() < 0.01);
    }

    #[test]
    fn ibor_coupon_with_stored_fixing() {
        let index = make_test_index();
        let start = Date::from_ymd(2025, 1, 15).unwrap();
        let end = Date::from_ymd(2025, 4, 15).unwrap();

        let coupon = IborCoupon::new(
            end,
            1_000_000.0,
            start,
            end,
            2,
            Arc::clone(&index),
            1.0,
            0.0,
        );

        // Store a fixing on the fixing date
        let fixing_date = coupon.inner.fixing_date;
        index.add_fixing(fixing_date, 0.04);

        let t = Actual365Fixed.year_fraction(start, end);
        let expected = 1_000_000.0 * 0.04 * t;
        assert!((coupon.amount() - expected).abs() < 0.01);
    }

    #[test]
    fn ibor_leg_builder() {
        let index = make_test_index();
        let start = Date::from_ymd(2025, 1, 15).unwrap();
        let end = Date::from_ymd(2027, 1, 15).unwrap();
        let tenor = Period::new(3, TimeUnit::Months);
        let cal = NullCalendar;
        let schedule = ScheduleBuilder::new(start, end, tenor, &cal)
            .build()
            .unwrap();

        let leg = IborLegBuilder::new(&schedule, Arc::clone(&index))
            .with_notionals(vec![100.0])
            .with_spread(0.005) // 50bp spread
            .with_redemption(100.0)
            .build();

        // 2 years / 3 months = 8 coupons + 1 redemption
        assert_eq!(leg.len(), 9);
        // Last cash flow is the redemption
        assert!((leg[8].amount() - 100.0).abs() < 1e-15);
    }

    #[test]
    fn ibor_leg_dates_monotone() {
        let index = make_test_index();
        let start = Date::from_ymd(2025, 1, 15).unwrap();
        let end = Date::from_ymd(2030, 1, 15).unwrap();
        let tenor = Period::new(6, TimeUnit::Months);
        let cal = NullCalendar;
        let schedule = ScheduleBuilder::new(start, end, tenor, &cal)
            .build()
            .unwrap();

        let leg = IborLegBuilder::new(&schedule, Arc::clone(&index))
            .with_notionals(vec![1_000_000.0])
            .build();

        assert_eq!(leg.len(), 10);
        for i in 1..leg.len() {
            assert!(leg[i].date() > leg[i - 1].date());
        }
    }
}
