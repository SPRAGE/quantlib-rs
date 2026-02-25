//! Concrete overnight index factory functions.
//!
//! Provides pre-configured constructors for common overnight indexes:
//! SOFR, €STR (ESTR), SONIA, TONA, AONIA, CORRA, SARON.

use crate::overnight_index::OvernightIndex;
use ql_time::{
    calendars::australia::Australia, calendars::canada::Canada, calendars::japan::Japan,
    calendars::switzerland::Switzerland, calendars::target::Target,
    calendars::united_kingdom::UnitedKingdomSettlement,
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

/// Create a TONA (Tokyo Overnight Average Rate) index.
///
/// Also known as TONAR. The Bank of Japan overnight call rate.
///
/// - Currency: JPY
/// - Calendar: Japan
/// - Day counter: Actual/365 (Fixed)
/// - Fixing days: 0
pub fn tona() -> OvernightIndex {
    OvernightIndex::new(
        "JPY-TONA".to_string(),
        0,
        &ql_currencies::currencies::asia::JPY,
        Japan,
        Actual365Fixed,
    )
}

/// Create an AONIA (AUD Overnight Index Average) index.
///
/// The Reserve Bank of Australia interbank overnight rate.
///
/// - Currency: AUD
/// - Calendar: Australia
/// - Day counter: Actual/365 (Fixed)
/// - Fixing days: 0
pub fn aonia() -> OvernightIndex {
    OvernightIndex::new(
        "AUD-AONIA".to_string(),
        0,
        &ql_currencies::currencies::oceania::AUD,
        Australia,
        Actual365Fixed,
    )
}

/// Create a CORRA (Canadian Overnight Repo Rate Average) index.
///
/// - Currency: CAD
/// - Calendar: Canada
/// - Day counter: Actual/365 (Fixed)
/// - Fixing days: 0
pub fn corra() -> OvernightIndex {
    OvernightIndex::new(
        "CAD-CORRA".to_string(),
        0,
        &ql_currencies::currencies::america::CAD,
        Canada,
        Actual365Fixed,
    )
}

/// Create a SARON (Swiss Average Rate Overnight) index.
///
/// - Currency: CHF
/// - Calendar: Switzerland
/// - Day counter: Actual/360
/// - Fixing days: 0
pub fn saron() -> OvernightIndex {
    OvernightIndex::new(
        "CHF-SARON".to_string(),
        0,
        &ql_currencies::currencies::europe::CHF,
        Switzerland,
        Actual360,
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

    #[test]
    fn tona_properties() {
        let idx = tona();
        assert_eq!(idx.name(), "JPY-TONA");
        assert_eq!(idx.currency().code, "JPY");
        assert_eq!(idx.fixing_days(), 0);
        assert_eq!(idx.tenor(), Period::new(1, TimeUnit::Days));
    }

    #[test]
    fn aonia_properties() {
        let idx = aonia();
        assert_eq!(idx.name(), "AUD-AONIA");
        assert_eq!(idx.currency().code, "AUD");
        assert_eq!(idx.fixing_days(), 0);
    }

    #[test]
    fn corra_properties() {
        let idx = corra();
        assert_eq!(idx.name(), "CAD-CORRA");
        assert_eq!(idx.currency().code, "CAD");
        assert_eq!(idx.fixing_days(), 0);
    }

    #[test]
    fn saron_properties() {
        let idx = saron();
        assert_eq!(idx.name(), "CHF-SARON");
        assert_eq!(idx.currency().code, "CHF");
        assert_eq!(idx.fixing_days(), 0);
    }
}
