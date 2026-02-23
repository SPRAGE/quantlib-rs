//! Canada (Settlement) calendar.
//!
//! Translates `ql/time/calendars/canada.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// Canada (Settlement) calendar.
///
/// Weekends and the following holidays are observed:
/// * New Year's Day (Jan 1, adjusted)
/// * Family Day (3rd Monday of February, since 2008)
/// * Good Friday (em-3)
/// * Victoria Day (Monday on or before May 24)
/// * Canada Day (Jul 1, adjusted)
/// * Civic Holiday (1st Monday of August)
/// * Labour Day (1st Monday of September)
/// * National Day for Truth and Reconciliation (Sep 30, since 2021, adjusted)
/// * Thanksgiving (2nd Monday of October)
/// * Remembrance Day (Nov 11, adjusted)
/// * Christmas (Dec 25, adjusted)
/// * Boxing Day (Dec 26, adjusted)
#[derive(Debug, Clone, Copy, Default)]
pub struct Canada;

impl Calendar for Canada {
    fn name(&self) -> &str {
        "Canada (Settlement)"
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

        if // New Year's Day (adjusted)
           (d == 1 && m == 1)
            || (d == 2 && m == 1 && w == Weekday::Monday)
            || (d == 3 && m == 1 && w == Weekday::Monday)
            // Family Day (3rd Monday of February, since 2008)
            || (y >= 2008 && w == Weekday::Monday && m == 2 && (15..=21).contains(&d))
            // Good Friday
            || (dd == em - 3)
            // Victoria Day (Monday on or before May 24)
            || (w == Weekday::Monday && m == 5 && (18..=24).contains(&d))
            // Canada Day (Jul 1, adjusted)
            || (d == 1 && m == 7)
            || (d == 2 && m == 7 && w == Weekday::Monday)
            || (d == 3 && m == 7 && w == Weekday::Monday)
            // Civic Holiday (1st Monday of August)
            || (w == Weekday::Monday && m == 8 && (1..=7).contains(&d))
            // Labour Day (1st Monday of September)
            || (w == Weekday::Monday && m == 9 && (1..=7).contains(&d))
            // National Day for Truth and Reconciliation (Sep 30, since 2021, adjusted)
            || (y >= 2021 && d == 30 && m == 9)
            || (y >= 2021 && d == 1 && m == 10 && w == Weekday::Monday)  // Sep 30 = Sun
            || (y >= 2021 && d == 2 && m == 10 && w == Weekday::Monday)  // Sep 30 = Sat
            // Thanksgiving (2nd Monday of October)
            || (w == Weekday::Monday && m == 10 && (8..=14).contains(&d))
            // Remembrance Day (Nov 11, adjusted)
            || (d == 11 && m == 11)
            || (d == 12 && m == 11 && w == Weekday::Monday)
            || (d == 13 && m == 11 && w == Weekday::Monday)
            // Christmas (Dec 25, adjusted)
            || (d == 25 && m == 12)
            || (d == 27 && m == 12 && (w == Weekday::Monday || w == Weekday::Tuesday))
            // Boxing Day (Dec 26, adjusted)
            || (d == 26 && m == 12)
            || (d == 28 && m == 12 && (w == Weekday::Monday || w == Weekday::Tuesday))
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
        let cal = Canada;
        assert!(!cal.is_business_day(date(2023, 1, 1)));
    }

    #[test]
    fn family_day_2023() {
        // 3rd Monday of February 2023 = Feb 20
        let cal = Canada;
        assert!(!cal.is_business_day(date(2023, 2, 20)));
    }

    #[test]
    fn good_friday_2023() {
        let cal = Canada;
        assert!(!cal.is_business_day(date(2023, 4, 7)));
    }

    #[test]
    fn victoria_day_2023() {
        // Monday on or before May 24, 2023: May 22
        let cal = Canada;
        assert!(!cal.is_business_day(date(2023, 5, 22)));
    }

    #[test]
    fn canada_day() {
        let cal = Canada;
        // 2023-07-01 is Saturday → adjusted to Monday Jul 3
        assert!(!cal.is_business_day(date(2023, 7, 3)));
    }

    #[test]
    fn labour_day_2023() {
        // 1st Monday of September 2023 = Sep 4
        let cal = Canada;
        assert!(!cal.is_business_day(date(2023, 9, 4)));
    }

    #[test]
    fn truth_reconciliation_2023() {
        // Sep 30, 2023 is Saturday → adjusted to Monday Oct 2
        let cal = Canada;
        assert!(!cal.is_business_day(date(2023, 10, 2)));
    }

    #[test]
    fn thanksgiving_2023() {
        // 2nd Monday of October 2023 = Oct 9
        let cal = Canada;
        assert!(!cal.is_business_day(date(2023, 10, 9)));
    }

    #[test]
    fn normal_business_day() {
        let cal = Canada;
        // 2023-03-15 is a Wednesday
        assert!(cal.is_business_day(date(2023, 3, 15)));
    }
}
