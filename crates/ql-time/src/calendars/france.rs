//! France calendar.
//!
//! Translates `ql/time/calendars/france.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// France calendar.
///
/// Weekends and the following holidays are observed:
/// * New Year's Day (Jan 1)
/// * Easter Monday (em)
/// * Labour Day (May 1)
/// * Victory Day (May 8)
/// * Ascension Thursday (em+38)
/// * Whit Monday (em+49)
/// * Bastille Day (Jul 14)
/// * Assumption of Mary (Aug 15)
/// * All Saints' Day (Nov 1)
/// * Armistice Day (Nov 11)
/// * Christmas Day (Dec 25)
#[derive(Debug, Clone, Copy, Default)]
pub struct France;

impl Calendar for France {
    fn name(&self) -> &str {
        "France"
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
            // Easter Monday
            || (dd == em)
            // Labour Day
            || (d == 1 && m == 5)
            // Victory Day
            || (d == 8 && m == 5)
            // Ascension Thursday (39 days after Easter Sunday = em + 38)
            || (dd == em + 38)
            // Whit Monday (49 days after Easter Monday = em + 49)
            || (dd == em + 49)
            // Bastille Day
            || (d == 14 && m == 7)
            // Assumption of Mary
            || (d == 15 && m == 8)
            // All Saints' Day
            || (d == 1 && m == 11)
            // Armistice Day
            || (d == 11 && m == 11)
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
        let cal = France;
        assert!(!cal.is_business_day(date(2023, 1, 1)));
    }

    #[test]
    fn easter_monday_2023() {
        // Easter Monday 2023: April 10
        let cal = France;
        assert!(!cal.is_business_day(date(2023, 4, 10)));
    }

    #[test]
    fn bastille_day() {
        let cal = France;
        assert!(!cal.is_business_day(date(2023, 7, 14)));
    }

    #[test]
    fn ascension_2023() {
        // Easter Monday 2023: April 10
        // Ascension = em + 38 = day 100 + 38 = day 138
        // April has 30 days, so day 138 = May 18
        let cal = France;
        assert!(!cal.is_business_day(date(2023, 5, 18)));
    }

    #[test]
    fn whit_monday_2023() {
        // Easter Monday 2023: April 10
        // Whit Monday = em + 49 => May 29
        let cal = France;
        assert!(!cal.is_business_day(date(2023, 5, 29)));
    }

    #[test]
    fn armistice_day() {
        let cal = France;
        // Nov 11, 2024 is a Monday
        assert!(!cal.is_business_day(date(2024, 11, 11)));
    }

    #[test]
    fn normal_business_day() {
        let cal = France;
        // 2023-06-15 is a Thursday
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
