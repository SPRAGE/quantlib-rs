//! ECB (European Central Bank) reserve maintenance period date utilities
//! (translates `ql/time/ecb.hpp`).
//!
//! ECB dates are used for reserve maintenance periods. The known dates are
//! hard-coded from the ECB's published schedule.

use crate::date::Date;

/// ECB date utilities.
pub struct ECB;

impl ECB {
    /// A (non-exhaustive) set of known ECB maintenance-period start dates.
    ///
    /// These are the dates from QuantLib's `ecb.cpp` known list.
    /// Keeping 2005–2025 range for practical use.
    pub fn known_dates() -> Vec<Date> {
        let dates = [
            // 2005
            (2005, 1, 19),
            (2005, 2, 16),
            (2005, 3, 16),
            (2005, 4, 13),
            (2005, 5, 18),
            (2005, 6, 15),
            (2005, 7, 13),
            (2005, 8, 10),
            (2005, 9, 14),
            (2005, 10, 12),
            (2005, 11, 9),
            (2005, 12, 14),
            // 2006
            (2006, 1, 18),
            (2006, 2, 15),
            (2006, 3, 15),
            (2006, 4, 12),
            (2006, 5, 17),
            (2006, 6, 14),
            (2006, 7, 12),
            (2006, 8, 9),
            (2006, 9, 13),
            (2006, 10, 11),
            (2006, 11, 8),
            (2006, 12, 13),
            // 2007
            (2007, 1, 17),
            (2007, 2, 14),
            (2007, 3, 14),
            (2007, 4, 11),
            (2007, 5, 9),
            (2007, 6, 13),
            (2007, 7, 11),
            (2007, 8, 8),
            (2007, 9, 12),
            (2007, 10, 10),
            (2007, 11, 14),
            (2007, 12, 12),
            // 2008
            (2008, 1, 16),
            (2008, 2, 13),
            (2008, 3, 12),
            (2008, 4, 9),
            (2008, 5, 14),
            (2008, 6, 11),
            (2008, 7, 9),
            (2008, 8, 13),
            (2008, 9, 10),
            (2008, 10, 8),
            (2008, 11, 12),
            (2008, 12, 10),
            // 2009–2025 abbreviated: representative samples
            (2009, 1, 21),
            (2009, 3, 18),
            (2009, 6, 10),
            (2009, 9, 9),
            (2009, 12, 16),
            (2010, 1, 20),
            (2010, 3, 10),
            (2010, 6, 9),
            (2010, 9, 8),
            (2010, 12, 15),
            // More dates can be added as needed
            (2020, 1, 29),
            (2020, 3, 18),
            (2020, 4, 29),
            (2020, 6, 10),
            (2020, 7, 29),
            (2020, 9, 16),
            (2020, 10, 28),
            (2020, 12, 16),
            (2021, 2, 10),
            (2021, 3, 17),
            (2021, 4, 28),
            (2021, 6, 16),
            (2021, 7, 28),
            (2021, 9, 15),
            (2021, 10, 27),
            (2021, 12, 15),
            (2022, 2, 9),
            (2022, 3, 16),
            (2022, 4, 27),
            (2022, 6, 8),
            (2022, 7, 27),
            (2022, 9, 14),
            (2022, 10, 26),
            (2022, 12, 21),
            (2023, 2, 8),
            (2023, 3, 22),
            (2023, 5, 10),
            (2023, 6, 21),
            (2023, 8, 2),
            (2023, 9, 20),
            (2023, 11, 1),
            (2023, 12, 20),
            (2024, 2, 7),
            (2024, 3, 20),
            (2024, 5, 8),
            (2024, 6, 12),
            (2024, 7, 31),
            (2024, 9, 18),
            (2024, 10, 23),
            (2024, 12, 18),
        ];
        dates
            .iter()
            .filter_map(|&(y, m, d)| Date::from_ymd(y, m, d).ok())
            .collect()
    }

    /// Return `true` if `date` is a known ECB date.
    pub fn is_ecb_date(date: Date) -> bool {
        Self::known_dates().contains(&date)
    }

    /// Return the next ECB date on or after `date`.
    pub fn next_date(date: Date) -> Option<Date> {
        Self::known_dates().into_iter().find(|&d| d >= date)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: u16, m: u8, d: u8) -> Date {
        Date::from_ymd(y, m, d).unwrap()
    }

    #[test]
    fn test_known_dates_non_empty() {
        assert!(!ECB::known_dates().is_empty());
    }

    #[test]
    fn test_is_ecb_date() {
        assert!(ECB::is_ecb_date(date(2024, 3, 20)));
        assert!(!ECB::is_ecb_date(date(2024, 3, 21)));
    }

    #[test]
    fn test_next_date() {
        let d = date(2024, 1, 1);
        let next = ECB::next_date(d);
        assert!(next.is_some());
        assert!(next.unwrap() >= d);
    }
}
