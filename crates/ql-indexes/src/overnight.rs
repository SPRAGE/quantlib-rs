//! Concrete overnight index factory functions.
//!
//! Provides pre-configured constructors for common overnight indexes:
//! SOFR, €STR (ESTR), and SONIA.

use crate::overnight_index::OvernightIndex;
use ql_time::{
    calendars::target::Target, calendars::united_kingdom::UnitedKingdomSettlement,
    calendars::united_states::UnitedStatesSettlement, Actual360, Actual365Fixed,
};

/// Create a SOFR (Secured Overnight Financing Rate) index.
///
/// - Currency: USD
/// - Calendar: US (settlement)
/// - Day counter: Actual/360
/// - Fixing days: 0
pub fn sofr() -> OvernightIndex {
    OvernightIndex::new(
        "USD-SOFR".to_string(),
        0,
        &ql_currencies::currencies::america::USD,
        UnitedStatesSettlement,
        Actual360,
    )
}

/// Create an €STR (Euro Short-Term Rate) index.
///
/// - Currency: EUR
/// - Calendar: TARGET
/// - Day counter: Actual/360
/// - Fixing days: 0
pub fn estr() -> OvernightIndex {
    OvernightIndex::new(
        "EUR-ESTR".to_string(),
        0,
        &ql_currencies::currencies::europe::EUR,
        Target,
        Actual360,
    )
}

/// Create a SONIA (Sterling Overnight Index Average) index.
///
/// - Currency: GBP
/// - Calendar: UK (settlement)
/// - Day counter: Actual/365 (Fixed)
/// - Fixing days: 0
pub fn sonia() -> OvernightIndex {
    OvernightIndex::new(
        "GBP-SONIA".to_string(),
        0,
        &ql_currencies::currencies::europe::GBP,
        UnitedKingdomSettlement,
        Actual365Fixed,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::Index;
    use crate::interest_rate_index::InterestRateIndex;
    use ql_time::{Frequency, Period, TimeUnit};

    #[test]
    fn sofr_properties() {
        let idx = sofr();
        assert_eq!(idx.name(), "USD-SOFR");
        assert_eq!(idx.currency().code, "USD");
        assert_eq!(idx.fixing_days(), 0);
        assert_eq!(idx.tenor(), Period::new(1, TimeUnit::Days));
        assert_eq!(idx.frequency(), Frequency::Daily);
    }

    #[test]
    fn estr_properties() {
        let idx = estr();
        assert_eq!(idx.name(), "EUR-ESTR");
        assert_eq!(idx.currency().code, "EUR");
        assert_eq!(idx.fixing_days(), 0);
    }

    #[test]
    fn sonia_properties() {
        let idx = sonia();
        assert_eq!(idx.name(), "GBP-SONIA");
        assert_eq!(idx.currency().code, "GBP");
        assert_eq!(idx.fixing_days(), 0);
    }
}
