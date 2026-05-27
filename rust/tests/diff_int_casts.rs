//! Diff-test for the Pg.Ir-emitted integer width-cast V1 fmgr cluster.

use pg_fcinfo::{
    build_fcinfo, decode_i16, decode_i32, decode_i64,
    fmgr_clear_error, fmgr_take_error, FmgrErrorKind,
    FunctionCallInfoBaseData,
};
use proptest::prelude::*;

type FmgrFn = unsafe extern "C" fn(*mut FunctionCallInfoBaseData) -> u64;

extern "C" {
    fn c_i2toi4(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_i4toi2(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int28(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int48(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int82(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int84(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
}

fn invoke_i16_to_i32(fn_: FmgrFn, a: i16) -> (u64, Option<FmgrErrorKind>) {
    fmgr_clear_error();
    let mut fcinfo = build_fcinfo!(args = [(i16, a)]);
    let out = unsafe { fn_(&mut fcinfo) };
    (out, fmgr_take_error())
}

fn invoke_i32_to_i16(fn_: FmgrFn, a: i32) -> (u64, Option<FmgrErrorKind>) {
    fmgr_clear_error();
    let mut fcinfo = build_fcinfo!(args = [(i32, a)]);
    let out = unsafe { fn_(&mut fcinfo) };
    (out, fmgr_take_error())
}

fn invoke_i16_to_i64(fn_: FmgrFn, a: i16) -> (u64, Option<FmgrErrorKind>) {
    fmgr_clear_error();
    let mut fcinfo = build_fcinfo!(args = [(i16, a)]);
    let out = unsafe { fn_(&mut fcinfo) };
    (out, fmgr_take_error())
}

fn invoke_i32_to_i64(fn_: FmgrFn, a: i32) -> (u64, Option<FmgrErrorKind>) {
    fmgr_clear_error();
    let mut fcinfo = build_fcinfo!(args = [(i32, a)]);
    let out = unsafe { fn_(&mut fcinfo) };
    (out, fmgr_take_error())
}

fn invoke_i64_to_i16(fn_: FmgrFn, a: i64) -> (u64, Option<FmgrErrorKind>) {
    fmgr_clear_error();
    let mut fcinfo = build_fcinfo!(args = [(i64, a)]);
    let out = unsafe { fn_(&mut fcinfo) };
    (out, fmgr_take_error())
}

fn invoke_i64_to_i32(fn_: FmgrFn, a: i64) -> (u64, Option<FmgrErrorKind>) {
    fmgr_clear_error();
    let mut fcinfo = build_fcinfo!(args = [(i64, a)]);
    let out = unsafe { fn_(&mut fcinfo) };
    (out, fmgr_take_error())
}

macro_rules! compare_cast {
    ($name:ident, $invoke:ident, $input_ty:ty, $output_ty:ty, $in_decode:expr, $out_decode:expr, $c_fn:ident, $r_fn:path) => {
        fn $name(a: $input_ty) {
            let (c_out, c_err) = $invoke($c_fn, a);
            let (r_out, r_err) = $invoke($r_fn, a);
            assert_eq!(c_err, r_err,
                "{}({a}): C err = {c_err:?}, Rust err = {r_err:?}", stringify!($name));
            if r_err.is_none() {
                assert_eq!(c_out, r_out,
                    "{}({a}): C={c_out} Rust={r_out} (decoded C={} R={})",
                    stringify!($name), $out_decode(c_out), $out_decode(r_out));
            }
        }
    };
}

compare_cast!(cmp_i2toi4, invoke_i16_to_i32, i16, i32, decode_i16, decode_i32, c_i2toi4, pg_int_casts::i2toi4);
compare_cast!(cmp_i4toi2, invoke_i32_to_i16, i32, i16, decode_i32, decode_i16, c_i4toi2, pg_int_casts::i4toi2);
compare_cast!(cmp_int28, invoke_i16_to_i64, i16, i64, decode_i16, decode_i64, c_int28, pg_int_casts::int28);
compare_cast!(cmp_int48, invoke_i32_to_i64, i32, i64, decode_i32, decode_i64, c_int48, pg_int_casts::int48);
compare_cast!(cmp_int82, invoke_i64_to_i16, i64, i16, decode_i64, decode_i16, c_int82, pg_int_casts::int82);
compare_cast!(cmp_int84, invoke_i64_to_i32, i64, i32, decode_i64, decode_i32, c_int84, pg_int_casts::int84);

#[test]
fn boundary_widening() {
    // Widening casts never overflow
    for a in &[0i16, 1, -1, i16::MAX, i16::MIN] { cmp_i2toi4(*a); }
    for a in &[0i32, 1, -1, i32::MAX, i32::MIN] { cmp_int48(*a); }
    for a in &[0i16, 1, -1, i16::MAX, i16::MIN] { cmp_int28(*a); }
}

#[test]
fn boundary_narrowing() {
    // Narrowing casts check bounds
    for a in &[0i32, 1, -1, i32::MAX, i32::MIN] { cmp_i4toi2(*a); }
    for a in &[0i64, 1, -1, i64::MAX, i64::MIN] { cmp_int82(*a); }
    for a in &[0i64, 1, -1, i64::MAX, i64::MIN] { cmp_int84(*a); }
}

#[test]
fn overflow_i4toi2_on_both() {
    // i4toi2 overflows on values > i16::MAX
    let (_, c_err) = invoke_i32_to_i16(c_i4toi2, i32::MAX);
    assert_eq!(c_err, Some(FmgrErrorKind::NumericValueOutOfRange));
    let (_, r_err) = invoke_i32_to_i16(pg_int_casts::i4toi2, i32::MAX);
    assert_eq!(r_err, Some(FmgrErrorKind::NumericValueOutOfRange));
}

#[test]
fn overflow_int82_on_both() {
    // int82 overflows on values > i16::MAX
    let (_, c_err) = invoke_i64_to_i16(c_int82, i64::MAX);
    assert_eq!(c_err, Some(FmgrErrorKind::NumericValueOutOfRange));
    let (_, r_err) = invoke_i64_to_i16(pg_int_casts::int82, i64::MAX);
    assert_eq!(r_err, Some(FmgrErrorKind::NumericValueOutOfRange));
}

#[test]
fn overflow_int84_on_both() {
    // int84 overflows on values > i32::MAX
    let (_, c_err) = invoke_i64_to_i32(c_int84, i64::MAX);
    assert_eq!(c_err, Some(FmgrErrorKind::NumericValueOutOfRange));
    let (_, r_err) = invoke_i64_to_i32(pg_int_casts::int84, i64::MAX);
    assert_eq!(r_err, Some(FmgrErrorKind::NumericValueOutOfRange));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2048))]

    #[test] fn pt_i2toi4(a in any::<i16>())  { cmp_i2toi4(a); }
    #[test] fn pt_i4toi2(a in any::<i32>())  { cmp_i4toi2(a); }
    #[test] fn pt_int28(a in any::<i16>())   { cmp_int28(a); }
    #[test] fn pt_int48(a in any::<i32>())   { cmp_int48(a); }
    #[test] fn pt_int82(a in any::<i64>())   { cmp_int82(a); }
    #[test] fn pt_int84(a in any::<i64>())   { cmp_int84(a); }
}
