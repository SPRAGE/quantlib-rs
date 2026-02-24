//! Analytic pricing engine for two-asset correlation options.
//!
//! A two-asset correlation option pays off `max(S2 - X2, 0)` (call) or
//! `max(X2 - S2, 0)` (put) at expiry, but only if `S1 > X1` (call) or
//! `S1 < X1` (put) at expiry.
//!
//! The closed-form formula uses the bivariate normal distribution.
//!
//! Reference: Zhang (1998), "Exotic Options".

use ql_core::Real;
use super::bivariate_normal::bivariate_normal_cdf_dr78;
use ql_processes::GeneralizedBlackScholesProcess;
use std::sync::Arc;

/// Arguments for a two-asset correlation option.
#[derive(Debug, Clone)]
pub struct TwoAssetCorrelationArgs {
    /// Option type sign: +1 for call, -1 for put.
    pub option_type: Real,
    /// Strike on the first asset (trigger condition).
    pub strike1: Real,
    /// Strike on the second asset (payoff).
    pub strike2: Real,
    /// Time to maturity (year fraction).
    pub maturity: Real,
    /// Correlation between the two assets.
    pub correlation: Real,
}

/// Analytic pricing engine for two-asset correlation options.
///
/// Corresponds to `QuantLib::AnalyticTwoAssetCorrelationEngine`.
#[derive(Debug)]
pub struct AnalyticTwoAssetCorrelationEngine {
    process1: Arc<GeneralizedBlackScholesProcess>,
    process2: Arc<GeneralizedBlackScholesProcess>,
}

impl AnalyticTwoAssetCorrelationEngine {
    /// Create a new engine with two Black-Scholes processes and a correlation.
    pub fn new(
        process1: Arc<GeneralizedBlackScholesProcess>,
        process2: Arc<GeneralizedBlackScholesProcess>,
    ) -> Self {
        Self { process1, process2 }
    }

    /// Price the two-asset correlation option.
    pub fn calculate(&self, args: &TwoAssetCorrelationArgs) -> Real {
        let t = args.maturity;
        let s1 = self.process1.spot();
        let s2 = self.process2.spot();
        let x1 = args.strike1;
        let x2 = args.strike2;
        let rho = args.correlation;

        let sigma1 = self
            .process1
            .black_volatility()
            .expect("process1 must have a black vol surface")
            .black_vol_time(t, x1);

        let sigma2 = self
            .process2
            .black_volatility()
            .expect("process2 must have a black vol surface")
            .black_vol_time(t, x1);

        let r = self.process1.risk_free_rate().zero_rate_impl(t);
        let q1 = self.process1.dividend_yield().zero_rate_impl(t);
        let q2 = self.process2.dividend_yield().zero_rate_impl(t);
        let b1 = r - q1;
        let b2 = r - q2;

        let sqrt_t = t.sqrt();

        let y1 = ((s1 / x1).ln() + (b1 - sigma1 * sigma1 / 2.0) * t) / (sigma1 * sqrt_t);
        let y2 = ((s2 / x2).ln() + (b2 - sigma2 * sigma2 / 2.0) * t) / (sigma2 * sqrt_t);

        let phi = args.option_type; // +1 call, -1 put

        if phi > 0.0 {
            // Call
            s2 * ((b2 - r) * t).exp()
                * bivariate_normal_cdf_dr78(
                    y2 + sigma2 * sqrt_t,
                    y1 + rho * sigma2 * sqrt_t,
                    rho,
                )
                - x2 * (-r * t).exp() * bivariate_normal_cdf_dr78(y2, y1, rho)
        } else {
            // Put
            x2 * (-r * t).exp() * bivariate_normal_cdf_dr78(-y2, -y1, rho)
                - s2 * ((b2 - r) * t).exp()
                    * bivariate_normal_cdf_dr78(
                        -y2 - sigma2 * sqrt_t,
                        -y1 - rho * sigma2 * sqrt_t,
                        rho,
                    )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ql_termstructures::{BlackConstantVol, FlatForward, YieldTermStructure};
    use ql_time::{Actual360, Date, DayCounter};

    /// Test from QuantLib C++ `testAnalyticEngine` in
    /// `twoassetcorrelationoption.cpp`.
    #[test]
    fn test_two_asset_correlation() {
        let today = Date::from_ymd(2025, 1, 2).unwrap();
        let dc = Actual360;

        let r = 0.1;
        let q1 = 0.0;
        let q2 = 0.0;
        let sigma1 = 0.2;
        let sigma2 = 0.3;
        let rho = 0.75;

        let r_ts: Arc<dyn YieldTermStructure> =
            Arc::new(FlatForward::continuous(today, r, dc));
        let q1_ts: Arc<dyn YieldTermStructure> =
            Arc::new(FlatForward::continuous(today, q1, dc));
        let q2_ts: Arc<dyn YieldTermStructure> =
            Arc::new(FlatForward::continuous(today, q2, dc));
        let vol1_ts = Arc::new(BlackConstantVol::new(today, sigma1, dc));
        let vol2_ts = Arc::new(BlackConstantVol::new(today, sigma2, dc));

        let process1 = Arc::new(GeneralizedBlackScholesProcess::new(
            52.0,
            r_ts.clone(),
            q1_ts,
            vol1_ts,
        ));
        let process2 = Arc::new(GeneralizedBlackScholesProcess::new(
            65.0,
            r_ts,
            q2_ts,
            vol2_ts,
        ));

        let t = dc.year_fraction(today, today + 180);

        let args = TwoAssetCorrelationArgs {
            option_type: 1.0, // Call
            strike1: 50.0,
            strike2: 70.0,
            maturity: t,
            correlation: rho,
        };

        let engine = AnalyticTwoAssetCorrelationEngine::new(process1, process2);

        let calculated = engine.calculate(&args);
        let expected = 4.7073;
        let tolerance = 1e-4;

        assert!(
            (calculated - expected).abs() < tolerance,
            "two-asset correlation: expected {expected}, got {calculated}, error {}",
            (calculated - expected).abs()
        );
    }
}
