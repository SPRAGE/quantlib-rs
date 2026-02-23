//! Taiwan calendar.
//!
//! Translates `ql/time/calendars/taiwan.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// Taiwan calendar.
///
/// Weekends and the following fixed holidays are observed:
/// * New Year's Day / Founding Day (Jan 1)
/// * Peace Memorial Day (Feb 28)
/// * Tomb Sweeping Day (Apr 4 or Apr 5)
/// * Labour Day (May 1)
/// * National Day (Oct 10)
///
/// Note: Chinese New Year, Dragon Boat Festival, and Moon Festival are based
/// on the lunar calendar and vary yearly; they are not included here.
#[derive(Debug, Clone, Copy, Default)]
pub struct Taiwan;

impl Calendar for Taiwan {
    fn name(&self) -> &str {
        "Taiwan"
    }

    fn is_business_day(&self, date: Date) -> bool {
        let w = date.weekday();
        if matches!(w, Weekday::Saturday | Weekday::Sunday) {
            return false;
        }
        let m = date.month();
        let d = date.day_of_month();

        if // New Year's Day / Founding Day
           (d == 1 && m == 1)
            // Peace Memorial Day
            || (d == 28 && m == 2)
            // Tomb Sweeping Day (Apr 4 or Apr 5)
            || ((d == 4 || d == 5) && m == 4)
            // Labour Day
            || (d == 1 && m == 5)
            // National Day
            || (d == 10 && m == 10)
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
        let cal = Taiwan;
        assert!(!cal.is_business_day(date(2023, 1, 1)));
    }

    #[test]
    fn peace_memorial_day() {
        let cal = Taiwan;
        assert!(!cal.is_business_day(date(2023, 2, 28)));
    }

    #[test]
    fn national_day() {
        let cal = Taiwan;
        assert!(!cal.is_business_day(date(2023, 10, 10)));
    }

    #[test]
    fn normal_business_day() {
        let cal = Taiwan;
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
