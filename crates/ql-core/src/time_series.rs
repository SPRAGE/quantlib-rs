//! Generic time-series container (translates `ql/timeseries.hpp`).
//!
//! `TimeSeries<T>` is an ordered map from `Date` to `T`, providing the same
//! interface as the C++ `QuantLib::TimeSeries<T>`.

use std::collections::BTreeMap;

/// A generic time-indexed container backed by a `BTreeMap`.
///
/// Corresponds to `QuantLib::TimeSeries<T>`.  The Rust version is generic
/// over any `Clone` value type and does **not** require a `Null<T>()` —
/// missing keys simply return `None`.
#[derive(Debug, Clone, PartialEq)]
pub struct TimeSeries<K: Ord + Clone, V: Clone> {
    data: BTreeMap<K, V>,
}

impl<K: Ord + Clone, V: Clone> Default for TimeSeries<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Ord + Clone, V: Clone> std::iter::FromIterator<(K, V)> for TimeSeries<K, V> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        Self {
            data: iter.into_iter().collect(),
        }
    }
}

impl<K: Ord + Clone, V: Clone> TimeSeries<K, V> {
    // ── Constructors ─────────────────────────────────────────────────────

    /// Create an empty time series.
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }

    /// Build from parallel slices of keys and values.
    ///
    /// # Panics
    /// Panics if `keys.len() != values.len()`.
    pub fn from_key_values(keys: &[K], values: &[V]) -> Self {
        assert_eq!(
            keys.len(),
            values.len(),
            "TimeSeries: keys and values must have the same length"
        );
        let mut data = BTreeMap::new();
        for (k, v) in keys.iter().zip(values.iter()) {
            data.insert(k.clone(), v.clone());
        }
        Self { data }
    }

    /// Build from an iterator of `(K, V)` pairs.
    pub fn from_pairs(iter: impl IntoIterator<Item = (K, V)>) -> Self {
        Self {
            data: iter.into_iter().collect(),
        }
    }

    // ── Inspectors ───────────────────────────────────────────────────────

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Whether the series is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// The earliest key, or `None` if empty.
    pub fn first_key(&self) -> Option<&K> {
        self.data.keys().next()
    }

    /// The latest key, or `None` if empty.
    pub fn last_key(&self) -> Option<&K> {
        self.data.keys().next_back()
    }

    // ── Element access ───────────────────────────────────────────────────

    /// Look up a value by key.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.data.get(key)
    }

    /// Insert or overwrite a value.
    pub fn insert(&mut self, key: K, value: V) {
        self.data.insert(key, value);
    }

    /// Remove an entry, returning its value if present.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.data.remove(key)
    }

    /// Whether a key is present.
    pub fn contains_key(&self, key: &K) -> bool {
        self.data.contains_key(key)
    }

    // ── Bulk access ──────────────────────────────────────────────────────

    /// All keys in ascending order.
    pub fn keys(&self) -> Vec<K> {
        self.data.keys().cloned().collect()
    }

    /// All values in key-ascending order.
    pub fn values(&self) -> Vec<V> {
        self.data.values().cloned().collect()
    }

    /// Iterate over `(&K, &V)` in ascending key order.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.data.iter()
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Access the underlying `BTreeMap`.
    pub fn as_map(&self) -> &BTreeMap<K, V> {
        &self.data
    }
}

impl<K: Ord + Clone, V: Clone> std::ops::Index<&K> for TimeSeries<K, V> {
    type Output = V;

    fn index(&self, key: &K) -> &V {
        &self.data[key]
    }
}

impl<K: Ord + Clone + std::fmt::Debug, V: Clone + std::fmt::Display> std::fmt::Display
    for TimeSeries<K, V>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (k, v) in &self.data {
            writeln!(f, "{k:?} => {v}")?;
        }
        Ok(())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_construction() {
        let ts: TimeSeries<i32, f64> = TimeSeries::new();
        assert!(ts.is_empty());
        assert_eq!(ts.len(), 0);
        assert!(ts.first_key().is_none());
        assert!(ts.last_key().is_none());
    }

    #[test]
    fn from_key_values() {
        let keys = vec![3, 1, 2];
        let vals = vec![30.0, 10.0, 20.0];
        let ts = TimeSeries::from_key_values(&keys, &vals);
        assert_eq!(ts.len(), 3);
        assert_eq!(*ts.first_key().unwrap(), 1);
        assert_eq!(*ts.last_key().unwrap(), 3);
        assert_eq!(ts[&1], 10.0);
        assert_eq!(ts[&2], 20.0);
        assert_eq!(ts[&3], 30.0);
    }

    #[test]
    fn insert_and_get() {
        let mut ts = TimeSeries::new();
        ts.insert(10, "hello");
        ts.insert(20, "world");
        assert_eq!(ts.get(&10), Some(&"hello"));
        assert_eq!(ts.get(&20), Some(&"world"));
        assert_eq!(ts.get(&15), None);
    }

    #[test]
    fn remove_and_contains() {
        let mut ts = TimeSeries::from_key_values(&[1, 2, 3], &[10, 20, 30]);
        assert!(ts.contains_key(&2));
        assert_eq!(ts.remove(&2), Some(20));
        assert!(!ts.contains_key(&2));
        assert_eq!(ts.len(), 2);
    }

    #[test]
    fn keys_and_values_sorted() {
        let ts = TimeSeries::from_key_values(&[3, 1, 2], &[30, 10, 20]);
        assert_eq!(ts.keys(), vec![1, 2, 3]);
        assert_eq!(ts.values(), vec![10, 20, 30]);
    }

    #[test]
    fn iteration() {
        let ts = TimeSeries::from_key_values(&[1, 2, 3], &[10.0, 20.0, 30.0]);
        let collected: Vec<_> = ts.iter().map(|(k, v)| (*k, *v)).collect();
        assert_eq!(collected, vec![(1, 10.0), (2, 20.0), (3, 30.0)]);
    }

    #[test]
    fn clear() {
        let mut ts = TimeSeries::from_key_values(&[1, 2], &[10, 20]);
        assert!(!ts.is_empty());
        ts.clear();
        assert!(ts.is_empty());
    }

    #[test]
    fn overwrite_existing_key() {
        let mut ts = TimeSeries::new();
        ts.insert(1, 100);
        ts.insert(1, 200);
        assert_eq!(ts[&1], 200);
        assert_eq!(ts.len(), 1);
    }

    #[test]
    fn display_format() {
        let ts = TimeSeries::from_key_values(&[1, 2], &[3.125, 2.625]);
        let s = format!("{ts}");
        assert!(s.contains("3.125"));
        assert!(s.contains("2.625"));
    }
}
