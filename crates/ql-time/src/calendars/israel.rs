//! Israel (TASE) calendar.
//!
//! Translates `ql/time/calendars/israel.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// Israel (Tel Aviv Stock Exchange) calendar.
///
/// Weekend days are Friday and Saturday.
///
/// Hebrew-calendar-based holidays vary yearly and require a Hebrew calendar
/// table to determine exact dates. This implementation currently handles
/// only the weekend convention (Friday + Saturday).
#[derive(Debug, Clone, Copy, Default)]
pub struct Israel;

impl Calendar for Israel {
    fn name(&self) -> &str {
        "Israel (TASE)"
    }

    fn is_weekend(&self, date: Date) -> bool {
        matches!(date.weekday(), Weekday::Friday | Weekday::Saturday)
    }

    fn is_business_day(&self, date: Date) -> bool {
        if self.is_weekend(date) {
            return false;
        }
        // Hebrew-calendar holidays are not implemented.
        // A production implementation would look up a table of Hebrew dates.
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: u16, m: u8, d: u8) -> Date {
        Date::from_ymd(y, m, d).unwrap()
    }

    #[test]
    fn friday_is_weekend() {
        let cal = Israel;
        // 2023-06-16 is a Friday
        assert!(!cal.is_business_day(date(2023, 6, 16)));
    }

    #[test]
    fn saturday_is_weekend() {
        let cal = Israel;
        // 2023-06-17 is a Saturday
        assert!(!cal.is_business_day(date(2023, 6, 17)));
    }

    #[test]
    fn sunday_is_business_day() {
        let cal = Israel;
        // 2023-06-18 is a Sunday
        assert!(cal.is_business_day(date(2023, 6, 18)));
    }

    #[test]
    fn thursday_is_business_day() {
        let cal = Israel;
        // 2023-06-15 is a Thursday
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
