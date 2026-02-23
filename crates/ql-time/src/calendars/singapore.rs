//! Singapore (SGX) calendar.
//!
//! Translates `ql/time/calendars/singapore.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// Singapore (SGX) calendar.
///
/// Weekends and the following fixed/Easter-based holidays are observed:
/// * New Year's Day (Jan 1)
/// * Good Friday (em-3)
/// * Labour Day (May 1)
/// * National Day (Aug 9)
/// * Christmas Day (Dec 25)
///
/// Note: Chinese New Year, Hari Raya Puasa, Hari Raya Haji, Vesak Day, and
/// Deepavali vary yearly and are not included here.
#[derive(Debug, Clone, Copy, Default)]
pub struct Singapore;

impl Calendar for Singapore {
    fn name(&self) -> &str {
        "Singapore (SGX)"
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

        if // New Year's Day
           (d == 1 && m == 1)
            // Good Friday
            || (dd == em - 3)
            // Labour Day
            || (d == 1 && m == 5)
            // National Day
            || (d == 9 && m == 8)
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
        let cal = Singapore;
        assert!(!cal.is_business_day(date(2023, 1, 1)));
    }

    #[test]
    fn national_day() {
        let cal = Singapore;
        assert!(!cal.is_business_day(date(2023, 8, 9)));
    }

    #[test]
    fn good_friday_2023() {
        // Easter Monday 2023: April 10, Good Friday = April 7
        let cal = Singapore;
        assert!(!cal.is_business_day(date(2023, 4, 7)));
    }

    #[test]
    fn normal_business_day() {
        let cal = Singapore;
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
