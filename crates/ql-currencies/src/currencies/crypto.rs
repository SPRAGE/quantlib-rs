//! Cryptocurrency definitions (translates `ql/currencies/crypto.hpp`).

use crate::currency::Currency;

/// Bitcoin.
pub static BTC: Currency = Currency {
    name: "Bitcoin",
    code: "BTC",
    numeric_code: 0,
    symbol: "₿",
    fraction_symbol: "sat",
    fractions_per_unit: 100_000_000,
    rounding: 8,
};

/// Ethereum.
pub static ETH: Currency = Currency {
    name: "Ethereum",
    code: "ETH",
    numeric_code: 0,
    symbol: "Ξ",
    fraction_symbol: "wei",
    fractions_per_unit: 1_000_000_000,
    rounding: 8,
};
