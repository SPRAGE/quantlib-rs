//! Germany (Settlement) calendar.
//!
//! Translates `ql/time/calendars/germany.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// Germany (Settlement) calendar.
///
/// Weekends and the following holidays are observed:
/// * New Year's Day (Jan 1)
/// * Good Friday (em-3)
/// * Easter Monday (em)
/// * Labour Day (May 1)
/// * Ascension Thursday (em+38)
/// * Whit Monday (em+49)
/// * German Unity Day (Oct 3)
/// * Christmas Eve (Dec 24)
/// * Christmas Day (Dec 25)
/// * Boxing Day (Dec 26)
/// * New Year's Eve (Dec 31)
#[derive(Debug, Clone, Copy, Default)]
pub struct Germany;

impl Calendar for Germany {
    fn name(&self) -> &str {
        "Germany (Settlement)"
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

        if // New Year's Day
           (d == 1 && m == 1)
            // Good Friday
            || (dd == em - 3)
            // Easter Monday
            || (dd == em)
            // Labour Day
            || (d == 1 && m == 5)
            // Ascension Thursday (em + 38)
            || (dd == em + 38)
            // Whit Monday (em + 49)
            || (dd == em + 49)
            // German Unity Day
            || (d == 3 && m == 10)
            // Christmas Eve
            || (d == 24 && m == 12)
            // Christmas Day
            || (d == 25 && m == 12)
            // Boxing Day
            || (d == 26 && m == 12)
            // New Year's Eve
            || (d == 31 && m == 12)
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
    fn new_years_day() {
        let cal = Germany;
        assert!(!cal.is_business_day(date(2023, 1, 1)));
    }

    #[test]
    fn good_friday_and_easter_monday_2023() {
        // Easter Monday 2023: April 10 → Good Friday April 7
        let cal = Germany;
        assert!(!cal.is_business_day(date(2023, 4, 7)));  // Good Friday
        assert!(!cal.is_business_day(date(2023, 4, 10))); // Easter Monday
    }

    #[test]
    fn ascension_2023() {
        // Easter Monday 2023: April 10 (day 100)
        // Ascension = em + 38 => May 18
        let cal = Germany;
        assert!(!cal.is_business_day(date(2023, 5, 18)));
    }

    #[test]
    fn whit_monday_2023() {
        // Whit Monday = em + 49 => May 29
        let cal = Germany;
        assert!(!cal.is_business_day(date(2023, 5, 29)));
    }

    #[test]
    fn german_unity_day() {
        let cal = Germany;
        assert!(!cal.is_business_day(date(2023, 10, 3)));
    }

    #[test]
    fn christmas_period() {
        let cal = Germany;
        assert!(!cal.is_business_day(date(2023, 12, 24)));
        assert!(!cal.is_business_day(date(2023, 12, 25)));
        assert!(!cal.is_business_day(date(2023, 12, 26)));
        assert!(!cal.is_business_day(date(2023, 12, 31)));
    }

    #[test]
    fn normal_business_day() {
        let cal = Germany;
        // 2023-06-15 is a Thursday
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
