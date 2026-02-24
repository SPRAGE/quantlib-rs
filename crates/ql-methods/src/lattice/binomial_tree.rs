//! Recombining binomial trees for option pricing.
//!
//! Translates `ql/methods/lattices/binomialtree.hpp` — all seven classical
//! tree variants:
//!
//! | Variant | Type | Reference |
//! |---|---|---|
//! | [`BinomialTree::jarrow_rudd`] | Equal probabilities | Jarrow & Rudd (1983) |
//! | [`BinomialTree::cox_ross_rubinstein`] | Equal jumps | Cox, Ross & Rubinstein (1979) |
//! | [`BinomialTree::additive_eqp`] | Equal probabilities | Additive EQP |
//! | [`BinomialTree::trigeorgis`] | Equal jumps | Trigeorgis (1991) |
//! | [`BinomialTree::tian`] | Multiplicative | Tian (1993) |
//! | [`BinomialTree::leisen_reimer`] | Multiplicative | Leisen & Reimer (1996) |
//! | [`BinomialTree::joshi4`] | Multiplicative | Joshi (2008) |

use ql_core::Real;
use ql_processes::StochasticProcess1D;

/// The kind of underlying model for node values.
#[derive(Debug, Clone)]
enum UnderlyingKind {
    /// `x0 * exp(i * drift_per_step + (2j − i) * step)`
    ///
    /// Used by equal-probability (JR, AdditiveEQP) and equal-jump (CRR, Trigeorgis) trees.
    LogSpace { step: Real },
    /// `x0 * down^(i − j) * up^j`
    ///
    /// Used by multiplicative trees (Tian, Leisen-Reimer, Joshi4).
    Multiplicative { up: Real, down: Real },
}

/// A recombining binomial tree approximating a 1-D stochastic process.
///
/// The tree has `steps + 1` time layers, with layer `i` having `i + 1` nodes.
/// Node `(i, j)` represents the state after `j` up-moves and `i − j` down-moves.
///
/// Seven classical variants are provided via named constructors.
///
/// Corresponds to the C++ hierarchy rooted at `QuantLib::BinomialTree<T>`.
#[derive(Debug, Clone)]
pub struct BinomialTree {
    x0: Real,
    dt: Real,
    steps: usize,
    drift_per_step: Real,
    underlying: UnderlyingKind,
    pu: Real,
    pd: Real,
}

impl BinomialTree {
    // ── Accessors ────────────────────────────────────────────────────────

    /// Number of time steps.
    pub fn steps(&self) -> usize {
        self.steps
    }

    /// Time increment per step.
    pub fn dt(&self) -> Real {
        self.dt
    }

    /// Initial underlying value (spot price).
    pub fn x0(&self) -> Real {
        self.x0
    }

    /// Number of nodes at time step `i` (always `i + 1` for a binomial tree).
    pub fn size(&self, i: usize) -> usize {
        i + 1
    }

    /// Index of the descendant node at step `i+1` for a given `branch`.
    ///
    /// `branch = 0` → down, `branch = 1` → up.
    pub fn descendant(&self, _i: usize, index: usize, branch: usize) -> usize {
        index + branch
    }

    /// Underlying value (e.g. stock price) at node `(i, index)`.
    pub fn underlying(&self, i: usize, index: usize) -> Real {
        match &self.underlying {
            UnderlyingKind::LogSpace { step } => {
                let j = 2 * index as isize - i as isize;
                self.x0 * (i as Real * self.drift_per_step + j as Real * step).exp()
            }
            UnderlyingKind::Multiplicative { up, down } => {
                self.x0 * down.powi((i - index) as i32) * up.powi(index as i32)
            }
        }
    }

    /// Transition probability for `branch` (0 = down, 1 = up).
    pub fn probability(&self, _i: usize, _index: usize, branch: usize) -> Real {
        if branch == 1 {
            self.pu
        } else {
            self.pd
        }
    }

    // ── Named constructors ───────────────────────────────────────────────

    /// Jarrow-Rudd tree (equal probabilities, multiplicative).
    ///
    /// `p_up = p_down = 0.5`, step size `= σ √Δt`.
    pub fn jarrow_rudd(process: &dyn StochasticProcess1D, end: Real, steps: usize) -> Self {
        let (x0, dt, dps, std) = log_params(process, end, steps);
        Self {
            x0,
            dt,
            steps,
            drift_per_step: dps,
            underlying: UnderlyingKind::LogSpace { step: std },
            pu: 0.5,
            pd: 0.5,
        }
    }

    /// Cox-Ross-Rubinstein tree (equal jumps).
    ///
    /// `dx = σ √Δt`, `p_up = ½ + ½ μΔt / dx`.
    /// Drift is captured entirely by the probabilities, not in node values.
    pub fn cox_ross_rubinstein(
        process: &dyn StochasticProcess1D,
        end: Real,
        steps: usize,
    ) -> Self {
        let (x0, dt, dps, std) = log_params(process, end, steps);
        let dx = std;
        let pu = 0.5 + 0.5 * dps / dx;
        let pd = 1.0 - pu;
        assert!(
            pu >= 0.0 && pu <= 1.0,
            "CRR: invalid probability {pu} (try more steps)"
        );
        Self {
            x0,
            dt,
            steps,
            drift_per_step: 0.0,
            underlying: UnderlyingKind::LogSpace { step: dx },
            pu,
            pd,
        }
    }

    /// Additive equal-probabilities tree.
    ///
    /// `p_up = p_down = 0.5`, step size chosen to match variance.
    pub fn additive_eqp(process: &dyn StochasticProcess1D, end: Real, steps: usize) -> Self {
        let (x0, dt, dps, _std) = log_params(process, end, steps);
        let var = log_variance(process, dt);
        let up = -0.5 * dps + 0.5 * (4.0 * var - 3.0 * dps * dps).sqrt();
        Self {
            x0,
            dt,
            steps,
            drift_per_step: dps,
            underlying: UnderlyingKind::LogSpace { step: up },
            pu: 0.5,
            pd: 0.5,
        }
    }

    /// Trigeorgis tree (additive equal jumps).
    ///
    /// `dx = √(σ²Δt + μ²Δt²)`, `p_up = ½ + ½ μΔt / dx`.
    /// Drift is captured entirely by the probabilities, not in node values.
    pub fn trigeorgis(process: &dyn StochasticProcess1D, end: Real, steps: usize) -> Self {
        let (x0, dt, dps, _std) = log_params(process, end, steps);
        let var = log_variance(process, dt);
        let dx = (var + dps * dps).sqrt();
        let pu = 0.5 + 0.5 * dps / dx;
        let pd = 1.0 - pu;
        assert!(
            pu >= 0.0 && pu <= 1.0,
            "Trigeorgis: invalid probability {pu}"
        );
        Self {
            x0,
            dt,
            steps,
            drift_per_step: 0.0,
            underlying: UnderlyingKind::LogSpace { step: dx },
            pu,
            pd,
        }
    }

    /// Tian tree: third-moment matching, multiplicative.
    ///
    /// Matches the first three moments of the log-normal distribution.
    pub fn tian(process: &dyn StochasticProcess1D, end: Real, steps: usize) -> Self {
        let (x0, dt, dps, _std) = log_params(process, end, steps);
        let var = log_variance(process, dt);
        let q = var.exp(); // exp(σ²Δt)
        let r_m = dps.exp() * q.sqrt(); // exp(drift + σ²Δt/2) = exp((r-q)Δt)
        let up = 0.5 * r_m * q * (q + 1.0 + (q * q + 2.0 * q - 3.0).sqrt());
        let down = 0.5 * r_m * q * (q + 1.0 - (q * q + 2.0 * q - 3.0).sqrt());
        let pu = (r_m - down) / (up - down);
        let pd = 1.0 - pu;
        Self {
            x0,
            dt,
            steps,
            drift_per_step: dps,
            underlying: UnderlyingKind::Multiplicative { up, down },
            pu,
            pd,
        }
    }

    /// Leisen-Reimer tree: multiplicative, strike-dependent.
    ///
    /// Uses the Peizer-Pratt Method 2 inversion for improved convergence.
    ///
    /// # Panics
    /// Panics if `strike <= 0`.
    pub fn leisen_reimer(
        process: &dyn StochasticProcess1D,
        end: Real,
        steps: usize,
        strike: Real,
    ) -> Self {
        assert!(strike > 0.0, "strike must be positive");
        let odd_steps = if steps % 2 != 0 { steps } else { steps + 1 };
        let x0 = process.x0();
        let dt = end / odd_steps as Real;
        let dps = process.drift_1d(0.0, x0) * dt / x0;
        let total_var = log_total_variance(process, end);
        let ermqdt = (dps + 0.5 * total_var / odd_steps as Real).exp();
        let d2 = ((x0 / strike).ln() + dps * odd_steps as Real) / total_var.sqrt();

        let pu = peizer_pratt_2(d2, odd_steps);
        let pd = 1.0 - pu;
        let pdash = peizer_pratt_2(d2 + total_var.sqrt(), odd_steps);
        let up = ermqdt * pdash / pu;
        let down = (ermqdt - pu * up) / (1.0 - pu);
        Self {
            x0,
            dt,
            steps: odd_steps,
            drift_per_step: dps,
            underlying: UnderlyingKind::Multiplicative { up, down },
            pu,
            pd,
        }
    }

    /// Joshi 4th-order tree: multiplicative, strike-dependent.
    ///
    /// Fourth-order convergence in the number of steps.
    ///
    /// # Panics
    /// Panics if `strike <= 0`.
    pub fn joshi4(
        process: &dyn StochasticProcess1D,
        end: Real,
        steps: usize,
        strike: Real,
    ) -> Self {
        assert!(strike > 0.0, "strike must be positive");
        let odd_steps = if steps % 2 != 0 { steps } else { steps + 1 };
        let x0 = process.x0();
        let dt = end / odd_steps as Real;
        let dps = process.drift_1d(0.0, x0) * dt / x0;
        let total_var = log_total_variance(process, end);
        let ermqdt = (dps + 0.5 * total_var / odd_steps as Real).exp();
        let d2 = ((x0 / strike).ln() + dps * odd_steps as Real) / total_var.sqrt();

        let pu = joshi4_up_prob((odd_steps as Real - 1.0) / 2.0, d2);
        let pd = 1.0 - pu;
        let pdash =
            joshi4_up_prob((odd_steps as Real - 1.0) / 2.0, d2 + total_var.sqrt());
        let up = ermqdt * pdash / pu;
        let down = (ermqdt - pu * up) / (1.0 - pu);
        Self {
            x0,
            dt,
            steps: odd_steps,
            drift_per_step: dps,
            underlying: UnderlyingKind::Multiplicative { up, down },
            pu,
            pd,
        }
    }
}

// ─── Helper functions ─────────────────────────────────────────────────────────

/// Extract log-space parameters from a StochasticProcess1D.
///
/// Returns `(x0, dt, drift_per_step, std_dev_per_step)` in log-space.
fn log_params(
    process: &dyn StochasticProcess1D,
    end: Real,
    steps: usize,
) -> (Real, Real, Real, Real) {
    let x0 = process.x0();
    let dt = end / steps as Real;
    // Our Rust BSM process models S directly: drift_1d = (r-q-σ²/2)*x,
    // diffusion_1d = σ*x. Dividing by x0 gives the log-space parameters.
    let drift_per_step = process.drift_1d(0.0, x0) * dt / x0;
    let std_dev = process.std_deviation_1d(0.0, x0, dt) / x0;
    (x0, dt, drift_per_step, std_dev)
}

/// Log-space variance per step: σ²·Δt.
fn log_variance(process: &dyn StochasticProcess1D, dt: Real) -> Real {
    let x0 = process.x0();
    process.variance_1d(0.0, x0, dt) / (x0 * x0)
}

/// Total log-space variance over the full maturity: σ²·T.
fn log_total_variance(process: &dyn StochasticProcess1D, end: Real) -> Real {
    let x0 = process.x0();
    process.variance_1d(0.0, x0, end) / (x0 * x0)
}

/// Peizer-Pratt Method 2 inversion.
///
/// Maps a normal quantile `z` to a probability `p ∈ [0, 1]` for an `n`-step
/// binomial approximation. Requires `n` to be odd.
fn peizer_pratt_2(z: Real, n: usize) -> Real {
    let nf = n as Real;
    let r = z / (nf + 1.0 / 3.0 + 0.1 / (nf + 1.0));
    let ex = (-r * r * (nf + 1.0 / 6.0)).exp();
    0.5 + z.signum() * 0.5 * (1.0 - ex).sqrt()
}

/// Joshi 4th-order up-probability.
///
/// Higher-order correction to the Peizer-Pratt formula.
fn joshi4_up_prob(k: Real, dj: Real) -> Real {
    let alpha = dj / (8.0_f64).sqrt();
    let alpha2 = alpha * alpha;
    let alpha3 = alpha * alpha2;
    let alpha5 = alpha3 * alpha2;
    let alpha7 = alpha5 * alpha2;
    let beta = -0.375 * alpha - alpha3;
    let gamma = (5.0 / 6.0) * alpha5 + (13.0 / 12.0) * alpha3 + (25.0 / 128.0) * alpha;
    let delta = -0.1025 * alpha - 0.9285 * alpha3 - 1.43 * alpha5 - 0.5 * alpha7;
    let rootk = k.sqrt();
    let mut p = 0.5;
    p += alpha / rootk;
    p += beta / (k * rootk);
    p += gamma / (k * k * rootk);
    p += delta / (k * k * k * rootk);
    p
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ql_processes::GeneralizedBlackScholesProcess;
    use ql_termstructures::{BlackConstantVol, FlatForward};
    use ql_time::{Actual365Fixed, Date};
    use std::sync::Arc;

    /// Build a test BSM process: S=100, r=5%, q=0%, σ=20%.
    fn test_process() -> GeneralizedBlackScholesProcess {
        let ref_date = Date::from_ymd(2025, 1, 15).unwrap();
        let rf = Arc::new(FlatForward::continuous(ref_date, 0.05, Actual365Fixed));
        let div = Arc::new(FlatForward::continuous(ref_date, 0.0, Actual365Fixed));
        let vol = Arc::new(BlackConstantVol::new(ref_date, 0.20, Actual365Fixed));
        GeneralizedBlackScholesProcess::new(100.0, rf, div, vol)
    }

    /// BS price for reference: ATM call S=100, K=100, r=5%, σ=20%, T=1.
    fn bs_call_reference() -> Real {
        use ql_pricingengines::analytic_european_engine::black_scholes_merton;
        let (price, ..) =
            black_scholes_merton(ql_instruments::OptionType::Call, 100.0, 100.0, 0.05, 0.0, 0.20, 1.0);
        price
    }

    fn call_payoff(s: Real) -> Real {
        (s - 100.0_f64).max(0.0)
    }

    #[test]
    fn crr_european_call_converges_to_bs() {
        let process = test_process();
        let bs = bs_call_reference();
        let tree = BinomialTree::cox_ross_rubinstein(&process, 1.0, 500);
        let discount = (-0.05 * tree.dt()).exp();
        let price = crate::lattice::price_european(&tree, &call_payoff, discount);
        assert!(
            (price - bs).abs() < 0.10,
            "CRR({} steps): {price:.4} vs BS {bs:.4}",
            tree.steps()
        );
    }

    #[test]
    fn jr_european_call_converges_to_bs() {
        let process = test_process();
        let bs = bs_call_reference();
        let tree = BinomialTree::jarrow_rudd(&process, 1.0, 500);
        let discount = (-0.05 * tree.dt()).exp();
        let price = crate::lattice::price_european(&tree, &call_payoff, discount);
        assert!(
            (price - bs).abs() < 0.10,
            "JR({} steps): {price:.4} vs BS {bs:.4}",
            tree.steps()
        );
    }

    #[test]
    fn trigeorgis_european_call_converges_to_bs() {
        let process = test_process();
        let bs = bs_call_reference();
        let tree = BinomialTree::trigeorgis(&process, 1.0, 500);
        let discount = (-0.05 * tree.dt()).exp();
        let price = crate::lattice::price_european(&tree, &call_payoff, discount);
        assert!(
            (price - bs).abs() < 0.10,
            "Trigeorgis({} steps): {price:.4} vs BS {bs:.4}",
            tree.steps()
        );
    }

    #[test]
    fn tian_european_call_converges_to_bs() {
        let process = test_process();
        let bs = bs_call_reference();
        let tree = BinomialTree::tian(&process, 1.0, 500);
        let discount = (-0.05 * tree.dt()).exp();
        let price = crate::lattice::price_european(&tree, &call_payoff, discount);
        assert!(
            (price - bs).abs() < 0.10,
            "Tian({} steps): {price:.4} vs BS {bs:.4}",
            tree.steps()
        );
    }

    #[test]
    fn leisen_reimer_fast_convergence() {
        let process = test_process();
        let bs = bs_call_reference();
        // LR converges fast — even 51 steps should be accurate
        let tree = BinomialTree::leisen_reimer(&process, 1.0, 51, 100.0);
        let discount = (-0.05 * tree.dt()).exp();
        let price = crate::lattice::price_european(&tree, &call_payoff, discount);
        assert!(
            (price - bs).abs() < 0.05,
            "LR({} steps): {price:.4} vs BS {bs:.4}",
            tree.steps()
        );
    }

    #[test]
    fn joshi4_fast_convergence() {
        let process = test_process();
        let bs = bs_call_reference();
        let tree = BinomialTree::joshi4(&process, 1.0, 51, 100.0);
        let discount = (-0.05 * tree.dt()).exp();
        let price = crate::lattice::price_european(&tree, &call_payoff, discount);
        assert!(
            (price - bs).abs() < 0.05,
            "Joshi4({} steps): {price:.4} vs BS {bs:.4}",
            tree.steps()
        );
    }

    #[test]
    fn american_put_geq_european_put() {
        let process = test_process();
        let tree = BinomialTree::cox_ross_rubinstein(&process, 1.0, 200);
        let discount = (-0.05 * tree.dt()).exp();
        let payoff = |s: Real| (100.0 - s).max(0.0);

        let eu = crate::lattice::price_european(&tree, &payoff, discount);
        let am = crate::lattice::price_american(&tree, &payoff, discount);

        assert!(
            am >= eu - 1e-10,
            "American put {am:.4} < European put {eu:.4}"
        );
    }

    #[test]
    fn additive_eqp_european_call_converges_to_bs() {
        let process = test_process();
        let bs = bs_call_reference();
        let tree = BinomialTree::additive_eqp(&process, 1.0, 500);
        let discount = (-0.05 * tree.dt()).exp();
        let price = crate::lattice::price_european(&tree, &call_payoff, discount);
        assert!(
            (price - bs).abs() < 0.10,
            "AdditiveEQP({} steps): {price:.4} vs BS {bs:.4}",
            tree.steps()
        );
    }
}
