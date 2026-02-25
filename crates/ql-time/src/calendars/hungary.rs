//! Hungary calendar.
//!
//! Translates `ql/time/calendars/hungary.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// Hungary calendar.
///
/// Weekends and the following holidays are observed:
/// * New Year's Day (Jan 1)
/// * 1848 Revolution Day (Mar 15)
/// * Good Friday (em-3, since 2017)
/// * Easter Monday (em)
/// * Labour Day (May 1)
/// * Whit Monday (em+49)
/// * St. Stephen's Day (Aug 20)
/// * Republic Day (Oct 23)
/// * All Saints' Day (Nov 1)
/// * Christmas Day (Dec 25)
/// * Boxing Day (Dec 26)
#[derive(Debug, Clone, Copy, Default)]
pub struct Hungary;

impl Calendar for Hungary {
    fn name(&self) -> &str {
        "Hungary"
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
            // 1848 Revolution Day
            || (d == 15 && m == 3)
            // Good Friday (since 2017)
            || (dd == em - 3 && y >= 2017)
            // Easter Monday
            || (dd == em)
            // Labour Day
            || (d == 1 && m == 5)
            // Whit Monday
            || (dd == em + 49)
            // St. Stephen's Day
            || (d == 20 && m == 8)
            // Republic Day
            || (d == 23 && m == 10)
            // All Saints' Day
            || (d == 1 && m == 11)
            // Christmas Day
            || (d == 25 && m == 12)
            // Boxing Day
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
    fn new_years_day() {
        let cal = Hungary;
        assert!(!cal.is_business_day(date(2023, 1, 1)));
    }

    #[test]
    fn revolution_day() {
        let cal = Hungary;
        assert!(!cal.is_business_day(date(2023, 3, 15)));
    }

    #[test]
    fn good_friday_since_2017() {
        let cal = Hungary;
        // Easter Monday 2023: April 10, Good Friday = April 7
        assert!(!cal.is_business_day(date(2023, 4, 7)));
        // Good Friday not observed before 2017
        // Easter Monday 2016: March 28, Good Friday = March 25
        assert!(cal.is_business_day(date(2016, 3, 25)));
    }

    #[test]
    fn st_stephens_day() {
        let cal = Hungary;
        assert!(!cal.is_business_day(date(2023, 8, 20)));
    }

    #[test]
    fn normal_business_day() {
        let cal = Hungary;
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
