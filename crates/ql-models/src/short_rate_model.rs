//! Short-rate model trait hierarchy.
//!
//! Translates:
//! * `ql/models/shortrate/onefactormodel.hpp`
//! * `ql/models/shortrate/twofactormodel.hpp`
//!
//! The hierarchy:
//! ```text
//! CalibratedModel
//! └── ShortRateModel
//!     ├── OneFactorModel
//!     └── TwoFactorModel
//! ```

use crate::calibrated_model::CalibratedModel;
use ql_core::{Real, Time};
use ql_termstructures::YieldTermStructure;
use std::sync::Arc;

/// A general short-rate model.
///
/// Corresponds to `QuantLib::ShortRateModel`.
pub trait ShortRateModel: CalibratedModel {
    /// Return the discount bond price `P(t, T)` under the model.
    ///
    /// For affine models this has a closed-form `A(t,T) exp(-B(t,T) r)`.
    fn discount_bond(&self, t: Time, big_t: Time, rate: Real) -> Real;

    /// Return the yield term structure implied by the model.
    fn term_structure(&self) -> &Arc<dyn YieldTermStructure>;
}

/// A one-factor short-rate model.
///
/// The short rate follows `dr = μ(r,t) dt + σ(r,t) dW`.
///
/// Corresponds to `QuantLib::OneFactorModel`.
pub trait OneFactorModel: ShortRateModel {
    /// Instantaneous drift `μ(r, t)`.
    fn short_rate_drift(&self, t: Time, r: Real) -> Real;

    /// Instantaneous diffusion `σ(r, t)`.
    fn short_rate_diffusion(&self, t: Time, r: Real) -> Real;

    /// Create a `StochasticProcess1D` for the short rate
    /// (used by tree/lattice builders).
    fn dynamics_process(
        &self,
    ) -> Box<dyn ql_processes::StochasticProcess1D>;
}

/// A two-factor short-rate model.
///
/// The short rate is `r(t) = f(x(t), y(t))` where `x`, `y` are
/// correlated OU-like factors.
///
/// Corresponds to `QuantLib::TwoFactorModel`.
pub trait TwoFactorModel: ShortRateModel {
    /// Correlation between the two factors.
    fn correlation(&self) -> Real;
}

#[cfg(test)]
mod tests {
    // Trait tests will be in the concrete model modules.
}
