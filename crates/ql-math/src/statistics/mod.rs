//! Basic statistics accumulator (translates
//! `ql/math/statistics/generalstatistics.hpp`).

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
}
