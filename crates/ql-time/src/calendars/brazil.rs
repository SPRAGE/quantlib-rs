//! Brazil (Settlement) calendar.
//!
//! Translates `ql/time/calendars/brazil.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// Brazil (Settlement) calendar.
///
/// Weekends and the following holidays are observed:
/// * New Year's Day (Jan 1)
/// * Carnival Monday & Tuesday (em-49, em-48)
/// * Good Friday (em-3)
/// * Tiradentes Day (Apr 21)
/// * Labour Day (May 1)
/// * Corpus Christi (em+59)
/// * Independence Day (Sep 7)
/// * Our Lady of Aparecida (Oct 12)
/// * All Souls' Day (Nov 2)
/// * Republic Day (Nov 15)
/// * Christmas (Dec 25)
#[derive(Debug, Clone, Copy, Default)]
pub struct Brazil;

impl Calendar for Brazil {
    fn name(&self) -> &str {
        "Brazil (Settlement)"
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
            // Carnival Monday
            || (dd == em - 49)
            // Carnival Tuesday
            || (dd == em - 48)
            // Good Friday
            || (dd == em - 3)
            // Tiradentes Day
            || (d == 21 && m == 4)
            // Labour Day
            || (d == 1 && m == 5)
            // Corpus Christi
            || (dd == em + 59)
            // Independence Day
            || (d == 7 && m == 9)
            // Our Lady of Aparecida
            || (d == 12 && m == 10)
            // All Souls' Day
            || (d == 2 && m == 11)
            // Republic Day
            || (d == 15 && m == 11)
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
        let cal = Brazil;
        assert!(!cal.is_business_day(date(2023, 1, 1)));
    }

    #[test]
    fn carnival_2023() {
        // Easter Monday 2023: April 10
        // Carnival Monday = em - 49 => Feb 20
        // Carnival Tuesday = em - 48 => Feb 21
        // Ash Wednesday = em - 46 => Feb 22 (actually, let's verify: Apr 10 - 46 = Feb 23)
        let cal = Brazil;
        assert!(!cal.is_business_day(date(2023, 2, 20)));
        assert!(!cal.is_business_day(date(2023, 2, 21)));
    }

    #[test]
    fn good_friday_2023() {
        let cal = Brazil;
        assert!(!cal.is_business_day(date(2023, 4, 7)));
    }

    #[test]
    fn tiradentes_day() {
        let cal = Brazil;
        assert!(!cal.is_business_day(date(2023, 4, 21)));
    }

    #[test]
    fn independence_day() {
        let cal = Brazil;
        assert!(!cal.is_business_day(date(2023, 9, 7)));
    }

    #[test]
    fn normal_business_day() {
        let cal = Brazil;
        // 2023-06-15 is a Thursday
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
