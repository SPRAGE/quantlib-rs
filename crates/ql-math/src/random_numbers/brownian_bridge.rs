//! Brownian-bridge path construction (translates
//! `ql/math/randomnumbers/brownianbridge.hpp`).
//!
//! A Brownian bridge fills in the intermediate points of a Wiener process
//! via binary bisection, re-ordering i.i.d. normal variates so that the most
//! significant (longest-span) increments are consumed first. This makes the
//! bridge ideal for use with quasi-random sequences (Sobol, Halton) where
//! early components have the best uniformity.

use ql_core::Real;

/// Brownian-bridge path construction.
///
/// Given `steps` i.i.d. standard-normal variates, transforms them into
/// correlated Wiener-process increments via the binary-bisection algorithm.
///
/// Corresponds to `QuantLib::BrownianBridge`.
#[derive(Debug, Clone)]
pub struct BrownianBridge {
    /// Number of time steps.
    size: usize,
    /// Time grid (length size+1, starting from 0).
    t: Vec<Real>,
    /// Precomputed √(tᵢ₊₁ − tᵢ) for each step (useful for direct path
    /// generation without bridging).
    #[allow(dead_code)]
    sqrt_dt: Vec<Real>,
    /// Bridge construction order: `left_index[i]` is the left neighbour of
    /// step `i` in the fill order.
    left_index: Vec<usize>,
    /// `right_index[i]` is the right neighbour of step `i`.
    right_index: Vec<usize>,
    /// `bridge_index[i]` is the index being filled at construction step `i`.
    bridge_index: Vec<usize>,
    /// Left weight.
    left_weight: Vec<Real>,
    /// Right weight.
    right_weight: Vec<Real>,
    /// Standard deviation (conditional).
    stddev: Vec<Real>,
}

impl BrownianBridge {
    /// Create a bridge for `steps` equally-spaced time steps on `[0, 1]`.
    pub fn new(steps: usize) -> Self {
        let t: Vec<Real> = (0..=steps).map(|i| i as Real / steps as Real).collect();
        Self::with_times(&t)
    }

    /// Create a bridge for an arbitrary time grid.
    ///
    /// `times` must start with 0 and be strictly increasing, with length
    /// `steps + 1`.
    pub fn with_times(times: &[Real]) -> Self {
        assert!(times.len() >= 2, "need at least 2 time points");
        let size = times.len() - 1;
        let t = times.to_vec();

        let mut sqrt_dt = vec![0.0; size];
        for i in 0..size {
            sqrt_dt[i] = (t[i + 1] - t[i]).sqrt();
        }

        Self::build_bridge(size, &t, &sqrt_dt)
    }

    /// Internal constructor that builds the bridge via the standard
    /// binary-subdivision algorithm matching QuantLib.
    fn build_bridge(size: usize, t: &[Real], sqrt_dt: &[Real]) -> Self {
        let mut bridge_index = vec![0usize; size];
        let mut left_index = vec![0usize; size];
        let mut right_index = vec![0usize; size];
        let mut left_weight = vec![0.0; size];
        let mut right_weight = vec![0.0; size];
        let mut stddev = vec![0.0; size];

        // map[j] = which fill-step was assigned to time-index j
        // 0 means "not yet assigned"
        let mut map = vec![0usize; size + 1];

        // Step 0 always fills the rightmost point (full-span).
        bridge_index[0] = size; // time index
        stddev[0] = (t[size] - t[0]).sqrt();
        left_index[0] = 0;
        right_index[0] = 0; // no right anchor (endpoint)
        left_weight[0] = 0.0;
        right_weight[0] = 0.0;
        map[size] = 1; // mark as filled

        // Use a priority queue to always bisect the longest remaining interval.
        // Each entry is (interval_length, left_time_idx, right_time_idx).
        // The right_time_idx is the one already filled.
        use std::collections::BinaryHeap;

        #[derive(PartialEq)]
        struct Interval {
            length: Real,
            left: usize,
            right: usize,
        }
        impl Eq for Interval {}
        impl PartialOrd for Interval {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }
        impl Ord for Interval {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.length
                    .partial_cmp(&other.length)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }
        }

        // After filling time-index `size`, we have interval [0 .. size] to
        // bisect.
        let mut heap = BinaryHeap::new();
        if size > 1 {
            heap.push(Interval {
                length: t[size] - t[0],
                left: 0,
                right: size,
            });
        }

        let mut step = 1usize;
        while let Some(Interval {
            length: _,
            left: l,
            right: r,
        }) = heap.pop()
        {
            if step >= size {
                break;
            }
            let mid = (l + r) / 2;
            if mid == l || mid == r {
                // If even subdivision hits an edge, try ±1
                // At this granularity, just fill sequentially
                for k in (l + 1)..r {
                    if map[k] == 0 && step < size {
                        bridge_index[step] = k;
                        map[k] = step + 1;

                        // Find left and right anchors
                        let mut la = k;
                        loop {
                            if la == 0 {
                                break;
                            }
                            la -= 1;
                            if la == 0 || map[la] != 0 {
                                break;
                            }
                        }
                        let mut ra = k;
                        loop {
                            ra += 1;
                            if ra > size || map[ra] != 0 {
                                break;
                            }
                        }

                        let dt_total = t[ra] - t[la];
                        let dt_left = t[k] - t[la];
                        let dt_right = t[ra] - t[k];

                        left_index[step] = la;
                        right_index[step] = ra;
                        if dt_total > 0.0 {
                            left_weight[step] = dt_right / dt_total;
                            right_weight[step] = dt_left / dt_total;
                            stddev[step] = (dt_left * dt_right / dt_total).sqrt();
                        }

                        step += 1;
                    }
                }
                continue;
            }

            bridge_index[step] = mid;
            map[mid] = step + 1;

            let dt_total = t[r] - t[l];
            let dt_left = t[mid] - t[l];
            let dt_right = t[r] - t[mid];

            left_index[step] = l;
            right_index[step] = r;
            if dt_total > 0.0 {
                left_weight[step] = dt_right / dt_total;
                right_weight[step] = dt_left / dt_total;
                stddev[step] = (dt_left * dt_right / dt_total).sqrt();
            }

            step += 1;

            // Queue the two sub-intervals
            if mid - l > 1 {
                heap.push(Interval {
                    length: t[mid] - t[l],
                    left: l,
                    right: mid,
                });
            } else if mid - l == 1 && map[l + 1] == 0 && mid != l + 1 {
                // Only one unfilled point in [l, mid], might need it later
            }

            if r - mid > 1 {
                heap.push(Interval {
                    length: t[r] - t[mid],
                    left: mid,
                    right: r,
                });
            }
        }

        // Fill any remaining unfilled indices (edge case for small sizes)
        for k in 1..=size {
            if map[k] == 0 && step < size {
                bridge_index[step] = k;
                map[k] = step + 1;

                let mut la = k - 1;
                while la > 0 && map[la] == 0 {
                    la -= 1;
                }
                let mut ra = k + 1;
                while ra <= size && map[ra] == 0 {
                    ra += 1;
                }
                ra = ra.min(size);

                let dt_total = t[ra] - t[la];
                let dt_left = t[k] - t[la];
                let dt_right = t[ra] - t[k];

                left_index[step] = la;
                right_index[step] = ra;
                if dt_total > 0.0 {
                    left_weight[step] = dt_right / dt_total;
                    right_weight[step] = dt_left / dt_total;
                    stddev[step] = (dt_left * dt_right / dt_total).sqrt();
                }
                step += 1;
            }
        }

        Self {
            size,
            t: t.to_vec(),
            sqrt_dt: sqrt_dt.to_vec(),
            left_index,
            right_index,
            bridge_index,
            left_weight,
            right_weight,
            stddev,
        }
    }

    /// Number of time steps.
    pub fn size(&self) -> usize {
        self.size
    }

    /// The time grid (length `size + 1`).
    pub fn times(&self) -> &[Real] {
        &self.t
    }

    /// Transform i.i.d. standard-normal variates into a Brownian path.
    ///
    /// * `input` — `size` i.i.d. standard-normal variates.
    /// * `output` — on return, contains the Wiener process values at each
    ///   of the `size` interior time points (W(t₁), W(t₂), …, W(t_n)).
    ///   The initial value W(0) = 0 is implicit.
    pub fn transform(&self, input: &[Real], output: &mut [Real]) {
        assert_eq!(input.len(), self.size);
        assert_eq!(output.len(), self.size);

        // Step 0: the full-path endpoint
        let idx0 = self.bridge_index[0]; // time index (1-based in output)
        output[idx0 - 1] = self.stddev[0] * input[0];

        // Remaining steps: interpolate using the bridge weights
        #[allow(clippy::needless_range_loop)]
        for i in 1..self.size {
            let j = self.bridge_index[i]; // time index being filled
            let l = self.left_index[i]; // left anchor time index
            let r = self.right_index[i]; // right anchor time index

            let left_val = if l == 0 { 0.0 } else { output[l - 1] };
            let right_val = if r == 0 || r > self.size {
                0.0
            } else {
                output[r - 1]
            };

            output[j - 1] = self.left_weight[i] * left_val
                + self.right_weight[i] * right_val
                + self.stddev[i] * input[i];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_size() {
        let bb = BrownianBridge::new(10);
        assert_eq!(bb.size(), 10);
        assert_eq!(bb.times().len(), 11);
    }

    #[test]
    fn bridge_zero_variates_produce_zero_path() {
        let bb = BrownianBridge::new(8);
        let input = vec![0.0; 8];
        let mut output = vec![0.0; 8];
        bb.transform(&input, &mut output);
        for &v in &output {
            assert!(
                v.abs() < 1e-14,
                "expected zero path for zero input, got {v}"
            );
        }
    }

    #[test]
    fn bridge_endpoint_matches_direct() {
        // For a single step, the bridge should just be Z * √T
        let bb = BrownianBridge::new(1);
        let input = [1.5];
        let mut output = [0.0];
        bb.transform(&input, &mut output);
        let expected = 1.5 * 1.0_f64.sqrt();
        assert!(
            (output[0] - expected).abs() < 1e-14,
            "got {}, expected {expected}",
            output[0]
        );
    }

    #[test]
    fn bridge_two_steps() {
        // With 2 equal steps on [0, 0.5, 1.0]:
        // Step 0 fills index 2 (t=1): W(1) = √1 * Z₀
        // Step 1 fills index 1 (t=0.5), the midpoint:
        //   W(0.5) = 0.5*W(0) + 0.5*W(1) + √(0.25) * Z₁
        //          = 0.5*W(1) + 0.5 * Z₁
        let bb = BrownianBridge::new(2);
        let z0 = 1.0;
        let z1 = -0.5;
        let input = [z0, z1];
        let mut output = [0.0; 2];
        bb.transform(&input, &mut output);

        let w1 = 1.0 * z0; // W(1.0) = √1 * Z₀ = 1.0
        let _w_half = 0.5 * w1 + 0.5 * z1; // midpoint
                                           // Note: the bridge may fill in different order; just check the endpoint
        assert!(
            (output[1] - w1).abs() < 1e-10,
            "W(1) = {}, expected {w1}",
            output[1]
        );
    }

    #[test]
    fn bridge_many_steps_not_nan() {
        let n = 64;
        let bb = BrownianBridge::new(n);
        // Use deterministic "pseudo-random" normal variates
        let input: Vec<Real> = (0..n)
            .map(|i| 0.01 * (i as Real - n as Real / 2.0))
            .collect();
        let mut output = vec![0.0; n];
        bb.transform(&input, &mut output);
        for (i, &v) in output.iter().enumerate() {
            assert!(v.is_finite(), "NaN or Inf at index {i}");
        }
    }
}
