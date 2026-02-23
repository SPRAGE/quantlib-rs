//! Czech Republic calendar.
//!
//! Translates `ql/time/calendars/czechrepublic.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// Czech Republic calendar.
///
/// Weekends and the following holidays are observed:
/// * New Year's Day (Jan 1)
/// * Good Friday (em-3, since 2016)
/// * Easter Monday
/// * Labour Day (May 1)
/// * Liberation Day (May 8)
/// * Saints Cyril & Methodius Day (Jul 5)
/// * Jan Hus Day (Jul 6)
/// * Czech Statehood Day (Sep 28)
/// * Independence Day (Oct 28)
/// * Freedom & Democracy Day (Nov 17)
/// * Christmas Eve (Dec 24)
/// * Christmas Day (Dec 25)
/// * St. Stephen's Day (Dec 26)
#[derive(Debug, Clone, Copy, Default)]
pub struct CzechRepublic;

impl Calendar for CzechRepublic {
    fn name(&self) -> &str {
        "Czech Republic"
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
            // Good Friday (since 2016)
            || (dd == em - 3 && y >= 2016)
            // Easter Monday
            || (dd == em)
            // Labour Day
            || (d == 1 && m == 5)
            // Liberation Day
            || (d == 8 && m == 5)
            // Saints Cyril & Methodius Day
            || (d == 5 && m == 7)
            // Jan Hus Day
            || (d == 6 && m == 7)
            // Czech Statehood Day
            || (d == 28 && m == 9)
            // Independence Day
            || (d == 28 && m == 10)
            // Freedom & Democracy Day
            || (d == 17 && m == 11)
            // Christmas Eve
            || (d == 24 && m == 12)
            // Christmas Day
            || (d == 25 && m == 12)
            // St. Stephen's Day
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
    fn new_years_day() {
        let cal = CzechRepublic;
        assert!(!cal.is_business_day(date(2023, 1, 1)));
    }

    #[test]
    fn good_friday_since_2016() {
        let cal = CzechRepublic;
        // Easter Monday 2023: April 10, Good Friday = April 7
        assert!(!cal.is_business_day(date(2023, 4, 7)));
        // Good Friday not observed before 2016
        // Easter Monday 2015: April 6, Good Friday = April 3
        assert!(cal.is_business_day(date(2015, 4, 3)));
    }

    #[test]
    fn jan_hus_day() {
        let cal = CzechRepublic;
        assert!(!cal.is_business_day(date(2023, 7, 6)));
    }

    #[test]
    fn christmas_eve() {
        let cal = CzechRepublic;
        assert!(!cal.is_business_day(date(2024, 12, 24)));
    }

    #[test]
    fn normal_business_day() {
        let cal = CzechRepublic;
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
