//! Oceania currencies (translates `ql/currencies/oceania.hpp`).

use crate::currency::Currency;

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

/// New Zealand Dollar.
pub static NZD: Currency = Currency {
    name: "New Zealand Dollar",
    code: "NZD",
    numeric_code: 554,
    symbol: "NZ$",
    fraction_symbol: "¢",
    fractions_per_unit: 100,
    rounding: 2,
};
