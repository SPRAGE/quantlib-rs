//! # ql-time
//!
//! Date, calendar, day counter, schedule, and business-day-convention types.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

// ── Modules ───────────────────────────────────────────────────────────────────

/// ASX date utilities.
pub mod asx;

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

/// ECB date utilities.
pub mod ecb;

/// Payment / event frequency.
pub mod frequency;

/// IMM date utilities.
pub mod imm;

/// `InterestRate` — rate with compounding and day-counting conventions.
pub mod interest_rate;

/// `Month` — month of the year.
pub mod month;

/// `Period` — a time span in a `TimeUnit`.
pub mod period;

/// `Schedule` — an ordered sequence of dates.
pub mod schedule;

/// `TimeUnit` — days, weeks, months, years.
pub mod time_unit;

/// `Weekday` — day of the week.
pub mod weekday;

// ── Convenience re-exports ────────────────────────────────────────────────────

pub use asx::ASX;
pub use business_day_convention::BusinessDayConvention;
pub use calendar::{Calendar, NullCalendar, WeekendsOnly};
pub use calendars::bespoke_calendar::BespokeCalendar;
pub use calendars::joint_calendar::{JointCalendar, JointCalendarRule};
pub use date::Date;
pub use day_counter::{
    Actual360, Actual364, Actual36525, Actual365Fixed, Actual366, ActualActualAfb,
    ActualActualIsda, ActualActualIsma, Business252, DayCounter, OneDayCounter, SimpleDayCounter,
    Thirty360, Thirty360European, Thirty360German, Thirty360Italian, Thirty365,
};
pub use ecb::ECB;
pub use frequency::Frequency;
pub use imm::IMM;
pub use interest_rate::InterestRate;
pub use month::Month;
pub use period::Period;
pub use schedule::{DateGeneration, Schedule, ScheduleBuilder};
pub use time_unit::TimeUnit;
pub use weekday::Weekday;
