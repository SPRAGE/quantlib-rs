//! TARGET (Trans-European Automated Real-time Gross Settlement) calendar.
//!
//! Translates `ql/time/calendars/target.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::{is_leap_year, Date};
use crate::weekday::Weekday;

/// TARGET calendar (ECB's settlement system).
///
/// Weekends and the following fixed holidays are observed:
/// * New Year's Day (Jan 1)
/// * Good Friday
/// * Easter Monday
/// * Labour Day (May 1)
/// * Christmas Day (Dec 25)
/// * Boxing Day (Dec 26)
///
/// Valid from 1 January 1999.
#[derive(Debug, Clone, Copy, Default)]
pub struct Target;

impl Calendar for Target {
    fn name(&self) -> &str {
        "TARGET"
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
        let em = easter_monday_pub(y);

        if
        // New Year's Day
        (d == 1 && m == 1)
            // Good Friday (from 2000)
            || (dd == em - 3 && y >= 2000)
            // Easter Monday (from 2000)
            || (dd == em && y >= 2000)
            // Labour Day (from 2000)
            || (d == 1 && m == 5 && y >= 2000)
            // Christmas
            || (d == 25 && m == 12)
            // Boxing Day
            || (d == 26 && m == 12)
            // December 31, 1998, 1999, and 2001
            || (d == 31 && m == 12 && (y == 1998 || y == 1999 || y == 2001))
        {
            return false;
        }
        true
    }
}

/// Compute the day-of-year (1-based) for Easter Monday in `year`.
///
/// Uses the Anonymous Gregorian algorithm.  Exposed for use by other calendar
/// modules in this crate.
pub(crate) fn easter_monday_pub(year: u16) -> u16 {
    let y = year as i32;
    // Oudin's algorithm for Easter Sunday (requires signed arithmetic)
    let g = y % 19;
    let c = y / 100;
    let h = (c - c / 4 - (8 * c + 13) / 25 + 19 * g + 15) % 30;
    let i = h - (h / 28) * (1 - (h / 28) * (29 / (h + 1)) * ((21 - g) / 11));
    let j = (y + y / 4 + i + 2 - c + c / 4) % 7;
    let p = i - j;
    let e_day = 1 + (p + 27 + (p + 6) / 40) % 31;
    let e_month = 3 + (p + 26) / 30;
    // Easter Sunday day-of-year
    let mut doy = e_day as u16;
    for mon in 1..e_month {
        doy += days_in_month_u16(year, mon as u8);
    }
    doy + 1 // Easter Monday = Easter Sunday + 1
}

fn days_in_month_u16(year: u16, month: u8) -> u16 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
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
        let cal = Target;
        assert!(!cal.is_business_day(date(2023, 1, 1)));
    }

    #[test]
    fn christmas() {
        let cal = Target;
        assert!(!cal.is_business_day(date(2023, 12, 25)));
        assert!(!cal.is_business_day(date(2023, 12, 26)));
    }

    #[test]
    fn easter_2023() {
        // Easter Sunday 2023: April 9 → Good Friday April 7, Easter Monday April 10
        let cal = Target;
        assert!(!cal.is_business_day(date(2023, 4, 7))); // Good Friday
        assert!(!cal.is_business_day(date(2023, 4, 10))); // Easter Monday
        assert!(cal.is_business_day(date(2023, 4, 11))); // Tuesday after Easter
    }

    #[test]
    fn labour_day() {
        let cal = Target;
        assert!(!cal.is_business_day(date(2023, 5, 1)));
    }

    #[test]
    fn normal_business_day() {
        let cal = Target;
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
