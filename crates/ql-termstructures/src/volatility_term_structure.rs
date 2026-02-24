//! `VolatilityTermStructure` — base trait for volatility term structures
//! (translates `ql/termstructures/voltermstructure.hpp`).
//!
//! Extends `TermStructure` with a business-day convention and minimum/maximum
//! strike range.

use crate::term_structure::TermStructure;
use ql_core::{Real, Time};
use ql_time::BusinessDayConvention;

/// Base trait for all volatility term structures.
///
/// Corresponds to `QuantLib::VolatilityTermStructure`.
pub trait VolatilityTermStructure: TermStructure {
    /// The business-day convention used for option-expiry adjustments.
    fn business_day_convention(&self) -> BusinessDayConvention {
        BusinessDayConvention::Following
    }

    /// The minimum strike for which the term structure is defined.
    fn min_strike(&self) -> Real;

    /// The maximum strike for which the term structure is defined.
    fn max_strike(&self) -> Real;

    /// Convert an option tenor to a time fraction, adjusting the expiry date
    /// by the calendar and business-day convention.
    fn option_date_from_tenor(&self, _t: Time) -> Real {
        // Default: t is already a year fraction
        _t
    }
}
