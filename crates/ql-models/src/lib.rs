//! # ql-models
//!
//! Calibratable financial models (short-rate, equity, credit).
//!
//! ## Trait hierarchy
//!
//! ```text
//! CalibratedModel
//! ├── ShortRateModel
//! │   ├── OneFactorModel  → Vasicek, HullWhite, BlackKarasinski, CIR
//! │   └── TwoFactorModel  → G2
//! └── (equity models)     → HestonModel, BatesModel
//! ```

#![warn(missing_docs)]
#![forbid(unsafe_code)]

// ── Infrastructure ───────────────────────────────────────────────────────
pub mod calibrated_model;
pub mod short_rate_model;

// ── One-factor short-rate models ─────────────────────────────────────────
pub mod black_karasinski;
pub mod cox_ingersoll_ross;
pub mod hull_white_model;
pub mod vasicek;

// ── Two-factor short-rate models ─────────────────────────────────────────
pub mod g2_model;

// ── Equity models ────────────────────────────────────────────────────────
pub mod bates_model;
pub mod heston_model;

// ── Re-exports ───────────────────────────────────────────────────────────
pub use bates_model::BatesModel;
pub use black_karasinski::BlackKarasinski;
pub use calibrated_model::{
    BoundaryConstraint, CalibratedModel, Constraint, NoConstraint, Parameter, PositiveConstraint,
};
pub use cox_ingersoll_ross::CoxIngersollRoss;
pub use g2_model::G2Model;
pub use heston_model::HestonModel;
pub use hull_white_model::HullWhite;
pub use short_rate_model::{OneFactorModel, ShortRateModel, TwoFactorModel};
pub use vasicek::Vasicek;
