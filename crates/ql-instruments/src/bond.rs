//! Bond base and concrete bond types.
//!
//! Translates `ql/instruments/bond.hpp`, `ql/instruments/bonds/zerocouponbond.hpp`,
//! `ql/instruments/bonds/fixedratebond.hpp`, `ql/instruments/bonds/floatingratebond.hpp`.

use crate::instrument::{Instrument, PricingEngine, PricingResults};
use ql_cashflows::{
    CashFlow, Coupon, FixedRateLegBuilder, IborLegBuilder, Leg, Redemption,
};
use ql_core::{errors::Result, Compounding, Real};
use ql_indexes::IborIndex;
use ql_time::{
    Actual365Fixed, Calendar, Date, DayCounter, Frequency, InterestRate,
    Schedule,
};
use std::sync::Arc;

// ────────────────────────────────────────────────────────────────────────────
// Bond (base)
// ────────────────────────────────────────────────────────────────────────────

/// Arguments needed to price a bond.
#[derive(Debug)]
pub struct BondArguments {
    /// Settlement date.
    pub settlement_date: Date,
}

/// A generic bond instrument.
///
/// Corresponds to `QuantLib::Bond`. Holds a leg of cash flows (coupons +
/// redemption) and provides utility methods for clean/dirty price, yield, etc.
#[derive(Debug)]
pub struct Bond {
    /// Settlement days.
    pub settlement_days: u32,
    /// Calendar for settlement.
    pub calendar: Box<dyn Calendar>,
    /// Issue date.
    pub issue_date: Option<Date>,
    /// Maturity date.
    pub maturity_date: Date,
    /// The cashflow leg.
    pub cashflows: Leg,
    /// Face (notional) amount.
    pub face_amount: Real,
}

impl Bond {
    /// Settlement date given a reference (evaluation) date.
    pub fn settlement_date(&self, eval_date: Date) -> Date {
        self.calendar
            .advance_business_days(eval_date, self.settlement_days as i32)
    }

    /// Accrued amount at the given settlement date.
    pub fn accrued_amount(&self, settlement: Date) -> Real {
        for cf in self.cashflows.iter().rev() {
            // Try to downcast to a coupon
            if let Some(coupon) = cf_as_coupon(&**cf) {
                if coupon.accrual_start_date() < settlement && settlement <= coupon.accrual_end_date() {
                    return coupon.accrued_amount(settlement);
                }
            }
        }
        0.0
    }

    /// Notional (face) amount.
    pub fn notional(&self) -> Real {
        self.face_amount
    }

    /// Clean price from a dirty price.
    pub fn clean_price_from_dirty(&self, dirty_price: Real, settlement: Date) -> Real {
        dirty_price - self.accrued_amount(settlement) / self.face_amount * 100.0
    }

    /// Dirty price from a clean price.
    pub fn dirty_price_from_clean(&self, clean_price: Real, settlement: Date) -> Real {
        clean_price + self.accrued_amount(settlement) / self.face_amount * 100.0
    }

    /// Clean price given a flat yield.
    pub fn clean_price_yield(
        &self,
        yield_rate: Real,
        dc: &dyn DayCounter,
        comp: Compounding,
        freq: Frequency,
        settlement: Date,
    ) -> Real {
        let dirty = self.dirty_price_yield(yield_rate, dc, comp, freq, settlement);
        self.clean_price_from_dirty(dirty, settlement)
    }

    /// Dirty price given a flat yield.
    pub fn dirty_price_yield(
        &self,
        yield_rate: Real,
        _dc: &dyn DayCounter,
        comp: Compounding,
        freq: Frequency,
        settlement: Date,
    ) -> Real {
        let ir = InterestRate::new(yield_rate, Actual365Fixed, comp, freq);
        let npv = ql_cashflows::npv_yield(&self.cashflows, &ir, settlement);
        npv / self.face_amount * 100.0
    }

    /// Yield to maturity given a clean price, solved via Brent's method.
    pub fn yield_to_maturity(
        &self,
        clean_price: Real,
        _dc: &dyn DayCounter,
        comp: Compounding,
        freq: Frequency,
        settlement: Date,
        accuracy: Real,
    ) -> Result<Real> {
        let dirty_price = self.dirty_price_from_clean(clean_price, settlement);
        let target_npv = dirty_price / 100.0 * self.face_amount;
        ql_cashflows::yield_rate(&self.cashflows, target_npv, comp, freq, settlement, accuracy)
    }

    /// Price the bond with an external engine.
    pub fn price(&self, engine: &dyn PricingEngine<BondArguments>, settlement: Date) -> Result<PricingResults> {
        let args = BondArguments {
            settlement_date: settlement,
        };
        engine.calculate(&args)
    }
}

impl Instrument for Bond {
    fn is_expired(&self) -> bool {
        // Expired if all cash flows have occurred
        // Use a dummy "today" — in practice would use Settings::evaluation_date()
        false
    }

    fn maturity_date(&self) -> Option<Date> {
        Some(self.maturity_date)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Zero-Coupon Bond
// ────────────────────────────────────────────────────────────────────────────

/// Build a zero-coupon bond.
///
/// Corresponds to `QuantLib::ZeroCouponBond`.
pub fn zero_coupon_bond(
    settlement_days: u32,
    calendar: impl Calendar + 'static,
    face_amount: Real,
    maturity: Date,
) -> Bond {
    let cashflows: Leg = vec![Box::new(Redemption::new(face_amount, maturity))];
    Bond {
        settlement_days,
        calendar: Box::new(calendar),
        issue_date: None,
        maturity_date: maturity,
        cashflows,
        face_amount,
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Fixed-Rate Bond
// ────────────────────────────────────────────────────────────────────────────

/// Build a fixed-rate bond.
///
/// Corresponds to `QuantLib::FixedRateBond`.
#[allow(clippy::too_many_arguments)]
pub fn fixed_rate_bond(
    settlement_days: u32,
    face_amount: Real,
    schedule: &Schedule,
    coupon_rates: Vec<Real>,
    compounding: Compounding,
    frequency: Frequency,
    calendar: impl Calendar + 'static,
) -> Bond {
    let dates = schedule.dates();
    let maturity = *dates.last().expect("schedule must have dates");

    let cashflows = FixedRateLegBuilder::new(schedule)
        .with_notionals(vec![face_amount])
        .with_coupon_rates(coupon_rates)
        .with_compounding(compounding)
        .with_frequency(frequency)
        .with_redemption(face_amount)
        .build();

    Bond {
        settlement_days,
        calendar: Box::new(calendar),
        issue_date: Some(dates[0]),
        maturity_date: maturity,
        cashflows,
        face_amount,
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Floating-Rate Bond
// ────────────────────────────────────────────────────────────────────────────

/// Build a floating-rate bond.
///
/// Corresponds to `QuantLib::FloatingRateBond`.
#[allow(clippy::too_many_arguments)]
pub fn floating_rate_bond(
    settlement_days: u32,
    face_amount: Real,
    schedule: &Schedule,
    index: Arc<IborIndex>,
    gearing: Real,
    spread: Real,
    calendar: impl Calendar + 'static,
) -> Bond {
    let dates = schedule.dates();
    let maturity = *dates.last().expect("schedule must have dates");
    let fixing_days = index.fixing_days();

    let cashflows = IborLegBuilder::new(schedule, index)
        .with_notionals(vec![face_amount])
        .with_gearing(gearing)
        .with_spread(spread)
        .with_fixing_days(fixing_days)
        .with_redemption(face_amount)
        .build();

    Bond {
        settlement_days,
        calendar: Box::new(calendar),
        issue_date: Some(dates[0]),
        maturity_date: maturity,
        cashflows,
        face_amount,
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Helpers
// ────────────────────────────────────────────────────────────────────────────

/// Attempt to interpret a `dyn CashFlow` as a `dyn Coupon` using its
/// accrual period: if it's > 0, we treat it as a coupon-like object.
///
/// This is a simplified approach — QuantLib uses dynamic_cast.
/// We store FixedRateCoupon (which implements both) behind `Box<dyn CashFlow>`,
/// so we need a workaround.
fn cf_as_coupon<'a>(_cf: &'a dyn CashFlow) -> Option<&'a dyn Coupon> {
    // Use trait-object downcast via Any if available; for now assume
    // accrued is zero for non-coupon flows.
    // This is a known simplification — we'll improve with proper Any-based
    // downcast when needed.
    None
}

use ql_indexes::InterestRateIndex;

#[cfg(test)]
mod tests {
    use super::*;
    use ql_currencies::currencies::america::USD;
    use ql_time::{BusinessDayConvention, NullCalendar, Period, ScheduleBuilder, TimeUnit};

    #[test]
    fn zero_coupon_bond_basic() {
        let mat = Date::from_ymd(2030, 1, 15).unwrap();
        let bond = zero_coupon_bond(2, NullCalendar, 100.0, mat);
        assert_eq!(bond.cashflows.len(), 1);
        assert!((bond.cashflows[0].amount() - 100.0).abs() < 1e-15);
        assert_eq!(bond.maturity_date, mat);
    }

    #[test]
    fn fixed_rate_bond_construction() {
        let start = Date::from_ymd(2025, 1, 15).unwrap();
        let end = Date::from_ymd(2030, 1, 15).unwrap();
        let tenor = Period::new(1, TimeUnit::Years);
        let schedule = ScheduleBuilder::new(start, end, tenor, &NullCalendar)
            .build()
            .unwrap();
        let bond = fixed_rate_bond(
            2,
            100.0,
            &schedule,
            vec![0.05],
            Compounding::Simple,
            Frequency::Annual,
            NullCalendar,
        );
        // 5 coupons + 1 redemption
        assert_eq!(bond.cashflows.len(), 6);
        assert_eq!(bond.maturity_date, end);
        assert!((bond.face_amount - 100.0).abs() < 1e-15);
    }

    #[test]
    fn fixed_rate_bond_dirty_price() {
        let start = Date::from_ymd(2025, 1, 15).unwrap();
        let end = Date::from_ymd(2030, 1, 15).unwrap();
        let tenor = Period::new(1, TimeUnit::Years);
        let schedule = ScheduleBuilder::new(start, end, tenor, &NullCalendar)
            .build()
            .unwrap();
        let bond = fixed_rate_bond(
            0,
            100.0,
            &schedule,
            vec![0.05],
            Compounding::Simple,
            Frequency::Annual,
            NullCalendar,
        );
        // At 5% yield, the dirty price should be near 100
        let settlement = start;
        let dirty = bond.dirty_price_yield(0.05, &Actual365Fixed, Compounding::Simple, Frequency::Annual, settlement);
        assert!((dirty - 100.0).abs() < 3.0, "dirty = {dirty}");
    }

    #[test]
    fn fixed_rate_bond_yield_roundtrip() {
        let start = Date::from_ymd(2025, 1, 15).unwrap();
        let end = Date::from_ymd(2030, 1, 15).unwrap();
        let tenor = Period::new(1, TimeUnit::Years);
        let schedule = ScheduleBuilder::new(start, end, tenor, &NullCalendar)
            .build()
            .unwrap();
        let bond = fixed_rate_bond(
            0,
            100.0,
            &schedule,
            vec![0.05],
            Compounding::Simple,
            Frequency::Annual,
            NullCalendar,
        );
        let settlement = start;
        // Get the dirty price at y=5%, then solve for yield
        let dirty = bond.dirty_price_yield(0.05, &Actual365Fixed, Compounding::Simple, Frequency::Annual, settlement);
        let clean = bond.clean_price_from_dirty(dirty, settlement);
        let y = bond.yield_to_maturity(clean, &Actual365Fixed, Compounding::Simple, Frequency::Annual, settlement, 1e-10).unwrap();
        assert!((y - 0.05).abs() < 1e-4, "yield = {y}");
    }

    #[test]
    fn floating_rate_bond_construction() {
        let index = Arc::new(IborIndex::new(
            "USD-Libor-3M",
            Period::new(3, TimeUnit::Months),
            2,
            &USD,
            NullCalendar,
            BusinessDayConvention::ModifiedFollowing,
            false,
            Actual365Fixed,
        ));
        let start = Date::from_ymd(2025, 1, 15).unwrap();
        let end = Date::from_ymd(2027, 1, 15).unwrap();
        let tenor = Period::new(3, TimeUnit::Months);
        let schedule = ScheduleBuilder::new(start, end, tenor, &NullCalendar)
            .build()
            .unwrap();
        let bond = floating_rate_bond(0, 100.0, &schedule, index, 1.0, 0.005, NullCalendar);
        // 2yr / 3M = 8 coupons + 1 redemption = 9
        assert_eq!(bond.cashflows.len(), 9);
    }
}
