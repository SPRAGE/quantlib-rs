//! # ql-cashflows
//!
//! Cash flows, coupons (fixed, floating, CMS), and legs.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

pub mod cashflow;
pub mod cashflows;
pub mod coupon;
pub mod fixed_rate_coupon;
pub mod floating_rate_coupon;
pub mod inflation_coupon;

pub use cashflow::{CashFlow, Leg, Redemption, SimpleCashFlow};
pub use cashflows::{
    bps_curve, bps_yield, convexity, duration, maturity_date, next_cashflow_date, npv_curve,
    npv_yield, npv_z_spread, previous_cashflow_date, yield_rate, z_spread, Duration,
};
pub use coupon::Coupon;
pub use fixed_rate_coupon::{FixedRateCoupon, FixedRateLegBuilder};
pub use floating_rate_coupon::{FloatingRateCoupon, IborCoupon, IborLegBuilder};
pub use inflation_coupon::{CPICoupon, YoYInflationCoupon};
