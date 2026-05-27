//! Diff-test for the Pg.Ir-emitted int unary V1 fmgr cluster.

use pg_fcinfo::{
    build_fcinfo, decode_i16, decode_i32, decode_i64,
    fmgr_clear_error, fmgr_take_error, FmgrErrorKind,
    FunctionCallInfoBaseData,
};
use proptest::prelude::*;

type FmgrFn = unsafe extern "C" fn(*mut FunctionCallInfoBaseData) -> u64;

extern "C" {
    fn c_int2um(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int2abs(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int4um(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int4abs(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int8um(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int8abs(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
}

fn invoke_i16(fn_: FmgrFn, a: i16) -> (u64, Option<FmgrErrorKind>) {
    fmgr_clear_error();
    let mut fcinfo = build_fcinfo!(args = [(i16, a)]);
    let out = unsafe { fn_(&mut fcinfo) };
    (out, fmgr_take_error())
}

fn invoke_i32(fn_: FmgrFn, a: i32) -> (u64, Option<FmgrErrorKind>) {
    fmgr_clear_error();
    let mut fcinfo = build_fcinfo!(args = [(i32, a)]);
    let out = unsafe { fn_(&mut fcinfo) };
    (out, fmgr_take_error())
}

fn invoke_i64(fn_: FmgrFn, a: i64) -> (u64, Option<FmgrErrorKind>) {
    fmgr_clear_error();
    let mut fcinfo = build_fcinfo!(args = [(i64, a)]);
    let out = unsafe { fn_(&mut fcinfo) };
    (out, fmgr_take_error())
}

macro_rules! compare_fam {
    ($name:ident, $invoke:ident, $ty:ty, $decode:expr, $c_fn:ident, $r_fn:path) => {
        fn $name(a: $ty) {
            let (c_out, c_err) = $invoke($c_fn, a);
            let (r_out, r_err) = $invoke($r_fn, a);
            assert_eq!(c_err, r_err,
                "{}({a}): C err = {c_err:?}, Rust err = {r_err:?}", stringify!($name));
            if r_err.is_none() {
                assert_eq!(c_out, r_out,
                    "{}({a}): C={c_out} Rust={r_out} (decoded C={} R={})",
                    stringify!($name), $decode(c_out), $decode(r_out));
            }
        }
    };
}

compare_fam!(cmp_int2um,  invoke_i16, i16, decode_i16, c_int2um,  pg_int4_unary::int2um);
compare_fam!(cmp_int2abs, invoke_i16, i16, decode_i16, c_int2abs, pg_int4_unary::int2abs);
compare_fam!(cmp_int4um,  invoke_i32, i32, decode_i32, c_int4um,  pg_int4_unary::int4um);
compare_fam!(cmp_int4abs, invoke_i32, i32, decode_i32, c_int4abs, pg_int4_unary::int4abs);
compare_fam!(cmp_int8um,  invoke_i64, i64, decode_i64, c_int8um,  pg_int4_unary::int8um);
compare_fam!(cmp_int8abs, invoke_i64, i64, decode_i64, c_int8abs, pg_int4_unary::int8abs);

#[test]
fn boundary_um() {
    for a in &[0i16, 1, -1, i16::MAX, i16::MIN] { cmp_int2um(*a); }
    for a in &[0i32, 1, -1, i32::MAX, i32::MIN] { cmp_int4um(*a); }
    for a in &[0i64, 1, -1, i64::MAX, i64::MIN] { cmp_int8um(*a); }
}

#[test]
fn boundary_abs() {
    for a in &[0i16, 1, -1, i16::MAX, i16::MIN] { cmp_int2abs(*a); }
    for a in &[0i32, 1, -1, i32::MAX, i32::MIN] { cmp_int4abs(*a); }
    for a in &[0i64, 1, -1, i64::MAX, i64::MIN] { cmp_int8abs(*a); }
}

#[test]
fn overflow_um_corner_raises_on_both() {
    let (_, c_err) = invoke_i16(c_int2um, i16::MIN);
    assert_eq!(c_err, Some(FmgrErrorKind::NumericValueOutOfRange));
    let (_, r_err) = invoke_i16(pg_int4_unary::int2um, i16::MIN);
    assert_eq!(r_err, Some(FmgrErrorKind::NumericValueOutOfRange));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2048))]

    #[test] fn pt_int2um(a in any::<i16>())  { cmp_int2um(a); }
    #[test] fn pt_int2abs(a in any::<i16>()) { cmp_int2abs(a); }
    #[test] fn pt_int4um(a in any::<i32>())  { cmp_int4um(a); }
    #[test] fn pt_int4abs(a in any::<i32>()) { cmp_int4abs(a); }
    #[test] fn pt_int8um(a in any::<i64>())  { cmp_int8um(a); }
    #[test] fn pt_int8abs(a in any::<i64>()) { cmp_int8abs(a); }
}
