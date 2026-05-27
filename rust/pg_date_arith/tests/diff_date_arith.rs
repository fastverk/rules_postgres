//! Diff-test for the Pg.Ir-emitted date-arithmetic cluster.
//!
//! For random operand pairs (valid dates and day offsets), invoke both
//! the Rust body (Lean-emitted) and the vendored Postgres C body (via the
//! setjmp wrapper). Assert:
//!   - Both raise the same FmgrErrorKind (or both succeed).
//!   - When both succeed, the returned Datum is bit-identical.
//!
//! Overflow and range-check corners are exercised explicitly:
//!   - date_pli: boundary conditions on addition wraparound and sentinel
//!     handling.
//!   - date_mii: boundary conditions on subtraction wraparound and sentinel
//!     handling.
//!   - date_mi: both operands infinite (error), one operand infinite
//!     (error), both finite (success).

// Link against the pg_date_arith library to ensure symbols are available.
extern crate pg_date_arith;

use pg_fcinfo::{
    build_fcinfo, fmgr_clear_error, fmgr_take_error, FmgrErrorKind,
    FunctionCallInfoBaseData,
};
use proptest::prelude::*;

type FmgrFn = unsafe extern "C" fn(*mut FunctionCallInfoBaseData) -> u64;

extern "C" {
    fn c_date_pli(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_date_mii(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_date_mi(fcinfo: *mut FunctionCallInfoBaseData) -> u64;

    /* Rust function externs for the Rust-side diff-test. */
    fn date_pli(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn date_mii(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn date_mi(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
}

/* ─────── invokers ─────── */

fn invoke_date_int(fn_: FmgrFn, d: i32, i: i32) -> (u64, Option<FmgrErrorKind>) {
    fmgr_clear_error();
    let mut fcinfo = build_fcinfo!(args = [(i32, d), (i32, i)]);
    let out = unsafe { fn_(&mut fcinfo) };
    (out, fmgr_take_error())
}

fn invoke_date_date(fn_: FmgrFn, d1: i32, d2: i32) -> (u64, Option<FmgrErrorKind>) {
    fmgr_clear_error();
    let mut fcinfo = build_fcinfo!(args = [(i32, d1), (i32, d2)]);
    let out = unsafe { fn_(&mut fcinfo) };
    (out, fmgr_take_error())
}

/* ─────── diff-test harness ─────── */

fn assert_same_outcome_date_int(name: &str, rust_fn: FmgrFn, c_fn: FmgrFn, d: i32, i: i32) {
    let (rust_out, rust_err) = invoke_date_int(rust_fn, d, i);
    let (c_out, c_err) = invoke_date_int(c_fn, d, i);
    match (rust_err, c_err) {
        (None, None) => {
            assert_eq!(rust_out, c_out, "{}({}, {}): Datum mismatch", name, d, i);
        }
        (Some(re), Some(ce)) => {
            assert_eq!(
                re, ce,
                "{}({}, {}): FmgrErrorKind mismatch: Rust={:?} vs C={:?}",
                name, d, i, re, ce
            );
        }
        _ => panic!(
            "{}({}, {}): error mismatch: Rust={:?} vs C={:?}",
            name, d, i, rust_err, c_err
        ),
    }
}

fn assert_same_outcome_date_date(name: &str, rust_fn: FmgrFn, c_fn: FmgrFn, d1: i32, d2: i32) {
    let (rust_out, rust_err) = invoke_date_date(rust_fn, d1, d2);
    let (c_out, c_err) = invoke_date_date(c_fn, d1, d2);
    match (rust_err, c_err) {
        (None, None) => {
            assert_eq!(rust_out, c_out, "{}({}, {}): Datum mismatch", name, d1, d2);
        }
        (Some(re), Some(ce)) => {
            assert_eq!(
                re, ce,
                "{}({}, {}): FmgrErrorKind mismatch: Rust={:?} vs C={:?}",
                name, d1, d2, re, ce
            );
        }
        _ => panic!(
            "{}({}, {}): error mismatch: Rust={:?} vs C={:?}",
            name, d1, d2, rust_err, c_err
        ),
    }
}

/* ─────── property tests ─────── */

proptest! {
    #[test]
    fn prop_date_pli_matches_c(d in -1_000_000_000i32..=1_000_000_000i32,
                                 i in -100_000i32..=100_000i32) {
        assert_same_outcome_date_int("date_pli", date_pli, c_date_pli, d, i);
    }

    #[test]
    fn prop_date_mii_matches_c(d in -1_000_000_000i32..=1_000_000_000i32,
                                 i in -100_000i32..=100_000i32) {
        assert_same_outcome_date_int("date_mii", date_mii, c_date_mii, d, i);
    }

    #[test]
    fn prop_date_mi_matches_c(d1 in -1_000_000_000i32..=1_000_000_000i32,
                               d2 in -1_000_000_000i32..=1_000_000_000i32) {
        assert_same_outcome_date_date("date_mi", date_mi, c_date_mi, d1, d2);
    }
}

/* ─────── boundary / regression tests ─────── */

#[test]
fn test_date_pli_no_overflow_in_range() {
    // Normal case: adding a small number of days to a valid date
    assert_same_outcome_date_int("date_pli", date_pli, c_date_pli, 0, 100);
}

#[test]
fn test_date_pli_sentinel_pass_through() {
    // DATEVAL_NOBEGIN (i32::MIN) should pass through
    assert_same_outcome_date_int("date_pli", date_pli, c_date_pli, i32::MIN, 10);
    // DATEVAL_NOEND (i32::MAX) should pass through
    assert_same_outcome_date_int("date_pli", date_pli, c_date_pli, i32::MAX, 10);
}

#[test]
fn test_date_pli_overflow_addition() {
    // Try to overflow by adding to a large date (this actually may NOT overflow
    // depending on the values chosen, as Postgres uses modular arithmetic).
    assert_same_outcome_date_int("date_pli", date_pli, c_date_pli, 1_000_000_000, 1_000_000_000);
}

#[test]
fn test_date_pli_overflow_subtraction() {
    // Try to underflow by adding negative to a small date
    assert_same_outcome_date_int("date_pli", date_pli, c_date_pli, -1_000_000_000, -1_000_000_000);
}

#[test]
fn test_date_mii_no_overflow_in_range() {
    // Normal case: subtracting a small number of days from a valid date
    assert_same_outcome_date_int("date_mii", date_mii, c_date_mii, 0, 100);
}

#[test]
fn test_date_mii_sentinel_pass_through() {
    // DATEVAL_NOBEGIN (i32::MIN) should pass through
    assert_same_outcome_date_int("date_mii", date_mii, c_date_mii, i32::MIN, 10);
    // DATEVAL_NOEND (i32::MAX) should pass through
    assert_same_outcome_date_int("date_mii", date_mii, c_date_mii, i32::MAX, 10);
}

#[test]
fn test_date_mii_overflow_subtraction() {
    // Try to underflow by subtracting large positive from a small date
    assert_same_outcome_date_int("date_mii", date_mii, c_date_mii, -1_000_000_000, 1_000_000_000);
}

#[test]
fn test_date_mii_overflow_addition_via_negative() {
    // Try to overflow by subtracting negative (which is addition) from a large date
    assert_same_outcome_date_int("date_mii", date_mii, c_date_mii, 1_000_000_000, -1_000_000_000);
}

#[test]
fn test_date_mi_both_valid() {
    // Normal case: subtract two valid dates
    assert_same_outcome_date_date("date_mi", date_mi, c_date_mi, 1000, 500);
}

#[test]
fn test_date_mi_first_sentinel_nobegin() {
    // First operand is DATEVAL_NOBEGIN
    assert_same_outcome_date_date("date_mi", date_mi, c_date_mi, i32::MIN, 500);
}

#[test]
fn test_date_mi_first_sentinel_noend() {
    // First operand is DATEVAL_NOEND
    assert_same_outcome_date_date("date_mi", date_mi, c_date_mi, i32::MAX, 500);
}

#[test]
fn test_date_mi_second_sentinel_nobegin() {
    // Second operand is DATEVAL_NOBEGIN
    assert_same_outcome_date_date("date_mi", date_mi, c_date_mi, 500, i32::MIN);
}

#[test]
fn test_date_mi_second_sentinel_noend() {
    // Second operand is DATEVAL_NOEND
    assert_same_outcome_date_date("date_mi", date_mi, c_date_mi, 500, i32::MAX);
}

#[test]
fn test_date_mi_both_sentinels() {
    // Both operands are sentinels
    assert_same_outcome_date_date("date_mi", date_mi, c_date_mi, i32::MIN, i32::MAX);
}

