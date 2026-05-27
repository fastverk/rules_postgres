//! Smoke test for the pg_palloc C ABI surface.
//!
//! Compiles a small C harness that exercises palloc/repalloc/pfree/
//! pstrdup/MemoryContext{Create,Reset,Delete,SwitchTo} through the
//! pg_palloc.h header, then links it against pg_palloc's staticlib
//! and asserts the round-trip works end-to-end.
//!
//! This is the prerequisite gate before any palloc-using cluster
//! (array meta-ops, interval, bytea, …) vendors its C oracle: the C
//! side must be able to call palloc and see exactly the same arena
//! behavior the Rust side sees.

use std::os::raw::{c_char, c_void};

extern "C" {
    fn c_smoke_pstrdup_round_trip() -> i32;
    fn c_smoke_repalloc_grow_then_shrink() -> i32;
    fn c_smoke_explicit_context_independent_arenas() -> i32;
    fn c_smoke_reset_keeps_context_usable() -> i32;
}

// Re-export the pg_palloc symbols the C harness needs to call before
// its own functions are invoked.
//
// (The harness's functions are invoked from Rust below.)

#[test]
fn pstrdup_round_trip_via_c() {
    pg_palloc::with_memory_context(|_| {
        let rc = unsafe { c_smoke_pstrdup_round_trip() };
        assert_eq!(rc, 0, "c_smoke_pstrdup_round_trip failed with code {rc}");
    });
}

#[test]
fn repalloc_grow_then_shrink_via_c() {
    pg_palloc::with_memory_context(|_| {
        let rc = unsafe { c_smoke_repalloc_grow_then_shrink() };
        assert_eq!(rc, 0, "c_smoke_repalloc_grow_then_shrink failed with code {rc}");
    });
}

#[test]
fn explicit_context_independent_arenas_via_c() {
    let rc = unsafe { c_smoke_explicit_context_independent_arenas() };
    assert_eq!(rc, 0, "c_smoke_explicit_context_independent_arenas failed with code {rc}");
}

#[test]
fn reset_keeps_context_usable_via_c() {
    let rc = unsafe { c_smoke_reset_keeps_context_usable() };
    assert_eq!(rc, 0, "c_smoke_reset_keeps_context_usable failed with code {rc}");
}

// Touch types so the import isn't dead even though the C harness is
// what actually exercises them.
#[allow(dead_code)]
fn _types(_: *mut c_char, _: *mut c_void) {}
