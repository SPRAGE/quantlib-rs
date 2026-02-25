//! Austria calendar.
//!
//! Translates `ql/time/calendars/austria.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// Austria calendar.
///
/// Weekends and the following holidays are observed:
/// * New Year's Day (Jan 1)
/// * Epiphany (Jan 6)
/// * Easter Monday
/// * Labour Day (May 1)
/// * Ascension Thursday (em+38)
/// * Whit Monday (em+49)
/// * Corpus Christi (em+60)
/// * Assumption of Mary (Aug 15)
/// * National Day (Oct 26)
/// * All Saints' Day (Nov 1)
/// * Immaculate Conception (Dec 8)
/// * Christmas Day (Dec 25)
/// * St. Stephen's Day (Dec 26)
#[derive(Debug, Clone, Copy, Default)]
pub struct Austria;

impl Calendar for Austria {
    fn name(&self) -> &str {
        "Austria"
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
            // Labour Day
            || (d == 1 && m == 5)
            // Ascension Thursday
            || (dd == em + 38)
            // Whit Monday
            || (dd == em + 49)
            // Corpus Christi
            || (dd == em + 60)
            // Assumption of Mary
            || (d == 15 && m == 8)
            // National Day
            || (d == 26 && m == 10)
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
    fn new_years_day() {
        let cal = Austria;
        assert!(!cal.is_business_day(date(2023, 1, 1)));
    }

    #[test]
    fn epiphany() {
        let cal = Austria;
        assert!(!cal.is_business_day(date(2023, 1, 6)));
    }

    #[test]
    fn easter_monday_2023() {
        // Easter Monday 2023: April 10
        let cal = Austria;
        assert!(!cal.is_business_day(date(2023, 4, 10)));
    }

    #[test]
    fn national_day() {
        let cal = Austria;
        assert!(!cal.is_business_day(date(2023, 10, 26)));
    }

    #[test]
    fn normal_business_day() {
        let cal = Austria;
        // 2023-06-15 is a Thursday
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
