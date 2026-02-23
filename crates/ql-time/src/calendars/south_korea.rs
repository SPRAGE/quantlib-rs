//! South Korea calendar.
//!
//! Translates `ql/time/calendars/southkorea.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// South Korea calendar.
///
/// Weekends and the following fixed holidays are observed:
/// * New Year's Day (Jan 1)
/// * Independence Movement Day (Mar 1)
/// * Labour Day (May 1)
/// * Children's Day (May 5)
/// * Memorial Day (Jun 6)
/// * Liberation Day (Aug 15)
/// * National Foundation Day (Oct 3)
/// * Hangul Day (Oct 9)
/// * Christmas Day (Dec 25)
///
/// Note: Korean New Year (Lunar), Buddha's Birthday, and Chuseok vary
/// yearly based on the lunar calendar and are not included here.
#[derive(Debug, Clone, Copy, Default)]
pub struct SouthKorea;

impl Calendar for SouthKorea {
    fn name(&self) -> &str {
        "South Korea"
    }

    fn is_business_day(&self, date: Date) -> bool {
        let w = date.weekday();
        if matches!(w, Weekday::Saturday | Weekday::Sunday) {
            return false;
        }
        let m = date.month();
        let d = date.day_of_month();

        if // New Year's Day / Independence Movement Day / Labour Day
           (d == 1 && matches!(m, 1 | 3 | 5))
            // Children's Day
            || (d == 5 && m == 5)
            // Memorial Day
            || (d == 6 && m == 6)
            // Liberation Day
            || (d == 15 && m == 8)
            // National Foundation Day
            || (d == 3 && m == 10)
            // Hangul Day
            || (d == 9 && m == 10)
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
        let cal = SouthKorea;
        assert!(!cal.is_business_day(date(2023, 1, 1)));
    }

    #[test]
    fn independence_movement_day() {
        let cal = SouthKorea;
        assert!(!cal.is_business_day(date(2023, 3, 1)));
    }

    #[test]
    fn hangul_day() {
        let cal = SouthKorea;
        assert!(!cal.is_business_day(date(2023, 10, 9)));
    }

    #[test]
    fn liberation_day() {
        let cal = SouthKorea;
        assert!(!cal.is_business_day(date(2023, 8, 15)));
    }

    #[test]
    fn normal_business_day() {
        let cal = SouthKorea;
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
