//! Pre-defined world currency constants.
//!
//! Translates `ql/currencies/*.hpp`.

use crate::currency::Currency;

/// US Dollar.
pub static USD: Currency = Currency {
    name: "U.S. Dollar",
    code: "USD",
    numeric_code: 840,
    symbol: "$",
    fraction_symbol: "¢",
    fractions_per_unit: 100,
    rounding: 2,
};

/// Euro.
pub static EUR: Currency = Currency {
    name: "Euro",
    code: "EUR",
    numeric_code: 978,
    symbol: "€",
    fraction_symbol: "c",
    fractions_per_unit: 100,
    rounding: 2,
};

/// British pound sterling.
pub static GBP: Currency = Currency {
    name: "British Pound",
    code: "GBP",
    numeric_code: 826,
    symbol: "£",
    fraction_symbol: "p",
    fractions_per_unit: 100,
    rounding: 2,
};

/// Japanese Yen.
pub static JPY: Currency = Currency {
    name: "Japanese Yen",
    code: "JPY",
    numeric_code: 392,
    symbol: "¥",
    fraction_symbol: "¥",
    fractions_per_unit: 1,
    rounding: 0,
};

/// Swiss Franc.
pub static CHF: Currency = Currency {
    name: "Swiss Franc",
    code: "CHF",
    numeric_code: 756,
    symbol: "Fr",
    fraction_symbol: "c",
    fractions_per_unit: 100,
    rounding: 2,
};

/// Canadian Dollar.
pub static CAD: Currency = Currency {
    name: "Canadian Dollar",
    code: "CAD",
    numeric_code: 124,
    symbol: "CA$",
    fraction_symbol: "¢",
    fractions_per_unit: 100,
    rounding: 2,
};

/// Australian Dollar.
pub static AUD: Currency = Currency {
    name: "Australian Dollar",
    code: "AUD",
    numeric_code: 36,
    symbol: "A$",
    fraction_symbol: "¢",
    fractions_per_unit: 100,
    rounding: 2,
};

/// Hong Kong Dollar.
pub static HKD: Currency = Currency {
    name: "Hong Kong Dollar",
    code: "HKD",
    numeric_code: 344,
    symbol: "HK$",
    fraction_symbol: "¢",
    fractions_per_unit: 100,
    rounding: 2,
};

/// Norwegian Krone.
pub static NOK: Currency = Currency {
    name: "Norwegian Krone",
    code: "NOK",
    numeric_code: 578,
    symbol: "kr",
    fraction_symbol: "øre",
    fractions_per_unit: 100,
    rounding: 2,
};

/// Swedish Krona.
pub static SEK: Currency = Currency {
    name: "Swedish Krona",
    code: "SEK",
    numeric_code: 752,
    symbol: "kr",
    fraction_symbol: "öre",
    fractions_per_unit: 100,
    rounding: 2,
};

/// Danish Krone.
pub static DKK: Currency = Currency {
    name: "Danish Krone",
    code: "DKK",
    numeric_code: 208,
    symbol: "kr",
    fraction_symbol: "øre",
    fractions_per_unit: 100,
    rounding: 2,
};

/// Brazilian Real.
pub static BRL: Currency = Currency {
    name: "Brazilian Real",
    code: "BRL",
    numeric_code: 986,
    symbol: "R$",
    fraction_symbol: "centavo",
    fractions_per_unit: 100,
    rounding: 2,
};

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
}
