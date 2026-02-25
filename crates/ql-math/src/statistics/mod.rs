//! Statistics accumulators and utilities (translates
//! `ql/math/statistics/generalstatistics.hpp` and
//! `ql/math/statistics/incrementalstatistics.hpp`).

use ql_core::Real;

/// Incremental statistics accumulator.
///
/// Accumulates weighted samples and computes mean, variance, standard
/// deviation, min, max, and count.
#[derive(Debug, Clone, Default)]
pub struct Statistics {
    count: usize,
    sum_w: Real,
    sum_wx: Real,
    sum_wx2: Real,
    min: Real,
    max: Real,
}

impl Statistics {
    /// Create a new empty accumulator.
    pub fn new() -> Self {
        Self {
            count: 0,
            sum_w: 0.0,
            sum_wx: 0.0,
            sum_wx2: 0.0,
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
        }
    }

    /// Add a single sample with weight 1.
    pub fn add(&mut self, x: Real) {
        self.add_weighted(x, 1.0);
    }

    /// Add a weighted sample.
    pub fn add_weighted(&mut self, x: Real, weight: Real) {
        self.count += 1;
        self.sum_w += weight;
        self.sum_wx += weight * x;
        self.sum_wx2 += weight * x * x;
        if x < self.min {
            self.min = x;
        }
        if x > self.max {
            self.max = x;
        }
    }

    /// Number of samples.
    pub fn samples(&self) -> usize {
        self.count
    }

    /// Sum of weights.
    pub fn sum_weights(&self) -> Real {
        self.sum_w
    }

    /// Weighted mean.  Returns `None` if no samples have been added.
    pub fn mean(&self) -> Option<Real> {
        if self.sum_w == 0.0 {
            None
        } else {
            Some(self.sum_wx / self.sum_w)
        }
    }

    /// Weighted variance (unbiased, Bessel-corrected).  Returns `None` for
    /// fewer than 2 samples.
    pub fn variance(&self) -> Option<Real> {
        if self.sum_w == 0.0 || self.count < 2 {
            return None;
        }
        let m = self.sum_wx / self.sum_w;
        let s2 = self.sum_wx2 / self.sum_w - m * m;
        // Bessel correction: n / (n - 1)
        Some(s2 * self.count as Real / (self.count as Real - 1.0))
    }

    /// Standard deviation.  Returns `None` for fewer than 2 samples.
    pub fn std_dev(&self) -> Option<Real> {
        self.variance().map(|v| v.sqrt())
    }

    /// Minimum sample value.  Returns `None` if no samples have been added.
    pub fn minimum(&self) -> Option<Real> {
        if self.count == 0 {
            None
        } else {
            Some(self.min)
        }
    }

    /// Maximum sample value.  Returns `None` if no samples have been added.
    pub fn maximum(&self) -> Option<Real> {
        if self.count == 0 {
            None
        } else {
            Some(self.max)
        }
    }

    /// Reset the accumulator to its initial state.
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

// ── GeneralStatistics ─────────────────────────────────────────────────────────

/// Full statistics accumulator that stores all samples.
///
/// Supports mean, variance, std_dev, skewness, kurtosis, percentiles,
/// min, max.
///
/// Corresponds to `QuantLib::GeneralStatistics`.
#[derive(Debug, Clone, Default)]
pub struct GeneralStatistics {
    data: Vec<(Real, Real)>, // (value, weight)
    sorted: bool,
}

impl GeneralStatistics {
    /// Create a new, empty statistics collector.
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            sorted: false,
        }
    }

    /// Add a sample with weight 1.
    pub fn add(&mut self, x: Real) {
        self.add_weighted(x, 1.0);
    }

    /// Add a weighted sample.
    pub fn add_weighted(&mut self, x: Real, weight: Real) {
        self.data.push((x, weight));
        self.sorted = false;
    }

    /// Number of samples.
    pub fn samples(&self) -> usize {
        self.data.len()
    }

    /// Sum of weights.
    pub fn sum_weights(&self) -> Real {
        self.data.iter().map(|(_, w)| w).sum()
    }

    /// Weighted mean.
    pub fn mean(&self) -> Option<Real> {
        let sw = self.sum_weights();
        if sw == 0.0 {
            return None;
        }
        Some(self.data.iter().map(|(x, w)| w * x).sum::<Real>() / sw)
    }

    /// Weighted variance (Bessel-corrected).
    pub fn variance(&self) -> Option<Real> {
        if self.data.len() < 2 {
            return None;
        }
        let mean = self.mean()?;
        let sw = self.sum_weights();
        let n = self.data.len() as Real;
        let s2 = self
            .data
            .iter()
            .map(|(x, w)| w * (x - mean).powi(2))
            .sum::<Real>()
            / sw;
        Some(s2 * n / (n - 1.0))
    }

    /// Standard deviation.
    pub fn std_dev(&self) -> Option<Real> {
        self.variance().map(|v| v.sqrt())
    }

    /// Skewness (third standardized moment).
    pub fn skewness(&self) -> Option<Real> {
        if self.data.len() < 3 {
            return None;
        }
        let mean = self.mean()?;
        let sigma = self.std_dev()?;
        if sigma == 0.0 {
            return None;
        }
        let sw = self.sum_weights();
        let n = self.data.len() as Real;
        let m3 = self
            .data
            .iter()
            .map(|(x, w)| w * ((x - mean) / sigma).powi(3))
            .sum::<Real>()
            / sw;
        // Adjust for sample skewness
        Some(m3 * n * n / ((n - 1.0) * (n - 2.0)))
    }

    /// Excess kurtosis (fourth standardized moment minus 3).
    pub fn kurtosis(&self) -> Option<Real> {
        if self.data.len() < 4 {
            return None;
        }
        let mean = self.mean()?;
        let sigma = self.std_dev()?;
        if sigma == 0.0 {
            return None;
        }
        let sw = self.sum_weights();
        let n = self.data.len() as Real;
        let m4 = self
            .data
            .iter()
            .map(|(x, w)| w * ((x - mean) / sigma).powi(4))
            .sum::<Real>()
            / sw;
        // Excess kurtosis with sample correction
        let k = (n - 1.0) / ((n - 2.0) * (n - 3.0)) * ((n + 1.0) * m4 - 3.0 * (n - 1.0));
        Some(k)
    }

    /// Minimum sample value.
    pub fn minimum(&self) -> Option<Real> {
        self.data.iter().map(|(x, _)| *x).reduce(f64::min)
    }

    /// Maximum sample value.
    pub fn maximum(&self) -> Option<Real> {
        self.data.iter().map(|(x, _)| *x).reduce(f64::max)
    }

    /// Percentile (0..=100). Uses linear interpolation between sorted samples.
    pub fn percentile(&mut self, p: Real) -> Option<Real> {
        if self.data.is_empty() {
            return None;
        }
        assert!((0.0..=100.0).contains(&p), "percentile must be in [0, 100]");

        if !self.sorted {
            self.data.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            self.sorted = true;
        }

        let n = self.data.len();
        if n == 1 {
            return Some(self.data[0].0);
        }

        let rank = p / 100.0 * (n - 1) as Real;
        let lo = rank.floor() as usize;
        let hi = rank.ceil() as usize;
        let frac = rank - lo as Real;

        if lo == hi {
            Some(self.data[lo].0)
        } else {
            Some(self.data[lo].0 * (1.0 - frac) + self.data[hi].0 * frac)
        }
    }

    /// Median (50th percentile).
    pub fn median(&mut self) -> Option<Real> {
        self.percentile(50.0)
    }

    /// Reset the accumulator.
    pub fn reset(&mut self) {
        self.data.clear();
        self.sorted = false;
    }
}

// ── IncrementalStatistics ─────────────────────────────────────────────────────

/// Online (incremental) statistics accumulator using Welford's algorithm.
///
/// Computes running mean, variance, skewness, and kurtosis without storing all
/// samples.
///
/// Corresponds to `QuantLib::IncrementalStatistics`.
#[derive(Debug, Clone, Default)]
pub struct IncrementalStatistics {
    n: usize,
    sum_w: Real,
    m1: Real, // mean
    m2: Real, // sum of (x - mean)^2 * w (for variance)
    m3: Real, // for skewness
    m4: Real, // for kurtosis
    min: Real,
    max: Real,
}

impl IncrementalStatistics {
    /// Create a new empty accumulator.
    pub fn new() -> Self {
        Self {
            n: 0,
            sum_w: 0.0,
            m1: 0.0,
            m2: 0.0,
            m3: 0.0,
            m4: 0.0,
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
        }
    }

    /// Add a sample with weight 1.
    pub fn add(&mut self, x: Real) {
        self.add_weighted(x, 1.0);
    }

    /// Add a weighted sample, updating running moments.
    pub fn add_weighted(&mut self, x: Real, w: Real) {
        let n1 = self.sum_w;
        self.sum_w += w;
        self.n += 1;

        let delta = x - self.m1;
        let delta_n = delta * w / self.sum_w;
        let delta_n2 = delta_n * delta_n;
        let term1 = delta * delta_n * n1;

        self.m1 += delta_n;
        self.m4 +=
            term1 * delta_n2 * (self.sum_w * self.sum_w - 3.0 * w * self.sum_w + 3.0 * w * w)
                + 6.0 * delta_n2 * self.m2
                - 4.0 * delta_n * self.m3;
        self.m3 += term1 * delta_n * (self.sum_w - 2.0 * w) - 3.0 * delta_n * self.m2;
        self.m2 += term1;

        if x < self.min {
            self.min = x;
        }
        if x > self.max {
            self.max = x;
        }
    }

    /// Number of samples.
    pub fn samples(&self) -> usize {
        self.n
    }

    /// Mean.
    pub fn mean(&self) -> Option<Real> {
        if self.n == 0 {
            None
        } else {
            Some(self.m1)
        }
    }

    /// Variance (Bessel-corrected).
    pub fn variance(&self) -> Option<Real> {
        if self.n < 2 {
            return None;
        }
        Some(self.m2 / (self.sum_w - 1.0))
    }

    /// Standard deviation.
    pub fn std_dev(&self) -> Option<Real> {
        self.variance().map(|v| v.sqrt())
    }

    /// Error estimate (standard error of the mean): σ / √n.
    pub fn error_estimate(&self) -> Option<Real> {
        if self.n < 2 {
            return None;
        }
        self.std_dev().map(|sd| sd / (self.n as Real).sqrt())
    }

    /// Skewness.
    pub fn skewness(&self) -> Option<Real> {
        if self.n < 3 || self.m2 == 0.0 {
            return None;
        }
        Some(self.sum_w.sqrt() * self.m3 / self.m2.powf(1.5))
    }

    /// Excess kurtosis.
    pub fn kurtosis(&self) -> Option<Real> {
        if self.n < 4 || self.m2 == 0.0 {
            return None;
        }
        Some(self.sum_w * self.m4 / (self.m2 * self.m2) - 3.0)
    }

    /// Minimum.
    pub fn minimum(&self) -> Option<Real> {
        if self.n == 0 {
            None
        } else {
            Some(self.min)
        }
    }

    /// Maximum.
    pub fn maximum(&self) -> Option<Real> {
        if self.n == 0 {
            None
        } else {
            Some(self.max)
        }
    }

    /// Reset.
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

// ── ConvergenceStatistics ─────────────────────────────────────────────────────

/// A statistics accumulator that records convergence snapshots at power-of-2
/// sample counts.
///
/// Wraps an `IncrementalStatistics` and captures the running mean (and error
/// estimate) at 1, 2, 4, 8, 16, … samples. This is used in Monte Carlo
/// pricing to assess convergence.
///
/// Corresponds to `QuantLib::ConvergenceStatistics`.
#[derive(Debug, Clone)]
pub struct ConvergenceStatistics {
    inner: IncrementalStatistics,
    /// (samples, mean, error_estimate) snapshots
    snapshots: Vec<(usize, Real, Real)>,
    next_trigger: usize,
}

impl ConvergenceStatistics {
    /// Create a new convergence statistics accumulator.
    pub fn new() -> Self {
        Self {
            inner: IncrementalStatistics::new(),
            snapshots: Vec::new(),
            next_trigger: 1,
        }
    }

    /// Add a value.
    pub fn add(&mut self, x: Real) {
        self.inner.add(x);
        let n = self.inner.samples();
        if n == self.next_trigger {
            let mean = self.inner.mean().unwrap_or(0.0);
            let err = self.inner.error_estimate().unwrap_or(0.0);
            self.snapshots.push((n, mean, err));
            self.next_trigger *= 2;
        }
    }

    /// Add a weighted value.
    pub fn add_weighted(&mut self, x: Real, w: Real) {
        self.inner.add_weighted(x, w);
        let n = self.inner.samples();
        if n == self.next_trigger {
            let mean = self.inner.mean().unwrap_or(0.0);
            let err = self.inner.error_estimate().unwrap_or(0.0);
            self.snapshots.push((n, mean, err));
            self.next_trigger *= 2;
        }
    }

    /// Return convergence snapshots: `(samples, mean, error_estimate)`.
    pub fn convergence_table(&self) -> &[(usize, Real, Real)] {
        &self.snapshots
    }

    /// Access the underlying statistics accumulator.
    pub fn statistics(&self) -> &IncrementalStatistics {
        &self.inner
    }

    /// Reset.
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for ConvergenceStatistics {
    fn default() -> Self {
        Self::new()
    }
}

// ── SequenceStatistics ────────────────────────────────────────────────────────

/// Multi-dimensional statistics accumulator.
///
/// Keeps an independent `IncrementalStatistics` for each dimension, plus
/// accumulates the covariance matrix.
///
/// Corresponds to `QuantLib::SequenceStatistics`.
#[derive(Debug, Clone)]
pub struct SequenceStatistics {
    dim: usize,
    stats: Vec<IncrementalStatistics>,
    n: usize,
    /// Running sum of outer products (for covariance)
    sum_xy: Vec<Vec<Real>>,
    /// Running sum of means for covariance: sum_x[i]
    sum_x: Vec<Real>,
}

impl SequenceStatistics {
    /// Create for `dimension` variates.
    pub fn new(dimension: usize) -> Self {
        Self {
            dim: dimension,
            stats: (0..dimension)
                .map(|_| IncrementalStatistics::new())
                .collect(),
            n: 0,
            sum_xy: vec![vec![0.0; dimension]; dimension],
            sum_x: vec![0.0; dimension],
        }
    }

    /// Dimension.
    pub fn dimension(&self) -> usize {
        self.dim
    }

    /// Number of samples added.
    pub fn samples(&self) -> usize {
        self.n
    }

    /// Add a multi-dimensional sample.
    pub fn add(&mut self, sample: &[Real]) {
        assert_eq!(sample.len(), self.dim, "sample dimension mismatch");
        self.n += 1;
        for i in 0..self.dim {
            self.stats[i].add(sample[i]);
            self.sum_x[i] += sample[i];
            for j in 0..self.dim {
                self.sum_xy[i][j] += sample[i] * sample[j];
            }
        }
    }

    /// Mean vector.
    pub fn mean(&self) -> Vec<Real> {
        self.stats.iter().map(|s| s.mean().unwrap_or(0.0)).collect()
    }

    /// Variance vector.
    pub fn variance(&self) -> Vec<Real> {
        self.stats
            .iter()
            .map(|s| s.variance().unwrap_or(0.0))
            .collect()
    }

    /// Standard deviation vector.
    pub fn std_dev(&self) -> Vec<Real> {
        self.stats
            .iter()
            .map(|s| s.std_dev().unwrap_or(0.0))
            .collect()
    }

    /// Covariance matrix (sample covariance).
    pub fn covariance(&self) -> Vec<Vec<Real>> {
        if self.n < 2 {
            return vec![vec![0.0; self.dim]; self.dim];
        }
        let n = self.n as Real;
        let mut cov = vec![vec![0.0; self.dim]; self.dim];
        for i in 0..self.dim {
            for j in 0..self.dim {
                cov[i][j] = (self.sum_xy[i][j] - self.sum_x[i] * self.sum_x[j] / n) / (n - 1.0);
            }
        }
        cov
    }

    /// Correlation matrix.
    pub fn correlation(&self) -> Vec<Vec<Real>> {
        let cov = self.covariance();
        let mut corr = vec![vec![0.0; self.dim]; self.dim];
        for i in 0..self.dim {
            for j in 0..self.dim {
                let denom = (cov[i][i] * cov[j][j]).sqrt();
                corr[i][j] = if denom > 1e-30 {
                    cov[i][j] / denom
                } else {
                    0.0
                };
            }
        }
        corr
    }

    /// Access the statistics for dimension `i`.
    pub fn stat(&self, i: usize) -> &IncrementalStatistics {
        &self.stats[i]
    }

    /// Reset all accumulators.
    pub fn reset(&mut self) {
        *self = Self::new(self.dim);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_statistics() {
        let mut s = Statistics::new();
        for x in [1.0, 2.0, 3.0, 4.0, 5.0] {
            s.add(x);
        }
        assert_eq!(s.samples(), 5);
        assert!((s.mean().unwrap() - 3.0).abs() < 1e-12);
        assert!((s.variance().unwrap() - 2.5).abs() < 1e-12);
        assert!((s.std_dev().unwrap() - 2.5_f64.sqrt()).abs() < 1e-12);
        assert_eq!(s.minimum().unwrap(), 1.0);
        assert_eq!(s.maximum().unwrap(), 5.0);
    }

    #[test]
    fn empty_statistics() {
        let s = Statistics::new();
        assert!(s.mean().is_none());
        assert!(s.variance().is_none());
    }

    #[test]
    fn general_statistics_skewness_kurtosis() {
        let mut gs = GeneralStatistics::new();
        // Symmetric data → skewness ≈ 0
        for &x in &[1.0, 2.0, 3.0, 4.0, 5.0] {
            gs.add(x);
        }
        assert!(gs.mean().unwrap() - 3.0 < 1e-12);
        // Skewness of symmetric uniform data = 0
        assert!(
            gs.skewness().unwrap().abs() < 1e-10,
            "skewness = {}",
            gs.skewness().unwrap()
        );
    }

    #[test]
    fn general_statistics_percentile() {
        let mut gs = GeneralStatistics::new();
        for i in 1..=100 {
            gs.add(i as Real);
        }
        let median = gs.median().unwrap();
        assert!((median - 50.5).abs() < 1e-10, "median = {median}");
        let p25 = gs.percentile(25.0).unwrap();
        assert!((p25 - 25.75).abs() < 1e-10, "p25 = {p25}");
    }

    #[test]
    fn incremental_statistics_mean_variance() {
        let mut is = IncrementalStatistics::new();
        for &x in &[1.0, 2.0, 3.0, 4.0, 5.0] {
            is.add(x);
        }
        assert!((is.mean().unwrap() - 3.0).abs() < 1e-12);
        assert!(
            (is.variance().unwrap() - 2.5).abs() < 1e-10,
            "variance = {}",
            is.variance().unwrap()
        );
        assert_eq!(is.minimum().unwrap(), 1.0);
        assert_eq!(is.maximum().unwrap(), 5.0);
    }

    #[test]
    fn incremental_statistics_symmetric_skewness() {
        let mut is = IncrementalStatistics::new();
        // Symmetric data → skewness ≈ 0
        for &x in &[-2.0, -1.0, 0.0, 1.0, 2.0] {
            is.add(x);
        }
        assert!(
            is.skewness().unwrap().abs() < 1e-10,
            "skewness = {}",
            is.skewness().unwrap()
        );
    }

    #[test]
    fn convergence_statistics_snapshots() {
        let mut cs = ConvergenceStatistics::new();
        for i in 1..=128 {
            cs.add(i as Real);
        }
        let table = cs.convergence_table();
        // Should have snapshots at 1, 2, 4, 8, 16, 32, 64, 128
        assert_eq!(table.len(), 8);
        assert_eq!(table[0].0, 1); // 1 sample
        assert_eq!(table[7].0, 128); // 128 samples
                                     // Mean at 128 samples should be 64.5
        assert!((table[7].1 - 64.5).abs() < 1e-10);
    }

    #[test]
    fn sequence_statistics_2d() {
        let mut ss = SequenceStatistics::new(2);
        ss.add(&[1.0, 2.0]);
        ss.add(&[3.0, 4.0]);
        ss.add(&[5.0, 6.0]);
        let m = ss.mean();
        assert!((m[0] - 3.0).abs() < 1e-12);
        assert!((m[1] - 4.0).abs() < 1e-12);
        // Perfect correlation between the two series
        let corr = ss.correlation();
        assert!((corr[0][1] - 1.0).abs() < 1e-10, "corr = {}", corr[0][1]);
    }
}
