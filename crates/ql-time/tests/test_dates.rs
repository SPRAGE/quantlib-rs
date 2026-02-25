//! Tests ported from QuantLib `test-suite/dates.cpp`.
//!
//! These integration tests exercise the `Date`, `IMM`, `ASX`, and `ECB` types.

use std::collections::HashSet;

use ql_time::date::{days_in_month, is_leap_year};
use ql_time::weekday::Weekday;
use ql_time::{Date, ASX, ECB, IMM};

fn date(y: u16, m: u8, d: u8) -> Date {
    Date::from_ymd(y, m, d).unwrap()
}

// ─── ECB tests ────────────────────────────────────────────────────────────────

#[test]
fn ecb_dates() {
    let known = ECB::known_dates();
    assert!(!known.is_empty(), "empty ECB date vector");

    // Every known date must pass is_ecb_date
    for &d in &known {
        assert!(ECB::is_ecb_date(d), "{d} fails is_ecb_date check");
    }

    // The day before each ECB date must NOT be an ECB date
    for &d in &known {
        let prev = d - 1;
        assert!(!ECB::is_ecb_date(prev), "{prev} should not be an ECB date");
    }

    // next_date from before the first known date should return the first known date
    let first = known[0];
    let before_first = first - 1;
    let next = ECB::next_date(before_first);
    assert_eq!(next, Some(first), "next_date before first should give first");
}

// ─── IMM tests ────────────────────────────────────────────────────────────────

#[test]
fn imm_dates() {
    // Iterate over a range of dates and verify IMM invariants
    let start = date(2000, 1, 1);
    let end = date(2040, 1, 1);

    let mut counter = start;
    while counter <= end {
        let imm = IMM::next_date(counter);

        // IMM date must be >= counter
        assert!(
            imm >= counter,
            "{imm} is not >= {counter}"
        );

        // Must be a valid IMM date
        assert!(
            IMM::is_imm_date(imm),
            "{imm} is not an IMM date (calculated from {counter})"
        );

        // code/roundtrip: code should be Some for an IMM date
        let code = IMM::code(imm);
        assert!(
            code.is_some(),
            "IMM::code returned None for IMM date {imm}"
        );

        counter = counter + 1;
    }
}

#[test]
fn imm_specific_dates() {
    // 3rd Wednesday of March 2024 = March 20
    assert!(IMM::is_imm_date(date(2024, 3, 20)));
    assert_eq!(IMM::code(date(2024, 3, 20)), Some("H4".to_string()));

    // 3rd Wednesday of June 2024 = June 19
    assert!(IMM::is_imm_date(date(2024, 6, 19)));
    assert_eq!(IMM::code(date(2024, 6, 19)), Some("M4".to_string()));

    // 3rd Wednesday of Sep 2024 = Sep 18
    assert!(IMM::is_imm_date(date(2024, 9, 18)));
    assert_eq!(IMM::code(date(2024, 9, 18)), Some("U4".to_string()));

    // 3rd Wednesday of Dec 2024 = Dec 18
    assert!(IMM::is_imm_date(date(2024, 12, 18)));
    assert_eq!(IMM::code(date(2024, 12, 18)), Some("Z4".to_string()));
}

// ─── ASX tests ────────────────────────────────────────────────────────────────

#[test]
fn asx_dates() {
    // Iterate over a range of dates and verify ASX invariants
    let start = date(2000, 1, 1);
    let end = date(2040, 1, 1);

    let mut counter = start;
    while counter <= end {
        let asx = ASX::next_date(counter);

        // ASX date must be >= counter
        assert!(
            asx >= counter,
            "{asx} is not >= {counter}"
        );

        // Must be a valid ASX date
        assert!(
            ASX::is_asx_date(asx),
            "{asx} is not an ASX date (calculated from {counter})"
        );

        // code should be Some for an ASX date
        let code = ASX::code(asx);
        assert!(
            code.is_some(),
            "ASX::code returned None for ASX date {asx}"
        );

        counter = counter + 1;
    }
}

#[test]
fn asx_specific_dates() {
    // 2nd Friday of March 2024 = March 8
    assert!(ASX::is_asx_date(date(2024, 3, 8)));
    assert_eq!(ASX::code(date(2024, 3, 8)), Some("H4".to_string()));

    // 2nd Friday of June 2024 = June 14
    assert!(ASX::is_asx_date(date(2024, 6, 14)));
    assert_eq!(ASX::code(date(2024, 6, 14)), Some("M4".to_string()));
}

// ─── Date consistency test ────────────────────────────────────────────────────

#[test]
fn test_consistency() {
    // Ported from the C++ testConsistency test.
    // Iterate over the entire valid date range and check every invariant.
    let min_serial = Date::MIN.serial() + 1;
    let max_serial = Date::MAX.serial();

    let prev = Date::from_serial(min_serial - 1).unwrap();
    let mut dy_old = prev.day_of_year() as i32;
    let mut d_old = prev.day_of_month() as i32;
    let mut m_old = prev.month() as i32;
    let mut y_old = prev.year() as i32;
    let mut wd_old = prev.weekday().ordinal() as i32;

    for i in min_serial..=max_serial {
        let t = Date::from_serial(i).unwrap();
        let serial = t.serial();

        // Check serial number consistency
        assert_eq!(serial, i, "inconsistent serial for date {t}");

        let dy = t.day_of_year() as i32;
        let d = t.day_of_month() as i32;
        let m = t.month() as i32;
        let y = t.year() as i32;
        let wd = t.weekday().ordinal() as i32;

        // Check day-of-year increment
        assert!(
            (dy == dy_old + 1)
                || (dy == 1 && dy_old == 365 && !is_leap_year(y_old as u16))
                || (dy == 1 && dy_old == 366 && is_leap_year(y_old as u16)),
            "wrong day of year increment: date={t}, dy={dy}, prev={dy_old}"
        );
        dy_old = dy;

        // Check day/month/year increment
        assert!(
            (d == d_old + 1 && m == m_old && y == y_old)
                || (d == 1 && m == m_old + 1 && y == y_old)
                || (d == 1 && m == 1 && y == y_old + 1),
            "wrong day/month/year increment: date={t}, d/m/y={d}/{m}/{y}, \
             prev={d_old}/{m_old}/{y_old}"
        );
        d_old = d;
        m_old = m;
        y_old = y;

        // Check month range
        assert!(
            (1..=12).contains(&m),
            "invalid month: date={t}, month={m}"
        );

        // Check day range for the month
        let max_day = days_in_month(y as u16, m as u8) as i32;
        assert!(
            d >= 1 && d <= max_day,
            "invalid day of month: date={t}, day={d}, max={max_day}"
        );

        // Check weekday increment (wraps from 7 to 1)
        assert!(
            (wd == wd_old + 1) || (wd == 1 && wd_old == 7),
            "invalid weekday increment: date={t}, wd={wd}, prev_wd={wd_old}"
        );
        wd_old = wd;

        // Check roundtrip: construct from y/m/d, verify same serial
        let s = Date::from_ymd(y as u16, m as u8, d as u8).unwrap();
        assert_eq!(
            s.serial(),
            i,
            "roundtrip failed: date={t}, serial={i}, cloned serial={}",
            s.serial()
        );
    }
}

// ─── Hash test ────────────────────────────────────────────────────────────────

#[test]
fn can_hash() {
    use std::hash::{Hash, Hasher};

    fn hash_of(d: Date) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        d.hash(&mut hasher);
        hasher.finish()
    }

    let start = date(2020, 1, 1);
    let nb_tests = 500;

    // Check hash consistency: equal dates have equal hashes, different dates
    // have different hashes (for this range, at least).
    for i in 0..nb_tests {
        for j in 0..nb_tests {
            let lhs = start + i;
            let rhs = start + j;

            if lhs == rhs {
                assert_eq!(
                    hash_of(lhs),
                    hash_of(rhs),
                    "equal dates should have same hash: {lhs} vs {rhs}"
                );
            } else {
                assert_ne!(
                    hash_of(lhs),
                    hash_of(rhs),
                    "different dates should have different hash: {lhs} vs {rhs}"
                );
            }
        }
    }

    // Check Date works as HashSet key
    let mut set = HashSet::new();
    set.insert(start);
    assert!(set.contains(&start), "expected to find date in HashSet");
}

// ─── Null date test ───────────────────────────────────────────────────────────

#[test]
fn null_date() {
    let null = Date::NULL;
    assert!(null.is_null());
    // Serial number should work on null date
    assert_eq!(null.serial(), 0);
}

// ─── ISO date parsing ─────────────────────────────────────────────────────────
// Note: QuantLib has DateParser::parseISO. If our Date supports from_iso_string
// or similar, test it here. For now, test the YMD constructor equivalent.

#[test]
fn iso_dates() {
    // Equivalent to parsing "2006-01-15"
    let d = date(2006, 1, 15);
    assert_eq!(d.day_of_month(), 15);
    assert_eq!(d.month(), 1);
    assert_eq!(d.year(), 2006);
}

// ─── Leap year tests ─────────────────────────────────────────────────────────

#[test]
fn leap_years() {
    assert!(is_leap_year(2000));
    assert!(!is_leap_year(1900));
    assert!(is_leap_year(2004));
    assert!(!is_leap_year(2001));
    assert!(is_leap_year(2400));
    assert!(!is_leap_year(2100));
}

// ─── End of month tests ──────────────────────────────────────────────────────

#[test]
fn end_of_month() {
    // Feb end in leap year
    let d = date(2024, 2, 29);
    assert!(d.is_end_of_month());
    assert!(!date(2024, 2, 28).is_end_of_month());

    // Feb end in non-leap year
    let d = date(2023, 2, 28);
    assert!(d.is_end_of_month());

    // December
    let d = date(2023, 12, 31);
    assert!(d.is_end_of_month());
    assert!(!date(2023, 12, 30).is_end_of_month());
}

// ─── Date arithmetic tests ──────────────────────────────────────────────────

#[test]
fn date_arithmetic() {
    let d = date(2024, 1, 15);

    // Add days
    let d2 = d + 10;
    assert_eq!(d2, date(2024, 1, 25));

    // Subtract days
    let d3 = d - 15;
    assert_eq!(d3, date(2023, 12, 31));

    // Difference
    assert_eq!(d2 - d3, 25);

    // Add crossing month boundary
    let d4 = date(2024, 1, 31) + 1;
    assert_eq!(d4, date(2024, 2, 1));

    // Add crossing year boundary
    let d5 = date(2023, 12, 31) + 1;
    assert_eq!(d5, date(2024, 1, 1));
}

// ─── Weekday tests ──────────────────────────────────────────────────────────

#[test]
fn weekday_consistency() {
    // Known: 2024-01-01 is Monday
    assert_eq!(date(2024, 1, 1).weekday(), Weekday::Monday);
    assert_eq!(date(2024, 1, 2).weekday(), Weekday::Tuesday);
    assert_eq!(date(2024, 1, 6).weekday(), Weekday::Saturday);
    assert_eq!(date(2024, 1, 7).weekday(), Weekday::Sunday);
}
