//! `DayCounter` trait and built-in day-count conventions (translates
//! `ql/time/daycounter.hpp` and `ql/time/daycounters/`).
//!
//! A day counter computes the **day count fraction** — the fraction of a year
//! between two dates — used when discounting or accruing interest.

use crate::date::Date;
use ql_core::{Real, Time};

/// A convention for counting the fraction of a year between two dates.
///
/// Corresponds to `QuantLib::DayCounter`.
pub trait DayCounter: std::fmt::Debug + Send + Sync {
    /// Human-readable name of this convention (e.g. `"Actual/365 (Fixed)"`).
    fn name(&self) -> &str;

    /// Number of days between `d1` and `d2` according to this convention.
    fn day_count(&self, d1: Date, d2: Date) -> i64;

    /// Fraction of a year between `d1` and `d2`.
    ///
    /// The default implementation divides [`day_count`][Self::day_count] by
    /// the denominator implied by the convention name.  Most implementations
    /// override this for correctness.
    fn year_fraction(&self, d1: Date, d2: Date) -> Time;

    /// Fraction of a year between `d1` and `d2` with reference period hints.
    ///
    /// Needed for some ISDA-style conventions.  Defaults to
    /// [`year_fraction`][Self::year_fraction].
    fn year_fraction_with_ref(
        &self,
        d1: Date,
        d2: Date,
        _ref_start: Date,
        _ref_end: Date,
    ) -> Time {
        self.year_fraction(d1, d2)
    }
}

/// Actual/365 (Fixed) day counter.
///
/// `year_fraction = actual_days / 365`
#[derive(Debug, Clone, Copy, Default)]
pub struct Actual365Fixed;

impl DayCounter for Actual365Fixed {
    fn name(&self) -> &str {
        "Actual/365 (Fixed)"
    }

    fn day_count(&self, d1: Date, d2: Date) -> i64 {
        (d2.serial() - d1.serial()) as i64
    }

    fn year_fraction(&self, d1: Date, d2: Date) -> Time {
        self.day_count(d1, d2) as Real / 365.0
    }
}

/// Actual/360 day counter.
///
/// `year_fraction = actual_days / 360`
#[derive(Debug, Clone, Copy, Default)]
pub struct Actual360;

impl DayCounter for Actual360 {
    fn name(&self) -> &str {
        "Actual/360"
    }

    fn day_count(&self, d1: Date, d2: Date) -> i64 {
        (d2.serial() - d1.serial()) as i64
    }

    fn year_fraction(&self, d1: Date, d2: Date) -> Time {
        self.day_count(d1, d2) as Real / 360.0
    }
}

/// Actual/365.25 day counter.
///
/// `year_fraction = actual_days / 365.25`
#[derive(Debug, Clone, Copy, Default)]
pub struct Actual36525;

impl DayCounter for Actual36525 {
    fn name(&self) -> &str {
        "Actual/365.25"
    }

    fn day_count(&self, d1: Date, d2: Date) -> i64 {
        (d2.serial() - d1.serial()) as i64
    }

    fn year_fraction(&self, d1: Date, d2: Date) -> Time {
        self.day_count(d1, d2) as Real / 365.25
    }
}

/// Business/252 day counter (used in Brazil).
///
/// Counts business days (Mon–Fri, ignoring holidays) and divides by 252.
#[derive(Debug, Clone, Copy, Default)]
pub struct Business252;

impl DayCounter for Business252 {
    fn name(&self) -> &str {
        "Business/252"
    }

    fn day_count(&self, d1: Date, d2: Date) -> i64 {
        // Simple Mon-Fri count (no holiday calendar)
        let mut count = 0i64;
        let mut d = d1;
        while d < d2 {
            if d.weekday().is_weekday() {
                count += 1;
            }
            d = d + 1;
        }
        count
    }

    fn year_fraction(&self, d1: Date, d2: Date) -> Time {
        self.day_count(d1, d2) as Real / 252.0
    }
}

/// Thirty/360 day counter (Bond Basis / US).
///
/// `year_fraction = [360(Y2−Y1) + 30(M2−M1) + (D2−D1)] / 360`
#[derive(Debug, Clone, Copy, Default)]
pub struct Thirty360;

impl DayCounter for Thirty360 {
    fn name(&self) -> &str {
        "30/360"
    }

    fn day_count(&self, d1: Date, d2: Date) -> i64 {
        let y1 = d1.year() as i64;
        let m1 = d1.month() as i64;
        let mut dd1 = d1.day_of_month() as i64;
        let y2 = d2.year() as i64;
        let m2 = d2.month() as i64;
        let mut dd2 = d2.day_of_month() as i64;

        if dd2 == 31 && dd1 < 30 {
            dd2 = 1;
        }
        if dd1 == 31 {
            dd1 = 30;
        }

        360 * (y2 - y1) + 30 * (m2 - m1) + (dd2 - dd1)
    }

    fn year_fraction(&self, d1: Date, d2: Date) -> Time {
        self.day_count(d1, d2) as Real / 360.0
    }
}

/// Actual/Actual (ISDA) day counter.
///
/// The year fraction accounts for leap years by splitting the period at
/// year boundaries.
#[derive(Debug, Clone, Copy, Default)]
pub struct ActualActualIsda;

impl DayCounter for ActualActualIsda {
    fn name(&self) -> &str {
        "Actual/Actual (ISDA)"
    }

    fn day_count(&self, d1: Date, d2: Date) -> i64 {
        (d2.serial() - d1.serial()) as i64
    }

    fn year_fraction(&self, d1: Date, d2: Date) -> Time {
        use crate::date::is_leap_year;
        if d1 == d2 {
            return 0.0;
        }
        let (y1, _, _) = (d1.year(), d1.month(), d1.day_of_month());
        let (y2, _, _) = (d2.year(), d2.month(), d2.day_of_month());
        if y1 == y2 {
            let days_in_year = if is_leap_year(y1) { 366.0 } else { 365.0 };
            return self.day_count(d1, d2) as Real / days_in_year;
        }
        // Split at year boundary
        let jan1_y2 = Date::from_ymd(y2, 1, 1).expect("valid date");
        let days_in_y1 = if is_leap_year(y1) { 366.0 } else { 365.0 };
        let part1 = (jan1_y2.serial() - d1.serial()) as Real / days_in_y1;
        // Recurse for rest
        part1 + self.year_fraction(jan1_y2, d2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: u16, m: u8, d: u8) -> Date {
        Date::from_ymd(y, m, d).unwrap()
    }

    #[test]
    fn actual365_fixed() {
        let dc = Actual365Fixed;
        let d1 = date(2023, 1, 1);
        let d2 = date(2024, 1, 1);
        assert_eq!(dc.day_count(d1, d2), 365);
        assert!((dc.year_fraction(d1, d2) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn actual360() {
        let dc = Actual360;
        let d1 = date(2023, 1, 1);
        let d2 = date(2023, 7, 1);
        let expected = (date(2023, 7, 1).serial() - date(2023, 1, 1).serial()) as f64 / 360.0;
        assert!((dc.year_fraction(d1, d2) - expected).abs() < 1e-12);
    }

    #[test]
    fn thirty360() {
        let dc = Thirty360;
        let d1 = date(2023, 1, 1);
        let d2 = date(2024, 1, 1);
        // 30/360: 360 * 1 = 360 days, year_fraction = 1.0
        assert_eq!(dc.day_count(d1, d2), 360);
        assert!((dc.year_fraction(d1, d2) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn actual_actual_isda_same_year() {
        let dc = ActualActualIsda;
        let d1 = date(2023, 1, 1);
        let d2 = date(2024, 1, 1);
        // 365/365 = 1.0
        assert!((dc.year_fraction(d1, d2) - 1.0).abs() < 1e-9);
    }
}
