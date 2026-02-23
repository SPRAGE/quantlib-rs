//! Inflation index stub types.
//!
//! Placeholder types that record the signatures needed for inflation-index
//! support.  Full implementation (lag conventions, fixing interpolation,
//! seasonal adjustments) will come in a later phase.

use ql_currencies::currency::Currency;
use ql_time::{Frequency, Period};

/// Stub for a zero-coupon inflation index (e.g. US CPI, UK RPI, EU HICP).
///
/// Records headline metadata; actual fixing retrieval and seasonal-adjustment
/// logic will be added when term-structure scaffolding is in place.
#[derive(Debug, Clone)]
pub struct ZeroInflationIndex {
    name: String,
    family_name: String,
    currency: Currency,
    frequency: Frequency,
    availability_lag: Period,
    interpolated: bool,
    revised: bool,
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
}

/// Stub for a year-on-year inflation index.
///
/// Wraps a `ZeroInflationIndex` and computes year-on-year returns once the
/// full fixing infrastructure is available.
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

// ── Convenience constructors ──────────────────────────────────────────────────

/// Create a US CPI zero-coupon inflation index stub.
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

/// Create a UK RPI zero-coupon inflation index stub.
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

/// Create a EU HICP zero-coupon inflation index stub.
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
        assert_eq!(idx.name(), "USCPI");
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
        assert_eq!(yoy.name(), "USCPI");
        assert!(yoy.ratio());
        assert_eq!(
            yoy.underlying().availability_lag(),
            Period::new(1, TimeUnit::Months)
        );
    }
}
