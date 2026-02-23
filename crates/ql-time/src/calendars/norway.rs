//! Norway calendar.
//!
//! Translates `ql/time/calendars/norway.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// Norway calendar.
///
/// Weekends and the following holidays are observed:
/// * New Year's Day (Jan 1)
/// * Maundy Thursday (em-4)
/// * Good Friday (em-3)
/// * Easter Monday (em)
/// * Labour Day (May 1)
/// * Ascension Thursday (em+38)
/// * Constitution Day (May 17)
/// * Whit Monday (em+49)
/// * Christmas Day (Dec 25)
/// * Boxing Day (Dec 26)
#[derive(Debug, Clone, Copy, Default)]
pub struct Norway;

impl Calendar for Norway {
    fn name(&self) -> &str {
        "Norway"
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
            // Maundy Thursday
            || (dd == em - 4)
            // Good Friday
            || (dd == em - 3)
            // Easter Monday
            || (dd == em)
            // Labour Day
            || (d == 1 && m == 5)
            // Ascension Thursday
            || (dd == em + 38)
            // Constitution Day
            || (d == 17 && m == 5)
            // Whit Monday
            || (dd == em + 49)
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
        let cal = Norway;
        assert!(!cal.is_business_day(date(2023, 1, 1)));
    }

    #[test]
    fn constitution_day() {
        let cal = Norway;
        assert!(!cal.is_business_day(date(2023, 5, 17)));
    }

    #[test]
    fn maundy_thursday_2023() {
        // Easter Monday 2023: April 10, Maundy Thursday = April 6
        let cal = Norway;
        assert!(!cal.is_business_day(date(2023, 4, 6)));
    }

    #[test]
    fn normal_business_day() {
        let cal = Norway;
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
