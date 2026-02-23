//! Interest rate with compounding and day-counting conventions
//! (translates `ql/interestrate.hpp`).
//!
//! An `InterestRate` bundles a rate value with a `DayCounter`, a
//! `Compounding` convention, and a `Frequency`.  It can compute compound
//! factors, discount factors, equivalent rates, and implied rates.

use crate::day_counter::DayCounter;
use crate::frequency::Frequency;
use ql_core::{Compounding, Real, Time};
use std::sync::Arc;

/// An interest rate with associated compounding and day-counting conventions.
///
/// Corresponds to `QuantLib::InterestRate`.
#[derive(Debug, Clone)]
pub struct InterestRate {
    rate: Real,
    dc: Arc<dyn DayCounter>,
    compounding: Compounding,
    frequency: Frequency,
}

impl InterestRate {
    /// Create a new interest rate.
    ///
    /// # Arguments
    /// * `rate` — the annual rate as a decimal (e.g. 0.05 = 5%)
    /// * `dc` — day counter for year-fraction calculations
    /// * `compounding` — compounding convention
    /// * `frequency` — compounding frequency (ignored for Simple and Continuous)
    pub fn new(
        rate: Real,
        dc: impl DayCounter + 'static,
        compounding: Compounding,
        frequency: Frequency,
    ) -> Self {
        Self {
            rate,
            dc: Arc::new(dc),
            compounding,
            frequency,
        }
    }

    /// The rate value.
    pub fn rate(&self) -> Real {
        self.rate
    }

    /// The day counter.
    pub fn day_counter(&self) -> &dyn DayCounter {
        &*self.dc
    }

    /// The compounding convention.
    pub fn compounding(&self) -> Compounding {
        self.compounding
    }

    /// The compounding frequency.
    pub fn frequency(&self) -> Frequency {
        self.frequency
    }

    /// Compound factor for a given time period `t` (in years).
    ///
    /// Returns the ratio `P(t) / P(0)` where `P` is the notional value.
    ///
    /// # Panics
    /// Panics if `t < 0` or if `1 + r·t ≤ 0` for Simple compounding.
    pub fn compound_factor_time(&self, t: Time) -> Real {
        assert!(t >= 0.0, "negative time ({t}) not allowed");
        if t == 0.0 {
            return 1.0;
        }
        match self.compounding {
            Compounding::Simple => {
                let factor = 1.0 + self.rate * t;
                assert!(factor > 0.0, "negative compound factor");
                factor
            }
            Compounding::Compounded => {
                let freq = self.freq_value();
                (1.0 + self.rate / freq).powf(freq * t)
            }
            Compounding::Continuous => (self.rate * t).exp(),
            Compounding::SimpleThenCompounded => {
                // Simple up to 1/frequency, then compounded
                let freq = self.freq_value();
                let threshold = 1.0 / freq;
                if t <= threshold {
                    1.0 + self.rate * t
                } else {
                    (1.0 + self.rate / freq).powf(freq * t)
                }
            }
            Compounding::CompoundedThenSimple => {
                let freq = self.freq_value();
                let threshold = 1.0 / freq;
                if t <= threshold {
                    (1.0 + self.rate / freq).powf(freq * t)
                } else {
                    // Compounded for whole periods, simple for the stub
                    let periods = (freq * t).floor();
                    let stub = t - periods / freq;
                    let compounded_part = (1.0 + self.rate / freq).powf(periods);
                    compounded_part * (1.0 + self.rate * stub)
                }
            }
        }
    }

    /// Compound factor between two dates.
    pub fn compound_factor(
        &self,
        d1: crate::date::Date,
        d2: crate::date::Date,
    ) -> Real {
        let t = self.dc.year_fraction(d1, d2);
        self.compound_factor_time(t)
    }

    /// Discount factor for a given time period `t` (in years).
    ///
    /// `discount_factor = 1 / compound_factor`
    pub fn discount_factor_time(&self, t: Time) -> Real {
        1.0 / self.compound_factor_time(t)
    }

    /// Discount factor between two dates.
    pub fn discount_factor(
        &self,
        d1: crate::date::Date,
        d2: crate::date::Date,
    ) -> Real {
        1.0 / self.compound_factor(d1, d2)
    }

    /// Compute the rate equivalent to this one under different conventions.
    ///
    /// Returns a new `InterestRate` with the given compounding and frequency
    /// that produces the same compound factor over the time period `t`.
    pub fn equivalent_rate_time(
        &self,
        comp: Compounding,
        freq: Frequency,
        t: Time,
    ) -> InterestRate {
        Self::implied_rate_time(self.compound_factor_time(t), comp, freq, t)
    }

    /// Implied rate from a compound factor over time `t`.
    ///
    /// Given a `compound_factor` observed over period `t`, compute the
    /// annualized rate under the specified conventions.
    pub fn implied_rate_time(
        compound: Real,
        comp: Compounding,
        freq: Frequency,
        t: Time,
    ) -> InterestRate {
        assert!(compound > 0.0, "compound factor must be positive");
        let r = if t == 0.0 {
            0.0
        } else {
            match comp {
                Compounding::Simple => (compound - 1.0) / t,
                Compounding::Compounded => {
                    let f = freq_value(freq);
                    (compound.powf(1.0 / (f * t)) - 1.0) * f
                }
                Compounding::Continuous => compound.ln() / t,
                Compounding::SimpleThenCompounded => {
                    let f = freq_value(freq);
                    let threshold = 1.0 / f;
                    if t <= threshold {
                        (compound - 1.0) / t
                    } else {
                        (compound.powf(1.0 / (f * t)) - 1.0) * f
                    }
                }
                Compounding::CompoundedThenSimple => {
                    let f = freq_value(freq);
                    let threshold = 1.0 / f;
                    if t <= threshold {
                        (compound.powf(1.0 / (f * t)) - 1.0) * f
                    } else {
                        (compound - 1.0) / t
                    }
                }
            }
        };
        // Use a dummy day counter since we don't have one for the target
        InterestRate {
            rate: r,
            dc: Arc::new(crate::day_counter::Actual365Fixed),
            compounding: comp,
            frequency: freq,
        }
    }

    fn freq_value(&self) -> Real {
        freq_value(self.frequency)
    }
}

fn freq_value(freq: Frequency) -> Real {
    match freq {
        Frequency::NoFrequency | Frequency::Once => 1.0,
        _ => freq.periods_per_year().unwrap_or(1) as Real,
    }
}

impl std::fmt::Display for InterestRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:.4}% {:?} {:?} {}",
            self.rate * 100.0,
            self.compounding,
            self.frequency,
            self.dc.name(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::day_counter::Actual365Fixed;

    #[test]
    fn simple_compound_factor() {
        let ir = InterestRate::new(0.05, Actual365Fixed, Compounding::Simple, Frequency::Annual);
        // 1 + 0.05 * 1 = 1.05
        assert!((ir.compound_factor_time(1.0) - 1.05).abs() < 1e-12);
        // 1 + 0.05 * 2 = 1.10
        assert!((ir.compound_factor_time(2.0) - 1.10).abs() < 1e-12);
    }

    #[test]
    fn compounded_factor() {
        let ir = InterestRate::new(
            0.05,
            Actual365Fixed,
            Compounding::Compounded,
            Frequency::Annual,
        );
        // (1 + 0.05)^1 = 1.05
        assert!((ir.compound_factor_time(1.0) - 1.05).abs() < 1e-12);
        // (1 + 0.05)^2 = 1.1025
        assert!((ir.compound_factor_time(2.0) - 1.1025).abs() < 1e-12);
    }

    #[test]
    fn compounded_semiannual() {
        let ir = InterestRate::new(
            0.10,
            Actual365Fixed,
            Compounding::Compounded,
            Frequency::Semiannual,
        );
        // (1 + 0.10/2)^(2*1) = 1.05^2 = 1.1025
        assert!((ir.compound_factor_time(1.0) - 1.1025).abs() < 1e-12);
    }

    #[test]
    fn continuous_factor() {
        let ir = InterestRate::new(
            0.05,
            Actual365Fixed,
            Compounding::Continuous,
            Frequency::NoFrequency,
        );
        let expected = (0.05_f64).exp();
        assert!((ir.compound_factor_time(1.0) - expected).abs() < 1e-12);
    }

    #[test]
    fn discount_factor() {
        let ir = InterestRate::new(0.05, Actual365Fixed, Compounding::Simple, Frequency::Annual);
        let df = ir.discount_factor_time(1.0);
        assert!((df - 1.0 / 1.05).abs() < 1e-12);
    }

    #[test]
    fn implied_rate_simple() {
        // If compound factor is 1.10 over 2 years with simple compounding,
        // rate = (1.10 - 1) / 2 = 0.05
        let ir = InterestRate::implied_rate_time(
            1.10,
            Compounding::Simple,
            Frequency::Annual,
            2.0,
        );
        assert!((ir.rate() - 0.05).abs() < 1e-12);
    }

    #[test]
    fn implied_rate_continuous() {
        let compound = (0.05_f64 * 3.0).exp();
        let ir = InterestRate::implied_rate_time(
            compound,
            Compounding::Continuous,
            Frequency::NoFrequency,
            3.0,
        );
        assert!((ir.rate() - 0.05).abs() < 1e-12);
    }

    #[test]
    fn equivalent_rate_roundtrip() {
        let ir = InterestRate::new(
            0.05,
            Actual365Fixed,
            Compounding::Compounded,
            Frequency::Annual,
        );
        // Convert to continuous, then back to annual compounded
        let cont = ir.equivalent_rate_time(
            Compounding::Continuous,
            Frequency::NoFrequency,
            1.0,
        );
        let back = cont.equivalent_rate_time(
            Compounding::Compounded,
            Frequency::Annual,
            1.0,
        );
        assert!(
            (back.rate() - 0.05).abs() < 1e-10,
            "got {}",
            back.rate()
        );
    }

    #[test]
    fn zero_time_returns_one() {
        let ir = InterestRate::new(
            0.10,
            Actual365Fixed,
            Compounding::Continuous,
            Frequency::Annual,
        );
        assert!((ir.compound_factor_time(0.0) - 1.0).abs() < 1e-15);
    }

    #[test]
    fn display_format() {
        let ir = InterestRate::new(
            0.05,
            Actual365Fixed,
            Compounding::Continuous,
            Frequency::Annual,
        );
        let s = format!("{ir}");
        assert!(s.contains("5.0000%"));
        assert!(s.contains("Continuous"));
    }
}
