//! ASX (Australian Securities Exchange) date utilities
//! (translates `ql/time/asx.hpp`).
//!
//! ASX dates are the second Friday of March, June, September, and December.

use crate::date::Date;
use crate::weekday::Weekday;

/// ASX date utilities.
pub struct ASX;

impl ASX {
    /// Return `true` if `date` is an ASX date (2nd Friday of Mar/Jun/Sep/Dec).
    pub fn is_asx_date(date: Date) -> bool {
        let m = date.month();
        let d = date.day_of_month();
        let w = date.weekday();
        matches!(m, 3 | 6 | 9 | 12)
            && w == Weekday::Friday
            && (8..=14).contains(&d)
    }

    /// Return the next ASX date on or after `date`.
    pub fn next_date(date: Date) -> Date {
        let mut y = date.year();
        let imm_month = match date.month() {
            1..=3 => 3,
            4..=6 => 6,
            7..=9 => 9,
            10..=12 => 12,
            _ => unreachable!(),
        };

        let candidate = Self::asx_date_for_month(y, imm_month);
        if candidate >= date {
            return candidate;
        }

        let mut m = imm_month + 3;
        if m > 12 {
            m = 3;
            y += 1;
        }
        Self::asx_date_for_month(y, m)
    }

    /// Return the ASX code for the given date (e.g. `"H5"` for March 2025).
    ///
    /// Returns `None` if the date is not an ASX date.
    pub fn code(date: Date) -> Option<String> {
        if !Self::is_asx_date(date) {
            return None;
        }
        let m = date.month();
        let y = date.year() % 10;
        let month_code = match m {
            3 => 'H',
            6 => 'M',
            9 => 'U',
            12 => 'Z',
            _ => return None,
        };
        Some(format!("{month_code}{y}"))
    }

    /// Return the ASX date (2nd Friday) for the given year and ASX month.
    fn asx_date_for_month(year: u16, month: u8) -> Date {
        let first = Date::from_ymd(year, month, 1).expect("valid ASX month");
        let first_weekday = first.weekday();
        // Days until Friday: Friday = ordinal 5
        let days_to_fri = (5i32 - first_weekday.ordinal() as i32).rem_euclid(7);
        let first_fri = first + days_to_fri;
        // Second Friday = first + 7
        first_fri + 7
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: u16, m: u8, d: u8) -> Date {
        Date::from_ymd(y, m, d).unwrap()
    }

    #[test]
    fn test_is_asx_date() {
        // 2nd Friday of March 2024 = March 8
        assert!(ASX::is_asx_date(date(2024, 3, 8)));
        assert!(!ASX::is_asx_date(date(2024, 3, 15)));
    }

    #[test]
    fn test_next_date() {
        let d = date(2024, 1, 1);
        let next = ASX::next_date(d);
        assert!(ASX::is_asx_date(next));
        assert!(next >= d);
    }
}
