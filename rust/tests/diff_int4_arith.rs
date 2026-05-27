//! Diff-test for the Pg.Ir-emitted integer-arithmetic cluster.
//!
//! For random operand pairs of each width-combination, invoke both the
//! Rust body (Lean-emitted) and the vendored Postgres C body (via the
//! setjmp wrapper). Assert:
//!   - Both raise the same FmgrErrorKind (or both succeed).
//!   - When both succeed, the returned Datum is bit-identical.
//!
//! Overflow corners are exercised explicitly by the boundary tests so
//! the ereport path (TLS flag round-trip + longjmp wrapper) is checked
//! end-to-end on the deterministic edges, not just statistically by
//! proptest.

use pg_fcinfo::{
    build_fcinfo, decode_i16, decode_i32, decode_i64,
    fmgr_clear_error, fmgr_take_error, FmgrErrorKind,
    FunctionCallInfoBaseData,
};
use proptest::prelude::*;

type FmgrFn = unsafe extern "C" fn(*mut FunctionCallInfoBaseData) -> u64;

extern "C" {
    fn c_int2pl(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int2mi(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int2mul(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int4pl(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int4mi(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int4mul(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int8pl(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int8mi(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int8mul(fcinfo: *mut FunctionCallInfoBaseData) -> u64;

    fn c_int24pl(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int24mi(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int24mul(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int42pl(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int42mi(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int42mul(fcinfo: *mut FunctionCallInfoBaseData) -> u64;

    fn c_int48pl(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int48mi(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int48mul(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int84pl(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int84mi(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int84mul(fcinfo: *mut FunctionCallInfoBaseData) -> u64;

    fn c_int28pl(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int28mi(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int28mul(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int82pl(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int82mi(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int82mul(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
}

/* ─────── invokers (one per arg-width pair) ─────── */

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

fn invoke_il(fn_: FmgrFn, a: i16, b: i32) -> (u64, Option<FmgrErrorKind>) {
    fmgr_clear_error();
    let mut fcinfo = build_fcinfo!(args = [(i16, a), (i32, b)]);
    let out = unsafe { fn_(&mut fcinfo) };
    (out, fmgr_take_error())
}

fn invoke_li(fn_: FmgrFn, a: i32, b: i16) -> (u64, Option<FmgrErrorKind>) {
    fmgr_clear_error();
    let mut fcinfo = build_fcinfo!(args = [(i32, a), (i16, b)]);
    let out = unsafe { fn_(&mut fcinfo) };
    (out, fmgr_take_error())
}

fn invoke_lq(fn_: FmgrFn, a: i32, b: i64) -> (u64, Option<FmgrErrorKind>) {
    fmgr_clear_error();
    let mut fcinfo = build_fcinfo!(args = [(i32, a), (i64, b)]);
    let out = unsafe { fn_(&mut fcinfo) };
    (out, fmgr_take_error())
}

fn invoke_ql(fn_: FmgrFn, a: i64, b: i32) -> (u64, Option<FmgrErrorKind>) {
    fmgr_clear_error();
    let mut fcinfo = build_fcinfo!(args = [(i64, a), (i32, b)]);
    let out = unsafe { fn_(&mut fcinfo) };
    (out, fmgr_take_error())
}

fn invoke_iq(fn_: FmgrFn, a: i16, b: i64) -> (u64, Option<FmgrErrorKind>) {
    fmgr_clear_error();
    let mut fcinfo = build_fcinfo!(args = [(i16, a), (i64, b)]);
    let out = unsafe { fn_(&mut fcinfo) };
    (out, fmgr_take_error())
}

fn invoke_qi(fn_: FmgrFn, a: i64, b: i16) -> (u64, Option<FmgrErrorKind>) {
    fmgr_clear_error();
    let mut fcinfo = build_fcinfo!(args = [(i64, a), (i16, b)]);
    let out = unsafe { fn_(&mut fcinfo) };
    (out, fmgr_take_error())
}

/* ─────── per-family compare helpers ─────── */

macro_rules! compare_fam {
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

// Result type's decoder is what we want to print for diagnostics:
// int2*=i16, int4* and int24/int42=i32, int8* and any cross-int8=i64.
compare_fam!(compare_int2pl,  invoke_ii, i16, i16, decode_i16, c_int2pl,  pg_int4_arith::int2pl);
compare_fam!(compare_int2mi,  invoke_ii, i16, i16, decode_i16, c_int2mi,  pg_int4_arith::int2mi);
compare_fam!(compare_int2mul, invoke_ii, i16, i16, decode_i16, c_int2mul, pg_int4_arith::int2mul);

compare_fam!(compare_int4pl,  invoke_ll, i32, i32, decode_i32, c_int4pl,  pg_int4_arith::int4pl);
compare_fam!(compare_int4mi,  invoke_ll, i32, i32, decode_i32, c_int4mi,  pg_int4_arith::int4mi);
compare_fam!(compare_int4mul, invoke_ll, i32, i32, decode_i32, c_int4mul, pg_int4_arith::int4mul);

compare_fam!(compare_int8pl,  invoke_qq, i64, i64, decode_i64, c_int8pl,  pg_int4_arith::int8pl);
compare_fam!(compare_int8mi,  invoke_qq, i64, i64, decode_i64, c_int8mi,  pg_int4_arith::int8mi);
compare_fam!(compare_int8mul, invoke_qq, i64, i64, decode_i64, c_int8mul, pg_int4_arith::int8mul);

compare_fam!(compare_int24pl,  invoke_il, i16, i32, decode_i32, c_int24pl,  pg_int4_arith::int24pl);
compare_fam!(compare_int24mi,  invoke_il, i16, i32, decode_i32, c_int24mi,  pg_int4_arith::int24mi);
compare_fam!(compare_int24mul, invoke_il, i16, i32, decode_i32, c_int24mul, pg_int4_arith::int24mul);

compare_fam!(compare_int42pl,  invoke_li, i32, i16, decode_i32, c_int42pl,  pg_int4_arith::int42pl);
compare_fam!(compare_int42mi,  invoke_li, i32, i16, decode_i32, c_int42mi,  pg_int4_arith::int42mi);
compare_fam!(compare_int42mul, invoke_li, i32, i16, decode_i32, c_int42mul, pg_int4_arith::int42mul);

compare_fam!(compare_int48pl,  invoke_lq, i32, i64, decode_i64, c_int48pl,  pg_int4_arith::int48pl);
compare_fam!(compare_int48mi,  invoke_lq, i32, i64, decode_i64, c_int48mi,  pg_int4_arith::int48mi);
compare_fam!(compare_int48mul, invoke_lq, i32, i64, decode_i64, c_int48mul, pg_int4_arith::int48mul);

compare_fam!(compare_int84pl,  invoke_ql, i64, i32, decode_i64, c_int84pl,  pg_int4_arith::int84pl);
compare_fam!(compare_int84mi,  invoke_ql, i64, i32, decode_i64, c_int84mi,  pg_int4_arith::int84mi);
compare_fam!(compare_int84mul, invoke_ql, i64, i32, decode_i64, c_int84mul, pg_int4_arith::int84mul);

compare_fam!(compare_int28pl,  invoke_iq, i16, i64, decode_i64, c_int28pl,  pg_int4_arith::int28pl);
compare_fam!(compare_int28mi,  invoke_iq, i16, i64, decode_i64, c_int28mi,  pg_int4_arith::int28mi);
compare_fam!(compare_int28mul, invoke_iq, i16, i64, decode_i64, c_int28mul, pg_int4_arith::int28mul);

compare_fam!(compare_int82pl,  invoke_qi, i64, i16, decode_i64, c_int82pl,  pg_int4_arith::int82pl);
compare_fam!(compare_int82mi,  invoke_qi, i64, i16, decode_i64, c_int82mi,  pg_int4_arith::int82mi);
compare_fam!(compare_int82mul, invoke_qi, i64, i16, decode_i64, c_int82mul, pg_int4_arith::int82mul);

/* ─────── boundary tests ─────── */

#[test]
fn boundary_int2() {
    // Safe pairs (overflow checks against i16 bounds).
    let safe: &[(i16, i16)] = &[
        (0, 0), (1, -1), (i16::MAX, 0), (i16::MIN, 0),
        (i16::MAX / 2, i16::MAX / 2),
        (i16::MIN / 2, i16::MIN / 2),
    ];
    for &(a, b) in safe {
        compare_int2pl(a, b); compare_int2mi(a, b); compare_int2mul(a, b);
    }
    // Each op gets its own overflow corner.
    let pl_ov: &[(i16, i16)] = &[(i16::MAX, 1), (i16::MIN, -1), (i16::MAX, i16::MAX)];
    for &(a, b) in pl_ov { compare_int2pl(a, b); }
    let mi_ov: &[(i16, i16)] = &[(i16::MIN, 1), (i16::MAX, -1), (0, i16::MIN)];
    for &(a, b) in mi_ov { compare_int2mi(a, b); }
    let mul_ov: &[(i16, i16)] = &[(i16::MAX, 2), (i16::MIN, 2), (i16::MIN, -1), (256, 256)];
    for &(a, b) in mul_ov { compare_int2mul(a, b); }
}

#[test]
fn boundary_int4() {
    let safe: &[(i32, i32)] = &[
        (0, 0), (1, -1), (i32::MAX, 0), (i32::MIN, 0),
        (i32::MAX / 2, i32::MAX / 2),
        (i32::MIN / 2, i32::MIN / 2),
    ];
    for &(a, b) in safe {
        compare_int4pl(a, b); compare_int4mi(a, b); compare_int4mul(a, b);
    }
    let pl_ov: &[(i32, i32)] = &[(i32::MAX, 1), (i32::MAX, i32::MAX), (i32::MIN, -1), (i32::MIN, i32::MIN)];
    for &(a, b) in pl_ov { compare_int4pl(a, b); }
    let mi_ov: &[(i32, i32)] = &[(i32::MIN, 1), (i32::MAX, -1), (0, i32::MIN)];
    for &(a, b) in mi_ov { compare_int4mi(a, b); }
    let mul_ov: &[(i32, i32)] = &[(i32::MAX, 2), (i32::MIN, 2), (i32::MIN, -1), (65536, 65536)];
    for &(a, b) in mul_ov { compare_int4mul(a, b); }
}

#[test]
fn boundary_int8() {
    let safe: &[(i64, i64)] = &[
        (0, 0), (1, -1), (i64::MAX, 0), (i64::MIN, 0),
        (i64::MAX / 2, i64::MAX / 2),
        (i64::MIN / 2, i64::MIN / 2),
    ];
    for &(a, b) in safe {
        compare_int8pl(a, b); compare_int8mi(a, b); compare_int8mul(a, b);
    }
    let pl_ov: &[(i64, i64)] = &[(i64::MAX, 1), (i64::MIN, -1), (i64::MAX, i64::MAX)];
    for &(a, b) in pl_ov { compare_int8pl(a, b); }
    let mi_ov: &[(i64, i64)] = &[(i64::MIN, 1), (i64::MAX, -1), (0, i64::MIN)];
    for &(a, b) in mi_ov { compare_int8mi(a, b); }
    let mul_ov: &[(i64, i64)] = &[(i64::MAX, 2), (i64::MIN, 2), (i64::MIN, -1), (1i64 << 32, 1i64 << 32)];
    for &(a, b) in mul_ov { compare_int8mul(a, b); }
}

#[test]
fn boundary_int24_int42() {
    // int24: (i16, i32) → i32. Overflow possible (e.g. i16::MAX + i32::MAX).
    compare_int24pl(0, 0);
    compare_int24pl(1, -1);
    compare_int24pl(i16::MAX, i32::MAX);     // overflow
    compare_int24pl(i16::MIN, i32::MIN);     // overflow
    compare_int24mi(i16::MAX, i32::MIN);     // overflow
    compare_int24mul(i16::MAX, i32::MAX);    // overflow
    compare_int24mul(i16::MIN, i32::MIN);    // overflow
    // int42: (i32, i16) → i32, mirror.
    compare_int42pl(0, 0);
    compare_int42pl(i32::MAX, i16::MAX);     // overflow
    compare_int42mi(i32::MIN, i16::MAX);     // overflow
    compare_int42mul(i32::MAX, i16::MAX);    // overflow
}

#[test]
fn boundary_int48_int84() {
    // int48: (i32, i64) → i64. Cast widens i32 to i64; only the i64-side
    // overflow primitive can trip.
    compare_int48pl(0, 0);
    compare_int48pl(i32::MAX, i64::MAX);     // overflow
    compare_int48mi(i32::MIN, i64::MAX);     // overflow
    compare_int48mul(i32::MAX, i64::MAX);    // overflow
    // int84 mirror.
    compare_int84pl(i64::MAX, i32::MAX);     // overflow
    compare_int84mi(i64::MIN, i32::MAX);     // overflow
    compare_int84mul(i64::MAX, 2);           // overflow
}

#[test]
fn boundary_int28_int82() {
    // int28: (i16, i64) → i64.
    compare_int28pl(0, 0);
    compare_int28pl(i16::MAX, i64::MAX);     // overflow
    compare_int28mi(i16::MIN, i64::MAX);     // overflow
    compare_int28mul(i16::MAX, i64::MAX);    // overflow
    // int82 mirror.
    compare_int82pl(i64::MAX, i16::MAX);     // overflow
    compare_int82mi(i64::MIN, i16::MAX);     // overflow
    compare_int82mul(i64::MAX, 2);           // overflow
}

/* ─────── proptests (one per family) ─────── */

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    #[test] fn pt_int2pl(a in any::<i16>(), b in any::<i16>())  { compare_int2pl(a, b); }
    #[test] fn pt_int2mi(a in any::<i16>(), b in any::<i16>())  { compare_int2mi(a, b); }
    #[test] fn pt_int2mul(a in any::<i16>(), b in any::<i16>()) { compare_int2mul(a, b); }

    #[test] fn pt_int4pl(a in any::<i32>(), b in any::<i32>())  { compare_int4pl(a, b); }
    #[test] fn pt_int4mi(a in any::<i32>(), b in any::<i32>())  { compare_int4mi(a, b); }
    #[test] fn pt_int4mul(a in any::<i32>(), b in any::<i32>()) { compare_int4mul(a, b); }

    #[test] fn pt_int8pl(a in any::<i64>(), b in any::<i64>())  { compare_int8pl(a, b); }
    #[test] fn pt_int8mi(a in any::<i64>(), b in any::<i64>())  { compare_int8mi(a, b); }
    #[test] fn pt_int8mul(a in any::<i64>(), b in any::<i64>()) { compare_int8mul(a, b); }

    #[test] fn pt_int24pl(a in any::<i16>(), b in any::<i32>())  { compare_int24pl(a, b); }
    #[test] fn pt_int24mi(a in any::<i16>(), b in any::<i32>())  { compare_int24mi(a, b); }
    #[test] fn pt_int24mul(a in any::<i16>(), b in any::<i32>()) { compare_int24mul(a, b); }

    #[test] fn pt_int42pl(a in any::<i32>(), b in any::<i16>())  { compare_int42pl(a, b); }
    #[test] fn pt_int42mi(a in any::<i32>(), b in any::<i16>())  { compare_int42mi(a, b); }
    #[test] fn pt_int42mul(a in any::<i32>(), b in any::<i16>()) { compare_int42mul(a, b); }

    #[test] fn pt_int48pl(a in any::<i32>(), b in any::<i64>())  { compare_int48pl(a, b); }
    #[test] fn pt_int48mi(a in any::<i32>(), b in any::<i64>())  { compare_int48mi(a, b); }
    #[test] fn pt_int48mul(a in any::<i32>(), b in any::<i64>()) { compare_int48mul(a, b); }

    #[test] fn pt_int84pl(a in any::<i64>(), b in any::<i32>())  { compare_int84pl(a, b); }
    #[test] fn pt_int84mi(a in any::<i64>(), b in any::<i32>())  { compare_int84mi(a, b); }
    #[test] fn pt_int84mul(a in any::<i64>(), b in any::<i32>()) { compare_int84mul(a, b); }

    #[test] fn pt_int28pl(a in any::<i16>(), b in any::<i64>())  { compare_int28pl(a, b); }
    #[test] fn pt_int28mi(a in any::<i16>(), b in any::<i64>())  { compare_int28mi(a, b); }
    #[test] fn pt_int28mul(a in any::<i16>(), b in any::<i64>()) { compare_int28mul(a, b); }

    #[test] fn pt_int82pl(a in any::<i64>(), b in any::<i16>())  { compare_int82pl(a, b); }
    #[test] fn pt_int82mi(a in any::<i64>(), b in any::<i16>())  { compare_int82mi(a, b); }
    #[test] fn pt_int82mul(a in any::<i64>(), b in any::<i16>()) { compare_int82mul(a, b); }
}
