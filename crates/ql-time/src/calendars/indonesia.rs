//! Indonesia calendar.
//!
//! Translates `ql/time/calendars/indonesia.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// Indonesia calendar.
///
/// Weekends and the following fixed/Easter-based holidays are observed:
/// * New Year's Day (Jan 1)
/// * Good Friday (em-3)
/// * Labour Day (May 1)
/// * Ascension Thursday (em+38)
/// * Pancasila Day (Jun 1)
/// * Independence Day (Aug 17)
/// * Christmas Day (Dec 25)
///
/// Note: Islamic (Eid al-Fitr, Eid al-Adha, etc.) and Hindu (Nyepi) holidays
/// vary yearly and are not included here.
#[derive(Debug, Clone, Copy, Default)]
pub struct Indonesia;

impl Calendar for Indonesia {
    fn name(&self) -> &str {
        "Indonesia"
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
            // Good Friday
            || (dd == em - 3)
            // Labour Day
            || (d == 1 && m == 5)
            // Ascension Thursday
            || (dd == em + 38)
            // Pancasila Day
            || (d == 1 && m == 6)
            // Independence Day
            || (d == 17 && m == 8)
            // Christmas Day
            || (d == 25 && m == 12)
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
        let cal = Indonesia;
        assert!(!cal.is_business_day(date(2023, 1, 1)));
    }

    #[test]
    fn independence_day() {
        let cal = Indonesia;
        assert!(!cal.is_business_day(date(2023, 8, 17)));
    }

    #[test]
    fn pancasila_day() {
        let cal = Indonesia;
        assert!(!cal.is_business_day(date(2023, 6, 1)));
    }

    #[test]
    fn normal_business_day() {
        let cal = Indonesia;
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
