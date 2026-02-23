//! Japan calendar.
//!
//! Translates `ql/time/calendars/japan.hpp` / `.cpp`.

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// Japan calendar.
///
/// Weekends and the following holidays are observed:
/// * New Year's Day (Jan 1–3)
/// * Coming of Age Day (2nd Monday in January)
/// * National Foundation Day (Feb 11)
/// * Emperor's Birthday (Feb 23 since 2020; Dec 23 before 2019)
/// * Vernal Equinox Day (approx Mar 20–21)
/// * Showa Day (Apr 29)
/// * Constitution Memorial Day (May 3)
/// * Greenery Day (May 4)
/// * Children's Day (May 5)
/// * Marine Day (3rd Monday in July)
/// * Mountain Day (Aug 11, since 2016)
/// * Respect for the Aged Day (3rd Monday in September)
/// * Autumnal Equinox Day (approx Sep 22–23)
/// * Sports Day (2nd Monday in October)
/// * Culture Day (Nov 3)
/// * Labour Thanksgiving Day (Nov 23)
///
/// If a holiday falls on a Sunday, the following Monday is observed as a
/// substitute holiday (*furikae kyūjitsu*).
#[derive(Debug, Clone, Copy, Default)]
pub struct Japan;

/// Return an approximate Vernal Equinox day-of-month for March.
/// This uses a simplified formula; the actual date is proclaimed annually.
fn vernal_equinox_day(year: u16) -> u8 {
    // 20.8431 + 0.242194*(y-1980) - floor((y-1980)/4)
    let y = year as f64;
    let d = 20.8431 + 0.242194 * (y - 1980.0) - ((y - 1980.0) / 4.0).floor();
    d as u8
}

/// Return an approximate Autumnal Equinox day-of-month for September.
fn autumnal_equinox_day(year: u16) -> u8 {
    let y = year as f64;
    let d = 23.2488 + 0.242194 * (y - 1980.0) - ((y - 1980.0) / 4.0).floor();
    d as u8
}

impl Calendar for Japan {
    fn name(&self) -> &str {
        "Japan"
    }

    fn is_business_day(&self, date: Date) -> bool {
        let w = date.weekday();
        if matches!(w, Weekday::Saturday | Weekday::Sunday) {
            return false;
        }
        let y = date.year();
        let m = date.month();
        let d = date.day_of_month();

        let ve = vernal_equinox_day(y);
        let ae = autumnal_equinox_day(y);

        // Helper: check if a given (month, day) is a holiday (before substitute logic).
        // We'll check the date itself, and also if it's a Monday substitute for a
        // Sunday holiday.
        let is_fixed_holiday = |mon: u8, dom: u8| -> bool {
            (m == mon && d == dom)
                // Substitute holiday: if the holiday falls on Sunday, Monday is off
                || (m == mon && d == dom + 1 && w == Weekday::Monday)
        };

        // New Year's Day (Jan 1–3)
        if (m == 1 && (1..=3).contains(&d))
            // Substitute: Jan 4 if Jan 3 was Sunday
            || (m == 1 && d == 4 && w == Weekday::Monday)
        {
            return false;
        }
        // Coming of Age Day (2nd Monday in January)
        if w == Weekday::Monday && m == 1 && (8..=14).contains(&d) {
            return false;
        }
        // National Foundation Day (Feb 11)
        if is_fixed_holiday(2, 11) {
            return false;
        }
        // Emperor's Birthday
        if y >= 2020 && is_fixed_holiday(2, 23) {
            return false;
        }
        if y <= 2018 && is_fixed_holiday(12, 23) {
            return false;
        }
        // Vernal Equinox Day (March)
        if is_fixed_holiday(3, ve) {
            return false;
        }
        // Showa Day (Apr 29)
        if is_fixed_holiday(4, 29) {
            return false;
        }
        // Constitution Memorial Day (May 3)
        if is_fixed_holiday(5, 3) {
            return false;
        }
        // Greenery Day (May 4)
        if is_fixed_holiday(5, 4) {
            return false;
        }
        // Children's Day (May 5)
        if is_fixed_holiday(5, 5) {
            return false;
        }
        // Make-up day for May holidays: if May 3 is Sunday → May 6 off,
        // if May 4 is Sunday → May 6 off
        if m == 5 && d == 6 && matches!(w, Weekday::Tuesday | Weekday::Wednesday) {
            return false;
        }
        // Marine Day (3rd Monday in July)
        if w == Weekday::Monday && m == 7 && (15..=21).contains(&d) {
            // 2020 exception: moved to Jul 23 for Olympics
            if y != 2020 {
                return false;
            }
        }
        // 2020 Olympic special dates
        if y == 2020 && m == 7 && d == 23 {
            return false;
        }
        if y == 2020 && m == 7 && d == 24 {
            return false;
        }
        // Mountain Day (Aug 11, since 2016)
        if y >= 2016 && is_fixed_holiday(8, 11) {
            // 2020 exception: moved to Aug 10
            if y != 2020 {
                return false;
            }
        }
        if y == 2020 && m == 8 && d == 10 {
            return false;
        }
        // Respect for the Aged Day (3rd Monday in September)
        if w == Weekday::Monday && m == 9 && (15..=21).contains(&d) {
            return false;
        }
        // Autumnal Equinox Day (September)
        if is_fixed_holiday(9, ae) {
            return false;
        }
        // Citizen's Holiday: if a day is sandwiched between two holidays
        // (Respect for Aged and Autumnal Equinox can be 1 day apart)
        if m == 9 && d == ae.saturating_sub(1) && d >= 16 && w == Weekday::Tuesday {
            // Tuesday between Monday (Respect) and Wed (Equinox)
            return false;
        }
        // Sports Day (2nd Monday in October; Health and Sports Day before 2000)
        if w == Weekday::Monday && m == 10 && (8..=14).contains(&d) {
            return false;
        }
        // Culture Day (Nov 3)
        if is_fixed_holiday(11, 3) {
            return false;
        }
        // Labour Thanksgiving Day (Nov 23)
        if is_fixed_holiday(11, 23) {
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
    fn new_years_2023() {
        let cal = Japan;
        assert!(!cal.is_business_day(date(2023, 1, 1)));
        assert!(!cal.is_business_day(date(2023, 1, 2)));
        assert!(!cal.is_business_day(date(2023, 1, 3)));
    }

    #[test]
    fn coming_of_age_2023() {
        // 2nd Monday in January 2023 = Jan 9
        let cal = Japan;
        assert!(!cal.is_business_day(date(2023, 1, 9)));
    }

    #[test]
    fn emperor_birthday_2023() {
        // Feb 23, 2023 is a Thursday
        let cal = Japan;
        assert!(!cal.is_business_day(date(2023, 2, 23)));
    }

    #[test]
    fn showa_day_2023() {
        // Apr 29, 2023 is a Saturday (weekend)
        let cal = Japan;
        assert!(!cal.is_business_day(date(2023, 4, 29)));
    }

    #[test]
    fn culture_day_2023() {
        // Nov 3, 2023 is a Friday
        let cal = Japan;
        assert!(!cal.is_business_day(date(2023, 11, 3)));
    }

    #[test]
    fn normal_business_day() {
        let cal = Japan;
        // 2023-06-15 is a Thursday
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
