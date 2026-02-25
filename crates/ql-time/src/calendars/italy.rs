//! Italy (Settlement) calendar.
//!
//! Translates `ql/time/calendars/italy.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// Italy (Settlement) calendar.
///
/// Weekends and the following holidays are observed:
/// * New Year's Day (Jan 1)
/// * Epiphany (Jan 6)
/// * Easter Monday (em)
/// * Liberation Day (Apr 25)
/// * Labour Day (May 1)
/// * Republic Day (Jun 2)
/// * Assumption of Mary (Aug 15)
/// * All Saints' Day (Nov 1)
/// * Immaculate Conception (Dec 8)
/// * Christmas Day (Dec 25)
/// * St. Stephen's Day (Dec 26)
#[derive(Debug, Clone, Copy, Default)]
pub struct Italy;

impl Calendar for Italy {
    fn name(&self) -> &str {
        "Italy (Settlement)"
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
        // New Year's Day / Epiphany
        ((d == 1 || d == 6) && m == 1)
            // Easter Monday
            || (dd == em)
            // Liberation Day
            || (d == 25 && m == 4)
            // Labour Day
            || (d == 1 && m == 5)
            // Republic Day
            || (d == 2 && m == 6)
            // Assumption of Mary
            || (d == 15 && m == 8)
            // All Saints' Day
            || (d == 1 && m == 11)
            // Immaculate Conception
            || (d == 8 && m == 12)
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
    fn epiphany() {
        let cal = Italy;
        // Jan 6, 2023 is a Friday
        assert!(!cal.is_business_day(date(2023, 1, 6)));
    }

    #[test]
    fn easter_monday_2023() {
        let cal = Italy;
        // Easter Monday 2023: April 10
        assert!(!cal.is_business_day(date(2023, 4, 10)));
    }

    #[test]
    fn liberation_day() {
        let cal = Italy;
        // Apr 25, 2023 is a Tuesday
        assert!(!cal.is_business_day(date(2023, 4, 25)));
    }

    #[test]
    fn immaculate_conception() {
        let cal = Italy;
        // Dec 8, 2023 is a Friday
        assert!(!cal.is_business_day(date(2023, 12, 8)));
    }

    #[test]
    fn normal_business_day() {
        let cal = Italy;
        // 2023-06-15 is a Thursday
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
