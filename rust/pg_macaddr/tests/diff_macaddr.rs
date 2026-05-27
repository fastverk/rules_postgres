//! Differential test for macaddr comparison and hash cluster.
//!
//! Tests all 9 functions: macaddr_eq/ne/lt/le/gt/ge/cmp (7 comparison ops)
//! and hashmacaddr/hashmacaddrextended (2 hash ops). Verifies the Rust translation
//! matches the C oracle on randomized 6-byte macaddr inputs.

use pg_fcinfo::{build_fcinfo, decode_u32, decode_u64, encode_ptr, FunctionCallInfoBaseData};
use proptest::prelude::*;

extern "C" {
    fn c_macaddr_eq(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_macaddr_ne(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_macaddr_lt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_macaddr_le(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_macaddr_gt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_macaddr_ge(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_macaddr_cmp(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_hashmacaddr(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_hashmacaddrextended(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
}

// Rust entry points (emitted by Lean)
use pg_macaddr::{
    macaddr_eq, macaddr_ne, macaddr_lt, macaddr_le, macaddr_gt, macaddr_ge, macaddr_cmp,
    hashmacaddr, hashmacaddrextended,
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

fn compare_cmp(name: &str, c_fn: FmgrFn, r_fn: FmgrFn, m1: &[u8; 6], m2: &[u8; 6]) {
    let ptr1 = m1.as_ptr();
    let ptr2 = m2.as_ptr();

    let mut fcinfo_c = build_fcinfo!(args = [(ptr, ptr1), (ptr, ptr2)]);
    let mut fcinfo_r = build_fcinfo!(args = [(ptr, ptr1), (ptr, ptr2)]);

    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { r_fn(&mut fcinfo_r) };

    let repr = format!("macaddr[0]={:02x} vs macaddr[0]={:02x}", m1[0], m2[0]);
    assert_cmp_eq(name, c_out, r_out, &repr);
}

fn compare_hash(name: &str, c_fn: FmgrFn, r_fn: FmgrFn, macaddr: &[u8; 6]) {
    let ptr = macaddr.as_ptr();

    let mut fcinfo_c = build_fcinfo!(args = [(ptr, ptr)]);
    let mut fcinfo_r = build_fcinfo!(args = [(ptr, ptr)]);

    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { r_fn(&mut fcinfo_r) };

    let repr = format!("macaddr[0]={:02x}", macaddr[0]);
    assert_hash_eq(name, c_out, r_out, &repr, false);
}

fn compare_hash_extended(name: &str, c_fn: FmgrFn, r_fn: FmgrFn, macaddr: &[u8; 6], seed: i64) {
    let ptr = macaddr.as_ptr();

    let mut fcinfo_c = build_fcinfo!(args = [(ptr, ptr), (i64, seed)]);
    let mut fcinfo_r = build_fcinfo!(args = [(ptr, ptr), (i64, seed)]);

    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { r_fn(&mut fcinfo_r) };

    let repr = format!("macaddr[0]={:02x}, seed={seed}", macaddr[0]);
    assert_hash_eq(name, c_out, r_out, &repr, true);
}

#[test]
fn test_macaddr_cmp_basic() {
    let mac1 = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05];
    let mac2 = [0x00, 0x01, 0x02, 0x03, 0x04, 0x06];

    compare_cmp("macaddr_eq", c_macaddr_eq, macaddr_eq, &mac1, &mac1);
    compare_cmp("macaddr_ne", c_macaddr_ne, macaddr_ne, &mac1, &mac2);
    compare_cmp("macaddr_lt", c_macaddr_lt, macaddr_lt, &mac1, &mac2);
    compare_cmp("macaddr_le", c_macaddr_le, macaddr_le, &mac1, &mac1);
    compare_cmp("macaddr_gt", c_macaddr_gt, macaddr_gt, &mac2, &mac1);
    compare_cmp("macaddr_ge", c_macaddr_ge, macaddr_ge, &mac1, &mac1);
    compare_cmp("macaddr_cmp", c_macaddr_cmp, macaddr_cmp, &mac1, &mac2);
}

#[test]
fn test_macaddr_hash_basic() {
    let macaddr = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05];

    compare_hash("hashmacaddr", c_hashmacaddr, hashmacaddr, &macaddr);
    compare_hash_extended("hashmacaddrextended", c_hashmacaddrextended, hashmacaddrextended, &macaddr, 42);
}

proptest! {
    #[test]
    fn prop_macaddr_cmp(mac1 in prop::array::uniform6(0u8..), mac2 in prop::array::uniform6(0u8..)) {
        compare_cmp("macaddr_eq", c_macaddr_eq, macaddr_eq, &mac1, &mac2);
        compare_cmp("macaddr_ne", c_macaddr_ne, macaddr_ne, &mac1, &mac2);
        compare_cmp("macaddr_lt", c_macaddr_lt, macaddr_lt, &mac1, &mac2);
        compare_cmp("macaddr_le", c_macaddr_le, macaddr_le, &mac1, &mac2);
        compare_cmp("macaddr_gt", c_macaddr_gt, macaddr_gt, &mac1, &mac2);
        compare_cmp("macaddr_ge", c_macaddr_ge, macaddr_ge, &mac1, &mac2);
        compare_cmp("macaddr_cmp", c_macaddr_cmp, macaddr_cmp, &mac1, &mac2);
    }

    #[test]
    fn prop_macaddr_hash(macaddr in prop::array::uniform6(0u8..)) {
        compare_hash("hashmacaddr", c_hashmacaddr, hashmacaddr, &macaddr);
    }

    #[test]
    fn prop_macaddr_hash_extended(macaddr in prop::array::uniform6(0u8..), seed in any::<i64>()) {
        compare_hash_extended("hashmacaddrextended", c_hashmacaddrextended, hashmacaddrextended, &macaddr, seed);
    }
}
