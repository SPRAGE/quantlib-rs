//! Chile calendar.
//!
//! Translates `ql/time/calendars/chile.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// Chile calendar.
///
/// Weekends and the following holidays are observed:
/// * New Year's Day (Jan 1)
/// * Good Friday (em-3)
/// * Easter Saturday (em-2)
/// * Labour Day (May 1)
/// * Naval Glorias (May 21)
/// * Corpus Christi (em+60, removed in 2007, re-added from 2009)
/// * San Pedro y San Pablo (Jun 29)
/// * Virgen del Carmen (Jul 16)
/// * Assumption of Mary (Aug 15)
/// * Independence Day (Sep 18)
/// * Army Day (Sep 19)
/// * Día de la Raza (Oct 12)
/// * Reformation Day (Oct 31)
/// * All Saints' Day (Nov 1)
/// * Immaculate Conception (Dec 8)
/// * Christmas Day (Dec 25)
#[derive(Debug, Clone, Copy, Default)]
pub struct Chile;

impl Calendar for Chile {
    fn name(&self) -> &str {
        "Chile"
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
            // Good Friday
            || (dd == em - 3)
            // Easter Saturday
            || (dd == em - 2)
            // Labour Day
            || (d == 1 && m == 5)
            // Naval Glorias
            || (d == 21 && m == 5)
            // Corpus Christi (removed in 2007, re-added from 2009)
            || (dd == em + 60 && !(2007..2009).contains(&y))
            // San Pedro y San Pablo
            || (d == 29 && m == 6)
            // Virgen del Carmen
            || (d == 16 && m == 7)
            // Assumption of Mary
            || (d == 15 && m == 8)
            // Independence Day
            || (d == 18 && m == 9)
            // Army Day
            || (d == 19 && m == 9)
            // Día de la Raza
            || (d == 12 && m == 10)
            // Reformation Day
            || (d == 31 && m == 10)
            // All Saints' Day
            || (d == 1 && m == 11)
            // Immaculate Conception
            || (d == 8 && m == 12)
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
        let cal = Chile;
        assert!(!cal.is_business_day(date(2023, 1, 1)));
    }

    #[test]
    fn independence_day() {
        let cal = Chile;
        assert!(!cal.is_business_day(date(2023, 9, 18)));
    }

    #[test]
    fn army_day() {
        let cal = Chile;
        assert!(!cal.is_business_day(date(2023, 9, 19)));
    }

    #[test]
    fn corpus_christi_removed_2007_readded_2009() {
        let cal = Chile;
        // 2023: Corpus Christi should be observed (em+60)
        // Easter Monday 2023: April 10 (doy 100), + 60 = doy 160 = Jun 9
        assert!(!cal.is_business_day(date(2023, 6, 9)));
        // 2008: Corpus Christi NOT observed
        // Easter Monday 2008: March 24 (doy 84), + 60 = doy 144 = May 23
        assert!(cal.is_business_day(date(2008, 5, 23)));
    }

    #[test]
    fn normal_business_day() {
        let cal = Chile;
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
