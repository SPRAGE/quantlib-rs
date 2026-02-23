//! Random number generators (translates `ql/math/randomnumbers/`).
//!
//! Provides wrappers around the `rand` and `rand_mt` crates that match the
//! QuantLib RNG interface.

use ql_core::Real;
use rand_mt::Mt19937GenRand64;

/// A uniform pseudo-random number generator based on the Mersenne Twister
/// MT19937-64 algorithm.
///
/// Corresponds to `QuantLib::MersenneTwisterUniformRng`.
pub struct MersenneTwisterUniformRng {
    rng: Mt19937GenRand64,
}

impl MersenneTwisterUniformRng {
    /// Create a new generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: Mt19937GenRand64::new(seed),
        }
    }

    /// Generate the next uniform deviate in `[0, 1)`.
    pub fn next_real(&mut self) -> Real {
        // Map u64 to [0.0, 1.0)
        let u: u64 = self.rng.next_u64();
        u as f64 / (u64::MAX as f64 + 1.0)
    }

    /// Generate the next integer deviate.
    pub fn next_int32(&mut self) -> u32 {
        self.rng.next_u32()
    }
}

/// An inverse-cumulative normal random number generator.
///
/// Wraps a uniform RNG and transforms its output through the inverse CDF of
/// the standard normal distribution.
///
/// Corresponds to `QuantLib::InverseCumulativeNormal` RNG.
pub struct InverseCumulativeNormalRng {
    inner: MersenneTwisterUniformRng,
}

impl InverseCumulativeNormalRng {
    /// Create a new generator backed by a Mersenne Twister with the given
    /// seed.
    pub fn new(seed: u64) -> Self {
        Self {
            inner: MersenneTwisterUniformRng::new(seed),
        }
    }

    /// Generate the next standard-normal deviate.
    pub fn next_real(&mut self) -> Real {
        // Avoid exact 0 or 1 which would produce ±∞
        let u = loop {
            let u = self.inner.next_real();
            if u > 0.0 && u < 1.0 {
                break u;
            }
        };
        crate::distributions::normal_cdf_inverse(u)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mt_range() {
        let mut rng = MersenneTwisterUniformRng::new(42);
        for _ in 0..1_000 {
            let x = rng.next_real();
            assert!(x >= 0.0 && x < 1.0);
        }
    }

    #[test]
    fn icn_rng_reasonable_range() {
        let mut rng = InverseCumulativeNormalRng::new(42);
        let samples: Vec<Real> = (0..1_000).map(|_| rng.next_real()).collect();
        let mean = samples.iter().sum::<Real>() / 1_000.0;
        // With 1000 samples, mean should be within a few std-devs of 0
        assert!(mean.abs() < 0.1, "mean {mean} out of expected range");
    }
}
