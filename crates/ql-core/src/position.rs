//! Position type (translates `ql/position.hpp`).

/// Long or short position.
///
/// Corresponds to `QuantLib::Position::Type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Position {
    /// Long position (buyer).
    Long,
    /// Short position (seller).
    Short,
}

impl Position {
    /// Return the sign (+1 for Long, -1 for Short).
    pub fn sign(&self) -> f64 {
        match self {
            Position::Long => 1.0,
            Position::Short => -1.0,
        }
    }

    /// Return the integer sign (+1 for Long, -1 for Short).
    pub fn integer_sign(&self) -> i32 {
        match self {
            Position::Long => 1,
            Position::Short => -1,
        }
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Position::Long => write!(f, "Long"),
            Position::Short => write!(f, "Short"),
        }
    }
}
