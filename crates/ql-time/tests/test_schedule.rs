//! Tests ported from QuantLib `test-suite/schedule.cpp`.
//!
//! These integration tests exercise the `Schedule`, `ScheduleBuilder`, and
//! `DateGeneration` types.

use ql_time::calendar::{NullCalendar, WeekendsOnly};
use ql_time::calendars::japan::Japan;
use ql_time::calendars::target::Target;
use ql_time::calendars::united_states::UnitedStatesGovernmentBond;
use ql_time::schedule::{cds_maturity, DateGeneration, ScheduleBuilder};
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

#[test]
fn test_double_first_date_with_eom_adjustment() {
    // Backward, 6M, GovernmentBond, ModifiedFollowing + Following, EOM.
    // The first date should not be duplicated due to EOM convention.
    let cal = UnitedStatesGovernmentBond;
    let sched = ScheduleBuilder::new(
        date(1996, 8, 22),
        date(1997, 8, 31),
        Period::new(6, TimeUnit::Months),
        &cal,
    )
    .with_convention(BusinessDayConvention::ModifiedFollowing)
    .with_termination_convention(BusinessDayConvention::Following)
    .with_rule(DateGeneration::Backward)
    .end_of_month(true)
    .build()
    .unwrap();

    let expected = [
        date(1996, 8, 22),
        date(1996, 8, 30),
        date(1997, 2, 28),
        date(1997, 9, 2),
    ];
    check_dates(&sched, &expected);
}

// ─────── testFirstDateWithEomAdjustment ───────

#[test]
fn test_first_date_with_eom_adjustment() {
    // Forward, 6M, GovernmentBond, ModifiedFollowing, EOM + first_date.
    let cal = UnitedStatesGovernmentBond;
    let sched = ScheduleBuilder::new(
        date(1996, 8, 10),
        date(1998, 8, 10),
        Period::new(6, TimeUnit::Months),
        &cal,
    )
    .with_first_date(date(1997, 2, 28))
    .with_convention(BusinessDayConvention::ModifiedFollowing)
    .with_termination_convention(BusinessDayConvention::ModifiedFollowing)
    .with_rule(DateGeneration::Forward)
    .end_of_month(true)
    .build()
    .unwrap();

    let expected = [
        date(1996, 8, 12),
        date(1997, 2, 28),
        date(1997, 8, 29),
        date(1998, 2, 27),
        date(1998, 8, 10),
    ];
    check_dates(&sched, &expected);
}

// ─────── testNextToLastWithEomAdjustment ───────

#[test]
fn test_next_to_last_with_eom_adjustment() {
    // Backward, 6M, GovernmentBond, ModifiedFollowing, EOM + next_to_last.
    let cal = UnitedStatesGovernmentBond;
    let sched = ScheduleBuilder::new(
        date(1996, 8, 10),
        date(1998, 8, 10),
        Period::new(6, TimeUnit::Months),
        &cal,
    )
    .with_next_to_last_date(date(1998, 2, 28))
    .with_convention(BusinessDayConvention::ModifiedFollowing)
    .with_termination_convention(BusinessDayConvention::ModifiedFollowing)
    .with_rule(DateGeneration::Backward)
    .end_of_month(true)
    .build()
    .unwrap();

    let expected = [
        date(1996, 8, 12),
        date(1996, 8, 30),
        date(1997, 2, 28),
        date(1997, 8, 29),
        date(1998, 2, 27),
        date(1998, 8, 10),
    ];
    check_dates(&sched, &expected);
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

// ═══════════════════════════════════════════════════════════════════════════
// CDS schedule tests
// ═══════════════════════════════════════════════════════════════════════════

/// Helper: build a CDS schedule just like the C++ `makeCdsSchedule`.
fn make_cds_schedule(from: Date, to: Date, rule: DateGeneration) -> ql_time::Schedule {
    let cal = WeekendsOnly;
    ScheduleBuilder::new(from, to, Period::new(3, TimeUnit::Months), &cal)
        .with_convention(BusinessDayConvention::Following)
        .with_termination_convention(BusinessDayConvention::Unadjusted)
        .with_rule(rule)
        .build()
        .unwrap()
}

/// Helper: run the CDS convention grid test for a given rule.
/// Each entry maps (trade_date, tenor) → (expected_start, expected_end).
fn test_cds_conventions(inputs: &[((Date, Period), (Date, Date))], rule: DateGeneration) {
    for &((from, ref tenor), (exp_start, exp_end)) in inputs {
        let maturity = cds_maturity(from, tenor, rule)
            .unwrap_or_else(|e| panic!("cds_maturity failed for {from}, {tenor:?}: {e}"))
            .unwrap_or_else(|| panic!("cds_maturity returned None for {from}, {tenor:?}"));
        assert_eq!(
            maturity, exp_end,
            "maturity mismatch for from={from}, tenor={tenor:?}"
        );

        let s = make_cds_schedule(from, maturity, rule);
        assert_eq!(
            s.start_date().unwrap(),
            exp_start,
            "start date mismatch for from={from}, tenor={tenor:?}"
        );
        assert_eq!(
            s.end_date().unwrap(),
            exp_end,
            "end date mismatch for from={from}, tenor={tenor:?}"
        );
    }
}

// ──────────────── testCDS2015Convention ───────────────

#[test]
fn test_cds2015_convention() {
    let rule = DateGeneration::CDS2015;
    let tenor = Period::new(5, TimeUnit::Years);

    // Trade date Dec 12, 2016
    let trade_date = date(2016, 12, 12);
    let maturity = cds_maturity(trade_date, &tenor, rule).unwrap().unwrap();
    let exp_start = date(2016, 9, 20);
    let exp_maturity = date(2021, 12, 20);
    assert_eq!(maturity, exp_maturity);

    let s = make_cds_schedule(trade_date, maturity, rule);
    assert_eq!(s.start_date().unwrap(), exp_start);
    assert_eq!(s.end_date().unwrap(), exp_maturity);

    // Using trade_date + 5Y as termination directly
    let raw_mat = trade_date.advance(tenor.length, tenor.unit).unwrap();
    let s = make_cds_schedule(trade_date, raw_mat, rule);
    assert_eq!(s.start_date().unwrap(), exp_start);
    assert_eq!(s.end_date().unwrap(), exp_maturity);

    // Trade date = 1 Mar 2017
    let trade_date = date(2017, 3, 1);
    let maturity = cds_maturity(trade_date, &tenor, rule).unwrap().unwrap();
    assert_eq!(maturity, exp_maturity); // same maturity
    let s = make_cds_schedule(trade_date, maturity, rule);
    assert_eq!(s.start_date().unwrap(), date(2016, 12, 20));
    assert_eq!(s.end_date().unwrap(), exp_maturity);

    // Using raw maturity = 1 Mar 2022
    let raw_mat = trade_date.advance(tenor.length, tenor.unit).unwrap();
    let s = make_cds_schedule(trade_date, raw_mat, rule);
    assert_eq!(s.start_date().unwrap(), date(2016, 12, 20));
    assert_eq!(s.end_date().unwrap(), date(2022, 3, 20));

    // Trade date = 20 Mar 2017
    let trade_date = date(2017, 3, 20);
    let maturity = cds_maturity(trade_date, &tenor, rule).unwrap().unwrap();
    assert_eq!(maturity, date(2022, 6, 20));
    let s = make_cds_schedule(trade_date, maturity, rule);
    assert_eq!(s.start_date().unwrap(), date(2017, 3, 20));
    assert_eq!(s.end_date().unwrap(), date(2022, 6, 20));
}

// ──────────────── testCDS2015ConventionGrid ──────────

#[test]
fn test_cds2015_convention_grid() {
    let m3 = Period::new(3, TimeUnit::Months);
    let m6 = Period::new(6, TimeUnit::Months);
    let m9 = Period::new(9, TimeUnit::Months);
    let y1 = Period::new(1, TimeUnit::Years);
    let y5 = Period::new(5, TimeUnit::Years);
    let m0 = Period::new(0, TimeUnit::Months);

    let inputs: Vec<((Date, Period), (Date, Date))> = vec![
        // 3M
        (
            (date(2016, 3, 19), m3),
            (date(2015, 12, 21), date(2016, 3, 20)),
        ),
        (
            (date(2016, 3, 20), m3),
            (date(2015, 12, 21), date(2016, 9, 20)),
        ),
        (
            (date(2016, 3, 21), m3),
            (date(2016, 3, 21), date(2016, 9, 20)),
        ),
        (
            (date(2016, 6, 19), m3),
            (date(2016, 3, 21), date(2016, 9, 20)),
        ),
        (
            (date(2016, 6, 20), m3),
            (date(2016, 6, 20), date(2016, 9, 20)),
        ),
        (
            (date(2016, 6, 21), m3),
            (date(2016, 6, 20), date(2016, 9, 20)),
        ),
        (
            (date(2016, 9, 19), m3),
            (date(2016, 6, 20), date(2016, 9, 20)),
        ),
        (
            (date(2016, 9, 20), m3),
            (date(2016, 9, 20), date(2017, 3, 20)),
        ),
        (
            (date(2016, 9, 21), m3),
            (date(2016, 9, 20), date(2017, 3, 20)),
        ),
        (
            (date(2016, 12, 19), m3),
            (date(2016, 9, 20), date(2017, 3, 20)),
        ),
        (
            (date(2016, 12, 20), m3),
            (date(2016, 12, 20), date(2017, 3, 20)),
        ),
        (
            (date(2016, 12, 21), m3),
            (date(2016, 12, 20), date(2017, 3, 20)),
        ),
        // 6M
        (
            (date(2016, 3, 19), m6),
            (date(2015, 12, 21), date(2016, 6, 20)),
        ),
        (
            (date(2016, 3, 20), m6),
            (date(2015, 12, 21), date(2016, 12, 20)),
        ),
        (
            (date(2016, 3, 21), m6),
            (date(2016, 3, 21), date(2016, 12, 20)),
        ),
        (
            (date(2016, 6, 19), m6),
            (date(2016, 3, 21), date(2016, 12, 20)),
        ),
        (
            (date(2016, 6, 20), m6),
            (date(2016, 6, 20), date(2016, 12, 20)),
        ),
        (
            (date(2016, 6, 21), m6),
            (date(2016, 6, 20), date(2016, 12, 20)),
        ),
        (
            (date(2016, 9, 19), m6),
            (date(2016, 6, 20), date(2016, 12, 20)),
        ),
        (
            (date(2016, 9, 20), m6),
            (date(2016, 9, 20), date(2017, 6, 20)),
        ),
        (
            (date(2016, 9, 21), m6),
            (date(2016, 9, 20), date(2017, 6, 20)),
        ),
        (
            (date(2016, 12, 19), m6),
            (date(2016, 9, 20), date(2017, 6, 20)),
        ),
        (
            (date(2016, 12, 20), m6),
            (date(2016, 12, 20), date(2017, 6, 20)),
        ),
        (
            (date(2016, 12, 21), m6),
            (date(2016, 12, 20), date(2017, 6, 20)),
        ),
        // 9M
        (
            (date(2016, 3, 19), m9),
            (date(2015, 12, 21), date(2016, 9, 20)),
        ),
        (
            (date(2016, 3, 20), m9),
            (date(2015, 12, 21), date(2017, 3, 20)),
        ),
        (
            (date(2016, 3, 21), m9),
            (date(2016, 3, 21), date(2017, 3, 20)),
        ),
        (
            (date(2016, 6, 19), m9),
            (date(2016, 3, 21), date(2017, 3, 20)),
        ),
        (
            (date(2016, 6, 20), m9),
            (date(2016, 6, 20), date(2017, 3, 20)),
        ),
        (
            (date(2016, 6, 21), m9),
            (date(2016, 6, 20), date(2017, 3, 20)),
        ),
        (
            (date(2016, 9, 19), m9),
            (date(2016, 6, 20), date(2017, 3, 20)),
        ),
        (
            (date(2016, 9, 20), m9),
            (date(2016, 9, 20), date(2017, 9, 20)),
        ),
        (
            (date(2016, 9, 21), m9),
            (date(2016, 9, 20), date(2017, 9, 20)),
        ),
        (
            (date(2016, 12, 19), m9),
            (date(2016, 9, 20), date(2017, 9, 20)),
        ),
        (
            (date(2016, 12, 20), m9),
            (date(2016, 12, 20), date(2017, 9, 20)),
        ),
        (
            (date(2016, 12, 21), m9),
            (date(2016, 12, 20), date(2017, 9, 20)),
        ),
        // 1Y
        (
            (date(2016, 3, 19), y1),
            (date(2015, 12, 21), date(2016, 12, 20)),
        ),
        (
            (date(2016, 3, 20), y1),
            (date(2015, 12, 21), date(2017, 6, 20)),
        ),
        (
            (date(2016, 3, 21), y1),
            (date(2016, 3, 21), date(2017, 6, 20)),
        ),
        (
            (date(2016, 6, 19), y1),
            (date(2016, 3, 21), date(2017, 6, 20)),
        ),
        (
            (date(2016, 6, 20), y1),
            (date(2016, 6, 20), date(2017, 6, 20)),
        ),
        (
            (date(2016, 6, 21), y1),
            (date(2016, 6, 20), date(2017, 6, 20)),
        ),
        (
            (date(2016, 9, 19), y1),
            (date(2016, 6, 20), date(2017, 6, 20)),
        ),
        (
            (date(2016, 9, 20), y1),
            (date(2016, 9, 20), date(2017, 12, 20)),
        ),
        (
            (date(2016, 9, 21), y1),
            (date(2016, 9, 20), date(2017, 12, 20)),
        ),
        (
            (date(2016, 12, 19), y1),
            (date(2016, 9, 20), date(2017, 12, 20)),
        ),
        (
            (date(2016, 12, 20), y1),
            (date(2016, 12, 20), date(2017, 12, 20)),
        ),
        (
            (date(2016, 12, 21), y1),
            (date(2016, 12, 20), date(2017, 12, 20)),
        ),
        // 5Y
        (
            (date(2016, 3, 19), y5),
            (date(2015, 12, 21), date(2020, 12, 20)),
        ),
        (
            (date(2016, 3, 20), y5),
            (date(2015, 12, 21), date(2021, 6, 20)),
        ),
        (
            (date(2016, 3, 21), y5),
            (date(2016, 3, 21), date(2021, 6, 20)),
        ),
        (
            (date(2016, 6, 19), y5),
            (date(2016, 3, 21), date(2021, 6, 20)),
        ),
        (
            (date(2016, 6, 20), y5),
            (date(2016, 6, 20), date(2021, 6, 20)),
        ),
        (
            (date(2016, 6, 21), y5),
            (date(2016, 6, 20), date(2021, 6, 20)),
        ),
        (
            (date(2016, 9, 19), y5),
            (date(2016, 6, 20), date(2021, 6, 20)),
        ),
        (
            (date(2016, 9, 20), y5),
            (date(2016, 9, 20), date(2021, 12, 20)),
        ),
        (
            (date(2016, 9, 21), y5),
            (date(2016, 9, 20), date(2021, 12, 20)),
        ),
        (
            (date(2016, 12, 19), y5),
            (date(2016, 9, 20), date(2021, 12, 20)),
        ),
        (
            (date(2016, 12, 20), y5),
            (date(2016, 12, 20), date(2021, 12, 20)),
        ),
        (
            (date(2016, 12, 21), y5),
            (date(2016, 12, 20), date(2021, 12, 20)),
        ),
        // 0M
        (
            (date(2016, 3, 20), m0),
            (date(2015, 12, 21), date(2016, 6, 20)),
        ),
        (
            (date(2016, 3, 21), m0),
            (date(2016, 3, 21), date(2016, 6, 20)),
        ),
        (
            (date(2016, 6, 19), m0),
            (date(2016, 3, 21), date(2016, 6, 20)),
        ),
        (
            (date(2016, 9, 20), m0),
            (date(2016, 9, 20), date(2016, 12, 20)),
        ),
        (
            (date(2016, 9, 21), m0),
            (date(2016, 9, 20), date(2016, 12, 20)),
        ),
        (
            (date(2016, 12, 19), m0),
            (date(2016, 9, 20), date(2016, 12, 20)),
        ),
    ];

    test_cds_conventions(&inputs, DateGeneration::CDS2015);
}

// ──────────────── testCDSConventionGrid ──────────────

#[test]
fn test_cds_convention_grid() {
    let m3 = Period::new(3, TimeUnit::Months);
    let m6 = Period::new(6, TimeUnit::Months);
    let m9 = Period::new(9, TimeUnit::Months);
    let y1 = Period::new(1, TimeUnit::Years);
    let y5 = Period::new(5, TimeUnit::Years);
    let m0 = Period::new(0, TimeUnit::Months);

    let inputs: Vec<((Date, Period), (Date, Date))> = vec![
        // 3M
        (
            (date(2016, 3, 19), m3),
            (date(2015, 12, 21), date(2016, 6, 20)),
        ),
        (
            (date(2016, 3, 20), m3),
            (date(2015, 12, 21), date(2016, 9, 20)),
        ),
        (
            (date(2016, 3, 21), m3),
            (date(2016, 3, 21), date(2016, 9, 20)),
        ),
        (
            (date(2016, 6, 19), m3),
            (date(2016, 3, 21), date(2016, 9, 20)),
        ),
        (
            (date(2016, 6, 20), m3),
            (date(2016, 6, 20), date(2016, 12, 20)),
        ),
        (
            (date(2016, 6, 21), m3),
            (date(2016, 6, 20), date(2016, 12, 20)),
        ),
        (
            (date(2016, 9, 19), m3),
            (date(2016, 6, 20), date(2016, 12, 20)),
        ),
        (
            (date(2016, 9, 20), m3),
            (date(2016, 9, 20), date(2017, 3, 20)),
        ),
        (
            (date(2016, 9, 21), m3),
            (date(2016, 9, 20), date(2017, 3, 20)),
        ),
        (
            (date(2016, 12, 19), m3),
            (date(2016, 9, 20), date(2017, 3, 20)),
        ),
        (
            (date(2016, 12, 20), m3),
            (date(2016, 12, 20), date(2017, 6, 20)),
        ),
        (
            (date(2016, 12, 21), m3),
            (date(2016, 12, 20), date(2017, 6, 20)),
        ),
        // 6M
        (
            (date(2016, 3, 19), m6),
            (date(2015, 12, 21), date(2016, 9, 20)),
        ),
        (
            (date(2016, 3, 20), m6),
            (date(2015, 12, 21), date(2016, 12, 20)),
        ),
        (
            (date(2016, 3, 21), m6),
            (date(2016, 3, 21), date(2016, 12, 20)),
        ),
        (
            (date(2016, 6, 19), m6),
            (date(2016, 3, 21), date(2016, 12, 20)),
        ),
        (
            (date(2016, 6, 20), m6),
            (date(2016, 6, 20), date(2017, 3, 20)),
        ),
        (
            (date(2016, 6, 21), m6),
            (date(2016, 6, 20), date(2017, 3, 20)),
        ),
        (
            (date(2016, 9, 19), m6),
            (date(2016, 6, 20), date(2017, 3, 20)),
        ),
        (
            (date(2016, 9, 20), m6),
            (date(2016, 9, 20), date(2017, 6, 20)),
        ),
        (
            (date(2016, 9, 21), m6),
            (date(2016, 9, 20), date(2017, 6, 20)),
        ),
        (
            (date(2016, 12, 19), m6),
            (date(2016, 9, 20), date(2017, 6, 20)),
        ),
        (
            (date(2016, 12, 20), m6),
            (date(2016, 12, 20), date(2017, 9, 20)),
        ),
        (
            (date(2016, 12, 21), m6),
            (date(2016, 12, 20), date(2017, 9, 20)),
        ),
        // 9M
        (
            (date(2016, 3, 19), m9),
            (date(2015, 12, 21), date(2016, 12, 20)),
        ),
        (
            (date(2016, 3, 20), m9),
            (date(2015, 12, 21), date(2017, 3, 20)),
        ),
        (
            (date(2016, 3, 21), m9),
            (date(2016, 3, 21), date(2017, 3, 20)),
        ),
        (
            (date(2016, 6, 19), m9),
            (date(2016, 3, 21), date(2017, 3, 20)),
        ),
        (
            (date(2016, 6, 20), m9),
            (date(2016, 6, 20), date(2017, 6, 20)),
        ),
        (
            (date(2016, 6, 21), m9),
            (date(2016, 6, 20), date(2017, 6, 20)),
        ),
        (
            (date(2016, 9, 19), m9),
            (date(2016, 6, 20), date(2017, 6, 20)),
        ),
        (
            (date(2016, 9, 20), m9),
            (date(2016, 9, 20), date(2017, 9, 20)),
        ),
        (
            (date(2016, 9, 21), m9),
            (date(2016, 9, 20), date(2017, 9, 20)),
        ),
        (
            (date(2016, 12, 19), m9),
            (date(2016, 9, 20), date(2017, 9, 20)),
        ),
        (
            (date(2016, 12, 20), m9),
            (date(2016, 12, 20), date(2017, 12, 20)),
        ),
        (
            (date(2016, 12, 21), m9),
            (date(2016, 12, 20), date(2017, 12, 20)),
        ),
        // 1Y
        (
            (date(2016, 3, 19), y1),
            (date(2015, 12, 21), date(2017, 3, 20)),
        ),
        (
            (date(2016, 3, 20), y1),
            (date(2015, 12, 21), date(2017, 6, 20)),
        ),
        (
            (date(2016, 3, 21), y1),
            (date(2016, 3, 21), date(2017, 6, 20)),
        ),
        (
            (date(2016, 6, 19), y1),
            (date(2016, 3, 21), date(2017, 6, 20)),
        ),
        (
            (date(2016, 6, 20), y1),
            (date(2016, 6, 20), date(2017, 9, 20)),
        ),
        (
            (date(2016, 6, 21), y1),
            (date(2016, 6, 20), date(2017, 9, 20)),
        ),
        (
            (date(2016, 9, 19), y1),
            (date(2016, 6, 20), date(2017, 9, 20)),
        ),
        (
            (date(2016, 9, 20), y1),
            (date(2016, 9, 20), date(2017, 12, 20)),
        ),
        (
            (date(2016, 9, 21), y1),
            (date(2016, 9, 20), date(2017, 12, 20)),
        ),
        (
            (date(2016, 12, 19), y1),
            (date(2016, 9, 20), date(2017, 12, 20)),
        ),
        (
            (date(2016, 12, 20), y1),
            (date(2016, 12, 20), date(2018, 3, 20)),
        ),
        (
            (date(2016, 12, 21), y1),
            (date(2016, 12, 20), date(2018, 3, 20)),
        ),
        // 5Y
        (
            (date(2016, 3, 19), y5),
            (date(2015, 12, 21), date(2021, 3, 20)),
        ),
        (
            (date(2016, 3, 20), y5),
            (date(2015, 12, 21), date(2021, 6, 20)),
        ),
        (
            (date(2016, 3, 21), y5),
            (date(2016, 3, 21), date(2021, 6, 20)),
        ),
        (
            (date(2016, 6, 19), y5),
            (date(2016, 3, 21), date(2021, 6, 20)),
        ),
        (
            (date(2016, 6, 20), y5),
            (date(2016, 6, 20), date(2021, 9, 20)),
        ),
        (
            (date(2016, 6, 21), y5),
            (date(2016, 6, 20), date(2021, 9, 20)),
        ),
        (
            (date(2016, 9, 19), y5),
            (date(2016, 6, 20), date(2021, 9, 20)),
        ),
        (
            (date(2016, 9, 20), y5),
            (date(2016, 9, 20), date(2021, 12, 20)),
        ),
        (
            (date(2016, 9, 21), y5),
            (date(2016, 9, 20), date(2021, 12, 20)),
        ),
        (
            (date(2016, 12, 19), y5),
            (date(2016, 9, 20), date(2021, 12, 20)),
        ),
        (
            (date(2016, 12, 20), y5),
            (date(2016, 12, 20), date(2022, 3, 20)),
        ),
        (
            (date(2016, 12, 21), y5),
            (date(2016, 12, 20), date(2022, 3, 20)),
        ),
        // 0M
        (
            (date(2016, 3, 19), m0),
            (date(2015, 12, 21), date(2016, 3, 20)),
        ),
        (
            (date(2016, 3, 20), m0),
            (date(2015, 12, 21), date(2016, 6, 20)),
        ),
        (
            (date(2016, 3, 21), m0),
            (date(2016, 3, 21), date(2016, 6, 20)),
        ),
        (
            (date(2016, 6, 19), m0),
            (date(2016, 3, 21), date(2016, 6, 20)),
        ),
        (
            (date(2016, 6, 20), m0),
            (date(2016, 6, 20), date(2016, 9, 20)),
        ),
        (
            (date(2016, 6, 21), m0),
            (date(2016, 6, 20), date(2016, 9, 20)),
        ),
        (
            (date(2016, 9, 19), m0),
            (date(2016, 6, 20), date(2016, 9, 20)),
        ),
        (
            (date(2016, 9, 20), m0),
            (date(2016, 9, 20), date(2016, 12, 20)),
        ),
        (
            (date(2016, 9, 21), m0),
            (date(2016, 9, 20), date(2016, 12, 20)),
        ),
        (
            (date(2016, 12, 19), m0),
            (date(2016, 9, 20), date(2016, 12, 20)),
        ),
        (
            (date(2016, 12, 20), m0),
            (date(2016, 12, 20), date(2017, 3, 20)),
        ),
        (
            (date(2016, 12, 21), m0),
            (date(2016, 12, 20), date(2017, 3, 20)),
        ),
    ];

    test_cds_conventions(&inputs, DateGeneration::CDS);
}

// ──────────────── testOldCDSConventionGrid ───────────

#[test]
fn test_old_cds_convention_grid() {
    let m3 = Period::new(3, TimeUnit::Months);
    let m6 = Period::new(6, TimeUnit::Months);
    let m9 = Period::new(9, TimeUnit::Months);
    let y1 = Period::new(1, TimeUnit::Years);
    let y5 = Period::new(5, TimeUnit::Years);

    let inputs: Vec<((Date, Period), (Date, Date))> = vec![
        // 3M
        (
            (date(2016, 3, 19), m3),
            (date(2016, 3, 19), date(2016, 6, 20)),
        ),
        (
            (date(2016, 3, 20), m3),
            (date(2016, 3, 20), date(2016, 9, 20)),
        ),
        (
            (date(2016, 3, 21), m3),
            (date(2016, 3, 21), date(2016, 9, 20)),
        ),
        (
            (date(2016, 6, 19), m3),
            (date(2016, 6, 19), date(2016, 9, 20)),
        ),
        (
            (date(2016, 6, 20), m3),
            (date(2016, 6, 20), date(2016, 12, 20)),
        ),
        (
            (date(2016, 6, 21), m3),
            (date(2016, 6, 21), date(2016, 12, 20)),
        ),
        (
            (date(2016, 9, 19), m3),
            (date(2016, 9, 19), date(2016, 12, 20)),
        ),
        (
            (date(2016, 9, 20), m3),
            (date(2016, 9, 20), date(2017, 3, 20)),
        ),
        (
            (date(2016, 9, 21), m3),
            (date(2016, 9, 21), date(2017, 3, 20)),
        ),
        (
            (date(2016, 12, 19), m3),
            (date(2016, 12, 19), date(2017, 3, 20)),
        ),
        (
            (date(2016, 12, 20), m3),
            (date(2016, 12, 20), date(2017, 6, 20)),
        ),
        (
            (date(2016, 12, 21), m3),
            (date(2016, 12, 21), date(2017, 6, 20)),
        ),
        // 6M
        (
            (date(2016, 3, 19), m6),
            (date(2016, 3, 19), date(2016, 9, 20)),
        ),
        (
            (date(2016, 3, 20), m6),
            (date(2016, 3, 20), date(2016, 12, 20)),
        ),
        (
            (date(2016, 3, 21), m6),
            (date(2016, 3, 21), date(2016, 12, 20)),
        ),
        (
            (date(2016, 6, 19), m6),
            (date(2016, 6, 19), date(2016, 12, 20)),
        ),
        (
            (date(2016, 6, 20), m6),
            (date(2016, 6, 20), date(2017, 3, 20)),
        ),
        (
            (date(2016, 6, 21), m6),
            (date(2016, 6, 21), date(2017, 3, 20)),
        ),
        (
            (date(2016, 9, 19), m6),
            (date(2016, 9, 19), date(2017, 3, 20)),
        ),
        (
            (date(2016, 9, 20), m6),
            (date(2016, 9, 20), date(2017, 6, 20)),
        ),
        (
            (date(2016, 9, 21), m6),
            (date(2016, 9, 21), date(2017, 6, 20)),
        ),
        (
            (date(2016, 12, 19), m6),
            (date(2016, 12, 19), date(2017, 6, 20)),
        ),
        (
            (date(2016, 12, 20), m6),
            (date(2016, 12, 20), date(2017, 9, 20)),
        ),
        (
            (date(2016, 12, 21), m6),
            (date(2016, 12, 21), date(2017, 9, 20)),
        ),
        // 9M
        (
            (date(2016, 3, 19), m9),
            (date(2016, 3, 19), date(2016, 12, 20)),
        ),
        (
            (date(2016, 3, 20), m9),
            (date(2016, 3, 20), date(2017, 3, 20)),
        ),
        (
            (date(2016, 3, 21), m9),
            (date(2016, 3, 21), date(2017, 3, 20)),
        ),
        (
            (date(2016, 6, 19), m9),
            (date(2016, 6, 19), date(2017, 3, 20)),
        ),
        (
            (date(2016, 6, 20), m9),
            (date(2016, 6, 20), date(2017, 6, 20)),
        ),
        (
            (date(2016, 6, 21), m9),
            (date(2016, 6, 21), date(2017, 6, 20)),
        ),
        (
            (date(2016, 9, 19), m9),
            (date(2016, 9, 19), date(2017, 6, 20)),
        ),
        (
            (date(2016, 9, 20), m9),
            (date(2016, 9, 20), date(2017, 9, 20)),
        ),
        (
            (date(2016, 9, 21), m9),
            (date(2016, 9, 21), date(2017, 9, 20)),
        ),
        (
            (date(2016, 12, 19), m9),
            (date(2016, 12, 19), date(2017, 9, 20)),
        ),
        (
            (date(2016, 12, 20), m9),
            (date(2016, 12, 20), date(2017, 12, 20)),
        ),
        (
            (date(2016, 12, 21), m9),
            (date(2016, 12, 21), date(2017, 12, 20)),
        ),
        // 1Y
        (
            (date(2016, 3, 19), y1),
            (date(2016, 3, 19), date(2017, 3, 20)),
        ),
        (
            (date(2016, 3, 20), y1),
            (date(2016, 3, 20), date(2017, 6, 20)),
        ),
        (
            (date(2016, 3, 21), y1),
            (date(2016, 3, 21), date(2017, 6, 20)),
        ),
        (
            (date(2016, 6, 19), y1),
            (date(2016, 6, 19), date(2017, 6, 20)),
        ),
        (
            (date(2016, 6, 20), y1),
            (date(2016, 6, 20), date(2017, 9, 20)),
        ),
        (
            (date(2016, 6, 21), y1),
            (date(2016, 6, 21), date(2017, 9, 20)),
        ),
        (
            (date(2016, 9, 19), y1),
            (date(2016, 9, 19), date(2017, 9, 20)),
        ),
        (
            (date(2016, 9, 20), y1),
            (date(2016, 9, 20), date(2017, 12, 20)),
        ),
        (
            (date(2016, 9, 21), y1),
            (date(2016, 9, 21), date(2017, 12, 20)),
        ),
        (
            (date(2016, 12, 19), y1),
            (date(2016, 12, 19), date(2017, 12, 20)),
        ),
        (
            (date(2016, 12, 20), y1),
            (date(2016, 12, 20), date(2018, 3, 20)),
        ),
        (
            (date(2016, 12, 21), y1),
            (date(2016, 12, 21), date(2018, 3, 20)),
        ),
        // 5Y
        (
            (date(2016, 3, 19), y5),
            (date(2016, 3, 19), date(2021, 3, 20)),
        ),
        (
            (date(2016, 3, 20), y5),
            (date(2016, 3, 20), date(2021, 6, 20)),
        ),
        (
            (date(2016, 3, 21), y5),
            (date(2016, 3, 21), date(2021, 6, 20)),
        ),
        (
            (date(2016, 6, 19), y5),
            (date(2016, 6, 19), date(2021, 6, 20)),
        ),
        (
            (date(2016, 6, 20), y5),
            (date(2016, 6, 20), date(2021, 9, 20)),
        ),
        (
            (date(2016, 6, 21), y5),
            (date(2016, 6, 21), date(2021, 9, 20)),
        ),
        (
            (date(2016, 9, 19), y5),
            (date(2016, 9, 19), date(2021, 9, 20)),
        ),
        (
            (date(2016, 9, 20), y5),
            (date(2016, 9, 20), date(2021, 12, 20)),
        ),
        (
            (date(2016, 9, 21), y5),
            (date(2016, 9, 21), date(2021, 12, 20)),
        ),
        (
            (date(2016, 12, 19), y5),
            (date(2016, 12, 19), date(2021, 12, 20)),
        ),
        (
            (date(2016, 12, 20), y5),
            (date(2016, 12, 20), date(2022, 3, 20)),
        ),
        (
            (date(2016, 12, 21), y5),
            (date(2016, 12, 21), date(2022, 3, 20)),
        ),
    ];

    test_cds_conventions(&inputs, DateGeneration::OldCDS);
}

// ──────────────── testCDS2015ConventionSampleDates ───

#[test]
fn test_cds2015_convention_sample_dates() {
    let rule = DateGeneration::CDS2015;
    let tenor = Period::new(1, TimeUnit::Years);

    // Trade date = Fri 18 Sep 2015
    let mut exp = vec![
        date(2015, 6, 22),
        date(2015, 9, 21),
        date(2015, 12, 21),
        date(2016, 3, 21),
        date(2016, 6, 20),
    ];
    let maturity = cds_maturity(date(2015, 9, 18), &tenor, rule)
        .unwrap()
        .unwrap();
    let s = make_cds_schedule(date(2015, 9, 18), maturity, rule);
    check_dates(&s, &exp);

    // Trade date = Sat 19 Sep 2015 — no change
    let maturity = cds_maturity(date(2015, 9, 19), &tenor, rule)
        .unwrap()
        .unwrap();
    let s = make_cds_schedule(date(2015, 9, 19), maturity, rule);
    check_dates(&s, &exp);

    // Trade date = Sun 20 Sep 2015 — roll to new maturity, keep old start
    let maturity = cds_maturity(date(2015, 9, 20), &tenor, rule)
        .unwrap()
        .unwrap();
    let s = make_cds_schedule(date(2015, 9, 20), maturity, rule);
    exp.push(date(2016, 9, 20));
    exp.push(date(2016, 12, 20));
    check_dates(&s, &exp);

    // Trade date = Mon 21 Sep 2015 — first period drops out
    let maturity = cds_maturity(date(2015, 9, 21), &tenor, rule)
        .unwrap()
        .unwrap();
    let s = make_cds_schedule(date(2015, 9, 21), maturity, rule);
    exp.remove(0);
    check_dates(&s, &exp);

    // Another sample: Sat 20 Jun 2009
    let maturity = date(2009, 12, 20);
    let s = make_cds_schedule(date(2009, 6, 20), maturity, rule);
    let exp2 = vec![
        date(2009, 3, 20),
        date(2009, 6, 22),
        date(2009, 9, 21),
        date(2009, 12, 20),
    ];
    check_dates(&s, &exp2);

    // Sun 21 Jun 2009 — same
    let s = make_cds_schedule(date(2009, 6, 21), maturity, rule);
    check_dates(&s, &exp2);

    // Mon 22 Jun 2009 — first period drops
    let s = make_cds_schedule(date(2009, 6, 22), maturity, rule);
    check_dates(&s, &exp2[1..]);
}

// ──────────────── testCDSConventionSampleDates ───────

#[test]
fn test_cds_convention_sample_dates() {
    let rule = DateGeneration::CDS;
    let tenor = Period::new(1, TimeUnit::Years);

    // Trade date = Fri 18 Sep 2015
    let mut exp = vec![
        date(2015, 6, 22),
        date(2015, 9, 21),
        date(2015, 12, 21),
        date(2016, 3, 21),
        date(2016, 6, 20),
        date(2016, 9, 20),
    ];
    let maturity = cds_maturity(date(2015, 9, 18), &tenor, rule)
        .unwrap()
        .unwrap();
    let s = make_cds_schedule(date(2015, 9, 18), maturity, rule);
    check_dates(&s, &exp);

    // Trade date = Sat 19 Sep 2015 — no change
    let maturity = cds_maturity(date(2015, 9, 19), &tenor, rule)
        .unwrap()
        .unwrap();
    let s = make_cds_schedule(date(2015, 9, 19), maturity, rule);
    check_dates(&s, &exp);

    // Trade date = Sun 20 Sep 2015 — roll
    let maturity = cds_maturity(date(2015, 9, 20), &tenor, rule)
        .unwrap()
        .unwrap();
    let s = make_cds_schedule(date(2015, 9, 20), maturity, rule);
    exp.push(date(2016, 12, 20));
    check_dates(&s, &exp);

    // Trade date = Mon 21 Sep 2015 — first period drops out
    let maturity = cds_maturity(date(2015, 9, 21), &tenor, rule)
        .unwrap()
        .unwrap();
    let s = make_cds_schedule(date(2015, 9, 21), maturity, rule);
    exp.remove(0);
    check_dates(&s, &exp);

    // Another sample: Sat 20 Jun 2009
    let maturity = date(2009, 12, 20);
    let s = make_cds_schedule(date(2009, 6, 20), maturity, rule);
    let exp2 = vec![
        date(2009, 3, 20),
        date(2009, 6, 22),
        date(2009, 9, 21),
        date(2009, 12, 20),
    ];
    check_dates(&s, &exp2);

    // Sun 21 Jun 2009
    let s = make_cds_schedule(date(2009, 6, 21), maturity, rule);
    check_dates(&s, &exp2);

    // Mon 22 Jun 2009 — first period drops
    let s = make_cds_schedule(date(2009, 6, 22), maturity, rule);
    check_dates(&s, &exp2[1..]);
}

// ──────────────── testOldCDSConventionSampleDates ────

#[test]
fn test_old_cds_convention_sample_dates() {
    let rule = DateGeneration::OldCDS;
    let tenor = Period::new(1, TimeUnit::Years);

    // trade date plus 1D = Fri 18 Sep 2015
    let mut exp = vec![
        date(2015, 9, 18),
        date(2015, 12, 21),
        date(2016, 3, 21),
        date(2016, 6, 20),
        date(2016, 9, 20),
    ];
    let td = date(2015, 9, 18);
    let maturity = cds_maturity(td, &tenor, rule).unwrap().unwrap();
    let s = make_cds_schedule(td, maturity, rule);
    check_dates(&s, &exp);

    // trade date plus 1D = Sat 19 Sep 2015 — start date stays (not adjusted for OldCDS)
    exp[0] = date(2015, 9, 19);
    let maturity = cds_maturity(date(2015, 9, 19), &tenor, rule)
        .unwrap()
        .unwrap();
    let s = make_cds_schedule(date(2015, 9, 19), maturity, rule);
    check_dates(&s, &exp);

    // trade date plus 1D = Sun 20 Sep 2015 — roll
    exp[0] = date(2015, 9, 20);
    let maturity = cds_maturity(date(2015, 9, 20), &tenor, rule)
        .unwrap()
        .unwrap();
    let s = make_cds_schedule(date(2015, 9, 20), maturity, rule);
    exp.push(date(2016, 12, 20));
    check_dates(&s, &exp);

    // trade date plus 1D = Mon 21 Sep 2015 — no change
    exp[0] = date(2015, 9, 21);
    let maturity = cds_maturity(date(2015, 9, 21), &tenor, rule)
        .unwrap()
        .unwrap();
    let s = make_cds_schedule(date(2015, 9, 21), maturity, rule);
    check_dates(&s, &exp);

    // 30-day stub rule: 19 Nov 2015 + 30D = 19 Dec <= 20 Dec → short front
    exp[0] = date(2015, 11, 19);
    let s = make_cds_schedule(date(2015, 11, 19), maturity, rule);
    check_dates(&s, &exp);

    // 20 Nov 2015 + 30D = 20 Dec <= 20 Dec → still short front
    exp[0] = date(2015, 11, 20);
    let s = make_cds_schedule(date(2015, 11, 20), maturity, rule);
    check_dates(&s, &exp);

    // 21 Nov 2015 + 30D = 21 Dec > 20 Dec → long front stub
    exp[0] = date(2015, 11, 21);
    let s = make_cds_schedule(date(2015, 11, 21), maturity, rule);
    exp.remove(1); // the Dec 21 2015 date drops
    check_dates(&s, &exp);
}

// ──────────────── testCDS2015ZeroMonthsMatured ───────

#[test]
fn test_cds2015_zero_months_matured() {
    let rule = DateGeneration::CDS2015;
    let tenor = Period::new(0, TimeUnit::Months);

    let inputs = [
        date(2015, 12, 20),
        date(2016, 2, 15),
        date(2016, 3, 19),
        date(2016, 6, 20),
        date(2016, 8, 15),
        date(2016, 9, 19),
        date(2016, 12, 20),
    ];

    for td in &inputs {
        let result = cds_maturity(*td, &tenor, rule).unwrap();
        assert!(
            result.is_none(),
            "CDS2015 0M should be matured for trade date {td}"
        );
    }
}
