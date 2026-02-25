//! Slovakia calendar.
//!
//! Translates `ql/time/calendars/slovakia.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// Slovakia calendar.
///
/// Weekends and the following holidays are observed:
/// * New Year's Day / Republic Day (Jan 1)
/// * Epiphany (Jan 6)
/// * Good Friday (em-3)
/// * Easter Monday (em)
/// * Labour Day (May 1)
/// * Victory Day (May 8)
/// * Saints Cyril & Methodius Day (Jul 5)
/// * SNP Anniversary (Aug 29)
/// * Constitution Day (Sep 1)
/// * Our Lady of Seven Sorrows (Sep 15)
/// * All Saints' Day (Nov 1)
/// * Freedom & Democracy Day (Nov 17)
/// * Christmas Eve (Dec 24)
/// * Christmas Day (Dec 25)
/// * St. Stephen's Day (Dec 26)
#[derive(Debug, Clone, Copy, Default)]
pub struct Slovakia;

impl Calendar for Slovakia {
    fn name(&self) -> &str {
        "Slovakia"
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
        // New Year's Day / Republic Day / Epiphany
        ((d == 1 || d == 6) && m == 1)
            // Good Friday
            || (dd == em - 3)
            // Easter Monday
            || (dd == em)
            // Labour Day
            || (d == 1 && m == 5)
            // Victory Day
            || (d == 8 && m == 5)
            // Saints Cyril & Methodius Day
            || (d == 5 && m == 7)
            // SNP Anniversary
            || (d == 29 && m == 8)
            // Constitution Day
            || (d == 1 && m == 9)
            // Our Lady of Seven Sorrows
            || (d == 15 && m == 9)
            // All Saints' Day
            || (d == 1 && m == 11)
            // Freedom & Democracy Day
            || (d == 17 && m == 11)
            // Christmas Eve
            || (d == 24 && m == 12)
            // Christmas Day
            || (d == 25 && m == 12)
            // St. Stephen's Day
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
        let cal = Slovakia;
        assert!(!cal.is_business_day(date(2023, 1, 1)));
    }

    #[test]
    fn epiphany() {
        let cal = Slovakia;
        assert!(!cal.is_business_day(date(2023, 1, 6)));
    }

    #[test]
    fn snp_anniversary() {
        let cal = Slovakia;
        assert!(!cal.is_business_day(date(2023, 8, 29)));
    }

    #[test]
    fn freedom_democracy_day() {
        let cal = Slovakia;
        assert!(!cal.is_business_day(date(2023, 11, 17)));
    }

    #[test]
    fn normal_business_day() {
        let cal = Slovakia;
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
