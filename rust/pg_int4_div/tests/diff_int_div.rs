//! Diff-test for the Pg.Ir-emitted integer-division cluster.
//!
//! For operand pairs of each width, invoke both the Rust body (Lean-emitted)
//! and the vendored Postgres C body (via the setjmp wrapper). Assert:
//!   - Both raise the same FmgrErrorKind (or both succeed).
//!   - When both succeed, the returned Datum is bit-identical.
//!
//! Division-by-zero and overflow (INT_MIN / -1) corners are exercised
//! explicitly by the boundary tests so the ereport path (TLS flag round-trip +
//! longjmp wrapper) is checked end-to-end on the deterministic edges.
//! Proptest covers normal cases with b != 0 to avoid trivial crashes.

use pg_fcinfo::{
    build_fcinfo, decode_i16, decode_i32, decode_i64,
    fmgr_clear_error, fmgr_take_error, FmgrErrorKind,
    FunctionCallInfoBaseData,
};
use proptest::prelude::*;

type FmgrFn = unsafe extern "C" fn(*mut FunctionCallInfoBaseData) -> u64;

extern "C" {
    fn c_int2div(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int4div(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int8div(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
}

/* ─────── invokers ─────── */

fn invoke_ii(fn_: FmgrFn, a: i16, b: i16) -> (u64, Option<FmgrErrorKind>) {
    fmgr_clear_error();
    let mut fcinfo = build_fcinfo!(args = [(i16, a), (i16, b)]);
    let out = unsafe { fn_(&mut fcinfo) };
    (out, fmgr_take_error())
}

fn invoke_ll(fn_: FmgrFn, a: i32, b: i32) -> (u64, Option<FmgrErrorKind>) {
    fmgr_clear_error();
    let mut fcinfo = build_fcinfo!(args = [(i32, a), (i32, b)]);
    let out = unsafe { fn_(&mut fcinfo) };
    (out, fmgr_take_error())
}

fn invoke_qq(fn_: FmgrFn, a: i64, b: i64) -> (u64, Option<FmgrErrorKind>) {
    fmgr_clear_error();
    let mut fcinfo = build_fcinfo!(args = [(i64, a), (i64, b)]);
    let out = unsafe { fn_(&mut fcinfo) };
    (out, fmgr_take_error())
}

/* ─────── per-family compare helpers ─────── */

macro_rules! compare_fam {
    ($name:ident, $invoke:ident, $atype:ty, $decode_check:expr,
     $c_fn:ident, $r_fn:path) => {
        fn $name(a: $atype, b: $atype) {
            let (c_out, c_err) = $invoke($c_fn, a, b);
            let (r_out, r_err) = $invoke($r_fn, a, b);
            assert_eq!(
                c_err, r_err,
                "{}({a}, {b}): C error = {c_err:?}, Rust error = {r_err:?}",
                stringify!($name),
            );
            if r_err.is_none() {
                assert_eq!(
                    c_out, r_out,
                    "{}({a}, {b}): C={c_out} Rust={r_out} (decoded: C={} R={})",
                    stringify!($name),
                    $decode_check(c_out),
                    $decode_check(r_out),
                );
            }
        }
    };
}

compare_fam!(compare_int2div, invoke_ii, i16, decode_i16, c_int2div, pg_int4_div::int2div);
compare_fam!(compare_int4div, invoke_ll, i32, decode_i32, c_int4div, pg_int4_div::int4div);
compare_fam!(compare_int8div, invoke_qq, i64, decode_i64, c_int8div, pg_int4_div::int8div);

/* ─────── boundary and error tests ─────── */

#[test]
fn boundary_int2div() {
    // Division by zero — both should error with ERRCODE_DIVISION_BY_ZERO.
    compare_int2div(0, 0);
    compare_int2div(1, 0);
    compare_int2div(i16::MAX, 0);
    compare_int2div(i16::MIN, 0);

    // INT_MIN / -1 overflow — both should error with ERRCODE_NUMERIC_VALUE_OUT_OF_RANGE.
    compare_int2div(i16::MIN, -1);

    // INT_MIN / -2 should succeed (negation of INT_MIN / 2 would overflow,
    // but division by -2 doesn't trigger the special case).
    compare_int2div(i16::MIN, -2);

    // Normal successful divisions.
    compare_int2div(10, 2);
    compare_int2div(10, 3);
    compare_int2div(-10, 2);
    compare_int2div(-10, -3);
    compare_int2div(i16::MAX, 1);
    compare_int2div(i16::MIN + 1, 1);
}

#[test]
fn boundary_int4div() {
    // Division by zero.
    compare_int4div(0, 0);
    compare_int4div(1, 0);
    compare_int4div(i32::MAX, 0);
    compare_int4div(i32::MIN, 0);

    // INT_MIN / -1 overflow.
    compare_int4div(i32::MIN, -1);

    // Normal successful divisions.
    compare_int4div(10, 2);
    compare_int4div(10, 3);
    compare_int4div(-10, 2);
    compare_int4div(-10, -3);
    compare_int4div(i32::MAX, 1);
    compare_int4div(i32::MIN + 1, 1);
}

#[test]
fn boundary_int8div() {
    // Division by zero.
    compare_int8div(0, 0);
    compare_int8div(1, 0);
    compare_int8div(i64::MAX, 0);
    compare_int8div(i64::MIN, 0);

    // INT_MIN / -1 overflow.
    compare_int8div(i64::MIN, -1);

    // Normal successful divisions.
    compare_int8div(10, 2);
    compare_int8div(10, 3);
    compare_int8div(-10, 2);
    compare_int8div(-10, -3);
    compare_int8div(i64::MAX, 1);
    compare_int8div(i64::MIN + 1, 1);
}

/* ─────── proptest with all divisor cases (errors caught by boundary tests) ─────── */

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn prop_int2div(a in any::<i16>(), b in any::<i16>()) {
        // Boundary tests explicitly cover division-by-zero and INT_MIN / -1.
        // Proptest exercises the normal division cases with random inputs.
        compare_int2div(a, b);
    }

    #[test]
    fn prop_int4div(a in any::<i32>(), b in any::<i32>()) {
        compare_int4div(a, b);
    }

    #[test]
    fn prop_int8div(a in any::<i64>(), b in any::<i64>()) {
        compare_int8div(a, b);
    }
}
