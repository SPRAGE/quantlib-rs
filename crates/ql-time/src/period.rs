//! `Period` — a time span expressed in a [`TimeUnit`] (translates
//! `ql/time/period.hpp`).

use crate::frequency::Frequency;
use crate::time_unit::TimeUnit;
use ql_core::errors::{Error, Result};

/// A time span made up of an integer length and a [`TimeUnit`].
///
/// Corresponds to `QuantLib::Period`.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Period {
    /// Number of units.
    pub length: i32,
    /// The unit of time.
    pub unit: TimeUnit,
}

impl Period {
    /// Create a new period.
    pub fn new(length: i32, unit: TimeUnit) -> Self {
        Self { length, unit }
    }

    /// Construct a `Period` from a [`Frequency`].
    ///
    /// # Errors
    /// Returns an error for `NoFrequency` and `OtherFrequency`.
    pub fn from_frequency(freq: Frequency) -> Result<Self> {
        match freq {
            Frequency::NoFrequency | Frequency::OtherFrequency => Err(Error::InvalidArgument(
                format!("cannot convert {freq} to a Period"),
            )),
            Frequency::Once => Ok(Period::new(0, TimeUnit::Years)),
            Frequency::Annual => Ok(Period::new(1, TimeUnit::Years)),
            Frequency::Semiannual => Ok(Period::new(6, TimeUnit::Months)),
            Frequency::EveryFourthMonth => Ok(Period::new(4, TimeUnit::Months)),
            Frequency::Quarterly => Ok(Period::new(3, TimeUnit::Months)),
            Frequency::Bimonthly => Ok(Period::new(2, TimeUnit::Months)),
            Frequency::Monthly => Ok(Period::new(1, TimeUnit::Months)),
            Frequency::EveryFourthWeek => Ok(Period::new(4, TimeUnit::Weeks)),
            Frequency::Biweekly => Ok(Period::new(2, TimeUnit::Weeks)),
            Frequency::Weekly => Ok(Period::new(1, TimeUnit::Weeks)),
            Frequency::Daily => Ok(Period::new(1, TimeUnit::Days)),
        }
    }

    /// Negate the period (reverse direction).
    pub fn negated(self) -> Self {
        Self {
            length: -self.length,
            unit: self.unit,
        }
    }

    /// Normalise the period by converting weeks to days and years to months.
    ///
    /// Returns a new period in a canonical form (Days or Months).
    pub fn normalized(self) -> Self {
        let (length, unit) = match self.unit {
            TimeUnit::Days => (self.length, TimeUnit::Days),
            TimeUnit::Weeks => (self.length * 7, TimeUnit::Days),
            TimeUnit::Months => {
                let y = self.length / 12;
                let m = self.length % 12;
                if m == 0 {
                    (y, TimeUnit::Years)
                } else {
                    (self.length, TimeUnit::Months)
                }
            }
            TimeUnit::Years => (self.length, TimeUnit::Years),
            _ => (self.length, self.unit),
        };
        Period { length, unit }
    }
}

impl std::ops::Neg for Period {
    type Output = Self;
    fn neg(self) -> Self {
        self.negated()
    }
}

impl std::ops::Mul<i32> for Period {
    type Output = Self;
    fn mul(self, rhs: i32) -> Self {
        Period {
            length: self.length * rhs,
            unit: self.unit,
        }
    }
}

impl std::ops::Mul<Period> for i32 {
    type Output = Period;
    fn mul(self, rhs: Period) -> Period {
        rhs * self
    }
}

impl std::fmt::Display for Period {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let abbr = match self.unit {
            TimeUnit::Days => "D",
            TimeUnit::Weeks => "W",
            TimeUnit::Months => "M",
            TimeUnit::Years => "Y",
            TimeUnit::Hours => "h",
            TimeUnit::Minutes => "min",
            TimeUnit::Seconds => "s",
            TimeUnit::Milliseconds => "ms",
            TimeUnit::Microseconds => "us",
        };
        write!(f, "{}{abbr}", self.length)
    }
}

impl std::fmt::Debug for Period {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Period({self})")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display() {
        assert_eq!(Period::new(3, TimeUnit::Months).to_string(), "3M");
        assert_eq!(Period::new(1, TimeUnit::Years).to_string(), "1Y");
        assert_eq!(Period::new(-6, TimeUnit::Months).to_string(), "-6M");
    }

    #[test]
    fn from_frequency() {
        assert_eq!(
            Period::from_frequency(Frequency::Quarterly).unwrap(),
            Period::new(3, TimeUnit::Months)
        );
        assert!(Period::from_frequency(Frequency::NoFrequency).is_err());
    }
}
