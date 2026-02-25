//! Turkey calendar.
//!
//! Translates `ql/time/calendars/turkey.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// Turkey calendar.
///
/// Weekends and the following fixed holidays are observed:
/// * New Year's Day (Jan 1)
/// * National Sovereignty and Children's Day (Apr 23)
/// * Labour Day (May 1)
/// * Commemoration of Atatürk, Youth & Sports Day (May 19)
/// * Democracy and National Unity Day (Jul 15, since 2017)
/// * Victory Day (Aug 30)
/// * Republic Day (Oct 29)
///
/// Note: Ramadan (Eid al-Fitr) and Eid al-Adha vary yearly based on the
/// Islamic calendar and are not included here.
#[derive(Debug, Clone, Copy, Default)]
pub struct Turkey;

impl Calendar for Turkey {
    fn name(&self) -> &str {
        "Turkey"
    }

    fn is_business_day(&self, date: Date) -> bool {
        let w = date.weekday();
        if matches!(w, Weekday::Saturday | Weekday::Sunday) {
            return false;
        }
        let y = date.year();
        let m = date.month();
        let d = date.day_of_month();

        if
        // New Year's Day
        (d == 1 && m == 1)
            // National Sovereignty and Children's Day
            || (d == 23 && m == 4)
            // Labour Day
            || (d == 1 && m == 5)
            // Commemoration of Atatürk, Youth & Sports Day
            || (d == 19 && m == 5)
            // Democracy and National Unity Day (since 2017)
            || (d == 15 && m == 7 && y >= 2017)
            // Victory Day
            || (d == 30 && m == 8)
            // Republic Day
            || (d == 29 && m == 10)
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
        let cal = Turkey;
        assert!(!cal.is_business_day(date(2023, 1, 1)));
    }

    #[test]
    fn national_sovereignty_day() {
        let cal = Turkey;
        assert!(!cal.is_business_day(date(2023, 4, 23)));
    }

    #[test]
    fn democracy_day_since_2017() {
        let cal = Turkey;
        // 2023-07-15 is a Saturday → weekend anyway; test 2024-07-15 (Monday)
        assert!(!cal.is_business_day(date(2024, 7, 15)));
        // Not observed before 2017: 2016-07-15 is a Friday
        assert!(cal.is_business_day(date(2016, 7, 15)));
    }

    #[test]
    fn republic_day() {
        let cal = Turkey;
        assert!(!cal.is_business_day(date(2023, 10, 29)));
    }

    #[test]
    fn normal_business_day() {
        let cal = Turkey;
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
