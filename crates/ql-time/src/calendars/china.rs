//! China (SSE — Shanghai Stock Exchange) calendar.
//!
//! Translates `ql/time/calendars/china.hpp` / `.cpp`.
//!
//! **Note:** Chinese holidays (Spring Festival, Qingming, Dragon Boat,
//! Mid-Autumn) follow the Chinese lunar calendar and vary significantly from
//! year to year.  This implementation covers the fixed holidays and provides a
//! basic version.  Exact dates for lunar-dependent holidays require yearly
//! updates via hard-coded tables, as in the C++ QuantLib implementation.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// China (SSE — Shanghai Stock Exchange) calendar.
///
/// Weekends and the following holidays are observed:
/// * New Year's Day (Jan 1)
/// * Spring Festival (Lunar New Year — varies yearly; basic: Jan 1-3)
/// * Qingming Festival (around Apr 4-5)
/// * Labour Day (May 1)
/// * Dragon Boat Festival (varies yearly)
/// * Mid-Autumn Festival (varies yearly)
/// * National Day (Oct 1-7)
///
/// This basic implementation covers the fixed holidays: New Year (Jan 1),
/// Qingming (Apr 5), Labour Day (May 1), and National Day (Oct 1-3).
/// Full lunar-holiday coverage would require yearly hard-coded tables.
#[derive(Debug, Clone, Copy, Default)]
pub struct China;

impl Calendar for China {
    fn name(&self) -> &str {
        "China (SSE)"
    }

    fn is_business_day(&self, date: Date) -> bool {
        let w = date.weekday();
        if matches!(w, Weekday::Saturday | Weekday::Sunday) {
            return false;
        }
        let m = date.month();
        let d = date.day_of_month();

        if // New Year's Day
           (d == 1 && m == 1)
            // Qingming Festival (approximate fixed date)
            || (d == 5 && m == 4)
            // Labour Day
            || (d == 1 && m == 5)
            // National Day (Oct 1-3 at minimum)
            || (m == 10 && (1..=3).contains(&d))
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
        let cal = China;
        assert!(!cal.is_business_day(date(2023, 1, 1)));
    }

    #[test]
    fn labour_day() {
        let cal = China;
        assert!(!cal.is_business_day(date(2023, 5, 1)));
    }

    #[test]
    fn national_day() {
        let cal = China;
        assert!(!cal.is_business_day(date(2023, 10, 1)));
        assert!(!cal.is_business_day(date(2023, 10, 2)));
        assert!(!cal.is_business_day(date(2023, 10, 3)));
    }

    #[test]
    fn qingming() {
        let cal = China;
        assert!(!cal.is_business_day(date(2023, 4, 5)));
    }

    #[test]
    fn normal_business_day() {
        let cal = China;
        // 2023-06-15 is a Thursday
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
