//! Sweden calendar.
//!
//! Translates `ql/time/calendars/sweden.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// Sweden calendar.
///
/// Weekends and the following holidays are observed:
/// * New Year's Day (Jan 1)
/// * Epiphany (Jan 6)
/// * Good Friday (em-3)
/// * Easter Monday (em)
/// * Labour Day (May 1)
/// * Ascension Thursday (em+38)
/// * National Day (Jun 6)
/// * Midsummer Eve (Friday between Jun 19–25)
/// * All Saints' Day (Saturday between Oct 31 – Nov 6)
/// * Christmas Eve (Dec 24)
/// * Christmas Day (Dec 25)
/// * Boxing Day (Dec 26)
/// * New Year's Eve (Dec 31)
#[derive(Debug, Clone, Copy, Default)]
pub struct Sweden;

impl Calendar for Sweden {
    fn name(&self) -> &str {
        "Sweden"
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

        if // New Year's Day / Epiphany
           ((d == 1 || d == 6) && m == 1)
            // Good Friday
            || (dd == em - 3)
            // Easter Monday
            || (dd == em)
            // Labour Day
            || (d == 1 && m == 5)
            // Ascension Thursday
            || (dd == em + 38)
            // National Day
            || (d == 6 && m == 6)
            // Midsummer Eve (Friday between Jun 19–25)
            || (matches!(w, Weekday::Friday) && m == 6 && (19..=25).contains(&d))
            // All Saints' Day is Saturday (Oct 31 – Nov 6) — the Friday
            // before is sometimes a half day; we mark the Saturday (already
            // a non-business day as weekend).  No extra weekday off.
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
        let cal = Sweden;
        assert!(!cal.is_business_day(date(2023, 1, 1)));
    }

    #[test]
    fn epiphany() {
        let cal = Sweden;
        assert!(!cal.is_business_day(date(2023, 1, 6)));
    }

    #[test]
    fn midsummer_eve_2023() {
        // 2023: Jun 23 is a Friday
        let cal = Sweden;
        assert!(!cal.is_business_day(date(2023, 6, 23)));
    }

    #[test]
    fn national_day() {
        let cal = Sweden;
        assert!(!cal.is_business_day(date(2023, 6, 6)));
    }

    #[test]
    fn new_years_eve() {
        let cal = Sweden;
        assert!(!cal.is_business_day(date(2024, 12, 31)));
    }

    #[test]
    fn normal_business_day() {
        let cal = Sweden;
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
