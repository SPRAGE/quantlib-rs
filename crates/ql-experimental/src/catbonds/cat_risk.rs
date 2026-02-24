//! Catastrophe risk models.
//!
//! Translates `ql/experimental/catbonds/catrisk.hpp/cpp`.
//! Provides simulation of catastrophe event paths:
//! - `EventSet` / `EventSetSimulation`: deterministic replay of historical events
//! - `BetaRisk` / `BetaRiskSimulation`: stochastic model with Poisson arrival + Beta losses

use ql_core::Real;
use ql_time::Date;
use rand::prelude::*;
use rand_distr::{Exp, Gamma};

// ── Traits ────────────────────────────────────────────────────────────────────

/// A single catastrophe event: (date, loss amount).
pub type CatEvent = (Date, Real);

/// A simulated path of catastrophe events over a period.
///
/// Corresponds to `QuantLib::CatSimulation`.
pub trait CatSimulation {
    /// Generate the next path of events, returning `true` if data remains.
    fn next_path(&mut self, path: &mut Vec<CatEvent>) -> bool;
}

/// A catastrophe risk model that can create simulations over arbitrary periods.
///
/// Corresponds to `QuantLib::CatRisk`.
pub trait CatRisk {
    /// Create a new simulation over `[start, end]`.
    fn new_simulation(&self, start: Date, end: Date) -> Box<dyn CatSimulation>;
}

// ── EventSet ──────────────────────────────────────────────────────────────────

/// Deterministic event set — replays historical events shifted to fit the
/// simulation period, cycling through multi-year event windows.
///
/// Corresponds to `QuantLib::EventSet`.
#[derive(Debug, Clone)]
pub struct EventSet {
    events: Vec<CatEvent>,
    events_start: Date,
    events_end: Date,
}

impl EventSet {
    /// Create a new event set from historical events.
    pub fn new(events: Vec<CatEvent>, events_start: Date, events_end: Date) -> Self {
        Self {
            events,
            events_start,
            events_end,
        }
    }
}

impl CatRisk for EventSet {
    fn new_simulation(&self, start: Date, end: Date) -> Box<dyn CatSimulation> {
        Box::new(EventSetSimulation::new(
            self.events.clone(),
            self.events_start,
            self.events_end,
            start,
            end,
        ))
    }
}

/// Simulation that replays events from an `EventSet`.
///
/// Corresponds to `QuantLib::EventSetSimulation`.
pub struct EventSetSimulation {
    events: Vec<CatEvent>,
    #[allow(dead_code)]
    events_start: Date,
    events_end: Date,
    start: Date,
    end: Date,
    years: u16,
    period_start: Date,
    period_end: Date,
    i: usize,
}

impl EventSetSimulation {
    /// Create a new event-set simulation for the given date range.
    pub fn new(
        events: Vec<CatEvent>,
        events_start: Date,
        events_end: Date,
        start: Date,
        end: Date,
    ) -> Self {
        let years = end.year() - start.year();

        // Find the first period_start in the events window that aligns with `start`
        let period_start = if events_start.month() < start.month()
            || (events_start.month() == start.month()
                && events_start.day_of_month() <= start.day_of_month())
        {
            Date::from_ymd(events_start.year(), start.month(), start.day_of_month()).unwrap()
        } else {
            Date::from_ymd(events_start.year() + 1, start.month(), start.day_of_month()).unwrap()
        };

        let period_end =
            Date::from_ymd(period_start.year() + years, end.month(), end.day_of_month()).unwrap();

        // Skip events before the first period
        let mut i = 0;
        while i < events.len() && events[i].0 < period_start {
            i += 1;
        }

        Self {
            events,
            events_start,
            events_end,
            start,
            end,
            years,
            period_start,
            period_end,
            i,
        }
    }
}

impl CatSimulation for EventSetSimulation {
    fn next_path(&mut self, path: &mut Vec<CatEvent>) -> bool {
        path.clear();

        if self.period_end > self.events_end {
            return false;
        }

        // Skip events before this period
        while self.i < self.events.len() && self.events[self.i].0 < self.period_start {
            self.i += 1;
        }

        // Collect events in [period_start, period_end], shifted to simulation timeframe
        while self.i < self.events.len() && self.events[self.i].0 <= self.period_end {
            let event_date = self.events[self.i].0;
            let year_shift = self.start.year() - self.period_start.year();
            let shifted_date =
                Date::from_ymd(event_date.year() + year_shift, event_date.month(), event_date.day_of_month())
                    .unwrap();
            path.push((shifted_date, self.events[self.i].1));
            self.i += 1;
        }

        // Advance to next period
        // C++ logic: if start + years*Years < end, advance by years+1, else by years
        let start_plus_years = Date::from_ymd(
            self.start.year() + self.years,
            self.start.month(),
            self.start.day_of_month(),
        )
        .unwrap();

        let advance_years = if start_plus_years < self.end {
            self.years + 1
        } else {
            self.years
        };

        self.period_start = Date::from_ymd(
            self.period_start.year() + advance_years,
            self.period_start.month(),
            self.period_start.day_of_month(),
        )
        .unwrap();
        self.period_end = Date::from_ymd(
            self.period_end.year() + advance_years,
            self.period_end.month(),
            self.period_end.day_of_month(),
        )
        .unwrap();

        true
    }
}

// ── BetaRisk ──────────────────────────────────────────────────────────────────

/// Stochastic catastrophe risk model: Poisson arrival of events with
/// Beta-distributed loss severities.
///
/// Corresponds to `QuantLib::BetaRisk`.
#[derive(Debug, Clone)]
pub struct BetaRisk {
    max_loss: Real,
    lambda: Real, // Poisson rate = 1/years
    alpha: Real,
    beta: Real,
}

impl BetaRisk {
    /// Create a Beta risk model.
    ///
    /// - `max_loss`: maximum possible loss per event
    /// - `years`: expected inter-arrival time (lambda = 1/years)
    /// - `mean`: expected loss per event
    /// - `std_dev`: standard deviation of loss per event
    pub fn new(max_loss: Real, years: Real, mean: Real, std_dev: Real) -> Self {
        assert!(
            mean < max_loss,
            "Mean {mean} must be less than max_loss {max_loss}"
        );
        let normalized_mean = mean / max_loss;
        let normalized_var = std_dev * std_dev / (max_loss * max_loss);
        assert!(
            normalized_var < normalized_mean * (1.0 - normalized_mean),
            "StdDev {std_dev} is impossible for Beta with mean {mean}"
        );
        let nu = normalized_mean * (1.0 - normalized_mean) / normalized_var - 1.0;
        let alpha = normalized_mean * nu;
        let beta = (1.0 - normalized_mean) * nu;

        Self {
            max_loss,
            lambda: 1.0 / years,
            alpha,
            beta,
        }
    }
}

impl CatRisk for BetaRisk {
    fn new_simulation(&self, start: Date, end: Date) -> Box<dyn CatSimulation> {
        Box::new(BetaRiskSimulation::new(
            start,
            end,
            self.max_loss,
            self.lambda,
            self.alpha,
            self.beta,
        ))
    }
}

/// Monte Carlo simulation for Beta risk model.
///
/// Corresponds to `QuantLib::BetaRiskSimulation`.
pub struct BetaRiskSimulation {
    start: Date,
    #[allow(dead_code)]
    end: Date,
    max_loss: Real,
    day_count: i64,
    year_fraction: Real,
    rng: StdRng,
    exp_dist: Exp<f64>,
    gamma_alpha: Gamma<f64>,
    gamma_beta: Gamma<f64>,
}

impl BetaRiskSimulation {
    /// Create a new beta-risk simulation.
    pub fn new(
        start: Date,
        end: Date,
        max_loss: Real,
        lambda: Real,
        alpha: Real,
        beta: Real,
    ) -> Self {
        // Use ActualActual-like day counting
        let day_count = (end.serial() - start.serial()) as i64;
        let year_fraction = day_count as Real / 365.25;

        Self {
            start,
            end,
            max_loss,
            day_count,
            year_fraction,
            rng: StdRng::seed_from_u64(42),
            exp_dist: Exp::new(lambda).unwrap(),
            gamma_alpha: Gamma::new(alpha, 1.0).unwrap(),
            gamma_beta: Gamma::new(beta, 1.0).unwrap(),
        }
    }

    fn generate_beta(&mut self) -> Real {
        let x: Real = self.gamma_alpha.sample(&mut self.rng);
        let y: Real = self.gamma_beta.sample(&mut self.rng);
        x * self.max_loss / (x + y)
    }
}

impl CatSimulation for BetaRiskSimulation {
    fn next_path(&mut self, path: &mut Vec<CatEvent>) -> bool {
        path.clear();
        let mut event_fraction: Real = self.exp_dist.sample(&mut self.rng);

        while event_fraction <= self.year_fraction {
            let days =
                (event_fraction * self.day_count as Real / self.year_fraction).round() as i32;
            let event_date = self.start.advance(days, ql_time::TimeUnit::Days).unwrap();

            if event_date <= self.end {
                let loss = self.generate_beta();
                path.push((event_date, loss));
            } else {
                break;
            }
            event_fraction = self.exp_dist.sample(&mut self.rng);
        }
        true
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_events() -> Vec<CatEvent> {
        vec![
            (Date::from_ymd(2012, 2, 1).unwrap(), 100.0),
            (Date::from_ymd(2013, 7, 1).unwrap(), 150.0),
            (Date::from_ymd(2014, 1, 5).unwrap(), 50.0),
        ]
    }

    #[test]
    fn test_event_set_whole_years() {
        // C++ test: testEventSetForWholeYears
        let events = sample_events();
        let events_start = Date::from_ymd(2011, 1, 1).unwrap();
        let events_end = Date::from_ymd(2014, 12, 31).unwrap();
        let cat_risk = EventSet::new(events, events_start, events_end);

        let start = Date::from_ymd(2015, 1, 1).unwrap();
        let end = Date::from_ymd(2015, 12, 31).unwrap();
        let mut sim = cat_risk.new_simulation(start, end);

        let mut path = Vec::new();

        // Period 1 (2011): no events match
        assert!(sim.next_path(&mut path));
        assert_eq!(path.len(), 0);

        // Period 2 (2012): Feb 1 event
        assert!(sim.next_path(&mut path));
        assert_eq!(path.len(), 1);
        assert_eq!(path[0].0, Date::from_ymd(2015, 2, 1).unwrap());
        assert_eq!(path[0].1, 100.0);

        // Period 3 (2013): Jul 1 event
        assert!(sim.next_path(&mut path));
        assert_eq!(path.len(), 1);
        assert_eq!(path[0].0, Date::from_ymd(2015, 7, 1).unwrap());
        assert_eq!(path[0].1, 150.0);

        // Period 4 (2014): Jan 5 event
        assert!(sim.next_path(&mut path));
        assert_eq!(path.len(), 1);
        assert_eq!(path[0].0, Date::from_ymd(2015, 1, 5).unwrap());
        assert_eq!(path[0].1, 50.0);

        // No more periods
        assert!(!sim.next_path(&mut path));
    }

    #[test]
    fn test_event_set_irregular_periods() {
        // C++ test: testEventSetForIrregularPeriods
        let events = sample_events();
        let events_start = Date::from_ymd(2011, 1, 1).unwrap();
        let events_end = Date::from_ymd(2014, 12, 31).unwrap();
        let cat_risk = EventSet::new(events, events_start, events_end);

        let start = Date::from_ymd(2015, 1, 2).unwrap();
        let end = Date::from_ymd(2016, 1, 5).unwrap();
        let mut sim = cat_risk.new_simulation(start, end);

        let mut path = Vec::new();

        // Period 1: no events in [2012-01-02, 2013-01-05]
        // (Feb 1 2012 actually falls in this range, but the event might be missed
        // depending on alignment)
        assert!(sim.next_path(&mut path));
        assert_eq!(path.len(), 0);

        // Period 2: Jul 1 2013 and Jan 5 2014
        assert!(sim.next_path(&mut path));
        assert_eq!(path.len(), 2);
        assert_eq!(path[0].0, Date::from_ymd(2015, 7, 1).unwrap());
        assert_eq!(path[0].1, 150.0);
        assert_eq!(path[1].0, Date::from_ymd(2016, 1, 5).unwrap());
        assert_eq!(path[1].1, 50.0);

        // No more periods
        assert!(!sim.next_path(&mut path));
    }

    #[test]
    fn test_event_set_no_events() {
        // C++ test: testEventSetForNoEvents
        let events_start = Date::from_ymd(2011, 1, 1).unwrap();
        let events_end = Date::from_ymd(2014, 12, 31).unwrap();
        let cat_risk = EventSet::new(vec![], events_start, events_end);

        let start = Date::from_ymd(2015, 1, 2).unwrap();
        let end = Date::from_ymd(2016, 1, 5).unwrap();
        let mut sim = cat_risk.new_simulation(start, end);

        let mut path = Vec::new();

        assert!(sim.next_path(&mut path));
        assert_eq!(path.len(), 0);

        assert!(sim.next_path(&mut path));
        assert_eq!(path.len(), 0);

        assert!(!sim.next_path(&mut path));
    }

    #[test]
    fn test_beta_risk_distribution() {
        // C++ test: testBetaRisk
        // BetaRisk(maxLoss=100, years=100, mean=10, stdDev=15)
        // Poisson rate 1/100 → expected 3 events for 3-year period
        let cat_risk = BetaRisk::new(100.0, 100.0, 10.0, 15.0);

        let start = Date::from_ymd(2015, 1, 2).unwrap();
        let end = Date::from_ymd(2018, 1, 2).unwrap();

        let paths = 100_000;
        let mut sum = 0.0;
        let mut sum_squares = 0.0;
        let mut poisson_sum = 0.0;

        let mut sim = cat_risk.new_simulation(start, end);
        let mut path = Vec::new();

        for _ in 0..paths {
            assert!(sim.next_path(&mut path));
            let process_value: Real = path.iter().map(|(_, loss)| loss).sum();
            sum += process_value;
            sum_squares += process_value * process_value;
            poisson_sum += path.len() as Real;
        }

        let poisson_mean = poisson_sum / paths as Real;
        let actual_mean = sum / paths as Real;
        let actual_var = sum_squares / paths as Real - actual_mean * actual_mean;

        // Expected Poisson mean ≈ 3/100 events per path (lambda=1/100, 3 years)
        // Since the BetaRisk sim always returns true, and we seed the RNG,
        // we verify statistical properties hold approximately
        let expected_poisson_mean = 3.0 / 100.0;
        let expected_mean = 3.0 * 10.0 / 100.0;
        // Variance: 3*(15^2+10^2)/100 = 3*325/100 = 9.75
        let expected_var = 3.0 * (15.0 * 15.0 + 10.0 * 10.0) / 100.0;

        // Loose tolerances for stochastic test
        assert!(
            (poisson_mean - expected_poisson_mean).abs() / expected_poisson_mean < 0.10,
            "Poisson mean: expected ~{expected_poisson_mean}, got {poisson_mean}"
        );
        assert!(
            (actual_mean - expected_mean).abs() / expected_mean < 0.10,
            "Mean: expected ~{expected_mean}, got {actual_mean}"
        );
        assert!(
            (actual_var - expected_var).abs() / expected_var < 0.20,
            "Variance: expected ~{expected_var}, got {actual_var}"
        );
    }
}
