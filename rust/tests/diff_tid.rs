//! Differential test for tid comparison, clamping and hash cluster.
//!
//! Tests all 10 functions: tideq/tidne/tidlt/tidle/tidgt/tidge/bttidcmp (7 comparison ops),
//! tidlarger/tidsmaller (2 clamping ops), and hashtid/hashtidextended (2 hash ops).
//! Verifies the Rust translation matches the C oracle on randomized 6-byte tid inputs.

use pg_fcinfo::{build_fcinfo, decode_u32, decode_u64, encode_ptr, FunctionCallInfoBaseData};
use proptest::prelude::*;

extern "C" {
    fn c_tideq(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_tidne(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_tidlt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_tidle(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_tidgt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_tidge(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_bttidcmp(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_tidlarger(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_tidsmaller(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_hashtid(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_hashtidextended(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
}

// Rust entry points (emitted by Lean)
use pg_tid::{
    tideq, tidne, tidlt, tidle, tidgt, tidge, bttidcmp,
    tidlarger, tidsmaller,
    hashtid, hashtidextended,
};

type FmgrFn = unsafe extern "C" fn(*mut FunctionCallInfoBaseData) -> u64;

fn assert_cmp_eq(name: &str, c_out: u64, r_out: u64, repr: &str) {
    assert_eq!(
        c_out, r_out,
        "{name}({repr}): C={c_out} Rust={r_out}"
    );
}

fn assert_hash_eq(name: &str, c_out: u64, r_out: u64, repr: &str, is_extended: bool) {
    if is_extended {
        // Extended variant returns full u64
        assert_eq!(
            c_out, r_out,
            "{name}({repr}): C={c_out:x} Rust={r_out:x}"
        );
    } else {
        // Standard variant returns u32 in lower bits
        let c_val = decode_u32(c_out);
        let r_val = decode_u32(r_out);
        assert_eq!(
            c_val, r_val,
            "{name}({repr}): C={c_val:x} Rust={r_val:x}"
        );
    }
}

fn compare_cmp(name: &str, c_fn: FmgrFn, r_fn: FmgrFn, t1: &[u8; 6], t2: &[u8; 6]) {
    let ptr1 = t1.as_ptr();
    let ptr2 = t2.as_ptr();

    let mut fcinfo_c = build_fcinfo!(args = [(ptr, ptr1), (ptr, ptr2)]);
    let mut fcinfo_r = build_fcinfo!(args = [(ptr, ptr1), (ptr, ptr2)]);

    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { r_fn(&mut fcinfo_r) };

    let repr = format!("tid[0]={:02x} vs tid[0]={:02x}", t1[0], t2[0]);
    assert_cmp_eq(name, c_out, r_out, &repr);
}

fn compare_clamp(name: &str, c_fn: FmgrFn, r_fn: FmgrFn, t1: &[u8; 6], t2: &[u8; 6]) {
    let ptr1 = t1.as_ptr();
    let ptr2 = t2.as_ptr();

    let mut fcinfo_c = build_fcinfo!(args = [(ptr, ptr1), (ptr, ptr2)]);
    let mut fcinfo_r = build_fcinfo!(args = [(ptr, ptr1), (ptr, ptr2)]);

    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { r_fn(&mut fcinfo_r) };

    let repr = format!("tid[0]={:02x} vs tid[0]={:02x}", t1[0], t2[0]);
    assert_cmp_eq(name, c_out, r_out, &repr);
}

fn compare_hash(name: &str, c_fn: FmgrFn, r_fn: FmgrFn, tid: &[u8; 6]) {
    let ptr = tid.as_ptr();

    let mut fcinfo_c = build_fcinfo!(args = [(ptr, ptr)]);
    let mut fcinfo_r = build_fcinfo!(args = [(ptr, ptr)]);

    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { r_fn(&mut fcinfo_r) };

    let repr = format!("tid[0]={:02x}", tid[0]);
    assert_hash_eq(name, c_out, r_out, &repr, false);
}

fn compare_hash_extended(name: &str, c_fn: FmgrFn, r_fn: FmgrFn, tid: &[u8; 6], seed: i64) {
    let ptr = tid.as_ptr();

    let mut fcinfo_c = build_fcinfo!(args = [(ptr, ptr), (i64, seed)]);
    let mut fcinfo_r = build_fcinfo!(args = [(ptr, ptr), (i64, seed)]);

    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { r_fn(&mut fcinfo_r) };

    let repr = format!("tid[0]={:02x}, seed={seed}", tid[0]);
    assert_hash_eq(name, c_out, r_out, &repr, true);
}

#[test]
fn test_tid_cmp_basic() {
    let tid1 = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05];
    let tid2 = [0x00, 0x01, 0x02, 0x03, 0x04, 0x06];

    compare_cmp("tideq", c_tideq, tideq, &tid1, &tid1);
    compare_cmp("tidne", c_tidne, tidne, &tid1, &tid2);
    compare_cmp("tidlt", c_tidlt, tidlt, &tid1, &tid2);
    compare_cmp("tidle", c_tidle, tidle, &tid1, &tid1);
    compare_cmp("tidgt", c_tidgt, tidgt, &tid2, &tid1);
    compare_cmp("tidge", c_tidge, tidge, &tid1, &tid1);
    compare_cmp("bttidcmp", c_bttidcmp, bttidcmp, &tid1, &tid2);
}

#[test]
fn test_tid_clamp_basic() {
    let tid1 = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05];
    let tid2 = [0x00, 0x01, 0x02, 0x03, 0x04, 0x06];

    compare_clamp("tidlarger", c_tidlarger, tidlarger, &tid1, &tid2);
    compare_clamp("tidsmaller", c_tidsmaller, tidsmaller, &tid1, &tid2);
}

#[test]
fn test_tid_hash_basic() {
    let tid = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05];

    compare_hash("hashtid", c_hashtid, hashtid, &tid);
    compare_hash_extended("hashtidextended", c_hashtidextended, hashtidextended, &tid, 42);
}

proptest! {
    #[test]
    fn prop_tid_cmp(tid1 in prop::array::uniform6(0u8..), tid2 in prop::array::uniform6(0u8..)) {
        compare_cmp("tideq", c_tideq, tideq, &tid1, &tid2);
        compare_cmp("tidne", c_tidne, tidne, &tid1, &tid2);
        compare_cmp("tidlt", c_tidlt, tidlt, &tid1, &tid2);
        compare_cmp("tidle", c_tidle, tidle, &tid1, &tid2);
        compare_cmp("tidgt", c_tidgt, tidgt, &tid1, &tid2);
        compare_cmp("tidge", c_tidge, tidge, &tid1, &tid2);
        compare_cmp("bttidcmp", c_bttidcmp, bttidcmp, &tid1, &tid2);
    }

    #[test]
    fn prop_tid_clamp(tid1 in prop::array::uniform6(0u8..), tid2 in prop::array::uniform6(0u8..)) {
        compare_clamp("tidlarger", c_tidlarger, tidlarger, &tid1, &tid2);
        compare_clamp("tidsmaller", c_tidsmaller, tidsmaller, &tid1, &tid2);
    }

    #[test]
    fn prop_tid_hash(tid in prop::array::uniform6(0u8..)) {
        compare_hash("hashtid", c_hashtid, hashtid, &tid);
    }

    #[test]
    fn prop_tid_hash_extended(tid in prop::array::uniform6(0u8..), seed in any::<i64>()) {
        compare_hash_extended("hashtidextended", c_hashtidextended, hashtidextended, &tid, seed);
    }
}
