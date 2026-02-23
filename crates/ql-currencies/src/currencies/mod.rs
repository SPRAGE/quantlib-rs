//! Pre-defined world currencies, organized by region.
//!
//! Translates `ql/currencies/*.hpp`.

pub mod africa;
pub mod america;
pub mod asia;
pub mod crypto;
pub mod europe;
pub mod oceania;

// Re-export all currencies at the `currencies` module level for convenience.
pub use africa::*;
pub use america::*;
pub use asia::*;
pub use crypto::*;
pub use europe::*;
pub use oceania::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usd_properties() {
        assert_eq!(USD.code, "USD");
        assert_eq!(USD.numeric_code, 840);
        assert_eq!(USD.fractions_per_unit, 100);
    }

    #[test]
    fn eur_display() {
        assert_eq!(format!("{}", &EUR), "EUR");
    }

    #[test]
    fn jpy_no_fractions() {
        assert_eq!(JPY.fractions_per_unit, 1);
        assert_eq!(JPY.rounding, 0);
    }

    #[test]
    fn all_currencies_have_code() {
        let all: Vec<&crate::currency::Currency> = vec![
            &USD, &CAD, &BRL, &MXN, &ARS, &CLP, &COP, &PEN,
            &EUR, &GBP, &CHF, &NOK, &SEK, &DKK, &PLN, &CZK, &HUF,
            &RON, &BGN, &HRK, &ISK, &TRY, &RUB,
            &JPY, &CNY, &HKD, &SGD, &KRW, &INR, &TWD, &THB,
            &MYR, &IDR, &PHP, &ILS, &SAR, &AED,
            &AUD, &NZD,
            &ZAR, &NGN, &EGP, &KES, &GHS, &MAD, &TND,
            &BTC, &ETH,
        ];
        for c in all {
            assert!(!c.code.is_empty(), "currency has empty code: {:?}", c.name);
            assert!(c.numeric_code > 0 || c.code == "BTC" || c.code == "ETH",
                "suspect numeric code for {}", c.code);
        }
    }
}
