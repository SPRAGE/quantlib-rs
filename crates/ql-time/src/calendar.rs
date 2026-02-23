//! `Calendar` trait and concrete calendar implementations.
//!
//! A calendar knows which dates are business days and can adjust dates
//! according to a [`BusinessDayConvention`].

use crate::business_day_convention::BusinessDayConvention;
use crate::date::Date;
use crate::weekday::Weekday;

/// A financial calendar.
///
/// Corresponds to `QuantLib::Calendar`.
pub trait Calendar: std::fmt::Debug + Send + Sync {
    /// Human-readable name (e.g. `"United States (NYSE)"`).
    fn name(&self) -> &str;

    /// Return `true` if `date` is a business day in this calendar.
    fn is_business_day(&self, date: Date) -> bool;

    /// Return `true` if `date` is a holiday (non-business) day.
    fn is_holiday(&self, date: Date) -> bool {
        !self.is_business_day(date)
    }

    /// Return `true` if `date` is a weekend according to this calendar.
    ///
    /// Most calendars consider Saturday and Sunday as weekends.
    fn is_weekend(&self, date: Date) -> bool {
        matches!(date.weekday(), Weekday::Saturday | Weekday::Sunday)
    }

    /// Return `true` if `date` is the last business day of its month.
    fn is_end_of_month(&self, date: Date) -> bool {
        let next = date + 1;
        date.month() != self.adjust(next, BusinessDayConvention::Following).month()
    }

    /// Return the last business day of the month containing `date`.
    fn end_of_month(&self, date: Date) -> Date {
        self.adjust(date.end_of_month(), BusinessDayConvention::Preceding)
    }

    /// Adjust `date` according to the given business-day convention.
    fn adjust(&self, mut date: Date, convention: BusinessDayConvention) -> Date {
        match convention {
            BusinessDayConvention::Unadjusted => date,
            BusinessDayConvention::Following => {
                while self.is_holiday(date) {
                    date = date + 1;
                }
                date
            }
            BusinessDayConvention::ModifiedFollowing => {
                let adjusted = self.adjust(date, BusinessDayConvention::Following);
                if adjusted.month() != date.month() {
                    self.adjust(date, BusinessDayConvention::Preceding)
                } else {
                    adjusted
                }
            }
            BusinessDayConvention::Preceding => {
                while self.is_holiday(date) {
                    date = date - 1;
                }
                date
            }
            BusinessDayConvention::ModifiedPreceding => {
                let adjusted = self.adjust(date, BusinessDayConvention::Preceding);
                if adjusted.month() != date.month() {
                    self.adjust(date, BusinessDayConvention::Following)
                } else {
                    adjusted
                }
            }
            BusinessDayConvention::Nearest => {
                if self.is_business_day(date) {
                    return date;
                }
                let fwd = self.adjust(date, BusinessDayConvention::Following);
                let bwd = self.adjust(date, BusinessDayConvention::Preceding);
                let days_fwd = (fwd.serial() - date.serial()).abs();
                let days_bwd = (date.serial() - bwd.serial()).abs();
                if days_fwd <= days_bwd {
                    fwd
                } else {
                    bwd
                }
            }
            BusinessDayConvention::EndOfMonth => {
                self.end_of_month(date)
            }
        }
    }

    /// Advance `date` by `n` business days.
    fn advance_business_days(&self, mut date: Date, n: i32) -> Date {
        let step: i32 = if n >= 0 { 1 } else { -1 };
        let mut remaining = n.abs();
        while remaining > 0 {
            date = date + step;
            if self.is_business_day(date) {
                remaining -= 1;
            }
        }
        date
    }

    /// Count the number of business days between `d1` (exclusive) and `d2`
    /// (inclusive).  Returns a negative number if `d2 < d1`.
    fn business_days_between(&self, d1: Date, d2: Date) -> i32 {
        if d1 == d2 {
            return 0;
        }
        let sign = if d2 > d1 { 1 } else { -1 };
        let (start, end) = if d2 > d1 { (d1, d2) } else { (d2, d1) };
        let mut count = 0;
        let mut d = start + 1;
        while d <= end {
            if self.is_business_day(d) {
                count += 1;
            }
            d = d + 1;
        }
        sign * count
    }
}

/// A null calendar — treats every day as a business day.
///
/// Equivalent to `QuantLib::NullCalendar`.
#[derive(Debug, Clone, Copy, Default)]
pub struct NullCalendar;

impl Calendar for NullCalendar {
    fn name(&self) -> &str {
        "Null"
    }

    fn is_business_day(&self, _date: Date) -> bool {
        true
    }

    fn is_weekend(&self, _date: Date) -> bool {
        false
    }
}

/// A calendar that treats only Saturdays and Sundays as non-business days,
/// with no additional holidays.
///
/// Useful as a base for country-specific calendars.
#[derive(Debug, Clone, Copy, Default)]
pub struct WeekendsOnly;

impl Calendar for WeekendsOnly {
    fn name(&self) -> &str {
        "Weekends Only"
    }

    fn is_business_day(&self, date: Date) -> bool {
        !self.is_weekend(date)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: u16, m: u8, d: u8) -> Date {
        Date::from_ymd(y, m, d).unwrap()
    }

    #[test]
    fn null_calendar_always_business() {
        let cal = NullCalendar;
        assert!(cal.is_business_day(date(2023, 12, 25)));
        assert!(cal.is_business_day(date(2023, 1, 1)));
    }

    #[test]
    fn weekends_only_saturday() {
        let cal = WeekendsOnly;
        // 2023-09-02 is a Saturday
        let sat = date(2023, 9, 2);
        assert!(!cal.is_business_day(sat));
        let mon = date(2023, 9, 4);
        assert!(cal.is_business_day(mon));
    }

    #[test]
    fn adjust_following() {
        let cal = WeekendsOnly;
        // 2023-09-02 is Saturday → next business day is Monday 2023-09-04
        let sat = date(2023, 9, 2);
        let adjusted = cal.adjust(sat, BusinessDayConvention::Following);
        assert_eq!(adjusted, date(2023, 9, 4));
    }

    #[test]
    fn adjust_preceding() {
        let cal = WeekendsOnly;
        let sat = date(2023, 9, 2);
        let adjusted = cal.adjust(sat, BusinessDayConvention::Preceding);
        assert_eq!(adjusted, date(2023, 9, 1)); // Friday
    }

    #[test]
    fn business_days_between() {
        let cal = WeekendsOnly;
        let d1 = date(2023, 9, 4); // Monday
        let d2 = date(2023, 9, 8); // Friday
        // Tue, Wed, Thu, Fri = 4 business days (d1 exclusive)
        assert_eq!(cal.business_days_between(d1, d2), 4);
    }
}
