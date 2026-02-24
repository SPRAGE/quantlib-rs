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
pub mod vasicek;
pub mod hull_white_model;
pub mod cox_ingersoll_ross;
pub mod black_karasinski;

// ── Two-factor short-rate models ─────────────────────────────────────────
pub mod g2_model;

// ── Equity models ────────────────────────────────────────────────────────
pub mod heston_model;
pub mod bates_model;

// ── Re-exports ───────────────────────────────────────────────────────────
pub use calibrated_model::{CalibratedModel, Parameter, Constraint, NoConstraint, PositiveConstraint, BoundaryConstraint};
pub use short_rate_model::{ShortRateModel, OneFactorModel, TwoFactorModel};
pub use vasicek::Vasicek;
pub use hull_white_model::HullWhite;
pub use cox_ingersoll_ross::CoxIngersollRoss;
pub use black_karasinski::BlackKarasinski;
pub use g2_model::G2Model;
pub use heston_model::HestonModel;
pub use bates_model::BatesModel;
