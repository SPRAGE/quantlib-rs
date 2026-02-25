//! Rate helpers for yield-curve bootstrapping
//! (translates `ql/termstructures/yield/ratehelpers.hpp`).
//!
//! A *rate helper* encapsulates a market-observable quote (e.g. a deposit rate,
//! FRA rate, or par swap rate) together with the conventions needed to derive a
//! discount-factor constraint at a *pillar date*.  The bootstrapper uses these
//! helpers to iteratively build a [`PiecewiseYieldCurve`](super::PiecewiseYieldCurve).

use ql_core::{errors::Result, Rate, Real, Time};
use ql_time::{
    BusinessDayConvention, Calendar, Date, DateGeneration, DayCounter, Frequency, Period, Schedule,
    ScheduleBuilder, TimeUnit,
};

// ── BootstrapCurve (temporary curve view used during bootstrap) ──────────────

/// A lightweight, mutable curve used during the bootstrap iteration.
///
/// During bootstrapping, the solver needs to probe discount factors from a
/// partially-constructed curve.  `BootstrapCurve` stores the pillar dates,
/// times, and zero rates accumulated so far, together with an interpolation
/// object that is rebuilt each time a rate changes.
#[derive(Debug)]
pub struct BootstrapCurve<'a> {
    /// Reference date.
    pub reference_date: Date,
    /// Day counter for time conversion.
    pub day_counter: &'a dyn DayCounter,
    /// Pillar times (first entry = 0 for the reference date).
    pub times: &'a [Real],
    /// Continuously-compounded zero rates at each pillar.
    pub rates: &'a [Rate],
    /// Interpolation object (rebuilt as pillars are added).
    pub interp: &'a dyn ql_math::Interpolation1D,
}

impl<'a> BootstrapCurve<'a> {
    /// Time from the reference date to `date`.
    pub fn time_from_reference(&self, date: Date) -> Time {
        self.day_counter.year_fraction(self.reference_date, date)
    }

    /// Continuously-compounded zero rate for a given time.
    pub fn zero_rate(&self, t: Time) -> Rate {
        if t <= 0.0 {
            return if self.rates.is_empty() {
                0.0
            } else {
                self.rates[0]
            };
        }
        self.interp.operator(t)
    }

    /// Discount factor for a given time.
    pub fn discount(&self, t: Time) -> Real {
        if t <= 0.0 {
            return 1.0;
        }
        let z = self.zero_rate(t);
        (-z * t).exp()
    }

    /// Discount factor for a given date.
    pub fn discount_date(&self, date: Date) -> Real {
        self.discount(self.time_from_reference(date))
    }
}

// ── RateHelper trait ──────────────────────────────────────────────────────────

/// A single market quote that constrains the yield curve at a pillar date.
///
/// Corresponds to `QuantLib::RateHelper`.
pub trait RateHelper: std::fmt::Debug + Send + Sync {
    /// The pillar date — the date up to which this helper constrains the curve.
    fn pillar_date(&self) -> Date;

    /// The market-quoted rate (e.g. deposit rate, swap rate).
    fn quote(&self) -> Real;

    /// The model-implied quote given the (partially bootstrapped) curve.
    ///
    /// The bootstrapper adjusts the zero rate at this helper's pillar until
    /// `implied_quote(curve) == quote()`.
    fn implied_quote(&self, curve: &BootstrapCurve<'_>) -> Real;
}

// ── DepositRateHelper ─────────────────────────────────────────────────────────

/// A deposit (money-market) rate helper.
///
/// Constrains the curve at the deposit's maturity date.  The implied quote is
/// the simple (Act/N) forward rate over the `[settlement, maturity]` period.
///
/// Corresponds to `QuantLib::DepositRateHelper`.
#[derive(Debug)]
pub struct DepositRateHelper {
    rate: Rate,
    settlement_date: Date,
    maturity_date: Date,
    day_counter: Box<dyn DayCounter>,
}

impl DepositRateHelper {
    /// Create a deposit rate helper from explicit settlement and maturity dates.
    pub fn new(
        rate: Rate,
        settlement_date: Date,
        maturity_date: Date,
        day_counter: impl DayCounter + 'static,
    ) -> Self {
        Self {
            rate,
            settlement_date,
            maturity_date,
            day_counter: Box::new(day_counter),
        }
    }

    /// Create a deposit rate helper from a tenor and conventions.
    ///
    /// `fixing_days` business days after the reference date gives the
    /// settlement date; advancing by `tenor` gives the maturity.
    pub fn from_tenor(
        rate: Rate,
        tenor: Period,
        fixing_days: u32,
        calendar: &dyn Calendar,
        convention: BusinessDayConvention,
        end_of_month: bool,
        day_counter: impl DayCounter + 'static,
        reference_date: Date,
    ) -> Self {
        let settlement = calendar.advance_business_days(reference_date, fixing_days as i32);
        let maturity = if end_of_month && calendar.is_end_of_month(settlement) {
            let raw = settlement
                .advance(tenor.length, tenor.unit)
                .expect("valid date");
            raw.end_of_month()
        } else {
            let raw = settlement
                .advance(tenor.length, tenor.unit)
                .expect("valid date");
            calendar.adjust(raw, convention)
        };
        Self {
            rate,
            settlement_date: settlement,
            maturity_date: maturity,
            day_counter: Box::new(day_counter),
        }
    }

    /// The settlement date of the deposit.
    pub fn settlement_date(&self) -> Date {
        self.settlement_date
    }

    /// The maturity date of the deposit.
    pub fn maturity_date(&self) -> Date {
        self.maturity_date
    }
}

impl RateHelper for DepositRateHelper {
    fn pillar_date(&self) -> Date {
        self.maturity_date
    }

    fn quote(&self) -> Real {
        self.rate
    }

    fn implied_quote(&self, curve: &BootstrapCurve<'_>) -> Real {
        // Simple-compounding forward rate over [settlement, maturity]:
        //   R = (P(t_settle) / P(t_maturity) - 1) / tau
        let tau = self
            .day_counter
            .year_fraction(self.settlement_date, self.maturity_date);
        if tau <= 0.0 {
            return 0.0;
        }
        let df_settle = curve.discount_date(self.settlement_date);
        let df_maturity = curve.discount_date(self.maturity_date);
        if df_maturity <= 0.0 {
            return 0.0;
        }
        (df_settle / df_maturity - 1.0) / tau
    }
}

// ── FraRateHelper ─────────────────────────────────────────────────────────────

/// A forward-rate-agreement (FRA) rate helper.
///
/// Constrains the curve at the FRA maturity date.  The implied quote is the
/// simple forward rate between the FRA's value date and maturity.
///
/// Corresponds to `QuantLib::FraRateHelper`.
#[derive(Debug)]
pub struct FraRateHelper {
    rate: Rate,
    value_date: Date,
    maturity_date: Date,
    day_counter: Box<dyn DayCounter>,
}

impl FraRateHelper {
    /// Create a FRA rate helper from explicit value and maturity dates.
    pub fn new(
        rate: Rate,
        value_date: Date,
        maturity_date: Date,
        day_counter: impl DayCounter + 'static,
    ) -> Self {
        Self {
            rate,
            value_date,
            maturity_date,
            day_counter: Box::new(day_counter),
        }
    }

    /// Create a FRA rate helper from month offsets.
    ///
    /// `months_to_start` and `months_to_end` are counted from the settlement
    /// date (= reference_date + fixing_days).
    pub fn from_months(
        rate: Rate,
        months_to_start: u32,
        months_to_end: u32,
        fixing_days: u32,
        calendar: &dyn Calendar,
        convention: BusinessDayConvention,
        day_counter: impl DayCounter + 'static,
        reference_date: Date,
    ) -> Self {
        let settlement = calendar.advance_business_days(reference_date, fixing_days as i32);
        let value_date = calendar.adjust(
            settlement
                .advance(months_to_start as i32, TimeUnit::Months)
                .expect("valid date"),
            convention,
        );
        let maturity_date = calendar.adjust(
            settlement
                .advance(months_to_end as i32, TimeUnit::Months)
                .expect("valid date"),
            convention,
        );
        Self {
            rate,
            value_date,
            maturity_date,
            day_counter: Box::new(day_counter),
        }
    }

    /// The FRA value (start) date.
    pub fn value_date(&self) -> Date {
        self.value_date
    }

    /// The FRA maturity (end) date.
    pub fn maturity_date(&self) -> Date {
        self.maturity_date
    }
}

impl RateHelper for FraRateHelper {
    fn pillar_date(&self) -> Date {
        self.maturity_date
    }

    fn quote(&self) -> Real {
        self.rate
    }

    fn implied_quote(&self, curve: &BootstrapCurve<'_>) -> Real {
        let tau = self
            .day_counter
            .year_fraction(self.value_date, self.maturity_date);
        if tau <= 0.0 {
            return 0.0;
        }
        let df_start = curve.discount_date(self.value_date);
        let df_end = curve.discount_date(self.maturity_date);
        if df_end <= 0.0 {
            return 0.0;
        }
        (df_start / df_end - 1.0) / tau
    }
}

// ── SwapRateHelper ────────────────────────────────────────────────────────────

/// A par-swap rate helper.
///
/// Constrains the curve at the swap's maturity date.  The implied quote is the
/// par swap rate computed from the curve's discount factors on the fixed-leg
/// payment schedule.
///
/// Corresponds to `QuantLib::SwapRateHelper`.
#[derive(Debug)]
pub struct SwapRateHelper {
    rate: Rate,
    /// Fixed-leg payment dates including start & end.
    fixed_schedule: Schedule,
    /// Fixed-leg day counter (for accrual fractions).
    fixed_day_counter: Box<dyn DayCounter>,
}

impl SwapRateHelper {
    /// Create a swap-rate helper from an already-built fixed-leg schedule.
    pub fn new(
        rate: Rate,
        fixed_schedule: Schedule,
        fixed_day_counter: impl DayCounter + 'static,
    ) -> Self {
        Self {
            rate,
            fixed_schedule,
            fixed_day_counter: Box::new(fixed_day_counter),
        }
    }

    /// Create a swap-rate helper from conventions.
    ///
    /// Builds a forward schedule from `settlement_date` to
    /// `settlement_date + swap_tenor` with the given fixed-leg frequency.
    pub fn from_conventions(
        rate: Rate,
        swap_tenor: Period,
        calendar: &dyn Calendar,
        fixed_frequency: Frequency,
        fixed_convention: BusinessDayConvention,
        fixed_day_counter: impl DayCounter + 'static,
        reference_date: Date,
        fixing_days: u32,
    ) -> Result<Self> {
        let settlement = calendar.advance_business_days(reference_date, fixing_days as i32);
        let maturity = calendar.adjust(
            settlement
                .advance(swap_tenor.length, swap_tenor.unit)
                .expect("valid date"),
            fixed_convention,
        );

        let fixed_tenor = frequency_to_period(fixed_frequency);
        let fixed_schedule = ScheduleBuilder::new(settlement, maturity, fixed_tenor, calendar)
            .with_convention(fixed_convention)
            .with_termination_convention(fixed_convention)
            .with_rule(DateGeneration::Forward)
            .build()?;

        Ok(Self {
            rate,
            fixed_schedule,
            fixed_day_counter: Box::new(fixed_day_counter),
        })
    }

    /// The fixed-leg payment schedule.
    pub fn fixed_schedule(&self) -> &Schedule {
        &self.fixed_schedule
    }
}

impl RateHelper for SwapRateHelper {
    fn pillar_date(&self) -> Date {
        self.fixed_schedule
            .end_date()
            .expect("schedule has an end date")
    }

    fn quote(&self) -> Real {
        self.rate
    }

    fn implied_quote(&self, curve: &BootstrapCurve<'_>) -> Real {
        // Par swap rate = (P(t_start) - P(t_end)) / Annuity
        // where Annuity = sum_i delta_i * P(t_i)
        let dates = self.fixed_schedule.dates();
        if dates.len() < 2 {
            return 0.0;
        }

        let start = dates[0];
        let end = dates[dates.len() - 1];
        let df_start = curve.discount_date(start);
        let df_end = curve.discount_date(end);

        let mut annuity = 0.0;
        for i in 1..dates.len() {
            let delta = self.fixed_day_counter.year_fraction(dates[i - 1], dates[i]);
            let df = curve.discount_date(dates[i]);
            annuity += delta * df;
        }

        if annuity.abs() < 1e-16 {
            return 0.0;
        }
        (df_start - df_end) / annuity
    }
}

// ── FuturesRateHelper ─────────────────────────────────────────────────────────

/// An interest-rate-futures rate helper (e.g. Eurodollar, Euribor futures).
///
/// Constrains the curve at the futures' maturity date.  The market price is
/// quoted as `100 - rate` (in percentage terms); the implied quote is the
/// simple forward rate between the futures' value date and maturity, adjusted
/// for convexity.
///
/// Corresponds to `QuantLib::FuturesRateHelper`.
#[derive(Debug)]
pub struct FuturesRateHelper {
    /// Implied forward rate = (100 - price) / 100 - convexity_adjustment.
    rate: Rate,
    value_date: Date,
    maturity_date: Date,
    day_counter: Box<dyn DayCounter>,
    convexity_adjustment: Real,
}

impl FuturesRateHelper {
    /// Create from a price (e.g. 96.50 → rate = 0.035).
    ///
    /// The convexity adjustment is *subtracted* from the futures-implied rate
    /// to give the forward rate.
    pub fn from_price(
        price: Real,
        value_date: Date,
        maturity_date: Date,
        day_counter: impl DayCounter + 'static,
        convexity_adjustment: Real,
    ) -> Self {
        Self {
            rate: (100.0 - price) / 100.0 - convexity_adjustment,
            value_date,
            maturity_date,
            day_counter: Box::new(day_counter),
            convexity_adjustment,
        }
    }

    /// Create from an explicit forward rate.
    pub fn from_rate(
        rate: Rate,
        value_date: Date,
        maturity_date: Date,
        day_counter: impl DayCounter + 'static,
    ) -> Self {
        Self {
            rate,
            value_date,
            maturity_date,
            day_counter: Box::new(day_counter),
            convexity_adjustment: 0.0,
        }
    }

    /// The convexity adjustment.
    pub fn convexity_adjustment(&self) -> Real {
        self.convexity_adjustment
    }
}

impl RateHelper for FuturesRateHelper {
    fn pillar_date(&self) -> Date {
        self.maturity_date
    }

    fn quote(&self) -> Real {
        self.rate
    }

    fn implied_quote(&self, curve: &BootstrapCurve<'_>) -> Real {
        let tau = self
            .day_counter
            .year_fraction(self.value_date, self.maturity_date);
        if tau <= 0.0 {
            return 0.0;
        }
        let df_start = curve.discount_date(self.value_date);
        let df_end = curve.discount_date(self.maturity_date);
        if df_end <= 0.0 {
            return 0.0;
        }
        (df_start / df_end - 1.0) / tau
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Convert a [`Frequency`] to a [`Period`].
fn frequency_to_period(freq: Frequency) -> Period {
    match freq {
        Frequency::Annual => Period::new(1, TimeUnit::Years),
        Frequency::Semiannual => Period::new(6, TimeUnit::Months),
        Frequency::Quarterly => Period::new(3, TimeUnit::Months),
        Frequency::Monthly => Period::new(1, TimeUnit::Months),
        Frequency::Biweekly => Period::new(2, TimeUnit::Weeks),
        Frequency::Weekly => Period::new(1, TimeUnit::Weeks),
        Frequency::Daily => Period::new(1, TimeUnit::Days),
        _ => Period::new(1, TimeUnit::Years),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ql_time::Actual360;

    /// Build a trivial `BootstrapCurve` from a flat zero rate for testing.
    fn flat_curve(
        _reference_date: Date,
        max_t: Real,
        flat_rate: Rate,
    ) -> (Vec<Real>, Vec<Rate>, ql_math::LinearInterpolation) {
        let times = vec![0.0, max_t];
        let rates = vec![flat_rate, flat_rate];
        let interp = ql_math::LinearInterpolation::new(&times, &rates).unwrap();
        (times, rates, interp)
    }

    #[test]
    fn deposit_helper_implied_equals_quote_on_flat_curve() {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let settle = ref_date;
        let mat = Date::from_ymd(2025, 4, 2).unwrap();

        // Flat 5% continuous curve
        let (times, rates, interp) = flat_curve(ref_date, 5.0, 0.05);
        let bc = BootstrapCurve {
            reference_date: ref_date,
            day_counter: &Actual360,
            times: &times,
            rates: &rates,
            interp: &interp,
        };

        let helper = DepositRateHelper::new(0.0, settle, mat, Actual360);
        let implied = helper.implied_quote(&bc);

        // The simple forward rate from a flat continuous curve:
        // R = (exp(r*tau) - 1) / tau   where tau = Actual360 fraction
        let tau = Actual360.year_fraction(settle, mat);
        let expected = ((0.05 * tau).exp() - 1.0) / tau;
        assert!(
            (implied - expected).abs() < 1e-10,
            "implied={implied} expected={expected}"
        );
    }

    #[test]
    fn swap_helper_implied_par_rate_on_flat_curve() {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let mat = Date::from_ymd(2030, 1, 2).unwrap();

        let schedule = Schedule::from_dates(vec![
            ref_date,
            Date::from_ymd(2026, 1, 2).unwrap(),
            Date::from_ymd(2027, 1, 4).unwrap(),
            Date::from_ymd(2028, 1, 3).unwrap(),
            Date::from_ymd(2029, 1, 2).unwrap(),
            mat,
        ]);

        let (times, rates, interp) = flat_curve(ref_date, 10.0, 0.04);
        let bc = BootstrapCurve {
            reference_date: ref_date,
            day_counter: &Actual360,
            times: &times,
            rates: &rates,
            interp: &interp,
        };

        let helper = SwapRateHelper::new(0.04, schedule, Actual360);
        let implied = helper.implied_quote(&bc);

        // On a flat curve the par swap rate should be close to the zero rate
        // (not exact because of simple vs continuous compounding).
        assert!(
            (implied - 0.04).abs() < 0.005,
            "implied par rate={implied}, expected near 0.04"
        );
    }

    #[test]
    fn fra_helper_implied_equals_quote_on_flat_curve() {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let start = Date::from_ymd(2025, 4, 2).unwrap();
        let end = Date::from_ymd(2025, 7, 2).unwrap();

        let (times, rates, interp) = flat_curve(ref_date, 5.0, 0.03);
        let bc = BootstrapCurve {
            reference_date: ref_date,
            day_counter: &Actual360,
            times: &times,
            rates: &rates,
            interp: &interp,
        };

        let helper = FraRateHelper::new(0.0, start, end, Actual360);
        let implied = helper.implied_quote(&bc);

        let tau = Actual360.year_fraction(start, end);
        let t_start = Actual360.year_fraction(ref_date, start);
        let t_end = Actual360.year_fraction(ref_date, end);
        let df_start = (-0.03 * t_start).exp();
        let df_end = (-0.03 * t_end).exp();
        let expected = (df_start / df_end - 1.0) / tau;
        assert!(
            (implied - expected).abs() < 1e-10,
            "implied={implied} expected={expected}"
        );
    }

    #[test]
    fn futures_helper_from_price() {
        let value = Date::from_ymd(2025, 3, 19).unwrap();
        let mat = Date::from_ymd(2025, 6, 18).unwrap();
        let helper = FuturesRateHelper::from_price(96.0, value, mat, Actual360, 0.001);
        // rate = (100 - 96)/100 - 0.001 = 0.04 - 0.001 = 0.039
        assert!((helper.quote() - 0.039).abs() < 1e-12);
    }
}
