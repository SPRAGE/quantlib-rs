//! United Kingdom calendars (translates `ql/time/calendars/unitedkingdom.hpp`).

use crate::calendar::Calendar;
use crate::date::Date;
use crate::weekday::Weekday;

/// United Kingdom — Settlement (London) calendar.
///
/// Holidays:
/// * New Year's Day (Jan 1; if Sun → Mon)
/// * Good Friday
/// * Easter Monday
/// * Early May Bank Holiday (1st Mon in May; or special dates)
/// * Spring Bank Holiday (last Mon in May; or special dates)
/// * Summer Bank Holiday (last Mon in Aug)
/// * Christmas Day (Dec 25; if Sun → Mon)
/// * Boxing Day (Dec 26; if Sun or Mon → Tue; if Sat → Mon)
#[derive(Debug, Clone, Copy, Default)]
pub struct UnitedKingdomSettlement;

impl Calendar for UnitedKingdomSettlement {
    fn name(&self) -> &str {
        "UK (Settlement)"
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

        // New Year's Day (possibly moved to Monday)
        if (d == 1 && m == 1)
            || ((d == 2 || d == 3) && m == 1 && w == Weekday::Monday)
        {
            return false;
        }
        // Good Friday
        if dd == em - 3 {
            return false;
        }
        // Easter Monday
        if dd == em {
            return false;
        }
        // Early May Bank Holiday (1st Monday in May)
        // Special exceptions: 2002 (Golden Jubilee), 2011 (Royal Wedding),
        // 2012 (Diamond Jubilee), 2020 (VE Day), 2023 (Coronation)
        if m == 5 {
            // Early May Bank Holiday (1st Monday in May)
            let is_first_monday = w == Weekday::Monday && d <= 7;
            // Special years where May Day bank holiday is moved to a different date
            let is_moved_year = matches!(y, 2020 | 2023);
            // Special substitute dates (moved May Day or extra Jubilee/Coronation holidays)
            let is_special_date = matches!(
                (y, m, d),
                (2002, 5, 27) |  // Golden Jubilee substitute (last Mon May)
                (2020, 5, 8)  |  // VE Day (May Day moved to May 8)
                (2023, 5, 8)     // Coronation (May Day moved to May 8)
            );
            // Normal first Monday in May is a bank holiday unless moved elsewhere
            if is_first_monday && !is_moved_year {
                return false;
            }
            // Special/substitute dates are bank holidays
            if is_special_date {
                return false;
            }
        }
        // Royal Wedding 2011 (April 29), Diamond Jubilee 2012 (June 4–5),
        // Golden Jubilee 2002 (June 3)
        if matches!((y, m, d), (2011, 4, 29) | (2002, 6, 3) | (2012, 6, 4) | (2012, 6, 5)) {
            return false;
        }
        // Spring Bank Holiday (last Monday in May) — in 2002 moved to Jun 3 above
        if m == 5 && w == Weekday::Monday && d >= 25 && y != 2002 {
            return false;
        }
        // Summer Bank Holiday (last Monday in August)
        if m == 8 && w == Weekday::Monday && d >= 25 {
            return false;
        }
        // Christmas (December 25) or substitute
        if (d == 25 && m == 12)
            || (d == 27 && m == 12 && matches!(w, Weekday::Monday | Weekday::Tuesday))
        {
            return false;
        }
        // Boxing Day (December 26) or substitute
        if (d == 26 && m == 12)
            || (d == 28 && m == 12 && matches!(w, Weekday::Monday | Weekday::Tuesday))
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
    fn new_years_day_2023() {
        let cal = UnitedKingdomSettlement;
        // Jan 2, 2023 (Monday, as Jan 1 was Sunday)
        assert!(!cal.is_business_day(date(2023, 1, 2)));
    }

    #[test]
    fn good_friday_2023() {
        let cal = UnitedKingdomSettlement;
        assert!(!cal.is_business_day(date(2023, 4, 7)));
    }

    #[test]
    fn normal_business_day() {
        let cal = UnitedKingdomSettlement;
        assert!(cal.is_business_day(date(2023, 3, 15)));
    }
}
