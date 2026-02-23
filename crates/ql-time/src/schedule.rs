//! `Schedule` — a sequence of dates (translates `ql/time/schedule.hpp`).
//!
//! A `Schedule` generates the payment/accrual dates for a financial
//! instrument given a start date, end date, tenor, calendar, and
//! business-day conventions.

use crate::business_day_convention::BusinessDayConvention;
use crate::calendar::Calendar;
use crate::date::Date;
use crate::period::Period;
use crate::weekday::Weekday;
use ql_core::errors::{Error, Result};

/// Date generation rule for schedules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DateGeneration {
    /// Dates generated backward from the end date.
    Backward,
    /// Dates generated forward from the start date.
    Forward,
    /// Zero coupons — only start and end dates.
    Zero,
    /// Third Wednesday rule (IMM dates).
    ThirdWednesday,
    /// Twentieth rule (CDS standard schedule).
    Twentieth,
    /// Twentieth IMM rule.
    TwentiethIMM,
    /// OldCDS rule.
    OldCDS,
    /// CDS rule.
    CDS,
    /// CDS 2015 rule.
    CDS2015,
}

/// An ordered sequence of coupon/payment dates.
///
/// Corresponds to `QuantLib::Schedule`.
#[derive(Debug, Clone)]
pub struct Schedule {
    dates: Vec<Date>,
    is_regular: Vec<bool>,
}

impl Schedule {
    /// Return all dates in the schedule.
    pub fn dates(&self) -> &[Date] {
        &self.dates
    }

    /// Number of dates.
    pub fn size(&self) -> usize {
        self.dates.len()
    }

    /// Return `true` if the schedule is empty.
    pub fn is_empty(&self) -> bool {
        self.dates.is_empty()
    }

    /// Return the `i`-th date.
    pub fn date(&self, i: usize) -> Date {
        self.dates[i]
    }

    /// Return the start (effective) date.
    pub fn start_date(&self) -> Option<Date> {
        self.dates.first().copied()
    }

    /// Return the end (termination) date.
    pub fn end_date(&self) -> Option<Date> {
        self.dates.last().copied()
    }

    /// Return `true` if the period at index `i` is a full (regular) period.
    pub fn is_regular(&self, i: usize) -> bool {
        // is_regular has one entry per *period* (i.e., size - 1 entries)
        self.is_regular.get(i).copied().unwrap_or(true)
    }

    /// Build a schedule from an explicit list of dates.
    pub fn from_dates(dates: Vec<Date>) -> Self {
        let n = if dates.len() > 1 { dates.len() - 1 } else { 0 };
        Self {
            is_regular: vec![true; n],
            dates,
        }
    }
}

/// Builder for [`Schedule`].
///
/// Corresponds to `QuantLib::MakeSchedule`.
#[derive(Debug)]
pub struct ScheduleBuilder<'a> {
    effective_date: Date,
    termination_date: Date,
    tenor: Period,
    calendar: &'a dyn Calendar,
    convention: BusinessDayConvention,
    termination_convention: BusinessDayConvention,
    rule: DateGeneration,
    end_of_month: bool,
    first_date: Option<Date>,
    next_to_last_date: Option<Date>,
}

impl<'a> ScheduleBuilder<'a> {
    /// Begin building a schedule.
    pub fn new(
        effective_date: Date,
        termination_date: Date,
        tenor: Period,
        calendar: &'a dyn Calendar,
    ) -> Self {
        Self {
            effective_date,
            termination_date,
            tenor,
            calendar,
            convention: BusinessDayConvention::ModifiedFollowing,
            termination_convention: BusinessDayConvention::ModifiedFollowing,
            rule: DateGeneration::Backward,
            end_of_month: false,
            first_date: None,
            next_to_last_date: None,
        }
    }

    /// Set the business-day convention for intermediate dates.
    pub fn with_convention(mut self, c: BusinessDayConvention) -> Self {
        self.convention = c;
        self
    }

    /// Set the business-day convention for the termination date.
    pub fn with_termination_convention(mut self, c: BusinessDayConvention) -> Self {
        self.termination_convention = c;
        self
    }

    /// Set the date-generation rule.
    pub fn with_rule(mut self, rule: DateGeneration) -> Self {
        self.rule = rule;
        self
    }

    /// Whether to snap dates to the end of the month.
    pub fn end_of_month(mut self, flag: bool) -> Self {
        self.end_of_month = flag;
        self
    }

    /// Optional first irregular coupon date.
    pub fn with_first_date(mut self, d: Date) -> Self {
        self.first_date = Some(d);
        self
    }

    /// Optional next-to-last (penultimate) irregular coupon date.
    pub fn with_next_to_last_date(mut self, d: Date) -> Self {
        self.next_to_last_date = Some(d);
        self
    }

    /// Build the `Schedule`.
    pub fn build(self) -> Result<Schedule> {
        let start = self.effective_date;
        let end = self.termination_date;

        if start >= end {
            return Err(Error::InvalidArgument(
                "effective date must be before termination date".into(),
            ));
        }

        // Zero coupon — just start and end.
        if self.tenor.length == 0
            || self.rule == DateGeneration::Zero
        {
            let dates = vec![
                self.calendar.adjust(start, self.convention),
                self.calendar.adjust(end, self.termination_convention),
            ];
            return Ok(Schedule {
                is_regular: vec![false],
                dates,
            });
        }

        let mut dates: Vec<Date> = Vec::new();
        let mut is_regular: Vec<bool> = Vec::new();

        match self.rule {
            DateGeneration::Forward => {
                dates.push(start);
                let mut seed = start;
                if let Some(fd) = self.first_date {
                    dates.push(self.calendar.adjust(fd, self.convention));
                    is_regular.push(false);
                    seed = fd;
                }
                let mut n = 1i32;
                loop {
                    let next = seed
                        .advance(n * self.tenor.length, self.tenor.unit)
                        .map_err(|e| Error::Date(e.to_string()))?;
                    if next >= end {
                        break;
                    }
                    let adj = self.calendar.adjust(next, self.convention);
                    if self.end_of_month && self.calendar.is_end_of_month(adj) {
                        let eom = self.calendar.end_of_month(next);
                        dates.push(eom);
                    } else {
                        dates.push(adj);
                    }
                    is_regular.push(true);
                    n += 1;
                }
                // Add next-to-last if provided
                if let Some(ntl) = self.next_to_last_date {
                    if dates.last().copied() != Some(self.calendar.adjust(ntl, self.convention)) {
                        dates.push(self.calendar.adjust(ntl, self.convention));
                        is_regular.push(false);
                    }
                }
                // Terminal date
                let term = self.calendar.adjust(end, self.termination_convention);
                // Regular if the last intermediate date is exactly one tenor before end
                let expected_last = end
                    .advance(-self.tenor.length, self.tenor.unit)
                    .ok()
                    .map(|d| self.calendar.adjust(d, self.convention));
                is_regular.push(dates.last().copied() == expected_last);
                dates.push(term);
            }

            DateGeneration::ThirdWednesday => {
                // Generate dates forward, then snap every intermediate date to
                // the third Wednesday of its month.
                dates.push(start);
                let mut n = 1i32;
                loop {
                    let next = start
                        .advance(n * self.tenor.length, self.tenor.unit)
                        .map_err(|e| Error::Date(e.to_string()))?;
                    if next >= end {
                        break;
                    }
                    // Snap to 3rd Wednesday
                    let tw = Date::nth_weekday(3, Weekday::Wednesday, next.year(), next.month())
                        .map_err(|e| Error::Date(e.to_string()))?;
                    dates.push(tw);
                    is_regular.push(true);
                    n += 1;
                }
                is_regular.push(true);
                dates.push(end);
            }

            DateGeneration::Twentieth | DateGeneration::TwentiethIMM => {
                // Generate dates forward, snapping intermediate dates to the
                // 20th of the month.
                dates.push(start);
                let mut n = 1i32;
                loop {
                    let next = start
                        .advance(n * self.tenor.length, self.tenor.unit)
                        .map_err(|e| Error::Date(e.to_string()))?;
                    if next >= end {
                        break;
                    }
                    // Snap to the 20th
                    let twentieth = Date::from_ymd(next.year(), next.month(), 20)
                        .map_err(|e| Error::Date(e.to_string()))?;
                    dates.push(twentieth);
                    is_regular.push(true);
                    n += 1;
                }
                is_regular.push(true);
                dates.push(end);
            }

            DateGeneration::CDS | DateGeneration::CDS2015 | DateGeneration::OldCDS => {
                // CDS schedules: generate quarterly dates snapped to the 20th
                // of Mar, Jun, Sep, Dec (the standard IMM months).
                // For CDS/CDS2015, the stub is at the front (short first).
                let cds_months = [3u8, 6, 9, 12];
                let mut raw = Vec::new();
                // Walk backwards from end, stepping 3 months at a time on
                // the 20th of CDS months.
                let mut y = end.year();
                let mut m = end.month();
                // Find the CDS month on or before the end
                if !cds_months.contains(&m) {
                    // Find the most recent CDS month
                    for &cm in cds_months.iter().rev() {
                        if cm <= m {
                            m = cm;
                            break;
                        }
                    }
                    if m > end.month() {
                        // Wrapped around year — previous December
                        m = 12;
                        y -= 1;
                    }
                }

                // Generate dates backward from the CDS month on or before end
                loop {
                    let d = Date::from_ymd(y, m, 20)
                        .map_err(|e| Error::Date(e.to_string()))?;
                    if d <= start {
                        // Include one more for CDS (the previous 20th before start)
                        raw.push(d);
                        break;
                    }
                    raw.push(d);
                    // Move back 3 months
                    if m <= 3 {
                        m = m + 12 - 3;
                        y -= 1;
                    } else {
                        m -= 3;
                    }
                }
                raw.reverse();

                // Ensure start and end are included
                dates = raw;
                // If the first date is before start, keep it (it becomes the
                // effective date for the stub).
                // Always include the termination date.
                if dates.last().map(|d| *d < end).unwrap_or(true) {
                    dates.push(end);
                }

                // Build is_regular (all regular for CDS)
                is_regular = vec![true; dates.len().saturating_sub(1)];
                // First period might be a stub
                if dates.len() > 1 {
                    is_regular[0] = false;
                }
            }

            DateGeneration::Backward => {
                dates.push(end);
                let mut seed = end;
                if let Some(ntl) = self.next_to_last_date {
                    dates.insert(0, self.calendar.adjust(ntl, self.convention));
                    is_regular.push(false);
                    seed = ntl;
                }
                let mut n = 1i32;
                loop {
                    let prev = seed
                        .advance(-n * self.tenor.length, self.tenor.unit)
                        .map_err(|e| Error::Date(e.to_string()))?;
                    if prev <= start {
                        break;
                    }
                    let adj = self.calendar.adjust(prev, self.convention);
                    dates.insert(0, adj);
                    is_regular.push(true);
                    n += 1;
                }
                // Add first date if provided and not already present
                if let Some(fd) = self.first_date {
                    let adj_fd = self.calendar.adjust(fd, self.convention);
                    if dates.first().copied() != Some(adj_fd) {
                        dates.insert(0, adj_fd);
                        is_regular.push(false);
                    }
                }
                // Start date — is regular if next date is exactly one tenor after start
                let expected_next = start
                    .advance(self.tenor.length, self.tenor.unit)
                    .ok()
                    .map(|d| self.calendar.adjust(d, self.convention));
                is_regular.push(dates.first().copied() == expected_next);
                dates.insert(0, self.calendar.adjust(start, self.convention));
                is_regular.reverse();
            }

            DateGeneration::Zero => {
                // Already handled above; this arm is unreachable.
                unreachable!("Zero coupon is handled before the match");
            }
        }

        // Deduplicate adjacent equal dates
        dates.dedup();

        Ok(Schedule { dates, is_regular })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::WeekendsOnly;
    use crate::time_unit::TimeUnit;

    fn date(y: u16, m: u8, d: u8) -> Date {
        Date::from_ymd(y, m, d).unwrap()
    }

    #[test]
    fn zero_coupon_schedule() {
        let cal = WeekendsOnly;
        let sched = ScheduleBuilder::new(
            date(2023, 1, 1),
            date(2025, 1, 1),
            Period::new(0, TimeUnit::Years),
            &cal,
        )
        .build()
        .unwrap();
        assert_eq!(sched.size(), 2);
    }

    #[test]
    fn annual_backward_schedule() {
        let cal = WeekendsOnly;
        let sched = ScheduleBuilder::new(
            date(2020, 1, 1),
            date(2023, 1, 1),
            Period::new(1, TimeUnit::Years),
            &cal,
        )
        .with_rule(DateGeneration::Backward)
        .build()
        .unwrap();
        // Should have: 2020-01-01, 2021-01-01, 2022-01-01, 2023-01-01
        assert_eq!(sched.size(), 4);
        assert_eq!(sched.start_date().unwrap(), date(2020, 1, 1));
        assert_eq!(sched.end_date().unwrap(), date(2023, 1, 1));
    }

    #[test]
    fn forward_schedule() {
        let cal = WeekendsOnly;
        let sched = ScheduleBuilder::new(
            date(2020, 1, 2),  // Thursday
            date(2023, 1, 2),  // Monday
            Period::new(1, TimeUnit::Years),
            &cal,
        )
        .with_rule(DateGeneration::Forward)
        .build()
        .unwrap();
        assert_eq!(sched.size(), 4);
        assert_eq!(sched.start_date().unwrap(), date(2020, 1, 2));
        assert_eq!(sched.end_date().unwrap(), date(2023, 1, 2));
    }

    #[test]
    fn third_wednesday_quarterly() {
        let cal = WeekendsOnly;
        // Quarterly schedule with ThirdWednesday rule for 2024
        let sched = ScheduleBuilder::new(
            date(2024, 1, 1),
            date(2025, 1, 1),
            Period::new(3, TimeUnit::Months),
            &cal,
        )
        .with_rule(DateGeneration::ThirdWednesday)
        .build()
        .unwrap();
        // start=Jan 1, then 3rd Wed of Apr, Jul, Oct, end=Jan 1
        // 3rd Wed Apr 2024 = Apr 17
        // 3rd Wed Jul 2024 = Jul 17
        // 3rd Wed Oct 2024 = Oct 16
        assert_eq!(sched.size(), 5);
        assert_eq!(sched.date(1), date(2024, 4, 17));
        assert_eq!(sched.date(2), date(2024, 7, 17));
        assert_eq!(sched.date(3), date(2024, 10, 16));
    }

    #[test]
    fn twentieth_quarterly() {
        let cal = WeekendsOnly;
        let sched = ScheduleBuilder::new(
            date(2024, 1, 1),
            date(2025, 1, 1),
            Period::new(3, TimeUnit::Months),
            &cal,
        )
        .with_rule(DateGeneration::Twentieth)
        .build()
        .unwrap();
        // start, 20th of Apr, Jul, Oct, end
        assert_eq!(sched.size(), 5);
        assert_eq!(sched.date(1), date(2024, 4, 20));
        assert_eq!(sched.date(2), date(2024, 7, 20));
        assert_eq!(sched.date(3), date(2024, 10, 20));
    }

    #[test]
    fn cds_schedule() {
        let cal = WeekendsOnly;
        // CDS schedule: 1-year with quarterly standard dates
        let sched = ScheduleBuilder::new(
            date(2024, 1, 15),
            date(2025, 3, 20),
            Period::new(3, TimeUnit::Months),
            &cal,
        )
        .with_rule(DateGeneration::CDS)
        .build()
        .unwrap();
        // Should snap to 20th of Mar, Jun, Sep, Dec
        // Dates should be: Dec 20 (before start), Mar 20, Jun 20, Sep 20, Dec 20, Mar 20
        assert!(sched.size() >= 4);
        // All intermediate dates should be on the 20th
        for i in 0..sched.size() {
            let d = sched.date(i);
            if i > 0 && i < sched.size() - 1 {
                assert_eq!(d.day_of_month(), 20, "intermediate date should be 20th: {d}");
            }
        }
    }
}
