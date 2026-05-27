//! Smoke test for the interval-arithmetic Pg.Ir cluster (v0-v1:
//! `interval_um`, `interval_pl`, `interval_mi`).
//!
//! Validates the round-trip across both sides:
//!   - Allocate an Interval input(s) in a shared MemoryContext.
//!   - Construct a fcinfo whose args[N].value is the input pointer(s).
//!   - Invoke each side (Rust translation, vendored C oracle).
//!   - Both sides palloc THEIR OWN result Interval in the same arena.
//!   - Compare the result-pointer *contents* field-by-field (pointer
//!     equality would be wrong — different palloc calls produce
//!     different addresses for the two sides).
//!   - On error, both must raise FmgrErrorKind::DatetimeValueOutOfRange.

use pg_fcinfo::{
    build_fcinfo, decode_interval_p, encode_interval_p,
    fmgr_clear_error, fmgr_take_error, FmgrErrorKind,
    FunctionCallInfoBaseData, Interval,
};
use pg_palloc::{palloc, with_memory_context, MemoryContext};
use proptest::prelude::*;
use std::mem::size_of;
use std::ptr;

extern "C" {
    fn c_interval_um(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_interval_pl(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
    fn c_interval_mi(fcinfo: *mut FunctionCallInfoBaseData) -> u64;
}

type FmgrFn = unsafe extern "C" fn(*mut FunctionCallInfoBaseData) -> u64;

/// Allocate an Interval in the current MemoryContext, initialize it
/// with the given field values, and return the pointer (cast to Datum
/// for passing in fcinfo.args[0].value).
unsafe fn alloc_interval(time: i64, day: i32, month: i32) -> *mut Interval {
    let p = palloc(size_of::<Interval>()) as *mut Interval;
    (*p).time = time;
    (*p).day = day;
    (*p).month = month;
    p
}

/// Invoke one fmgr stub with a typed Interval input. Returns
/// (output_datum, error_state). The caller decodes the output if no
/// error.
unsafe fn invoke(fn_: FmgrFn, input: *const Interval) -> (u64, Option<FmgrErrorKind>) {
    fmgr_clear_error();
    // Pass the pointer as a Datum (u64). build_fcinfo!'s mut_ptr tag
    // encodes a *mut T into Datum exactly that way.
    let mut fcinfo = build_fcinfo!(args = [(mut_ptr, input as *mut std::ffi::c_void)]);
    let out = fn_(&mut fcinfo);
    (out, fmgr_take_error())
}

unsafe fn compare_interval_um(input: *const Interval) {
    let (c_out, c_err) = invoke(c_interval_um, input);
    let (r_out, r_err) = invoke(pg_interval::interval_um, input);
    assert_eq!(
        c_err, r_err,
        "interval_um({:?}): C err = {c_err:?}, Rust err = {r_err:?}",
        *input,
    );
    if r_err.is_none() {
        let c_iv = &*decode_interval_p(c_out);
        let r_iv = &*decode_interval_p(r_out);
        assert_eq!(c_iv.time,  r_iv.time,
            "interval_um({:?}): time differs C={} R={}", *input, c_iv.time, r_iv.time);
        assert_eq!(c_iv.day,   r_iv.day,
            "interval_um({:?}): day differs C={} R={}", *input, c_iv.day, r_iv.day);
        assert_eq!(c_iv.month, r_iv.month,
            "interval_um({:?}): month differs C={} R={}", *input, c_iv.month, r_iv.month);
    }
}

/// Invoke a binary interval function with two inputs. Returns
/// (output_datum, error_state).
unsafe fn invoke_binary(fn_: unsafe extern "C" fn(*mut FunctionCallInfoBaseData) -> u64,
                       span1: *const Interval, span2: *const Interval) -> (u64, Option<FmgrErrorKind>) {
    fmgr_clear_error();
    let mut fcinfo = build_fcinfo!(args = [
        (mut_ptr, span1 as *mut std::ffi::c_void),
        (mut_ptr, span2 as *mut std::ffi::c_void)
    ]);
    let out = fn_(&mut fcinfo);
    (out, fmgr_take_error())
}

unsafe fn compare_interval_pl(span1: *const Interval, span2: *const Interval) {
    let (c_out, c_err) = invoke_binary(c_interval_pl, span1, span2);
    let (r_out, r_err) = invoke_binary(pg_interval::interval_pl, span1, span2);
    assert_eq!(
        c_err, r_err,
        "interval_pl({:?}, {:?}): C err = {c_err:?}, Rust err = {r_err:?}",
        *span1, *span2,
    );
    if r_err.is_none() {
        let c_iv = &*decode_interval_p(c_out);
        let r_iv = &*decode_interval_p(r_out);
        assert_eq!(c_iv.time,  r_iv.time,
            "interval_pl({:?}, {:?}): time differs C={} R={}", *span1, *span2, c_iv.time, r_iv.time);
        assert_eq!(c_iv.day,   r_iv.day,
            "interval_pl({:?}, {:?}): day differs C={} R={}", *span1, *span2, c_iv.day, r_iv.day);
        assert_eq!(c_iv.month, r_iv.month,
            "interval_pl({:?}, {:?}): month differs C={} R={}", *span1, *span2, c_iv.month, r_iv.month);
    }
}

unsafe fn compare_interval_mi(span1: *const Interval, span2: *const Interval) {
    let (c_out, c_err) = invoke_binary(c_interval_mi, span1, span2);
    let (r_out, r_err) = invoke_binary(pg_interval::interval_mi, span1, span2);
    assert_eq!(
        c_err, r_err,
        "interval_mi({:?}, {:?}): C err = {c_err:?}, Rust err = {r_err:?}",
        *span1, *span2,
    );
    if r_err.is_none() {
        let c_iv = &*decode_interval_p(c_out);
        let r_iv = &*decode_interval_p(r_out);
        assert_eq!(c_iv.time,  r_iv.time,
            "interval_mi({:?}, {:?}): time differs C={} R={}", *span1, *span2, c_iv.time, r_iv.time);
        assert_eq!(c_iv.day,   r_iv.day,
            "interval_mi({:?}, {:?}): day differs C={} R={}", *span1, *span2, c_iv.day, r_iv.day);
        assert_eq!(c_iv.month, r_iv.month,
            "interval_mi({:?}, {:?}): month differs C={} R={}", *span1, *span2, c_iv.month, r_iv.month);
    }
}

#[test]
fn boundary_interval_um() {
    with_memory_context(|_ctx: *mut MemoryContext| unsafe {
        // Normal cases — negate works, no overflow.
        for &(t, d, m) in &[
            (0i64, 0i32, 0i32),
            (1, 1, 1),
            (-1, -1, -1),
            (1_000_000, 30, 12),
            (-1_000_000, -30, -12),
        ] {
            let p = alloc_interval(t, d, m);
            compare_interval_um(p);
        }

        // Sentinel cases — NOBEGIN maps to NOEND and vice versa.
        // (Tested by checking pointer contents match.)
        // -infinity input → +infinity output.
        let neg_inf = alloc_interval(i64::MIN, i32::MIN, i32::MIN);
        compare_interval_um(neg_inf);
        // +infinity input → -infinity output.
        let pos_inf = alloc_interval(i64::MAX, i32::MAX, i32::MAX);
        compare_interval_um(pos_inf);

        // Overflow corners — negating MIN values overflows.
        // INT64_MIN's negation overflows int64.
        let p1 = alloc_interval(i64::MIN, 0, 0);
        compare_interval_um(p1);
        // INT32_MIN day's negation overflows int32.
        let p2 = alloc_interval(0, i32::MIN, 0);
        compare_interval_um(p2);
        // INT32_MIN month's negation overflows int32.
        let p3 = alloc_interval(0, 0, i32::MIN);
        compare_interval_um(p3);

        // The trickiest case: a non-sentinel input whose negation
        // LANDS on the +infinity sentinel post-negate. Real Postgres'
        // `INTERVAL_NOT_FINITE(result)` check catches this. Concretely:
        // input (-i64::MAX, -i32::MAX, -i32::MAX) negates to (+MAX,
        // +MAX, +MAX) which IS the +infinity sentinel, so PG raises.
        let p4 = alloc_interval(-i64::MAX, -i32::MAX, -i32::MAX);
        compare_interval_um(p4);
    });
}

#[test]
fn boundary_interval_pl() {
    with_memory_context(|_ctx: *mut MemoryContext| unsafe {
        // Normal + normal = normal (no overflow).
        let p1 = alloc_interval(1000, 10, 2);
        let p2 = alloc_interval(2000, 20, 5);
        compare_interval_pl(p1, p2);

        // NOBEGIN + (normal or NOEND) case.
        let nobegin = alloc_interval(i64::MIN, i32::MIN, i32::MIN);
        let normal = alloc_interval(100, 5, 1);
        let noend = alloc_interval(i64::MAX, i32::MAX, i32::MAX);

        // NOBEGIN + normal → NOBEGIN
        compare_interval_pl(nobegin, normal);
        // NOBEGIN + NOEND → ERROR
        compare_interval_pl(nobegin, noend);

        // NOEND + (normal or NOBEGIN) case.
        // NOEND + normal → NOEND
        compare_interval_pl(noend, normal);
        // NOEND + NOBEGIN → ERROR
        compare_interval_pl(noend, nobegin);

        // normal + (NOBEGIN or NOEND) — the second operand's infinity dominates.
        // normal + NOBEGIN → NOBEGIN (memcpy)
        compare_interval_pl(normal, nobegin);
        // normal + NOEND → NOEND (memcpy)
        compare_interval_pl(normal, noend);

        // Overflow corner on the finite path: large values that overflow.
        let big_pos = alloc_interval(i64::MAX / 2, i32::MAX / 2, i32::MAX / 2);
        let big_pos2 = alloc_interval(i64::MAX / 2 + 100, i32::MAX / 2 + 100, i32::MAX / 2 + 100);
        compare_interval_pl(big_pos, big_pos2); // Should error (overflow)
    });
}

#[test]
fn boundary_interval_mi() {
    with_memory_context(|_ctx: *mut MemoryContext| unsafe {
        // Normal - normal = normal (no overflow).
        let p1 = alloc_interval(5000, 50, 12);
        let p2 = alloc_interval(2000, 20, 5);
        compare_interval_mi(p1, p2);

        // NOBEGIN - (normal or NOBEGIN) case.
        let nobegin = alloc_interval(i64::MIN, i32::MIN, i32::MIN);
        let normal = alloc_interval(100, 5, 1);

        // NOBEGIN - normal → NOBEGIN
        compare_interval_mi(nobegin, normal);
        // NOBEGIN - NOBEGIN → ERROR
        compare_interval_mi(nobegin, nobegin);

        let noend = alloc_interval(i64::MAX, i32::MAX, i32::MAX);

        // NOEND - (normal or NOEND) case.
        // NOEND - normal → NOEND
        compare_interval_mi(noend, normal);
        // NOEND - NOEND → ERROR
        compare_interval_mi(noend, noend);

        // normal - (NOBEGIN or NOEND) — negates the second operand's infinity.
        // normal - NOBEGIN → NOEND (complement)
        compare_interval_mi(normal, nobegin);
        // normal - NOEND → NOBEGIN (complement)
        compare_interval_mi(normal, noend);

        // Overflow corner on the finite path.
        let big_pos = alloc_interval(i64::MAX / 2, i32::MAX / 2, i32::MAX / 2);
        let big_neg = alloc_interval(-(i64::MAX / 2) - 100, -(i32::MAX / 2) - 100, -(i32::MAX / 2) - 100);
        compare_interval_mi(big_pos, big_neg); // Should error (overflow when subtracting negatives)
    });
}

#[test]
fn no_error_set_when_input_normal() {
    with_memory_context(|_| unsafe {
        let p = alloc_interval(42, 7, 3);
        fmgr_clear_error();
        let mut fcinfo = build_fcinfo!(args = [(mut_ptr, p as *mut std::ffi::c_void)]);
        let out = pg_interval::interval_um(&mut fcinfo);
        assert!(fmgr_take_error().is_none());
        assert_ne!(out, 0);
        let result = &*decode_interval_p(out);
        assert_eq!(result.time, -42);
        assert_eq!(result.day, -7);
        assert_eq!(result.month, -3);
    });
}

#[test]
fn double_negate_is_identity_on_normal_values() {
    with_memory_context(|_| unsafe {
        let p1 = alloc_interval(123_456, 9, -5);
        let mut fc1 = build_fcinfo!(args = [(mut_ptr, p1 as *mut std::ffi::c_void)]);
        fmgr_clear_error();
        let out1 = pg_interval::interval_um(&mut fc1);
        assert!(fmgr_take_error().is_none());

        let mut fc2 = build_fcinfo!(args = [(mut_ptr, out1 as *mut std::ffi::c_void)]);
        fmgr_clear_error();
        let out2 = pg_interval::interval_um(&mut fc2);
        assert!(fmgr_take_error().is_none());

        let back = &*decode_interval_p(out2);
        assert_eq!(back.time,  123_456);
        assert_eq!(back.day,   9);
        assert_eq!(back.month, -5);
    });
}

// Suppress unused-import lint on path imports that aid the test
// expansion but aren't dereferenced syntactically here.
#[allow(dead_code)]
fn _types(_: *mut std::ffi::c_void) {
    let _ = encode_interval_p;
    let _ = ptr::null_mut::<Interval>();
}

// ─── Proptest sweep ─────────────────────────────────────────────────────────
//
// Random (time, day, month) triples. Each test case allocates its own
// MemoryContext (via with_memory_context) so arena cleanup is
// per-iteration — no cross-case state leakage. proptest's default
// shrinker will narrow down failing inputs to minimal corner cases.

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2048))]

    #[test]
    fn diff_interval_um_proptest(
        time  in any::<i64>(),
        day   in any::<i32>(),
        month in any::<i32>(),
    ) {
        with_memory_context(|_| unsafe {
            let p = alloc_interval(time, day, month);
            compare_interval_um(p);
        });
    }

    #[test]
    fn diff_interval_pl_proptest(
        t1 in any::<i64>(), d1 in any::<i32>(), m1 in any::<i32>(),
        t2 in any::<i64>(), d2 in any::<i32>(), m2 in any::<i32>(),
    ) {
        with_memory_context(|_| unsafe {
            let p1 = alloc_interval(t1, d1, m1);
            let p2 = alloc_interval(t2, d2, m2);
            compare_interval_pl(p1, p2);
        });
    }

    #[test]
    fn diff_interval_mi_proptest(
        t1 in any::<i64>(), d1 in any::<i32>(), m1 in any::<i32>(),
        t2 in any::<i64>(), d2 in any::<i32>(), m2 in any::<i32>(),
    ) {
        with_memory_context(|_| unsafe {
            let p1 = alloc_interval(t1, d1, m1);
            let p2 = alloc_interval(t2, d2, m2);
            compare_interval_mi(p1, p2);
        });
    }
}
