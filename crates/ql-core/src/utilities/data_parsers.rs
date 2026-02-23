//! Data parsing helpers (translates `ql/utilities/dataparsers.hpp`).
//!
//! Provides functions to parse dates and periods from string representations,
//! mirroring QuantLib's `DateParser` and `PeriodParser`.

/// Parse a period string like `"3M"`, `"1Y"`, `"30D"`, `"2W"`.
///
/// Returns `(length, unit_char)` on success.
///
/// # Errors
/// Returns `None` if the string cannot be parsed.
pub fn parse_period_string(s: &str) -> Option<(i32, char)> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let unit_char = s.chars().last()?;
    if !matches!(unit_char, 'D' | 'd' | 'W' | 'w' | 'M' | 'm' | 'Y' | 'y') {
        return None;
    }
    let num_str = &s[..s.len() - 1];
    let length: i32 = num_str.parse().ok()?;
    Some((length, unit_char.to_ascii_uppercase()))
}

/// Parse a date string in ISO 8601 format (`YYYY-MM-DD`).
///
/// Returns `(year, month, day)` on success.
pub fn parse_iso_date(s: &str) -> Option<(u16, u8, u8)> {
    let s = s.trim();
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let year: u16 = parts[0].parse().ok()?;
    let month: u8 = parts[1].parse().ok()?;
    let day: u8 = parts[2].parse().ok()?;
    Some((year, month, day))
}

/// Parse a date string in `DD/MM/YYYY` format.
///
/// Returns `(year, month, day)` on success.
pub fn parse_date_slash(s: &str) -> Option<(u16, u8, u8)> {
    let s = s.trim();
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() != 3 {
        return None;
    }
    let day: u8 = parts[0].parse().ok()?;
    let month: u8 = parts[1].parse().ok()?;
    let year: u16 = parts[2].parse().ok()?;
    Some((year, month, day))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_period() {
        assert_eq!(parse_period_string("3M"), Some((3, 'M')));
        assert_eq!(parse_period_string("1Y"), Some((1, 'Y')));
        assert_eq!(parse_period_string("30D"), Some((30, 'D')));
        assert_eq!(parse_period_string("2W"), Some((2, 'W')));
        assert_eq!(parse_period_string(""), None);
        assert_eq!(parse_period_string("abc"), None);
    }

    #[test]
    fn test_parse_iso_date() {
        assert_eq!(parse_iso_date("2023-06-15"), Some((2023, 6, 15)));
        assert_eq!(parse_iso_date("bad"), None);
    }

    #[test]
    fn test_parse_date_slash() {
        assert_eq!(parse_date_slash("15/06/2023"), Some((2023, 6, 15)));
    }
}
