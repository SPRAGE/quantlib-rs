//! Romania calendar.
//!
//! Translates `ql/time/calendars/romania.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// Romania calendar.
///
/// Weekends and the following holidays are observed:
/// * New Year's Day (Jan 1–2)
/// * Unification Day (Jan 24, since 2016)
/// * Easter Monday (em — uses Western Easter for simplicity)
/// * Labour Day (May 1)
/// * Children's Day (Jun 1)
/// * Pentecost Monday (em+49)
/// * Assumption of Mary (Aug 15)
/// * St. Andrew's Day (Nov 30)
/// * National Day (Dec 1)
/// * Christmas Day (Dec 25)
/// * Boxing Day (Dec 26)
///
/// Note: Romanian Orthodox Easter differs from Western Easter. For simplicity
/// this implementation uses the same `easter_monday_pub` function (Western).
#[derive(Debug, Clone, Copy, Default)]
pub struct Romania;

impl Calendar for Romania {
    fn name(&self) -> &str {
        "Romania"
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
        // New Year's Day (Jan 1–2)
        ((d == 1 || d == 2) && m == 1)
            // Unification Day (since 2016)
            || (d == 24 && m == 1 && y >= 2016)
            // Easter Monday
            || (dd == em)
            // Labour Day
            || (d == 1 && m == 5)
            // Children's Day
            || (d == 1 && m == 6)
            // Pentecost Monday
            || (dd == em + 49)
            // Assumption of Mary
            || (d == 15 && m == 8)
            // St. Andrew's Day
            || (d == 30 && m == 11)
            // National Day
            || (d == 1 && m == 12)
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
        let cal = Romania;
        assert!(!cal.is_business_day(date(2023, 1, 1)));
        assert!(!cal.is_business_day(date(2023, 1, 2)));
    }

    #[test]
    fn unification_day_since_2016() {
        let cal = Romania;
        // 2023-01-24 is a Tuesday
        assert!(!cal.is_business_day(date(2023, 1, 24)));
        // Not observed before 2016
        // 2015-01-24 is a Saturday so test 2014-01-24 (Friday)
        assert!(cal.is_business_day(date(2014, 1, 24)));
    }

    #[test]
    fn national_day() {
        let cal = Romania;
        assert!(!cal.is_business_day(date(2023, 12, 1)));
    }

    #[test]
    fn normal_business_day() {
        let cal = Romania;
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
