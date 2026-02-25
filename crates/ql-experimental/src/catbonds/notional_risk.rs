//! Risky notional models for catastrophe bonds.
//!
//! Translates `ql/experimental/catbonds/riskynotional.hpp/cpp`.
//! These model how catastrophe events reduce the bond's notional:
//! - `DigitalNotionalRisk`: notional drops to 0 if cumulative loss exceeds threshold
//! - `ProportionalNotionalRisk`: notional reduces proportionally between attachment/exhaustion

use ql_core::Real;
use ql_time::Date;

use super::cat_risk::CatEvent;

// ── NotionalPath ──────────────────────────────────────────────────────────────

/// Tracks the remaining notional fraction over time as catastrophe events occur.
///
/// Corresponds to `QuantLib::NotionalPath`.
#[derive(Debug, Clone)]
pub struct NotionalPath {
    /// Sorted list of (date, remaining_notional_rate) reductions.
    reductions: Vec<(Date, Real)>,
}

impl NotionalPath {
    /// Create an empty notional path.
    pub fn new() -> Self {
        Self {
            reductions: Vec::new(),
        }
    }

    /// Reset the path (clear all reductions).
    pub fn reset(&mut self) {
        self.reductions.clear();
    }

    /// Add a notional reduction event.
    pub fn add_reduction(&mut self, date: Date, new_rate: Real) {
        self.reductions.push((date, new_rate));
    }

    /// The remaining notional fraction on `date`.
    pub fn notional_rate(&self, date: Date) -> Real {
        // Find the last reduction on or before `date`
        let mut rate = 1.0;
        for &(d, r) in &self.reductions {
            if d <= date {
                rate = r;
            }
        }
        rate
    }

    /// Total loss fraction = 1 − final notional rate.
    pub fn loss(&self) -> Real {
        if self.reductions.is_empty() {
            0.0
        } else {
            1.0 - self.reductions.last().unwrap().1
        }
    }
}

impl Default for NotionalPath {
    fn default() -> Self {
        Self::new()
    }
}

// ── NotionalRisk ──────────────────────────────────────────────────────────────

/// Trait for updating notional paths based on catastrophe events.
///
/// Corresponds to `QuantLib::NotionalRisk`.
pub trait NotionalRisk {
    /// Update the notional path given a sequence of catastrophe events.
    fn update_path(&self, events: &[CatEvent], path: &mut NotionalPath);
}

// ── DigitalNotionalRisk ───────────────────────────────────────────────────────

/// Digital (binary) notional risk: if any cumulative loss exceeds `threshold`,
/// the notional drops to zero.
///
/// Corresponds to `QuantLib::DigitalNotionalRisk`.
#[derive(Debug, Clone)]
pub struct DigitalNotionalRisk {
    threshold: Real,
}

impl DigitalNotionalRisk {
    /// Create a new digital notional risk with the given loss threshold.
    pub fn new(threshold: Real) -> Self {
        Self { threshold }
    }
}

impl NotionalRisk for DigitalNotionalRisk {
    fn update_path(&self, events: &[CatEvent], path: &mut NotionalPath) {
        path.reset();
        let mut losses = 0.0;
        for &(date, loss) in events {
            losses += loss;
            if losses > self.threshold {
                path.add_reduction(date, 0.0);
                break;
            }
        }
    }
}

// ── ProportionalNotionalRisk ──────────────────────────────────────────────────

/// Proportional notional risk: notional reduces proportionally between
/// `attachment` and `exhaustion` levels.
///
/// - Below `attachment`: no reduction
/// - Above `exhaustion`: notional is zero
/// - Between: `remaining = (exhaustion − cumulative_loss) / (exhaustion − attachment)`
///
/// Corresponds to `QuantLib::ProportionalNotionalRisk`.
#[derive(Debug, Clone)]
pub struct ProportionalNotionalRisk {
    attachment: Real,
    exhaustion: Real,
}

impl ProportionalNotionalRisk {
    /// Create a new proportional notional risk between attachment and exhaustion levels.
    pub fn new(attachment: Real, exhaustion: Real) -> Self {
        assert!(
            exhaustion > attachment,
            "exhaustion ({exhaustion}) must exceed attachment ({attachment})"
        );
        Self {
            attachment,
            exhaustion,
        }
    }
}

impl NotionalRisk for ProportionalNotionalRisk {
    fn update_path(&self, events: &[CatEvent], path: &mut NotionalPath) {
        path.reset();
        let mut losses = 0.0;
        let mut previous_notional = 1.0;

        for &(date, loss) in events {
            losses += loss;
            if losses > self.attachment && previous_notional > 0.0 {
                previous_notional =
                    ((self.exhaustion - losses) / (self.exhaustion - self.attachment)).max(0.0);
                path.add_reduction(date, previous_notional);
            }
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notional_path_basic() {
        let mut path = NotionalPath::new();
        assert_eq!(path.loss(), 0.0);
        assert_eq!(path.notional_rate(Date::from_ymd(2020, 6, 1).unwrap()), 1.0);

        let d1 = Date::from_ymd(2020, 3, 1).unwrap();
        let d2 = Date::from_ymd(2020, 6, 1).unwrap();
        path.add_reduction(d1, 0.7);
        path.add_reduction(d2, 0.3);

        assert_eq!(path.notional_rate(Date::from_ymd(2020, 1, 1).unwrap()), 1.0);
        assert_eq!(path.notional_rate(d1), 0.7);
        assert_eq!(path.notional_rate(Date::from_ymd(2020, 4, 1).unwrap()), 0.7);
        assert_eq!(path.notional_rate(d2), 0.3);
        assert!((path.loss() - 0.7).abs() < 1e-10);
    }

    #[test]
    fn test_digital_notional_risk() {
        let risk = DigitalNotionalRisk::new(100.0);
        let mut path = NotionalPath::new();

        // Events that exceed threshold
        let events: Vec<CatEvent> = vec![
            (Date::from_ymd(2020, 3, 1).unwrap(), 60.0),
            (Date::from_ymd(2020, 6, 1).unwrap(), 50.0), // cumulative 110 > 100
            (Date::from_ymd(2020, 9, 1).unwrap(), 30.0),
        ];
        risk.update_path(&events, &mut path);

        assert_eq!(path.loss(), 1.0);
        assert_eq!(path.notional_rate(Date::from_ymd(2020, 6, 1).unwrap()), 0.0);

        // Events below threshold
        let events_small: Vec<CatEvent> = vec![
            (Date::from_ymd(2020, 3, 1).unwrap(), 30.0),
            (Date::from_ymd(2020, 6, 1).unwrap(), 40.0),
        ];
        risk.update_path(&events_small, &mut path);

        assert_eq!(path.loss(), 0.0); // no reduction
    }

    #[test]
    fn test_proportional_notional_risk() {
        // attachment=500, exhaustion=1500
        let risk = ProportionalNotionalRisk::new(500.0, 1500.0);
        let mut path = NotionalPath::new();

        // Single event of 1000: cumulative = 1000
        // remaining = (1500 - 1000) / (1500 - 500) = 500/1000 = 0.5
        let events: Vec<CatEvent> = vec![(Date::from_ymd(2020, 6, 1).unwrap(), 1000.0)];
        risk.update_path(&events, &mut path);

        assert!((path.loss() - 0.5).abs() < 1e-10);

        // Events that fully exhaust
        let events_doom: Vec<CatEvent> = vec![(Date::from_ymd(2020, 6, 1).unwrap(), 2000.0)];
        risk.update_path(&events_doom, &mut path);

        assert!((path.loss() - 1.0).abs() < 1e-10);

        // Events below attachment
        let events_small: Vec<CatEvent> = vec![(Date::from_ymd(2020, 6, 1).unwrap(), 300.0)];
        risk.update_path(&events_small, &mut path);

        assert_eq!(path.loss(), 0.0);
    }

    #[test]
    fn test_proportional_notional_risk_incremental() {
        // attachment=500, exhaustion=1500
        let risk = ProportionalNotionalRisk::new(500.0, 1500.0);
        let mut path = NotionalPath::new();

        // Two events: 400 (below attachment), then 200 more (cumulative 600 > 500)
        // remaining = (1500 - 600) / (1500 - 500) = 900/1000 = 0.9
        let events: Vec<CatEvent> = vec![
            (Date::from_ymd(2020, 3, 1).unwrap(), 400.0),
            (Date::from_ymd(2020, 6, 1).unwrap(), 200.0),
        ];
        risk.update_path(&events, &mut path);

        assert!((path.loss() - 0.1).abs() < 1e-10);
    }
}
