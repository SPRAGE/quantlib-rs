//! India (NSE — National Stock Exchange) calendar.
//!
//! Translates `ql/time/calendars/india.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// India (NSE) calendar.
///
/// Weekends and the following holidays are observed:
/// * Republic Day (Jan 26)
/// * Good Friday (em-3)
/// * Dr. Ambedkar Jayanti (Apr 14)
/// * Labour Day (May 1)
/// * Independence Day (Aug 15)
/// * Gandhi Jayanti (Oct 2)
/// * Christmas Day (Dec 25)
///
/// Note: Many Indian holidays (Holi, Diwali, Eid, etc.) depend on lunar or
/// regional calendars and vary each year. Only fixed and Easter-derived
/// holidays are implemented here.
#[derive(Debug, Clone, Copy, Default)]
pub struct India;

impl Calendar for India {
    fn name(&self) -> &str {
        "India (NSE)"
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
        // Republic Day
        (d == 26 && m == 1)
            // Good Friday
            || (dd == em - 3)
            // Dr. Ambedkar Jayanti
            || (d == 14 && m == 4)
            // Labour Day
            || (d == 1 && m == 5)
            // Independence Day
            || (d == 15 && m == 8)
            // Gandhi Jayanti
            || (d == 2 && m == 10)
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
    fn republic_day() {
        let cal = India;
        // Jan 26, 2023 is a Thursday
        assert!(!cal.is_business_day(date(2023, 1, 26)));
    }

    #[test]
    fn good_friday_2023() {
        let cal = India;
        // Easter Monday 2023: April 10 → Good Friday April 7
        assert!(!cal.is_business_day(date(2023, 4, 7)));
    }

    #[test]
    fn independence_day() {
        let cal = India;
        // Aug 15, 2023 is a Tuesday
        assert!(!cal.is_business_day(date(2023, 8, 15)));
    }

    #[test]
    fn normal_business_day() {
        let cal = India;
        // 2023-06-15 is a Thursday
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
