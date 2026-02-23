//! `IborIndex` — interbank offered-rate index (translates `ql/indexes/iborindex.hpp`).

use crate::index::{FixingStore, Index};
use crate::interest_rate_index::{
    advance_fixing_days, advance_period, InterestRateIndex, InterestRateIndexData,
};
use ql_core::{errors::Result, Real};
use ql_currencies::Currency;
use ql_time::{
    BusinessDayConvention, Calendar, Date, DayCounter, Frequency, Period,
};

/// An Interbank Offered Rate index (e.g. Euribor, USD LIBOR).
///
/// Corresponds to `QuantLib::IborIndex`.
#[derive(Debug)]
pub struct IborIndex {
    pub(crate) data: InterestRateIndexData,
}

impl IborIndex {
    /// Create a new IBOR index.
    pub fn new(
        name: impl Into<String>,
        tenor: Period,
        fixing_days: u32,
        currency: &'static Currency,
        calendar: impl Calendar + 'static,
        convention: BusinessDayConvention,
        end_of_month: bool,
        day_counter: impl DayCounter + 'static,
    ) -> Self {
        let n = name.into();
        Self {
            data: InterestRateIndexData {
                name: n,
                tenor,
                fixing_days,
                currency,
                calendar: Box::new(calendar),
                day_counter: Box::new(day_counter),
                frequency: Frequency::Annual, // IBOR rates quote annually
                convention,
                eom: end_of_month,
                fixings: FixingStore::new(),
            },
        }
    }
}

impl Index for IborIndex {
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

impl InterestRateIndex for IborIndex {
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
        advance_period(
            &*self.data.calendar,
            value_date,
            self.data.tenor,
            self.data.convention,
            self.data.eom,
        )
    }

    fn business_day_convention(&self) -> BusinessDayConvention {
        self.data.convention
    }

    fn end_of_month(&self) -> bool {
        self.data.eom
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ql_time::{Actual360, NullCalendar, TimeUnit};

    fn make_test_ibor() -> IborIndex {
        IborIndex::new(
            "TEST-IBOR-3M",
            Period::new(3, TimeUnit::Months),
            2,
            &ql_currencies::currencies::america::USD,
            NullCalendar,
            BusinessDayConvention::ModifiedFollowing,
            false,
            Actual360,
        )
    }

    #[test]
    fn ibor_name_and_currency() {
        let idx = make_test_ibor();
        assert_eq!(idx.name(), "TEST-IBOR-3M");
        assert_eq!(idx.currency().code, "USD");
    }

    #[test]
    fn ibor_fixing_store() {
        let idx = make_test_ibor();
        let d = Date::from_ymd(2025, 3, 17).unwrap();
        idx.add_fixing(d, 0.045);
        assert_eq!(idx.fixing(d, false).unwrap(), 0.045);
    }

    #[test]
    fn ibor_missing_fixing() {
        let idx = make_test_ibor();
        let d = Date::from_ymd(2025, 3, 17).unwrap();
        assert!(idx.fixing(d, false).is_err());
    }

    #[test]
    fn ibor_value_date() {
        let idx = make_test_ibor();
        // NullCalendar: every day is a business day
        let fix_date = Date::from_ymd(2025, 3, 17).unwrap(); // Monday
        let vd = idx.value_date(fix_date);
        // 2 business days later with NullCalendar → March 19
        assert_eq!(vd, Date::from_ymd(2025, 3, 19).unwrap());
    }

    #[test]
    fn ibor_maturity_date() {
        let idx = make_test_ibor();
        let vd = Date::from_ymd(2025, 3, 19).unwrap();
        let mat = idx.maturity_date(vd);
        // 3 months later → June 19
        assert_eq!(mat, Date::from_ymd(2025, 6, 19).unwrap());
    }
}
