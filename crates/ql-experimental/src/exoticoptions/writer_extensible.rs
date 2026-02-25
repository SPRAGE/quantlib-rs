//! Analytic pricing engine for writer-extensible options.
//!
//! A writer-extensible option is automatically extended when the option
//! finishes out-of-the-money at the first expiry. The extension gives the
//! holder a new option with a possibly different strike until the second expiry.
//!
//! Reference: Haug (2007); Longstaff (1990).

use super::bivariate_normal::bivariate_normal_cdf_dr78;
use ql_core::Real;
use ql_math::distributions::normal_cdf;
use ql_processes::GeneralizedBlackScholesProcess;
use std::sync::Arc;

/// Arguments for a writer-extensible option.
#[derive(Debug, Clone)]
pub struct WriterExtensibleOptionArgs {
    /// Option type sign: +1 for call, -1 for put.
    pub option_type: Real,
    /// Strike of the first option.
    pub strike1: Real,
    /// Strike of the second (extended) option.
    pub strike2: Real,
    /// Time to first expiry (year fraction).
    pub t1: Real,
    /// Time to second expiry (year fraction).
    pub t2: Real,
}

/// Analytic pricing engine for writer-extensible options.
///
/// Corresponds to `QuantLib::AnalyticWriterExtensibleOptionEngine`.
#[derive(Debug)]
pub struct AnalyticWriterExtensibleOptionEngine {
    process: Arc<GeneralizedBlackScholesProcess>,
}

impl AnalyticWriterExtensibleOptionEngine {
    /// Create a new engine.
    pub fn new(process: Arc<GeneralizedBlackScholesProcess>) -> Self {
        Self { process }
    }

    /// Price the writer-extensible option.
    pub fn calculate(&self, args: &WriterExtensibleOptionArgs) -> Real {
        let spot = self.process.spot();
        let t1 = args.t1;
        let t2 = args.t2;
        let x1 = args.strike1;
        let x2 = args.strike2;

        let r = self.process.risk_free_rate().zero_rate_impl(t1);
        let q = self.process.dividend_yield().zero_rate_impl(t1);
        let b = r - q;

        let vol = self
            .process
            .black_volatility()
            .expect("process must have a black vol surface")
            .black_vol_time(t1, x1);

        let forward = spot * (b * t1).exp();
        let std_dev = vol * t1.sqrt();
        let discount = (-r * t1).exp();

        // Black formula for the base option
        let d1 = (forward / x1).ln() / std_dev + 0.5 * std_dev;
        let d2 = d1 - std_dev;

        let phi = args.option_type;
        let black = if phi > 0.0 {
            discount * (forward * normal_cdf(d1) - x1 * normal_cdf(d2))
        } else {
            discount * (x1 * normal_cdf(-d2) - forward * normal_cdf(-d1))
        };

        // Bivariate component
        let ro = (t1 / t2).sqrt();
        let z1 = ((spot / x2).ln() + (b + vol * vol / 2.0) * t2) / (vol * t2.sqrt());
        let z2 = ((spot / x1).ln() + (b + vol * vol / 2.0) * t1) / (vol * t1.sqrt());

        // C++ uses BivariateCumulativeNormalDistributionWe04DP(-ro) for writer-extensible
        // Our bivariate_normal_cdf takes correlation directly, and the C++ feeds -rho.

        if phi > 0.0 {
            // Call case
            let biv1 = bivariate_normal_cdf_dr78(z1, -z2, -ro);
            let biv2 = bivariate_normal_cdf_dr78(z1 - vol * t2.sqrt(), -z2 + vol * t1.sqrt(), -ro);
            black + spot * ((b - r) * t2).exp() * biv1 - x2 * (-r * t2).exp() * biv2
        } else {
            // Put case
            let biv1 = bivariate_normal_cdf_dr78(-z1, z2, -ro);
            let biv2 = bivariate_normal_cdf_dr78(-z1 + vol * t2.sqrt(), z2 - vol * t1.sqrt(), -ro);
            black - spot * ((b - r) * t2).exp() * biv1 + x2 * (-r * t2).exp() * biv2
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ql_termstructures::{BlackConstantVol, FlatForward, YieldTermStructure};
    use ql_time::{Actual360, Date, DayCounter};

    /// Test from QuantLib C++ `testAnalyticWriterExtensibleOptionEngine`.
    #[test]
    fn test_writer_extensible_call() {
        let today = Date::from_ymd(2025, 1, 2).unwrap();
        let dc = Actual360;

        let spot = 80.0;
        let r = 0.10;
        let q = 0.0;
        let vol = 0.30;
        let strike1 = 90.0;
        let strike2 = 82.0;

        let ex_date1 = today + 180;
        let ex_date2 = today + 270;

        let t1 = dc.year_fraction(today, ex_date1);
        let t2 = dc.year_fraction(today, ex_date2);

        let r_ts: Arc<dyn YieldTermStructure> =
            Arc::new(FlatForward::continuous(today, r, Actual360));
        let q_ts: Arc<dyn YieldTermStructure> =
            Arc::new(FlatForward::continuous(today, q, Actual360));
        let vol_ts = Arc::new(BlackConstantVol::new(today, vol, Actual360));

        let process = Arc::new(GeneralizedBlackScholesProcess::new(
            spot, r_ts, q_ts, vol_ts,
        ));

        let args = WriterExtensibleOptionArgs {
            option_type: 1.0, // Call
            strike1,
            strike2,
            t1,
            t2,
        };

        let engine = AnalyticWriterExtensibleOptionEngine::new(process);
        let calculated = engine.calculate(&args);
        let expected = 6.8238;
        let tolerance = 1e-4;

        assert!(
            (calculated - expected).abs() < tolerance,
            "writer-extensible call: expected {expected}, got {calculated}, error {}",
            (calculated - expected).abs()
        );
    }
}
