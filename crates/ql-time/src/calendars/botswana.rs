//! Botswana calendar.
//!
//! Translates `ql/time/calendars/botswana.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// Botswana calendar.
///
/// Weekends and the following holidays are observed:
/// * New Year's Day (Jan 1)
/// * New Year's Holiday (Jan 2)
/// * Good Friday (em-3)
/// * Easter Saturday (em-2)
/// * Easter Monday (em)
/// * Labour Day (May 1)
/// * Ascension Thursday (em+38)
/// * President's Day (3rd Monday in July)
/// * Botswana Day (Sep 30)
/// * Botswana Day Holiday (Oct 1)
/// * Christmas Day (Dec 25)
/// * Boxing Day (Dec 26)
#[derive(Debug, Clone, Copy, Default)]
pub struct Botswana;

impl Calendar for Botswana {
    fn name(&self) -> &str {
        "Botswana"
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
        // New Year's Day + Holiday
        ((d == 1 || d == 2) && m == 1)
            // Good Friday
            || (dd == em - 3)
            // Easter Saturday
            || (dd == em - 2)
            // Easter Monday
            || (dd == em)
            // Labour Day
            || (d == 1 && m == 5)
            // Ascension Thursday
            || (dd == em + 38)
            // President's Day (3rd Monday in July)
            || (matches!(w, Weekday::Monday) && m == 7 && (15..=21).contains(&d))
            // Botswana Day
            || (d == 30 && m == 9)
            // Botswana Day Holiday
            || (d == 1 && m == 10)
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
        let cal = Botswana;
        assert!(!cal.is_business_day(date(2023, 1, 1)));
        assert!(!cal.is_business_day(date(2023, 1, 2)));
    }

    #[test]
    fn presidents_day_2023() {
        // 3rd Monday in July 2023 = Jul 17
        let cal = Botswana;
        assert!(!cal.is_business_day(date(2023, 7, 17)));
    }

    #[test]
    fn botswana_day() {
        let cal = Botswana;
        // 2024-09-30 is a Monday
        assert!(!cal.is_business_day(date(2024, 9, 30)));
        assert!(!cal.is_business_day(date(2024, 10, 1)));
    }

    #[test]
    fn normal_business_day() {
        let cal = Botswana;
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
