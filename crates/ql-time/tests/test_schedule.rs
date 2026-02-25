//! Tests ported from QuantLib `test-suite/schedule.cpp`.
//!
//! These integration tests exercise the `Schedule`, `ScheduleBuilder`, and
//! `DateGeneration` types.

use ql_time::calendar::{NullCalendar, WeekendsOnly};
use ql_time::calendars::japan::Japan;
use ql_time::calendars::target::Target;
use ql_time::schedule::{DateGeneration, ScheduleBuilder};
use ql_time::{BusinessDayConvention, Date, Frequency, Period, TimeUnit};

fn date(y: u16, m: u8, d: u8) -> Date {
    Date::from_ymd(y, m, d).unwrap()
}

/// Assert that the schedule dates match `expected` exactly.
fn check_dates(s: &ql_time::Schedule, expected: &[Date]) {
    assert_eq!(
        s.size(),
        expected.len(),
        "expected {} dates, found {}.\n  actual:   {:?}\n  expected: {:?}",
        expected.len(),
        s.size(),
        s.dates(),
        expected,
    );
    for (i, exp) in expected.iter().enumerate() {
        assert_eq!(
            s.date(i),
            *exp,
            "at index {}: expected {}, found {}",
            i,
            exp,
            s.date(i),
        );
    }
}

// ───────────────────────── testDailySchedule ─────────────────────────

#[test]
fn test_daily_schedule() {
    // C++ test: schedule with daily frequency.
    // The schedule should skip Saturday 21st and Sunday 22nd.
    // Previously, it would adjust them to Friday 20th, resulting
    // in three copies of the same date.
    let start = date(2012, 1, 17);
    let cal = Target;
    let sched = ScheduleBuilder::new(start, start + 7, Period::new(1, TimeUnit::Days), &cal)
        .with_convention(BusinessDayConvention::Preceding)
        .with_termination_convention(BusinessDayConvention::Preceding)
        .build()
        .unwrap();

    let expected = [
        date(2012, 1, 17),
        date(2012, 1, 18),
        date(2012, 1, 19),
        date(2012, 1, 20),
        date(2012, 1, 23),
        date(2012, 1, 24),
    ];
    check_dates(&sched, &expected);
}

// ───────────────── testEomAdjustment (3 conventions) ─────────────────

#[test]
fn test_eom_adjustment_unadjusted() {
    let start = date(2024, 2, 29);
    let end = date(2025, 2, 28); // Feb 29 2024 + 1Y
    let cal = Target;
    let sched = ScheduleBuilder::new(start, end, Period::new(1, TimeUnit::Months), &cal)
        .with_convention(BusinessDayConvention::Unadjusted)
        .with_termination_convention(BusinessDayConvention::Unadjusted)
        .with_rule(DateGeneration::Forward)
        .end_of_month(true)
        .build()
        .unwrap();

    let expected = [
        date(2024, 2, 29),
        date(2024, 3, 31),
        date(2024, 4, 30),
        date(2024, 5, 31),
        date(2024, 6, 30),
        date(2024, 7, 31),
        date(2024, 8, 31),
        date(2024, 9, 30),
        date(2024, 10, 31),
        date(2024, 11, 30),
        date(2024, 12, 31),
        date(2025, 1, 31),
        date(2025, 2, 28),
    ];
    check_dates(&sched, &expected);
}

#[test]
fn test_eom_adjustment_following() {
    let start = date(2024, 2, 29);
    let end = date(2025, 2, 28);
    let cal = Target;
    let sched = ScheduleBuilder::new(start, end, Period::new(1, TimeUnit::Months), &cal)
        .with_convention(BusinessDayConvention::Following)
        .with_termination_convention(BusinessDayConvention::Following)
        .with_rule(DateGeneration::Forward)
        .end_of_month(true)
        .build()
        .unwrap();

    // With Following convention, weekend/holiday dates adjust forward.
    // Mar 31 2024 = Sun (Easter) → Apr 2 (Mon, since Mar 29 = Good Friday)
    // Apr 30 2024 = Tue → Apr 30
    // May 31 2024 = Fri → May 31
    // Jun 30 2024 = Sun → Jul 1
    // Jul 31 2024 = Wed → Jul 31
    // Aug 31 2024 = Sat → Sep 2
    // Sep 30 2024 = Mon → Sep 30
    // Oct 31 2024 = Thu → Oct 31
    // Nov 30 2024 = Sat → Dec 2
    // Dec 31 2024 = Tue → Dec 31
    // Jan 31 2025 = Fri → Jan 31
    // Feb 28 2025 = Fri → Feb 28
    let expected = [
        date(2024, 2, 29),
        date(2024, 4, 2),
        date(2024, 4, 30),
        date(2024, 5, 31),
        date(2024, 7, 1),
        date(2024, 7, 31),
        date(2024, 9, 2),
        date(2024, 9, 30),
        date(2024, 10, 31),
        date(2024, 12, 2),
        date(2024, 12, 31),
        date(2025, 1, 31),
        date(2025, 2, 28),
    ];
    check_dates(&sched, &expected);
}

#[test]
fn test_eom_adjustment_modified_preceding() {
    let start = date(2024, 2, 29);
    let end = date(2025, 2, 28);
    let cal = Target;
    let sched = ScheduleBuilder::new(start, end, Period::new(1, TimeUnit::Months), &cal)
        .with_convention(BusinessDayConvention::ModifiedPreceding)
        .with_termination_convention(BusinessDayConvention::ModifiedPreceding)
        .with_rule(DateGeneration::Forward)
        .end_of_month(true)
        .build()
        .unwrap();

    // ModifiedPreceding: adjust backward, but stay in the same month.
    // Mar 31 2024 = Sun → Preceding = Fri Mar 29 = Good Friday (TARGET)
    //   → Preceding = Thu Mar 28. Mar 28 is in March, OK.
    // Apr 30 2024 = Tue → Apr 30
    // May 31 2024 = Fri → May 31
    // Jun 30 2024 = Sun → Preceding = Fri Jun 28. Jun 28 in June, OK.
    // Jul 31 2024 = Wed → Jul 31
    // Aug 31 2024 = Sat → Preceding = Fri Aug 30. Aug 30 in Aug, OK.
    // Sep 30 2024 = Mon → Sep 30
    // Oct 31 2024 = Thu → Oct 31
    // Nov 30 2024 = Sat → Preceding = Fri Nov 29. Nov 29 in Nov, OK.
    // Dec 31 2024 = Tue → Dec 31
    // Jan 31 2025 = Fri → Jan 31
    // Feb 28 2025 = Fri → Feb 28
    let expected = [
        date(2024, 2, 29),
        date(2024, 3, 28),
        date(2024, 4, 30),
        date(2024, 5, 31),
        date(2024, 6, 28),
        date(2024, 7, 31),
        date(2024, 8, 30),
        date(2024, 9, 30),
        date(2024, 10, 31),
        date(2024, 11, 29),
        date(2024, 12, 31),
        date(2025, 1, 31),
        date(2025, 2, 28),
    ];
    check_dates(&sched, &expected);
}

// ───────────── testEndDateWithEomAdjustment ──────────────

#[test]
fn test_end_date_with_eom_adjustment() {
    // Forward, 6M, Japan calendar, ModifiedFollowing, EOM.
    let cal = Japan;
    let sched = ScheduleBuilder::new(
        date(2009, 9, 30),
        date(2012, 6, 15),
        Period::new(6, TimeUnit::Months),
        &cal,
    )
    .with_convention(BusinessDayConvention::ModifiedFollowing)
    .with_termination_convention(BusinessDayConvention::ModifiedFollowing)
    .with_rule(DateGeneration::Forward)
    .end_of_month(true)
    .build()
    .unwrap();

    let expected = [
        date(2009, 9, 30),
        date(2010, 3, 31),
        date(2010, 9, 30),
        date(2011, 3, 31),
        date(2011, 9, 30),
        date(2012, 3, 30),
        date(2012, 6, 15),
    ];
    check_dates(&sched, &expected);
}

// ───────── testDatesPastEndDateWithEomAdjustment ─────────

#[test]
fn test_dates_past_end_date_with_eom_adjustment() {
    // Forward, 1Y, TARGET, EOM, Unadjusted.
    // March 31st 2015, coming from the EOM adjustment of March 28th,
    // should be discarded as past the end date.
    let cal = Target;
    let sched = ScheduleBuilder::new(
        date(2013, 3, 28),
        date(2015, 3, 30),
        Period::new(1, TimeUnit::Years),
        &cal,
    )
    .with_convention(BusinessDayConvention::Unadjusted)
    .with_termination_convention(BusinessDayConvention::Unadjusted)
    .with_rule(DateGeneration::Forward)
    .end_of_month(true)
    .build()
    .unwrap();

    let expected = [date(2013, 3, 28), date(2014, 3, 31), date(2015, 3, 30)];
    check_dates(&sched, &expected);

    // The last period should not be regular.
    // C++ isRegular(2) is 1-indexed; Rust is_regular is 0-indexed.
    assert!(!sched.is_regular(1), "last period should not be regular");
}

// ──────── testDatesSameAsEndDateWithEomAdjustment ────────

#[test]
fn test_dates_same_as_end_date_with_eom_adjustment() {
    // Forward, 1Y, TARGET, EOM, Unadjusted.
    // March 31st 2015, coming from the EOM adjustment of March 28th,
    // should be kept since it equals the end date.
    let cal = Target;
    let sched = ScheduleBuilder::new(
        date(2013, 3, 28),
        date(2015, 3, 31),
        Period::new(1, TimeUnit::Years),
        &cal,
    )
    .with_convention(BusinessDayConvention::Unadjusted)
    .with_termination_convention(BusinessDayConvention::Unadjusted)
    .with_rule(DateGeneration::Forward)
    .end_of_month(true)
    .build()
    .unwrap();

    let expected = [date(2013, 3, 28), date(2014, 3, 31), date(2015, 3, 31)];
    check_dates(&sched, &expected);

    // The last period should be regular.
    // C++ isRegular(2) is 1-indexed; Rust is_regular is 0-indexed.
    assert!(sched.is_regular(1), "last period should be regular");
}

// ──────── testForwardDatesWithEomAdjustment ──────────

#[test]
fn test_forward_dates_with_eom_adjustment() {
    // Forward, 6M, Unadjusted, EOM.
    // The last date should not be adjusted for EOM when termination date
    // convention is unadjusted.
    // Original C++ uses UnitedStates::GovernmentBond, but since the
    // convention is Unadjusted the calendar is irrelevant.
    let cal = WeekendsOnly;
    let sched = ScheduleBuilder::new(
        date(1996, 8, 31),
        date(1997, 9, 15),
        Period::new(6, TimeUnit::Months),
        &cal,
    )
    .with_convention(BusinessDayConvention::Unadjusted)
    .with_termination_convention(BusinessDayConvention::Unadjusted)
    .with_rule(DateGeneration::Forward)
    .end_of_month(true)
    .build()
    .unwrap();

    let expected = [
        date(1996, 8, 31),
        date(1997, 2, 28),
        date(1997, 8, 31),
        date(1997, 9, 15),
    ];
    check_dates(&sched, &expected);
}

// ──────── testBackwardDatesWithEomAdjustment ──────────

#[test]
fn test_backward_dates_with_eom_adjustment() {
    // Backward, 6M, Unadjusted, EOM.
    // The first date should not be adjusted for EOM going backward when
    // termination date convention is unadjusted.
    // Original C++ uses UnitedStates::GovernmentBond; Unadjusted → irrelevant.
    let cal = WeekendsOnly;
    let sched = ScheduleBuilder::new(
        date(1996, 8, 22),
        date(1997, 8, 31),
        Period::new(6, TimeUnit::Months),
        &cal,
    )
    .with_convention(BusinessDayConvention::Unadjusted)
    .with_termination_convention(BusinessDayConvention::Unadjusted)
    .with_rule(DateGeneration::Backward)
    .end_of_month(true)
    .build()
    .unwrap();

    let expected = [
        date(1996, 8, 22),
        date(1996, 8, 31),
        date(1997, 2, 28),
        date(1997, 8, 31),
    ];
    check_dates(&sched, &expected);
}

// ─────── testDoubleFirstDateWithEomAdjustment ────────
// Requires GovernmentBond calendar with ModifiedFollowing — skipped for now.

#[test]
#[ignore = "requires UnitedStates::GovernmentBond calendar (not yet ported)"]
fn test_double_first_date_with_eom_adjustment() {
    // Backward, 6M, GovernmentBond, ModifiedFollowing + Following, EOM.
    // The first date should not be duplicated due to EOM convention.
    // TODO: port once GovernmentBond calendar is available.
}

// ─────── testFirstDateWithEomAdjustment ───────

#[test]
#[ignore = "requires UnitedStates::GovernmentBond calendar (not yet ported)"]
fn test_first_date_with_eom_adjustment() {
    // Forward, 6M, GovernmentBond, ModifiedFollowing, EOM + first_date.
    // TODO: port once GovernmentBond calendar is available.
}

// ─────── testNextToLastWithEomAdjustment ───────

#[test]
#[ignore = "requires UnitedStates::GovernmentBond calendar (not yet ported)"]
fn test_next_to_last_with_eom_adjustment() {
    // Backward, 6M, GovernmentBond, ModifiedFollowing, EOM + next_to_last.
    // TODO: port once GovernmentBond calendar is available.
}

// ──── testEffectiveDateWithEomAdjustment ─────

#[test]
fn test_effective_date_with_eom_adjustment() {
    // Forward schedule with EOM adjustment and effective date and first
    // date in the same month.
    let cal = NullCalendar;
    let sched = ScheduleBuilder::new(
        date(2023, 1, 16),
        date(2023, 3, 16),
        Period::new(1, TimeUnit::Months),
        &cal,
    )
    .with_first_date(date(2023, 1, 31))
    .with_convention(BusinessDayConvention::Unadjusted)
    .with_termination_convention(BusinessDayConvention::Unadjusted)
    .with_rule(DateGeneration::Forward)
    .end_of_month(true)
    .build()
    .unwrap();

    // Check that the effective date is NOT moved to end of month.
    let expected = [
        date(2023, 1, 16),
        date(2023, 1, 31),
        date(2023, 2, 28),
        date(2023, 3, 16),
    ];
    check_dates(&sched, &expected);
}

// ──────────────── testFourWeeksTenor ─────────────────

#[test]
fn test_four_weeks_tenor() {
    // A four-weeks tenor should not cause an error.
    let cal = Target;
    let result = ScheduleBuilder::new(
        date(2016, 1, 13),
        date(2016, 5, 4),
        Period::new(4, TimeUnit::Weeks),
        &cal,
    )
    .with_convention(BusinessDayConvention::Following)
    .with_termination_convention(BusinessDayConvention::Following)
    .with_rule(DateGeneration::Forward)
    .build();
    assert!(
        result.is_ok(),
        "a four-weeks tenor caused an error: {result:?}"
    );
}

// ──────────────── testOnceFrequency ──────────────────

#[test]
fn test_once_frequency() {
    let cal = NullCalendar;
    let tenor = Period::from_frequency(Frequency::Once).unwrap();
    let sched = ScheduleBuilder::new(date(2016, 1, 13), date(2019, 1, 13), tenor, &cal)
        .with_convention(BusinessDayConvention::Unadjusted)
        .with_termination_convention(BusinessDayConvention::Unadjusted)
        .with_rule(DateGeneration::Forward)
        .build()
        .unwrap();

    assert_eq!(sched.size(), 2);
    assert_eq!(sched.date(0), date(2016, 1, 13));
    assert_eq!(sched.date(1), date(2019, 1, 13));
}

// ────────── testScheduleAlwaysHasAStartDate ──────────

#[test]
fn test_schedule_always_has_a_start_date() {
    // Variations of schedules should always have the start date as
    // the first element.
    // Using WeekendsOnly + Unadjusted (calendar is irrelevant since
    // convention is Unadjusted).
    let cal = WeekendsOnly;

    // Backward schedule with first_date
    let sched = ScheduleBuilder::new(
        date(2017, 1, 10),
        date(2026, 2, 28),
        Period::new(6, TimeUnit::Months),
        &cal,
    )
    .with_first_date(date(2017, 8, 31))
    .with_convention(BusinessDayConvention::Unadjusted)
    .with_termination_convention(BusinessDayConvention::Unadjusted)
    .with_rule(DateGeneration::Backward)
    .end_of_month(false)
    .build()
    .unwrap();
    assert_eq!(
        sched.date(0),
        date(2017, 1, 10),
        "the first element should always be the start date"
    );

    // Backward schedule without first_date
    let sched = ScheduleBuilder::new(
        date(2017, 1, 10),
        date(2026, 2, 28),
        Period::new(6, TimeUnit::Months),
        &cal,
    )
    .with_convention(BusinessDayConvention::Unadjusted)
    .with_termination_convention(BusinessDayConvention::Unadjusted)
    .with_rule(DateGeneration::Backward)
    .end_of_month(false)
    .build()
    .unwrap();
    assert_eq!(
        sched.date(0),
        date(2017, 1, 10),
        "the first element should always be the start date"
    );

    // Backward schedule where start == first expected generated date
    let sched = ScheduleBuilder::new(
        date(2017, 8, 31),
        date(2026, 2, 28),
        Period::new(6, TimeUnit::Months),
        &cal,
    )
    .with_convention(BusinessDayConvention::Unadjusted)
    .with_termination_convention(BusinessDayConvention::Unadjusted)
    .with_rule(DateGeneration::Backward)
    .end_of_month(false)
    .build()
    .unwrap();
    assert_eq!(
        sched.date(0),
        date(2017, 8, 31),
        "the first element should always be the start date"
    );
}

// ──────────────── testShortEomSchedule ───────────────

#[test]
fn test_short_eom_schedule() {
    let cal = Target;
    let sched = ScheduleBuilder::new(
        date(2019, 2, 21),
        date(2019, 2, 28),
        Period::new(1, TimeUnit::Years),
        &cal,
    )
    .with_convention(BusinessDayConvention::ModifiedFollowing)
    .with_termination_convention(BusinessDayConvention::ModifiedFollowing)
    .with_rule(DateGeneration::Backward)
    .end_of_month(true)
    .build()
    .unwrap();

    assert_eq!(sched.size(), 2);
    assert_eq!(sched.date(0), date(2019, 2, 21));
    assert_eq!(sched.date(1), date(2019, 2, 28));
}

// ──────────── testFirstDateOnMaturity ────────────────

#[test]
fn test_first_date_on_maturity() {
    // When the first date equals the maturity date, the schedule
    // should contain just start and end.
    // Using WeekendsOnly + Unadjusted (original uses GovernmentBond).
    let cal = WeekendsOnly;

    // Backward
    let sched = ScheduleBuilder::new(
        date(2016, 9, 20),
        date(2016, 12, 20),
        Period::new(3, TimeUnit::Months),
        &cal,
    )
    .with_first_date(date(2016, 12, 20))
    .with_convention(BusinessDayConvention::Unadjusted)
    .with_termination_convention(BusinessDayConvention::Unadjusted)
    .with_rule(DateGeneration::Backward)
    .build()
    .unwrap();

    let expected = [date(2016, 9, 20), date(2016, 12, 20)];
    check_dates(&sched, &expected);

    // Forward
    let sched = ScheduleBuilder::new(
        date(2016, 9, 20),
        date(2016, 12, 20),
        Period::new(3, TimeUnit::Months),
        &cal,
    )
    .with_first_date(date(2016, 12, 20))
    .with_convention(BusinessDayConvention::Unadjusted)
    .with_termination_convention(BusinessDayConvention::Unadjusted)
    .with_rule(DateGeneration::Forward)
    .build()
    .unwrap();

    check_dates(&sched, &expected);
}

// ─────────── testNextToLastDateOnStart ───────────────

#[test]
fn test_next_to_last_date_on_start() {
    // When the next-to-last date equals the start date, the schedule
    // should contain just start and end.
    let cal = WeekendsOnly;

    let sched = ScheduleBuilder::new(
        date(2016, 9, 20),
        date(2016, 12, 20),
        Period::new(3, TimeUnit::Months),
        &cal,
    )
    .with_next_to_last_date(date(2016, 9, 20))
    .with_convention(BusinessDayConvention::Unadjusted)
    .with_termination_convention(BusinessDayConvention::Unadjusted)
    .with_rule(DateGeneration::Backward)
    .build()
    .unwrap();

    let expected = [date(2016, 9, 20), date(2016, 12, 20)];
    check_dates(&sched, &expected);
}

// ─────────────── testDateConstructor ─────────────────

#[test]
fn test_date_constructor() {
    use ql_time::Schedule;

    let dates = vec![
        date(2015, 5, 16),
        date(2015, 5, 18),
        date(2016, 5, 18),
        date(2017, 12, 31),
    ];

    // Schedule from explicit dates without metadata
    let schedule1 = Schedule::from_dates(dates.clone());
    assert_eq!(schedule1.size(), dates.len());
    for (i, d) in dates.iter().enumerate() {
        assert_eq!(schedule1.date(i), *d);
    }

    // Schedule from explicit dates with regularity flags
    let regular = vec![false, true, false];
    let schedule2 = Schedule::from_dates_with_regular(dates.clone(), regular.clone());
    for (i, &r) in regular.iter().enumerate() {
        // is_regular is 1-indexed per period in C++; our Rust API uses 0-indexed.
        assert_eq!(
            schedule2.is_regular(i),
            r,
            "period {} regularity mismatch",
            i
        );
    }
}

// ─────────────── testTruncation ──────────────────────

#[test]
fn test_truncation() {
    let cal = Japan;
    let sched = ScheduleBuilder::new(
        date(2009, 9, 30),
        date(2020, 6, 15),
        Period::new(6, TimeUnit::Months),
        &cal,
    )
    .with_convention(BusinessDayConvention::ModifiedFollowing)
    .with_termination_convention(BusinessDayConvention::ModifiedFollowing)
    .with_rule(DateGeneration::Forward)
    .end_of_month(true)
    .build()
    .unwrap();

    // ── Until ──
    {
        let t = sched.until(date(2014, 1, 1));
        let expected = [
            date(2009, 9, 30),
            date(2010, 3, 31),
            date(2010, 9, 30),
            date(2011, 3, 31),
            date(2011, 9, 30),
            date(2012, 3, 30),
            date(2012, 9, 28),
            date(2013, 3, 29),
            date(2013, 9, 30),
            date(2014, 1, 1),
        ];
        check_dates(&t, &expected);
        assert!(
            !*t.is_regular_vec().last().unwrap(),
            "last period of until() with non-schedule date should be irregular"
        );
    }

    // ── Until, truncation date on a schedule date ──
    {
        let t = sched.until(date(2013, 9, 30));
        let expected = [
            date(2009, 9, 30),
            date(2010, 3, 31),
            date(2010, 9, 30),
            date(2011, 3, 31),
            date(2011, 9, 30),
            date(2012, 3, 30),
            date(2012, 9, 28),
            date(2013, 3, 29),
            date(2013, 9, 30),
        ];
        check_dates(&t, &expected);
        assert!(
            *t.is_regular_vec().last().unwrap(),
            "last period of until() on schedule date should be regular"
        );
    }

    // ── After ──
    {
        let t = sched.after(date(2014, 1, 1));
        let expected = [
            date(2014, 1, 1),
            date(2014, 3, 31),
            date(2014, 9, 30),
            date(2015, 3, 31),
            date(2015, 9, 30),
            date(2016, 3, 31),
            date(2016, 9, 30),
            date(2017, 3, 31),
            date(2017, 9, 29),
            date(2018, 3, 30),
            date(2018, 9, 28),
            date(2019, 3, 29),
            date(2019, 9, 30),
            date(2020, 3, 31),
            date(2020, 6, 15),
        ];
        check_dates(&t, &expected);
        assert!(
            !*t.is_regular_vec().first().unwrap(),
            "first period of after() with non-schedule date should be irregular"
        );
    }

    // ── After, truncation date on a schedule date ──
    {
        let t = sched.after(date(2018, 9, 28));
        let expected = [
            date(2018, 9, 28),
            date(2019, 3, 29),
            date(2019, 9, 30),
            date(2020, 3, 31),
            date(2020, 6, 15),
        ];
        check_dates(&t, &expected);
        assert!(
            *t.is_regular_vec().first().unwrap(),
            "first period of after() on schedule date should be regular"
        );
    }
}
