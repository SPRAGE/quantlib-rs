//! Tests ported from QuantLib `test-suite/calendars.cpp`.
//!
//! These integration tests exercise the `Calendar` trait, individual calendar
//! implementations, `JointCalendar`, and `BespokeCalendar`.

use ql_time::calendar::Calendar;
use ql_time::calendars::brazil::Brazil;
use ql_time::calendars::denmark::Denmark;
use ql_time::calendars::germany::Germany;
use ql_time::calendars::japan::Japan;
use ql_time::calendars::target::Target;
use ql_time::calendars::united_kingdom::UnitedKingdomSettlement;
use ql_time::calendars::united_states::{UnitedStatesNyse, UnitedStatesSettlement};
use ql_time::{BespokeCalendar, BusinessDayConvention, Date, JointCalendar, JointCalendarRule};

fn date(y: u16, m: u8, d: u8) -> Date {
    Date::from_ymd(y, m, d).unwrap()
}

/// Collect all non-weekend holidays in the inclusive range `[from, to]`.
///
/// Mirrors C++ `Calendar::holidayList(from, to, false)` — weekends excluded.
fn holiday_list(cal: &dyn Calendar, from: Date, to: Date) -> Vec<Date> {
    let mut holidays = Vec::new();
    let mut d = from;
    while d <= to {
        if cal.is_holiday(d) && !cal.is_weekend(d) {
            holidays.push(d);
        }
        d += 1;
    }
    holidays
}

/// Collect all holidays including weekends in the range `[from, to]`.
#[allow(dead_code)]
fn holiday_list_with_weekends(cal: &dyn Calendar, from: Date, to: Date) -> Vec<Date> {
    let mut holidays = Vec::new();
    let mut d = from;
    while d <= to {
        if cal.is_holiday(d) {
            holidays.push(d);
        }
        d += 1;
    }
    holidays
}

/// Assert that every date in `expected` is a holiday, and every holiday in the
/// range is in `expected`.  Panics on mismatches, similar to the C++ helper.
fn check_holidays(cal: &dyn Calendar, from: Date, to: Date, expected: &[Date]) {
    let calculated = holiday_list(cal, from, to);
    let calc_set: std::collections::HashSet<_> = calculated.iter().copied().collect();
    let exp_set: std::collections::HashSet<_> = expected.iter().copied().collect();

    for &d in &calculated {
        assert!(
            exp_set.contains(&d),
            "{}: {} calculated as holiday but not expected ({})",
            cal.name(),
            d,
            d.weekday()
        );
    }
    for &d in expected {
        assert!(
            calc_set.contains(&d),
            "{}: {} expected as holiday but not found ({})",
            cal.name(),
            d,
            d.weekday()
        );
    }
}

// ─── TARGET holidays ──────────────────────────────────────────────────────────

#[test]
fn test_target_holidays() {
    let expected: Vec<Date> = vec![
        date(1999, 1, 1),
        date(1999, 12, 31),
        date(2000, 4, 21),
        date(2000, 4, 24),
        date(2000, 5, 1),
        date(2000, 12, 25),
        date(2000, 12, 26),
        date(2001, 1, 1),
        date(2001, 4, 13),
        date(2001, 4, 16),
        date(2001, 5, 1),
        date(2001, 12, 25),
        date(2001, 12, 26),
        date(2001, 12, 31),
        date(2002, 1, 1),
        date(2002, 3, 29),
        date(2002, 4, 1),
        date(2002, 5, 1),
        date(2002, 12, 25),
        date(2002, 12, 26),
        date(2003, 1, 1),
        date(2003, 4, 18),
        date(2003, 4, 21),
        date(2003, 5, 1),
        date(2003, 12, 25),
        date(2003, 12, 26),
        date(2004, 1, 1),
        date(2004, 4, 9),
        date(2004, 4, 12),
        date(2005, 3, 25),
        date(2005, 3, 28),
        date(2005, 12, 26),
        date(2006, 4, 14),
        date(2006, 4, 17),
        date(2006, 5, 1),
        date(2006, 12, 25),
        date(2006, 12, 26),
    ];

    let cal = Target;
    check_holidays(&cal, date(1999, 1, 1), date(2006, 12, 31), &expected);
}

// ─── US Settlement holidays ──────────────────────────────────────────────────

#[test]
fn test_us_settlement_holidays() {
    let expected_2004_2005: Vec<Date> = vec![
        // 2004
        date(2004, 1, 1),
        date(2004, 1, 19),
        date(2004, 2, 16),
        date(2004, 5, 31),
        date(2004, 7, 5),
        date(2004, 9, 6),
        date(2004, 10, 11),
        date(2004, 11, 11),
        date(2004, 11, 25),
        date(2004, 12, 24),
        // Dec 31 (Friday) −− observed for New Year 2005
        date(2004, 12, 31),
        // 2005
        date(2005, 1, 17),
        date(2005, 2, 21),
        date(2005, 5, 30),
        date(2005, 7, 4),
        date(2005, 9, 5),
        date(2005, 10, 10),
        date(2005, 11, 11),
        date(2005, 11, 24),
        date(2005, 12, 26),
    ];

    let cal = UnitedStatesSettlement;
    check_holidays(&cal, date(2004, 1, 1), date(2005, 12, 31), &expected_2004_2005);
}

// ─── US NYSE holidays ────────────────────────────────────────────────────────

#[test]
fn test_us_nyse_holidays() {
    let expected_2004_2006: Vec<Date> = vec![
        // 2004
        date(2004, 1, 1),
        date(2004, 1, 19),
        date(2004, 2, 16),
        date(2004, 4, 9),
        date(2004, 5, 31),
        date(2004, 6, 11), // Reagan's funeral
        date(2004, 7, 5),
        date(2004, 9, 6),
        date(2004, 11, 25),
        date(2004, 12, 24),
        // 2005
        date(2005, 1, 17),
        date(2005, 2, 21),
        date(2005, 3, 25),
        date(2005, 5, 30),
        date(2005, 7, 4),
        date(2005, 9, 5),
        date(2005, 11, 24),
        date(2005, 12, 26),
        // 2006
        date(2006, 1, 2),
        date(2006, 1, 16),
        date(2006, 2, 20),
        date(2006, 4, 14),
        date(2006, 5, 29),
        date(2006, 7, 4),
        date(2006, 9, 4),
        date(2006, 11, 23),
        date(2006, 12, 25),
    ];

    let cal = UnitedStatesNyse;
    check_holidays(&cal, date(2004, 1, 1), date(2006, 12, 31), &expected_2004_2006);
}

/// The C++ test also verifies historical closings for NYSE.
#[test]
fn test_us_nyse_historical_closings() {
    let cal = UnitedStatesNyse;

    let historical_closings = vec![
        date(2012, 10, 30), // Hurricane Sandy
        date(2012, 10, 29), // Hurricane Sandy
        date(2004, 6, 11),  // Reagan's funeral
        date(2001, 9, 14),  // September 11, 2001
        date(2001, 9, 13),
        date(2001, 9, 12),
        date(2001, 9, 11),
    ];

    for d in &historical_closings {
        assert!(
            cal.is_holiday(*d),
            "NYSE: {} should be holiday (historical close)",
            d
        );
    }
}

// ─── Brazil holidays ─────────────────────────────────────────────────────────

#[test]
fn test_brazil_holidays() {
    let expected: Vec<Date> = vec![
        // 2005
        // Jan 1 is Saturday — not included
        date(2005, 2, 7),
        date(2005, 2, 8),
        date(2005, 3, 25),
        date(2005, 4, 21),
        // May 1 is Sunday — not included
        date(2005, 5, 26),
        date(2005, 9, 7),
        date(2005, 10, 12),
        date(2005, 11, 2),
        date(2005, 11, 15),
        // Dec 25 is Sunday — not included
        // 2006
        // Jan 1 is Sunday — not included
        date(2006, 2, 27),
        date(2006, 2, 28),
        date(2006, 4, 14),
        date(2006, 4, 21),
        date(2006, 5, 1),
        date(2006, 6, 15),
        date(2006, 9, 7),
        date(2006, 10, 12),
        date(2006, 11, 2),
        date(2006, 11, 15),
        date(2006, 12, 25),
    ];

    let cal = Brazil;
    check_holidays(&cal, date(2005, 1, 1), date(2006, 12, 31), &expected);
}

// ─── Denmark holidays ────────────────────────────────────────────────────────

#[test]
fn test_denmark_holidays() {
    let expected: Vec<Date> = vec![
        // 2020
        date(2020, 1, 1),
        date(2020, 4, 9),
        date(2020, 4, 10),
        date(2020, 4, 13),
        date(2020, 5, 8),
        date(2020, 5, 21),
        date(2020, 5, 22),
        date(2020, 6, 1),
        date(2020, 6, 5),
        date(2020, 12, 24),
        date(2020, 12, 25),
        // Dec 26 is Saturday — excluded from weekday-only list
        date(2020, 12, 31),
        // 2021
        date(2021, 1, 1),
        date(2021, 4, 1),
        date(2021, 4, 2),
        date(2021, 4, 5),
        date(2021, 4, 30),
        date(2021, 5, 13),
        date(2021, 5, 14),
        date(2021, 5, 24),
        // Jun 5 is Saturday — excluded
        date(2021, 12, 24),
        // Dec 25 is Saturday — excluded
        // Dec 26 is Sunday — excluded
        date(2021, 12, 31),
        // 2022
        // Jan 1 is Saturday — excluded
        date(2022, 4, 14),
        date(2022, 4, 15),
        date(2022, 4, 18),
        date(2022, 5, 13),
        date(2022, 5, 26),
        date(2022, 5, 27),
        // Jun 5 is Sunday — excluded
        date(2022, 6, 6),
        // Dec 24 is Saturday — excluded
        // Dec 25 is Sunday — excluded
        date(2022, 12, 26),
        // Dec 31 is Saturday — excluded
    ];

    let cal = Denmark;
    check_holidays(&cal, date(2020, 1, 1), date(2022, 12, 31), &expected);
}

// ─── Germany holidays ────────────────────────────────────────────────────────

#[test]
fn test_germany_settlement_holidays() {
    // Germany (Settlement) has Christmas Eve and New Year's Eve as holidays
    // beyond the Frankfurt Stock Exchange calendar.
    let expected: Vec<Date> = vec![
        // 2003
        date(2003, 1, 1),
        date(2003, 4, 18),
        date(2003, 4, 21),
        date(2003, 5, 1),
        date(2003, 5, 29), // Ascension Thursday
        date(2003, 6, 9),  // Whit Monday
        date(2003, 10, 3),
        date(2003, 12, 24),
        date(2003, 12, 25),
        date(2003, 12, 26),
        date(2003, 12, 31),
        // 2004
        date(2004, 1, 1),
        date(2004, 4, 9),
        date(2004, 4, 12),
        // May 1 is Saturday — excluded
        date(2004, 5, 20), // Ascension Thursday
        date(2004, 5, 31), // Whit Monday
        // Oct 3 is Sunday — excluded
        date(2004, 12, 24),
        // Dec 25 is Saturday — excluded
        // Dec 26 is Sunday — excluded
        date(2004, 12, 31),
    ];

    let cal = Germany;
    check_holidays(&cal, date(2003, 1, 1), date(2004, 12, 31), &expected);
}

// ─── Joint calendars ─────────────────────────────────────────────────────────

#[test]
fn test_joint_calendars() {
    let c1 = Target;
    let c2 = UnitedKingdomSettlement;
    let c3 = UnitedStatesNyse;
    let c4 = Japan;
    let c5 = Germany;

    let c12h = JointCalendar::new(
        vec![Box::new(c1), Box::new(c2)],
        JointCalendarRule::JoinHolidays,
    );
    let c12b = JointCalendar::new(
        vec![Box::new(c1), Box::new(c2)],
        JointCalendarRule::JoinBusinessDays,
    );
    let c123h = JointCalendar::new(
        vec![Box::new(c1), Box::new(c2), Box::new(c3)],
        JointCalendarRule::JoinHolidays,
    );
    let c123b = JointCalendar::new(
        vec![Box::new(c1), Box::new(c2), Box::new(c3)],
        JointCalendarRule::JoinBusinessDays,
    );
    let c1234h = JointCalendar::new(
        vec![Box::new(c1), Box::new(c2), Box::new(c3), Box::new(c4)],
        JointCalendarRule::JoinHolidays,
    );
    let c1234b = JointCalendar::new(
        vec![Box::new(c1), Box::new(c2), Box::new(c3), Box::new(c4)],
        JointCalendarRule::JoinBusinessDays,
    );
    let cvh = JointCalendar::new(
        vec![
            Box::new(c1),
            Box::new(c2),
            Box::new(c3),
            Box::new(c4),
            Box::new(c5),
        ],
        JointCalendarRule::JoinHolidays,
    );

    // Test over two years (2024-2025)
    let first_date = date(2024, 1, 1);
    let end_date = date(2025, 12, 31);

    let mut d = first_date;
    while d <= end_date {
        let b1 = c1.is_business_day(d);
        let b2 = c2.is_business_day(d);
        let b3 = c3.is_business_day(d);
        let b4 = c4.is_business_day(d);
        let b5 = c5.is_business_day(d);

        // JoinHolidays: business day only if ALL components say so
        assert_eq!(
            b1 && b2,
            c12h.is_business_day(d),
            "at {d}: joint c12h (JoinHolidays) inconsistent with components"
        );

        // JoinBusinessDays: business day if ANY component says so
        assert_eq!(
            b1 || b2,
            c12b.is_business_day(d),
            "at {d}: joint c12b (JoinBusinessDays) inconsistent with components"
        );

        assert_eq!(
            b1 && b2 && b3,
            c123h.is_business_day(d),
            "at {d}: joint c123h inconsistent"
        );
        assert_eq!(
            b1 || b2 || b3,
            c123b.is_business_day(d),
            "at {d}: joint c123b inconsistent"
        );
        assert_eq!(
            b1 && b2 && b3 && b4,
            c1234h.is_business_day(d),
            "at {d}: joint c1234h inconsistent"
        );
        assert_eq!(
            b1 || b2 || b3 || b4,
            c1234b.is_business_day(d),
            "at {d}: joint c1234b inconsistent"
        );
        assert_eq!(
            b1 && b2 && b3 && b4 && b5,
            cvh.is_business_day(d),
            "at {d}: joint cvh inconsistent"
        );

        d += 1;
    }
}

// ─── End of month ────────────────────────────────────────────────────────────

#[test]
fn test_end_of_month() {
    let cal = Target;

    // Iterate from a few months after min to a few months before max.
    // We can't cover the full range economically, so sample 2000–2100.
    let mut counter = date(2000, 1, 1);
    let last = date(2100, 12, 31);

    while counter <= last {
        let eom = cal.end_of_month(counter);

        // eom must be an end-of-month
        assert!(
            cal.is_end_of_month(eom),
            "{} {} is not the last business day in {}/{}",
            eom.weekday(),
            eom,
            eom.month(),
            eom.year()
        );

        // eom must be in the same month as counter
        assert_eq!(
            eom.month(),
            counter.month(),
            "{eom} is not in the same month as {counter}"
        );

        // The next business day must be in a different month
        let next = cal.adjust(eom + 1, BusinessDayConvention::Following);
        assert_ne!(
            next.month(),
            eom.month(),
            "next business day after EOM {} is {} — same month",
            eom,
            next
        );

        counter += 1;
    }
}

// ─── Start of month ──────────────────────────────────────────────────────────

/// Test that the first business day of a month can be found via `adjust`.
/// The C++ test uses `Calendar::startOfMonth`; we don't have that method yet,
/// but the logic is: adjust the 1st of the month with Following.
#[test]
fn test_start_of_month() {
    let cal = Target;

    for year in 2000..=2100u16 {
        for month in 1..=12u8 {
            let first = date(year, month, 1);
            let som = cal.adjust(first, BusinessDayConvention::Following);

            // som must be in the same month
            assert_eq!(
                som.month(),
                month,
                "start of month for {year}/{month} landed in different month: {som}"
            );

            // som must be a business day
            assert!(
                cal.is_business_day(som),
                "start of month {som} is not a business day"
            );

            // The day before som must be in a previous month OR be a holiday
            if som > first {
                let prev = som - 1;
                assert!(
                    cal.is_holiday(prev) || prev.month() != month,
                    "day before SOM {prev} should be a holiday or in a different month"
                );
            }
        }
    }
}

// ─── Business days between ───────────────────────────────────────────────────

#[test]
fn test_business_days_between() {
    let cal = Brazil;

    // Our Rust `business_days_between(d1, d2)` counts (d1, d2] — d1 exclusive,
    // d2 inclusive.
    //
    // Verify basic properties:
    // 1) business_days_between(d, d) == 0
    // 2) business_days_between(d1, d2) == -business_days_between(d2, d1)
    // 3) Known value: Feb 1 2002 (Fri) to Feb 4 2002 (Mon) = 1 business day
    //    in the interval (Feb 1, Feb 4] = {Feb 2 (Sat), Feb 3 (Sun), Feb 4 (Mon)}
    //    Only Feb 4 is a business day → 1.
    let d1 = date(2002, 2, 1);
    let d2 = date(2002, 2, 4);
    assert_eq!(cal.business_days_between(d1, d1), 0);
    assert_eq!(cal.business_days_between(d1, d2), 1);
    assert_eq!(cal.business_days_between(d2, d1), -1);

    // More extensive test: count over a known range and check symmetry.
    let test_dates = vec![
        date(2002, 2, 1),
        date(2002, 2, 4),
        date(2003, 5, 16),
        date(2003, 12, 17),
        date(2004, 12, 17),
        date(2005, 12, 19),
        date(2006, 1, 2),
        date(2006, 3, 13),
        date(2006, 5, 15),
    ];

    for i in 0..test_dates.len() {
        for j in 0..test_dates.len() {
            let fwd = cal.business_days_between(test_dates[i], test_dates[j]);
            let bwd = cal.business_days_between(test_dates[j], test_dates[i]);
            assert_eq!(
                fwd, -bwd,
                "asymmetry: bdb({}, {}) = {fwd} but bdb({}, {}) = {bwd}",
                test_dates[i], test_dates[j], test_dates[j], test_dates[i]
            );
        }
    }

    // All business days between d and d+1 on a weekday should be 0 or 1.
    let mut d = date(2002, 1, 1);
    let end = date(2006, 12, 31);
    while d < end {
        let bdb = cal.business_days_between(d, d + 1);
        assert!(
            bdb == 0 || bdb == 1,
            "bdb({d}, {}) should be 0 or 1, got {bdb}",
            d + 1
        );
        // It should be 1 exactly when d+1 is a business day.
        let expected = if cal.is_business_day(d + 1) { 1 } else { 0 };
        assert_eq!(
            bdb, expected,
            "bdb({d}, {}) = {bdb}, expected {expected}",
            d + 1
        );
        d += 1;
    }
}

// ─── Bespoke calendar ────────────────────────────────────────────────────────

#[test]
fn test_bespoke_calendar() {
    let mut cal = BespokeCalendar::new("Test");

    let sat = date(2008, 10, 4);  // Saturday
    let sun = date(2008, 10, 5);  // Sunday
    let mon = date(2008, 10, 6);  // Monday
    let tue = date(2008, 10, 7);  // Tuesday

    // Rust BespokeCalendar treats Sat/Sun as weekends by default (unlike C++).
    assert!(!cal.is_business_day(sat), "Sat is a weekend");
    assert!(!cal.is_business_day(sun), "Sun is a weekend");
    assert!(cal.is_business_day(mon));
    assert!(cal.is_business_day(tue));

    // Add Monday as an explicit holiday.
    cal.add_holiday(mon);
    assert!(!cal.is_business_day(mon), "Mon should now be a holiday");
    assert!(cal.is_business_day(tue));
    assert_eq!(cal.holiday_count(), 1);

    // Remove it.
    cal.remove_holiday(mon);
    assert!(cal.is_business_day(mon), "Mon should be restored as business day");
    assert_eq!(cal.holiday_count(), 0);
}

// ─── Adjust conventions ──────────────────────────────────────────────────────

#[test]
fn test_adjust_conventions() {
    let cal = Target;

    // 2004-04-10 is Saturday, 2004-04-11 is Sunday (Easter).
    // 2004-04-09 is Good Friday (holiday). So the next business day is
    // Monday 2004-04-12 (Easter Monday is also a holiday for TARGET!).
    // Actually, TARGET: Good Friday and Easter Monday are holidays.
    // Fri Apr 9 = Good Friday (holiday)
    // Sat Apr 10 (weekend)
    // Sun Apr 11 (weekend)
    // Mon Apr 12 = Easter Monday (holiday)
    // Tue Apr 13 = first business day

    let good_friday = date(2004, 4, 9);
    let easter_monday = date(2004, 4, 12);
    let tuesday = date(2004, 4, 13);

    assert!(cal.is_holiday(good_friday));
    assert!(cal.is_holiday(easter_monday));
    assert!(cal.is_business_day(tuesday));

    // Following from Good Friday → Tuesday
    assert_eq!(
        cal.adjust(good_friday, BusinessDayConvention::Following),
        tuesday
    );

    // Preceding from Easter Monday → Thursday Apr 8
    let thursday = date(2004, 4, 8);
    assert_eq!(
        cal.adjust(easter_monday, BusinessDayConvention::Preceding),
        thursday
    );

    // ModifiedFollowing at month boundary: Mar 31, 2006 is Good Friday.
    // Following would give Apr 3, which is a different month → fall back to Preceding.
    // Apr 3 is Easter Monday (also holiday) → Apr 4 (Tuesday)? No, Modified
    // goes Preceding from the original date (Mar 31).
    // Preceding from Mar 31: Mar 30 (Thursday) is a business day → Mar 30.
    let _good_friday_2006 = date(2006, 3, 31);
    // Actually, let me check: 2006 Easter Sunday is April 16, so Good Friday
    // is April 14. March 31, 2006 is a Friday and a business day. Let me
    // pick a better example.
    //
    // May 1, 2004 is Saturday (TARGET holiday too). Following → Monday May 3.
    // Same month, so ModifiedFollowing also → May 3.
    let may_1 = date(2004, 5, 1);
    assert_eq!(
        cal.adjust(may_1, BusinessDayConvention::ModifiedFollowing),
        date(2004, 5, 3)
    );

    // Nearest: from Saturday May 1, 2004:
    // Following → Mon May 3 (2 days ahead)
    // Preceding → Fri Apr 30 (1 day back)
    // Nearest → Apr 30 (closer)
    assert_eq!(
        cal.adjust(may_1, BusinessDayConvention::Nearest),
        date(2004, 4, 30)
    );

    // Unadjusted always returns the original.
    assert_eq!(
        cal.adjust(may_1, BusinessDayConvention::Unadjusted),
        may_1
    );
}

// ─── Advance business days ───────────────────────────────────────────────────

#[test]
fn test_advance_business_days() {
    let cal = Target;

    // From Thursday Apr 8, 2004, advance 1 business day.
    // Apr 9 = Good Friday (holiday), Apr 10-11 = weekend, Apr 12 = Easter Monday
    // (holiday), so next biz day = Apr 13 (Tuesday).
    let thursday = date(2004, 4, 8);
    assert_eq!(cal.advance_business_days(thursday, 1), date(2004, 4, 13));

    // Advance 0 should return the same date.
    assert_eq!(cal.advance_business_days(thursday, 0), thursday);

    // Go backward 1 business day from Tuesday Apr 13
    // → Thursday Apr 8 (since Apr 12, 11, 10, 9 are non-business days).
    assert_eq!(
        cal.advance_business_days(date(2004, 4, 13), -1),
        thursday
    );

    // Advance 5 business days from Monday Jan 5, 2004
    // → Mon Jan 12 (straight week, no holidays).
    assert_eq!(
        cal.advance_business_days(date(2004, 1, 5), 5),
        date(2004, 1, 12)
    );
}

// ─── Spot-check various calendars ────────────────────────────────────────────

/// Quick sanity checks for a handful of calendars on known dates.
#[test]
fn test_spot_check_various_calendars() {
    // New Year's Day is a holiday almost everywhere.
    let new_year_2024 = date(2024, 1, 1); // Monday
    assert!(Target.is_holiday(new_year_2024));
    assert!(UnitedStatesSettlement.is_holiday(new_year_2024));
    assert!(UnitedStatesNyse.is_holiday(new_year_2024));
    assert!(Germany.is_holiday(new_year_2024));
    assert!(Japan.is_holiday(new_year_2024));

    // Christmas Day
    let xmas = date(2024, 12, 25); // Wednesday
    assert!(Target.is_holiday(xmas));
    assert!(UnitedStatesSettlement.is_holiday(xmas));
    assert!(UnitedStatesNyse.is_holiday(xmas));
    assert!(Germany.is_holiday(xmas));
    assert!(UnitedKingdomSettlement.is_holiday(xmas));

    // Japanese Constitution Memorial Day (May 3) — 2024 is Friday
    assert!(Japan.is_holiday(date(2024, 5, 3)));

    // Japan Golden Week
    assert!(Japan.is_holiday(date(2024, 4, 29))); // Showa Day
    assert!(Japan.is_holiday(date(2024, 5, 3)));  // Constitution Memorial Day
    assert!(Japan.is_holiday(date(2024, 5, 4)));  // Greenery Day (Saturday)
    assert!(Japan.is_holiday(date(2024, 5, 6)));  // Children's Day substitute

    // A normal Wednesday should be a business day everywhere.
    let normal_wed = date(2024, 3, 20); // March 20 = Vernal Equinox Day in Japan!
    assert!(Target.is_business_day(normal_wed));
    assert!(UnitedStatesSettlement.is_business_day(normal_wed));
    // Japan: March 20 is Vernal Equinox Day, so it's a holiday.
    assert!(Japan.is_holiday(normal_wed));
}

// ─── UK Settlement holidays ──────────────────────────────────────────────────

#[test]
fn test_uk_settlement_holidays() {
    let expected: Vec<Date> = vec![
        // 2004
        date(2004, 1, 1),
        date(2004, 4, 9),
        date(2004, 4, 12),
        date(2004, 5, 3),
        date(2004, 5, 31),
        date(2004, 8, 30),
        date(2004, 12, 27),
        date(2004, 12, 28),
        // 2005
        date(2005, 1, 3),
        date(2005, 3, 25),
        date(2005, 3, 28),
        date(2005, 5, 2),
        date(2005, 5, 30),
        date(2005, 8, 29),
        date(2005, 12, 26),
        date(2005, 12, 27),
        // 2006
        date(2006, 1, 2),
        date(2006, 4, 14),
        date(2006, 4, 17),
        date(2006, 5, 1),
        date(2006, 5, 29),
        date(2006, 8, 28),
        date(2006, 12, 25),
        date(2006, 12, 26),
        // 2007
        date(2007, 1, 1),
        date(2007, 4, 6),
        date(2007, 4, 9),
        date(2007, 5, 7),
        date(2007, 5, 28),
        date(2007, 8, 27),
        date(2007, 12, 25),
        date(2007, 12, 26),
    ];

    let cal = UnitedKingdomSettlement;
    check_holidays(&cal, date(2004, 1, 1), date(2007, 12, 31), &expected);
}
