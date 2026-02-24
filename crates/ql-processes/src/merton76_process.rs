//! Merton jump-diffusion process
//! (translates `ql/processes/merton76process.hpp`).
//!
//! ```text
//! dS/S = (r − d − λ·k) dt + σ dW + J dN
//! ```
//!
//! where `λ` is the jump intensity, `J` is the log-jump size
//! (normal with mean `δ` and std dev `ν`), `k = exp(δ + ν²/2) - 1`,
//! and `N` is a Poisson process.
//!
//! The Merton76 process extends the standard GBS process by adding
//! Poisson-distributed jumps. For simulation purposes, the base diffusion
//! is a `GeneralizedBlackScholesProcess`; the jump component is layered on
//! in the pricing engine.

use crate::black_scholes_process::GeneralizedBlackScholesProcess;
use crate::stochastic_process::{StochasticProcess, StochasticProcess1D};
use ql_core::{Real, Time};
use ql_math::{Array, Matrix};
use std::sync::Arc;

/// A Merton (1976) jump-diffusion process.
///
/// The continuous part follows a `GeneralizedBlackScholesProcess`.
/// Jump parameters (intensity, mean, vol) are stored here and used by
/// the pricing engine.
///
/// Corresponds to `QuantLib::Merton76Process`.
#[derive(Debug)]
pub struct Merton76Process {
    /// The underlying Black-Scholes diffusion process.
    pub bs_process: Arc<GeneralizedBlackScholesProcess>,
    /// Jump intensity λ (average number of jumps per year).
    pub jump_intensity: Real,
    /// Mean of log-jump size δ.
    pub log_jump_mean: Real,
    /// Vol of log-jump size ν.
    pub log_jump_vol: Real,
}

impl Merton76Process {
    /// Create a new Merton jump-diffusion process.
    ///
    /// # Arguments
    /// * `bs_process` — underlying GBS process
    /// * `jump_intensity` — Poisson intensity λ
    /// * `log_jump_mean` — mean of log(1+J), δ
    /// * `log_jump_vol` — std dev of log(1+J), ν
    pub fn new(
        bs_process: Arc<GeneralizedBlackScholesProcess>,
        jump_intensity: Real,
        log_jump_mean: Real,
        log_jump_vol: Real,
    ) -> Self {
        Self {
            bs_process,
            jump_intensity,
            log_jump_mean,
            log_jump_vol,
        }
    }

    /// Compensator k = E[J] = exp(δ + ν²/2) − 1.
    pub fn jump_compensator(&self) -> Real {
        (self.log_jump_mean + 0.5 * self.log_jump_vol * self.log_jump_vol).exp() - 1.0
    }
}

/// The Merton76 process delegates its continuous dynamics to the underlying
/// BS process. It implements `StochasticProcess` (not `StochasticProcess1D`)
/// to avoid conflicting with the blanket impl.
impl StochasticProcess for Merton76Process {
    fn size(&self) -> usize {
        1
    }

    fn factors(&self) -> usize {
        1
    }

    fn initial_values(&self) -> Array {
        self.bs_process.initial_values()
    }

    fn drift(&self, t: Time, x: &Array) -> Array {
        self.bs_process.drift(t, x)
    }

    fn diffusion(&self, t: Time, x: &Array) -> Matrix {
        self.bs_process.diffusion(t, x)
    }

    fn expectation(&self, t: Time, x: &Array, dt: Time) -> Array {
        self.bs_process.expectation(t, x, dt)
    }

    fn std_deviation(&self, t: Time, x: &Array, dt: Time) -> Matrix {
        self.bs_process.std_deviation(t, x, dt)
    }

    fn evolve(&self, t: Time, x: &Array, dt: Time, dw: &Array) -> Array {
        self.bs_process.evolve(t, x, dt, dw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ql_termstructures::{BlackConstantVol, FlatForward};
    use ql_time::{Actual365Fixed, Date};

    fn make_merton() -> Merton76Process {
        let ref_date = Date::from_ymd(2025, 1, 2).unwrap();
        let r: Arc<dyn ql_termstructures::YieldTermStructure> =
            Arc::new(FlatForward::continuous(ref_date, 0.05, Actual365Fixed));
        let vol: Arc<dyn ql_termstructures::BlackVolTermStructure> =
            Arc::new(BlackConstantVol::new(ref_date, 0.20, Actual365Fixed));
        Merton76Process {
            bs_process: Arc::new(crate::black_scholes_process::black_scholes_process(
                100.0, r, vol,
            )),
            jump_intensity: 1.0,
            log_jump_mean: -0.1,
            log_jump_vol: 0.15,
        }
    }

    #[test]
    fn merton_compensator() {
        let m = make_merton();
        let k = m.jump_compensator();
        // k = exp(-0.1 + 0.5*0.0225) - 1 = exp(-0.08875) - 1 ≈ -0.0849
        let expected = (-0.1 + 0.5 * 0.0225_f64).exp() - 1.0;
        assert!((k - expected).abs() < 1e-10);
    }

    #[test]
    fn merton_size() {
        let m = make_merton();
        assert_eq!(m.size(), 1);
        assert_eq!(m.factors(), 1);
    }
}
