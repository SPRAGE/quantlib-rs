//! Asian currencies (translates `ql/currencies/asia.hpp`).

use crate::currency::Currency;

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

/// Chinese Yuan Renminbi.
pub static CNY: Currency = Currency {
    name: "Chinese Yuan",
    code: "CNY",
    numeric_code: 156,
    symbol: "¥",
    fraction_symbol: "f",
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

/// Singapore Dollar.
pub static SGD: Currency = Currency {
    name: "Singapore Dollar",
    code: "SGD",
    numeric_code: 702,
    symbol: "S$",
    fraction_symbol: "¢",
    fractions_per_unit: 100,
    rounding: 2,
};

/// South Korean Won.
pub static KRW: Currency = Currency {
    name: "South Korean Won",
    code: "KRW",
    numeric_code: 410,
    symbol: "₩",
    fraction_symbol: "j",
    fractions_per_unit: 1,
    rounding: 0,
};

/// Indian Rupee.
pub static INR: Currency = Currency {
    name: "Indian Rupee",
    code: "INR",
    numeric_code: 356,
    symbol: "₹",
    fraction_symbol: "p",
    fractions_per_unit: 100,
    rounding: 2,
};

/// New Taiwan Dollar.
pub static TWD: Currency = Currency {
    name: "New Taiwan Dollar",
    code: "TWD",
    numeric_code: 901,
    symbol: "NT$",
    fraction_symbol: "¢",
    fractions_per_unit: 100,
    rounding: 2,
};

/// Thai Baht.
pub static THB: Currency = Currency {
    name: "Thai Baht",
    code: "THB",
    numeric_code: 764,
    symbol: "฿",
    fraction_symbol: "st",
    fractions_per_unit: 100,
    rounding: 2,
};

/// Malaysian Ringgit.
pub static MYR: Currency = Currency {
    name: "Malaysian Ringgit",
    code: "MYR",
    numeric_code: 458,
    symbol: "RM",
    fraction_symbol: "sen",
    fractions_per_unit: 100,
    rounding: 2,
};

/// Indonesian Rupiah.
pub static IDR: Currency = Currency {
    name: "Indonesian Rupiah",
    code: "IDR",
    numeric_code: 360,
    symbol: "Rp",
    fraction_symbol: "sen",
    fractions_per_unit: 100,
    rounding: 2,
};

/// Philippine Peso.
pub static PHP: Currency = Currency {
    name: "Philippine Peso",
    code: "PHP",
    numeric_code: 608,
    symbol: "₱",
    fraction_symbol: "¢",
    fractions_per_unit: 100,
    rounding: 2,
};

/// Israeli New Shekel.
pub static ILS: Currency = Currency {
    name: "Israeli New Shekel",
    code: "ILS",
    numeric_code: 376,
    symbol: "₪",
    fraction_symbol: "ag",
    fractions_per_unit: 100,
    rounding: 2,
};

/// Saudi Riyal.
pub static SAR: Currency = Currency {
    name: "Saudi Riyal",
    code: "SAR",
    numeric_code: 682,
    symbol: "SR",
    fraction_symbol: "h",
    fractions_per_unit: 100,
    rounding: 2,
};

/// UAE Dirham.
pub static AED: Currency = Currency {
    name: "UAE Dirham",
    code: "AED",
    numeric_code: 784,
    symbol: "AED",
    fraction_symbol: "f",
    fractions_per_unit: 100,
    rounding: 2,
};
