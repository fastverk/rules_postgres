//! Diff-test for the Pg.Ir-emitted float arithmetic V1 fmgr cluster.

use pg_fcinfo::{
    build_fcinfo, decode_f32, decode_f64,
    fmgr_clear_error, fmgr_take_error, FmgrErrorKind,
    FunctionCallInfoBaseData,
};
use proptest::prelude::*;

type FmgrFn = unsafe extern "C" fn(*mut FunctionCallInfoBaseData) -> u64;

extern "C" {
    fn c_float4pl(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_float4mi(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_float4mul(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_float4div(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_float8pl(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_float8mi(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_float8mul(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_float8div(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
}

fn invoke_f32(fn_: FmgrFn, a: f32, b: f32) -> (u64, Option<FmgrErrorKind>) {
    fmgr_clear_error();
    let mut fcinfo = build_fcinfo!(args = [(f32, a), (f32, b)]);
    let out = unsafe { fn_(&mut fcinfo) };
    (out, fmgr_take_error())
}

fn invoke_f64(fn_: FmgrFn, a: f64, b: f64) -> (u64, Option<FmgrErrorKind>) {
    fmgr_clear_error();
    let mut fcinfo = build_fcinfo!(args = [(f64, a), (f64, b)]);
    let out = unsafe { fn_(&mut fcinfo) };
    (out, fmgr_take_error())
}

macro_rules! compare_fam {
    ($name:ident, $invoke:ident, $ty:ty, $decode:expr, $c_fn:ident, $r_fn:path) => {
        fn $name(a: $ty, b: $ty) {
            let (c_out, c_err) = $invoke($c_fn, a, b);
            let (r_out, r_err) = $invoke($r_fn, a, b);
            assert_eq!(c_err, r_err,
                "{}({a}, {b}): C err = {c_err:?}, Rust err = {r_err:?}", stringify!($name));
            if r_err.is_none() {
                // For floats, we compare bit-for-bit (Datum encoding).
                // For NaN, both sides should return the same bit pattern.
                assert_eq!(c_out, r_out,
                    "{}({a}, {b}): C={c_out:064b} Rust={r_out:064b} (decoded C={} R={})",
                    stringify!($name), $decode(c_out), $decode(r_out));
            }
        }
    };
}

compare_fam!(cmp_float4pl,  invoke_f32, f32, decode_f32, c_float4pl,  pg_float_arith::float4pl);
compare_fam!(cmp_float4mi,  invoke_f32, f32, decode_f32, c_float4mi,  pg_float_arith::float4mi);
compare_fam!(cmp_float4mul, invoke_f32, f32, decode_f32, c_float4mul, pg_float_arith::float4mul);
compare_fam!(cmp_float4div, invoke_f32, f32, decode_f32, c_float4div, pg_float_arith::float4div);
compare_fam!(cmp_float8pl,  invoke_f64, f64, decode_f64, c_float8pl,  pg_float_arith::float8pl);
compare_fam!(cmp_float8mi,  invoke_f64, f64, decode_f64, c_float8mi,  pg_float_arith::float8mi);
compare_fam!(cmp_float8mul, invoke_f64, f64, decode_f64, c_float8mul, pg_float_arith::float8mul);
compare_fam!(cmp_float8div, invoke_f64, f64, decode_f64, c_float8div, pg_float_arith::float8div);

#[test]
fn boundary_f32_pl() {
    let vals = &[
        0.0_f32, -0.0_f32, 1.0_f32, -1.0_f32,
        f32::MAX, f32::MIN, f32::EPSILON,
        f32::NAN, f32::INFINITY, f32::NEG_INFINITY,
    ];
    for &a in vals {
        for &b in vals {
            cmp_float4pl(a, b);
        }
    }
}

#[test]
fn boundary_f32_mi() {
    let vals = &[
        0.0_f32, -0.0_f32, 1.0_f32, -1.0_f32,
        f32::MAX, f32::MIN, f32::EPSILON,
        f32::NAN, f32::INFINITY, f32::NEG_INFINITY,
    ];
    for &a in vals {
        for &b in vals {
            cmp_float4mi(a, b);
        }
    }
}

#[test]
fn boundary_f32_mul() {
    let vals = &[
        0.0_f32, -0.0_f32, 1.0_f32, -1.0_f32,
        f32::MAX, f32::MIN, f32::EPSILON,
        f32::NAN, f32::INFINITY, f32::NEG_INFINITY,
    ];
    for &a in vals {
        for &b in vals {
            cmp_float4mul(a, b);
        }
    }
}

#[test]
fn boundary_f32_div() {
    let vals = &[
        0.0_f32, -0.0_f32, 1.0_f32, -1.0_f32,
        f32::MAX, f32::MIN, f32::EPSILON,
        f32::NAN, f32::INFINITY, f32::NEG_INFINITY,
    ];
    for &a in vals {
        for &b in vals {
            cmp_float4div(a, b);
        }
    }
}

#[test]
fn boundary_f64_pl() {
    let vals = &[
        0.0_f64, -0.0_f64, 1.0_f64, -1.0_f64,
        f64::MAX, f64::MIN, f64::EPSILON,
        f64::NAN, f64::INFINITY, f64::NEG_INFINITY,
    ];
    for &a in vals {
        for &b in vals {
            cmp_float8pl(a, b);
        }
    }
}

#[test]
fn boundary_f64_mi() {
    let vals = &[
        0.0_f64, -0.0_f64, 1.0_f64, -1.0_f64,
        f64::MAX, f64::MIN, f64::EPSILON,
        f64::NAN, f64::INFINITY, f64::NEG_INFINITY,
    ];
    for &a in vals {
        for &b in vals {
            cmp_float8mi(a, b);
        }
    }
}

#[test]
fn boundary_f64_mul() {
    let vals = &[
        0.0_f64, -0.0_f64, 1.0_f64, -1.0_f64,
        f64::MAX, f64::MIN, f64::EPSILON,
        f64::NAN, f64::INFINITY, f64::NEG_INFINITY,
    ];
    for &a in vals {
        for &b in vals {
            cmp_float8mul(a, b);
        }
    }
}

#[test]
fn boundary_f64_div() {
    let vals = &[
        0.0_f64, -0.0_f64, 1.0_f64, -1.0_f64,
        f64::MAX, f64::MIN, f64::EPSILON,
        f64::NAN, f64::INFINITY, f64::NEG_INFINITY,
    ];
    for &a in vals {
        for &b in vals {
            cmp_float8div(a, b);
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2048))]

    #[test] fn pt_float4pl(a in any::<f32>(), b in any::<f32>())  { cmp_float4pl(a, b); }
    #[test] fn pt_float4mi(a in any::<f32>(), b in any::<f32>())  { cmp_float4mi(a, b); }
    #[test] fn pt_float4mul(a in any::<f32>(), b in any::<f32>()) { cmp_float4mul(a, b); }
    #[test] fn pt_float4div(a in any::<f32>(), b in any::<f32>()) { cmp_float4div(a, b); }
    #[test] fn pt_float8pl(a in any::<f64>(), b in any::<f64>())  { cmp_float8pl(a, b); }
    #[test] fn pt_float8mi(a in any::<f64>(), b in any::<f64>())  { cmp_float8mi(a, b); }
    #[test] fn pt_float8mul(a in any::<f64>(), b in any::<f64>()) { cmp_float8mul(a, b); }
    #[test] fn pt_float8div(a in any::<f64>(), b in any::<f64>()) { cmp_float8div(a, b); }
}
