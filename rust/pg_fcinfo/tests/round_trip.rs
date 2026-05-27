//! End-to-end round-trip test: `build_fcinfo!` macro → C fixture
//! (mimicking a real Postgres V1 fmgr function) → `decode_*` →
//! original Rust value.
//!
//! This is the integration validation for phase 2 of the Datum
//! scaffold generator. It proves:
//!
//!   1. `build_fcinfo!` produces a `FunctionCallInfoBaseData` whose
//!      layout the C side reads correctly.
//!   2. `encode_*` helpers (invoked by the macro) produce Datum
//!      values that the C side's `PG_GETARG_*` macros decode back to
//!      the original Rust value.
//!   3. `decode_*` reverses what the C side produced via `PG_RETURN_*`.
//!   4. The result-isnull path (out-parameter `fcinfo.isnull`) works.
//!
//! When Stream C lifts a real Postgres V1 function (e.g.,
//! `gbt_bool_compress`), this same harness pattern wires up — the
//! only difference is the C function comes from Postgres' source tree
//! linked via the rename shim, instead of `tests/fixture.c`.

use pg_fcinfo::{
    build_fcinfo, decode_bool, decode_i32, decode_i64, verify_abi, FunctionCallInfoBaseData,
};

extern "C" {
    fn c_test_double_i32(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_test_add_i32(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_test_not_bool(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_test_echo_nargs(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_test_return_null(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
}

#[test]
fn abi_layout_verified_before_round_trips() {
    // Sanity check: don't even try the round-trips if the layout
    // doesn't match. Failures here mean the rest of the test is
    // operating on garbage.
    verify_abi().expect("ABI mismatch — fix the Rust mirrors before trusting round-trip results");
}

#[test]
fn round_trip_i32_double() {
    for input in [i32::MIN, -1, 0, 1, 42, i32::MAX] {
        let mut fcinfo = build_fcinfo!(args = [(i32, input)]);
        let result = unsafe { c_test_double_i32(&mut fcinfo) };
        let decoded = decode_i32(result);
        // C does `v * 2` with i32 overflow wrap (standard integer behavior
        // on the cast); compare against wrapping_mul.
        let expected = input.wrapping_mul(2);
        assert_eq!(decoded, expected, "double(i32 {input}) round-trip");
    }
}

#[test]
fn round_trip_i32_add_two_args() {
    for (a, b) in [
        (0, 0),
        (1, 2),
        (-5, 10),
        (i32::MAX, 1), // overflow case
        (i32::MIN, -1),
    ] {
        let mut fcinfo = build_fcinfo!(args = [(i32, a), (i32, b)]);
        let result = unsafe { c_test_add_i32(&mut fcinfo) };
        let decoded = decode_i32(result);
        let expected = a.wrapping_add(b);
        assert_eq!(decoded, expected, "add_i32({a}, {b}) round-trip");
    }
}

#[test]
fn round_trip_bool_not() {
    for input in [false, true] {
        let mut fcinfo = build_fcinfo!(args = [(bool, input)]);
        let result = unsafe { c_test_not_bool(&mut fcinfo) };
        let decoded = decode_bool(result);
        assert_eq!(decoded, !input, "not({input}) round-trip");
    }
}

#[test]
fn fcinfo_nargs_field_is_read_correctly() {
    // `build_fcinfo!` sets nargs from the count of (tag, val) tuples;
    // c_test_echo_nargs reads that field and returns it as i64.
    for n in 0..=5 {
        let fcinfo = match n {
            0 => build_fcinfo!(args = []),
            1 => build_fcinfo!(args = [(i32, 0)]),
            2 => build_fcinfo!(args = [(i32, 0), (i32, 0)]),
            3 => build_fcinfo!(args = [(i32, 0), (i32, 0), (i32, 0)]),
            4 => build_fcinfo!(args = [(i32, 0), (i32, 0), (i32, 0), (i32, 0)]),
            5 => build_fcinfo!(args = [(i32, 0), (i32, 0), (i32, 0), (i32, 0), (i32, 0)]),
            _ => unreachable!(),
        };
        let mut f = fcinfo;
        assert_eq!(f.nargs, n as i16, "macro set nargs to {n}");
        let result = unsafe { c_test_echo_nargs(&mut f) };
        let decoded = decode_i64(result);
        assert_eq!(decoded, n as i64, "C read nargs as {n}");
    }
}

#[test]
fn result_isnull_path() {
    let mut fcinfo = build_fcinfo!(args = []);
    assert!(!fcinfo.isnull, "isnull starts false");
    let _ = unsafe { c_test_return_null(&mut fcinfo) };
    assert!(
        fcinfo.isnull,
        "C side set isnull=true via the fcinfo.isnull out-parameter"
    );
}
