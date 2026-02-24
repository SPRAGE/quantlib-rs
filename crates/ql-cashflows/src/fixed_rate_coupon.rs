//! Fixed-rate coupons and leg builders.
//!
//! Translates `ql/cashflows/fixedratecoupon.hpp` and parts of
//! `ql/cashflows/fixedratecoupon.cpp`.

use crate::cashflow::{CashFlow, Leg, Redemption};
use crate::coupon::Coupon;
use ql_core::{Compounding, Real};
use ql_time::{
    Actual365Fixed, BusinessDayConvention, Date, DayCounter, Frequency, InterestRate, Schedule,
};

/// A coupon paying a fixed interest rate.
///
/// Corresponds to `QuantLib::FixedRateCoupon`.
#[derive(Debug)]
pub struct FixedRateCoupon {
    /// Notional (face) amount.
    nominal: Real,
    /// Payment date.
    payment_date: Date,
    /// The fixed interest rate (as `InterestRate`).
    rate: InterestRate,
    /// Accrual start date.
    accrual_start: Date,
    /// Accrual end date.
    accrual_end: Date,
    /// Reference period start (for irregular first/last coupons).
    ref_start: Date,
    /// Reference period end.
    ref_end: Date,
    /// Day counter (cached from rate).
    day_counter: Box<dyn DayCounter>,
    /// Accrual period (year fraction).
    accrual_period: Real,
}

impl FixedRateCoupon {
    /// Create a new fixed-rate coupon.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        payment_date: Date,
        nominal: Real,
        rate: InterestRate,
        accrual_start: Date,
        accrual_end: Date,
        ref_start: Date,
        ref_end: Date,
    ) -> Self {
        let dc = rate.day_counter();
        let accrual_period = dc.year_fraction(accrual_start, accrual_end);
        Self {
            nominal,
            payment_date,
            rate,
            accrual_start,
            accrual_end,
            ref_start,
            ref_end,
            day_counter: Box::new(Actual365Fixed), // placeholder, used for accrued
            accrual_period,
        }
    }

    /// Convenience: create from a simple rate, defaulting to continuous compounding.
    pub fn from_rate(
        payment_date: Date,
        nominal: Real,
        rate_value: Real,
        day_counter: impl DayCounter + 'static,
        accrual_start: Date,
        accrual_end: Date,
    ) -> Self {
        let ir = InterestRate::new(
            rate_value,
            Actual365Fixed,
            Compounding::Simple,
            Frequency::Annual,
        );
        let accrual_period = day_counter.year_fraction(accrual_start, accrual_end);
        Self {
            nominal,
            payment_date,
            rate: ir,
            accrual_start,
            accrual_end,
            ref_start: accrual_start,
            ref_end: accrual_end,
            day_counter: Box::new(day_counter),
            accrual_period,
        }
    }

    /// The coupon's `InterestRate`.
    pub fn interest_rate(&self) -> &InterestRate {
        &self.rate
    }
}

impl CashFlow for FixedRateCoupon {
    fn date(&self) -> Date {
        self.payment_date
    }

    fn amount(&self) -> Real {
        // amount = nominal * (compound_factor - 1)
        self.nominal * (self.rate.compound_factor_time(self.accrual_period) - 1.0)
    }
}

impl Coupon for FixedRateCoupon {
    fn nominal(&self) -> Real {
        self.nominal
    }

    fn accrual_start_date(&self) -> Date {
        self.accrual_start
    }

    fn accrual_end_date(&self) -> Date {
        self.accrual_end
    }

    fn reference_period_start(&self) -> Date {
        self.ref_start
    }

    fn reference_period_end(&self) -> Date {
        self.ref_end
    }

    fn accrual_period(&self) -> Real {
        self.accrual_period
    }

    fn day_counter(&self) -> &dyn DayCounter {
        &*self.day_counter
    }

    fn rate(&self) -> Real {
        self.rate.rate()
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Fixed-rate leg builder
// ────────────────────────────────────────────────────────────────────────────

/// Build a fixed-rate leg from a schedule and rate.
///
/// Corresponds to `QuantLib::FixedRateLeg`.
pub struct FixedRateLegBuilder<'a> {
    schedule: &'a Schedule,
    notionals: Vec<Real>,
    coupon_rates: Vec<Real>,
    compounding: Compounding,
    frequency: Frequency,
    day_counter: Box<dyn DayCounter>,
    payment_convention: BusinessDayConvention,
    add_redemption: bool,
    redemption_amount: Real,
}

impl<'a> FixedRateLegBuilder<'a> {
    /// Create a new builder from a schedule.
    pub fn new(schedule: &'a Schedule) -> Self {
        Self {
            schedule,
            notionals: vec![1.0],
            coupon_rates: vec![0.0],
            compounding: Compounding::Simple,
            frequency: Frequency::Annual,
            day_counter: Box::new(Actual365Fixed),
            payment_convention: BusinessDayConvention::Following,
            add_redemption: false,
            redemption_amount: 100.0,
        }
    }

    /// Set the notional(s). The last value is extended for all remaining periods.
    pub fn with_notionals(mut self, notionals: Vec<Real>) -> Self {
        self.notionals = notionals;
        self
    }

    /// Set a single coupon rate for all periods.
    pub fn with_coupon_rate(mut self, rate: Real) -> Self {
        self.coupon_rates = vec![rate];
        self
    }

    /// Set coupon rates per period.
    pub fn with_coupon_rates(mut self, rates: Vec<Real>) -> Self {
        self.coupon_rates = rates;
        self
    }

    /// Set the compounding convention.
    pub fn with_compounding(mut self, compounding: Compounding) -> Self {
        self.compounding = compounding;
        self
    }

    /// Set the frequency.
    pub fn with_frequency(mut self, frequency: Frequency) -> Self {
        self.frequency = frequency;
        self
    }

    /// Set the day counter.
    pub fn with_day_counter(mut self, dc: impl DayCounter + 'static) -> Self {
        self.day_counter = Box::new(dc);
        self
    }

    /// Set the payment business day convention.
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
        let n = dates.len().saturating_sub(1); // number of periods
        let mut leg: Leg = Vec::with_capacity(n + if self.add_redemption { 1 } else { 0 });

        for i in 0..n {
            let start = dates[i];
            let end = dates[i + 1];
            let payment = end; // simplified: pay on period end

            let notional = self.notionals[i.min(self.notionals.len() - 1)];
            let coupon_rate = self.coupon_rates[i.min(self.coupon_rates.len() - 1)];

            let ir = InterestRate::new(
                coupon_rate,
                Actual365Fixed,
                self.compounding,
                self.frequency,
            );

            leg.push(Box::new(FixedRateCoupon::new(
                payment, notional, ir, start, end, start, end,
            )));
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
    use ql_time::{Actual365Fixed, NullCalendar, Period, ScheduleBuilder, TimeUnit};

    #[test]
    fn fixed_rate_coupon_amount() {
        let start = Date::from_ymd(2025, 1, 15).unwrap();
        let end = Date::from_ymd(2025, 7, 15).unwrap();
        let ir = InterestRate::new(0.05, Actual365Fixed, Compounding::Simple, Frequency::Annual);
        let c = FixedRateCoupon::new(end, 1_000_000.0, ir, start, end, start, end);
        // simple: amount = N * r * t = 1e6 * 0.05 * 181/365
        let t = Actual365Fixed.year_fraction(start, end);
        let expected = 1_000_000.0 * 0.05 * t;
        assert!((c.amount() - expected).abs() < 0.01);
    }

    #[test]
    fn fixed_rate_coupon_rate() {
        let start = Date::from_ymd(2025, 1, 15).unwrap();
        let end = Date::from_ymd(2025, 7, 15).unwrap();
        let ir = InterestRate::new(0.05, Actual365Fixed, Compounding::Simple, Frequency::Annual);
        let c = FixedRateCoupon::new(end, 1_000_000.0, ir, start, end, start, end);
        assert!((c.rate() - 0.05).abs() < 1e-15);
    }

    #[test]
    fn fixed_rate_leg_builder() {
        let start = Date::from_ymd(2025, 1, 15).unwrap();
        let end = Date::from_ymd(2027, 1, 15).unwrap();
        let tenor = Period::new(6, TimeUnit::Months);
        let cal = NullCalendar;
        let schedule = ScheduleBuilder::new(start, end, tenor, &cal).build().unwrap();

        let leg = FixedRateLegBuilder::new(&schedule)
            .with_notionals(vec![100.0])
            .with_coupon_rate(0.05)
            .with_redemption(100.0)
            .build();

        // 2 years / 6 months = 4 coupons + 1 redemption = 5 cash flows
        assert_eq!(leg.len(), 5);
        // All coupons have positive amount
        for cf in &leg[..4] {
            assert!(cf.amount() > 0.0);
        }
        // Last cash flow is the redemption
        assert!((leg[4].amount() - 100.0).abs() < 1e-15);
    }

    #[test]
    fn fixed_rate_leg_dates_monotone() {
        let start = Date::from_ymd(2025, 1, 15).unwrap();
        let end = Date::from_ymd(2030, 1, 15).unwrap();
        let tenor = Period::new(1, TimeUnit::Years);
        let cal = NullCalendar;
        let schedule = ScheduleBuilder::new(start, end, tenor, &cal).build().unwrap();

        let leg = FixedRateLegBuilder::new(&schedule)
            .with_coupon_rate(0.03)
            .build();

        assert_eq!(leg.len(), 5);
        for i in 1..leg.len() {
            assert!(leg[i].date() > leg[i - 1].date());
        }
    }
}
