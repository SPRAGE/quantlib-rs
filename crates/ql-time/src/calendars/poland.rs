//! Poland calendar.
//!
//! Translates `ql/time/calendars/poland.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// Poland calendar.
///
/// Weekends and the following holidays are observed:
/// * New Year's Day (Jan 1)
/// * Epiphany (Jan 6, since 2011)
/// * Easter Monday (em)
/// * Labour Day (May 1)
/// * Constitution Day (May 3)
/// * Corpus Christi (em+60)
/// * Assumption of Mary (Aug 15)
/// * All Saints' Day (Nov 1)
/// * Independence Day (Nov 11)
/// * Christmas Day (Dec 25)
/// * St. Stephen's Day (Dec 26)
#[derive(Debug, Clone, Copy, Default)]
pub struct Poland;

impl Calendar for Poland {
    fn name(&self) -> &str {
        "Poland"
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
            // Epiphany (since 2011)
            || (d == 6 && m == 1 && y >= 2011)
            // Easter Monday
            || (dd == em)
            // Labour Day
            || (d == 1 && m == 5)
            // Constitution Day
            || (d == 3 && m == 5)
            // Corpus Christi
            || (dd == em + 60)
            // Assumption of Mary
            || (d == 15 && m == 8)
            // All Saints' Day
            || (d == 1 && m == 11)
            // Independence Day
            || (d == 11 && m == 11)
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
        let cal = Poland;
        assert!(!cal.is_business_day(date(2023, 1, 1)));
    }

    #[test]
    fn epiphany_since_2011() {
        let cal = Poland;
        assert!(!cal.is_business_day(date(2023, 1, 6)));
        // Not observed before 2011
        // 2010-01-06 is a Wednesday
        assert!(cal.is_business_day(date(2010, 1, 6)));
    }

    #[test]
    fn constitution_day() {
        let cal = Poland;
        assert!(!cal.is_business_day(date(2023, 5, 3)));
    }

    #[test]
    fn independence_day() {
        let cal = Poland;
        // 2024-11-11 is a Monday
        assert!(!cal.is_business_day(date(2024, 11, 11)));
    }

    #[test]
    fn normal_business_day() {
        let cal = Poland;
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
