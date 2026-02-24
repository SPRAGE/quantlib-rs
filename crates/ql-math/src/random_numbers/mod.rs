//! Random number generators (translates `ql/math/randomnumbers/`).
//!
//! Provides wrappers around the `rand` and `rand_mt` crates that match the
//! QuantLib RNG interface, plus quasi-random sequences (Halton, Sobol).

pub mod sobol;

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

// ── Random Sequence Generator ─────────────────────────────────────────────────

/// Generates sequences of pseudo-random numbers as `Vec<Real>`.
///
/// Corresponds to `QuantLib::RandomSequenceGenerator<MersenneTwisterUniformRng>`.
pub struct RandomSequenceGenerator {
    rng: MersenneTwisterUniformRng,
    dimension: usize,
}

impl RandomSequenceGenerator {
    /// Create a generator that produces `dimension`-dimensional uniform random
    /// vectors.
    pub fn new(dimension: usize, seed: u64) -> Self {
        Self {
            rng: MersenneTwisterUniformRng::new(seed),
            dimension,
        }
    }

    /// Dimension of the generated sequences.
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Generate the next random vector in `[0, 1)^dimension`.
    pub fn next_sequence(&mut self) -> Vec<Real> {
        (0..self.dimension).map(|_| self.rng.next_real()).collect()
    }
}

// ── Halton Low-Discrepancy Sequence ───────────────────────────────────────────

/// Halton quasi-random sequence generator.
///
/// Generates low-discrepancy sequences in `[0, 1)^d` using the Halton
/// construction with the first `d` prime bases.
///
/// Corresponds to `QuantLib::HaltonRsg`.
pub struct HaltonRsg {
    dimension: usize,
    bases: Vec<u64>,
    index: u64,
}

impl HaltonRsg {
    /// Create a new Halton sequence generator of the given dimension.
    ///
    /// Optionally skip the first `skip` elements.
    pub fn new(dimension: usize, skip: u64) -> Self {
        let bases = first_primes(dimension);
        Self {
            dimension,
            bases,
            index: skip,
        }
    }

    /// Dimension of the generated sequences.
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// The current sequence index (number of vectors generated so far, plus
    /// any initial skip).
    pub fn sequence_index(&self) -> u64 {
        self.index
    }

    /// Generate the next quasi-random vector.
    pub fn next_sequence(&mut self) -> Vec<Real> {
        self.index += 1;
        self.bases
            .iter()
            .map(|&base| van_der_corput(self.index, base))
            .collect()
    }
}

/// Van der Corput sequence element for the given integer `n` in the given
/// `base`.
fn van_der_corput(mut n: u64, base: u64) -> Real {
    let mut result = 0.0;
    let mut denom = 1.0;
    while n > 0 {
        denom *= base as Real;
        result += (n % base) as Real / denom;
        n /= base;
    }
    result
}

/// Return the first `count` prime numbers.
fn first_primes(count: usize) -> Vec<u64> {
    if count == 0 {
        return Vec::new();
    }
    let mut primes = Vec::with_capacity(count);
    let mut candidate = 2u64;
    while primes.len() < count {
        if is_prime(candidate) {
            primes.push(candidate);
        }
        candidate += 1;
    }
    primes
}

fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n < 4 {
        return true;
    }
    if n % 2 == 0 || n % 3 == 0 {
        return false;
    }
    let mut i = 5;
    while i * i <= n {
        if n % i == 0 || n % (i + 2) == 0 {
            return false;
        }
        i += 6;
    }
    true
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

    #[test]
    fn random_sequence_generator() {
        let mut rsg = RandomSequenceGenerator::new(5, 42);
        assert_eq!(rsg.dimension(), 5);
        let seq = rsg.next_sequence();
        assert_eq!(seq.len(), 5);
        for &v in &seq {
            assert!(v >= 0.0 && v < 1.0, "value {v} out of range");
        }
    }

    #[test]
    fn halton_low_discrepancy() {
        let mut halton = HaltonRsg::new(3, 0);
        assert_eq!(halton.dimension(), 3);
        // The first Halton point (index 1) in bases (2, 3, 5):
        // base 2: 1/2 = 0.5
        // base 3: 1/3 ≈ 0.333
        // base 5: 1/5 = 0.2
        let pt = halton.next_sequence();
        assert_eq!(pt.len(), 3);
        assert!((pt[0] - 0.5).abs() < 1e-12);
        assert!((pt[1] - 1.0 / 3.0).abs() < 1e-12);
        assert!((pt[2] - 0.2).abs() < 1e-12);
    }

    #[test]
    fn halton_fills_unit_cube() {
        let mut halton = HaltonRsg::new(2, 0);
        for _ in 0..100 {
            let pt = halton.next_sequence();
            for &v in &pt {
                assert!(v >= 0.0 && v < 1.0, "value {v} out of [0, 1)");
            }
        }
    }
}
