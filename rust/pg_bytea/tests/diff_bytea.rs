//! Smoke test for the bytea cluster (v0: byteacat only).
//!
//! Validates the round-trip across both sides:
//!   - Allocate two bytea inputs (4-byte header + payload) in a
//!     shared MemoryContext.
//!   - Construct fcinfo whose args[0..2].value are the input pointers.
//!   - Invoke each side (Rust translation, vendored C oracle).
//!   - Both sides palloc their own result varlena in the same arena.
//!   - Compare the result-varlena CONTENTS: total varsize equal, and
//!     payload bytes identical via memcmp.
//!   - On error: nothing — byteacat doesn't ereport. (palloc OOM would
//!     abort the process; not exercised here.)

use pg_fcinfo::{
    build_fcinfo, decode_bool, decode_i32, set_varsize_4b, vardata_4b_mut, vardata_any,
    varsize_4b, varsize_any_exhdr, FunctionCallInfoBaseData, VARHDRSZ,
};
use pg_palloc::{palloc, with_memory_context, MemoryContext};
use proptest::prelude::*;
use std::ffi::c_void;

extern "C" {
    fn c_byteacat(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_byteaoctetlen(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_byteaeq(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
}

type FmgrFn = unsafe extern "C" fn(*mut FunctionCallInfoBaseData) -> u64;

/// Allocate a varlena holding `payload` in the current MemoryContext.
/// Returns the varlena pointer (i.e., pointer at the 4-byte header).
unsafe fn alloc_bytea(payload: &[u8]) -> *mut u8 {
    let total = VARHDRSZ + payload.len();
    let p = palloc(total);
    set_varsize_4b(p, total);
    if !payload.is_empty() {
        let dst = vardata_4b_mut(p);
        std::ptr::copy_nonoverlapping(payload.as_ptr(), dst, payload.len());
    }
    p
}

/// Invoke one fmgr stub with two bytea inputs. Returns the output Datum
/// (which is a varlena pointer cast to u64).
unsafe fn invoke(fn_: FmgrFn, t1: *const u8, t2: *const u8) -> u64 {
    let mut fcinfo = build_fcinfo!(args = [
        (mut_ptr, t1 as *mut c_void),
        (mut_ptr, t2 as *mut c_void),
    ]);
    fn_(&mut fcinfo)
}

/// Compare two varlena pointers' contents — same total size, same
/// payload bytes.
unsafe fn assert_varlena_eq(label: &str, a: *const u8, b: *const u8) {
    let sa = varsize_4b(a);
    let sb = varsize_4b(b);
    assert_eq!(sa, sb, "{label}: varsize differs (C={sa} R={sb})");
    let exhdr = varsize_any_exhdr(a);
    if exhdr > 0 {
        let pa = vardata_any(a);
        let pb = vardata_any(b);
        let bytes_a = std::slice::from_raw_parts(pa, exhdr);
        let bytes_b = std::slice::from_raw_parts(pb, exhdr);
        assert_eq!(bytes_a, bytes_b, "{label}: payload differs");
    }
}

unsafe fn compare_byteacat(payload_a: &[u8], payload_b: &[u8]) {
    let t1 = alloc_bytea(payload_a);
    let t2 = alloc_bytea(payload_b);
    let c_out = invoke(c_byteacat, t1, t2) as *const u8;
    let r_out = invoke(pg_bytea::byteacat, t1, t2) as *const u8;
    assert_varlena_eq(
        &format!("byteacat([{}],[{}])", payload_a.len(), payload_b.len()),
        c_out, r_out,
    );
    // And both should equal the expected concatenation.
    let expected_size = VARHDRSZ + payload_a.len() + payload_b.len();
    assert_eq!(varsize_4b(c_out), expected_size);
    let exhdr = varsize_any_exhdr(c_out);
    let combined = std::slice::from_raw_parts(vardata_any(c_out), exhdr);
    let mut expected = Vec::with_capacity(payload_a.len() + payload_b.len());
    expected.extend_from_slice(payload_a);
    expected.extend_from_slice(payload_b);
    assert_eq!(combined, expected.as_slice(), "concatenation mismatch");
}

#[test]
fn boundary_byteacat() {
    with_memory_context(|_ctx: *mut MemoryContext| unsafe {
        compare_byteacat(b"", b"");
        compare_byteacat(b"hello", b"");
        compare_byteacat(b"", b"world");
        compare_byteacat(b"hello", b" world");
        compare_byteacat(b"\x00\x01\x02", b"\xff\xfe\xfd"); // bytea = binary, must handle NULs
        compare_byteacat(b"a", b"b");
        let big_a = vec![0xAA_u8; 1024];
        let big_b = vec![0x55_u8; 2048];
        compare_byteacat(&big_a, &big_b);
    });
}

unsafe fn invoke1(fn_: FmgrFn, t: *const u8) -> u64 {
    let mut fcinfo = build_fcinfo!(args = [(mut_ptr, t as *mut c_void)]);
    fn_(&mut fcinfo)
}

unsafe fn compare_byteaoctetlen(payload: &[u8]) {
    let t = alloc_bytea(payload);
    let c = decode_i32(invoke1(c_byteaoctetlen, t));
    let r = decode_i32(invoke1(pg_bytea::byteaoctetlen, t));
    assert_eq!(c, r, "byteaoctetlen(len={}): C={c} R={r}", payload.len());
    assert_eq!(c as usize, payload.len(), "expected payload.len()");
}

unsafe fn compare_byteaeq(a: &[u8], b: &[u8]) {
    let t1 = alloc_bytea(a);
    let t2 = alloc_bytea(b);
    let c_out = invoke(c_byteaeq, t1, t2);
    let r_out = invoke(pg_bytea::byteaeq, t1, t2);
    assert_eq!(c_out, r_out, "byteaeq({},{}): C={c_out} R={r_out}", a.len(), b.len());
    let expected = a == b;
    assert_eq!(decode_bool(c_out), expected, "byteaeq expected {expected}");
}

#[test]
fn boundary_byteaoctetlen() {
    with_memory_context(|_| unsafe {
        for p in &[
            b"".as_ref(),
            b"x",
            b"hello",
            &[0u8; 100][..],
            &[0xFFu8; 4096][..],
        ] {
            compare_byteaoctetlen(p);
        }
    });
}

#[test]
fn boundary_byteaeq() {
    with_memory_context(|_| unsafe {
        // Equal cases
        compare_byteaeq(b"", b"");
        compare_byteaeq(b"hello", b"hello");
        compare_byteaeq(b"\x00\x01\x02", b"\x00\x01\x02");
        // Unequal-length fast path
        compare_byteaeq(b"", b"x");
        compare_byteaeq(b"hello", b"hellox");
        // Same-length-but-different
        compare_byteaeq(b"hello", b"world");
        compare_byteaeq(b"\x00\x00", b"\x00\x01");
        // Bigger pair
        let big_a = vec![0xAA_u8; 4096];
        let mut big_b = big_a.clone();
        big_b[1234] = 0x55;
        compare_byteaeq(&big_a, &big_b);
        compare_byteaeq(&big_a, &big_a);
    });
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    #[test]
    fn diff_byteacat_proptest(
        a in prop::collection::vec(any::<u8>(), 0..256),
        b in prop::collection::vec(any::<u8>(), 0..256),
    ) {
        with_memory_context(|_| unsafe {
            compare_byteacat(&a, &b);
        });
    }

    #[test]
    fn diff_byteaoctetlen_proptest(p in prop::collection::vec(any::<u8>(), 0..512)) {
        with_memory_context(|_| unsafe {
            compare_byteaoctetlen(&p);
        });
    }

    // Equal pairs: a is generated freely; b is set equal to a half the time
    // to exercise the success path.
    #[test]
    fn diff_byteaeq_proptest(
        a in prop::collection::vec(any::<u8>(), 0..256),
        b in prop::collection::vec(any::<u8>(), 0..256),
        same in any::<bool>(),
    ) {
        with_memory_context(|_| unsafe {
            let b_use: Vec<u8> = if same { a.clone() } else { b };
            compare_byteaeq(&a, &b_use);
        });
    }
}
