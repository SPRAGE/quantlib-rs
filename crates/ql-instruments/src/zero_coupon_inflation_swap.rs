//! Zero-coupon inflation swap (translates `ql/instruments/zerocouponinflationswap.hpp`).
//!
//! A zero-coupon inflation swap exchanges a fixed payment for an
//! inflation-linked payment at maturity:
//!
//! * **Fixed leg**: `notional × [(1 + fixed_rate)^T − 1]`
//! * **Inflation leg**: `notional × [CPI(T) / CPI(0) − 1]`
//!
//! The NPV is computed as the difference discounted to today.

use crate::instrument::Instrument;
use ql_core::Real;
use ql_time::Date;

/// Type of the inflation swap leg.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwapPayerType {
    /// Payer pays fixed, receives inflation.
    Payer,
    /// Receiver pays inflation, receives fixed.
    Receiver,
}

/// A zero-coupon inflation swap.
///
/// Corresponds to `QuantLib::ZeroCouponInflationSwap`.
#[derive(Debug)]
pub struct ZeroCouponInflationSwap {
    /// Payer or receiver.
    pub swap_type: SwapPayerType,
    /// Notional principal.
    pub notional: Real,
    /// Start date.
    pub start_date: Date,
    /// Maturity date.
    pub maturity_date: Date,
    /// Fixed rate (e.g. 0.025 for 2.5%).
    pub fixed_rate: Real,
    /// Base CPI at inception.
    pub base_cpi: Real,
    /// Observed/projected CPI at maturity.
    pub observation_cpi: Option<Real>,
    /// Discount factor to today for the payment.
    pub discount_factor: Option<Real>,
}

impl ZeroCouponInflationSwap {
    /// Create a new zero-coupon inflation swap.
    pub fn new(
        swap_type: SwapPayerType,
        notional: Real,
        start_date: Date,
        maturity_date: Date,
        fixed_rate: Real,
        base_cpi: Real,
    ) -> Self {
        Self {
            swap_type,
            notional,
            start_date,
            maturity_date,
            fixed_rate,
            base_cpi,
            observation_cpi: None,
            discount_factor: None,
        }
    }

    /// Set the observed/projected CPI and discount factor for NPV calculation.
    pub fn with_market_data(mut self, observation_cpi: Real, discount_factor: Real) -> Self {
        self.observation_cpi = Some(observation_cpi);
        self.discount_factor = Some(discount_factor);
        self
    }

    /// Year fraction from start to maturity (simple).
    fn year_fraction(&self) -> Real {
        let days = self.maturity_date.serial() - self.start_date.serial();
        days as f64 / 365.0
    }

    /// Fixed leg payment at maturity.
    pub fn fixed_leg_amount(&self) -> Real {
        let t = self.year_fraction();
        self.notional * ((1.0 + self.fixed_rate).powf(t) - 1.0)
    }

    /// Inflation leg payment at maturity (requires `observation_cpi` to be set).
    pub fn inflation_leg_amount(&self) -> Option<Real> {
        self.observation_cpi
            .map(|cpi| self.notional * (cpi / self.base_cpi - 1.0))
    }

    /// Net present value from the payer's perspective.
    ///
    /// Payer pays the fixed leg and receives the inflation leg.
    pub fn npv(&self) -> Option<Real> {
        let infl = self.inflation_leg_amount()?;
        let fixed = self.fixed_leg_amount();
        let df = self.discount_factor.unwrap_or(1.0);
        let sign = match self.swap_type {
            SwapPayerType::Payer => 1.0,
            SwapPayerType::Receiver => -1.0,
        };
        Some(sign * (infl - fixed) * df)
    }
}

impl Instrument for ZeroCouponInflationSwap {
    fn is_expired(&self) -> bool {
        false // would check vs evaluation date in production
    }
    fn maturity_date(&self) -> Option<Date> {
        Some(self.maturity_date)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_leg_compound() {
        let swap = ZeroCouponInflationSwap::new(
            SwapPayerType::Payer,
            1_000_000.0,
            Date::from_ymd(2020, 1, 1).unwrap(),
            Date::from_ymd(2025, 1, 1).unwrap(),
            0.025,
            300.0,
        );
        // T ≈ 1826/365 ≈ 5.00274 years
        let fixed = swap.fixed_leg_amount();
        // (1.025)^5.00274 - 1 ≈ 0.13141...
        assert!(fixed > 130_000.0 && fixed < 135_000.0);
    }

    #[test]
    fn inflation_leg_ratio() {
        let swap = ZeroCouponInflationSwap::new(
            SwapPayerType::Payer,
            1_000_000.0,
            Date::from_ymd(2020, 1, 1).unwrap(),
            Date::from_ymd(2025, 1, 1).unwrap(),
            0.025,
            300.0,
        )
        .with_market_data(340.0, 0.95);
        let infl = swap.inflation_leg_amount().unwrap();
        // 1M * (340/300 - 1) = 1M * 0.13333 = 133_333.33
        assert!((infl - 133_333.33333).abs() < 1.0);
    }

    #[test]
    fn npv_payer() {
        let swap = ZeroCouponInflationSwap::new(
            SwapPayerType::Payer,
            1_000_000.0,
            Date::from_ymd(2020, 1, 1).unwrap(),
            Date::from_ymd(2025, 1, 2).unwrap(), // exactly 5y + 1d
            0.025,
            300.0,
        )
        .with_market_data(340.0, 1.0);
        let npv = swap.npv().unwrap();
        // inflation leg ≈ 133_333, fixed leg ≈ 131_xxx, NPV > 0 for payer
        // (receiving more inflation than paying fixed)
        assert!(npv > 0.0);
    }

    #[test]
    fn npv_receiver_negates() {
        let swap_payer = ZeroCouponInflationSwap::new(
            SwapPayerType::Payer,
            1_000_000.0,
            Date::from_ymd(2020, 1, 1).unwrap(),
            Date::from_ymd(2025, 1, 2).unwrap(),
            0.025,
            300.0,
        )
        .with_market_data(340.0, 1.0);
        let swap_recv = ZeroCouponInflationSwap::new(
            SwapPayerType::Receiver,
            1_000_000.0,
            Date::from_ymd(2020, 1, 1).unwrap(),
            Date::from_ymd(2025, 1, 2).unwrap(),
            0.025,
            300.0,
        )
        .with_market_data(340.0, 1.0);
        let npv_p = swap_payer.npv().unwrap();
        let npv_r = swap_recv.npv().unwrap();
        assert!((npv_p + npv_r).abs() < 1e-10);
    }

    #[test]
    fn at_the_money_swap() {
        // When inflation exactly matches fixed rate, NPV ≈ 0
        // fixed rate = 2.5% over ~5y → (1.025)^5 − 1 ≈ 0.13141
        // set CPI so that CPI(T)/CPI(0) − 1 ≈ 0.13141 → CPI(T) ≈ 339.42
        let start = Date::from_ymd(2020, 1, 1).unwrap();
        let end = Date::from_ymd(2025, 1, 1).unwrap();
        let base: f64 = 300.0;
        let rate: f64 = 0.025;
        let t = (end.serial() - start.serial()) as f64 / 365.0;
        let atm_cpi = base * (1.0 + rate).powf(t);
        let swap =
            ZeroCouponInflationSwap::new(SwapPayerType::Payer, 1_000_000.0, start, end, rate, base)
                .with_market_data(atm_cpi, 1.0);
        assert!(swap.npv().unwrap().abs() < 1.0); // < $1 error
    }
}
