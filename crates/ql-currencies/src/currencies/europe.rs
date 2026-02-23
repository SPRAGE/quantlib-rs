//! European currencies (translates `ql/currencies/europe.hpp`).

use crate::currency::Currency;

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

/// British Pound Sterling.
pub static GBP: Currency = Currency {
    name: "British Pound",
    code: "GBP",
    numeric_code: 826,
    symbol: "£",
    fraction_symbol: "p",
    fractions_per_unit: 100,
    rounding: 2,
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

/// Polish Zloty.
pub static PLN: Currency = Currency {
    name: "Polish Zloty",
    code: "PLN",
    numeric_code: 985,
    symbol: "zł",
    fraction_symbol: "gr",
    fractions_per_unit: 100,
    rounding: 2,
};

/// Czech Koruna.
pub static CZK: Currency = Currency {
    name: "Czech Koruna",
    code: "CZK",
    numeric_code: 203,
    symbol: "Kč",
    fraction_symbol: "h",
    fractions_per_unit: 100,
    rounding: 2,
};

/// Hungarian Forint.
pub static HUF: Currency = Currency {
    name: "Hungarian Forint",
    code: "HUF",
    numeric_code: 348,
    symbol: "Ft",
    fraction_symbol: "f",
    fractions_per_unit: 1,
    rounding: 0,
};

/// Romanian Leu.
pub static RON: Currency = Currency {
    name: "Romanian Leu",
    code: "RON",
    numeric_code: 946,
    symbol: "lei",
    fraction_symbol: "ban",
    fractions_per_unit: 100,
    rounding: 2,
};

/// Bulgarian Lev.
pub static BGN: Currency = Currency {
    name: "Bulgarian Lev",
    code: "BGN",
    numeric_code: 975,
    symbol: "лв",
    fraction_symbol: "st",
    fractions_per_unit: 100,
    rounding: 2,
};

/// Croatian Kuna.
pub static HRK: Currency = Currency {
    name: "Croatian Kuna",
    code: "HRK",
    numeric_code: 191,
    symbol: "kn",
    fraction_symbol: "lp",
    fractions_per_unit: 100,
    rounding: 2,
};

/// Icelandic Krona.
pub static ISK: Currency = Currency {
    name: "Icelandic Krona",
    code: "ISK",
    numeric_code: 352,
    symbol: "kr",
    fraction_symbol: "a",
    fractions_per_unit: 100,
    rounding: 2,
};

/// Turkish Lira.
pub static TRY: Currency = Currency {
    name: "Turkish Lira",
    code: "TRY",
    numeric_code: 949,
    symbol: "₺",
    fraction_symbol: "kr",
    fractions_per_unit: 100,
    rounding: 2,
};

/// Russian Ruble.
pub static RUB: Currency = Currency {
    name: "Russian Ruble",
    code: "RUB",
    numeric_code: 643,
    symbol: "₽",
    fraction_symbol: "kop",
    fractions_per_unit: 100,
    rounding: 2,
};
