//! Differential test for UUID comparison and hash cluster.
//!
//! Tests all 9 functions: uuid_eq/ne/lt/le/gt/ge/cmp (7 comparison ops)
//! and uuid_hash/hash_extended (2 hash ops). Verifies the Rust translation
//! matches the C oracle on randomized 16-byte UUID inputs.

use pg_fcinfo::{build_fcinfo, decode_u32, decode_u64, encode_ptr, FunctionCallInfoBaseData};
use proptest::prelude::*;

extern "C" {
    fn c_uuid_eq(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_uuid_ne(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_uuid_lt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_uuid_le(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_uuid_gt(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_uuid_ge(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_uuid_cmp(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_uuid_hash(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_uuid_hash_extended(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
}

// Rust entry points (emitted by Lean)
use pg_uuid::{
    uuid_eq, uuid_ne, uuid_lt, uuid_le, uuid_gt, uuid_ge, uuid_cmp,
    uuid_hash, uuid_hash_extended,
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

fn compare_cmp(name: &str, c_fn: FmgrFn, r_fn: FmgrFn, u1: &[u8; 16], u2: &[u8; 16]) {
    let ptr1 = u1.as_ptr();
    let ptr2 = u2.as_ptr();

    let mut fcinfo_c = build_fcinfo!(args = [(ptr, ptr1), (ptr, ptr2)]);
    let mut fcinfo_r = build_fcinfo!(args = [(ptr, ptr1), (ptr, ptr2)]);

    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { r_fn(&mut fcinfo_r) };

    let repr = format!("uuid[0]={:02x}.. vs uuid[0]={:02x}..", u1[0], u2[0]);
    assert_cmp_eq(name, c_out, r_out, &repr);
}

fn compare_hash(name: &str, c_fn: FmgrFn, r_fn: FmgrFn, uuid: &[u8; 16]) {
    let ptr = uuid.as_ptr();

    let mut fcinfo_c = build_fcinfo!(args = [(ptr, ptr)]);
    let mut fcinfo_r = build_fcinfo!(args = [(ptr, ptr)]);

    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { r_fn(&mut fcinfo_r) };

    let repr = format!("uuid[0]={:02x}..", uuid[0]);
    assert_hash_eq(name, c_out, r_out, &repr, false);
}

fn compare_hash_extended(name: &str, c_fn: FmgrFn, r_fn: FmgrFn, uuid: &[u8; 16], seed: i64) {
    let ptr = uuid.as_ptr();

    let mut fcinfo_c = build_fcinfo!(args = [(ptr, ptr), (i64, seed)]);
    let mut fcinfo_r = build_fcinfo!(args = [(ptr, ptr), (i64, seed)]);

    let c_out = unsafe { c_fn(&mut fcinfo_c) };
    let r_out = unsafe { r_fn(&mut fcinfo_r) };

    let repr = format!("uuid[0]={:02x}.., seed={seed}", uuid[0]);
    assert_hash_eq(name, c_out, r_out, &repr, true);
}

#[test]
fn test_uuid_cmp_basic() {
    let uuid1 = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
        0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
    ];
    let uuid2 = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
        0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x10,
    ];

    compare_cmp("uuid_eq", c_uuid_eq, uuid_eq, &uuid1, &uuid1);
    compare_cmp("uuid_ne", c_uuid_ne, uuid_ne, &uuid1, &uuid2);
    compare_cmp("uuid_lt", c_uuid_lt, uuid_lt, &uuid1, &uuid2);
    compare_cmp("uuid_le", c_uuid_le, uuid_le, &uuid1, &uuid1);
    compare_cmp("uuid_gt", c_uuid_gt, uuid_gt, &uuid2, &uuid1);
    compare_cmp("uuid_ge", c_uuid_ge, uuid_ge, &uuid1, &uuid1);
    compare_cmp("uuid_cmp", c_uuid_cmp, uuid_cmp, &uuid1, &uuid2);
}

#[test]
fn test_uuid_hash_basic() {
    let uuid = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
        0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
    ];

    compare_hash("uuid_hash", c_uuid_hash, uuid_hash, &uuid);
    compare_hash_extended("uuid_hash_extended", c_uuid_hash_extended, uuid_hash_extended, &uuid, 42);
}

proptest! {
    #[test]
    fn prop_uuid_cmp(uuid1 in prop::array::uniform16(0u8..), uuid2 in prop::array::uniform16(0u8..)) {
        compare_cmp("uuid_eq", c_uuid_eq, uuid_eq, &uuid1, &uuid2);
        compare_cmp("uuid_ne", c_uuid_ne, uuid_ne, &uuid1, &uuid2);
        compare_cmp("uuid_lt", c_uuid_lt, uuid_lt, &uuid1, &uuid2);
        compare_cmp("uuid_le", c_uuid_le, uuid_le, &uuid1, &uuid2);
        compare_cmp("uuid_gt", c_uuid_gt, uuid_gt, &uuid1, &uuid2);
        compare_cmp("uuid_ge", c_uuid_ge, uuid_ge, &uuid1, &uuid2);
        compare_cmp("uuid_cmp", c_uuid_cmp, uuid_cmp, &uuid1, &uuid2);
    }

    #[test]
    fn prop_uuid_hash(uuid in prop::array::uniform16(0u8..)) {
        compare_hash("uuid_hash", c_uuid_hash, uuid_hash, &uuid);
    }

    #[test]
    fn prop_uuid_hash_extended(uuid in prop::array::uniform16(0u8..), seed in any::<i64>()) {
        compare_hash_extended("uuid_hash_extended", c_uuid_hash_extended, uuid_hash_extended, &uuid, seed);
    }
}
