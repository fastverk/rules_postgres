//! Diff-test for the Pg.Ir-emitted cash-arithmetic cluster.
//!
//! For random operand pairs, invoke both the Rust body (Lean-emitted) and
//! the vendored Postgres C body (via the setjmp wrapper). Assert:
//!   - Both raise the same FmgrErrorKind (or both succeed).
//!   - When both succeed, the returned Datum is bit-identical.
//!
//! Overflow and division-by-zero corners are exercised explicitly so
//! the ereport path (TLS flag round-trip + longjmp wrapper) is checked
//! end-to-end on the deterministic edges.

use pg_fcinfo::{
    build_fcinfo, decode_i64,
    fmgr_clear_error, fmgr_take_error, FmgrErrorKind,
    FunctionCallInfoBaseData,
};
use proptest::prelude::*;

type FmgrFn = unsafe extern "C" fn(*mut FunctionCallInfoBaseData) -> u64;

extern "C" {
    fn c_cash_pl(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_cash_mi(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_cash_mul_int4(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_cash_div_int4(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_cashlarger(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_cashsmaller(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
}

/* ─────── invokers (one per arg-type pair) ─────── */

fn invoke_qq(fn_: FmgrFn, a: i64, b: i64) -> (u64, Option<FmgrErrorKind>) {
    fmgr_clear_error();
    let mut fcinfo = build_fcinfo!(args = [(i64, a), (i64, b)]);
    let out = unsafe { fn_(&mut fcinfo) };
    (out, fmgr_take_error())
}

fn invoke_ql(fn_: FmgrFn, a: i64, b: i32) -> (u64, Option<FmgrErrorKind>) {
    fmgr_clear_error();
    let mut fcinfo = build_fcinfo!(args = [(i64, a), (i32, b)]);
    let out = unsafe { fn_(&mut fcinfo) };
    (out, fmgr_take_error())
}

/* ─────── per-function compare helpers ─────── */

macro_rules! compare_fn {
    ($name:ident, $invoke:ident, $atype:ty, $btype:ty, $decode_check:expr,
     $c_fn:ident, $r_fn:path) => {
        fn $name(a: $atype, b: $btype) {
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

// All cash operations return i64; use decode_i64 for diagnostics.
compare_fn!(compare_cash_pl,       invoke_qq, i64, i64, decode_i64, c_cash_pl,       pg_cash_arith::cash_pl);
compare_fn!(compare_cash_mi,       invoke_qq, i64, i64, decode_i64, c_cash_mi,       pg_cash_arith::cash_mi);
compare_fn!(compare_cash_mul_int4, invoke_ql, i64, i32, decode_i64, c_cash_mul_int4, pg_cash_arith::cash_mul_int4);
compare_fn!(compare_cash_div_int4, invoke_ql, i64, i32, decode_i64, c_cash_div_int4, pg_cash_arith::cash_div_int4);
compare_fn!(compare_cashlarger,    invoke_qq, i64, i64, decode_i64, c_cashlarger,    pg_cash_arith::cashlarger);
compare_fn!(compare_cashsmaller,   invoke_qq, i64, i64, decode_i64, c_cashsmaller,   pg_cash_arith::cashsmaller);

/* ─────── boundary tests ─────── */

#[test]
fn boundary_cash_pl_mi() {
    // Safe pairs (overflow checks against i64 bounds).
    let safe: &[(i64, i64)] = &[
        (0, 0), (1, -1), (i64::MAX, 0), (i64::MIN, 0),
        (i64::MAX / 2, i64::MAX / 2),
        (i64::MIN / 2, i64::MIN / 2),
    ];
    for &(a, b) in safe {
        compare_cash_pl(a, b);
        compare_cash_mi(a, b);
    }
    // Overflow cases for pl.
    let pl_ov: &[(i64, i64)] = &[(i64::MAX, 1), (i64::MIN, -1), (i64::MAX, i64::MAX)];
    for &(a, b) in pl_ov { compare_cash_pl(a, b); }
    // Overflow cases for mi.
    let mi_ov: &[(i64, i64)] = &[(i64::MIN, 1), (i64::MAX, -1), (0, i64::MIN)];
    for &(a, b) in mi_ov { compare_cash_mi(a, b); }
}

#[test]
fn boundary_cash_mul_int4() {
    // Safe pairs.
    let safe: &[(i64, i32)] = &[
        (0, 0), (1, 1), (i64::MAX, 0), (i64::MIN, 0),
        (1000, 1000), (-1000, 1000),
    ];
    for &(a, b) in safe {
        compare_cash_mul_int4(a, b);
    }
    // Overflow cases.
    let ov: &[(i64, i32)] = &[
        (i64::MAX, 2), (i64::MIN, 2), (i64::MIN, -1),
        (i64::MAX / 2, 3),
    ];
    for &(a, b) in ov { compare_cash_mul_int4(a, b); }
}

#[test]
fn boundary_cash_div_int4() {
    // Safe pairs (non-zero divisor).
    let safe: &[(i64, i32)] = &[
        (0, 1), (10, 2), (10, 3), (-10, 3), (i64::MAX, 1), (i64::MIN, 1),
    ];
    for &(a, b) in safe {
        compare_cash_div_int4(a, b);
    }
    // Division by zero.
    let div_by_zero: &[(i64, i32)] = &[
        (0, 0), (1, 0), (-1, 0), (i64::MAX, 0), (i64::MIN, 0),
    ];
    for &(a, b) in div_by_zero {
        compare_cash_div_int4(a, b);
    }
}

#[test]
fn boundary_cashlarger_smaller() {
    // Simple comparison tests.
    let pairs: &[(i64, i64)] = &[
        (0, 0), (1, 2), (2, 1), (-1, 1), (i64::MAX, i64::MIN),
        (i64::MAX, 0), (i64::MIN, 0),
    ];
    for &(a, b) in pairs {
        compare_cashlarger(a, b);
        compare_cashsmaller(a, b);
    }
}

/* ─────── proptests (one per function) ─────── */

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test] fn pt_cash_pl(a in any::<i64>(), b in any::<i64>())        { compare_cash_pl(a, b); }
    #[test] fn pt_cash_mi(a in any::<i64>(), b in any::<i64>())        { compare_cash_mi(a, b); }
    #[test] fn pt_cash_mul_int4(a in any::<i64>(), b in any::<i32>())  { compare_cash_mul_int4(a, b); }
    #[test] fn pt_cash_div_int4(a in any::<i64>(), b in any::<i32>())  { compare_cash_div_int4(a, b); }
    #[test] fn pt_cashlarger(a in any::<i64>(), b in any::<i64>())     { compare_cashlarger(a, b); }
    #[test] fn pt_cashsmaller(a in any::<i64>(), b in any::<i64>())    { compare_cashsmaller(a, b); }
}
