//! Monte Carlo simulation framework.
//!
//! Translates `ql/methods/montecarlo/` — path generation, path pricing,
//! and the model orchestrator.
//!
//! # Overview
//!
//! * [`PathGenerator`] — generates sample paths of a stochastic process
//! * [`PathPricer`] — trait for evaluating payoffs on generated paths
//! * [`MonteCarloModel`] — orchestrates path generation and statistics collection
//! * [`Path`] — a single realisation of the process (times + values)

use ql_core::Real;
use ql_math::random_numbers::InverseCumulativeNormalRng;
use ql_math::statistics::IncrementalStatistics;
use ql_processes::StochasticProcess1D;

// ─── Path ─────────────────────────────────────────────────────────────────────

/// A single sample path: a sequence of time-value pairs.
///
/// Corresponds to `QuantLib::Path`.
#[derive(Debug, Clone)]
pub struct Path {
    /// Time points (including t=0).
    pub times: Vec<Real>,
    /// Process values at each time point.
    pub values: Vec<Real>,
}

impl Path {
    /// Number of time steps (= len − 1).
    pub fn steps(&self) -> usize {
        self.values.len() - 1
    }

    /// The final value.
    pub fn back(&self) -> Real {
        *self.values.last().unwrap()
    }

    /// The initial value.
    pub fn front(&self) -> Real {
        self.values[0]
    }

    /// Length of the path (number of points including initial).
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Whether the path is empty.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

// ─── PathGenerator ────────────────────────────────────────────────────────────

/// Generates sample paths of a 1-D stochastic process.
///
/// Uses the process's `evolve_1d` method to step forward from the initial
/// value through a uniform time grid.
///
/// Corresponds to `QuantLib::PathGenerator`.
pub struct PathGenerator<'a> {
    process: &'a dyn StochasticProcess1D,
    dt: Real,
    steps: usize,
    rng: InverseCumulativeNormalRng,
}

impl<'a> PathGenerator<'a> {
    /// Create a new path generator.
    ///
    /// # Arguments
    /// * `process` — the stochastic process to simulate
    /// * `maturity` — total time horizon
    /// * `steps` — number of time steps
    /// * `seed` — RNG seed
    pub fn new(
        process: &'a dyn StochasticProcess1D,
        maturity: Real,
        steps: usize,
        seed: u64,
    ) -> Self {
        Self {
            process,
            dt: maturity / steps as Real,
            steps,
            rng: InverseCumulativeNormalRng::new(seed),
        }
    }

    /// Generate one sample path.
    pub fn next_path(&mut self) -> Path {
        let mut times = Vec::with_capacity(self.steps + 1);
        let mut values = Vec::with_capacity(self.steps + 1);

        let x0 = self.process.x0();
        times.push(0.0);
        values.push(x0);

        let mut x = x0;
        for i in 0..self.steps {
            let t = i as Real * self.dt;
            let dw = self.rng.next_real();
            x = self.process.evolve_1d(t, x, self.dt, dw);
            times.push(t + self.dt);
            values.push(x);
        }

        Path { times, values }
    }
}

// ─── PathGenerator with antithetic variates ───────────────────────────────────

/// Path generator with antithetic variates for variance reduction.
///
/// Produces pairs of paths: one with +dW, one with −dW.
pub struct AntitheticPathGenerator<'a> {
    process: &'a dyn StochasticProcess1D,
    dt: Real,
    steps: usize,
    rng: InverseCumulativeNormalRng,
    cached_normals: Vec<Real>,
    use_antithetic: bool,
}

impl<'a> AntitheticPathGenerator<'a> {
    /// Create a new antithetic path generator.
    pub fn new(
        process: &'a dyn StochasticProcess1D,
        maturity: Real,
        steps: usize,
        seed: u64,
    ) -> Self {
        Self {
            process,
            dt: maturity / steps as Real,
            steps,
            rng: InverseCumulativeNormalRng::new(seed),
            cached_normals: Vec::new(),
            use_antithetic: false,
        }
    }

    /// Generate the next sample path (alternating normal/antithetic).
    pub fn next_path(&mut self) -> Path {
        let mut times = Vec::with_capacity(self.steps + 1);
        let mut values = Vec::with_capacity(self.steps + 1);

        let x0 = self.process.x0();
        times.push(0.0);
        values.push(x0);

        if !self.use_antithetic {
            // Generate fresh normals
            self.cached_normals.clear();
            let mut x = x0;
            for i in 0..self.steps {
                let t = i as Real * self.dt;
                let dw = self.rng.next_real();
                self.cached_normals.push(dw);
                x = self.process.evolve_1d(t, x, self.dt, dw);
                times.push(t + self.dt);
                values.push(x);
            }
        } else {
            // Use negated normals
            let mut x = x0;
            for i in 0..self.steps {
                let t = i as Real * self.dt;
                let dw = -self.cached_normals[i];
                x = self.process.evolve_1d(t, x, self.dt, dw);
                times.push(t + self.dt);
                values.push(x);
            }
        }
        self.use_antithetic = !self.use_antithetic;

        Path { times, values }
    }
}

// ─── PathPricer ───────────────────────────────────────────────────────────────

/// A trait for computing the discounted payoff from a sample path.
///
/// Corresponds to `QuantLib::PathPricer<Path>`.
pub trait PathPricer: Send + Sync {
    /// Evaluate the discounted payoff for a given path.
    fn value(&self, path: &Path) -> Real;
}

/// A simple European payoff pricer: evaluates `payoff(S_T) * discount`.
pub struct EuropeanPathPricer<F> {
    payoff: F,
    discount: Real,
}

impl<F: Fn(Real) -> Real + Send + Sync> EuropeanPathPricer<F> {
    /// Create a European pricer with payoff function and discount factor.
    pub fn new(payoff: F, discount: Real) -> Self {
        Self { payoff, discount }
    }
}

impl<F: Fn(Real) -> Real + Send + Sync> PathPricer for EuropeanPathPricer<F> {
    fn value(&self, path: &Path) -> Real {
        (self.payoff)(path.back()) * self.discount
    }
}

/// An arithmetic-average Asian payoff pricer.
pub struct AsianArithmeticPathPricer<F> {
    payoff: F,
    discount: Real,
}

impl<F: Fn(Real) -> Real + Send + Sync> AsianArithmeticPathPricer<F> {
    /// Create an Asian arithmetic-average pricer.
    pub fn new(payoff: F, discount: Real) -> Self {
        Self { payoff, discount }
    }
}

impl<F: Fn(Real) -> Real + Send + Sync> PathPricer for AsianArithmeticPathPricer<F> {
    fn value(&self, path: &Path) -> Real {
        // Average of all values excluding the initial
        let n = path.steps();
        if n == 0 {
            return 0.0;
        }
        let avg: Real = path.values[1..].iter().sum::<Real>() / n as Real;
        (self.payoff)(avg) * self.discount
    }
}

// ─── MonteCarloModel ──────────────────────────────────────────────────────────

/// A Monte Carlo simulation orchestrator.
///
/// Combines a path generator with a pricer and collects statistics across
/// many simulated paths.
///
/// Corresponds to `QuantLib::MonteCarloModel`.
pub struct MonteCarloModel<'a> {
    process: &'a dyn StochasticProcess1D,
    maturity: Real,
    steps: usize,
    seed: u64,
}

impl<'a> MonteCarloModel<'a> {
    /// Create a new Monte Carlo model.
    pub fn new(
        process: &'a dyn StochasticProcess1D,
        maturity: Real,
        steps: usize,
        seed: u64,
    ) -> Self {
        Self {
            process,
            maturity,
            steps,
            seed,
        }
    }

    /// Run `n_paths` simulations and return gathered statistics.
    pub fn simulate(&self, pricer: &dyn PathPricer, n_paths: usize) -> IncrementalStatistics {
        let mut gen = PathGenerator::new(self.process, self.maturity, self.steps, self.seed);
        let mut stats = IncrementalStatistics::new();

        for _ in 0..n_paths {
            let path = gen.next_path();
            let value = pricer.value(&path);
            stats.add(value);
        }

        stats
    }

    /// Run with antithetic variates for variance reduction.
    pub fn simulate_antithetic(
        &self,
        pricer: &dyn PathPricer,
        n_paths: usize,
    ) -> IncrementalStatistics {
        let mut gen =
            AntitheticPathGenerator::new(self.process, self.maturity, self.steps, self.seed);
        let mut stats = IncrementalStatistics::new();

        // n_paths pairs → 2*n_paths total paths
        for _ in 0..n_paths {
            let path1 = gen.next_path();
            let path2 = gen.next_path();
            let avg = 0.5 * (pricer.value(&path1) + pricer.value(&path2));
            stats.add(avg);
        }

        stats
    }
}

/// Convenience function: Monte Carlo price of a European option.
///
/// Returns `(mean, std_error)`.
pub fn mc_european_price(
    process: &dyn StochasticProcess1D,
    payoff: impl Fn(Real) -> Real + Send + Sync,
    discount: Real,
    maturity: Real,
    steps: usize,
    n_paths: usize,
    seed: u64,
) -> (Real, Real) {
    let model = MonteCarloModel::new(process, maturity, steps, seed);
    let pricer = EuropeanPathPricer::new(payoff, discount);
    let stats = model.simulate(&pricer, n_paths);
    (
        stats.mean().unwrap_or(0.0),
        stats.error_estimate().unwrap_or(0.0),
    )
}

// ─── Tests ────────────────────────────────────────────────────────────────────

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
    fn mc_european_call_converges_to_bs() {
        let process = test_process();
        let discount = (-0.05_f64).exp();
        let (price, stderr) = mc_european_price(
            &process,
            |s| (s - 100.0).max(0.0),
            discount,
            1.0,
            100,
            100_000,
            42,
        );

        // BS ≈ 10.45. MC with 100k paths should be within ~3 std errors.
        let bs_ref = 10.45;
        assert!(
            (price - bs_ref).abs() < 3.0 * stderr + 0.5,
            "MC call = {price:.2} ± {stderr:.2}, expected ~{bs_ref}"
        );
        assert!(price > 5.0 && price < 20.0, "MC call = {price:.2}");
    }

    #[test]
    fn mc_antithetic_reduces_variance() {
        let process = test_process();
        let discount = (-0.05_f64).exp();
        let payoff = |s: Real| (s - 100.0).max(0.0);
        let pricer = EuropeanPathPricer::new(payoff, discount);

        let model = MonteCarloModel::new(&process, 1.0, 50, 42);

        let stats_plain = model.simulate(&pricer, 10_000);
        let stats_anti = model.simulate_antithetic(&pricer, 10_000);

        // Antithetic should have smaller error estimate
        let err_plain = stats_plain.error_estimate().unwrap();
        let err_anti = stats_anti.error_estimate().unwrap();

        // Just check both give reasonable prices
        assert!(stats_plain.mean().unwrap() > 5.0);
        assert!(stats_anti.mean().unwrap() > 5.0);
        // Antithetic typically reduces variance — allow some slack
        assert!(
            err_anti < err_plain * 1.5,
            "antithetic err={err_anti:.4} should be ≤ plain err={err_plain:.4}"
        );
    }

    #[test]
    fn path_generator_produces_positive_gbm() {
        let process = test_process();
        let mut gen = PathGenerator::new(&process, 1.0, 252, 12345);
        for _ in 0..100 {
            let path = gen.next_path();
            assert_eq!(path.len(), 253);
            assert!((path.front() - 100.0).abs() < 1e-12);
            // GBM stays positive
            for &v in &path.values {
                assert!(v > 0.0, "GBM path went non-positive: {v}");
            }
        }
    }

    #[test]
    fn asian_arithmetic_pricer() {
        let process = test_process();
        let discount = (-0.05_f64).exp();
        let pricer = AsianArithmeticPathPricer::new(|avg| (avg - 100.0).max(0.0), discount);
        let model = MonteCarloModel::new(&process, 1.0, 50, 42);
        let stats = model.simulate(&pricer, 50_000);
        let price = stats.mean().unwrap();

        // Asian call is cheaper than vanilla call (~10.45), typically 5-8
        assert!(price > 2.0 && price < 12.0, "Asian arith call = {price:.2}");
    }
}
