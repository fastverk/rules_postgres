//! Differential test for the integer-type hash V1 fmgr cluster.
//!
//! For each family (hashchar / hashint2 / hashint4 / hashint8 /
//! hashoid / hashenum), build an fcinfo with a single random arg,
//! invoke the Rust translation AND the vendored Postgres C body (via
//! the `c_*` rename wrapper), and assert the returned Datum is
//! bit-identical.
//!
//! Cluster lift: same shape as pg_int4_cmp's diff_int4_cmp.rs but with
//! 1-arg → uint32 (hashint8 also uses 1 arg but with int64 splitting
//! before the hash call). Extended variants (hashint4extended etc.)
//! are deferred to v1.

use pg_fcinfo::{build_fcinfo, decode_u32, FunctionCallInfoBaseData};
use proptest::prelude::*;

extern "C" {
    fn c_hashchar(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_hashint2(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_hashint4(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_hashint8(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_hashoid(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_hashenum(fcinfo: *mut FunctionCallInfoBaseData) -> u64;

    fn c_hashcharextended(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_hashint2extended(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_hashint4extended(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_hashint8extended(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_hashoidextended(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_hashenumextended(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
}

type FmgrFn = unsafe extern "C" fn(*mut FunctionCallInfoBaseData) -> u64;

fn assert_hash_eq(name: &str, c_out: u64, r_out: u64, repr: String) {
    // Hash functions return a uint32 packaged as Datum; the upper bits
    // of the u64 should be zero for both. The whole-u64 compare is the
    // strictest check.
    assert_eq!(
        c_out, r_out,
        "{name}({repr}): C={c_out} (u32={}) Rust={r_out} (u32={})",
        decode_u32(c_out),
        decode_u32(r_out),
    );
}

fn compare_hash_i8(name: &str, c_fn: FmgrFn, r_fn: FmgrFn, a: i8) {
    let mut fcinfo_c = build_fcinfo!(args = [(i8, a)]);
    let mut fcinfo_r = build_fcinfo!(args = [(i8, a)]);
    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { r_fn(&mut fcinfo_r) };
    assert_hash_eq(name, c_out, r_out, format!("i8 {a}"));
}

fn compare_hash_i16(name: &str, c_fn: FmgrFn, r_fn: FmgrFn, a: i16) {
    let mut fcinfo_c = build_fcinfo!(args = [(i16, a)]);
    let mut fcinfo_r = build_fcinfo!(args = [(i16, a)]);
    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { r_fn(&mut fcinfo_r) };
    assert_hash_eq(name, c_out, r_out, format!("i16 {a}"));
}

fn compare_hash_i32(name: &str, c_fn: FmgrFn, r_fn: FmgrFn, a: i32) {
    let mut fcinfo_c = build_fcinfo!(args = [(i32, a)]);
    let mut fcinfo_r = build_fcinfo!(args = [(i32, a)]);
    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { r_fn(&mut fcinfo_r) };
    assert_hash_eq(name, c_out, r_out, format!("i32 {a}"));
}

fn compare_hash_i64(name: &str, c_fn: FmgrFn, r_fn: FmgrFn, a: i64) {
    let mut fcinfo_c = build_fcinfo!(args = [(i64, a)]);
    let mut fcinfo_r = build_fcinfo!(args = [(i64, a)]);
    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { r_fn(&mut fcinfo_r) };
    assert_hash_eq(name, c_out, r_out, format!("i64 {a}"));
}

fn compare_hash_u32(name: &str, c_fn: FmgrFn, r_fn: FmgrFn, a: u32) {
    let mut fcinfo_c = build_fcinfo!(args = [(u32, a)]);
    let mut fcinfo_r = build_fcinfo!(args = [(u32, a)]);
    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { r_fn(&mut fcinfo_r) };
    assert_hash_eq(name, c_out, r_out, format!("u32 {a}"));
}

// ─── Extended variants: 2-arg (key, seed) → uint64 ──────────────────────────
//
// hash_uint32_extended returns a full uint64 Datum, so the upper 32 bits
// of the returned u64 carry the high half of the hash (not zero like
// the non-extended variants). The whole-u64 compare is the right check.

fn compare_hash_ext_i8(name: &str, c_fn: FmgrFn, r_fn: FmgrFn, a: i8, seed: i64) {
    let mut fcinfo_c = build_fcinfo!(args = [(i8, a), (i64, seed)]);
    let mut fcinfo_r = build_fcinfo!(args = [(i8, a), (i64, seed)]);
    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { r_fn(&mut fcinfo_r) };
    assert_hash_eq(name, c_out, r_out, format!("i8 {a}, seed {seed}"));
}

fn compare_hash_ext_i16(name: &str, c_fn: FmgrFn, r_fn: FmgrFn, a: i16, seed: i64) {
    let mut fcinfo_c = build_fcinfo!(args = [(i16, a), (i64, seed)]);
    let mut fcinfo_r = build_fcinfo!(args = [(i16, a), (i64, seed)]);
    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { r_fn(&mut fcinfo_r) };
    assert_hash_eq(name, c_out, r_out, format!("i16 {a}, seed {seed}"));
}

fn compare_hash_ext_i32(name: &str, c_fn: FmgrFn, r_fn: FmgrFn, a: i32, seed: i64) {
    let mut fcinfo_c = build_fcinfo!(args = [(i32, a), (i64, seed)]);
    let mut fcinfo_r = build_fcinfo!(args = [(i32, a), (i64, seed)]);
    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { r_fn(&mut fcinfo_r) };
    assert_hash_eq(name, c_out, r_out, format!("i32 {a}, seed {seed}"));
}

fn compare_hash_ext_i64(name: &str, c_fn: FmgrFn, r_fn: FmgrFn, a: i64, seed: i64) {
    let mut fcinfo_c = build_fcinfo!(args = [(i64, a), (i64, seed)]);
    let mut fcinfo_r = build_fcinfo!(args = [(i64, a), (i64, seed)]);
    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { r_fn(&mut fcinfo_r) };
    assert_hash_eq(name, c_out, r_out, format!("i64 {a}, seed {seed}"));
}

fn compare_hash_ext_u32(name: &str, c_fn: FmgrFn, r_fn: FmgrFn, a: u32, seed: i64) {
    let mut fcinfo_c = build_fcinfo!(args = [(u32, a), (i64, seed)]);
    let mut fcinfo_r = build_fcinfo!(args = [(u32, a), (i64, seed)]);
    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { r_fn(&mut fcinfo_r) };
    assert_hash_eq(name, c_out, r_out, format!("u32 {a}, seed {seed}"));
}

// ─── Boundary cases ──────────────────────────────────────────────────────────

#[test]
fn boundary_cases_hash() {
    // hashchar
    for &a in &[0i8, 1, -1, i8::MIN, i8::MAX] {
        compare_hash_i8("hashchar", c_hashchar, pg_int4_hash::hashchar, a);
    }
    // hashint2
    for &a in &[0i16, 1, -1, i16::MIN, i16::MAX] {
        compare_hash_i16("hashint2", c_hashint2, pg_int4_hash::hashint2, a);
    }
    // hashint4
    for &a in &[0i32, 1, -1, i32::MIN, i32::MAX] {
        compare_hash_i32("hashint4", c_hashint4, pg_int4_hash::hashint4, a);
    }
    // hashint8 — boundary at the int4 splitting points.
    for &a in &[
        0i64, 1, -1, i64::MIN, i64::MAX,
        i32::MIN as i64, i32::MAX as i64,
        0x1_0000_0000, -0x1_0000_0000,
    ] {
        compare_hash_i64("hashint8", c_hashint8, pg_int4_hash::hashint8, a);
    }
    // hashoid / hashenum
    for &a in &[0u32, 1, u32::MAX, u32::MAX - 1] {
        compare_hash_u32("hashoid",  c_hashoid,  pg_int4_hash::hashoid,  a);
        compare_hash_u32("hashenum", c_hashenum, pg_int4_hash::hashenum, a);
    }
}

#[test]
fn boundary_cases_hash_extended() {
    // seed=0 exercises the "skip the mix() prelude" branch in
    // hash_uint32_extended; seed=i64::MIN, MAX, and 1 exercise the
    // mix() path with corner-value seeds. For each width, pair the
    // typical extreme values with each seed.
    let seeds: [i64; 5] = [0, 1, -1, i64::MIN, i64::MAX];
    for &seed in &seeds {
        for &a in &[0i8, 1, -1, i8::MIN, i8::MAX] {
            compare_hash_ext_i8("hashcharextended", c_hashcharextended,
                pg_int4_hash::hashcharextended, a, seed);
        }
        for &a in &[0i16, 1, -1, i16::MIN, i16::MAX] {
            compare_hash_ext_i16("hashint2extended", c_hashint2extended,
                pg_int4_hash::hashint2extended, a, seed);
        }
        for &a in &[0i32, 1, -1, i32::MIN, i32::MAX] {
            compare_hash_ext_i32("hashint4extended", c_hashint4extended,
                pg_int4_hash::hashint4extended, a, seed);
        }
        for &a in &[
            0i64, 1, -1, i64::MIN, i64::MAX,
            i32::MIN as i64, i32::MAX as i64,
            0x1_0000_0000, -0x1_0000_0000,
        ] {
            compare_hash_ext_i64("hashint8extended", c_hashint8extended,
                pg_int4_hash::hashint8extended, a, seed);
        }
        for &a in &[0u32, 1, u32::MAX, u32::MAX - 1] {
            compare_hash_ext_u32("hashoidextended",  c_hashoidextended,
                pg_int4_hash::hashoidextended,  a, seed);
            compare_hash_ext_u32("hashenumextended", c_hashenumextended,
                pg_int4_hash::hashenumextended, a, seed);
        }
    }
}

// ─── Proptest sweeps ─────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2048))]

    #[test]
    fn diff_hashchar_proptest(a in any::<i8>()) {
        compare_hash_i8("hashchar", c_hashchar, pg_int4_hash::hashchar, a);
    }

    #[test]
    fn diff_hashint2_proptest(a in any::<i16>()) {
        compare_hash_i16("hashint2", c_hashint2, pg_int4_hash::hashint2, a);
    }

    #[test]
    fn diff_hashint4_proptest(a in any::<i32>()) {
        compare_hash_i32("hashint4", c_hashint4, pg_int4_hash::hashint4, a);
    }

    #[test]
    fn diff_hashint8_proptest(a in any::<i64>()) {
        compare_hash_i64("hashint8", c_hashint8, pg_int4_hash::hashint8, a);
    }

    #[test]
    fn diff_hashoid_proptest(a in any::<u32>()) {
        compare_hash_u32("hashoid", c_hashoid, pg_int4_hash::hashoid, a);
    }

    #[test]
    fn diff_hashenum_proptest(a in any::<u32>()) {
        compare_hash_u32("hashenum", c_hashenum, pg_int4_hash::hashenum, a);
    }

    // ─── *extended sweeps ────────────────────────────────────────────────

    #[test]
    fn diff_hashcharextended_proptest(a in any::<i8>(), seed in any::<i64>()) {
        compare_hash_ext_i8("hashcharextended", c_hashcharextended,
            pg_int4_hash::hashcharextended, a, seed);
    }

    #[test]
    fn diff_hashint2extended_proptest(a in any::<i16>(), seed in any::<i64>()) {
        compare_hash_ext_i16("hashint2extended", c_hashint2extended,
            pg_int4_hash::hashint2extended, a, seed);
    }

    #[test]
    fn diff_hashint4extended_proptest(a in any::<i32>(), seed in any::<i64>()) {
        compare_hash_ext_i32("hashint4extended", c_hashint4extended,
            pg_int4_hash::hashint4extended, a, seed);
    }

    #[test]
    fn diff_hashint8extended_proptest(a in any::<i64>(), seed in any::<i64>()) {
        compare_hash_ext_i64("hashint8extended", c_hashint8extended,
            pg_int4_hash::hashint8extended, a, seed);
    }

    #[test]
    fn diff_hashoidextended_proptest(a in any::<u32>(), seed in any::<i64>()) {
        compare_hash_ext_u32("hashoidextended", c_hashoidextended,
            pg_int4_hash::hashoidextended, a, seed);
    }

    #[test]
    fn diff_hashenumextended_proptest(a in any::<u32>(), seed in any::<i64>()) {
        compare_hash_ext_u32("hashenumextended", c_hashenumextended,
            pg_int4_hash::hashenumextended, a, seed);
    }
}
