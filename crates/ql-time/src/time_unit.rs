//! `TimeUnit` — units of time used in `Period` (translates
//! `ql/time/timeunit.hpp`).

/// A unit of time.
///
/// Corresponds to `QuantLib::TimeUnit`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeUnit {
    /// Calendar days.
    Days,
    /// Calendar weeks (7 days).
    Weeks,
    /// Calendar months.
    Months,
    /// Calendar years (12 months).
    Years,
    /// Hours (used in some short-date calculations).
    Hours,
    /// Minutes.
    Minutes,
    /// Seconds.
    Seconds,
    /// Milliseconds.
    Milliseconds,
    /// Microseconds.
    Microseconds,
}

impl std::fmt::Display for TimeUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeUnit::Days => write!(f, "Day(s)"),
            TimeUnit::Weeks => write!(f, "Week(s)"),
            TimeUnit::Months => write!(f, "Month(s)"),
            TimeUnit::Years => write!(f, "Year(s)"),
            TimeUnit::Hours => write!(f, "Hour(s)"),
            TimeUnit::Minutes => write!(f, "Minute(s)"),
            TimeUnit::Seconds => write!(f, "Second(s)"),
            TimeUnit::Milliseconds => write!(f, "Millisecond(s)"),
            TimeUnit::Microseconds => write!(f, "Microsecond(s)"),
        }
    }
}
