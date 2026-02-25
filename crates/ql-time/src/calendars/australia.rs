//! Australia calendar.
//!
//! Translates `ql/time/calendars/australia.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// Australia calendar.
///
/// Weekends and the following holidays are observed:
/// * New Year's Day (Jan 1, adjusted to Monday if on weekend)
/// * Australia Day (Jan 26, adjusted to Monday if on weekend)
/// * Good Friday
/// * Easter Saturday
/// * Easter Monday
/// * Anzac Day (Apr 25)
/// * Queen's Birthday (2nd Monday of June)
/// * Christmas Day (Dec 25, adjusted)
/// * Boxing Day (Dec 26, adjusted)
#[derive(Debug, Clone, Copy, Default)]
pub struct Australia;

impl Calendar for Australia {
    fn name(&self) -> &str {
        "Australia"
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
        // New Year's Day (adjusted)
        (d == 1 && m == 1)
            || (d == 2 && m == 1 && w == Weekday::Monday)  // Sun -> Mon
            || (d == 3 && m == 1 && w == Weekday::Monday)  // Sat -> Mon (via Sun adj)
            // Australia Day (Jan 26, adjusted)
            || (d == 26 && m == 1)
            || (d == 27 && m == 1 && w == Weekday::Monday)
            || (d == 28 && m == 1 && w == Weekday::Monday)
            // Good Friday
            || (dd == em - 3)
            // Easter Saturday
            || (dd == em - 2)
            // Easter Monday
            || (dd == em)
            // Anzac Day (Apr 25)
            || (d == 25 && m == 4)
            // Queen's Birthday (2nd Monday of June)
            || (w == Weekday::Monday && m == 6 && (8..=14).contains(&d))
            // Christmas (Dec 25, adjusted)
            || (d == 25 && m == 12)
            || (d == 27 && m == 12 && (w == Weekday::Monday || w == Weekday::Tuesday))
            // Boxing Day (Dec 26, adjusted)
            || (d == 26 && m == 12)
            || (d == 28 && m == 12 && (w == Weekday::Monday || w == Weekday::Tuesday))
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
        let cal = Australia;
        assert!(!cal.is_business_day(date(2023, 1, 1)));
    }

    #[test]
    fn australia_day() {
        let cal = Australia;
        assert!(!cal.is_business_day(date(2023, 1, 26)));
    }

    #[test]
    fn good_friday_and_easter_2023() {
        // Easter Monday 2023: April 10 → Good Friday April 7, Easter Saturday April 8
        let cal = Australia;
        assert!(!cal.is_business_day(date(2023, 4, 7))); // Good Friday
        assert!(!cal.is_business_day(date(2023, 4, 8))); // Easter Saturday (also Saturday)
        assert!(!cal.is_business_day(date(2023, 4, 10))); // Easter Monday
    }

    #[test]
    fn anzac_day() {
        let cal = Australia;
        assert!(!cal.is_business_day(date(2023, 4, 25)));
    }

    #[test]
    fn queens_birthday_2023() {
        // 2nd Monday of June 2023 = Jun 12
        let cal = Australia;
        assert!(!cal.is_business_day(date(2023, 6, 12)));
    }

    #[test]
    fn normal_business_day() {
        let cal = Australia;
        // 2023-03-15 is a Wednesday
        assert!(cal.is_business_day(date(2023, 3, 15)));
    }

    #[test]
    fn christmas_on_weekend_adjusted() {
        // 2021: Dec 25 = Saturday, Dec 26 = Sunday
        // Christmas adj to Mon Dec 27, Boxing Day adj to Tue Dec 28
        let cal = Australia;
        assert!(!cal.is_business_day(date(2021, 12, 27)));
        assert!(!cal.is_business_day(date(2021, 12, 28)));
    }
}
