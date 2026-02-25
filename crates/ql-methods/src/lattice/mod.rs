//! Lattice methods for option pricing.
//!
//! Translates `ql/methods/lattices/` — binomial trees, trinomial trees,
//! and the backward-induction lattice framework.
//!
//! # Overview
//!
//! * [`BinomialTree`] — recombining binomial tree with 7 classical variants
//!   (Jarrow-Rudd, CRR, Trigeorgis, Tian, Leisen-Reimer, Joshi4, Additive EQP)
//! * [`TrinomialTree`] — recombining trinomial tree
//! * [`TimeGrid`] — grid of time points used by tree methods
//! * [`price_european`] / [`price_american`] — backward-induction pricing

pub mod binomial_tree;
pub mod trinomial_tree;

pub use binomial_tree::BinomialTree;
pub use trinomial_tree::TrinomialTree;

use ql_core::Real;

// ─── TimeGrid ─────────────────────────────────────────────────────────────────

/// A grid of time points used by lattice methods.
///
/// Corresponds to `QuantLib::TimeGrid`.
#[derive(Debug, Clone)]
pub struct TimeGrid {
    times: Vec<Real>,
    dts: Vec<Real>,
}

impl TimeGrid {
    /// Create a uniform time grid from 0 to `end` with `steps` intervals.
    pub fn uniform(end: Real, steps: usize) -> Self {
        assert!(steps > 0, "steps must be > 0");
        let dt = end / steps as Real;
        let times: Vec<Real> = (0..=steps).map(|i| i as Real * dt).collect();
        let dts = vec![dt; steps];
        Self { times, dts }
    }

    /// Create from a set of mandatory time points, ensuring at least `min_steps`
    /// total intervals.
    pub fn from_times(mandatory: &[Real], min_steps: usize) -> Self {
        let mut all_times: Vec<Real> = vec![0.0];
        all_times.extend_from_slice(mandatory);
        all_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        all_times.dedup_by(|a, b| (*a - *b).abs() < 1e-12);

        let end = *all_times.last().unwrap();
        if min_steps > all_times.len() - 1 {
            let dt = end / min_steps as Real;
            for i in 1..=min_steps {
                let t = i as Real * dt;
                if all_times.iter().all(|&x| (x - t).abs() > 1e-12) {
                    all_times.push(t);
                }
            }
            all_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        }

        let dts: Vec<Real> = all_times.windows(2).map(|w| w[1] - w[0]).collect();
        Self {
            times: all_times,
            dts,
        }
    }

    /// Number of time points (= steps + 1).
    pub fn size(&self) -> usize {
        self.times.len()
    }

    /// Number of steps (= time points − 1).
    pub fn steps(&self) -> usize {
        self.times.len() - 1
    }

    /// Time at index `i`.
    pub fn time(&self, i: usize) -> Real {
        self.times[i]
    }

    /// Time step between index `i` and `i+1`.
    pub fn dt(&self, i: usize) -> Real {
        self.dts[i]
    }

    /// Final time.
    pub fn end(&self) -> Real {
        *self.times.last().unwrap()
    }

    /// All time points.
    pub fn times(&self) -> &[Real] {
        &self.times
    }
}

// ─── Backward-induction pricing ───────────────────────────────────────────────

/// Price a European option by backward induction on a binomial tree.
///
/// # Arguments
/// * `tree` — the binomial tree (already constructed)
/// * `payoff` — payoff function `S → value` (e.g. `|s| (s - K).max(0.0)`)
/// * `discount` — per-step discount factor, typically `exp(−r · Δt)`
pub fn price_european(tree: &BinomialTree, payoff: &dyn Fn(Real) -> Real, discount: Real) -> Real {
    let n = tree.steps();

    // Terminal values at maturity
    let mut values: Vec<Real> = (0..tree.size(n))
        .map(|j| payoff(tree.underlying(n, j)))
        .collect();

    // Roll back through the tree
    for i in (0..n).rev() {
        for j in 0..tree.size(i) {
            let pu = tree.probability(i, j, 1); // up branch
            let pd = tree.probability(i, j, 0); // down branch
            let d_up = tree.descendant(i, j, 1);
            let d_down = tree.descendant(i, j, 0);
            values[j] = discount * (pu * values[d_up] + pd * values[d_down]);
        }
    }

    values[0]
}

/// Price an American option by backward induction on a binomial tree.
///
/// Same as European pricing, but allows early exercise at every node.
pub fn price_american(tree: &BinomialTree, payoff: &dyn Fn(Real) -> Real, discount: Real) -> Real {
    let n = tree.steps();

    let mut values: Vec<Real> = (0..tree.size(n))
        .map(|j| payoff(tree.underlying(n, j)))
        .collect();

    for i in (0..n).rev() {
        for j in 0..tree.size(i) {
            let pu = tree.probability(i, j, 1);
            let pd = tree.probability(i, j, 0);
            let d_up = tree.descendant(i, j, 1);
            let d_down = tree.descendant(i, j, 0);
            let hold = discount * (pu * values[d_up] + pd * values[d_down]);
            let exercise = payoff(tree.underlying(i, j));
            values[j] = hold.max(exercise);
        }
    }

    values[0]
}

/// Price a European option on a trinomial tree by backward induction.
#[allow(clippy::needless_range_loop)]
pub fn price_european_trinomial(
    tree: &TrinomialTree,
    payoff: &dyn Fn(Real) -> Real,
    discount: Real,
) -> Real {
    let n = tree.steps();

    let mut values: Vec<Real> = (0..tree.size(n))
        .map(|j| payoff(tree.underlying(n, j)))
        .collect();

    for i in (0..n).rev() {
        let mut new_values = vec![0.0; tree.size(i)];
        for j in 0..tree.size(i) {
            let pd = tree.probability(i, j, 0);
            let pm = tree.probability(i, j, 1);
            let pu = tree.probability(i, j, 2);
            let d_down = tree.descendant(i, j, 0);
            let d_mid = tree.descendant(i, j, 1);
            let d_up = tree.descendant(i, j, 2);
            new_values[j] =
                discount * (pd * values[d_down] + pm * values[d_mid] + pu * values[d_up]);
        }
        values = new_values;
    }

    values[0]
}

/// Price an American option on a trinomial tree by backward induction.
#[allow(clippy::needless_range_loop)]
pub fn price_american_trinomial(
    tree: &TrinomialTree,
    payoff: &dyn Fn(Real) -> Real,
    discount: Real,
) -> Real {
    let n = tree.steps();

    let mut values: Vec<Real> = (0..tree.size(n))
        .map(|j| payoff(tree.underlying(n, j)))
        .collect();

    for i in (0..n).rev() {
        let mut new_values = vec![0.0; tree.size(i)];
        for j in 0..tree.size(i) {
            let pd = tree.probability(i, j, 0);
            let pm = tree.probability(i, j, 1);
            let pu = tree.probability(i, j, 2);
            let d_down = tree.descendant(i, j, 0);
            let d_mid = tree.descendant(i, j, 1);
            let d_up = tree.descendant(i, j, 2);
            let hold = discount * (pd * values[d_down] + pm * values[d_mid] + pu * values[d_up]);
            let exercise = payoff(tree.underlying(i, j));
            new_values[j] = hold.max(exercise);
        }
        values = new_values;
    }

    values[0]
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_grid_uniform() {
        let g = TimeGrid::uniform(1.0, 4);
        assert_eq!(g.size(), 5);
        assert_eq!(g.steps(), 4);
        assert!((g.time(0) - 0.0).abs() < 1e-15);
        assert!((g.time(4) - 1.0).abs() < 1e-15);
        assert!((g.dt(0) - 0.25).abs() < 1e-15);
    }

    #[test]
    fn time_grid_from_mandatory_times() {
        let g = TimeGrid::from_times(&[0.5, 1.0], 4);
        assert!(g.steps() >= 4);
        // Must contain 0.0, 0.5, and 1.0
        assert!(g.times().contains(&0.0));
        assert!(g.times().iter().any(|&t| (t - 0.5).abs() < 1e-12));
        assert!(g.times().iter().any(|&t| (t - 1.0).abs() < 1e-12));
    }
}
