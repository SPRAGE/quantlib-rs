//! United States calendars (translates `ql/time/calendars/unitedstates.hpp`).

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// United States — Settlement (federal holidays) calendar.
///
/// Holidays:
/// * New Year's Day (Jan 1; if Sun → Mon; if Sat → Fri)
/// * Martin Luther King Jr. Day (3rd Mon in Jan, from 1983)
/// * Presidents' Day (3rd Mon in Feb)
/// * Memorial Day (last Mon in May)
/// * Juneteenth (Jun 19, from 2022; if Sun → Mon; if Sat → Fri)
/// * Independence Day (Jul 4; if Sun → Mon; if Sat → Fri)
/// * Labor Day (1st Mon in Sep)
/// * Columbus Day (2nd Mon in Oct)
/// * Veterans' Day (Nov 11; if Sun → Mon; if Sat → Fri)
/// * Thanksgiving Day (4th Thu in Nov)
/// * Christmas Day (Dec 25; if Sun → Mon; if Sat → Fri)
#[derive(Debug, Clone, Copy, Default)]
pub struct UnitedStatesSettlement;

impl Calendar for UnitedStatesSettlement {
    fn name(&self) -> &str {
        "US (Settlement)"
    }

    fn is_business_day(&self, date: Date) -> bool {
        let w = date.weekday();
        if matches!(w, Weekday::Saturday | Weekday::Sunday) {
            return false;
        }
        let y = date.year();
        let m = date.month();
        let d = date.day_of_month();

        if is_us_settlement_holiday(y, m, d, w) {
            return false;
        }
        true
    }
}

fn is_us_settlement_holiday(y: u16, m: u8, d: u8, w: Weekday) -> bool {
    // New Year's Day (possibly moved to Monday if on Sunday)
    if (d == 1 && m == 1)
        || (d == 2 && m == 1 && w == Weekday::Monday)  // Jan 1 on Sunday → Jan 2
    {
        return true;
    }
    // New Year's Day observed on preceding Friday when Jan 1 is Saturday
    if d == 31 && m == 12 && w == Weekday::Friday {
        return true;
    }
    // MLK Day (3rd Monday of January, since 1983)
    if y >= 1983 && m == 1 && w == Weekday::Monday && (15..=21).contains(&d) {
        return true;
    }
    // Presidents' Day (3rd Monday of February)
    if m == 2 && w == Weekday::Monday && (15..=21).contains(&d) {
        return true;
    }
    // Memorial Day (last Monday of May)
    if m == 5 && w == Weekday::Monday && d >= 25 {
        return true;
    }
    // Juneteenth (June 19, from 2022)
    if y >= 2022
        && m == 6
        && ((d == 19 && !matches!(w, Weekday::Saturday | Weekday::Sunday))
            || (d == 20 && w == Weekday::Monday)  // Jun 19 on Sunday
            || (d == 18 && w == Weekday::Friday))  // Jun 19 on Saturday
    {
        return true;
    }
    // Independence Day (July 4)
    if (d == 4 && m == 7)
        || (d == 5 && m == 7 && w == Weekday::Monday)  // Jul 4 on Sunday
        || (d == 3 && m == 7 && w == Weekday::Friday)   // Jul 4 on Saturday
    {
        return true;
    }
    // Labor Day (1st Monday of September)
    if m == 9 && w == Weekday::Monday && d <= 7 {
        return true;
    }
    // Columbus Day (2nd Monday of October)
    if m == 10 && w == Weekday::Monday && (8..=14).contains(&d) {
        return true;
    }
    // Veterans' Day (November 11)
    if (d == 11 && m == 11)
        || (d == 12 && m == 11 && w == Weekday::Monday)  // Nov 11 on Sunday
        || (d == 10 && m == 11 && w == Weekday::Friday)   // Nov 11 on Saturday
    {
        return true;
    }
    // Thanksgiving (4th Thursday of November)
    if m == 11 && w == Weekday::Thursday && (22..=28).contains(&d) {
        return true;
    }
    // Christmas (December 25)
    if (d == 25 && m == 12)
        || (d == 26 && m == 12 && w == Weekday::Monday)  // Dec 25 on Sunday
        || (d == 24 && m == 12 && w == Weekday::Friday)   // Dec 25 on Saturday
    {
        return true;
    }
    false
}

/// United States — NYSE calendar.
#[derive(Debug, Clone, Copy, Default)]
pub struct UnitedStatesNyse;

impl Calendar for UnitedStatesNyse {
    fn name(&self) -> &str {
        "US (NYSE)"
    }

    fn is_business_day(&self, date: Date) -> bool {
        let w = date.weekday();
        if matches!(w, Weekday::Saturday | Weekday::Sunday) {
            return false;
        }
        let y = date.year();
        let m = date.month();
        let d = date.day_of_month();
        let dd = date.day_of_year();
        let em = super::target::easter_monday_pub(y);

        // NYSE does NOT observe Columbus Day or Veterans' Day
        if is_nyse_holiday(y, m, d, w, dd, em) {
            return false;
        }
        true
    }
}

fn is_nyse_holiday(y: u16, m: u8, d: u8, w: Weekday, dd: u16, em: u16) -> bool {
    // New Year's Day (possibly moved to Monday if on Sunday)
    if (d == 1 && m == 1)
        || (d == 2 && m == 1 && w == Weekday::Monday)
    {
        return true;
    }
    // MLK Day (3rd Monday of January, since 1998 for NYSE)
    if y >= 1998 && m == 1 && w == Weekday::Monday && (15..=21).contains(&d) {
        return true;
    }
    // Presidents' Day / Washington's Birthday (3rd Monday of February)
    if m == 2 && w == Weekday::Monday && (15..=21).contains(&d) {
        return true;
    }
    // Good Friday
    if dd == em - 3 {
        return true;
    }
    // Memorial Day (last Monday of May)
    if m == 5 && w == Weekday::Monday && d >= 25 {
        return true;
    }
    // Juneteenth (June 19, from 2022)
    if y >= 2022
        && m == 6
        && ((d == 19 && !matches!(w, Weekday::Saturday | Weekday::Sunday))
            || (d == 20 && w == Weekday::Monday)
            || (d == 18 && w == Weekday::Friday))
    {
        return true;
    }
    // Independence Day (July 4)
    if (d == 4 && m == 7)
        || (d == 5 && m == 7 && w == Weekday::Monday)
        || (d == 3 && m == 7 && w == Weekday::Friday)
    {
        return true;
    }
    // Labor Day (1st Monday of September)
    if m == 9 && w == Weekday::Monday && d <= 7 {
        return true;
    }
    // Thanksgiving (4th Thursday of November)
    if m == 11 && w == Weekday::Thursday && (22..=28).contains(&d) {
        return true;
    }
    // Christmas (December 25)
    if (d == 25 && m == 12)
        || (d == 26 && m == 12 && w == Weekday::Monday)
        || (d == 24 && m == 12 && w == Weekday::Friday)
    {
        return true;
    }
    // Historical closings
    if is_nyse_historical_closing(y, m, d) {
        return true;
    }
    false
}

fn is_nyse_historical_closing(y: u16, m: u8, d: u8) -> bool {
    matches!(
        (y, m, d),
        // Hurricane Sandy (2012)
        (2012, 10, 29) | (2012, 10, 30)
        // President Reagan's funeral (2004)
        | (2004, 6, 11)
        // September 11, 2001
        | (2001, 9, 11) | (2001, 9, 12) | (2001, 9, 13) | (2001, 9, 14)
        // President Nixon's funeral (1994)
        | (1994, 4, 27)
        // Hurricane Gloria (1985)
        | (1985, 9, 27)
        // 1977 Blackout
        | (1977, 7, 14)
        // President Johnson's funeral (1973)
        | (1973, 1, 25)
        // President Truman's funeral (1972)
        | (1972, 12, 28)
        // Lunar exploration national day of participation (1969)
        | (1969, 7, 21)
        // President Eisenhower's funeral (1969)
        | (1969, 3, 31)
        // Heavy snow (1969)
        | (1969, 2, 10)
        // Day after Independence Day (1968)
        | (1968, 7, 5)
        // Mourning for MLK (1968)
        | (1968, 4, 9)
        // President Kennedy's funeral (1963)
        | (1963, 11, 25)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: u16, m: u8, d: u8) -> Date {
        Date::from_ymd(y, m, d).unwrap()
    }

    #[test]
    fn independence_day_2023() {
        let cal = UnitedStatesSettlement;
        // July 4, 2023 is a Tuesday
        assert!(!cal.is_business_day(date(2023, 7, 4)));
    }

    #[test]
    fn thanksgiving_2023() {
        let cal = UnitedStatesSettlement;
        // 4th Thursday of November 2023 = Nov 23
        assert!(!cal.is_business_day(date(2023, 11, 23)));
    }

    #[test]
    fn normal_day() {
        let cal = UnitedStatesSettlement;
        assert!(cal.is_business_day(date(2023, 6, 15)));
    }
}
