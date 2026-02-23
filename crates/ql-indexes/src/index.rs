//! `Index` — base trait for all market indexes (translates `ql/index.hpp`).

use ql_core::Real;
use ql_time::Date;
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

/// Opaque store for historical fixings.
///
/// Thread-safe map from `Date` to fixing value.
#[derive(Debug, Clone, Default)]
pub struct FixingStore {
    data: Arc<RwLock<BTreeMap<Date, Real>>>,
}

impl FixingStore {
    /// Create a new, empty fixing store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a fixing.
    pub fn add(&self, date: Date, value: Real) {
        self.data.write().unwrap().insert(date, value);
    }

    /// Look up a fixing.
    pub fn get(&self, date: Date) -> Option<Real> {
        self.data.read().unwrap().get(&date).copied()
    }

    /// Number of stored fixings.
    pub fn len(&self) -> usize {
        self.data.read().unwrap().len()
    }

    /// Whether the store is empty.
    pub fn is_empty(&self) -> bool {
        self.data.read().unwrap().is_empty()
    }

    /// Clear all fixings.
    pub fn clear(&self) {
        self.data.write().unwrap().clear();
    }
}

/// Base trait for all market indexes.
///
/// Corresponds to `QuantLib::Index`.
pub trait Index: std::fmt::Debug + Send + Sync {
    /// Unique name (e.g. `"EUR-Euribor-6M"`).
    fn name(&self) -> &str;

    /// Calendar used by this index.
    fn fixing_calendar(&self) -> &dyn ql_time::Calendar;

    /// Whether `date` is a valid fixing date.
    fn is_valid_fixing_date(&self, date: Date) -> bool {
        self.fixing_calendar().is_business_day(date)
    }

    /// Return the fixing for `date`, looking up the historical store or
    /// forecasting as needed.
    ///
    /// If `force_forecast` is true, always forecast (ignore stored fixings).
    fn fixing(&self, date: Date, force_forecast: bool) -> ql_core::errors::Result<Real>;

    /// Reference to the historic-fixing store.
    fn fixing_store(&self) -> &FixingStore;

    /// Record a historical fixing.
    fn add_fixing(&self, date: Date, value: Real) {
        self.fixing_store().add(date, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixing_store_round_trip() {
        let store = FixingStore::new();
        let d = Date::from_ymd(2025, 1, 15).unwrap();
        store.add(d, 0.035);
        assert_eq!(store.get(d), Some(0.035));
        assert!(store.get(Date::from_ymd(2025, 1, 16).unwrap()).is_none());
    }

    #[test]
    fn fixing_store_len() {
        let store = FixingStore::new();
        assert!(store.is_empty());
        store.add(Date::from_ymd(2025, 1, 15).unwrap(), 0.01);
        store.add(Date::from_ymd(2025, 1, 16).unwrap(), 0.02);
        assert_eq!(store.len(), 2);
        store.clear();
        assert!(store.is_empty());
    }
}
