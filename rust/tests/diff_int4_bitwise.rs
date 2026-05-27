//! Diff-test for the Pg.Ir-emitted int bitwise V1 fmgr cluster.

use pg_fcinfo::{build_fcinfo, FunctionCallInfoBaseData};
use proptest::prelude::*;

type FmgrFn = unsafe extern "C" fn(*mut FunctionCallInfoBaseData) -> u64;

extern "C" {
    fn c_int2and(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int2or(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int2xor(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int2not(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int2shl(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int2shr(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int4and(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int4or(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int4xor(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int4not(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int4shl(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int4shr(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int8and(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int8or(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int8xor(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int8not(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int8shl(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int8shr(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
}

// Binary same-width helpers
fn cmp_bin_i16(name: &str, c: FmgrFn, r: FmgrFn, a: i16, b: i16) {
    let mut fc1 = build_fcinfo!(args = [(i16, a), (i16, b)]);
    let mut fc2 = build_fcinfo!(args = [(i16, a), (i16, b)]);
    let c_out = unsafe { c(&mut fc1) }; let r_out = unsafe { r(&mut fc2) };
    assert_eq!(c_out, r_out, "{name}({a},{b})");
}
fn cmp_bin_i32(name: &str, c: FmgrFn, r: FmgrFn, a: i32, b: i32) {
    let mut fc1 = build_fcinfo!(args = [(i32, a), (i32, b)]);
    let mut fc2 = build_fcinfo!(args = [(i32, a), (i32, b)]);
    let c_out = unsafe { c(&mut fc1) }; let r_out = unsafe { r(&mut fc2) };
    assert_eq!(c_out, r_out, "{name}({a},{b})");
}
fn cmp_bin_i64(name: &str, c: FmgrFn, r: FmgrFn, a: i64, b: i64) {
    let mut fc1 = build_fcinfo!(args = [(i64, a), (i64, b)]);
    let mut fc2 = build_fcinfo!(args = [(i64, a), (i64, b)]);
    let c_out = unsafe { c(&mut fc1) }; let r_out = unsafe { r(&mut fc2) };
    assert_eq!(c_out, r_out, "{name}({a},{b})");
}
// Shift helpers (arg2 = i32)
fn cmp_shift_i16(name: &str, c: FmgrFn, r: FmgrFn, a: i16, b: i32) {
    let mut fc1 = build_fcinfo!(args = [(i16, a), (i32, b)]);
    let mut fc2 = build_fcinfo!(args = [(i16, a), (i32, b)]);
    let c_out = unsafe { c(&mut fc1) }; let r_out = unsafe { r(&mut fc2) };
    assert_eq!(c_out, r_out, "{name}({a},{b})");
}
fn cmp_shift_i32(name: &str, c: FmgrFn, r: FmgrFn, a: i32, b: i32) {
    let mut fc1 = build_fcinfo!(args = [(i32, a), (i32, b)]);
    let mut fc2 = build_fcinfo!(args = [(i32, a), (i32, b)]);
    let c_out = unsafe { c(&mut fc1) }; let r_out = unsafe { r(&mut fc2) };
    assert_eq!(c_out, r_out, "{name}({a},{b})");
}
fn cmp_shift_i64(name: &str, c: FmgrFn, r: FmgrFn, a: i64, b: i32) {
    let mut fc1 = build_fcinfo!(args = [(i64, a), (i32, b)]);
    let mut fc2 = build_fcinfo!(args = [(i64, a), (i32, b)]);
    let c_out = unsafe { c(&mut fc1) }; let r_out = unsafe { r(&mut fc2) };
    assert_eq!(c_out, r_out, "{name}({a},{b})");
}
// Unary
fn cmp_not_i16(name: &str, c: FmgrFn, r: FmgrFn, a: i16) {
    let mut fc1 = build_fcinfo!(args = [(i16, a)]);
    let mut fc2 = build_fcinfo!(args = [(i16, a)]);
    let c_out = unsafe { c(&mut fc1) }; let r_out = unsafe { r(&mut fc2) };
    assert_eq!(c_out, r_out, "{name}({a})");
}
fn cmp_not_i32(name: &str, c: FmgrFn, r: FmgrFn, a: i32) {
    let mut fc1 = build_fcinfo!(args = [(i32, a)]);
    let mut fc2 = build_fcinfo!(args = [(i32, a)]);
    let c_out = unsafe { c(&mut fc1) }; let r_out = unsafe { r(&mut fc2) };
    assert_eq!(c_out, r_out, "{name}({a})");
}
fn cmp_not_i64(name: &str, c: FmgrFn, r: FmgrFn, a: i64) {
    let mut fc1 = build_fcinfo!(args = [(i64, a)]);
    let mut fc2 = build_fcinfo!(args = [(i64, a)]);
    let c_out = unsafe { c(&mut fc1) }; let r_out = unsafe { r(&mut fc2) };
    assert_eq!(c_out, r_out, "{name}({a})");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    // and/or/xor — binary same-width
    #[test] fn pt_int2and(a in any::<i16>(), b in any::<i16>()) { cmp_bin_i16("int2and", c_int2and, pg_int4_bitwise::int2and, a, b); }
    #[test] fn pt_int2or (a in any::<i16>(), b in any::<i16>()) { cmp_bin_i16("int2or",  c_int2or,  pg_int4_bitwise::int2or,  a, b); }
    #[test] fn pt_int2xor(a in any::<i16>(), b in any::<i16>()) { cmp_bin_i16("int2xor", c_int2xor, pg_int4_bitwise::int2xor, a, b); }
    #[test] fn pt_int4and(a in any::<i32>(), b in any::<i32>()) { cmp_bin_i32("int4and", c_int4and, pg_int4_bitwise::int4and, a, b); }
    #[test] fn pt_int4or (a in any::<i32>(), b in any::<i32>()) { cmp_bin_i32("int4or",  c_int4or,  pg_int4_bitwise::int4or,  a, b); }
    #[test] fn pt_int4xor(a in any::<i32>(), b in any::<i32>()) { cmp_bin_i32("int4xor", c_int4xor, pg_int4_bitwise::int4xor, a, b); }
    #[test] fn pt_int8and(a in any::<i64>(), b in any::<i64>()) { cmp_bin_i64("int8and", c_int8and, pg_int4_bitwise::int8and, a, b); }
    #[test] fn pt_int8or (a in any::<i64>(), b in any::<i64>()) { cmp_bin_i64("int8or",  c_int8or,  pg_int4_bitwise::int8or,  a, b); }
    #[test] fn pt_int8xor(a in any::<i64>(), b in any::<i64>()) { cmp_bin_i64("int8xor", c_int8xor, pg_int4_bitwise::int8xor, a, b); }

    // not — unary
    #[test] fn pt_int2not(a in any::<i16>()) { cmp_not_i16("int2not", c_int2not, pg_int4_bitwise::int2not, a); }
    #[test] fn pt_int4not(a in any::<i32>()) { cmp_not_i32("int4not", c_int4not, pg_int4_bitwise::int4not, a); }
    #[test] fn pt_int8not(a in any::<i64>()) { cmp_not_i64("int8not", c_int8not, pg_int4_bitwise::int8not, a); }

    // shl/shr — bound arg2 to [0, width-1] so both Rust and C agree
    // unambiguously. (Beyond that, behavior is UB-but-arch-defined in C
    // and well-defined-via-mod-width in Rust's wrapping_shl/_shr; on
    // x86_64 they happen to match, but to keep the proptest portable
    // we stay inside the universally-defined range.)
    #[test] fn pt_int2shl(a in any::<i16>(), b in 0i32..16) { cmp_shift_i16("int2shl", c_int2shl, pg_int4_bitwise::int2shl, a, b); }
    #[test] fn pt_int2shr(a in any::<i16>(), b in 0i32..16) { cmp_shift_i16("int2shr", c_int2shr, pg_int4_bitwise::int2shr, a, b); }
    #[test] fn pt_int4shl(a in any::<i32>(), b in 0i32..32) { cmp_shift_i32("int4shl", c_int4shl, pg_int4_bitwise::int4shl, a, b); }
    #[test] fn pt_int4shr(a in any::<i32>(), b in 0i32..32) { cmp_shift_i32("int4shr", c_int4shr, pg_int4_bitwise::int4shr, a, b); }
    #[test] fn pt_int8shl(a in any::<i64>(), b in 0i32..64) { cmp_shift_i64("int8shl", c_int8shl, pg_int4_bitwise::int8shl, a, b); }
    #[test] fn pt_int8shr(a in any::<i64>(), b in 0i32..64) { cmp_shift_i64("int8shr", c_int8shr, pg_int4_bitwise::int8shr, a, b); }
}
