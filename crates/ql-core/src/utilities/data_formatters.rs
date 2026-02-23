//! Data formatting helpers (translates `ql/utilities/dataformatters.hpp`).
//!
//! Provides formatting functions for rates, volatilities, and ordinal
//! numbers, mirroring QuantLib's `io::rate()`, `io::volatility()`, and
//! `io::ordinal()`.

use ql_core_types::{Rate, Real, Volatility};

// We use the parent crate's type aliases via a private helper module.
// Since we *are* inside ql-core, we just use the raw types directly.
mod ql_core_types {
    pub type Real = f64;
    pub type Rate = f64;
    pub type Volatility = f64;
}

/// Format a rate as a percentage string (e.g. `0.05` → `"5.000000 %"`).
pub fn format_rate(r: Rate) -> String {
    format!("{:.6} %", r * 100.0)
}

/// Format a volatility as a percentage string (e.g. `0.20` → `"20.000000 %"`).
pub fn format_volatility(v: Volatility) -> String {
    format!("{:.6} %", v * 100.0)
}

/// Format a real number with the given number of decimal places.
pub fn format_real(value: Real, decimals: usize) -> String {
    format!("{:.prec$}", value, prec = decimals)
}

/// Return the English ordinal suffix for `n` (e.g. `1` → `"st"`, `2` → `"nd"`).
pub fn ordinal_suffix(n: u32) -> &'static str {
    match n % 100 {
        11..=13 => "th",
        _ => match n % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        },
    }
}

/// Format a number with its ordinal suffix (e.g. `1` → `"1st"`, `22` → `"22nd"`).
pub fn format_ordinal(n: u32) -> String {
    format!("{n}{}", ordinal_suffix(n))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_rate() {
        assert_eq!(format_rate(0.05), "5.000000 %");
    }

    #[test]
    fn test_ordinal() {
        assert_eq!(format_ordinal(1), "1st");
        assert_eq!(format_ordinal(2), "2nd");
        assert_eq!(format_ordinal(3), "3rd");
        assert_eq!(format_ordinal(4), "4th");
        assert_eq!(format_ordinal(11), "11th");
        assert_eq!(format_ordinal(12), "12th");
        assert_eq!(format_ordinal(13), "13th");
        assert_eq!(format_ordinal(21), "21st");
        assert_eq!(format_ordinal(22), "22nd");
    }
}
