//! New Zealand calendar.
//!
//! Translates `ql/time/calendars/newzealand.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// New Zealand calendar.
///
/// Weekends and the following holidays are observed:
/// * New Year's Day (Jan 1, adjusted)
/// * Day after New Year's (Jan 2, adjusted)
/// * Waitangi Day (Feb 6, adjusted)
/// * Good Friday (em-3)
/// * Easter Monday (em)
/// * Anzac Day (Apr 25)
/// * Queen's Birthday (1st Monday in June)
/// * Matariki (approx Jun 24 — simplified fixed date)
/// * Labour Day (4th Monday in October)
/// * Christmas Day (Dec 25, adjusted)
/// * Boxing Day (Dec 26, adjusted)
#[derive(Debug, Clone, Copy, Default)]
pub struct NewZealand;

impl Calendar for NewZealand {
    fn name(&self) -> &str {
        "New Zealand"
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

        if // New Year's Day (adjusted: if Sat → Mon, if Sun → Mon, if Mon → Tue)
           (d == 1 && m == 1)
            || (d == 3 && m == 1 && w == Weekday::Monday)   // Jan 1 = Sat
            || (d == 2 && m == 1 && w == Weekday::Monday)   // Jan 1 = Sun
            // Day after New Year's (Jan 2, adjusted)
            || (d == 2 && m == 1 && w != Weekday::Monday)
            || (d == 4 && m == 1 && w == Weekday::Monday)   // Jan 2 = Sat
            || (d == 3 && m == 1 && w == Weekday::Tuesday)  // Jan 2 = Sun → Tue (Jan 1 sub on Mon)
            // Waitangi Day (Feb 6, adjusted since 2014)
            || (d == 6 && m == 2)
            || (y >= 2014 && d == 7 && m == 2 && w == Weekday::Monday)  // Sun → Mon
            // Good Friday
            || (dd == em - 3)
            // Easter Monday
            || (dd == em)
            // Anzac Day (Apr 25, adjusted since 2014)
            || (d == 25 && m == 4)
            || (y >= 2014 && d == 26 && m == 4 && w == Weekday::Monday)  // Sun → Mon
            // Queen's Birthday (1st Monday in June)
            || (w == Weekday::Monday && m == 6 && (1..=7).contains(&d))
            // Matariki (simplified as Jun 24; since 2022)
            || (y >= 2022 && d == 24 && m == 6)
            // Labour Day (4th Monday in October)
            || (w == Weekday::Monday && m == 10 && (22..=28).contains(&d))
            // Christmas Day (adjusted)
            || (d == 25 && m == 12)
            || (d == 27 && m == 12 && (w == Weekday::Monday || w == Weekday::Tuesday))
            // Boxing Day (adjusted)
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
    fn waitangi_day_2023() {
        let cal = NewZealand;
        // Feb 6, 2023 is a Monday
        assert!(!cal.is_business_day(date(2023, 2, 6)));
    }

    #[test]
    fn good_friday_and_easter_monday_2023() {
        let cal = NewZealand;
        assert!(!cal.is_business_day(date(2023, 4, 7)));  // Good Friday
        assert!(!cal.is_business_day(date(2023, 4, 10))); // Easter Monday
    }

    #[test]
    fn queens_birthday_2023() {
        // 1st Monday in June 2023 = Jun 5
        let cal = NewZealand;
        assert!(!cal.is_business_day(date(2023, 6, 5)));
    }

    #[test]
    fn labour_day_2023() {
        // 4th Monday in October 2023 = Oct 23
        let cal = NewZealand;
        assert!(!cal.is_business_day(date(2023, 10, 23)));
    }

    #[test]
    fn normal_business_day() {
        let cal = NewZealand;
        // 2023-03-15 is a Wednesday
        assert!(cal.is_business_day(date(2023, 3, 15)));
    }
}
