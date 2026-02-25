//! `InterestRateIndex` — base for interest-rate indexes
//! (translates `ql/indexes/interestrateindex.hpp`).

use crate::index::{FixingStore, Index};
use ql_currencies::Currency;
use ql_time::{BusinessDayConvention, Calendar, Date, DayCounter, Frequency, Period};

/// Common data shared by all interest-rate indexes.
///
/// Corresponds to `QuantLib::InterestRateIndex`.
pub trait InterestRateIndex: Index {
    /// The index tenor (e.g. 3M, 6M, ON).
    fn tenor(&self) -> Period;

    /// Settlement days (business days between trade and value date).
    fn fixing_days(&self) -> u32;

    /// The currency in which the index is denominated.
    fn currency(&self) -> &'static Currency;

    /// Day counter used for accrual.
    fn day_counter(&self) -> &dyn DayCounter;

    /// Compounding frequency (Annual for most IBOR, NoFrequency for ON).
    fn frequency(&self) -> Frequency;

    /// Value date corresponding to a given fixing date.
    fn value_date(&self, fixing_date: Date) -> Date;

    /// Maturity date corresponding to a given value date.
    fn maturity_date(&self, value_date: Date) -> Date;

    /// Business-day convention for adjusting dates.
    fn business_day_convention(&self) -> BusinessDayConvention;

    /// Whether this is an end-of-month index.
    fn end_of_month(&self) -> bool;
}

/// Advance `date` by `n` business days in the given calendar.
pub(crate) fn advance_fixing_days(cal: &dyn Calendar, date: Date, n: u32) -> Date {
    cal.advance_business_days(date, n as i32)
}

/// Advance `date` by a period using the calendar and conventions.
pub(crate) fn advance_period(
    cal: &dyn Calendar,
    date: Date,
    period: Period,
    convention: BusinessDayConvention,
    eom: bool,
) -> Date {
    let raw = date
        .advance(period.length, period.unit)
        .expect("advance by period");
    let adjusted = cal.adjust(raw, convention);
    if eom && cal.is_end_of_month(date) {
        cal.end_of_month(adjusted)
    } else {
        adjusted
    }
}

/// Common data bundle for concrete interest-rate index implementations.
#[derive(Debug)]
pub(crate) struct InterestRateIndexData {
    pub name: String,
    pub tenor: Period,
    pub fixing_days: u32,
    pub currency: &'static Currency,
    pub calendar: Box<dyn Calendar>,
    pub day_counter: Box<dyn DayCounter>,
    pub frequency: Frequency,
    pub convention: BusinessDayConvention,
    pub eom: bool,
    pub fixings: FixingStore,
}
