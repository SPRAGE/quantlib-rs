//! `Date` type (translates `ql/time/date.hpp`).
//!
//! QuantLib represents dates as a serial number of days since an epoch.
//! The epoch in QuantLib is **December 31, 1899** (serial = 0 corresponds to
//! Jan 1 1900).
//!
//! # Serial number convention
//! * Serial 0 is used as the "null date" sentinel.
//! * Serial 1 = January 1, 1900.
//! * The valid date range is 1900-01-01 to 2199-12-31.

use crate::time_unit::TimeUnit;
use crate::weekday::Weekday;
use ql_core::errors::{Error, Result};

/// A calendar date represented as a serial number.
///
/// Corresponds to `QuantLib::Date`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Date(i32);

// ── Constants ─────────────────────────────────────────────────────────────────

impl Date {
    /// The null date sentinel (serial 0).
    pub const NULL: Date = Date(0);

    /// Minimum valid date: January 1, 1901.
    pub const MIN: Date = Date(367);

    /// Maximum valid date: December 31, 2199.
    pub const MAX: Date = Date(109_573);

    // ── Constructors ─────────────────────────────────────────────────────────

    /// Create a date from a serial number.
    ///
    /// Returns an error if `serial <= 0` (which would be the null sentinel or
    /// before the epoch) or out of range.
    pub fn from_serial(serial: i32) -> Result<Self> {
        if serial <= 0 {
            return Err(Error::Date("serial number must be positive".into()));
        }
        let d = Date(serial);
        if d > Self::MAX {
            return Err(Error::Date(format!("serial {serial} exceeds maximum date")));
        }
        Ok(d)
    }

    /// Create a date from year, month (1–12), and day-of-month (1–31).
    pub fn from_ymd(year: u16, month: u8, day: u8) -> Result<Self> {
        if !(1900..=2199).contains(&year) {
            return Err(Error::Date(format!(
                "year {year} out of range [1900, 2199]"
            )));
        }
        if !(1..=12).contains(&month) {
            return Err(Error::Date(format!("month {month} out of range [1, 12]")));
        }
        let days_in = days_in_month(year, month);
        if day == 0 || day > days_in {
            return Err(Error::Date(format!(
                "day {day} out of range [1, {days_in}] for {year}-{month:02}"
            )));
        }
        Ok(Date(serial_from_ymd(year, month, day)))
    }

    /// Create a date from an (unchecked) serial number.
    #[allow(dead_code)]
    pub(crate) fn from_serial_unchecked(serial: i32) -> Self {
        debug_assert!(
            serial > 0 && Date(serial) <= Self::MAX,
            "invalid date serial {serial}"
        );
        Date(serial)
    }

    // ── Accessors ─────────────────────────────────────────────────────────────

    /// Return the serial number.
    pub fn serial(&self) -> i32 {
        self.0
    }

    /// Return `true` if this is the null date sentinel.
    pub fn is_null(&self) -> bool {
        self.0 == 0
    }

    /// Return the year (1900–2199).
    pub fn year(&self) -> u16 {
        ymd_from_serial(self.0).0
    }

    /// Return the month (1–12).
    pub fn month(&self) -> u8 {
        ymd_from_serial(self.0).1
    }

    /// Return the day of the month (1–31).
    pub fn day_of_month(&self) -> u8 {
        ymd_from_serial(self.0).2
    }

    /// Return the day of the year (1–366).
    pub fn day_of_year(&self) -> u16 {
        let (y, m, d) = ymd_from_serial(self.0);
        let mut doy = d as u16;
        for mon in 1..m {
            doy += days_in_month(y, mon) as u16;
        }
        doy
    }

    /// Return the weekday.
    pub fn weekday(&self) -> Weekday {
        // QuantLib epoch Jan 1, 1900 is a Monday (ordinal 1).
        // serial 1 → Monday, serial 2 → Tuesday, …
        let w = ((self.0 - 1).rem_euclid(7) + 1) as u8;
        Weekday::from_ordinal(w).expect("rem_euclid always in 1..=7")
    }

    // ── Arithmetic ────────────────────────────────────────────────────────────

    /// Advance by `n` days.  Returns an error if the result is out of range.
    pub fn add_days(self, n: i32) -> Result<Self> {
        let serial = self.0 + n;
        if serial <= 0 || Date(serial) > Self::MAX {
            return Err(Error::Date(format!(
                "date arithmetic: result {serial} out of range"
            )));
        }
        Ok(Date(serial))
    }

    /// Advance by a period expressed in the given time unit.
    pub fn advance(self, n: i32, unit: TimeUnit) -> Result<Self> {
        match unit {
            TimeUnit::Days => self.add_days(n),
            TimeUnit::Weeks => self.add_days(n * 7),
            TimeUnit::Months => {
                let (mut y, mut m, d) = ymd_from_serial(self.0);
                let total_months = m as i32 + n;
                // Normalise months to 1–12
                let full_years = total_months.div_euclid(12);
                let rem_months = total_months.rem_euclid(12);
                let (new_m, extra_y) = if rem_months == 0 {
                    (12u8, full_years - 1)
                } else {
                    (rem_months as u8, full_years)
                };
                let new_y = y as i32 + extra_y;
                if !(1900..=2199).contains(&new_y) {
                    return Err(Error::Date(format!("year {new_y} out of range")));
                }
                y = new_y as u16;
                m = new_m;
                let new_d = d.min(days_in_month(y, m));
                Ok(Date(serial_from_ymd(y, m, new_d)))
            }
            TimeUnit::Years => self.advance(n * 12, TimeUnit::Months),
            _ => Err(Error::Date(format!("advance() does not support {unit}"))),
        }
    }

    /// Return the number of calendar days between `self` and `other`.
    /// Positive if `other > self`.
    pub fn days_between(self, other: Date) -> i32 {
        other.0 - self.0
    }

    /// Return the last day of the month containing this date.
    pub fn end_of_month(self) -> Self {
        let (y, m, _) = ymd_from_serial(self.0);
        let last = days_in_month(y, m);
        Date(serial_from_ymd(y, m, last))
    }

    /// Return `true` if this is the last calendar day of its month.
    pub fn is_end_of_month(self) -> bool {
        self == self.end_of_month()
    }

    /// Return the *n*-th occurrence of `weekday` in the month of `year`/`month`.
    ///
    /// For example, `nth_weekday(3, Weekday::Wednesday, 2024, 3)` returns the
    /// third Wednesday of March 2024 (2024-03-20).
    ///
    /// # Errors
    /// Returns an error if the result is out of the valid date range or if `n`
    /// is zero or larger than the number of such weekdays in the month.
    pub fn nth_weekday(n: u8, weekday: Weekday, year: u16, month: u8) -> Result<Self> {
        if n == 0 {
            return Err(Error::Date("nth_weekday: n must be >= 1".into()));
        }
        // Start from the 1st of the month
        let first = Date::from_ymd(year, month, 1)?;
        let first_wd = first.weekday().ordinal(); // 1=Mon..7=Sun
        let target_wd = weekday.ordinal();
        // Days to advance from the 1st to reach the first occurrence
        let skip = ((target_wd as i32 - first_wd as i32).rem_euclid(7)) as u8;
        let day = 1 + skip + 7 * (n - 1);
        if day > days_in_month(year, month) {
            return Err(Error::Date(format!(
                "nth_weekday: {n}-th {weekday:?} does not exist in {year}-{month:02}"
            )));
        }
        Date::from_ymd(year, month, day)
    }
}

// ── Arithmetic operators ──────────────────────────────────────────────────────

impl std::ops::Add<i32> for Date {
    type Output = Self;
    fn add(self, rhs: i32) -> Self {
        self.add_days(rhs).expect("date addition overflow")
    }
}

impl std::ops::Sub<i32> for Date {
    type Output = Self;
    fn sub(self, rhs: i32) -> Self {
        self.add_days(-rhs).expect("date subtraction underflow")
    }
}

impl std::ops::Sub<Date> for Date {
    type Output = i32;
    fn sub(self, rhs: Date) -> i32 {
        self.0 - rhs.0
    }
}

impl std::ops::AddAssign<i32> for Date {
    fn add_assign(&mut self, rhs: i32) {
        *self = self.add_days(rhs).expect("date addition overflow");
    }
}

impl std::ops::SubAssign<i32> for Date {
    fn sub_assign(&mut self, rhs: i32) {
        *self = self.add_days(-rhs).expect("date subtraction underflow");
    }
}

// ── Display ───────────────────────────────────────────────────────────────────

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_null() {
            return write!(f, "null date");
        }
        let (y, m, d) = ymd_from_serial(self.0);
        let mon = [
            "January",
            "February",
            "March",
            "April",
            "May",
            "June",
            "July",
            "August",
            "September",
            "October",
            "November",
            "December",
        ][m as usize - 1];
        write!(f, "{d} {mon} {y}")
    }
}

impl std::fmt::Debug for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_null() {
            return write!(f, "Date(null)");
        }
        let (y, m, d) = ymd_from_serial(self.0);
        write!(f, "Date({y:04}-{m:02}-{d:02})")
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Whether a given year is a leap year.
pub fn is_leap_year(year: u16) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Number of days in a given month/year.
pub fn days_in_month(year: u16, month: u8) -> u8 {
    debug_assert!((1..=12).contains(&month));
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
        _ => unreachable!(),
    }
}

/// Convert (year, month, day) to a QuantLib serial number.
///
/// Serial 1 = 1900-01-01.
fn serial_from_ymd(year: u16, month: u8, day: u8) -> i32 {
    // QuantLib implementation: count days from Jan 1, 1900
    let y = year as i32;
    let m = month as i32;
    let d = day as i32;

    // Days in years 1900..year
    let mut serial = (y - 1900) * 365;
    // Leap years in [1900, year)
    serial += (y - 1901) / 4 - (y - 1901) / 100 + (y - 1601) / 400;
    // 1900 itself is special: QuantLib treats 1900 as non-leap
    // (matches the Excel "1900 leap year bug" – QuantLib does the same)
    // Days in months 1..m for the current year
    serial += MONTH_OFFSET[m as usize - 1] as i32;
    if m > 2 && is_leap_year(year) {
        serial += 1;
    }
    // Days in the current month
    serial += d;
    serial
}

/// Decompose a serial number into (year, month, day).
fn ymd_from_serial(serial: i32) -> (u16, u8, u8) {
    // Estimate year
    let mut y = (serial / 365 + 1900) as u16;
    // Adjust until serial falls within the year
    loop {
        let start_of_year = serial_from_ymd(y, 1, 1);
        if serial < start_of_year {
            y -= 1;
        } else if serial >= serial_from_ymd(y + 1, 1, 1) {
            y += 1;
        } else {
            break;
        }
    }
    let start_of_year = serial_from_ymd(y, 1, 1);
    let doy = serial - start_of_year + 1; // 1-based
                                          // Find month
    let mut m = 1u8;
    let mut remaining = doy;
    loop {
        let days = days_in_month(y, m) as i32;
        if remaining <= days {
            break;
        }
        remaining -= days;
        m += 1;
    }
    (y, m, remaining as u8)
}

/// Cumulative day-of-year offset at the start of each month (non-leap).
const MONTH_OFFSET: [u16; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epoch() {
        let d = Date::from_ymd(1900, 1, 1).unwrap();
        assert_eq!(d.serial(), 1);
    }

    #[test]
    fn test_roundtrip() {
        let dates = [
            (1900, 1, 1),
            (1900, 12, 31),
            (2000, 2, 29), // leap
            (2100, 2, 28), // non-leap century
            (2000, 1, 1),
            (2023, 6, 15),
            (2199, 12, 31),
        ];
        for (y, m, d) in dates {
            let date = Date::from_ymd(y, m, d).unwrap();
            assert_eq!(date.year(), y, "year mismatch for {y}-{m:02}-{d:02}");
            assert_eq!(date.month(), m, "month mismatch for {y}-{m:02}-{d:02}");
            assert_eq!(date.day_of_month(), d, "day mismatch for {y}-{m:02}-{d:02}");
        }
    }

    #[test]
    fn test_weekday() {
        // 2024-01-01 is a Monday
        let d = Date::from_ymd(2024, 1, 1).unwrap();
        assert_eq!(d.weekday(), Weekday::Monday);
        // 2024-01-06 is a Saturday
        let d2 = Date::from_ymd(2024, 1, 6).unwrap();
        assert_eq!(d2.weekday(), Weekday::Saturday);
    }

    #[test]
    fn test_advance_months() {
        let d = Date::from_ymd(2023, 1, 31).unwrap();
        // Jan 31 + 1 month = Feb 28 (clamp to end of month)
        let next = d.advance(1, TimeUnit::Months).unwrap();
        assert_eq!(next.month(), 2);
        assert_eq!(next.day_of_month(), 28);
    }

    #[test]
    fn test_end_of_month() {
        let d = Date::from_ymd(2024, 2, 15).unwrap();
        let eom = d.end_of_month();
        assert_eq!(eom.day_of_month(), 29); // 2024 is a leap year
    }

    #[test]
    fn test_arithmetic() {
        let d = Date::from_ymd(2023, 1, 1).unwrap();
        let d2 = d + 31;
        assert_eq!(d2.month(), 2);
        assert_eq!(d2.day_of_month(), 1);
        assert_eq!(Date::from_ymd(2023, 2, 1).unwrap() - d, 31);
    }

    #[test]
    fn test_nth_weekday() {
        // 3rd Wednesday of March 2024 = March 20
        let d = Date::nth_weekday(3, Weekday::Wednesday, 2024, 3).unwrap();
        assert_eq!(d, Date::from_ymd(2024, 3, 20).unwrap());
        assert_eq!(d.weekday(), Weekday::Wednesday);

        // 1st Monday of January 2024 = January 1
        let d2 = Date::nth_weekday(1, Weekday::Monday, 2024, 1).unwrap();
        assert_eq!(d2, Date::from_ymd(2024, 1, 1).unwrap());

        // 5th Monday of January 2024 = January 29
        let d3 = Date::nth_weekday(5, Weekday::Monday, 2024, 1).unwrap();
        assert_eq!(d3, Date::from_ymd(2024, 1, 29).unwrap());
    }

    #[test]
    fn test_nth_weekday_out_of_range() {
        // There is no 5th Wednesday in February 2024
        assert!(Date::nth_weekday(5, Weekday::Wednesday, 2024, 2).is_err());
        // n == 0 is invalid
        assert!(Date::nth_weekday(0, Weekday::Monday, 2024, 1).is_err());
    }
}
