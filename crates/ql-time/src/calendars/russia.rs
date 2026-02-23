//! Russia calendar.
//!
//! Translates `ql/time/calendars/russia.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// Russia calendar.
///
/// Weekends and the following holidays are observed:
/// * New Year's Holidays (Jan 1–5)
/// * Orthodox Christmas (Jan 7)
/// * Defender of the Fatherland Day (Feb 23)
/// * International Women's Day (Mar 8)
/// * Labour Day (May 1)
/// * Victory Day (May 9)
/// * Russia Day (Jun 12)
/// * Unity Day (Nov 4)
///
/// If a holiday falls on Saturday or Sunday, the next Monday is also a holiday.
#[derive(Debug, Clone, Copy, Default)]
pub struct Russia;

impl Russia {
    /// Returns `true` if the given date is a fixed Russian holiday (regardless
    /// of weekday).
    fn is_fixed_holiday(m: u8, d: u8) -> bool {
        // New Year's Holidays / Orthodox Christmas
        (((1..=5).contains(&d) || d == 7) && m == 1)
        // Defender of the Fatherland Day
        || (m == 2 && d == 23)
        // International Women's Day
        || (m == 3 && d == 8)
        // Labour Day
        || (m == 5 && d == 1)
        // Victory Day
        || (m == 5 && d == 9)
        // Russia Day
        || (m == 6 && d == 12)
        // Unity Day
        || (m == 11 && d == 4)
    }
}

impl Calendar for Russia {
    fn name(&self) -> &str {
        "Russia"
    }

    fn is_business_day(&self, date: Date) -> bool {
        let w = date.weekday();
        if matches!(w, Weekday::Saturday | Weekday::Sunday) {
            return false;
        }
        let m = date.month();
        let d = date.day_of_month();

        // Check if today is a fixed holiday
        if Self::is_fixed_holiday(m, d) {
            return false;
        }

        // Transfer rule: if a holiday fell on Saturday (yesterday-2) or
        // Sunday (yesterday-1) the next Monday is off.
        if matches!(w, Weekday::Monday) {
            // Check Saturday (d-2) and Sunday (d-1)
            // We need to handle month boundaries, so use Date arithmetic.
            let sat = date - 2;
            let sun = date - 1;
            if Self::is_fixed_holiday(sat.month(), sat.day_of_month())
                || Self::is_fixed_holiday(sun.month(), sun.day_of_month())
            {
                return false;
            }
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
    fn new_year_holidays() {
        let cal = Russia;
        assert!(!cal.is_business_day(date(2023, 1, 1)));
        assert!(!cal.is_business_day(date(2023, 1, 2)));
        assert!(!cal.is_business_day(date(2023, 1, 3)));
        assert!(!cal.is_business_day(date(2023, 1, 4)));
        assert!(!cal.is_business_day(date(2023, 1, 5)));
    }

    #[test]
    fn orthodox_christmas() {
        let cal = Russia;
        // 2024-01-07 is a Sunday → not a business day (weekend)
        assert!(!cal.is_business_day(date(2024, 1, 7)));
        // Transfer: 2024-01-08 is Monday, should be off
        assert!(!cal.is_business_day(date(2024, 1, 8)));
    }

    #[test]
    fn victory_day() {
        let cal = Russia;
        assert!(!cal.is_business_day(date(2023, 5, 9)));
    }

    #[test]
    fn russia_day() {
        let cal = Russia;
        assert!(!cal.is_business_day(date(2023, 6, 12)));
    }

    #[test]
    fn normal_business_day() {
        let cal = Russia;
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
