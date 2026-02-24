//! Variance Gamma model wrapper.
//!
//! A thin wrapper around `VarianceGammaProcess` that can be used with the
//! model-engine pattern.

use ql_processes::VarianceGammaProcess;
use std::sync::Arc;

/// Variance Gamma model — wraps a `VarianceGammaProcess`.
///
/// Corresponds to `QuantLib::VarianceGammaModel`.
#[derive(Debug)]
pub struct VarianceGammaModel {
    /// The underlying VG process.
    pub process: Arc<VarianceGammaProcess>,
}

impl VarianceGammaModel {
    /// Create a new VG model from a process.
    pub fn new(process: Arc<VarianceGammaProcess>) -> Self {
        Self { process }
    }

    /// Volatility parameter σ.
    pub fn sigma(&self) -> f64 {
        self.process.sigma
    }

    /// Variance rate ν.
    pub fn nu(&self) -> f64 {
        self.process.nu
    }

    /// Drift parameter θ.
    pub fn theta(&self) -> f64 {
        self.process.theta
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ql_termstructures::FlatForward;
    use ql_termstructures::YieldTermStructure;
    use ql_time::{Actual365Fixed, Date};

    fn flat_ts(rate: f64) -> Arc<dyn YieldTermStructure> {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        Arc::new(FlatForward::continuous(ref_date, rate, Actual365Fixed))
    }

    #[test]
    fn model_accessors() {
        let process = Arc::new(VarianceGammaProcess::new(
            6000.0,
            flat_ts(0.05),
            flat_ts(0.0),
            0.20,
            0.05,
            -0.50,
        ));
        let model = VarianceGammaModel::new(process);
        assert!((model.sigma() - 0.20).abs() < 1e-12);
        assert!((model.nu() - 0.05).abs() < 1e-12);
        assert!((model.theta() - (-0.50)).abs() < 1e-12);
    }
}
