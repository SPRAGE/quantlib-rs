//! Hong Kong (HKEx) calendar.
//!
//! Translates `ql/time/calendars/hongkong.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// Hong Kong (HKEx) calendar.
///
/// Weekends and the following holidays are observed:
/// * New Year's Day (Jan 1)
/// * Lunar New Year (approx Jan 22–24 — simplified)
/// * Good Friday (em-3)
/// * Easter Saturday (em-2)
/// * Easter Monday (em)
/// * Ching Ming Festival (Apr 5)
/// * Labour Day (May 1)
/// * Buddha's Birthday (approx May 26)
/// * SAR Establishment Day (Jul 1)
/// * National Day (Oct 1)
/// * Christmas Day (Dec 25)
/// * Boxing Day (Dec 26)
#[derive(Debug, Clone, Copy, Default)]
pub struct HongKong;

impl Calendar for HongKong {
    fn name(&self) -> &str {
        "Hong Kong (HKEx)"
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
        // New Year's Day / Lunar New Year (simplified fixed dates)
        ((d == 1 || (22..=24).contains(&d)) && m == 1)
            // Good Friday
            || (dd == em - 3)
            // Easter Saturday (day before Easter Monday)
            || (dd == em - 2)
            // Easter Monday
            || (dd == em)
            // Ching Ming Festival
            || (d == 5 && m == 4)
            // Labour Day
            || (d == 1 && m == 5)
            // Buddha's Birthday (approximate)
            || (d == 26 && m == 5)
            // SAR Establishment Day
            || (d == 1 && m == 7)
            // National Day
            || (d == 1 && m == 10)
            // Christmas Day
            || (d == 25 && m == 12)
            // Boxing Day
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
    fn good_friday_and_easter_monday_2023() {
        let cal = HongKong;
        // Easter Monday 2023: April 10 → Good Friday April 7
        assert!(!cal.is_business_day(date(2023, 4, 7))); // Good Friday
        assert!(!cal.is_business_day(date(2023, 4, 8))); // Easter Saturday (Sat, already weekend)
        assert!(!cal.is_business_day(date(2023, 4, 10))); // Easter Monday
    }

    #[test]
    fn sar_establishment_day() {
        let cal = HongKong;
        // Jul 1, 2024 is a Monday
        assert!(!cal.is_business_day(date(2024, 7, 1)));
    }

    #[test]
    fn national_day() {
        let cal = HongKong;
        // Oct 1, 2024 is a Tuesday
        assert!(!cal.is_business_day(date(2024, 10, 1)));
    }

    #[test]
    fn normal_business_day() {
        let cal = HongKong;
        // 2023-06-15 is a Thursday
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
