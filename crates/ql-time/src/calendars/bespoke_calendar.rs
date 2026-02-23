//! Bespoke calendar — a calendar with user-defined holidays.
//!
//! Corresponds to `QuantLib::BespokeCalendar`.

use crate::calendar::Calendar;
use crate::date::Date;
use std::collections::HashSet;

/// A calendar where holidays are added manually at run time.
///
/// Corresponds to `QuantLib::BespokeCalendar`.
#[derive(Debug, Clone)]
pub struct BespokeCalendar {
    name: String,
    holidays: HashSet<i32>,
}

impl BespokeCalendar {
    /// Create a new bespoke calendar with the given name and no holidays.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            holidays: HashSet::new(),
        }
    }

    /// Add a holiday. Weekends are already non-business days.
    pub fn add_holiday(&mut self, date: Date) {
        self.holidays.insert(date.serial());
    }

    /// Remove a previously added holiday.
    pub fn remove_holiday(&mut self, date: Date) {
        self.holidays.remove(&date.serial());
    }

    /// Return the number of explicitly-added holidays.
    pub fn holiday_count(&self) -> usize {
        self.holidays.len()
    }
}

impl Calendar for BespokeCalendar {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_business_day(&self, date: Date) -> bool {
        !self.is_weekend(date) && !self.holidays.contains(&date.serial())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: u16, m: u8, d: u8) -> Date {
        Date::from_ymd(y, m, d).unwrap()
    }

    #[test]
    fn empty_bespoke_is_weekends_only() {
        let cal = BespokeCalendar::new("Test");
        assert_eq!(cal.name(), "Test");
        // Monday is a business day
        assert!(cal.is_business_day(date(2024, 1, 8)));
        // Saturday is not
        assert!(!cal.is_business_day(date(2024, 1, 6)));
    }

    #[test]
    fn add_and_remove_holiday() {
        let mut cal = BespokeCalendar::new("Custom");
        let holiday = date(2024, 3, 15); // Friday
        assert!(cal.is_business_day(holiday));

        cal.add_holiday(holiday);
        assert!(!cal.is_business_day(holiday));
        assert_eq!(cal.holiday_count(), 1);

        cal.remove_holiday(holiday);
        assert!(cal.is_business_day(holiday));
        assert_eq!(cal.holiday_count(), 0);
    }

    #[test]
    fn multiple_holidays() {
        let mut cal = BespokeCalendar::new("Multi");
        cal.add_holiday(date(2024, 12, 25)); // Christmas (Wednesday)
        cal.add_holiday(date(2024, 12, 26)); // Boxing Day (Thursday)
        cal.add_holiday(date(2025, 1, 1)); // New Year's Day (Wednesday)

        assert!(!cal.is_business_day(date(2024, 12, 25)));
        assert!(!cal.is_business_day(date(2024, 12, 26)));
        assert!(!cal.is_business_day(date(2025, 1, 1)));
        // Regular weekday not added as holiday
        assert!(cal.is_business_day(date(2024, 12, 24)));
        assert_eq!(cal.holiday_count(), 3);
    }
}
