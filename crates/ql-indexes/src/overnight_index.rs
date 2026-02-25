//! `OvernightIndex` — overnight rate index (translates `ql/indexes/iborindex.hpp`
//! `OvernightIndex` subclass).

use crate::index::{FixingStore, Index};
use crate::interest_rate_index::{advance_fixing_days, InterestRateIndex, InterestRateIndexData};
use ql_core::{errors::Result, Real};
use ql_currencies::Currency;
use ql_time::{BusinessDayConvention, Calendar, Date, DayCounter, Frequency, Period, TimeUnit};

/// An overnight rate index (e.g. SOFR, ESTR, SONIA).
///
/// Overnight indexes have a tenor of 1 day. The fixing date **is** the value
/// date (fixing_days = 0 for most, 1 for some like ESTR).
///
/// Corresponds to `QuantLib::OvernightIndex`.
#[derive(Debug)]
pub struct OvernightIndex {
    pub(crate) data: InterestRateIndexData,
}

impl OvernightIndex {
    /// Create a new overnight index.
    pub fn new(
        name: impl Into<String>,
        fixing_days: u32,
        currency: &'static Currency,
        calendar: impl Calendar + 'static,
        day_counter: impl DayCounter + 'static,
    ) -> Self {
        let n = name.into();
        Self {
            data: InterestRateIndexData {
                name: n,
                tenor: Period::new(1, TimeUnit::Days),
                fixing_days,
                currency,
                calendar: Box::new(calendar),
                day_counter: Box::new(day_counter),
                frequency: Frequency::Daily,
                convention: BusinessDayConvention::Following,
                eom: false,
                fixings: FixingStore::new(),
            },
        }
    }
}

impl Index for OvernightIndex {
    fn name(&self) -> &str {
        &self.data.name
    }

    fn fixing_calendar(&self) -> &dyn Calendar {
        &*self.data.calendar
    }

    fn fixing(&self, date: Date, force_forecast: bool) -> Result<Real> {
        if !force_forecast {
            if let Some(v) = self.data.fixings.get(date) {
                return Ok(v);
            }
        }
        Err(ql_core::errors::Error::Runtime(format!(
            "{}: missing fixing for {} (term structures not yet implemented)",
            self.data.name, date
        )))
    }

    fn fixing_store(&self) -> &FixingStore {
        &self.data.fixings
    }
}

impl InterestRateIndex for OvernightIndex {
    fn tenor(&self) -> Period {
        self.data.tenor
    }

    fn fixing_days(&self) -> u32 {
        self.data.fixing_days
    }

    fn currency(&self) -> &'static Currency {
        self.data.currency
    }

    fn day_counter(&self) -> &dyn DayCounter {
        &*self.data.day_counter
    }

    fn frequency(&self) -> Frequency {
        self.data.frequency
    }

    fn value_date(&self, fixing_date: Date) -> Date {
        advance_fixing_days(&*self.data.calendar, fixing_date, self.data.fixing_days)
    }

    fn maturity_date(&self, value_date: Date) -> Date {
        // Overnight: maturity is the next business day
        advance_fixing_days(&*self.data.calendar, value_date, 1)
    }

    fn business_day_convention(&self) -> BusinessDayConvention {
        self.data.convention
    }

    fn end_of_month(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ql_time::{Actual360, NullCalendar};

    fn make_test_on() -> OvernightIndex {
        OvernightIndex::new(
            "TEST-ON",
            0,
            &ql_currencies::currencies::america::USD,
            NullCalendar,
            Actual360,
        )
    }

    #[test]
    fn overnight_name_and_tenor() {
        let idx = make_test_on();
        assert_eq!(idx.name(), "TEST-ON");
        assert_eq!(idx.tenor(), Period::new(1, TimeUnit::Days));
    }

    #[test]
    fn overnight_value_date_zero_fixing_days() {
        let idx = make_test_on();
        let d = Date::from_ymd(2025, 6, 10).unwrap();
        // With 0 fixing days, value date = fixing date
        assert_eq!(idx.value_date(d), d);
    }

    #[test]
    fn overnight_maturity_date() {
        let idx = make_test_on();
        let vd = Date::from_ymd(2025, 6, 10).unwrap();
        // Maturity = next business day (NullCalendar → next calendar day)
        assert_eq!(idx.maturity_date(vd), Date::from_ymd(2025, 6, 11).unwrap());
    }

    #[test]
    fn overnight_fixing_roundtrip() {
        let idx = make_test_on();
        let d = Date::from_ymd(2025, 6, 10).unwrap();
        idx.add_fixing(d, 0.053);
        assert_eq!(idx.fixing(d, false).unwrap(), 0.053);
    }
}
