//! Concrete IBOR index factory functions.
//!
//! Provides pre-configured constructors for common interbank indexes:
//! Euribor, USD LIBOR, GBP LIBOR, JPY LIBOR.

use crate::ibor_index::IborIndex;
use ql_time::{
    Actual360, Actual365Fixed, BusinessDayConvention, Period, TimeUnit,
    calendars::target::Target,
    calendars::united_kingdom::UnitedKingdomSettlement,
    calendars::united_states::UnitedStatesSettlement,
    calendars::japan::Japan,
};

/// Create a Euribor index with the given tenor.
///
/// - Currency: EUR
/// - Calendar: TARGET
/// - Day counter: Actual/360
/// - Fixing days: 2
/// - Convention: Modified Following
/// - End of month: true
pub fn euribor(tenor: Period) -> IborIndex {
    let name = format!("EUR-Euribor-{tenor}");
    IborIndex::new(
        name,
        tenor,
        2,
        &ql_currencies::currencies::europe::EUR,
        Target,
        BusinessDayConvention::ModifiedFollowing,
        true,
        Actual360,
    )
}

/// Create a Euribor index with the given number of months.
pub fn euribor_months(months: i32) -> IborIndex {
    euribor(Period::new(months, TimeUnit::Months))
}

/// Create a USD LIBOR index with the given tenor.
///
/// - Currency: USD
/// - Calendar: US (settlement) + UK
/// - Day counter: Actual/360
/// - Fixing days: 2
/// - Convention: Modified Following
/// - End of month: true
pub fn usd_libor(tenor: Period) -> IborIndex {
    let name = format!("USD-LIBOR-{tenor}");
    IborIndex::new(
        name,
        tenor,
        2,
        &ql_currencies::currencies::america::USD,
        UnitedStatesSettlement,
        BusinessDayConvention::ModifiedFollowing,
        true,
        Actual360,
    )
}

/// Create a GBP LIBOR index with the given tenor.
///
/// - Currency: GBP
/// - Calendar: UK
/// - Day counter: Actual/365 (Fixed)
/// - Fixing days: 0
/// - Convention: Modified Following
/// - End of month: true
pub fn gbp_libor(tenor: Period) -> IborIndex {
    let name = format!("GBP-LIBOR-{tenor}");
    IborIndex::new(
        name,
        tenor,
        0,
        &ql_currencies::currencies::europe::GBP,
        UnitedKingdomSettlement,
        BusinessDayConvention::ModifiedFollowing,
        true,
        Actual365Fixed,
    )
}

/// Create a JPY LIBOR index with the given tenor.
///
/// - Currency: JPY
/// - Calendar: Japan
/// - Day counter: Actual/360
/// - Fixing days: 2
/// - Convention: Modified Following
/// - End of month: true
pub fn jpy_libor(tenor: Period) -> IborIndex {
    let name = format!("JPY-LIBOR-{tenor}");
    IborIndex::new(
        name,
        tenor,
        2,
        &ql_currencies::currencies::asia::JPY,
        Japan,
        BusinessDayConvention::ModifiedFollowing,
        true,
        Actual360,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::Index;
    use crate::interest_rate_index::InterestRateIndex;

    #[test]
    fn euribor_6m() {
        let idx = euribor_months(6);
        assert!(idx.name().contains("Euribor"));
        assert_eq!(idx.currency().code, "EUR");
        assert_eq!(idx.tenor(), Period::new(6, TimeUnit::Months));
        assert_eq!(idx.fixing_days(), 2);
        assert!(idx.end_of_month());
    }

    #[test]
    fn usd_libor_3m() {
        let idx = usd_libor(Period::new(3, TimeUnit::Months));
        assert!(idx.name().contains("USD-LIBOR"));
        assert_eq!(idx.currency().code, "USD");
        assert_eq!(idx.fixing_days(), 2);
    }

    #[test]
    fn gbp_libor_6m() {
        let idx = gbp_libor(Period::new(6, TimeUnit::Months));
        assert!(idx.name().contains("GBP-LIBOR"));
        assert_eq!(idx.currency().code, "GBP");
        // GBP LIBOR has 0 fixing days
        assert_eq!(idx.fixing_days(), 0);
    }

    #[test]
    fn jpy_libor_3m() {
        let idx = jpy_libor(Period::new(3, TimeUnit::Months));
        assert!(idx.name().contains("JPY-LIBOR"));
        assert_eq!(idx.currency().code, "JPY");
    }
}
