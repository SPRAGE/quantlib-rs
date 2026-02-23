//! `Frequency` — how often events recur (translates `ql/time/frequency.hpp`).

/// Event / payment frequency.
///
/// Corresponds to `QuantLib::Frequency`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Frequency {
    /// No events — used as a sentinel.
    NoFrequency = -1,
    /// Once (maturity only).
    Once = 0,
    /// Annual (once per year).
    Annual = 1,
    /// Semi-annual (twice per year).
    Semiannual = 2,
    /// Every third year.
    EveryFourthMonth = 3,
    /// Quarterly (four times per year).
    Quarterly = 4,
    /// Bi-monthly (six times per year).
    Bimonthly = 6,
    /// Monthly (twelve times per year).
    Monthly = 12,
    /// Every fourth week (thirteen times per year).
    EveryFourthWeek = 13,
    /// Bi-weekly (twenty-six times per year).
    Biweekly = 26,
    /// Weekly (fifty-two times per year).
    Weekly = 52,
    /// Daily.
    Daily = 365,
    /// Other / custom frequency.
    OtherFrequency = 999,
}

impl Frequency {
    /// Number of periods per year.  Returns `None` for `NoFrequency` and
    /// `OtherFrequency`.
    pub fn periods_per_year(&self) -> Option<u32> {
        match self {
            Frequency::NoFrequency | Frequency::OtherFrequency => None,
            Frequency::Once => Some(0),
            Frequency::Annual => Some(1),
            Frequency::Semiannual => Some(2),
            Frequency::EveryFourthMonth => Some(3),
            Frequency::Quarterly => Some(4),
            Frequency::Bimonthly => Some(6),
            Frequency::Monthly => Some(12),
            Frequency::EveryFourthWeek => Some(13),
            Frequency::Biweekly => Some(26),
            Frequency::Weekly => Some(52),
            Frequency::Daily => Some(365),
        }
    }
}

impl std::fmt::Display for Frequency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Frequency::NoFrequency => "No-Frequency",
            Frequency::Once => "Once",
            Frequency::Annual => "Annual",
            Frequency::Semiannual => "Semiannual",
            Frequency::EveryFourthMonth => "Every-Fourth-Month",
            Frequency::Quarterly => "Quarterly",
            Frequency::Bimonthly => "Bimonthly",
            Frequency::Monthly => "Monthly",
            Frequency::EveryFourthWeek => "Every-Fourth-Week",
            Frequency::Biweekly => "Biweekly",
            Frequency::Weekly => "Weekly",
            Frequency::Daily => "Daily",
            Frequency::OtherFrequency => "Other-Frequency",
        };
        write!(f, "{s}")
    }
}
