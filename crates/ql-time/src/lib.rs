//! # ql-time
//!
//! Date, calendar, day counter, schedule, and business-day-convention types.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

// ── Modules ───────────────────────────────────────────────────────────────────

/// Business-day adjustment conventions.
pub mod business_day_convention;

/// Calendar trait and built-in implementations.
pub mod calendar;

/// Concrete calendar implementations (country / exchange specific).
pub mod calendars;

/// `Date` type.
pub mod date;

/// `DayCounter` trait and built-in day-count conventions.
pub mod day_counter;

/// Payment / event frequency.
pub mod frequency;

/// `Period` — a time span in a `TimeUnit`.
pub mod period;

/// `Schedule` — an ordered sequence of dates.
pub mod schedule;

/// `TimeUnit` — days, weeks, months, years.
pub mod time_unit;

/// `Weekday` — day of the week.
pub mod weekday;

// ── Convenience re-exports ────────────────────────────────────────────────────

pub use business_day_convention::BusinessDayConvention;
pub use calendar::{Calendar, NullCalendar, WeekendsOnly};
pub use date::Date;
pub use day_counter::{
    Actual360, Actual36525, Actual365Fixed, ActualActualIsda, Business252, DayCounter, Thirty360,
};
pub use frequency::Frequency;
pub use period::Period;
pub use schedule::{DateGeneration, Schedule, ScheduleBuilder};
pub use time_unit::TimeUnit;
pub use weekday::Weekday;
