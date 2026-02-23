//! Switzerland calendar.
//!
//! Translates `ql/time/calendars/switzerland.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// Switzerland calendar.
///
/// Weekends and the following holidays are observed:
/// * New Year's Day (Jan 1)
/// * Berchtoldstag (Jan 2)
/// * Good Friday (em-3)
/// * Easter Monday (em)
/// * Ascension Thursday (em+38)
/// * Whit Monday (em+49)
/// * Swiss National Day (Aug 1)
/// * Christmas Day (Dec 25)
/// * St. Stephen's Day (Dec 26)
#[derive(Debug, Clone, Copy, Default)]
pub struct Switzerland;

impl Calendar for Switzerland {
    fn name(&self) -> &str {
        "Switzerland"
    }

    fn is_business_day(&self, date: Date) -> bool {
        let w = date.weekday();
        if matches!(w, Weekday::Saturday | Weekday::Sunday) {
            return false;
        }
        let y = date.year();
        let m = date.month();
        let d = date.day_of_month();
        let dd = date.day_of_year();
        let em = super::target::easter_monday_pub(y);

        if // New Year's Day / Berchtoldstag
           ((d == 1 || d == 2) && m == 1)
            // Good Friday
            || (dd == em - 3)
            // Easter Monday
            || (dd == em)
            // Ascension Thursday (em + 38)
            || (dd == em + 38)
            // Whit Monday (em + 49)
            || (dd == em + 49)
            // Swiss National Day
            || (d == 1 && m == 8)
            // Christmas Day
            || (d == 25 && m == 12)
            // St. Stephen's Day
            || (d == 26 && m == 12)
        {
            return false;
        }
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
    fn berchtoldstag() {
        let cal = Switzerland;
        // Jan 2, 2023 is a Monday
        assert!(!cal.is_business_day(date(2023, 1, 2)));
    }

    #[test]
    fn good_friday_and_easter_monday_2023() {
        let cal = Switzerland;
        assert!(!cal.is_business_day(date(2023, 4, 7)));  // Good Friday
        assert!(!cal.is_business_day(date(2023, 4, 10))); // Easter Monday
    }

    #[test]
    fn ascension_2023() {
        // Easter Monday 2023: April 10 → Ascension = em + 38 => May 18
        let cal = Switzerland;
        assert!(!cal.is_business_day(date(2023, 5, 18)));
    }

    #[test]
    fn whit_monday_2023() {
        // Whit Monday = em + 49 => May 29
        let cal = Switzerland;
        assert!(!cal.is_business_day(date(2023, 5, 29)));
    }

    #[test]
    fn swiss_national_day() {
        let cal = Switzerland;
        // Aug 1, 2023 is a Tuesday
        assert!(!cal.is_business_day(date(2023, 8, 1)));
    }

    #[test]
    fn normal_business_day() {
        let cal = Switzerland;
        // 2023-06-15 is a Thursday
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
