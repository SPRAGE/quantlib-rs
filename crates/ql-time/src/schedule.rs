//! `Schedule` — a sequence of dates (translates `ql/time/schedule.hpp`).
//!
//! A `Schedule` generates the payment/accrual dates for a financial
//! instrument given a start date, end date, tenor, calendar, and
//! business-day conventions.

use crate::business_day_convention::BusinessDayConvention;
use crate::calendar::{Calendar, NullCalendar};
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

    /// Build a schedule from an explicit list of dates and regularity flags.
    pub fn from_dates_with_regular(dates: Vec<Date>, is_regular: Vec<bool>) -> Self {
        Self { dates, is_regular }
    }

    /// Return a truncated schedule containing all dates before `truncation_date`.
    ///
    /// If `truncation_date` falls on an existing schedule date, that date
    /// becomes the new terminal date.  Otherwise `truncation_date` is appended
    /// and the last period is marked irregular.
    pub fn until(&self, truncation_date: Date) -> Self {
        let mut dates = Vec::new();
        let mut is_regular = Vec::new();
        for (i, &d) in self.dates.iter().enumerate() {
            if d < truncation_date {
                dates.push(d);
                if i > 0 {
                    is_regular.push(self.is_regular(i));
                }
            } else if d == truncation_date {
                dates.push(d);
                if i > 0 {
                    is_regular.push(self.is_regular(i));
                }
                break;
            } else {
                // d > truncation_date — append truncation_date as irregular end
                dates.push(truncation_date);
                is_regular.push(false);
                break;
            }
        }
        Self { dates, is_regular }
    }

    /// Return a truncated schedule containing all dates after `truncation_date`.
    ///
    /// If `truncation_date` falls on an existing schedule date, that date
    /// becomes the new start date.  Otherwise `truncation_date` is prepended
    /// and the first period is marked irregular.
    pub fn after(&self, truncation_date: Date) -> Self {
        let mut dates = Vec::new();
        let mut is_regular = Vec::new();
        let mut found = false;
        for (i, &d) in self.dates.iter().enumerate() {
            if d < truncation_date {
                continue;
            }
            if d == truncation_date {
                dates.push(d);
                found = true;
            } else {
                // d > truncation_date
                if !found {
                    dates.push(truncation_date);
                    is_regular.push(false);
                    found = true;
                }
                dates.push(d);
                if dates.len() > 1 && is_regular.len() < dates.len() - 1 {
                    // For dates that came from the original schedule, use their
                    // regularity.  The period spanning the truncation boundary
                    // was already marked above.
                    is_regular.push(self.is_regular(i));
                }
            }
        }
        Self { dates, is_regular }
    }

    /// Return the full is_regular vector (one entry per period).
    pub fn is_regular_vec(&self) -> &[bool] {
        &self.is_regular
    }
}

/// Advance a date using NullCalendar semantics, optionally snapping to
/// end-of-month.  This mirrors the C++ pattern:
///     `NullCalendar().advance(seed, n*tenor, convention, endOfMonth)`
fn null_advance_eom(seed: Date, tenor: &Period, n: i32, end_of_month: bool) -> Result<Date> {
    let mut next = seed
        .advance(n * tenor.length, tenor.unit)
        .map_err(|e| Error::Date(e.to_string()))?;
    if end_of_month && NullCalendar.is_end_of_month(seed) {
        next = next.end_of_month();
    }
    Ok(next)
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
    ///
    /// The implementation follows the same two-phase approach as the C++
    /// `QuantLib::Schedule` constructor:
    ///
    /// 1. **Generation** — raw dates are produced using `NullCalendar`-style
    ///    arithmetic (pure date advance, with optional end-of-month snap if the
    ///    seed *is* the last calendar day of its month).
    /// 2. **Adjustment** — a shared post-processing pass applies business-day
    ///    conventions, end-of-month snapping via the actual calendar, and
    ///    safety clean-ups.
    pub fn build(self) -> Result<Schedule> {
        let start = self.effective_date;
        let end = self.termination_date;
        let conv = self.convention;
        let term_conv = self.termination_convention;

        if start >= end {
            return Err(Error::InvalidArgument(
                "effective date must be before termination date".into(),
            ));
        }

        // Zero coupon — just start and end.
        if self.tenor.length == 0 || self.rule == DateGeneration::Zero {
            let dates = vec![
                self.calendar.adjust(start, conv),
                self.calendar.adjust(end, term_conv),
            ];
            return Ok(Schedule {
                is_regular: vec![false],
                dates,
            });
        }

        let mut dates: Vec<Date> = Vec::new();
        let mut is_regular: Vec<bool> = Vec::new();
        // `seed` is used after the match for the EOM adjustment check.
        // Forward sets it to `start`; Backward sets it to `end`.
        let mut seed = start;

        match self.rule {
            // ── Forward ────────────────────────────────────────────────
            DateGeneration::Forward => {
                dates.push(start);

                if let Some(fd) = self.first_date {
                    if fd != end {
                        dates.push(fd);
                        let expected = null_advance_eom(seed, &self.tenor, 1, self.end_of_month)?;
                        is_regular.push(expected == fd);
                        seed = fd;
                    }
                }

                let exit_date = self.next_to_last_date.unwrap_or(end);

                let mut periods = 1i32;
                loop {
                    let temp = null_advance_eom(seed, &self.tenor, periods, self.end_of_month)?;
                    if temp > exit_date {
                        if let Some(ntl) = self.next_to_last_date {
                            let adj_last = dates.last().map(|d| self.calendar.adjust(*d, conv));
                            let adj_ntl = self.calendar.adjust(ntl, conv);
                            if adj_last != Some(adj_ntl) {
                                dates.push(ntl);
                                is_regular.push(false);
                            }
                        }
                        break;
                    }
                    // Skip dates that would result in duplicates after
                    // adjustment.
                    let adj_last = dates.last().map(|d| self.calendar.adjust(*d, conv));
                    let adj_temp = self.calendar.adjust(temp, conv);
                    if adj_last != Some(adj_temp) {
                        dates.push(temp);
                        is_regular.push(true);
                    }
                    periods += 1;
                }

                // Terminal date.
                let adj_last = dates.last().map(|d| self.calendar.adjust(*d, conv));
                let adj_term = self.calendar.adjust(end, term_conv);
                if adj_last != Some(adj_term) {
                    dates.push(end);
                    is_regular.push(false);
                }
            }

            // ── Backward ───────────────────────────────────────────────
            DateGeneration::Backward => {
                dates.push(end);
                seed = end;

                if let Some(ntl) = self.next_to_last_date {
                    dates.push(ntl);
                    let expected = null_advance_eom(seed, &self.tenor, -1, self.end_of_month)?;
                    is_regular.push(expected == ntl);
                    seed = ntl;
                }

                let exit_date = self.first_date.unwrap_or(start);

                let mut periods = 1i32;
                loop {
                    let temp = null_advance_eom(seed, &self.tenor, -periods, self.end_of_month)?;
                    if temp < exit_date {
                        if let Some(fd) = self.first_date {
                            let adj_head = dates.last().map(|d| self.calendar.adjust(*d, conv));
                            let adj_fd = self.calendar.adjust(fd, conv);
                            if adj_head != Some(adj_fd) {
                                dates.push(fd);
                                is_regular.push(false);
                            }
                        }
                        break;
                    }
                    // Skip duplicates (after adjustment).
                    let adj_head = dates.last().map(|d| self.calendar.adjust(*d, conv));
                    let adj_temp = self.calendar.adjust(temp, conv);
                    if adj_head != Some(adj_temp) {
                        dates.push(temp);
                        is_regular.push(true);
                    }
                    periods += 1;
                }

                // Start date.
                let adj_head = dates.last().map(|d| self.calendar.adjust(*d, conv));
                let adj_start = self.calendar.adjust(start, conv);
                if adj_head != Some(adj_start) {
                    dates.push(start);
                    is_regular.push(false);
                }

                dates.reverse();
                is_regular.reverse();
            }

            // ── ThirdWednesday ─────────────────────────────────────────
            DateGeneration::ThirdWednesday => {
                dates.push(start);
                let mut n = 1i32;
                loop {
                    let next = start
                        .advance(n * self.tenor.length, self.tenor.unit)
                        .map_err(|e| Error::Date(e.to_string()))?;
                    if next >= end {
                        break;
                    }
                    let tw = Date::nth_weekday(3, Weekday::Wednesday, next.year(), next.month())
                        .map_err(|e| Error::Date(e.to_string()))?;
                    dates.push(tw);
                    is_regular.push(true);
                    n += 1;
                }
                is_regular.push(true);
                dates.push(end);
                // ThirdWednesday dates are not subject to the shared
                // adjustment; intermediate dates are snapped to the 3rd
                // Wednesday which overrides business-day conventions.
                dates.dedup();
                return Ok(Schedule { dates, is_regular });
            }

            // ── Twentieth / TwentiethIMM ───────────────────────────────
            DateGeneration::Twentieth | DateGeneration::TwentiethIMM => {
                dates.push(start);
                let mut n = 1i32;
                loop {
                    let next = start
                        .advance(n * self.tenor.length, self.tenor.unit)
                        .map_err(|e| Error::Date(e.to_string()))?;
                    if next >= end {
                        break;
                    }
                    let twentieth = Date::from_ymd(next.year(), next.month(), 20)
                        .map_err(|e| Error::Date(e.to_string()))?;
                    dates.push(twentieth);
                    is_regular.push(true);
                    n += 1;
                }
                is_regular.push(true);
                dates.push(end);
                // Twentieth rule typically uses Unadjusted; return directly.
                dates.dedup();
                return Ok(Schedule { dates, is_regular });
            }

            // ── CDS / CDS2015 / OldCDS ────────────────────────────────
            //
            // These rules use Forward generation starting from
            // previousTwentieth (for CDS/CDS2015) or the effective date
            // (for OldCDS).  The implementation follows the C++ QuantLib
            // constructor exactly.
            DateGeneration::CDS | DateGeneration::CDS2015 | DateGeneration::OldCDS => {
                let rule_val = self.rule;

                // ── Start date(s) ──
                if matches!(rule_val, DateGeneration::CDS | DateGeneration::CDS2015) {
                    let prev20th = previous_twentieth(start, rule_val)?;
                    if self.calendar.adjust(prev20th, conv) > start {
                        // Extra period before the previous 20th.
                        let extra = prev20th
                            .advance(-3, crate::time_unit::TimeUnit::Months)
                            .map_err(|e| Error::Date(e.to_string()))?;
                        dates.push(extra);
                        is_regular.push(true);
                    }
                    dates.push(prev20th);
                } else {
                    // OldCDS: start with the original effective date.
                    dates.push(start);
                }

                seed = *dates.last().unwrap();

                // ── First period ──
                let mut nxt = next_twentieth(start, rule_val)?;
                if rule_val == DateGeneration::OldCDS {
                    // 30-day stub distance rule (natural days).
                    let stub_days = 30;
                    if nxt - start < stub_days {
                        nxt = next_twentieth(
                            nxt.advance(1, crate::time_unit::TimeUnit::Days)
                                .map_err(|e| Error::Date(e.to_string()))?,
                            rule_val,
                        )?;
                    }
                }
                if nxt != start {
                    dates.push(nxt);
                    is_regular.push(matches!(
                        rule_val,
                        DateGeneration::CDS | DateGeneration::CDS2015
                    ));
                    seed = nxt;
                }

                // ── Forward loop (quarterly steps from seed) ──
                let mut periods = 1i32;
                loop {
                    let temp = seed
                        .advance(periods * 3, crate::time_unit::TimeUnit::Months)
                        .map_err(|e| Error::Date(e.to_string()))?;
                    // Snap to 20th (safety against month-length quirks).
                    let temp = Date::from_ymd(temp.year(), temp.month(), 20)
                        .map_err(|e| Error::Date(e.to_string()))?;
                    if temp > end {
                        break;
                    }
                    // Skip dates that would duplicate the previous one
                    // after adjustment.
                    if self.calendar.adjust(*dates.last().unwrap(), conv)
                        != self.calendar.adjust(temp, conv)
                    {
                        dates.push(temp);
                        is_regular.push(true);
                    }
                    periods += 1;
                }

                // ── Termination ──
                if self.calendar.adjust(*dates.last().unwrap(), term_conv)
                    != self.calendar.adjust(end, term_conv)
                {
                    let next20 = next_twentieth(end, rule_val)?;
                    dates.push(next20);
                    is_regular.push(true);
                }

                // Fall through to the shared adjustment pass.
            }

            DateGeneration::Zero => {
                unreachable!("Zero coupon is handled before the match");
            }
        }

        // ── Shared adjustment pass (Forward / Backward / CDS) ─────────
        //
        // 1. Adjust first date (unless Unadjusted or OldCDS).
        // 2. Adjust intermediate dates; if EOM mode and the calendar considers
        //    the seed an end-of-month date, snap each to Date::end_of_month()
        //    first, then apply convention.
        // 3. Adjust last date (unless Unadjusted or CDS/CDS2015).
        // 4. Safety: remove date[n-2] if >= date[n-1]; remove date[1] if <=
        //    date[0].

        let n = dates.len();
        if n == 0 {
            return Ok(Schedule { dates, is_regular });
        }

        // First date: NOT adjusted for OldCDS schedules.
        if conv != BusinessDayConvention::Unadjusted && self.rule != DateGeneration::OldCDS {
            dates[0] = self.calendar.adjust(dates[0], conv);
        }

        // Last date: NOT adjusted for CDS/CDS2015 (ISDA spec).
        if n > 1
            && term_conv != BusinessDayConvention::Unadjusted
            && !matches!(self.rule, DateGeneration::CDS | DateGeneration::CDS2015)
        {
            let last = n - 1;
            dates[last] = self.calendar.adjust(dates[last], term_conv);
        }

        // Intermediate dates.
        let eom_calendar = self.end_of_month && self.calendar.is_end_of_month(seed);
        if n > 2 {
            for d in dates.iter_mut().take(n - 1).skip(1) {
                if eom_calendar {
                    *d = self.calendar.adjust(d.end_of_month(), conv);
                } else {
                    *d = self.calendar.adjust(*d, conv);
                }
            }
        }

        // Safety: if second-to-last date >= last date (can happen with EOM
        // snap pushing past end), merge them.
        if dates.len() >= 2 {
            let last_idx = dates.len() - 1;
            let penult = last_idx - 1;
            if dates[penult] >= dates[last_idx] {
                if is_regular.len() >= 2 {
                    let idx = is_regular.len() - 2;
                    is_regular[idx] = dates[penult] == dates[last_idx];
                }
                dates[penult] = dates[last_idx];
                dates.pop();
                is_regular.pop();
            }
        }

        // Safety: if second date <= first date, merge them.
        if dates.len() >= 2 && dates[1] <= dates[0] {
            if is_regular.len() >= 2 {
                is_regular[1] = dates[1] == dates[0];
            }
            dates[1] = dates[0];
            dates.remove(0);
            is_regular.remove(0);
        }

        // Final dedup (belt-and-suspenders).
        dates.dedup();

        Ok(Schedule { dates, is_regular })
    }
}

// ── Free functions ────────────────────────────────────────────────────────

/// Return the 20th of the current or previous IMM quarter-month
/// (Mar/Jun/Sep/Dec) on or before `date`.
///
/// For `Twentieth` and `TwentiethIMM` rules the months do not need to be
/// quarterly.  For `OldCDS`, `CDS`, and `CDS2015` the result is rounded
/// down to the nearest quarter-month 20th.
///
/// Corresponds to `QuantLib::previousTwentieth()`.
pub fn previous_twentieth(d: Date, rule: DateGeneration) -> Result<Date> {
    let mut result =
        Date::from_ymd(d.year(), d.month(), 20).map_err(|e| Error::Date(e.to_string()))?;
    if result > d {
        result = result
            .advance(-1, crate::time_unit::TimeUnit::Months)
            .map_err(|e| Error::Date(e.to_string()))?;
    }
    if matches!(
        rule,
        DateGeneration::TwentiethIMM
            | DateGeneration::OldCDS
            | DateGeneration::CDS
            | DateGeneration::CDS2015
    ) {
        let m = result.month();
        if m % 3 != 0 {
            let skip = (m % 3) as i32;
            result = result
                .advance(-skip, crate::time_unit::TimeUnit::Months)
                .map_err(|e| Error::Date(e.to_string()))?;
        }
    }
    Ok(result)
}

/// Return the 20th of the next IMM quarter-month (Mar/Jun/Sep/Dec) on or
/// after `date`.
///
/// Corresponds to `QuantLib::nextTwentieth()`.
pub fn next_twentieth(d: Date, rule: DateGeneration) -> Result<Date> {
    let mut result =
        Date::from_ymd(d.year(), d.month(), 20).map_err(|e| Error::Date(e.to_string()))?;
    if result < d {
        result = result
            .advance(1, crate::time_unit::TimeUnit::Months)
            .map_err(|e| Error::Date(e.to_string()))?;
    }
    if matches!(
        rule,
        DateGeneration::TwentiethIMM
            | DateGeneration::OldCDS
            | DateGeneration::CDS
            | DateGeneration::CDS2015
    ) {
        let m = result.month();
        if m % 3 != 0 {
            let skip = (3 - m % 3) as i32;
            result = result
                .advance(skip, crate::time_unit::TimeUnit::Months)
                .map_err(|e| Error::Date(e.to_string()))?;
        }
    }
    Ok(result)
}

/// Compute the CDS maturity date for a given trade date, tenor, and CDS
/// date generation rule.
///
/// The tenor must be a multiple of 3 months (or a whole number of years).
/// The rule must be one of `CDS2015`, `CDS`, or `OldCDS`.
///
/// Corresponds to `QuantLib::cdsMaturity()`.
pub fn cds_maturity(
    trade_date: Date,
    tenor: &Period,
    rule: DateGeneration,
) -> Result<Option<Date>> {
    if !matches!(
        rule,
        DateGeneration::CDS2015 | DateGeneration::CDS | DateGeneration::OldCDS
    ) {
        return Err(Error::InvalidArgument(
            "cds_maturity should only be used with CDS2015, CDS, or OldCDS".into(),
        ));
    }
    if tenor.unit != crate::time_unit::TimeUnit::Years
        && !(tenor.unit == crate::time_unit::TimeUnit::Months && tenor.length % 3 == 0)
    {
        return Err(Error::InvalidArgument(
            "cds_maturity expects a tenor that is a multiple of 3 months".into(),
        ));
    }
    if rule == DateGeneration::OldCDS
        && tenor.length == 0
        && tenor.unit == crate::time_unit::TimeUnit::Months
    {
        return Err(Error::InvalidArgument(
            "A tenor of 0M is not supported for OldCDS".into(),
        ));
    }

    let mut anchor = previous_twentieth(trade_date, rule)?;

    if rule == DateGeneration::CDS2015 {
        let dec20 =
            Date::from_ymd(anchor.year(), 12, 20).map_err(|e| Error::Date(e.to_string()))?;
        let jun20 = Date::from_ymd(anchor.year(), 6, 20).map_err(|e| Error::Date(e.to_string()))?;
        if anchor == dec20 || anchor == jun20 {
            if tenor.length == 0 {
                return Ok(None); // matured
            }
            anchor = anchor
                .advance(-3, crate::time_unit::TimeUnit::Months)
                .map_err(|e| Error::Date(e.to_string()))?;
        }
    }

    // maturity = anchor + tenor + 3M
    let step1 = anchor
        .advance(tenor.length, tenor.unit)
        .map_err(|e| Error::Date(e.to_string()))?;
    let maturity = step1
        .advance(3, crate::time_unit::TimeUnit::Months)
        .map_err(|e| Error::Date(e.to_string()))?;

    if maturity <= trade_date {
        return Err(Error::InvalidArgument(format!(
            "CDS maturity {} <= trade date {}",
            maturity, trade_date
        )));
    }

    Ok(Some(maturity))
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
        .with_convention(BusinessDayConvention::Unadjusted)
        .with_termination_convention(BusinessDayConvention::Unadjusted)
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
            date(2020, 1, 2), // Thursday
            date(2023, 1, 2), // Monday
            Period::new(1, TimeUnit::Years),
            &cal,
        )
        .with_rule(DateGeneration::Forward)
        .with_convention(BusinessDayConvention::Unadjusted)
        .with_termination_convention(BusinessDayConvention::Unadjusted)
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
                assert_eq!(
                    d.day_of_month(),
                    20,
                    "intermediate date should be 20th: {d}"
                );
            }
        }
    }
}
