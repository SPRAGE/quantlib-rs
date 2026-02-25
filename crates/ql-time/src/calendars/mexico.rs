//! Mexico calendar.
//!
//! Translates `ql/time/calendars/mexico.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// Mexico (BMV) calendar.
///
/// Weekends and the following holidays are observed:
/// * New Year's Day (Jan 1)
/// * Constitution Day (1st Monday in February)
/// * Benito Juárez's Birthday (3rd Monday in March)
/// * Good Friday (em-3)
/// * Labour Day (May 1)
/// * Independence Day (Sep 16)
/// * Día de la Raza (Oct 12)
/// * Revolution Day (3rd Monday in November)
/// * Christmas Day (Dec 25)
#[derive(Debug, Clone, Copy, Default)]
pub struct Mexico;

impl Calendar for Mexico {
    fn name(&self) -> &str {
        "Mexico (BMV)"
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
            // Constitution Day (1st Monday in February)
            || (w == Weekday::Monday && m == 2 && (1..=7).contains(&d))
            // Benito Juárez's Birthday (3rd Monday in March)
            || (w == Weekday::Monday && m == 3 && (15..=21).contains(&d))
            // Good Friday
            || (dd == em - 3)
            // Labour Day
            || (d == 1 && m == 5)
            // Independence Day
            || (d == 16 && m == 9)
            // Día de la Raza (Oct 12)
            || (d == 12 && m == 10)
            // Revolution Day (3rd Monday in November)
            || (w == Weekday::Monday && m == 11 && (15..=21).contains(&d))
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
    fn constitution_day_2023() {
        // 1st Monday in February 2023 = Feb 6
        let cal = Mexico;
        assert!(!cal.is_business_day(date(2023, 2, 6)));
    }

    #[test]
    fn good_friday_2023() {
        let cal = Mexico;
        // Easter Monday 2023: April 10 → Good Friday April 7
        assert!(!cal.is_business_day(date(2023, 4, 7)));
    }

    #[test]
    fn independence_day() {
        let cal = Mexico;
        // Sep 16, 2024 is a Monday
        assert!(!cal.is_business_day(date(2024, 9, 16)));
    }

    #[test]
    fn revolution_day_2023() {
        // 3rd Monday in November 2023 = Nov 20
        let cal = Mexico;
        assert!(!cal.is_business_day(date(2023, 11, 20)));
    }

    #[test]
    fn normal_business_day() {
        let cal = Mexico;
        // 2023-06-15 is a Thursday
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
