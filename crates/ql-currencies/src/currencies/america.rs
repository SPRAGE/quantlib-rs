//! Americas currencies (translates `ql/currencies/america.hpp`).

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

/// Mexican Peso.
pub static MXN: Currency = Currency {
    name: "Mexican Peso",
    code: "MXN",
    numeric_code: 484,
    symbol: "Mex$",
    fraction_symbol: "¢",
    fractions_per_unit: 100,
    rounding: 2,
};

/// Argentine Peso.
pub static ARS: Currency = Currency {
    name: "Argentine Peso",
    code: "ARS",
    numeric_code: 32,
    symbol: "AR$",
    fraction_symbol: "¢",
    fractions_per_unit: 100,
    rounding: 2,
};

/// Chilean Peso.
pub static CLP: Currency = Currency {
    name: "Chilean Peso",
    code: "CLP",
    numeric_code: 152,
    symbol: "CLP$",
    fraction_symbol: "¢",
    fractions_per_unit: 1,
    rounding: 0,
};

/// Colombian Peso.
pub static COP: Currency = Currency {
    name: "Colombian Peso",
    code: "COP",
    numeric_code: 170,
    symbol: "COP$",
    fraction_symbol: "¢",
    fractions_per_unit: 100,
    rounding: 2,
};

/// Peruvian Sol.
pub static PEN: Currency = Currency {
    name: "Peruvian Sol",
    code: "PEN",
    numeric_code: 604,
    symbol: "S/.",
    fraction_symbol: "¢",
    fractions_per_unit: 100,
    rounding: 2,
};
