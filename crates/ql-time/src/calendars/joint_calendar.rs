//! Joint calendar — combines two or more calendars.
//!
//! Corresponds to `QuantLib::JointCalendar`.

use crate::calendar::Calendar;
use crate::date::Date;

/// Rule for combining multiple calendars.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JointCalendarRule {
    /// A day is a holiday if it is a holiday in **any** of the constituent
    /// calendars (i.e. the union of holiday sets — the intersection of
    /// business-day sets).
    JoinHolidays,
    /// A day is a business day if it is a business day in **any** of the
    /// constituent calendars (i.e. the intersection of holiday sets — the
    /// union of business-day sets).
    JoinBusinessDays,
}

/// A calendar that combines multiple calendars according to a
/// [`JointCalendarRule`].
///
/// Corresponds to `QuantLib::JointCalendar`.
pub struct JointCalendar {
    calendars: Vec<Box<dyn Calendar>>,
    rule: JointCalendarRule,
    name: String,
}

impl std::fmt::Debug for JointCalendar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JointCalendar")
            .field("name", &self.name)
            .field("rule", &self.rule)
            .finish()
    }
}

impl JointCalendar {
    /// Create a new joint calendar from a list of calendars and a combination
    /// rule.
    ///
    /// # Panics
    /// Panics if `calendars` is empty.
    pub fn new(calendars: Vec<Box<dyn Calendar>>, rule: JointCalendarRule) -> Self {
        assert!(!calendars.is_empty(), "JointCalendar requires at least one calendar");
        let names: Vec<&str> = calendars.iter().map(|c| c.name()).collect();
        let joiner = match rule {
            JointCalendarRule::JoinHolidays => ", ",
            JointCalendarRule::JoinBusinessDays => " | ",
        };
        let name = names.join(joiner);
        Self {
            calendars,
            rule,
            name,
        }
    }
}

impl Calendar for JointCalendar {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_business_day(&self, date: Date) -> bool {
        match self.rule {
            JointCalendarRule::JoinHolidays => {
                // A day is a business day only if ALL calendars say it is
                self.calendars.iter().all(|c| c.is_business_day(date))
            }
            JointCalendarRule::JoinBusinessDays => {
                // A day is a business day if ANY calendar says it is
                self.calendars.iter().any(|c| c.is_business_day(date))
            }
        }
    }

    fn is_weekend(&self, date: Date) -> bool {
        match self.rule {
            JointCalendarRule::JoinHolidays => {
                // Weekend if ANY calendar considers it a weekend
                self.calendars.iter().any(|c| c.is_weekend(date))
            }
            JointCalendarRule::JoinBusinessDays => {
                // Weekend only if ALL calendars consider it a weekend
                self.calendars.iter().all(|c| c.is_weekend(date))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::{NullCalendar, WeekendsOnly};
    use crate::calendars::target::Target;

    fn date(y: u16, m: u8, d: u8) -> Date {
        Date::from_ymd(y, m, d).unwrap()
    }

    #[test]
    fn join_holidays_both_must_agree() {
        // JoinHolidays: business day only if ALL calendars agree
        let cal = JointCalendar::new(
            vec![Box::new(WeekendsOnly), Box::new(Target)],
            JointCalendarRule::JoinHolidays,
        );
        // 2024-01-01 is a TARGET holiday (New Year's Day) and a Monday
        let new_year = date(2024, 1, 1);
        assert!(!cal.is_business_day(new_year));
        // 2024-01-02 is a normal Tuesday — business day in both
        let jan2 = date(2024, 1, 2);
        assert!(cal.is_business_day(jan2));
    }

    #[test]
    fn join_business_days_any_suffices() {
        // JoinBusinessDays: business day if ANY calendar says so
        let cal = JointCalendar::new(
            vec![Box::new(NullCalendar), Box::new(Target)],
            JointCalendarRule::JoinBusinessDays,
        );
        // NullCalendar considers every day a business day, so the joint
        // calendar should always be a business day
        let new_year = date(2024, 1, 1);
        assert!(cal.is_business_day(new_year));
        let sat = date(2024, 1, 6); // Saturday
        assert!(cal.is_business_day(sat));
    }

    #[test]
    fn name_formatting() {
        let cal_holidays = JointCalendar::new(
            vec![Box::new(WeekendsOnly), Box::new(Target)],
            JointCalendarRule::JoinHolidays,
        );
        assert_eq!(cal_holidays.name(), "Weekends Only, TARGET");

        let cal_biz = JointCalendar::new(
            vec![Box::new(WeekendsOnly), Box::new(Target)],
            JointCalendarRule::JoinBusinessDays,
        );
        assert_eq!(cal_biz.name(), "Weekends Only | TARGET");
    }

    #[test]
    fn join_holidays_weekends() {
        // WeekendsOnly has Sat/Sun weekends
        // If we join two WeekendsOnly calendars, weekends should remain Sat/Sun
        let cal = JointCalendar::new(
            vec![Box::new(WeekendsOnly), Box::new(WeekendsOnly)],
            JointCalendarRule::JoinHolidays,
        );
        let sat = date(2024, 1, 6);
        assert!(cal.is_weekend(sat));
        let mon = date(2024, 1, 8);
        assert!(!cal.is_weekend(mon));
    }
}
