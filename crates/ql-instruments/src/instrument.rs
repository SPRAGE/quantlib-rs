//! `Instrument` base trait.
//!
//! Translates `ql/instrument.hpp`.
//!
//! An `Instrument` is a financial product that can be priced. In QuantLib C++,
//! `Instrument` extends `LazyObject` (observer pattern). In Rust we use a
//! simpler trait-based approach: concrete instruments hold their market data
//! and compute results on demand.

use ql_core::{errors::Result, Real};
use ql_time::Date;
use std::collections::HashMap;

/// Results of pricing an instrument.
///
/// Contains the NPV and optionally additional named results
/// (e.g. "delta", "gamma", "theta").
#[derive(Debug, Clone, Default)]
pub struct PricingResults {
    /// Net present value.
    pub npv: Real,
    /// Error estimate (e.g. from MC simulation).
    pub error_estimate: Option<Real>,
    /// Additional named results.
    pub additional_results: HashMap<String, Real>,
}

impl PricingResults {
    /// Create pricing results with just an NPV.
    pub fn from_npv(npv: Real) -> Self {
        Self {
            npv,
            error_estimate: None,
            additional_results: HashMap::new(),
        }
    }

    /// Add a named result.
    pub fn with_result(mut self, key: impl Into<String>, value: Real) -> Self {
        self.additional_results.insert(key.into(), value);
        self
    }
}

/// Base trait for all pricing engines.
///
/// A pricing engine computes `PricingResults` for a specific instrument type.
///
/// Corresponds to `QuantLib::PricingEngine`.
pub trait PricingEngine<Args>: std::fmt::Debug + Send + Sync {
    /// Price the instrument described by `args`.
    fn calculate(&self, args: &Args) -> Result<PricingResults>;
}

/// Base trait for all financial instruments.
///
/// Corresponds to `QuantLib::Instrument`.
pub trait Instrument: std::fmt::Debug + Send + Sync {
    /// Whether the instrument is expired (past maturity).
    fn is_expired(&self) -> bool;

    /// The maturity or last relevant date.
    fn maturity_date(&self) -> Option<Date> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pricing_results_builder() {
        let r = PricingResults::from_npv(42.0)
            .with_result("delta", 0.55)
            .with_result("gamma", 0.02);
        assert!((r.npv - 42.0).abs() < 1e-15);
        assert!((r.additional_results["delta"] - 0.55).abs() < 1e-15);
        assert!((r.additional_results["gamma"] - 0.02).abs() < 1e-15);
    }
}
