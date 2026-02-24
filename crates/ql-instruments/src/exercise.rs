//! Option exercise types.
//!
//! Translates `ql/exercise.hpp`.
//!
//! An `Exercise` defines *when* an option can be exercised.

use ql_time::Date;
use std::fmt;

/// Type of exercise right.
///
/// Corresponds to `QuantLib::Exercise::Type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExerciseType {
    /// Can only be exercised at expiry.
    European,
    /// Can be exercised at any time up to expiry.
    American,
    /// Can be exercised on specific dates.
    Bermudan,
}

/// Exercise specification for an option.
///
/// Corresponds to `QuantLib::Exercise` and its subclasses.
#[derive(Debug, Clone)]
pub struct Exercise {
    /// The exercise type.
    pub exercise_type: ExerciseType,
    /// The exercise date(s).
    ///
    /// - European: single date (the expiry).
    /// - American: two dates (earliest, latest).
    /// - Bermudan: multiple sorted dates.
    dates: Vec<Date>,
}

impl Exercise {
    /// Create a European exercise (single expiry date).
    pub fn european(expiry: Date) -> Self {
        Self {
            exercise_type: ExerciseType::European,
            dates: vec![expiry],
        }
    }

    /// Create an American exercise (earliest to latest).
    pub fn american(earliest: Date, latest: Date) -> Self {
        Self {
            exercise_type: ExerciseType::American,
            dates: vec![earliest, latest],
        }
    }

    /// Create a Bermudan exercise from a set of exercise dates.
    pub fn bermudan(mut dates: Vec<Date>) -> Self {
        dates.sort();
        dates.dedup();
        Self {
            exercise_type: ExerciseType::Bermudan,
            dates,
        }
    }

    /// The last possible exercise date.
    pub fn last_date(&self) -> Date {
        *self.dates.last().expect("exercise has at least one date")
    }

    /// All exercise dates.
    pub fn dates(&self) -> &[Date] {
        &self.dates
    }

    /// The type of exercise.
    pub fn exercise_type(&self) -> ExerciseType {
        self.exercise_type
    }
}

impl fmt::Display for Exercise {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.exercise_type {
            ExerciseType::European => write!(f, "European({})", self.dates[0]),
            ExerciseType::American => {
                write!(f, "American({} – {})", self.dates[0], self.last_date())
            }
            ExerciseType::Bermudan => {
                write!(f, "Bermudan({} dates)", self.dates.len())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn european_exercise() {
        let expiry = Date::from_ymd(2026, 6, 15).unwrap();
        let ex = Exercise::european(expiry);
        assert_eq!(ex.exercise_type(), ExerciseType::European);
        assert_eq!(ex.last_date(), expiry);
        assert_eq!(ex.dates().len(), 1);
    }

    #[test]
    fn american_exercise() {
        let early = Date::from_ymd(2025, 1, 1).unwrap();
        let late = Date::from_ymd(2026, 6, 15).unwrap();
        let ex = Exercise::american(early, late);
        assert_eq!(ex.exercise_type(), ExerciseType::American);
        assert_eq!(ex.last_date(), late);
        assert_eq!(ex.dates().len(), 2);
    }

    #[test]
    fn bermudan_exercise() {
        let dates = vec![
            Date::from_ymd(2025, 6, 15).unwrap(),
            Date::from_ymd(2025, 12, 15).unwrap(),
            Date::from_ymd(2026, 6, 15).unwrap(),
        ];
        let ex = Exercise::bermudan(dates.clone());
        assert_eq!(ex.exercise_type(), ExerciseType::Bermudan);
        assert_eq!(ex.dates().len(), 3);
        assert_eq!(ex.last_date(), dates[2]);
    }
}
