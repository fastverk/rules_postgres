//! Diff-test for the Pg.Ir-emitted float-to-int cast V1 fmgr cluster.

use pg_fcinfo::{
    build_fcinfo, decode_i16, decode_i32, decode_f32, decode_f64,
    fmgr_clear_error, fmgr_take_error, FmgrErrorKind,
    FunctionCallInfoBaseData,
};
use proptest::prelude::*;

type FmgrFn = unsafe extern "C" fn(*mut FunctionCallInfoBaseData) -> u64;

extern "C" {
    fn c_ftoi2(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_ftoi4(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_dtoi2(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_dtoi4(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
}

fn invoke_f32_to_i16(fn_: FmgrFn, a: f32) -> (u64, Option<FmgrErrorKind>) {
    fmgr_clear_error();
    let mut fcinfo = build_fcinfo!(args = [(f32, a)]);
    let out = unsafe { fn_(&mut fcinfo) };
    (out, fmgr_take_error())
}

fn invoke_f32_to_i32(fn_: FmgrFn, a: f32) -> (u64, Option<FmgrErrorKind>) {
    fmgr_clear_error();
    let mut fcinfo = build_fcinfo!(args = [(f32, a)]);
    let out = unsafe { fn_(&mut fcinfo) };
    (out, fmgr_take_error())
}

fn invoke_f64_to_i16(fn_: FmgrFn, a: f64) -> (u64, Option<FmgrErrorKind>) {
    fmgr_clear_error();
    let mut fcinfo = build_fcinfo!(args = [(f64, a)]);
    let out = unsafe { fn_(&mut fcinfo) };
    (out, fmgr_take_error())
}

fn invoke_f64_to_i32(fn_: FmgrFn, a: f64) -> (u64, Option<FmgrErrorKind>) {
    fmgr_clear_error();
    let mut fcinfo = build_fcinfo!(args = [(f64, a)]);
    let out = unsafe { fn_(&mut fcinfo) };
    (out, fmgr_take_error())
}

macro_rules! compare_cast {
    ($name:ident, $invoke:ident, $ty:ty, $decode:expr, $c_fn:ident, $r_fn:path) => {
        fn $name(a: $ty) {
            let (c_out, c_err) = $invoke($c_fn, a);
            let (r_out, r_err) = $invoke($r_fn, a);
            assert_eq!(c_err, r_err,
                "{}({a}): C err = {c_err:?}, Rust err = {r_err:?}", stringify!($name));
            if r_err.is_none() {
                assert_eq!(c_out, r_out,
                    "{}({a}): C={} Rust={}", stringify!($name), $decode(c_out), $decode(r_out));
            }
        }
    };
}

compare_cast!(cmp_ftoi2, invoke_f32_to_i16, f32, decode_i16, c_ftoi2, pg_float_to_int::ftoi2);
compare_cast!(cmp_ftoi4, invoke_f32_to_i32, f32, decode_i32, c_ftoi4, pg_float_to_int::ftoi4);
compare_cast!(cmp_dtoi2, invoke_f64_to_i16, f64, decode_i16, c_dtoi2, pg_float_to_int::dtoi2);
compare_cast!(cmp_dtoi4, invoke_f64_to_i32, f64, decode_i32, c_dtoi4, pg_float_to_int::dtoi4);

#[test]
fn boundary_f32_to_i16() {
    let vals = &[
        // Normal values
        0.0_f32, -0.0_f32, 1.0_f32, -1.0_f32, 100.5_f32, -100.5_f32,
        // Boundaries for int16
        32767.0_f32, 32768.0_f32, -32768.0_f32, -32769.0_f32,
        // Out of range
        100000.0_f32, -100000.0_f32,
        // Special values
        f32::NAN, f32::INFINITY, f32::NEG_INFINITY,
    ];
    for &a in vals {
        cmp_ftoi2(a);
    }
}

#[test]
fn boundary_f32_to_i32() {
    let vals = &[
        // Normal values
        0.0_f32, -0.0_f32, 1.0_f32, -1.0_f32, 1000.5_f32, -1000.5_f32,
        // Boundaries for int32
        2147483647.0_f32, 2147483648.0_f32, -2147483648.0_f32, -2147483649.0_f32,
        // Out of range
        f32::MAX, f32::MIN,
        // Special values
        f32::NAN, f32::INFINITY, f32::NEG_INFINITY,
    ];
    for &a in vals {
        cmp_ftoi4(a);
    }
}

#[test]
fn boundary_f64_to_i16() {
    let vals = &[
        // Normal values
        0.0_f64, -0.0_f64, 1.0_f64, -1.0_f64, 100.5_f64, -100.5_f64,
        // Boundaries for int16
        32767.0_f64, 32768.0_f64, -32768.0_f64, -32769.0_f64,
        // Out of range
        100000.0_f64, -100000.0_f64,
        // Special values
        f64::NAN, f64::INFINITY, f64::NEG_INFINITY,
    ];
    for &a in vals {
        cmp_dtoi2(a);
    }
}

#[test]
fn boundary_f64_to_i32() {
    let vals = &[
        // Normal values
        0.0_f64, -0.0_f64, 1.0_f64, -1.0_f64, 1000.5_f64, -1000.5_f64,
        // Boundaries for int32
        2147483647.0_f64, 2147483648.0_f64, -2147483648.0_f64, -2147483649.0_f64,
        // Out of range
        f64::MAX, f64::MIN,
        // Special values
        f64::NAN, f64::INFINITY, f64::NEG_INFINITY,
    ];
    for &a in vals {
        cmp_dtoi4(a);
    }
}
