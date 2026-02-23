//! Exchange rate and currency conversion.
//!
//! Translates `ql/exchangerate.hpp` and `ql/exchangeratemanager.hpp`.

use crate::currency::{Currency, Money};
use ql_core::{
    errors::{Error, Result},
    Real,
};
use std::collections::HashMap;

/// An exchange rate between two currencies.
///
/// Corresponds to `QuantLib::ExchangeRate`.
#[derive(Debug, Clone)]
pub struct ExchangeRate {
    /// The source currency.
    pub source: &'static Currency,
    /// The target currency.
    pub target: &'static Currency,
    /// Rate: how many units of `target` one unit of `source` buys.
    pub rate: Real,
}

impl ExchangeRate {
    /// Create a new exchange rate.
    pub fn new(source: &'static Currency, target: &'static Currency, rate: Real) -> Self {
        Self {
            source,
            target,
            rate,
        }
    }

    /// Convert a monetary amount from `source` to `target` currency.
    pub fn exchange(&self, amount: &Money) -> Result<Money> {
        if amount.currency == self.source {
            Ok(Money::new(amount.value * self.rate, self.target))
        } else if amount.currency == self.target {
            Ok(Money::new(amount.value / self.rate, self.source))
        } else {
            Err(Error::Runtime(format!(
                "ExchangeRate({}/{}) cannot convert {}",
                self.source.code, self.target.code, amount.currency.code
            )))
        }
    }

    /// Return the inverse rate (target → source).
    pub fn inverse(&self) -> Self {
        Self {
            source: self.target,
            target: self.source,
            rate: 1.0 / self.rate,
        }
    }
}

/// A registry of exchange rates.
///
/// Stores direct rates and can chain through a common currency (typically
/// USD or EUR) to derive cross rates.
///
/// Corresponds loosely to `QuantLib::ExchangeRateManager`.
#[derive(Debug, Default)]
pub struct ExchangeRateManager {
    rates: HashMap<(&'static str, &'static str), ExchangeRate>,
}

impl ExchangeRateManager {
    /// Create an empty manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an exchange rate.
    pub fn add(&mut self, rate: ExchangeRate) {
        let key = (rate.source.code, rate.target.code);
        self.rates.insert(key, rate);
    }

    /// Look up a rate for `source → target`, searching direct rates and
    /// one-hop cross rates through any common currency.
    pub fn lookup(
        &self,
        source: &'static Currency,
        target: &'static Currency,
    ) -> Result<ExchangeRate> {
        if source == target {
            return Ok(ExchangeRate::new(source, target, 1.0));
        }

        // Direct hit?
        if let Some(r) = self.rates.get(&(source.code, target.code)) {
            return Ok(r.clone());
        }
        // Inverse direct?
        if let Some(r) = self.rates.get(&(target.code, source.code)) {
            return Ok(r.inverse());
        }

        // Try one-hop cross: source → X → target
        for rate_sx in self.rates.values() {
            let x = if rate_sx.source == source {
                rate_sx.target
            } else if rate_sx.target == source {
                rate_sx.source
            } else {
                continue;
            };

            // Now look for X → target
            if let Some(rate_xt) = self.rates.get(&(x.code, target.code)) {
                let sx_rate = if rate_sx.source == source {
                    rate_sx.rate
                } else {
                    1.0 / rate_sx.rate
                };
                let xt_rate = rate_xt.rate;
                return Ok(ExchangeRate::new(source, target, sx_rate * xt_rate));
            }
            if let Some(rate_tx) = self.rates.get(&(target.code, x.code)) {
                let sx_rate = if rate_sx.source == source {
                    rate_sx.rate
                } else {
                    1.0 / rate_sx.rate
                };
                let xt_rate = 1.0 / rate_tx.rate;
                return Ok(ExchangeRate::new(source, target, sx_rate * xt_rate));
            }
        }

        Err(Error::Runtime(format!(
            "no exchange rate found for {}/{}",
            source.code, target.code
        )))
    }

    /// Convert a monetary amount to the target currency.
    pub fn convert(&self, amount: &Money, target: &'static Currency) -> Result<Money> {
        let rate = self.lookup(amount.currency, target)?;
        rate.exchange(amount)
    }

    /// Remove all registered rates.
    pub fn clear(&mut self) {
        self.rates.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::currencies::{EUR, GBP, JPY, USD};

    #[test]
    fn direct_exchange() {
        let rate = ExchangeRate::new(&USD, &EUR, 0.85);
        let usd_100 = Money::new(100.0, &USD);
        let eur = rate.exchange(&usd_100).unwrap();
        assert_eq!(eur.currency, &EUR);
        assert!((eur.value - 85.0).abs() < 1e-12);
    }

    #[test]
    fn inverse_exchange() {
        let rate = ExchangeRate::new(&USD, &EUR, 0.85);
        let inv = rate.inverse();
        assert_eq!(inv.source, &EUR);
        assert_eq!(inv.target, &USD);
        assert!((inv.rate - 1.0 / 0.85).abs() < 1e-12);
    }

    #[test]
    fn manager_direct_lookup() {
        let mut mgr = ExchangeRateManager::new();
        mgr.add(ExchangeRate::new(&USD, &EUR, 0.85));
        let rate = mgr.lookup(&USD, &EUR).unwrap();
        assert!((rate.rate - 0.85).abs() < 1e-12);
    }

    #[test]
    fn manager_inverse_lookup() {
        let mut mgr = ExchangeRateManager::new();
        mgr.add(ExchangeRate::new(&USD, &EUR, 0.85));
        let rate = mgr.lookup(&EUR, &USD).unwrap();
        assert!((rate.rate - 1.0 / 0.85).abs() < 1e-12);
    }

    #[test]
    fn manager_cross_rate() {
        let mut mgr = ExchangeRateManager::new();
        mgr.add(ExchangeRate::new(&USD, &EUR, 0.85));
        mgr.add(ExchangeRate::new(&USD, &GBP, 0.75));
        // EUR → GBP via USD: EUR → USD → GBP
        // EUR → USD = 1/0.85, USD → GBP = 0.75
        let rate = mgr.lookup(&EUR, &GBP).unwrap();
        let expected = (1.0 / 0.85) * 0.75;
        assert!(
            (rate.rate - expected).abs() < 1e-10,
            "got {}, expected {}",
            rate.rate,
            expected
        );
    }

    #[test]
    fn manager_convert() {
        let mut mgr = ExchangeRateManager::new();
        mgr.add(ExchangeRate::new(&USD, &JPY, 110.0));
        let amount = Money::new(50.0, &USD);
        let jpy = mgr.convert(&amount, &JPY).unwrap();
        assert_eq!(jpy.currency, &JPY);
        assert!((jpy.value - 5500.0).abs() < 1e-10);
    }

    #[test]
    fn manager_same_currency() {
        let mgr = ExchangeRateManager::new();
        let rate = mgr.lookup(&USD, &USD).unwrap();
        assert!((rate.rate - 1.0).abs() < 1e-12);
    }

    #[test]
    fn money_arithmetic() {
        let a = Money::new(100.0, &USD);
        let b = Money::new(50.0, &USD);
        let sum = a.clone() + b.clone();
        assert!((sum.value - 150.0).abs() < 1e-12);
        let diff = a.clone() - b;
        assert!((diff.value - 50.0).abs() < 1e-12);
        let neg = -a.clone();
        assert!((neg.value - (-100.0)).abs() < 1e-12);
        let scaled = a * 2.0;
        assert!((scaled.value - 200.0).abs() < 1e-12);
    }
}
