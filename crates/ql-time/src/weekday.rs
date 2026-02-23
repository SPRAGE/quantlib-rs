//! `Weekday` — day-of-week enum (translates `ql/time/weekday.hpp`).

/// Day of the week.
///
/// Variants are numbered 1–7 (Monday = 1, Sunday = 7) to match the QuantLib
/// convention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Weekday {
    /// Monday (1).
    Monday = 1,
    /// Tuesday (2).
    Tuesday = 2,
    /// Wednesday (3).
    Wednesday = 3,
    /// Thursday (4).
    Thursday = 4,
    /// Friday (5).
    Friday = 5,
    /// Saturday (6).
    Saturday = 6,
    /// Sunday (7).
    Sunday = 7,
}

impl Weekday {
    /// Construct from the QuantLib ordinal (1 = Monday … 7 = Sunday).
    ///
    /// Returns `None` if the value is out of range.
    pub fn from_ordinal(n: u8) -> Option<Self> {
        match n {
            1 => Some(Weekday::Monday),
            2 => Some(Weekday::Tuesday),
            3 => Some(Weekday::Wednesday),
            4 => Some(Weekday::Thursday),
            5 => Some(Weekday::Friday),
            6 => Some(Weekday::Saturday),
            7 => Some(Weekday::Sunday),
            _ => None,
        }
    }

    /// Return `true` if this is Saturday or Sunday.
    pub fn is_weekend(&self) -> bool {
        matches!(self, Weekday::Saturday | Weekday::Sunday)
    }

    /// Return `true` if this is Monday–Friday.
    pub fn is_weekday(&self) -> bool {
        !self.is_weekend()
    }

    /// Return the QuantLib ordinal (1 = Monday … 7 = Sunday).
    pub fn ordinal(&self) -> u8 {
        *self as u8
    }
}

impl std::fmt::Display for Weekday {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Weekday::Monday => "Monday",
            Weekday::Tuesday => "Tuesday",
            Weekday::Wednesday => "Wednesday",
            Weekday::Thursday => "Thursday",
            Weekday::Friday => "Friday",
            Weekday::Saturday => "Saturday",
            Weekday::Sunday => "Sunday",
        };
        write!(f, "{name}")
    }
}
