//! Diff-test for the Pg.Ir-emitted float unary V1 fmgr cluster.

use pg_fcinfo::{
    build_fcinfo, decode_f32, decode_f64,
    fmgr_clear_error, fmgr_take_error, FmgrErrorKind,
    FunctionCallInfoBaseData,
};
use proptest::prelude::*;

type FmgrFn = unsafe extern "C" fn(*mut FunctionCallInfoBaseData) -> u64;

extern "C" {
    fn c_float4abs(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_float4um(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_float8abs(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_float8um(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
}

fn invoke_f32(fn_: FmgrFn, a: f32) -> (u64, Option<FmgrErrorKind>) {
    fmgr_clear_error();
    let mut fcinfo = build_fcinfo!(args = [(f32, a)]);
    let out = unsafe { fn_(&mut fcinfo) };
    (out, fmgr_take_error())
}

fn invoke_f64(fn_: FmgrFn, a: f64) -> (u64, Option<FmgrErrorKind>) {
    fmgr_clear_error();
    let mut fcinfo = build_fcinfo!(args = [(f64, a)]);
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
                // For floats, we compare bit-for-bit (Datum encoding).
                // For NaN, both sides should return the same bit pattern.
                assert_eq!(c_out, r_out,
                    "{}({a}): C={c_out:064b} Rust={r_out:064b} (decoded C={} R={})",
                    stringify!($name), $decode(c_out), $decode(r_out));
            }
        }
    };
}

compare_fam!(cmp_float4abs, invoke_f32, f32, decode_f32, c_float4abs, pg_float_unary::float4abs);
compare_fam!(cmp_float4um,  invoke_f32, f32, decode_f32, c_float4um,  pg_float_unary::float4um);
compare_fam!(cmp_float8abs, invoke_f64, f64, decode_f64, c_float8abs, pg_float_unary::float8abs);
compare_fam!(cmp_float8um,  invoke_f64, f64, decode_f64, c_float8um,  pg_float_unary::float8um);

#[test]
fn boundary_f32_abs() {
    let vals = &[
        0.0_f32,
        -0.0_f32,
        1.0_f32,
        -1.0_f32,
        f32::MAX,
        f32::MIN,
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
    ];
    for &a in vals {
        cmp_float4abs(a);
    }
}

#[test]
fn boundary_f32_um() {
    let vals = &[
        0.0_f32,
        -0.0_f32,
        1.0_f32,
        -1.0_f32,
        f32::MAX,
        f32::MIN,
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
    ];
    for &a in vals {
        cmp_float4um(a);
    }
}

#[test]
fn boundary_f64_abs() {
    let vals = &[
        0.0_f64,
        -0.0_f64,
        1.0_f64,
        -1.0_f64,
        f64::MAX,
        f64::MIN,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ];
    for &a in vals {
        cmp_float8abs(a);
    }
}

#[test]
fn boundary_f64_um() {
    let vals = &[
        0.0_f64,
        -0.0_f64,
        1.0_f64,
        -1.0_f64,
        f64::MAX,
        f64::MIN,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ];
    for &a in vals {
        cmp_float8um(a);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2048))]

    #[test] fn pt_float4abs(a in any::<f32>())  { cmp_float4abs(a); }
    #[test] fn pt_float4um(a in any::<f32>())   { cmp_float4um(a); }
    #[test] fn pt_float8abs(a in any::<f64>())  { cmp_float8abs(a); }
    #[test] fn pt_float8um(a in any::<f64>())   { cmp_float8um(a); }
}
