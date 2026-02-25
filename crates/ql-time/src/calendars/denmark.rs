//! Denmark calendar.
//!
//! Translates `ql/time/calendars/denmark.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// Denmark calendar.
///
/// Weekends and the following holidays are observed:
/// * New Year's Day (Jan 1)
/// * Maundy Thursday (em-4)
/// * Good Friday (em-3)
/// * Easter Monday (em)
/// * Great Prayer Day (em+25, removed from 2024 onwards)
/// * Ascension Thursday (em+38)
/// * Whit Monday (em+49)
/// * Constitution Day (Jun 5)
/// * Christmas Day (Dec 25)
/// * Boxing Day (Dec 26)
#[derive(Debug, Clone, Copy, Default)]
pub struct Denmark;

impl Calendar for Denmark {
    fn name(&self) -> &str {
        "Denmark"
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
            // Great Prayer Day (removed from 2024)
            || (dd == em + 25 && y < 2024)
            // Ascension Thursday
            || (dd == em + 38)
            // Day after Ascension (from 2009)
            || (dd == em + 39 && y >= 2009)
            // Whit Monday
            || (dd == em + 49)
            // Constitution Day
            || (d == 5 && m == 6)
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
        let cal = Denmark;
        assert!(!cal.is_business_day(date(2023, 1, 1)));
    }

    #[test]
    fn maundy_thursday_2023() {
        // Easter Monday 2023: April 10, Maundy Thursday = April 6
        let cal = Denmark;
        assert!(!cal.is_business_day(date(2023, 4, 6)));
    }

    #[test]
    fn great_prayer_day_removed_2024() {
        let cal = Denmark;
        // 2023: Great Prayer Day observed (em+25)
        // Easter Monday 2023: April 10, em+25 = May 5
        assert!(!cal.is_business_day(date(2023, 5, 5)));
        // 2024: Great Prayer Day removed
        // Easter Monday 2024: April 1, em+25 = April 26 (Saturday — skip this test)
        // Use 2025: Easter Monday 2025: April 21, em+25 = May 16
        assert!(cal.is_business_day(date(2025, 5, 16)));
    }

    #[test]
    fn constitution_day() {
        let cal = Denmark;
        assert!(!cal.is_business_day(date(2023, 6, 5)));
    }

    #[test]
    fn normal_business_day() {
        let cal = Denmark;
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
