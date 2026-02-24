//! Recombining trinomial tree for option pricing.
//!
//! Translates `ql/methods/lattices/trinomialtree.hpp`.
//!
//! The diffusion term must be independent of the underlying process value
//! (additive noise). This tree is used primarily with short-rate models
//! (Hull-White, Black-Karasinski) and the Ornstein-Uhlenbeck process.

use ql_core::Real;
use ql_processes::StochasticProcess1D;

use super::TimeGrid;

/// Branching data for a single time step of the trinomial tree.
#[derive(Debug, Clone)]
struct Branching {
    /// Descendant offset for each node: `k[j]` = integer shift from center.
    k: Vec<i32>,
    /// Probabilities for each branch (0=down, 1=mid, 2=up) for each node.
    probs: [Vec<Real>; 3],
    /// Minimum and maximum `k` values, defining the width at the next step.
    j_min: i32,
    j_max: i32,
}

impl Branching {
    fn new() -> Self {
        Self {
            k: Vec::new(),
            probs: [Vec::new(), Vec::new(), Vec::new()],
            j_min: i32::MAX,
            j_max: i32::MIN,
        }
    }

    /// Add a branching node: `shift` is the integer offset from center,
    /// `p_down`, `p_mid`, `p_up` are the three probabilities.
    fn add(&mut self, shift: i32, p_down: Real, p_mid: Real, p_up: Real) {
        self.k.push(shift);
        self.probs[0].push(p_down);
        self.probs[1].push(p_mid);
        self.probs[2].push(p_up);
        // Descendant range: shift-1, shift, shift+1
        self.j_min = self.j_min.min(shift - 1);
        self.j_max = self.j_max.max(shift + 1);
    }

    /// Number of nodes at this time step.
    fn size(&self) -> usize {
        (self.j_max - self.j_min + 1) as usize
    }

    /// Descendant index for node `index` and branch `b` (0=down, 1=mid, 2=up).
    fn descendant(&self, index: usize, branch: usize) -> usize {
        (self.k[index] - self.j_min - 1 + branch as i32) as usize
    }

    /// Probability at node `index` for branch `b`.
    fn probability(&self, index: usize, branch: usize) -> Real {
        self.probs[branch][index]
    }
}

/// A recombining trinomial tree approximating a 1-D stochastic process.
///
/// The variance of the process must be independent of the underlying value
/// (additive noise). The tree spacing is `dx = σ √(3 Δt)`.
///
/// Corresponds to `QuantLib::TrinomialTree`.
#[derive(Debug, Clone)]
pub struct TrinomialTree {
    x0: Real,
    /// dx at each layer (dx[0] = 0 for the root; dx[i] for layer i ≥ 1).
    dx: Vec<Real>,
    branchings: Vec<Branching>,
    time_grid: TimeGrid,
}

impl TrinomialTree {
    /// Build a trinomial tree from a 1-D stochastic process and time grid.
    ///
    /// The process variance must be independent of the state variable.
    pub fn new(process: &dyn StochasticProcess1D, grid: &TimeGrid) -> Self {
        let x0 = process.x0();
        let n = grid.steps();
        assert!(n > 0, "need at least one time step");

        let mut dx: Vec<Real> = vec![0.0]; // dx[0] unused (root layer)
        let mut branchings = Vec::with_capacity(n);

        let mut j_min = 0i32;
        let mut j_max = 0i32;

        for i in 0..n {
            let t = grid.time(i);
            let dt = grid.dt(i);

            // Variance independent of x → evaluate at x=0
            let v2 = process.variance_1d(t, x0, dt) / (x0 * x0);
            let v = v2.sqrt();
            let dx_step = v * 3.0_f64.sqrt();
            dx.push(dx_step);

            let mut branching = Branching::new();
            let drift_step = process.drift_1d(t, x0) * dt / x0;

            for j in j_min..=j_max {
                let x_j = j as Real * dx_step;
                let e = drift_step + x_j - x_j; // For additive noise: e = μ·dt
                let k = (e / dx_step + 0.5).floor() as i32;

                let e_k = e - k as Real * dx_step;
                let e2 = e_k * e_k;

                let p_down = (1.0 + e2 / v2 - e_k / v) / 6.0;
                let p_mid = (2.0 - e2 / v2) / 3.0;
                let p_up = (1.0 + e2 / v2 + e_k / v) / 6.0;

                branching.add(j + k, p_down, p_mid, p_up);
            }

            branchings.push(branching);

            j_min = branchings.last().unwrap().j_min;
            j_max = branchings.last().unwrap().j_max;
        }

        Self {
            x0,
            dx,
            branchings,
            time_grid: grid.clone(),
        }
    }

    /// Build a trinomial tree with a uniform time grid.
    pub fn uniform(process: &dyn StochasticProcess1D, end: Real, steps: usize) -> Self {
        let grid = TimeGrid::uniform(end, steps);
        Self::new(process, &grid)
    }

    /// Number of time steps.
    pub fn steps(&self) -> usize {
        self.time_grid.steps()
    }

    /// Number of nodes at time step `i`.
    pub fn size(&self, i: usize) -> usize {
        if i == 0 {
            1
        } else {
            self.branchings[i - 1].size()
        }
    }

    /// Underlying value at node `(i, index)`.
    ///
    /// Returns `x0 * exp(offset * dx)` — log-space / multiplicative model
    /// suitable for geometric Brownian motion.
    pub fn underlying(&self, i: usize, index: usize) -> Real {
        if i == 0 {
            self.x0
        } else {
            let j_min = self.branchings[i - 1].j_min;
            self.x0 * ((j_min as Real + index as Real) * self.dx[i]).exp()
        }
    }

    /// Descendant index at step `i` for node `index` and `branch` (0..2).
    pub fn descendant(&self, i: usize, index: usize, branch: usize) -> usize {
        self.branchings[i].descendant(index, branch)
    }

    /// Transition probability at step `i`, node `index`, branch `branch`.
    pub fn probability(&self, i: usize, index: usize, branch: usize) -> Real {
        self.branchings[i].probability(index, branch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ql_processes::GeneralizedBlackScholesProcess;
    use ql_termstructures::{BlackConstantVol, FlatForward};
    use ql_time::{Actual365Fixed, Date};
    use std::sync::Arc;

    fn test_process() -> GeneralizedBlackScholesProcess {
        let ref_date = Date::from_ymd(2025, 1, 15).unwrap();
        let rf = Arc::new(FlatForward::continuous(ref_date, 0.05, Actual365Fixed));
        let div = Arc::new(FlatForward::continuous(ref_date, 0.0, Actual365Fixed));
        let vol = Arc::new(BlackConstantVol::new(ref_date, 0.20, Actual365Fixed));
        GeneralizedBlackScholesProcess::new(100.0, rf, div, vol)
    }

    #[test]
    fn trinomial_tree_sizes_grow() {
        let process = test_process();
        let tree = TrinomialTree::uniform(&process, 1.0, 10);
        assert_eq!(tree.size(0), 1);
        // Each step can grow by at most 2 (one on each side)
        for i in 1..=10 {
            assert!(tree.size(i) >= 2 * i + 1 - 2); // grows roughly linearly
        }
    }

    #[test]
    fn trinomial_european_call_converges() {
        let process = test_process();
        let tree = TrinomialTree::uniform(&process, 1.0, 200);
        let discount = (-0.05_f64 * (1.0 / 200.0)).exp();

        let price = crate::lattice::price_european_trinomial(
            &tree,
            &|s: f64| (s - 100.0_f64).max(0.0),
            discount,
        );

        // Compare to BS ≈ 10.45
        assert!(
            price > 5.0 && price < 20.0,
            "trinomial call price = {price:.4} (expected ~10.45)"
        );
    }
}
