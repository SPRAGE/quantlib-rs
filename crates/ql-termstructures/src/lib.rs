//! # ql-termstructures
//!
//! Yield curves, volatility surfaces, default-probability term structures,
//! and inflation term structures.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

// ── Modules ───────────────────────────────────────────────────────────────────

/// `TermStructure` — base trait for all term structures.
pub mod term_structure;

/// `YieldTermStructure` — yield / interest-rate term structures.
pub mod yield_term_structure;

/// `FlatForward` — constant forward-rate yield curve.
pub mod flat_forward;

/// `InterpolatedZeroCurve` — zero-rate interpolated yield curve.
pub mod interpolated_zero_curve;

/// `InterpolatedDiscountCurve` — discount-factor interpolated yield curve.
pub mod interpolated_discount_curve;

/// `InterpolatedForwardCurve` — instantaneous forward-rate interpolated yield curve.
pub mod interpolated_forward_curve;

/// `VolatilityTermStructure` — base trait for volatility term structures.
pub mod volatility_term_structure;

/// `BlackVolTermStructure` — Black-volatility term structures and `BlackConstantVol`.
pub mod black_vol_term_structure;

/// `BlackVarianceSurface` — 2D Black-variance surface with bilinear interpolation.
pub mod black_variance_surface;

/// `LocalVolTermStructure` — local-volatility term structures and `LocalConstantVol`.
pub mod local_vol_term_structure;

/// `LocalVolSurface` — Dupire local volatility surface from a Black vol surface.
pub mod local_vol_surface;

/// `SmileSection` — abstract smile interface and concrete smile sections
/// (Flat, SABR, SVI).
pub mod smile_section;

/// Smile calibration framework — per-expiry SABR/SVI calibration and
/// `SmileSurface` for the full volatility surface.
pub mod smile_calibration;

/// `DefaultProbabilityTermStructure` — credit default-probability curves.
pub mod default_probability_term_structure;

/// Inflation term structures: zero-inflation and year-on-year inflation curves.
pub mod inflation_term_structure;

// ── Convenience re-exports ────────────────────────────────────────────────────

pub use black_variance_surface::{BlackVarianceSurface, Extrapolation};
pub use black_vol_term_structure::{BlackConstantVol, BlackVolTermStructure};
pub use default_probability_term_structure::{
    DefaultProbabilityTermStructure, FlatHazardRate, InterpolatedHazardRateCurve,
};
pub use flat_forward::FlatForward;
pub use inflation_term_structure::{
    FlatYoYInflationCurve, FlatZeroInflationCurve, InflationTermStructure,
    YoYInflationTermStructure, ZeroInflationTermStructure,
};
pub use interpolated_discount_curve::InterpolatedDiscountCurve;
pub use interpolated_forward_curve::InterpolatedForwardCurve;
pub use interpolated_zero_curve::{
    CubicNatural, InterpolatedZeroCurve, InterpolationBuilder, Linear, LogLinear,
};
pub use local_vol_term_structure::{LocalConstantVol, LocalVolTermStructure};
pub use local_vol_surface::LocalVolSurface;
pub use smile_section::{
    calibrate_svi, FlatSmileSection, SabrSmileSection, SmileOptionType, SmileSection,
    SviParameters, SviSmileSection, VolatilityType,
};
pub use smile_calibration::{
    calibrate_sabr_surface, calibrate_svi_surface, ExpirySmileData, SmileCalibrationResult,
    SmileSurface,
};
pub use term_structure::TermStructure;
pub use volatility_term_structure::VolatilityTermStructure;
pub use yield_term_structure::{YieldTermStructure, YieldTermStructureData};
