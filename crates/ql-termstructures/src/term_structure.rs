//! `TermStructure` — base trait for all term structures
//! (translates `ql/termstructure.hpp`).
//!
//! Every term structure has a **reference date**, a **day counter**, and a
//! **maximum date** (the furthest point at which extrapolation is allowed).

use ql_core::Time;
use ql_time::{Calendar, Date, DayCounter};

/// Base trait for all term structures.
///
/// Corresponds to `QuantLib::TermStructure`.
pub trait TermStructure: std::fmt::Debug + Send + Sync {
    /// The date at which discount = 1.0 and from which time is measured.
    fn reference_date(&self) -> Date;

    /// The day counter used for date → time-fraction conversions.
    fn day_counter(&self) -> &dyn DayCounter;

    /// The calendar used for date adjustments.
    fn calendar(&self) -> &dyn Calendar;

    /// The latest date for which the curve can be used.
    fn max_date(&self) -> Date;

    /// The latest time for which the curve can be used.
    fn max_time(&self) -> Time {
        self.time_from_reference(self.max_date())
    }

    /// Convert a date to a year fraction relative to the reference date.
    fn time_from_reference(&self, date: Date) -> Time {
        self.day_counter()
            .year_fraction(self.reference_date(), date)
    }

    /// Check whether a date is in the valid range of the term structure.
    fn check_range_date(&self, date: Date) -> bool {
        date >= self.reference_date() && date <= self.max_date()
    }

    /// Check whether a time is in the valid range of the term structure.
    fn check_range_time(&self, t: Time) -> bool {
        t >= 0.0 && t <= self.max_time()
    }
}
