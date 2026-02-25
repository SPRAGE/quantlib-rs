//! Saudi Arabia calendar.
//!
//! Translates `ql/time/calendars/saudiarabia.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// Saudi Arabia calendar.
///
/// Weekend days are Friday and Saturday.
///
/// Fixed holidays:
/// * National Day (Sep 23)
///
/// Note: Eid al-Fitr and Eid al-Adha vary yearly based on the Islamic
/// calendar and are not included here.
#[derive(Debug, Clone, Copy, Default)]
pub struct SaudiArabia;

impl Calendar for SaudiArabia {
    fn name(&self) -> &str {
        "Saudi Arabia"
    }

    fn is_weekend(&self, date: Date) -> bool {
        matches!(date.weekday(), Weekday::Friday | Weekday::Saturday)
    }

    fn is_business_day(&self, date: Date) -> bool {
        if self.is_weekend(date) {
            return false;
        }
        let m = date.month();
        let d = date.day_of_month();

        if
        // National Day
        d == 23 && m == 9 {
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
    fn friday_is_weekend() {
        let cal = SaudiArabia;
        // 2023-06-16 is a Friday
        assert!(!cal.is_business_day(date(2023, 6, 16)));
    }

    #[test]
    fn saturday_is_weekend() {
        let cal = SaudiArabia;
        // 2023-06-17 is a Saturday
        assert!(!cal.is_business_day(date(2023, 6, 17)));
    }

    #[test]
    fn sunday_is_business_day() {
        let cal = SaudiArabia;
        // 2023-06-18 is a Sunday
        assert!(cal.is_business_day(date(2023, 6, 18)));
    }

    #[test]
    fn national_day() {
        let cal = SaudiArabia;
        // 2023-09-23 is a Saturday → weekend anyway; test 2024-09-23 (Monday)
        assert!(!cal.is_business_day(date(2024, 9, 23)));
    }

    #[test]
    fn normal_business_day() {
        let cal = SaudiArabia;
        // 2023-06-15 is a Thursday
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
