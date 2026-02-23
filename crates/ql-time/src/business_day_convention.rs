//! Business-day convention (translates `ql/time/businessdayconvention.hpp`).

/// How to adjust a date that falls on a non-business day.
///
/// Corresponds to `QuantLib::BusinessDayConvention`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BusinessDayConvention {
    /// Choose the first business day after the given holiday.
    Following,
    /// Choose the first business day after the given holiday unless it belongs
    /// to a different month; in that case choose the first business day before
    /// the holiday.
    ModifiedFollowing,
    /// Choose the first business day before the given holiday.
    Preceding,
    /// Choose the first business day before the given holiday unless it belongs
    /// to a different month; in that case choose the first business day after
    /// the holiday.
    ModifiedPreceding,
    /// Do not adjust (keep the original date).
    Unadjusted,
    /// Choose the nearest business day.  In case of a tie, use the following
    /// convention.
    Nearest,
    /// End of month — choose the last business day of the same month.
    EndOfMonth,
}

impl std::fmt::Display for BusinessDayConvention {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BusinessDayConvention::Following => "Following",
            BusinessDayConvention::ModifiedFollowing => "Modified Following",
            BusinessDayConvention::Preceding => "Preceding",
            BusinessDayConvention::ModifiedPreceding => "Modified Preceding",
            BusinessDayConvention::Unadjusted => "Unadjusted",
            BusinessDayConvention::Nearest => "Nearest",
            BusinessDayConvention::EndOfMonth => "End of Month",
        };
        write!(f, "{s}")
    }
}
