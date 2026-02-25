//! Analytic pricing engine for simple chooser options.
//!
//! A simple chooser option gives the holder the right to choose, at a future
//! date `t_c` (the choosing date), whether the option will be a European call
//! or put with the **same** strike `K` and expiry `T`.
//!
//! The closed-form formula is (Rubinstein 1991):
//!
//! $$V = S e^{-qT} N(d) - K e^{-rT} N(d - \sigma\sqrt{T})
//!       - S e^{-qT} N(-y) + K e^{-rT} N(-y + \sigma\sqrt{t_c})$$
//!
//! where
//! $$d = \frac{\ln(S/K) + (r - q + \sigma^2/2)T}{\sigma\sqrt{T}}, \quad
//!   y = \frac{\ln(S/K) + (r-q)T + \sigma^2 t_c / 2}{\sigma\sqrt{t_c}}$$
//!
//! Reference: "Complete Guide to Option Pricing Formulas", Haug, pp. 39-40.

use ql_core::Real;
use ql_math::distributions::normal_cdf;
use ql_processes::GeneralizedBlackScholesProcess;
use std::sync::Arc;

/// Arguments for a simple chooser option.
#[derive(Debug, Clone)]
pub struct SimpleChooserOptionArgs {
    /// The date (as year fraction from reference) at which the holder chooses call or put.
    pub choosing_time: Real,
    /// Strike price (same for call and put paths).
    pub strike: Real,
    /// Time to maturity of the underlying option (year fraction from reference).
    pub maturity_time: Real,
}

/// Analytic pricing engine for simple chooser options.
///
/// Implements the Rubinstein (1991) closed-form formula.
///
/// Corresponds to `QuantLib::AnalyticSimpleChooserEngine`.
#[derive(Debug)]
pub struct AnalyticSimpleChooserEngine {
    process: Arc<GeneralizedBlackScholesProcess>,
}

impl AnalyticSimpleChooserEngine {
    /// Create a new engine with the given Black-Scholes process.
    pub fn new(process: Arc<GeneralizedBlackScholesProcess>) -> Self {
        Self { process }
    }

    /// Price the simple chooser option.
    pub fn calculate(&self, args: &SimpleChooserOptionArgs) -> Real {
        let spot = self.process.spot();
        let strike = args.strike;
        let t_maturity = args.maturity_time;
        let t_choosing = args.choosing_time;

        let r = self.process.risk_free_rate().zero_rate_impl(t_maturity);
        let q = self.process.dividend_yield().zero_rate_impl(t_maturity);
        let sigma = self
            .process
            .black_volatility()
            .expect("process must have a black vol surface")
            .black_vol_time(t_maturity, strike);

        assert!(spot > 0.0, "spot must be positive");
        assert!(strike > 0.0, "strike must be positive");
        assert!(sigma > 0.0, "volatility must be positive");
        assert!(
            t_choosing > 0.0,
            "choosing date must be after evaluation date"
        );

        let sqrt_t = t_maturity.sqrt();
        let sqrt_tc = t_choosing.sqrt();

        let d =
            ((spot / strike).ln() + (r - q + 0.5 * sigma * sigma) * t_maturity) / (sigma * sqrt_t);

        let y = ((spot / strike).ln() + (r - q) * t_maturity + 0.5 * sigma * sigma * t_choosing)
            / (sigma * sqrt_tc);

        // V = S·exp(-q·T)·N(d)  - K·exp(-r·T)·N(d - σ√T)
        //   - S·exp(-q·T)·N(-y) + K·exp(-r·T)·N(-y + σ√tc)
        let df_q = (-q * t_maturity).exp();
        let df_r = (-r * t_maturity).exp();

        spot * df_q * normal_cdf(d)
            - strike * df_r * normal_cdf(d - sigma * sqrt_t)
            - spot * df_q * normal_cdf(-y)
            + strike * df_r * normal_cdf(-y + sigma * sqrt_tc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ql_termstructures::{BlackConstantVol, FlatForward, YieldTermStructure};
    use ql_time::{Actual360, Date, DayCounter};

    fn make_process(
        ref_date: Date,
        spot: Real,
        q: Real,
        r: Real,
        vol: Real,
    ) -> Arc<GeneralizedBlackScholesProcess> {
        let r_ts: Arc<dyn YieldTermStructure> =
            Arc::new(FlatForward::continuous(ref_date, r, Actual360));
        let q_ts: Arc<dyn YieldTermStructure> =
            Arc::new(FlatForward::continuous(ref_date, q, Actual360));
        let vol_ts = Arc::new(BlackConstantVol::new(ref_date, vol, Actual360));
        Arc::new(GeneralizedBlackScholesProcess::new(
            spot, r_ts, q_ts, vol_ts,
        ))
    }

    /// Test from Haug "Complete Guide to Option Pricing Formulas", pp. 39-40.
    /// Also matches QuantLib C++ test `testAnalyticSimpleChooserEngine`.
    #[test]
    fn test_simple_chooser() {
        let today = Date::from_ymd(2025, 1, 2).unwrap();
        let process = make_process(today, 50.0, 0.0, 0.08, 0.25);

        let dc = Actual360;
        let exercise_date = today + 180;
        let choosing_date = today + 90;
        let t_maturity = dc.year_fraction(today, exercise_date);
        let t_choosing = dc.year_fraction(today, choosing_date);

        let args = SimpleChooserOptionArgs {
            choosing_time: t_choosing,
            strike: 50.0,
            maturity_time: t_maturity,
        };

        let engine = AnalyticSimpleChooserEngine::new(process);
        let calculated = engine.calculate(&args);
        let expected = 6.1071;
        let tolerance = 3e-5;

        assert!(
            (calculated - expected).abs() < tolerance,
            "simple chooser: expected {expected}, got {calculated}, error {}",
            (calculated - expected).abs()
        );
    }
}
