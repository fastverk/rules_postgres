//! Smoke test for the text cluster (v0: textcat only).
//!
//! Validates the round-trip across both sides:
//!   - Allocate two text inputs (4-byte header + payload) in a
//!     shared MemoryContext.
//!   - Construct fcinfo whose args[0..2].value are the input pointers.
//!   - Invoke each side (Rust translation, vendored C oracle).
//!   - Both sides palloc their own result varlena in the same arena.
//!   - Compare the result-varlena CONTENTS: total varsize equal, and
//!     payload bytes identical via memcmp.
//!   - On error: nothing — textcat doesn't ereport. (palloc OOM would
//!     abort the process; not exercised here.)

use pg_fcinfo::{
    build_fcinfo, set_varsize_4b, vardata_4b_mut, vardata_any,
    varsize_4b, varsize_any_exhdr, FunctionCallInfoBaseData, VARHDRSZ,
};
use pg_palloc::{palloc, with_memory_context, MemoryContext};
use proptest::prelude::*;
use std::ffi::c_void;

extern "C" {
    fn c_textcat(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
}

type FmgrFn = unsafe extern "C" fn(*mut FunctionCallInfoBaseData) -> u64;

/// Allocate a varlena holding `payload` in the current MemoryContext.
/// Returns the varlena pointer (i.e., pointer at the 4-byte header).
unsafe fn alloc_text(payload: &[u8]) -> *mut u8 {
    let total = VARHDRSZ + payload.len();
    let p = palloc(total);
    set_varsize_4b(p, total);
    if !payload.is_empty() {
        let dst = vardata_4b_mut(p);
        std::ptr::copy_nonoverlapping(payload.as_ptr(), dst, payload.len());
    }
    p
}

/// Invoke one fmgr stub with two text inputs. Returns the output Datum
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

unsafe fn compare_textcat(payload_a: &[u8], payload_b: &[u8]) {
    let t1 = alloc_text(payload_a);
    let t2 = alloc_text(payload_b);
    let c_out = invoke(c_textcat, t1, t2) as *const u8;
    let r_out = invoke(pg_text::textcat, t1, t2) as *const u8;
    assert_varlena_eq(
        &format!("textcat([{}],[{}])", payload_a.len(), payload_b.len()),
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
fn boundary_textcat() {
    with_memory_context(|_ctx: *mut MemoryContext| unsafe {
        compare_textcat(b"", b"");
        compare_textcat(b"hello", b"");
        compare_textcat(b"", b"world");
        compare_textcat(b"hello", b" world");
        compare_textcat(b"\x00\x01\x02", b"\xff\xfe\xfd"); // text = binary, must handle NULs
        compare_textcat(b"a", b"b");
        let big_a = vec![0xAA_u8; 1024];
        let big_b = vec![0x55_u8; 2048];
        compare_textcat(&big_a, &big_b);
    });
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    #[test]
    fn diff_textcat_proptest(
        a in prop::collection::vec(any::<u8>(), 0..256),
        b in prop::collection::vec(any::<u8>(), 0..256),
    ) {
        with_memory_context(|_| unsafe {
            compare_textcat(&a, &b);
        });
    }
}
