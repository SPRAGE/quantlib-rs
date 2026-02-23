//! Ukraine calendar.
//!
//! Translates `ql/time/calendars/ukraine.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// Ukraine calendar.
///
/// Weekends and the following holidays are observed:
/// * New Year's Day (Jan 1)
/// * Orthodox Christmas (Jan 7)
/// * International Women's Day (Mar 8)
/// * Labour Day (May 1–2)
/// * Victory Day (May 9)
/// * Constitution Day (Jun 28)
/// * Independence Day (Aug 24)
/// * Defender's Day (Oct 14, since 2015)
/// * Christmas (Dec 25, since 2017)
#[derive(Debug, Clone, Copy, Default)]
pub struct Ukraine;

impl Calendar for Ukraine {
    fn name(&self) -> &str {
        "Ukraine"
    }

    fn is_business_day(&self, date: Date) -> bool {
        let w = date.weekday();
        if matches!(w, Weekday::Saturday | Weekday::Sunday) {
            return false;
        }
        let y = date.year();
        let m = date.month();
        let d = date.day_of_month();

        if // New Year's Day / Orthodox Christmas
           ((d == 1 || d == 7) && m == 1)
            // International Women's Day
            || (d == 8 && m == 3)
            // Labour Day (May 1–2)
            || (d == 1 && m == 5)
            || (d == 2 && m == 5)
            // Victory Day
            || (d == 9 && m == 5)
            // Constitution Day
            || (d == 28 && m == 6)
            // Independence Day
            || (d == 24 && m == 8)
            // Defender's Day (since 2015)
            || (d == 14 && m == 10 && y >= 2015)
            // Christmas (since 2017)
            || (d == 25 && m == 12 && y >= 2017)
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
        let cal = Ukraine;
        assert!(!cal.is_business_day(date(2023, 1, 1)));
    }

    #[test]
    fn orthodox_christmas() {
        let cal = Ukraine;
        // 2024-01-07 is a Sunday → weekend; test 2025-01-07 (Tuesday)
        assert!(!cal.is_business_day(date(2025, 1, 7)));
    }

    #[test]
    fn defenders_day_since_2015() {
        let cal = Ukraine;
        // 2023-10-14 is a Saturday → weekend; test 2024-10-14 (Monday)
        assert!(!cal.is_business_day(date(2024, 10, 14)));
        // Not observed before 2015: 2014-10-14 is a Tuesday
        assert!(cal.is_business_day(date(2014, 10, 14)));
    }

    #[test]
    fn christmas_since_2017() {
        let cal = Ukraine;
        // 2023-12-25 is a Monday
        assert!(!cal.is_business_day(date(2023, 12, 25)));
        // Not observed before 2017: 2016-12-25 is a Sunday → weekend anyway
        // Test 2015-12-25 (Friday)
        assert!(cal.is_business_day(date(2015, 12, 25)));
    }

    #[test]
    fn normal_business_day() {
        let cal = Ukraine;
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
