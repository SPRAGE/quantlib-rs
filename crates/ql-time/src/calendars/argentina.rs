//! Argentina calendar.
//!
//! Translates `ql/time/calendars/argentina.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// Argentina calendar.
///
/// Weekends and the following holidays are observed:
/// * New Year's Day (Jan 1)
/// * Carnival Monday & Tuesday (Mon-Tue before Ash Wednesday)
/// * Truth & Justice Memorial Day (Mar 24)
/// * Malvinas Day (Apr 2)
/// * Good Friday
/// * Labour Day (May 1)
/// * Revolution Day (May 25)
/// * Flag Day (Jun 20)
/// * Independence Day (Jul 9)
/// * Death of General San Martín (3rd Monday of August)
/// * Respect for Cultural Diversity Day (2nd Monday of October)
/// * Sovereignty Day (4th Monday of November)
/// * Immaculate Conception (Dec 8)
/// * Christmas (Dec 25)
#[derive(Debug, Clone, Copy, Default)]
pub struct Argentina;

impl Calendar for Argentina {
    fn name(&self) -> &str {
        "Argentina"
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
            // Carnival Monday (em - 49)
            || (dd == em - 49)
            // Carnival Tuesday (em - 48)
            || (dd == em - 48)
            // Truth & Justice Memorial Day
            || (d == 24 && m == 3)
            // Malvinas Day
            || (d == 2 && m == 4)
            // Good Friday
            || (dd == em - 3)
            // Labour Day
            || (d == 1 && m == 5)
            // Revolution Day
            || (d == 25 && m == 5)
            // Flag Day
            || (d == 20 && m == 6)
            // Independence Day
            || (d == 9 && m == 7)
            // Death of General San Martín (3rd Monday of August)
            || (w == Weekday::Monday && m == 8 && (15..=21).contains(&d))
            // Respect for Cultural Diversity Day (2nd Monday of October)
            || (w == Weekday::Monday && m == 10 && (8..=14).contains(&d))
            // Sovereignty Day (4th Monday of November)
            || (w == Weekday::Monday && m == 11 && (22..=28).contains(&d))
            // Immaculate Conception
            || (d == 8 && m == 12)
            // Christmas
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
        let cal = Argentina;
        assert!(!cal.is_business_day(date(2023, 1, 1)));
    }

    #[test]
    fn good_friday_2023() {
        // Easter Monday 2023: April 10 → Good Friday April 7
        let cal = Argentina;
        assert!(!cal.is_business_day(date(2023, 4, 7)));
    }

    #[test]
    fn carnival_2023() {
        // Easter Monday 2023: April 10
        // Carnival Monday = em - 49 days => Feb 20
        // Carnival Tuesday = em - 48 days => Feb 21
        let cal = Argentina;
        assert!(!cal.is_business_day(date(2023, 2, 20)));
        assert!(!cal.is_business_day(date(2023, 2, 21)));
    }

    #[test]
    fn independence_day() {
        let cal = Argentina;
        assert!(!cal.is_business_day(date(2023, 7, 9)));
    }

    #[test]
    fn normal_business_day() {
        let cal = Argentina;
        // 2023-06-15 is a Thursday
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }

    #[test]
    fn death_of_san_martin_2023() {
        // 3rd Monday of August 2023 = Aug 21
        let cal = Argentina;
        assert!(!cal.is_business_day(date(2023, 8, 21)));
    }
}
