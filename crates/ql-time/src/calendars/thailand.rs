//! Thailand (SET) calendar (translates `ql/time/calendars/thailand.hpp`).
//!
//! Holidays observed by financial institutions are regulated by the Bank of
//! Thailand. If a holiday falls on a weekend the government announces a
//! replacement day (usually the following Monday). Some years have additional
//! one-off holidays.
//!
//! Fixed holidays:
//! * New Year's Day (Jan 1)
//! * Chakri Memorial Day (Apr 6)
//! * Songkran Festival (Apr 13-15, cancelled in 2020)
//! * Labour Day (May 1)
//! * Coronation Day (May 4, from 2019)
//! * H.M. Queen Suthida's Birthday (Jun 3, from 2019)
//! * H.M. King's Birthday (Jul 28, from 2017)
//! * H.M. Queen Sirikit's Birthday / Mother's Day (Aug 12)
//! * H.M. King Bhumibol Memorial Day (Oct 13, from 2017)
//! * Chulalongkorn Day (Oct 23)
//! * H.M. King Bhumibol's Birthday / Father's Day (Dec 5)
//! * Constitution Day (Dec 10)
//! * New Year's Eve (Dec 31)
//!
//! Variable holidays (Buddhist lunar calendar — data available 2000–2025):
//! * Makha Bucha Day
//! * Wisakha Bucha Day
//! * Asarnha Bucha Day (from 2007) / Buddhist Lent Day (until 2006)

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// Thailand (SET) calendar.
///
/// Corresponds to `QuantLib::Thailand`.
#[derive(Debug, Clone, Copy, Default)]
pub struct Thailand;

impl Calendar for Thailand {
    fn name(&self) -> &str {
        "Thailand stock exchange"
    }

    fn is_business_day(&self, date: Date) -> bool {
        let w = date.weekday();
        if matches!(w, Weekday::Saturday | Weekday::Sunday) {
            return false;
        }
        let d = date.day_of_month() as i32;
        let m = date.month();
        let y = date.year() as i32;
        let mon = w == Weekday::Monday;

        // --- Fixed holidays (with Monday substitution rules) ---

        // New Year's Day (Jan 1, with Mon sub when Jan 1 is a weekend)
        if (d == 1 || (d == 3 && mon)) && m == 1 {
            return false;
        }
        // Chakri Memorial Day (Apr 6)
        if (d == 6 || ((d == 7 || d == 8) && mon)) && m == 4 {
            return false;
        }
        // Songkran Festival (Apr 13-15, cancelled in 2020)
        if (d == 13 || d == 14 || d == 15) && m == 4 && y != 2020 {
            return false;
        }
        // Substitution Songkran (Mon/Tue following Apr 15, not 2020)
        if d == 16 && (w == Weekday::Monday || w == Weekday::Tuesday) && m == 4 && y != 2020 {
            return false;
        }
        // Labour Day (May 1)
        if (d == 1 || ((d == 2 || d == 3) && mon)) && m == 5 {
            return false;
        }
        // Coronation Day (May 4, from 2019)
        if (d == 4 || ((d == 5 || d == 6) && mon)) && m == 5 && y >= 2019 {
            return false;
        }
        // H.M. Queen Suthida's Birthday (Jun 3, from 2019)
        if (d == 3 || ((d == 4 || d == 5) && mon)) && m == 6 && y >= 2019 {
            return false;
        }
        // H.M. King's Birthday (Jul 28, from 2017)
        if (d == 28 || ((d == 29 || d == 30) && mon)) && m == 7 && y >= 2017 {
            return false;
        }
        // H.M. Queen Sirikit's Birthday / Mother's Day (Aug 12)
        if (d == 12 || ((d == 13 || d == 14) && mon)) && m == 8 {
            return false;
        }
        // H.M. King Bhumibol Memorial Day (Oct 13, from 2017)
        if (d == 13 || ((d == 14 || d == 15) && mon)) && m == 10 && y >= 2017 {
            return false;
        }
        // Chulalongkorn Day (Oct 23, moved in 2021 — see year-specific below)
        if (d == 23 || ((d == 24 || d == 25) && mon)) && m == 10 && y != 2021 {
            return false;
        }
        // H.M. King Bhumibol's Birthday / National Day / Father's Day (Dec 5)
        if (d == 5 || ((d == 6 || d == 7) && mon)) && m == 12 {
            return false;
        }
        // Constitution Day (Dec 10)
        if (d == 10 || ((d == 11 || d == 12) && mon)) && m == 12 {
            return false;
        }
        // New Year's Eve (Dec 31), with Mon sub on Jan 2 (but not 2024)
        if (d == 31 && m == 12) || (d == 2 && mon && m == 1 && y != 2024) {
            return false;
        }

        // --- Year-specific variable holidays (Makha Bucha, Wisakha Bucha, etc.) ---

        if y == 2000
            && ((d == 21 && m == 2) // Makha Bucha Day (Substitution)
             || (d == 5 && m == 5)  // Coronation Day
             || (d == 17 && m == 5) // Wisakha Bucha Day
             || (d == 17 && m == 7) // Buddhist Lent Day
             || (d == 23 && m == 10)) // Chulalongkorn Day
        {
            return false;
        }

        if y == 2001
            && ((d == 8 && m == 2)  // Makha Bucha Day
             || (d == 7 && m == 5)  // Wisakha Bucha Day
             || (d == 8 && m == 5)  // Coronation Day (Substitution)
             || (d == 6 && m == 7)  // Buddhist Lent Day
             || (d == 23 && m == 10)) // Chulalongkorn Day
        {
            return false;
        }

        // 2002, 2003, and 2004 are missing in QuantLib C++

        if y == 2005
            && ((d == 23 && m == 2)  // Makha Bucha Day
             || (d == 5 && m == 5)   // Coronation Day
             || (d == 23 && m == 5)  // Wisakha Bucha Day (Sub for Sun 22 May)
             || (d == 1 && m == 7)   // Mid Year Closing Day
             || (d == 22 && m == 7)  // Buddhist Lent Day
             || (d == 24 && m == 10)) // Chulalongkorn Day (Sub for Sun 23 Oct)
        {
            return false;
        }

        if y == 2006
            && ((d == 13 && m == 2)  // Makha Bucha Day
             || (d == 19 && m == 4)  // Special Holiday
             || (d == 5 && m == 5)   // Coronation Day
             || (d == 12 && m == 5)  // Wisakha Bucha Day
             || (d == 12 && m == 6)  // Special Holiday (60th Anniversary)
             || (d == 13 && m == 6)  // Special Holiday (60th Anniversary)
             || (d == 11 && m == 7)  // Buddhist Lent Day
             || (d == 23 && m == 10)) // Chulalongkorn Day
        {
            return false;
        }

        if y == 2007
            && ((d == 5 && m == 3)   // Makha Bucha Day (Sub for Sat 3 Mar)
             || (d == 7 && m == 5)   // Coronation Day (Sub for Sat 5 May)
             || (d == 31 && m == 5)  // Wisakha Bucha Day
             || (d == 30 && m == 7)  // Asarnha Bucha Day (Sub for Sun 29 Jul)
             || (d == 23 && m == 10) // Chulalongkorn Day
             || (d == 24 && m == 12)) // Special Holiday
        {
            return false;
        }

        if y == 2008
            && ((d == 21 && m == 2)  // Makha Bucha Day
             || (d == 5 && m == 5)   // Coronation Day
             || (d == 19 && m == 5)  // Wisakha Bucha Day
             || (d == 1 && m == 7)   // Mid Year Closing Day
             || (d == 17 && m == 7)  // Asarnha Bucha Day
             || (d == 23 && m == 10)) // Chulalongkorn Day
        {
            return false;
        }

        if y == 2009
            && ((d == 2 && m == 1)   // Special Holiday
             || (d == 9 && m == 2)   // Makha Bucha Day
             || (d == 5 && m == 5)   // Coronation Day
             || (d == 8 && m == 5)   // Wisakha Bucha Day
             || (d == 1 && m == 7)   // Mid Year Closing Day
             || (d == 6 && m == 7)   // Special Holiday
             || (d == 7 && m == 7)   // Asarnha Bucha Day
             || (d == 23 && m == 10)) // Chulalongkorn Day
        {
            return false;
        }

        if y == 2010
            && ((d == 1 && m == 3)   // Sub for Makha Bucha Day (Sun 28 Feb)
             || (d == 5 && m == 5)   // Coronation Day
             || (d == 20 && m == 5)  // Special Holiday
             || (d == 21 && m == 5)  // Special Holiday
             || (d == 28 && m == 5)  // Wisakha Bucha Day
             || (d == 1 && m == 7)   // Mid Year Closing Day
             || (d == 26 && m == 7)  // Asarnha Bucha Day
             || (d == 13 && m == 8)  // Special Holiday
             || (d == 25 && m == 10)) // Sub for Chulalongkorn Day (Sat 23 Oct)
        {
            return false;
        }

        if y == 2011
            && ((d == 18 && m == 2)  // Makha Bucha Day
             || (d == 5 && m == 5)   // Coronation Day
             || (d == 16 && m == 5)  // Special Holiday
             || (d == 17 && m == 5)  // Wisakha Bucha Day
             || (d == 1 && m == 7)   // Mid Year Closing Day
             || (d == 15 && m == 7)  // Asarnha Bucha Day
             || (d == 24 && m == 10)) // Sub for Chulalongkorn Day (Sun 23 Oct)
        {
            return false;
        }

        if y == 2012
            && ((d == 3 && m == 1)   // Special Holiday
             || (d == 7 && m == 3)   // Makha Bucha Day
             || (d == 9 && m == 4)   // Special Holiday
             || (d == 7 && m == 5)   // Sub for Coronation Day (Sat 5 May)
             || (d == 4 && m == 6)   // Wisakha Bucha Day
             || (d == 2 && m == 8)   // Asarnha Bucha Day
             || (d == 23 && m == 10)) // Chulalongkorn Day
        {
            return false;
        }

        if y == 2013
            && ((d == 25 && m == 2)  // Makha Bucha Day
             || (d == 6 && m == 5)   // Sub for Coronation Day (Sun 5 May)
             || (d == 24 && m == 5)  // Wisakha Bucha Day
             || (d == 1 && m == 7)   // Mid Year Closing Day
             || (d == 22 && m == 7)  // Asarnha Bucha Day
             || (d == 23 && m == 10) // Chulalongkorn Day
             || (d == 30 && m == 12)) // Special Holiday
        {
            return false;
        }

        if y == 2014
            && ((d == 14 && m == 2)  // Makha Bucha Day
             || (d == 5 && m == 5)   // Coronation Day
             || (d == 13 && m == 5)  // Wisakha Bucha Day
             || (d == 1 && m == 7)   // Mid Year Closing Day
             || (d == 11 && m == 7)  // Asarnha Bucha Day
             || (d == 11 && m == 8)  // Special Holiday
             || (d == 23 && m == 10)) // Chulalongkorn Day
        {
            return false;
        }

        if y == 2015
            && ((d == 2 && m == 1)   // Special Holiday
             || (d == 4 && m == 3)   // Makha Bucha Day
             || (d == 4 && m == 5)   // Special Holiday
             || (d == 5 && m == 5)   // Coronation Day
             || (d == 1 && m == 6)   // Wisakha Bucha Day
             || (d == 1 && m == 7)   // Mid Year Closing Day
             || (d == 30 && m == 7)  // Asarnha Bucha Day
             || (d == 23 && m == 10)) // Chulalongkorn Day
        {
            return false;
        }

        if y == 2016
            && ((d == 22 && m == 2)  // Makha Bucha Day
             || (d == 5 && m == 5)   // Coronation Day
             || (d == 6 && m == 5)   // Special Holiday
             || (d == 20 && m == 5)  // Wisakha Bucha Day
             || (d == 1 && m == 7)   // Mid Year Closing Day
             || (d == 18 && m == 7)  // Special Holiday
             || (d == 19 && m == 7)  // Asarnha Bucha Day
             || (d == 24 && m == 10)) // Sub for Chulalongkorn Day (Sun 23 Oct)
        {
            return false;
        }

        if y == 2017
            && ((d == 13 && m == 2)  // Makha Bucha Day
             || (d == 10 && m == 5)  // Wisakha Bucha Day
             || (d == 10 && m == 7)  // Asarnha Bucha Day
             || (d == 23 && m == 10) // Chulalongkorn Day
             || (d == 26 && m == 10)) // Special Holiday
        {
            return false;
        }

        if y == 2018
            && ((d == 1 && m == 3)   // Makha Bucha Day
             || (d == 29 && m == 5)  // Wisakha Bucha Day
             || (d == 27 && m == 7)  // Asarnha Bucha Day
             || (d == 23 && m == 10)) // Chulalongkorn Day
        {
            return false;
        }

        if y == 2019
            && ((d == 19 && m == 2)  // Makha Bucha Day
             || (d == 6 && m == 5)   // Special Holiday
             || (d == 20 && m == 5)  // Wisakha Bucha Day
             || (d == 16 && m == 7)) // Asarnha Bucha Day
        {
            return false;
        }

        if y == 2020
            && ((d == 10 && m == 2)  // Makha Bucha Day
             || (d == 6 && m == 5)   // Wisakha Bucha Day
             || (d == 6 && m == 7)   // Asarnha Bucha Day
             || (d == 27 && m == 7)  // Substitution for Songkran Festival
             || (d == 4 && m == 9)   // Substitution for Songkran Festival
             || (d == 7 && m == 9)   // Substitution for Songkran Festival
             || (d == 11 && m == 12)) // Special Holiday
        {
            return false;
        }

        if y == 2021
            && ((d == 12 && m == 2)  // Special Holiday
             || (d == 26 && m == 2)  // Makha Bucha Day
             || (d == 26 && m == 5)  // Wisakha Bucha Day
             || (d == 26 && m == 7)  // Sub for Asarnha Bucha Day (Sat 24 Jul)
             || (d == 24 && m == 9)  // Special Holiday
             || (d == 22 && m == 10)) // Sub for Chulalongkorn Day
        {
            return false;
        }

        if y == 2022
            && ((d == 16 && m == 2)  // Makha Bucha Day
             || (d == 16 && m == 5)  // Sub for Wisakha Bucha Day (Sun 15 May)
             || (d == 13 && m == 7)  // Asarnha Bucha Day
             || (d == 29 && m == 7)  // Additional special holiday
             || (d == 14 && m == 10) // Additional special holiday
             || (d == 24 && m == 10)) // Sub for Chulalongkorn Day (Sun 23 Oct)
        {
            return false;
        }

        if y == 2023
            && ((d == 6 && m == 3)   // Makha Bucha Day
             || (d == 5 && m == 5)   // Additional special holiday
             || (d == 5 && m == 6)   // Sub for Queen's Birthday & Wisakha Bucha
             || (d == 1 && m == 8)   // Asarnha Bucha Day
             || (d == 23 && m == 10) // Chulalongkorn Day
             || (d == 29 && m == 12)) // Sub for New Year's Eve (Sun 31 Dec)
        {
            return false;
        }

        if y == 2024
            && ((d == 26 && m == 2)  // Sub for Makha Bucha Day (Sat 24 Feb)
             || (d == 8 && m == 4)   // Sub for Chakri Memorial Day (Sat 6 Apr)
             || (d == 12 && m == 4)  // Additional Songkran holiday
             || (d == 6 && m == 5)   // Sub for Coronation Day (Sat 4 May)
             || (d == 22 && m == 5)  // Wisakha Bucha Day
             || (d == 22 && m == 7)  // Sub for Asarnha Bucha Day (Sat 20 Jul)
             || (d == 23 && m == 10)) // Chulalongkorn Day
        {
            return false;
        }

        if y == 2025
            && ((d == 12 && m == 2)  // Sub for Makha Bucha Day
             || (d == 7 && m == 4)   // Sub for Chakri Memorial Day (Sun 6 Apr)
             || (d == 5 && m == 5)   // Sub for Coronation Day (Sun 4 May)
             || (d == 12 && m == 5)  // Wisakha Bucha Day
             || (d == 10 && m == 7)  // Asarnha Bucha Day
             || (d == 23 && m == 10)) // Chulalongkorn Day
        {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: u16, m: u8, d: u8) -> Date {
        Date::from_ymd(y, m, d).unwrap()
    }

    #[test]
    fn name() {
        assert_eq!(Thailand.name(), "Thailand stock exchange");
    }

    #[test]
    fn weekends_are_holidays() {
        // Saturday 2023-01-07
        assert!(!Thailand.is_business_day(date(2023, 1, 7)));
        // Sunday 2023-01-08
        assert!(!Thailand.is_business_day(date(2023, 1, 8)));
    }

    #[test]
    fn new_years_day() {
        assert!(!Thailand.is_business_day(date(2023, 1, 1)));
        // 2024 Jan 1 is Monday — holiday
        assert!(!Thailand.is_business_day(date(2024, 1, 1)));
    }

    #[test]
    fn chakri_memorial_day() {
        // Apr 6 2023 is Thursday
        assert!(!Thailand.is_business_day(date(2023, 4, 6)));
    }

    #[test]
    fn songkran_cancelled_2020() {
        // Apr 13-15 2020 should be business days (cancelled due to COVID-19)
        assert!(Thailand.is_business_day(date(2020, 4, 13)));
        assert!(Thailand.is_business_day(date(2020, 4, 14)));
        assert!(Thailand.is_business_day(date(2020, 4, 15)));
    }

    #[test]
    fn songkran_normal_year() {
        assert!(!Thailand.is_business_day(date(2023, 4, 13)));
        assert!(!Thailand.is_business_day(date(2023, 4, 14)));
        assert!(!Thailand.is_business_day(date(2023, 4, 15)));
    }

    #[test]
    fn labour_day() {
        assert!(!Thailand.is_business_day(date(2023, 5, 1)));
    }

    #[test]
    fn queen_sirikit_birthday() {
        assert!(!Thailand.is_business_day(date(2023, 8, 12)));
    }

    #[test]
    fn constitution_day() {
        assert!(!Thailand.is_business_day(date(2023, 12, 10)));
    }

    #[test]
    fn makha_bucha_2023() {
        // Specific variable holiday for 2023
        assert!(!Thailand.is_business_day(date(2023, 3, 6)));
    }

    #[test]
    fn regular_business_day() {
        // 2023-01-09 is a Monday with no holiday
        assert!(Thailand.is_business_day(date(2023, 1, 9)));
    }

    #[test]
    fn king_birthday_from_2017() {
        // Jul 28 2023 is Friday
        assert!(!Thailand.is_business_day(date(2023, 7, 28)));
        // Jul 28 2018 is Saturday → Mon Jul 30 is sub
        assert!(!Thailand.is_business_day(date(2018, 7, 30)));
    }

    #[test]
    fn coronation_day_from_2019() {
        // May 4 2023 is Thursday
        assert!(!Thailand.is_business_day(date(2023, 5, 4)));
        // Before 2019 — May 4 2018 is Friday — should be a normal business day
        assert!(Thailand.is_business_day(date(2018, 5, 4)));
    }
}
