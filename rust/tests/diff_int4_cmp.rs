//! Differential test for the int{2,4,8} comparison V1 fmgr cluster.
//!
//! For each width × each of 6 operators (eq/ne/lt/le/gt/ge), generate
//! random typed pairs, run both the Rust translation and Postgres'
//! actual C implementation (via the c_* rename wrappers), and assert
//! their returned Datums match bit-for-bit.
//!
//! Cluster scale demonstrated:
//!   - 1 hand-translated canonical (int4eq body)
//!   - 17 mechanical-substitution siblings (5 ops × int4 + 6 ops × int2
//!     + 6 ops × int8)
//!   - 18 corpus entries total, all gated by real-Postgres diff-tests.

use pg_fcinfo::{build_fcinfo, decode_bool, FunctionCallInfoBaseData};
use proptest::prelude::*;

extern "C" {
    fn c_int4eq(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int4ne(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int4lt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int4le(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int4gt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int4ge(fcinfo: *mut FunctionCallInfoBaseData) -> u64;

    fn c_int2eq(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int2ne(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int2lt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int2le(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int2gt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int2ge(fcinfo: *mut FunctionCallInfoBaseData) -> u64;

    fn c_int8eq(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int8ne(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int8lt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int8le(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int8gt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int8ge(fcinfo: *mut FunctionCallInfoBaseData) -> u64;

    fn c_date_eq(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_date_ne(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_date_lt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_date_le(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_date_gt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_date_ge(fcinfo: *mut FunctionCallInfoBaseData) -> u64;

    fn c_timestamp_eq(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_timestamp_ne(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_timestamp_lt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_timestamp_le(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_timestamp_gt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_timestamp_ge(fcinfo: *mut FunctionCallInfoBaseData) -> u64;

    fn c_cash_eq(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_cash_ne(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_cash_lt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_cash_le(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_cash_gt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_cash_ge(fcinfo: *mut FunctionCallInfoBaseData) -> u64;

    fn c_pg_lsn_eq(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_pg_lsn_ne(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_pg_lsn_lt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_pg_lsn_le(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_pg_lsn_gt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_pg_lsn_ge(fcinfo: *mut FunctionCallInfoBaseData) -> u64;

    fn c_oideq(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_oidne(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_oidlt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_oidle(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_oidgt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_oidge(fcinfo: *mut FunctionCallInfoBaseData) -> u64;

    fn c_booleq(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_boolne(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_boollt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_boolle(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_boolgt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_boolge(fcinfo: *mut FunctionCallInfoBaseData) -> u64;

    fn c_float4eq(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_float4ne(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_float4lt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_float4le(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_float4gt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_float4ge(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_float8eq(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_float8ne(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_float8lt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_float8le(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_float8gt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_float8ge(fcinfo: *mut FunctionCallInfoBaseData) -> u64;

    fn c_chareq(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_charne(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_charlt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_charle(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_chargt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_charge(fcinfo: *mut FunctionCallInfoBaseData) -> u64;

    fn c_int24eq(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int24ne(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int24lt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int24le(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int24gt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int24ge(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int42eq(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int42ne(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int42lt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int42le(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int42gt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int42ge(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int28eq(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int28ne(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int28lt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int28le(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int28gt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int28ge(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int82eq(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int82ne(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int82lt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int82le(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int82gt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int82ge(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int48eq(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int48ne(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int48lt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int48le(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int48gt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int48ge(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int84eq(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int84ne(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int84lt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int84le(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int84gt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_int84ge(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
}

type FmgrFn = unsafe extern "C" fn(*mut FunctionCallInfoBaseData) -> u64;

// ─── Per-width comparison helpers ────────────────────────────────────────────

fn compare_i16(name: &str, c_fn: FmgrFn, rust_fn: FmgrFn, a: i16, b: i16) {
    // build_fcinfo!'s i16 tag encodes via encode_i16, which is what
    // Postgres' Int16GetDatum does. The C side's PG_GETARG_INT16
    // decodes symmetrically.
    let mut fcinfo_c = build_fcinfo!(args = [(i16, a), (i16, b)]);
    let mut fcinfo_r = build_fcinfo!(args = [(i16, a), (i16, b)]);
    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { rust_fn(&mut fcinfo_r) };
    assert_eq!(c_out, r_out,
        "{name}(i16 {a}, {b}): C={c_out} (bool={}) Rust={r_out} (bool={})",
        decode_bool(c_out), decode_bool(r_out));
}

fn compare_i32(name: &str, c_fn: FmgrFn, rust_fn: FmgrFn, a: i32, b: i32) {
    let mut fcinfo_c = build_fcinfo!(args = [(i32, a), (i32, b)]);
    let mut fcinfo_r = build_fcinfo!(args = [(i32, a), (i32, b)]);
    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { rust_fn(&mut fcinfo_r) };
    assert_eq!(c_out, r_out,
        "{name}(i32 {a}, {b}): C={c_out} (bool={}) Rust={r_out} (bool={})",
        decode_bool(c_out), decode_bool(r_out));
}

fn compare_i64(name: &str, c_fn: FmgrFn, rust_fn: FmgrFn, a: i64, b: i64) {
    let mut fcinfo_c = build_fcinfo!(args = [(i64, a), (i64, b)]);
    let mut fcinfo_r = build_fcinfo!(args = [(i64, a), (i64, b)]);
    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { rust_fn(&mut fcinfo_r) };
    assert_eq!(c_out, r_out,
        "{name}(i64 {a}, {b}): C={c_out} (bool={}) Rust={r_out} (bool={})",
        decode_bool(c_out), decode_bool(r_out));
}

fn compare_u32(name: &str, c_fn: FmgrFn, rust_fn: FmgrFn, a: u32, b: u32) {
    let mut fcinfo_c = build_fcinfo!(args = [(u32, a), (u32, b)]);
    let mut fcinfo_r = build_fcinfo!(args = [(u32, a), (u32, b)]);
    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { rust_fn(&mut fcinfo_r) };
    assert_eq!(c_out, r_out,
        "{name}(u32 {a}, {b}): C={c_out} (bool={}) Rust={r_out} (bool={})",
        decode_bool(c_out), decode_bool(r_out));
}

fn compare_u64(name: &str, c_fn: FmgrFn, rust_fn: FmgrFn, a: u64, b: u64) {
    let mut fcinfo_c = build_fcinfo!(args = [(u64, a), (u64, b)]);
    let mut fcinfo_r = build_fcinfo!(args = [(u64, a), (u64, b)]);
    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { rust_fn(&mut fcinfo_r) };
    assert_eq!(c_out, r_out,
        "{name}(u64 {a}, {b}): C={c_out} (bool={}) Rust={r_out} (bool={})",
        decode_bool(c_out), decode_bool(r_out));
}

fn compare_bool(name: &str, c_fn: FmgrFn, rust_fn: FmgrFn, a: bool, b: bool) {
    let mut fcinfo_c = build_fcinfo!(args = [(bool, a), (bool, b)]);
    let mut fcinfo_r = build_fcinfo!(args = [(bool, a), (bool, b)]);
    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { rust_fn(&mut fcinfo_r) };
    assert_eq!(c_out, r_out,
        "{name}(bool {a}, {b}): C={c_out} (bool={}) Rust={r_out} (bool={})",
        decode_bool(c_out), decode_bool(r_out));
}

fn compare_f32(name: &str, c_fn: FmgrFn, rust_fn: FmgrFn, a: f32, b: f32) {
    let mut fcinfo_c = build_fcinfo!(args = [(f32, a), (f32, b)]);
    let mut fcinfo_r = build_fcinfo!(args = [(f32, a), (f32, b)]);
    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { rust_fn(&mut fcinfo_r) };
    assert_eq!(c_out, r_out,
        "{name}(f32 {a:e}, {b:e}): C={c_out} (bool={}) Rust={r_out} (bool={})",
        decode_bool(c_out), decode_bool(r_out));
}

fn compare_f64(name: &str, c_fn: FmgrFn, rust_fn: FmgrFn, a: f64, b: f64) {
    let mut fcinfo_c = build_fcinfo!(args = [(f64, a), (f64, b)]);
    let mut fcinfo_r = build_fcinfo!(args = [(f64, a), (f64, b)]);
    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { rust_fn(&mut fcinfo_r) };
    assert_eq!(c_out, r_out,
        "{name}(f64 {a:e}, {b:e}): C={c_out} (bool={}) Rust={r_out} (bool={})",
        decode_bool(c_out), decode_bool(r_out));
}

fn compare_i8(name: &str, c_fn: FmgrFn, rust_fn: FmgrFn, a: i8, b: i8) {
    let mut fcinfo_c = build_fcinfo!(args = [(i8, a), (i8, b)]);
    let mut fcinfo_r = build_fcinfo!(args = [(i8, a), (i8, b)]);
    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { rust_fn(&mut fcinfo_r) };
    assert_eq!(c_out, r_out,
        "{name}(i8 {a}, {b}): C={c_out} (bool={}) Rust={r_out} (bool={})",
        decode_bool(c_out), decode_bool(r_out));
}

// Cross-type helpers — arg widths differ. The fcinfo arg slots are
// independently typed via build_fcinfo!'s per-arg tag.

fn compare_i16_i32(name: &str, c_fn: FmgrFn, rust_fn: FmgrFn, a: i16, b: i32) {
    let mut fcinfo_c = build_fcinfo!(args = [(i16, a), (i32, b)]);
    let mut fcinfo_r = build_fcinfo!(args = [(i16, a), (i32, b)]);
    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { rust_fn(&mut fcinfo_r) };
    assert_eq!(c_out, r_out,
        "{name}(i16 {a}, i32 {b}): C={c_out} (bool={}) Rust={r_out} (bool={})",
        decode_bool(c_out), decode_bool(r_out));
}

fn compare_i32_i16(name: &str, c_fn: FmgrFn, rust_fn: FmgrFn, a: i32, b: i16) {
    let mut fcinfo_c = build_fcinfo!(args = [(i32, a), (i16, b)]);
    let mut fcinfo_r = build_fcinfo!(args = [(i32, a), (i16, b)]);
    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { rust_fn(&mut fcinfo_r) };
    assert_eq!(c_out, r_out,
        "{name}(i32 {a}, i16 {b}): C={c_out} (bool={}) Rust={r_out} (bool={})",
        decode_bool(c_out), decode_bool(r_out));
}

fn compare_i16_i64(name: &str, c_fn: FmgrFn, rust_fn: FmgrFn, a: i16, b: i64) {
    let mut fcinfo_c = build_fcinfo!(args = [(i16, a), (i64, b)]);
    let mut fcinfo_r = build_fcinfo!(args = [(i16, a), (i64, b)]);
    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { rust_fn(&mut fcinfo_r) };
    assert_eq!(c_out, r_out,
        "{name}(i16 {a}, i64 {b}): C={c_out} (bool={}) Rust={r_out} (bool={})",
        decode_bool(c_out), decode_bool(r_out));
}

fn compare_i64_i16(name: &str, c_fn: FmgrFn, rust_fn: FmgrFn, a: i64, b: i16) {
    let mut fcinfo_c = build_fcinfo!(args = [(i64, a), (i16, b)]);
    let mut fcinfo_r = build_fcinfo!(args = [(i64, a), (i16, b)]);
    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { rust_fn(&mut fcinfo_r) };
    assert_eq!(c_out, r_out,
        "{name}(i64 {a}, i16 {b}): C={c_out} (bool={}) Rust={r_out} (bool={})",
        decode_bool(c_out), decode_bool(r_out));
}

fn compare_i32_i64(name: &str, c_fn: FmgrFn, rust_fn: FmgrFn, a: i32, b: i64) {
    let mut fcinfo_c = build_fcinfo!(args = [(i32, a), (i64, b)]);
    let mut fcinfo_r = build_fcinfo!(args = [(i32, a), (i64, b)]);
    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { rust_fn(&mut fcinfo_r) };
    assert_eq!(c_out, r_out,
        "{name}(i32 {a}, i64 {b}): C={c_out} (bool={}) Rust={r_out} (bool={})",
        decode_bool(c_out), decode_bool(r_out));
}

fn compare_i64_i32(name: &str, c_fn: FmgrFn, rust_fn: FmgrFn, a: i64, b: i32) {
    let mut fcinfo_c = build_fcinfo!(args = [(i64, a), (i32, b)]);
    let mut fcinfo_r = build_fcinfo!(args = [(i64, a), (i32, b)]);
    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { rust_fn(&mut fcinfo_r) };
    assert_eq!(c_out, r_out,
        "{name}(i64 {a}, i32 {b}): C={c_out} (bool={}) Rust={r_out} (bool={})",
        decode_bool(c_out), decode_bool(r_out));
}

// ─── Op tables per width ─────────────────────────────────────────────────────

fn int2_ops() -> Vec<(&'static str, FmgrFn, FmgrFn)> {
    vec![
        ("int2eq", c_int2eq, pg_int4_cmp::int2eq),
        ("int2ne", c_int2ne, pg_int4_cmp::int2ne),
        ("int2lt", c_int2lt, pg_int4_cmp::int2lt),
        ("int2le", c_int2le, pg_int4_cmp::int2le),
        ("int2gt", c_int2gt, pg_int4_cmp::int2gt),
        ("int2ge", c_int2ge, pg_int4_cmp::int2ge),
    ]
}

fn int4_ops() -> Vec<(&'static str, FmgrFn, FmgrFn)> {
    vec![
        ("int4eq", c_int4eq, pg_int4_cmp::int4eq),
        ("int4ne", c_int4ne, pg_int4_cmp::int4ne),
        ("int4lt", c_int4lt, pg_int4_cmp::int4lt),
        ("int4le", c_int4le, pg_int4_cmp::int4le),
        ("int4gt", c_int4gt, pg_int4_cmp::int4gt),
        ("int4ge", c_int4ge, pg_int4_cmp::int4ge),
    ]
}

fn int8_ops() -> Vec<(&'static str, FmgrFn, FmgrFn)> {
    vec![
        ("int8eq", c_int8eq, pg_int4_cmp::int8eq),
        ("int8ne", c_int8ne, pg_int4_cmp::int8ne),
        ("int8lt", c_int8lt, pg_int4_cmp::int8lt),
        ("int8le", c_int8le, pg_int4_cmp::int8le),
        ("int8gt", c_int8gt, pg_int4_cmp::int8gt),
        ("int8ge", c_int8ge, pg_int4_cmp::int8ge),
    ]
}

fn date_ops() -> Vec<(&'static str, FmgrFn, FmgrFn)> {
    vec![
        ("date_eq", c_date_eq, pg_int4_cmp::date_eq),
        ("date_ne", c_date_ne, pg_int4_cmp::date_ne),
        ("date_lt", c_date_lt, pg_int4_cmp::date_lt),
        ("date_le", c_date_le, pg_int4_cmp::date_le),
        ("date_gt", c_date_gt, pg_int4_cmp::date_gt),
        ("date_ge", c_date_ge, pg_int4_cmp::date_ge),
    ]
}

fn timestamp_ops() -> Vec<(&'static str, FmgrFn, FmgrFn)> {
    vec![
        ("timestamp_eq", c_timestamp_eq, pg_int4_cmp::timestamp_eq),
        ("timestamp_ne", c_timestamp_ne, pg_int4_cmp::timestamp_ne),
        ("timestamp_lt", c_timestamp_lt, pg_int4_cmp::timestamp_lt),
        ("timestamp_le", c_timestamp_le, pg_int4_cmp::timestamp_le),
        ("timestamp_gt", c_timestamp_gt, pg_int4_cmp::timestamp_gt),
        ("timestamp_ge", c_timestamp_ge, pg_int4_cmp::timestamp_ge),
    ]
}

fn cash_ops() -> Vec<(&'static str, FmgrFn, FmgrFn)> {
    vec![
        ("cash_eq", c_cash_eq, pg_int4_cmp::cash_eq),
        ("cash_ne", c_cash_ne, pg_int4_cmp::cash_ne),
        ("cash_lt", c_cash_lt, pg_int4_cmp::cash_lt),
        ("cash_le", c_cash_le, pg_int4_cmp::cash_le),
        ("cash_gt", c_cash_gt, pg_int4_cmp::cash_gt),
        ("cash_ge", c_cash_ge, pg_int4_cmp::cash_ge),
    ]
}

fn pg_lsn_ops() -> Vec<(&'static str, FmgrFn, FmgrFn)> {
    vec![
        ("pg_lsn_eq", c_pg_lsn_eq, pg_int4_cmp::pg_lsn_eq),
        ("pg_lsn_ne", c_pg_lsn_ne, pg_int4_cmp::pg_lsn_ne),
        ("pg_lsn_lt", c_pg_lsn_lt, pg_int4_cmp::pg_lsn_lt),
        ("pg_lsn_le", c_pg_lsn_le, pg_int4_cmp::pg_lsn_le),
        ("pg_lsn_gt", c_pg_lsn_gt, pg_int4_cmp::pg_lsn_gt),
        ("pg_lsn_ge", c_pg_lsn_ge, pg_int4_cmp::pg_lsn_ge),
    ]
}

fn oid_ops() -> Vec<(&'static str, FmgrFn, FmgrFn)> {
    vec![
        ("oideq", c_oideq, pg_int4_cmp::oideq),
        ("oidne", c_oidne, pg_int4_cmp::oidne),
        ("oidlt", c_oidlt, pg_int4_cmp::oidlt),
        ("oidle", c_oidle, pg_int4_cmp::oidle),
        ("oidgt", c_oidgt, pg_int4_cmp::oidgt),
        ("oidge", c_oidge, pg_int4_cmp::oidge),
    ]
}

fn bool_ops() -> Vec<(&'static str, FmgrFn, FmgrFn)> {
    vec![
        ("booleq", c_booleq, pg_int4_cmp::booleq),
        ("boolne", c_boolne, pg_int4_cmp::boolne),
        ("boollt", c_boollt, pg_int4_cmp::boollt),
        ("boolle", c_boolle, pg_int4_cmp::boolle),
        ("boolgt", c_boolgt, pg_int4_cmp::boolgt),
        ("boolge", c_boolge, pg_int4_cmp::boolge),
    ]
}

fn float4_ops() -> Vec<(&'static str, FmgrFn, FmgrFn)> {
    vec![
        ("float4eq", c_float4eq, pg_int4_cmp::float4eq),
        ("float4ne", c_float4ne, pg_int4_cmp::float4ne),
        ("float4lt", c_float4lt, pg_int4_cmp::float4lt),
        ("float4le", c_float4le, pg_int4_cmp::float4le),
        ("float4gt", c_float4gt, pg_int4_cmp::float4gt),
        ("float4ge", c_float4ge, pg_int4_cmp::float4ge),
    ]
}

fn float8_ops() -> Vec<(&'static str, FmgrFn, FmgrFn)> {
    vec![
        ("float8eq", c_float8eq, pg_int4_cmp::float8eq),
        ("float8ne", c_float8ne, pg_int4_cmp::float8ne),
        ("float8lt", c_float8lt, pg_int4_cmp::float8lt),
        ("float8le", c_float8le, pg_int4_cmp::float8le),
        ("float8gt", c_float8gt, pg_int4_cmp::float8gt),
        ("float8ge", c_float8ge, pg_int4_cmp::float8ge),
    ]
}

fn char_ops() -> Vec<(&'static str, FmgrFn, FmgrFn)> {
    vec![
        ("chareq", c_chareq, pg_int4_cmp::chareq),
        ("charne", c_charne, pg_int4_cmp::charne),
        ("charlt", c_charlt, pg_int4_cmp::charlt),
        ("charle", c_charle, pg_int4_cmp::charle),
        ("chargt", c_chargt, pg_int4_cmp::chargt),
        ("charge", c_charge, pg_int4_cmp::charge),
    ]
}

fn int24_ops() -> Vec<(&'static str, FmgrFn, FmgrFn)> {
    vec![
        ("int24eq", c_int24eq, pg_int4_cmp::int24eq),
        ("int24ne", c_int24ne, pg_int4_cmp::int24ne),
        ("int24lt", c_int24lt, pg_int4_cmp::int24lt),
        ("int24le", c_int24le, pg_int4_cmp::int24le),
        ("int24gt", c_int24gt, pg_int4_cmp::int24gt),
        ("int24ge", c_int24ge, pg_int4_cmp::int24ge),
    ]
}

fn int42_ops() -> Vec<(&'static str, FmgrFn, FmgrFn)> {
    vec![
        ("int42eq", c_int42eq, pg_int4_cmp::int42eq),
        ("int42ne", c_int42ne, pg_int4_cmp::int42ne),
        ("int42lt", c_int42lt, pg_int4_cmp::int42lt),
        ("int42le", c_int42le, pg_int4_cmp::int42le),
        ("int42gt", c_int42gt, pg_int4_cmp::int42gt),
        ("int42ge", c_int42ge, pg_int4_cmp::int42ge),
    ]
}

fn int28_ops() -> Vec<(&'static str, FmgrFn, FmgrFn)> {
    vec![
        ("int28eq", c_int28eq, pg_int4_cmp::int28eq),
        ("int28ne", c_int28ne, pg_int4_cmp::int28ne),
        ("int28lt", c_int28lt, pg_int4_cmp::int28lt),
        ("int28le", c_int28le, pg_int4_cmp::int28le),
        ("int28gt", c_int28gt, pg_int4_cmp::int28gt),
        ("int28ge", c_int28ge, pg_int4_cmp::int28ge),
    ]
}

fn int82_ops() -> Vec<(&'static str, FmgrFn, FmgrFn)> {
    vec![
        ("int82eq", c_int82eq, pg_int4_cmp::int82eq),
        ("int82ne", c_int82ne, pg_int4_cmp::int82ne),
        ("int82lt", c_int82lt, pg_int4_cmp::int82lt),
        ("int82le", c_int82le, pg_int4_cmp::int82le),
        ("int82gt", c_int82gt, pg_int4_cmp::int82gt),
        ("int82ge", c_int82ge, pg_int4_cmp::int82ge),
    ]
}

fn int48_ops() -> Vec<(&'static str, FmgrFn, FmgrFn)> {
    vec![
        ("int48eq", c_int48eq, pg_int4_cmp::int48eq),
        ("int48ne", c_int48ne, pg_int4_cmp::int48ne),
        ("int48lt", c_int48lt, pg_int4_cmp::int48lt),
        ("int48le", c_int48le, pg_int4_cmp::int48le),
        ("int48gt", c_int48gt, pg_int4_cmp::int48gt),
        ("int48ge", c_int48ge, pg_int4_cmp::int48ge),
    ]
}

fn int84_ops() -> Vec<(&'static str, FmgrFn, FmgrFn)> {
    vec![
        ("int84eq", c_int84eq, pg_int4_cmp::int84eq),
        ("int84ne", c_int84ne, pg_int4_cmp::int84ne),
        ("int84lt", c_int84lt, pg_int4_cmp::int84lt),
        ("int84le", c_int84le, pg_int4_cmp::int84le),
        ("int84gt", c_int84gt, pg_int4_cmp::int84gt),
        ("int84ge", c_int84ge, pg_int4_cmp::int84ge),
    ]
}

// ─── Boundary cases ──────────────────────────────────────────────────────────

#[test]
fn boundary_cases_int2() {
    let pairs: [(i16, i16); 7] = [
        (0, 0), (1, 1), (-1, 1),
        (i16::MIN, i16::MIN), (i16::MAX, i16::MAX),
        (i16::MIN, i16::MAX), (i16::MAX, i16::MIN),
    ];
    for &(a, b) in &pairs {
        for (name, c_fn, r_fn) in int2_ops() {
            compare_i16(name, c_fn, r_fn, a, b);
        }
    }
}

#[test]
fn boundary_cases_int4() {
    let pairs: [(i32, i32); 9] = [
        (0, 0), (1, 1), (-1, 1),
        (i32::MIN, i32::MIN), (i32::MAX, i32::MAX),
        (i32::MIN, i32::MAX), (i32::MAX, i32::MIN),
        (i32::MIN, -1), (i32::MAX, 0),
    ];
    for &(a, b) in &pairs {
        for (name, c_fn, r_fn) in int4_ops() {
            compare_i32(name, c_fn, r_fn, a, b);
        }
    }
}

#[test]
fn boundary_cases_int8() {
    let pairs: [(i64, i64); 9] = [
        (0, 0), (1, 1), (-1, 1),
        (i64::MIN, i64::MIN), (i64::MAX, i64::MAX),
        (i64::MIN, i64::MAX), (i64::MAX, i64::MIN),
        (i64::MIN, -1), (i64::MAX, 0),
    ];
    for &(a, b) in &pairs {
        for (name, c_fn, r_fn) in int8_ops() {
            compare_i64(name, c_fn, r_fn, a, b);
        }
    }
}

#[test]
fn boundary_cases_char() {
    // char eq/ne are signed (-1 vs 1 → ne true). lt/le/gt/ge cast both
    // operands to uint8 first → -1 (0xFF = 255) > 1 (0x01) in lt-order.
    let pairs: [(i8, i8); 9] = [
        (0, 0), (1, 1), (-1, 1),
        (i8::MIN, i8::MIN), (i8::MAX, i8::MAX),
        (i8::MIN, i8::MAX), (i8::MAX, i8::MIN),
        (-1, 0), (-128, 127),
    ];
    for &(a, b) in &pairs {
        for (name, c_fn, r_fn) in char_ops() {
            compare_i8(name, c_fn, r_fn, a, b);
        }
    }
}

#[test]
fn boundary_cases_cross_type() {
    // Same numeric value at different widths — exercise sign-extension
    // and widening behavior in the C-side promotion. Pick values that
    // straddle int2 boundaries since those are the most likely points
    // of divergence under bad widening.
    let i16_vals: [i16; 5] = [0, 1, -1, i16::MIN, i16::MAX];
    let i32_vals: [i32; 7] = [0, 1, -1, i16::MIN as i32, i16::MAX as i32, i32::MIN, i32::MAX];
    let i64_vals: [i64; 9] = [
        0, 1, -1, i16::MIN as i64, i16::MAX as i64,
        i32::MIN as i64, i32::MAX as i64, i64::MIN, i64::MAX,
    ];
    for &a in &i16_vals {
        for &b in &i32_vals {
            for (name, c_fn, r_fn) in int24_ops() { compare_i16_i32(name, c_fn, r_fn, a, b); }
        }
        for &b in &i64_vals {
            for (name, c_fn, r_fn) in int28_ops() { compare_i16_i64(name, c_fn, r_fn, a, b); }
        }
    }
    for &a in &i32_vals {
        for &b in &i16_vals {
            for (name, c_fn, r_fn) in int42_ops() { compare_i32_i16(name, c_fn, r_fn, a, b); }
        }
        for &b in &i64_vals {
            for (name, c_fn, r_fn) in int48_ops() { compare_i32_i64(name, c_fn, r_fn, a, b); }
        }
    }
    for &a in &i64_vals {
        for &b in &i16_vals {
            for (name, c_fn, r_fn) in int82_ops() { compare_i64_i16(name, c_fn, r_fn, a, b); }
        }
        for &b in &i32_vals {
            for (name, c_fn, r_fn) in int84_ops() { compare_i64_i32(name, c_fn, r_fn, a, b); }
        }
    }
}

// ─── Proptest sweeps ─────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2048))]

    #[test]
    fn diff_int2_cmp_proptest(a in any::<i16>(), b in any::<i16>()) {
        for (name, c_fn, r_fn) in int2_ops() {
            compare_i16(name, c_fn, r_fn, a, b);
        }
    }

    #[test]
    fn diff_int4_cmp_proptest(a in any::<i32>(), b in any::<i32>()) {
        for (name, c_fn, r_fn) in int4_ops() {
            compare_i32(name, c_fn, r_fn, a, b);
        }
    }

    #[test]
    fn diff_int8_cmp_proptest(a in any::<i64>(), b in any::<i64>()) {
        for (name, c_fn, r_fn) in int8_ops() {
            compare_i64(name, c_fn, r_fn, a, b);
        }
    }

    #[test]
    fn diff_date_cmp_proptest(a in any::<i32>(), b in any::<i32>()) {
        for (name, c_fn, r_fn) in date_ops() {
            compare_i32(name, c_fn, r_fn, a, b);
        }
    }

    #[test]
    fn diff_timestamp_cmp_proptest(a in any::<i64>(), b in any::<i64>()) {
        for (name, c_fn, r_fn) in timestamp_ops() {
            compare_i64(name, c_fn, r_fn, a, b);
        }
    }

    #[test]
    fn diff_cash_cmp_proptest(a in any::<i64>(), b in any::<i64>()) {
        for (name, c_fn, r_fn) in cash_ops() {
            compare_i64(name, c_fn, r_fn, a, b);
        }
    }

    #[test]
    fn diff_pg_lsn_cmp_proptest(a in any::<u64>(), b in any::<u64>()) {
        for (name, c_fn, r_fn) in pg_lsn_ops() {
            compare_u64(name, c_fn, r_fn, a, b);
        }
    }

    #[test]
    fn diff_oid_cmp_proptest(a in any::<u32>(), b in any::<u32>()) {
        for (name, c_fn, r_fn) in oid_ops() {
            compare_u32(name, c_fn, r_fn, a, b);
        }
    }

    #[test]
    fn diff_bool_cmp_proptest(a in any::<bool>(), b in any::<bool>()) {
        for (name, c_fn, r_fn) in bool_ops() {
            compare_bool(name, c_fn, r_fn, a, b);
        }
    }

    // For floats, use prop_oneof to include NaN, infinities, and
    // signed zero so we exercise the NaN-aware comparison helpers
    // along with the IEEE-754 corners.
    #[test]
    fn diff_float4_cmp_proptest(a in prop_oneof![Just(f32::NAN), Just(f32::INFINITY), Just(f32::NEG_INFINITY), Just(0.0_f32), Just(-0.0_f32), any::<f32>()],
                                 b in prop_oneof![Just(f32::NAN), Just(f32::INFINITY), Just(f32::NEG_INFINITY), Just(0.0_f32), Just(-0.0_f32), any::<f32>()]) {
        for (name, c_fn, r_fn) in float4_ops() {
            compare_f32(name, c_fn, r_fn, a, b);
        }
    }

    #[test]
    fn diff_float8_cmp_proptest(a in prop_oneof![Just(f64::NAN), Just(f64::INFINITY), Just(f64::NEG_INFINITY), Just(0.0_f64), Just(-0.0_f64), any::<f64>()],
                                 b in prop_oneof![Just(f64::NAN), Just(f64::INFINITY), Just(f64::NEG_INFINITY), Just(0.0_f64), Just(-0.0_f64), any::<f64>()]) {
        for (name, c_fn, r_fn) in float8_ops() {
            compare_f64(name, c_fn, r_fn, a, b);
        }
    }

    #[test]
    fn diff_char_cmp_proptest(a in any::<i8>(), b in any::<i8>()) {
        for (name, c_fn, r_fn) in char_ops() {
            compare_i8(name, c_fn, r_fn, a, b);
        }
    }

    #[test]
    fn diff_int24_cmp_proptest(a in any::<i16>(), b in any::<i32>()) {
        for (name, c_fn, r_fn) in int24_ops() {
            compare_i16_i32(name, c_fn, r_fn, a, b);
        }
    }

    #[test]
    fn diff_int42_cmp_proptest(a in any::<i32>(), b in any::<i16>()) {
        for (name, c_fn, r_fn) in int42_ops() {
            compare_i32_i16(name, c_fn, r_fn, a, b);
        }
    }

    #[test]
    fn diff_int28_cmp_proptest(a in any::<i16>(), b in any::<i64>()) {
        for (name, c_fn, r_fn) in int28_ops() {
            compare_i16_i64(name, c_fn, r_fn, a, b);
        }
    }

    #[test]
    fn diff_int82_cmp_proptest(a in any::<i64>(), b in any::<i16>()) {
        for (name, c_fn, r_fn) in int82_ops() {
            compare_i64_i16(name, c_fn, r_fn, a, b);
        }
    }

    #[test]
    fn diff_int48_cmp_proptest(a in any::<i32>(), b in any::<i64>()) {
        for (name, c_fn, r_fn) in int48_ops() {
            compare_i32_i64(name, c_fn, r_fn, a, b);
        }
    }

    #[test]
    fn diff_int84_cmp_proptest(a in any::<i64>(), b in any::<i32>()) {
        for (name, c_fn, r_fn) in int84_ops() {
            compare_i64_i32(name, c_fn, r_fn, a, b);
        }
    }
}
