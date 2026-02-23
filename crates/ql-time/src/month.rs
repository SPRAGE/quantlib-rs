//! `Month` — month-of-year enum (translates `ql/time/date.hpp` `Month` enum).

/// Month of the year.
///
/// Variants are numbered 1–12 (January = 1, December = 12) to match
/// the QuantLib convention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Month {
    /// January (1).
    January = 1,
    /// February (2).
    February = 2,
    /// March (3).
    March = 3,
    /// April (4).
    April = 4,
    /// May (5).
    May = 5,
    /// June (6).
    June = 6,
    /// July (7).
    July = 7,
    /// August (8).
    August = 8,
    /// September (9).
    September = 9,
    /// October (10).
    October = 10,
    /// November (11).
    November = 11,
    /// December (12).
    December = 12,
}

impl Month {
    /// Construct from a number (1 = January … 12 = December).
    ///
    /// Returns `None` if the value is out of range.
    pub fn from_number(n: u8) -> Option<Self> {
        match n {
            1 => Some(Month::January),
            2 => Some(Month::February),
            3 => Some(Month::March),
            4 => Some(Month::April),
            5 => Some(Month::May),
            6 => Some(Month::June),
            7 => Some(Month::July),
            8 => Some(Month::August),
            9 => Some(Month::September),
            10 => Some(Month::October),
            11 => Some(Month::November),
            12 => Some(Month::December),
            _ => None,
        }
    }

    /// Return the 1-based month number.
    pub fn number(&self) -> u8 {
        *self as u8
    }

    /// Return the three-letter abbreviation (`"Jan"`, `"Feb"`, …).
    pub fn short_name(&self) -> &'static str {
        match self {
            Month::January => "Jan",
            Month::February => "Feb",
            Month::March => "Mar",
            Month::April => "Apr",
            Month::May => "May",
            Month::June => "Jun",
            Month::July => "Jul",
            Month::August => "Aug",
            Month::September => "Sep",
            Month::October => "Oct",
            Month::November => "Nov",
            Month::December => "Dec",
        }
    }

    /// Return the full name (`"January"`, `"February"`, …).
    pub fn long_name(&self) -> &'static str {
        match self {
            Month::January => "January",
            Month::February => "February",
            Month::March => "March",
            Month::April => "April",
            Month::May => "May",
            Month::June => "June",
            Month::July => "July",
            Month::August => "August",
            Month::September => "September",
            Month::October => "October",
            Month::November => "November",
            Month::December => "December",
        }
    }
}

impl std::fmt::Display for Month {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.long_name())
    }
}

impl From<Month> for u8 {
    fn from(m: Month) -> u8 {
        m as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        for n in 1..=12u8 {
            let m = Month::from_number(n).unwrap();
            assert_eq!(m.number(), n);
        }
    }

    #[test]
    fn out_of_range() {
        assert!(Month::from_number(0).is_none());
        assert!(Month::from_number(13).is_none());
    }
}
