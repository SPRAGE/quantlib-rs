//! South Africa calendar.
//!
//! Translates `ql/time/calendars/southafrica.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// South Africa calendar.
///
/// Weekends and the following holidays are observed:
/// * New Year's Day (Jan 1)
/// * Human Rights Day (Mar 21)
/// * Good Friday (em-3)
/// * Family Day (em, Easter Monday)
/// * Freedom Day (Apr 27)
/// * Workers' Day (May 1)
/// * Youth Day (Jun 16)
/// * National Women's Day (Aug 9)
/// * Heritage Day (Sep 24)
/// * Day of Reconciliation (Dec 16)
/// * Christmas Day (Dec 25)
/// * Day of Goodwill (Dec 26)
///
/// If any public holiday falls on a Sunday, the following Monday is observed.
#[derive(Debug, Clone, Copy, Default)]
pub struct SouthAfrica;

impl Calendar for SouthAfrica {
    fn name(&self) -> &str {
        "South Africa"
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

        // Helper: fixed holiday with Sunday→Monday substitution
        let is_holiday = |hm: u8, hd: u8| -> bool {
            (d == hd && m == hm) || (d == hd + 1 && m == hm && w == Weekday::Monday)
        };

        // Handle month boundary for holidays on the last day would be rare
        // (none of the SA holidays fall on the 31st), so the +1 logic is safe.

        if
        // New Year's Day
        is_holiday(1, 1)
            // Human Rights Day
            || is_holiday(3, 21)
            // Good Friday
            || (dd == em - 3)
            // Family Day (Easter Monday)
            || (dd == em)
            // Freedom Day
            || is_holiday(4, 27)
            // Workers' Day
            || is_holiday(5, 1)
            // Youth Day
            || is_holiday(6, 16)
            // National Women's Day
            || is_holiday(8, 9)
            // Heritage Day
            || is_holiday(9, 24)
            // Day of Reconciliation
            || is_holiday(12, 16)
            // Christmas Day
            || is_holiday(12, 25)
            // Day of Goodwill
            || is_holiday(12, 26)
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
    fn human_rights_day() {
        let cal = SouthAfrica;
        // Mar 21, 2023 is a Tuesday
        assert!(!cal.is_business_day(date(2023, 3, 21)));
    }

    #[test]
    fn good_friday_and_family_day_2023() {
        let cal = SouthAfrica;
        // Easter Monday 2023: April 10 → Good Friday April 7
        assert!(!cal.is_business_day(date(2023, 4, 7))); // Good Friday
        assert!(!cal.is_business_day(date(2023, 4, 10))); // Family Day
    }

    #[test]
    fn freedom_day_sunday_substitute() {
        let cal = SouthAfrica;
        // Apr 27, 2025 is a Sunday → Monday Apr 28 is the substitute
        assert!(!cal.is_business_day(date(2025, 4, 28)));
        // Apr 28 itself should be marked as a holiday (Monday substitute)
        assert_eq!(date(2025, 4, 28).weekday(), Weekday::Monday);
    }

    #[test]
    fn normal_business_day() {
        let cal = SouthAfrica;
        // 2023-06-15 is a Thursday
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
