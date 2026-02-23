//! African currencies (translates `ql/currencies/africa.hpp`).

use crate::currency::Currency;

/// South African Rand.
pub static ZAR: Currency = Currency {
    name: "South African Rand",
    code: "ZAR",
    numeric_code: 710,
    symbol: "R",
    fraction_symbol: "c",
    fractions_per_unit: 100,
    rounding: 2,
};

/// Nigerian Naira.
pub static NGN: Currency = Currency {
    name: "Nigerian Naira",
    code: "NGN",
    numeric_code: 566,
    symbol: "₦",
    fraction_symbol: "k",
    fractions_per_unit: 100,
    rounding: 2,
};

/// Egyptian Pound.
pub static EGP: Currency = Currency {
    name: "Egyptian Pound",
    code: "EGP",
    numeric_code: 818,
    symbol: "E£",
    fraction_symbol: "pt",
    fractions_per_unit: 100,
    rounding: 2,
};

/// Kenyan Shilling.
pub static KES: Currency = Currency {
    name: "Kenyan Shilling",
    code: "KES",
    numeric_code: 404,
    symbol: "KSh",
    fraction_symbol: "c",
    fractions_per_unit: 100,
    rounding: 2,
};

/// Ghanaian Cedi.
pub static GHS: Currency = Currency {
    name: "Ghanaian Cedi",
    code: "GHS",
    numeric_code: 936,
    symbol: "GH₵",
    fraction_symbol: "p",
    fractions_per_unit: 100,
    rounding: 2,
};

/// Moroccan Dirham.
pub static MAD: Currency = Currency {
    name: "Moroccan Dirham",
    code: "MAD",
    numeric_code: 504,
    symbol: "MAD",
    fraction_symbol: "c",
    fractions_per_unit: 100,
    rounding: 2,
};

/// Tunisian Dinar.
pub static TND: Currency = Currency {
    name: "Tunisian Dinar",
    code: "TND",
    numeric_code: 788,
    symbol: "DT",
    fraction_symbol: "m",
    fractions_per_unit: 1000,
    rounding: 3,
};
