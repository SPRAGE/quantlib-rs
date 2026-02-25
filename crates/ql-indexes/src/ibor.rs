//! Concrete IBOR index factory functions.
//!
//! Provides pre-configured constructors for common interbank indexes:
//! Euribor, USD LIBOR, GBP LIBOR, JPY LIBOR, TIBOR, CDOR, BBSW, STIBOR,
//! NIBOR, CIBOR, WIBOR, PRIBOR, BUBOR, JIBAR.

use crate::ibor_index::IborIndex;
use ql_time::{
    calendars::australia::Australia, calendars::canada::Canada,
    calendars::czech_republic::CzechRepublic, calendars::denmark::Denmark,
    calendars::hungary::Hungary, calendars::japan::Japan, calendars::norway::Norway,
    calendars::poland::Poland, calendars::south_africa::SouthAfrica, calendars::sweden::Sweden,
    calendars::target::Target, calendars::united_kingdom::UnitedKingdomSettlement,
    calendars::united_states::UnitedStatesSettlement, Actual360, Actual365Fixed,
    BusinessDayConvention, Period, TimeUnit,
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

/// Create a TIBOR (Tokyo Interbank Offered Rate) index with the given tenor.
///
/// - Currency: JPY
/// - Calendar: Japan
/// - Day counter: Actual/365 (Fixed)
/// - Fixing days: 2
/// - Convention: Modified Following
/// - End of month: true
pub fn tibor(tenor: Period) -> IborIndex {
    let name = format!("JPY-TIBOR-{tenor}");
    IborIndex::new(
        name,
        tenor,
        2,
        &ql_currencies::currencies::asia::JPY,
        Japan,
        BusinessDayConvention::ModifiedFollowing,
        true,
        Actual365Fixed,
    )
}

/// Create a CDOR (Canadian Dollar Offered Rate) index with the given tenor.
///
/// - Currency: CAD
/// - Calendar: Canada
/// - Day counter: Actual/365 (Fixed)
/// - Fixing days: 0
/// - Convention: Modified Following
/// - End of month: true
pub fn cdor(tenor: Period) -> IborIndex {
    let name = format!("CAD-CDOR-{tenor}");
    IborIndex::new(
        name,
        tenor,
        0,
        &ql_currencies::currencies::america::CAD,
        Canada,
        BusinessDayConvention::ModifiedFollowing,
        true,
        Actual365Fixed,
    )
}

/// Create a BBSW (Bank Bill Swap Rate) index with the given tenor.
///
/// - Currency: AUD
/// - Calendar: Australia
/// - Day counter: Actual/365 (Fixed)
/// - Fixing days: 0
/// - Convention: Modified Following
/// - End of month: true
pub fn bbsw(tenor: Period) -> IborIndex {
    let name = format!("AUD-BBSW-{tenor}");
    IborIndex::new(
        name,
        tenor,
        0,
        &ql_currencies::currencies::oceania::AUD,
        Australia,
        BusinessDayConvention::ModifiedFollowing,
        true,
        Actual365Fixed,
    )
}

/// Create a STIBOR (Stockholm Interbank Offered Rate) index with the given tenor.
///
/// - Currency: SEK
/// - Calendar: Sweden
/// - Day counter: Actual/360
/// - Fixing days: 2
/// - Convention: Modified Following
/// - End of month: true
pub fn stibor(tenor: Period) -> IborIndex {
    let name = format!("SEK-STIBOR-{tenor}");
    IborIndex::new(
        name,
        tenor,
        2,
        &ql_currencies::currencies::europe::SEK,
        Sweden,
        BusinessDayConvention::ModifiedFollowing,
        true,
        Actual360,
    )
}

/// Create a NIBOR (Norwegian Interbank Offered Rate) index with the given tenor.
///
/// - Currency: NOK
/// - Calendar: Norway
/// - Day counter: Actual/360
/// - Fixing days: 2
/// - Convention: Modified Following
/// - End of month: true
pub fn nibor(tenor: Period) -> IborIndex {
    let name = format!("NOK-NIBOR-{tenor}");
    IborIndex::new(
        name,
        tenor,
        2,
        &ql_currencies::currencies::europe::NOK,
        Norway,
        BusinessDayConvention::ModifiedFollowing,
        true,
        Actual360,
    )
}

/// Create a CIBOR (Copenhagen Interbank Offered Rate) index with the given tenor.
///
/// - Currency: DKK
/// - Calendar: Denmark
/// - Day counter: Actual/360
/// - Fixing days: 2
/// - Convention: Modified Following
/// - End of month: true
pub fn cibor(tenor: Period) -> IborIndex {
    let name = format!("DKK-CIBOR-{tenor}");
    IborIndex::new(
        name,
        tenor,
        2,
        &ql_currencies::currencies::europe::DKK,
        Denmark,
        BusinessDayConvention::ModifiedFollowing,
        true,
        Actual360,
    )
}

/// Create a WIBOR (Warsaw Interbank Offered Rate) index with the given tenor.
///
/// - Currency: PLN
/// - Calendar: Poland
/// - Day counter: Actual/365 (Fixed)
/// - Fixing days: 2
/// - Convention: Modified Following
/// - End of month: true
pub fn wibor(tenor: Period) -> IborIndex {
    let name = format!("PLN-WIBOR-{tenor}");
    IborIndex::new(
        name,
        tenor,
        2,
        &ql_currencies::currencies::europe::PLN,
        Poland,
        BusinessDayConvention::ModifiedFollowing,
        true,
        Actual365Fixed,
    )
}

/// Create a PRIBOR (Prague Interbank Offered Rate) index with the given tenor.
///
/// - Currency: CZK
/// - Calendar: Czech Republic
/// - Day counter: Actual/360
/// - Fixing days: 2
/// - Convention: Modified Following
/// - End of month: true
pub fn pribor(tenor: Period) -> IborIndex {
    let name = format!("CZK-PRIBOR-{tenor}");
    IborIndex::new(
        name,
        tenor,
        2,
        &ql_currencies::currencies::europe::CZK,
        CzechRepublic,
        BusinessDayConvention::ModifiedFollowing,
        true,
        Actual360,
    )
}

/// Create a BUBOR (Budapest Interbank Offered Rate) index with the given tenor.
///
/// - Currency: HUF
/// - Calendar: Hungary
/// - Day counter: Actual/360
/// - Fixing days: 2
/// - Convention: Modified Following
/// - End of month: true
pub fn bubor(tenor: Period) -> IborIndex {
    let name = format!("HUF-BUBOR-{tenor}");
    IborIndex::new(
        name,
        tenor,
        2,
        &ql_currencies::currencies::europe::HUF,
        Hungary,
        BusinessDayConvention::ModifiedFollowing,
        true,
        Actual360,
    )
}

/// Create a JIBAR (Johannesburg Interbank Agreed Rate) index with the given tenor.
///
/// - Currency: ZAR
/// - Calendar: South Africa
/// - Day counter: Actual/365 (Fixed)
/// - Fixing days: 0
/// - Convention: Modified Following
/// - End of month: true
pub fn jibar(tenor: Period) -> IborIndex {
    let name = format!("ZAR-JIBAR-{tenor}");
    IborIndex::new(
        name,
        tenor,
        0,
        &ql_currencies::currencies::africa::ZAR,
        SouthAfrica,
        BusinessDayConvention::ModifiedFollowing,
        true,
        Actual365Fixed,
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

    #[test]
    fn tibor_3m() {
        let idx = tibor(Period::new(3, TimeUnit::Months));
        assert!(idx.name().contains("TIBOR"));
        assert_eq!(idx.currency().code, "JPY");
        assert_eq!(idx.fixing_days(), 2);
    }

    #[test]
    fn cdor_3m() {
        let idx = cdor(Period::new(3, TimeUnit::Months));
        assert!(idx.name().contains("CDOR"));
        assert_eq!(idx.currency().code, "CAD");
        assert_eq!(idx.fixing_days(), 0);
    }

    #[test]
    fn bbsw_3m() {
        let idx = bbsw(Period::new(3, TimeUnit::Months));
        assert!(idx.name().contains("BBSW"));
        assert_eq!(idx.currency().code, "AUD");
        assert_eq!(idx.fixing_days(), 0);
    }

    #[test]
    fn stibor_3m() {
        let idx = stibor(Period::new(3, TimeUnit::Months));
        assert!(idx.name().contains("STIBOR"));
        assert_eq!(idx.currency().code, "SEK");
        assert_eq!(idx.fixing_days(), 2);
    }

    #[test]
    fn nibor_3m() {
        let idx = nibor(Period::new(3, TimeUnit::Months));
        assert!(idx.name().contains("NIBOR"));
        assert_eq!(idx.currency().code, "NOK");
        assert_eq!(idx.fixing_days(), 2);
    }

    #[test]
    fn cibor_3m() {
        let idx = cibor(Period::new(3, TimeUnit::Months));
        assert!(idx.name().contains("CIBOR"));
        assert_eq!(idx.currency().code, "DKK");
        assert_eq!(idx.fixing_days(), 2);
    }

    #[test]
    fn wibor_3m() {
        let idx = wibor(Period::new(3, TimeUnit::Months));
        assert!(idx.name().contains("WIBOR"));
        assert_eq!(idx.currency().code, "PLN");
        assert_eq!(idx.fixing_days(), 2);
    }

    #[test]
    fn pribor_3m() {
        let idx = pribor(Period::new(3, TimeUnit::Months));
        assert!(idx.name().contains("PRIBOR"));
        assert_eq!(idx.currency().code, "CZK");
        assert_eq!(idx.fixing_days(), 2);
    }

    #[test]
    fn bubor_3m() {
        let idx = bubor(Period::new(3, TimeUnit::Months));
        assert!(idx.name().contains("BUBOR"));
        assert_eq!(idx.currency().code, "HUF");
        assert_eq!(idx.fixing_days(), 2);
    }

    #[test]
    fn jibar_3m() {
        let idx = jibar(Period::new(3, TimeUnit::Months));
        assert!(idx.name().contains("JIBAR"));
        assert_eq!(idx.currency().code, "ZAR");
        assert_eq!(idx.fixing_days(), 0);
    }
}
