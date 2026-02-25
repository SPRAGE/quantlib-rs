//! Iceland calendar.
//!
//! Translates `ql/time/calendars/iceland.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// Iceland calendar.
///
/// Weekends and the following holidays are observed:
/// * New Year's Day (Jan 1)
/// * Maundy Thursday (em-4)
/// * Good Friday (em-3)
/// * Easter Monday (em)
/// * First Day of Summer (first Thursday after Apr 18)
/// * Labour Day (May 1)
/// * Ascension Thursday (em+38)
/// * Whit Monday (em+49)
/// * Icelandic National Day (Jun 17)
/// * Commerce Day (first Monday in August)
/// * Christmas Eve (Dec 24, treated as full day off)
/// * Christmas Day (Dec 25)
/// * Boxing Day (Dec 26)
/// * New Year's Eve (Dec 31, treated as full day off)
#[derive(Debug, Clone, Copy, Default)]
pub struct Iceland;

impl Calendar for Iceland {
    fn name(&self) -> &str {
        "Iceland"
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

        if
        // New Year's Day
        (d == 1 && m == 1)
            // Maundy Thursday
            || (dd == em - 4)
            // Good Friday
            || (dd == em - 3)
            // Easter Monday
            || (dd == em)
            // First Day of Summer (first Thursday after Apr 18)
            || (matches!(w, Weekday::Thursday) && m == 4 && (19..=25).contains(&d))
            // Labour Day
            || (d == 1 && m == 5)
            // Ascension Thursday
            || (dd == em + 38)
            // Whit Monday
            || (dd == em + 49)
            // Icelandic National Day
            || (d == 17 && m == 6)
            // Commerce Day (first Monday in August)
            || (matches!(w, Weekday::Monday) && m == 8 && d <= 7)
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
        let cal = Iceland;
        assert!(!cal.is_business_day(date(2023, 1, 1)));
    }

    #[test]
    fn first_day_of_summer_2023() {
        // First Thursday after Apr 18, 2023
        // Apr 18, 2023 is Tuesday → first Thursday = Apr 20
        let cal = Iceland;
        assert!(!cal.is_business_day(date(2023, 4, 20)));
    }

    #[test]
    fn commerce_day_2023() {
        // First Monday in August 2023 = Aug 7
        let cal = Iceland;
        assert!(!cal.is_business_day(date(2023, 8, 7)));
    }

    #[test]
    fn christmas_eve() {
        let cal = Iceland;
        assert!(!cal.is_business_day(date(2024, 12, 24)));
    }

    #[test]
    fn normal_business_day() {
        let cal = Iceland;
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
