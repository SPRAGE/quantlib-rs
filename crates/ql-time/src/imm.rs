//! IMM (International Monetary Market) date utilities
//! (translates `ql/time/imm.hpp`).
//!
//! IMM dates are the third Wednesday of March, June, September, and December.

use crate::date::Date;
use crate::weekday::Weekday;

/// IMM date utilities.
pub struct IMM;

impl IMM {
    /// Return `true` if `date` is an IMM date (3rd Wednesday of Mar/Jun/Sep/Dec).
    pub fn is_imm_date(date: Date) -> bool {
        let m = date.month();
        let d = date.day_of_month();
        let w = date.weekday();
        matches!(m, 3 | 6 | 9 | 12) && w == Weekday::Wednesday && (15..=21).contains(&d)
    }

    /// Return the next IMM date on or after `date`.
    pub fn next_date(date: Date) -> Date {
        let mut y = date.year();
        let mut m = date.month();

        // Find the next IMM month
        let imm_month = match m {
            1..=3 => 3,
            4..=6 => 6,
            7..=9 => 9,
            10..=12 => 12,
            _ => unreachable!(),
        };

        let candidate = Self::imm_date_for_month(y, imm_month);
        if candidate >= date {
            return candidate;
        }

        // Move to next IMM month
        m = imm_month + 3;
        if m > 12 {
            m = 3;
            y += 1;
        }
        Self::imm_date_for_month(y, m)
    }

    /// Return the next IMM date strictly after `date`.
    pub fn next_date_after(date: Date) -> Date {
        let next = Self::next_date(date);
        if next == date {
            Self::next_date(date + 1)
        } else {
            next
        }
    }

    /// Return the IMM code for the given date (e.g. `"H5"` for March 2025).
    ///
    /// Returns `None` if the date is not an IMM date.
    pub fn code(date: Date) -> Option<String> {
        if !Self::is_imm_date(date) {
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

    /// Return the IMM date (3rd Wednesday) for the given year and IMM month.
    fn imm_date_for_month(year: u16, month: u8) -> Date {
        // Find the first day of the month
        let first = Date::from_ymd(year, month, 1).expect("valid IMM month");
        let first_weekday = first.weekday();

        // Days until Wednesday: Wednesday = ordinal 3
        let days_to_wed = (3i32 - first_weekday.ordinal() as i32).rem_euclid(7);
        // First Wednesday
        let first_wed = first + days_to_wed;
        // Third Wednesday = first + 14
        first_wed + 14
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: u16, m: u8, d: u8) -> Date {
        Date::from_ymd(y, m, d).unwrap()
    }

    #[test]
    fn test_is_imm_date() {
        // 3rd Wednesday of March 2024 = March 20
        assert!(IMM::is_imm_date(date(2024, 3, 20)));
        // Not an IMM date
        assert!(!IMM::is_imm_date(date(2024, 3, 21)));
        // Not an IMM month
        assert!(!IMM::is_imm_date(date(2024, 4, 17)));
    }

    #[test]
    fn test_next_date() {
        let d = date(2024, 1, 1);
        let next = IMM::next_date(d);
        assert!(IMM::is_imm_date(next));
        assert!(next >= d);
        assert_eq!(next.month(), 3);
    }

    #[test]
    fn test_code() {
        let d = date(2024, 3, 20); // 3rd Wed of March 2024
        assert_eq!(IMM::code(d), Some("H4".to_string()));

        let d2 = date(2024, 6, 19); // 3rd Wed of June 2024
        assert_eq!(IMM::code(d2), Some("M4".to_string()));
    }
}
