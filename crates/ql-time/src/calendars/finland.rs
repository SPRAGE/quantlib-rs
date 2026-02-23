//! Finland calendar.
//!
//! Translates `ql/time/calendars/finland.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// Finland calendar.
///
/// Weekends and the following holidays are observed:
/// * New Year's Day (Jan 1)
/// * Epiphany (Jan 6)
/// * Good Friday (em-3)
/// * Easter Monday (em)
/// * May Day (May 1)
/// * Ascension Thursday (em+38)
/// * Midsummer Eve (Friday between Jun 19–25)
/// * Independence Day (Dec 6)
/// * Christmas Eve (Dec 24)
/// * Christmas Day (Dec 25)
/// * Boxing Day (Dec 26)
#[derive(Debug, Clone, Copy, Default)]
pub struct Finland;

impl Calendar for Finland {
    fn name(&self) -> &str {
        "Finland"
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

        if // New Year's Day / Epiphany
           ((d == 1 || d == 6) && m == 1)
            // Good Friday
            || (dd == em - 3)
            // Easter Monday
            || (dd == em)
            // May Day
            || (d == 1 && m == 5)
            // Ascension Thursday
            || (dd == em + 38)
            // Midsummer Eve (Friday between Jun 19–25)
            || (matches!(w, Weekday::Friday) && m == 6 && (19..=25).contains(&d))
            // Independence Day
            || (d == 6 && m == 12)
            // Christmas Eve
            || (d == 24 && m == 12)
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
        let cal = Finland;
        assert!(!cal.is_business_day(date(2023, 1, 1)));
    }

    #[test]
    fn epiphany() {
        let cal = Finland;
        assert!(!cal.is_business_day(date(2023, 1, 6)));
    }

    #[test]
    fn midsummer_eve_2023() {
        // 2023: Jun 23 is a Friday
        let cal = Finland;
        assert!(!cal.is_business_day(date(2023, 6, 23)));
    }

    #[test]
    fn independence_day() {
        let cal = Finland;
        assert!(!cal.is_business_day(date(2023, 12, 6)));
    }

    #[test]
    fn normal_business_day() {
        let cal = Finland;
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
