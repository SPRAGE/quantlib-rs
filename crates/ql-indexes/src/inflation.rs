//! Inflation index types (translates `ql/indexes/inflationindex.hpp`).
//!
//! Provides `ZeroInflationIndex` (a CPI-like index) and `YoYInflationIndex`
//! (year-on-year view).  Both implement the `Index` trait and support
//! storing / retrieving historical fixings.

use crate::index::{FixingStore, Index};
use ql_core::Real;
use ql_currencies::currency::Currency;
use ql_time::{Date, Frequency, NullCalendar, Period};

// ── ZeroInflationIndex ────────────────────────────────────────────────────────

/// A zero-coupon (CPI-style) inflation index (e.g. US CPI, UK RPI, EU HICP).
///
/// Corresponds to `QuantLib::ZeroInflationIndex`.
#[derive(Debug, Clone)]
pub struct ZeroInflationIndex {
    name: String,
    family_name: String,
    currency: Currency,
    frequency: Frequency,
    availability_lag: Period,
    interpolated: bool,
    revised: bool,
    fixings: FixingStore,
}

impl ZeroInflationIndex {
    /// Create a new zero-coupon inflation index.
    pub fn new(
        name: impl Into<String>,
        family_name: impl Into<String>,
        currency: Currency,
        frequency: Frequency,
        availability_lag: Period,
        interpolated: bool,
        revised: bool,
    ) -> Self {
        Self {
            name: name.into(),
            family_name: family_name.into(),
            currency,
            frequency,
            availability_lag,
            interpolated,
            revised,
            fixings: FixingStore::new(),
        }
    }

    /// Index name (e.g. "USCPI").
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Family name (e.g. "CPI").
    pub fn family_name(&self) -> &str {
        &self.family_name
    }

    /// Index currency.
    pub fn currency(&self) -> &Currency {
        &self.currency
    }

    /// Publication frequency (typically Monthly).
    pub fn frequency(&self) -> Frequency {
        self.frequency
    }

    /// Lag between the reference period and the fixing availability date.
    pub fn availability_lag(&self) -> Period {
        self.availability_lag
    }

    /// Whether the index uses interpolated fixings.
    pub fn interpolated(&self) -> bool {
        self.interpolated
    }

    /// Whether published fixings may be revised.
    pub fn revised(&self) -> bool {
        self.revised
    }

    /// Add a historical fixing for a given reference date.
    pub fn add_fixing_value(&self, date: Date, value: Real) {
        self.fixings.add(date, value);
    }
}

impl Index for ZeroInflationIndex {
    fn name(&self) -> &str {
        &self.name
    }

    fn fixing_calendar(&self) -> &dyn ql_time::Calendar {
        // Inflation indexes are published without a specific business calendar –
        // any date is a valid fixing reference date.
        &NullCalendar
    }

    fn is_valid_fixing_date(&self, _date: Date) -> bool {
        true // all dates valid for inflation
    }

    fn fixing(&self, date: Date, _force_forecast: bool) -> ql_core::errors::Result<Real> {
        self.fixings.get(date).ok_or_else(|| {
            ql_core::errors::Error::Runtime(format!("missing {} fixing for {}", self.name, date))
        })
    }

    fn fixing_store(&self) -> &FixingStore {
        &self.fixings
    }
}

// ── YoYInflationIndex ─────────────────────────────────────────────────────────

/// A year-on-year inflation index.
///
/// Wraps a `ZeroInflationIndex` and computes year-on-year returns.
/// Corresponds to `QuantLib::YoYInflationIndex`.
#[derive(Debug, Clone)]
pub struct YoYInflationIndex {
    underlying: ZeroInflationIndex,
    ratio: bool,
}

impl YoYInflationIndex {
    /// Create a new year-on-year inflation index.
    ///
    /// If `ratio` is true the YoY rate is `I(t)/I(t-1) - 1`; otherwise it is
    /// the raw YoY difference (less common).
    pub fn new(underlying: ZeroInflationIndex, ratio: bool) -> Self {
        Self { underlying, ratio }
    }

    /// The underlying zero-coupon inflation index.
    pub fn underlying(&self) -> &ZeroInflationIndex {
        &self.underlying
    }

    /// Whether the YoY rate is computed as a ratio.
    pub fn ratio(&self) -> bool {
        self.ratio
    }

    /// Delegate name from underlying.
    pub fn name(&self) -> &str {
        self.underlying.name()
    }
}

impl Index for YoYInflationIndex {
    fn name(&self) -> &str {
        self.underlying.name()
    }

    fn fixing_calendar(&self) -> &dyn ql_time::Calendar {
        self.underlying.fixing_calendar()
    }

    fn is_valid_fixing_date(&self, date: Date) -> bool {
        self.underlying.is_valid_fixing_date(date)
    }

    fn fixing(&self, date: Date, force_forecast: bool) -> ql_core::errors::Result<Real> {
        let current = self.underlying.fixing(date, force_forecast)?;
        // Look up the same date one year ago
        let prev_date = date
            .advance(-1, ql_time::TimeUnit::Years)
            .map_err(|e| ql_core::errors::Error::Runtime(format!("{e}")))?;
        let previous = self.underlying.fixing(prev_date, force_forecast)?;
        if self.ratio {
            Ok(current / previous - 1.0)
        } else {
            Ok(current - previous)
        }
    }

    fn fixing_store(&self) -> &FixingStore {
        self.underlying.fixing_store()
    }
}

// ── Convenience constructors ──────────────────────────────────────────────────

/// Create a US CPI zero-coupon inflation index.
pub fn us_cpi() -> ZeroInflationIndex {
    use ql_time::TimeUnit;
    ZeroInflationIndex::new(
        "USCPI",
        "CPI",
        ql_currencies::currencies::america::USD.clone(),
        Frequency::Monthly,
        Period::new(1, TimeUnit::Months),
        false,
        true,
    )
}

/// Create a UK RPI zero-coupon inflation index.
pub fn uk_rpi() -> ZeroInflationIndex {
    use ql_time::TimeUnit;
    ZeroInflationIndex::new(
        "UKRPI",
        "RPI",
        ql_currencies::currencies::europe::GBP.clone(),
        Frequency::Monthly,
        Period::new(1, TimeUnit::Months),
        false,
        true,
    )
}

/// Create a EU HICP zero-coupon inflation index.
pub fn eu_hicp() -> ZeroInflationIndex {
    use ql_time::TimeUnit;
    ZeroInflationIndex::new(
        "EUHICP",
        "HICP",
        ql_currencies::currencies::europe::EUR.clone(),
        Frequency::Monthly,
        Period::new(1, TimeUnit::Months),
        false,
        true,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use ql_time::TimeUnit;

    #[test]
    fn us_cpi_properties() {
        let idx = us_cpi();
        assert_eq!(Index::name(&idx), "USCPI");
        assert_eq!(idx.family_name(), "CPI");
        assert_eq!(idx.currency().code, "USD");
        assert_eq!(idx.frequency(), Frequency::Monthly);
        assert!(!idx.interpolated());
        assert!(idx.revised());
    }

    #[test]
    fn yoy_wraps_zero_coupon() {
        let zero = us_cpi();
        let yoy = YoYInflationIndex::new(zero, true);
        assert_eq!(Index::name(&yoy), "USCPI");
        assert!(yoy.ratio());
        assert_eq!(
            yoy.underlying().availability_lag(),
            Period::new(1, TimeUnit::Months)
        );
    }

    #[test]
    fn zero_inflation_fixing_round_trip() {
        let idx = us_cpi();
        let d = Date::from_ymd(2024, 1, 1).unwrap();
        idx.add_fixing_value(d, 308.417);
        let fixing = idx.fixing(d, false).unwrap();
        assert!((fixing - 308.417).abs() < 1e-10);
    }

    #[test]
    fn zero_inflation_missing_fixing_error() {
        let idx = us_cpi();
        let d = Date::from_ymd(2024, 6, 1).unwrap();
        assert!(idx.fixing(d, false).is_err());
    }

    #[test]
    fn yoy_index_computes_ratio() {
        let idx = us_cpi();
        let d0 = Date::from_ymd(2023, 1, 1).unwrap();
        let d1 = Date::from_ymd(2024, 1, 1).unwrap();
        idx.add_fixing_value(d0, 300.0);
        idx.add_fixing_value(d1, 309.0);
        let yoy = YoYInflationIndex::new(idx, true);
        let rate = yoy.fixing(d1, false).unwrap();
        assert!((rate - 0.03).abs() < 1e-10); // 309/300 - 1 = 0.03
    }

    #[test]
    fn yoy_index_computes_difference() {
        let idx = us_cpi();
        let d0 = Date::from_ymd(2023, 1, 1).unwrap();
        let d1 = Date::from_ymd(2024, 1, 1).unwrap();
        idx.add_fixing_value(d0, 300.0);
        idx.add_fixing_value(d1, 309.0);
        let yoy = YoYInflationIndex::new(idx, false);
        let diff = yoy.fixing(d1, false).unwrap();
        assert!((diff - 9.0).abs() < 1e-10);
    }
}
