//! Swap index stub types.
//!
//! A `SwapIndex` represents the par-swap rate for a given tenor.  It wraps an
//! `IborIndex` for the floating leg and records fixed-leg conventions.
//!
//! Full implementation (coupling to the swap-rate helper and term structures)
//! will be added in a later phase.

use crate::ibor_index::IborIndex;
use ql_currencies::Currency;
use ql_time::{BusinessDayConvention, DayCounter, Frequency, Period};

/// A par-swap-rate index (e.g. EUR Euribor Swap ISDA Fix A).
///
/// Corresponds to `QuantLib::SwapIndex`.
#[derive(Debug)]
pub struct SwapIndex {
    name: String,
    swap_tenor: Period,
    ibor_index: IborIndex,
    fixed_leg_frequency: Frequency,
    fixed_leg_convention: BusinessDayConvention,
    fixed_leg_day_counter: Box<dyn DayCounter>,
}

impl SwapIndex {
    /// Create a new swap index.
    pub fn new(
        name: impl Into<String>,
        swap_tenor: Period,
        ibor_index: IborIndex,
        fixed_leg_frequency: Frequency,
        fixed_leg_convention: BusinessDayConvention,
        fixed_leg_day_counter: impl DayCounter + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            swap_tenor,
            ibor_index,
            fixed_leg_frequency,
            fixed_leg_convention,
            fixed_leg_day_counter: Box::new(fixed_leg_day_counter),
        }
    }

    /// Index name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Swap tenor (e.g. 10Y).
    pub fn swap_tenor(&self) -> Period {
        self.swap_tenor
    }

    /// The underlying floating-leg IBOR index.
    pub fn ibor_index(&self) -> &IborIndex {
        &self.ibor_index
    }

    /// Fixed-leg payment frequency.
    pub fn fixed_leg_frequency(&self) -> Frequency {
        self.fixed_leg_frequency
    }

    /// Fixed-leg business-day convention.
    pub fn fixed_leg_convention(&self) -> BusinessDayConvention {
        self.fixed_leg_convention
    }

    /// Fixed-leg day counter.
    pub fn fixed_leg_day_counter(&self) -> &dyn DayCounter {
        &*self.fixed_leg_day_counter
    }

    /// Currency — delegates to the underlying IBOR index.
    pub fn currency(&self) -> &'static Currency {
        use crate::interest_rate_index::InterestRateIndex;
        self.ibor_index.currency()
    }
}

// ── Convenience constructors ──────────────────────────────────────────────────

/// Create a EUR Euribor Swap ISDA Fix A (annual fixed, 30/360) stub.
pub fn euribor_swap_isda_fix_a(swap_tenor: Period) -> SwapIndex {
    use crate::ibor::euribor_months;
    use ql_time::Thirty360;
    SwapIndex::new(
        format!("EUR-EuriborSwapIsdaFixA-{swap_tenor}"),
        swap_tenor,
        euribor_months(6),
        Frequency::Annual,
        BusinessDayConvention::ModifiedFollowing,
        Thirty360,
    )
}

/// Create a USD LIBOR Swap ISDA Fix AM (semi-annual fixed, 30/360) stub.
pub fn usd_libor_swap_isda_fix_am(swap_tenor: Period) -> SwapIndex {
    use crate::ibor::usd_libor;
    use ql_time::{Thirty360, TimeUnit};
    SwapIndex::new(
        format!("USD-LiborSwapIsdaFixAm-{swap_tenor}"),
        swap_tenor,
        usd_libor(Period::new(3, TimeUnit::Months)),
        Frequency::Semiannual,
        BusinessDayConvention::ModifiedFollowing,
        Thirty360,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use ql_time::TimeUnit;

    #[test]
    fn euribor_swap_index() {
        let idx = euribor_swap_isda_fix_a(Period::new(10, TimeUnit::Years));
        assert!(idx.name().contains("EuriborSwapIsdaFixA"));
        assert_eq!(idx.swap_tenor(), Period::new(10, TimeUnit::Years));
        assert_eq!(idx.currency().code, "EUR");
        assert_eq!(idx.fixed_leg_frequency(), Frequency::Annual);
    }

    #[test]
    fn usd_swap_index() {
        let idx = usd_libor_swap_isda_fix_am(Period::new(5, TimeUnit::Years));
        assert!(idx.name().contains("LiborSwapIsdaFixAm"));
        assert_eq!(idx.currency().code, "USD");
        assert_eq!(idx.fixed_leg_frequency(), Frequency::Semiannual);
    }
}
