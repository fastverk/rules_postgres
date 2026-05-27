//! Postgres MemoryContext adapter for the cluster-driven C→Rust
//! translation pipeline.
//!
//! Promoted from the pgfnames spike. Same core design — `MemoryContext`
//! is a Rust newtype around `bumpalo::Bump`; `palloc/pfree/repalloc`
//! mirror Postgres' API; `pfree` is a no-op because bumpalo reclaims
//! on context reset.
//!
//! ## What changed from the spike
//!
//! ### Chunk header (8 bytes per allocation)
//!
//! The spike documented a sharp edge: `repalloc(p, smaller)` over-copied
//! by reading the truncated tail (because the spike had no way to know
//! the old allocation's size from the pointer alone). Closed here by
//! stashing an 8-byte size header before every allocation:
//!
//! ```text
//!   bump alloc:  [ chunk_size : u64 ] [ ............ user data ............ ]
//!                ^ raw pointer        ^ returned to caller (raw + 8)
//! ```
//!
//! `repalloc(p, new_size)` reads the header at `p - 8` to find the old
//! size, allocates `new_size + 8` bytes fresh, copies `min(old, new)`
//! bytes, and returns a pointer past the new header. `pfree(p)` is
//! still a no-op (bumpalo can't free; the header is harmless).
//!
//! All allocations are MAXALIGN-aligned (8 bytes on 64-bit) — matches
//! Postgres' `MAXALIGN` on the typical 64-bit build.
//!
//! ### Full Postgres palloc API surface
//!
//! Beyond what the spike exposed, this crate adds:
//!
//! - `MemoryContextAlloc(ctx, size)` / `MemoryContextAllocZero` /
//!   `MemoryContextAllocExtended` — explicit-context allocations that
//!   don't go through the thread-local current context.
//! - `palloc_extended(size, flags)` with `MCXT_ALLOC_*` flag bits.
//!   Today we honor `MCXT_ALLOC_ZERO` and `MCXT_ALLOC_NO_OOM` (the
//!   latter trivially since bumpalo can't really OOM without aborting).
//! - `MemoryContextReset(ctx)` — frees all allocations in a context
//!   but keeps the context alive (matches Postgres exactly).
//! - `MemoryContextDelete(ctx)` — destroys the context entirely. Calls
//!   `Box::from_raw` to actually drop the underlying Bump.
//! - `pnstrdup(s, n)` — bounded-length strdup.
//!
//! ### Companion C header
//!
//! `include/pg_palloc.h` declares the C-ABI surface so future C oracles
//! in palloc-using crates (e.g., interval, array meta-ops, bytea) can
//! `#include "pg_palloc.h"` and link against this crate.

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(clippy::missing_safety_doc)]

use bumpalo::Bump;
use std::cell::Cell;

/// Re-export of `bumpalo::Bump` so dependents of `pg_palloc` don't
/// also need to take a direct bumpalo dependency. Used by the
/// "safer Rust-only API" entry points (e.g., `pgfnames_safe` in the
/// pgfnames crate, future Rust-friendly translations).
pub use bumpalo::Bump as PgBump;
use std::ffi::{c_void, CStr};
use std::os::raw::c_char;
use std::ptr;

// ─── Chunk header ──────────────────────────────────────────────────────────

/// Size of the per-allocation header. Stored as a `u64` right before
/// every returned pointer; reading it gives the chunk's payload size
/// in bytes (excludes the header itself).
const CHUNK_HEADER_SIZE: usize = 8;

/// The alignment we promise to callers. Matches Postgres' `MAXALIGN`
/// on standard 64-bit builds.
const MAXALIGN: usize = 8;

/// Compute the total bump-arena allocation size for a chunk holding
/// `payload` user bytes. Always at least 1 (bumpalo rejects 0-size).
#[inline]
fn chunk_total_size(payload: usize) -> usize {
    CHUNK_HEADER_SIZE + payload.max(1)
}

/// SAFETY: `user_ptr` must be a pointer returned by `palloc` family
/// (i.e., 8 bytes past a header we wrote).
#[inline]
unsafe fn read_chunk_size(user_ptr: *mut u8) -> usize {
    let header_ptr = user_ptr.sub(CHUNK_HEADER_SIZE).cast::<u64>();
    *header_ptr as usize
}

#[inline]
unsafe fn write_chunk_size(raw_ptr: *mut u8, size: usize) {
    *raw_ptr.cast::<u64>() = size as u64;
}

// ─── MemoryContext ─────────────────────────────────────────────────────────

/// Opaque-to-C handle. C code only ever holds `*mut MemoryContext` and
/// never inspects fields — matches Postgres'
/// `typedef struct MemoryContextData *MemoryContext`.
pub struct MemoryContext {
    bump: Bump,
}

impl MemoryContext {
    pub fn new() -> Self {
        MemoryContext { bump: Bump::new() }
    }

    /// Direct access for safe-API callers.
    pub fn bump(&self) -> &Bump {
        &self.bump
    }

    /// `MemoryContextReset` — frees all allocations, keeps context.
    pub fn reset(&mut self) {
        self.bump.reset();
    }

    /// Allocate `size` payload bytes from this context, returning a
    /// user pointer with a chunk-size header behind it. Used by all
    /// the public palloc functions.
    unsafe fn alloc_chunk(&self, payload: usize) -> *mut u8 {
        let total = chunk_total_size(payload);
        let layout = std::alloc::Layout::from_size_align(total, MAXALIGN)
            .expect("invalid layout in pg_palloc::alloc_chunk");
        let raw = self.bump.alloc_layout(layout).as_ptr();
        write_chunk_size(raw, payload);
        raw.add(CHUNK_HEADER_SIZE)
    }
}

impl Default for MemoryContext {
    fn default() -> Self {
        Self::new()
    }
}

// ─── CurrentMemoryContext (thread-local) ──────────────────────────────────

thread_local! {
    static CURRENT_MEMORY_CONTEXT: Cell<*mut MemoryContext> =
        const { Cell::new(ptr::null_mut()) };
}

/// `MemoryContextSwitchTo(ctx)` — sets `CurrentMemoryContext`, returns
/// the previous value.
///
/// # Safety
/// `ctx` must point to a live `MemoryContext` (or be null). The caller
/// is responsible for keeping it alive while any palloc'd buffer from
/// it is in use.
#[no_mangle]
pub unsafe extern "C" fn MemoryContextSwitchTo(ctx: *mut MemoryContext) -> *mut MemoryContext {
    CURRENT_MEMORY_CONTEXT.with(|c| {
        let prev = c.get();
        c.set(ctx);
        prev
    })
}

/// Legacy snake_case alias kept for the pgfnames-spike call sites.
/// New code should prefer `MemoryContextSwitchTo`.
pub unsafe fn memory_context_switch_to(ctx: *mut MemoryContext) -> *mut MemoryContext {
    MemoryContextSwitchTo(ctx)
}

#[inline]
unsafe fn current_context<'a>() -> &'a MemoryContext {
    let ctx = CURRENT_MEMORY_CONTEXT.with(|c| c.get());
    assert!(
        !ctx.is_null(),
        "palloc/pstrdup/repalloc called with no CurrentMemoryContext set"
    );
    &*ctx
}

/// Returns the current MemoryContext as an opaque pointer. Postgres'
/// `CurrentMemoryContext` is a global variable; we mirror that via TLS.
#[no_mangle]
pub unsafe extern "C" fn GetCurrentMemoryContext() -> *mut MemoryContext {
    CURRENT_MEMORY_CONTEXT.with(|c| c.get())
}

// ─── palloc / palloc0 / palloc_extended ───────────────────────────────────

/// Flag bits for `palloc_extended` / `MemoryContextAllocExtended`. Match
/// Postgres' `src/include/utils/palloc.h`.
pub const MCXT_ALLOC_HUGE:     u32 = 0x01; // allow >1 GiB allocation
pub const MCXT_ALLOC_NO_OOM:   u32 = 0x02; // return NULL on OOM instead of ereport
pub const MCXT_ALLOC_ZERO:     u32 = 0x04; // zero-fill before returning

/// `void *palloc(Size size)`. Allocate `size` bytes from the current
/// memory context.
///
/// # Safety
/// `CurrentMemoryContext` must be set (via `MemoryContextSwitchTo`).
#[no_mangle]
pub unsafe extern "C" fn palloc(size: usize) -> *mut u8 {
    current_context().alloc_chunk(size)
}

/// `void *palloc0(Size size)` — palloc + zero-fill.
#[no_mangle]
pub unsafe extern "C" fn palloc0(size: usize) -> *mut u8 {
    let p = palloc(size);
    ptr::write_bytes(p, 0, size);
    p
}

/// `void *palloc_extended(Size size, int flags)`.
///
/// Honors `MCXT_ALLOC_ZERO` (zero-fill the payload). `MCXT_ALLOC_HUGE`
/// and `MCXT_ALLOC_NO_OOM` are accepted but no-op: bumpalo doesn't
/// distinguish huge allocations, and OOM aborts the process rather than
/// returning NULL (matches Postgres' own behavior when
/// `MCXT_ALLOC_NO_OOM` is NOT set, which is the common case).
#[no_mangle]
pub unsafe extern "C" fn palloc_extended(size: usize, flags: u32) -> *mut u8 {
    let p = palloc(size);
    if (flags & MCXT_ALLOC_ZERO) != 0 {
        ptr::write_bytes(p, 0, size);
    }
    p
}

// ─── pfree / repalloc ─────────────────────────────────────────────────────

/// `void pfree(void *pointer)`.
///
/// No-op. Memory is reclaimed by `MemoryContextReset` /
/// `MemoryContextDelete`. Double-frees are harmless.
#[no_mangle]
pub unsafe extern "C" fn pfree(_pointer: *mut u8) {}

/// `void *repalloc(void *pointer, Size size)`.
///
/// Reads the chunk header to find the old size, allocates fresh, and
/// copies `min(old, new)` bytes. The old allocation is leaked inside
/// the arena (reclaimed at MemoryContextReset).
///
/// # Safety
/// `pointer` must be a pointer previously returned by `palloc`,
/// `palloc0`, `palloc_extended`, `repalloc`, `MemoryContextAlloc*`, or
/// `pstrdup`/`pnstrdup` — i.e., something with a chunk header. Passing
/// `NULL` returns a fresh allocation of `size` bytes.
#[no_mangle]
pub unsafe extern "C" fn repalloc(pointer: *mut u8, size: usize) -> *mut u8 {
    if pointer.is_null() {
        return palloc(size);
    }
    let old_size = read_chunk_size(pointer);
    let new = palloc(size);
    let copy_bytes = old_size.min(size);
    if copy_bytes > 0 {
        ptr::copy_nonoverlapping(pointer, new, copy_bytes);
    }
    new
}

/// Alias for `repalloc` with no >1GiB limit on 64-bit (matches the
/// Postgres macro of the same name).
#[no_mangle]
pub unsafe extern "C" fn repalloc_huge(pointer: *mut u8, size: usize) -> *mut u8 {
    repalloc(pointer, size)
}

// ─── pstrdup / pnstrdup ───────────────────────────────────────────────────

/// `char *pstrdup(const char *in)` — palloc + memcpy of a NUL-terminated
/// string (NUL included).
#[no_mangle]
pub unsafe extern "C" fn pstrdup(s: *const c_char) -> *mut c_char {
    if s.is_null() {
        return ptr::null_mut();
    }
    let cs = CStr::from_ptr(s);
    let bytes = cs.to_bytes_with_nul();
    let dst = palloc(bytes.len());
    ptr::copy_nonoverlapping(bytes.as_ptr(), dst, bytes.len());
    dst.cast()
}

/// `char *pnstrdup(const char *in, Size len)` — palloc + memcpy of at
/// most `len` bytes (stopping at first NUL, then NUL-terminating).
#[no_mangle]
pub unsafe extern "C" fn pnstrdup(s: *const c_char, len: usize) -> *mut c_char {
    if s.is_null() {
        return ptr::null_mut();
    }
    let cs = CStr::from_ptr(s);
    let bytes = cs.to_bytes();
    let take = bytes.len().min(len);
    let dst = palloc(take + 1);
    ptr::copy_nonoverlapping(bytes.as_ptr(), dst, take);
    *dst.add(take) = 0;
    dst.cast()
}

// ─── MemoryContextAlloc family (explicit-context allocation) ──────────────

/// `void *MemoryContextAlloc(MemoryContext context, Size size)`.
/// Allocate from a specific context, ignoring the current one.
#[no_mangle]
pub unsafe extern "C" fn MemoryContextAlloc(
    context: *mut MemoryContext,
    size: usize,
) -> *mut u8 {
    assert!(!context.is_null(), "MemoryContextAlloc called with NULL context");
    (*context).alloc_chunk(size)
}

#[no_mangle]
pub unsafe extern "C" fn MemoryContextAllocZero(
    context: *mut MemoryContext,
    size: usize,
) -> *mut u8 {
    let p = MemoryContextAlloc(context, size);
    ptr::write_bytes(p, 0, size);
    p
}

#[no_mangle]
pub unsafe extern "C" fn MemoryContextAllocExtended(
    context: *mut MemoryContext,
    size: usize,
    flags: u32,
) -> *mut u8 {
    let p = MemoryContextAlloc(context, size);
    if (flags & MCXT_ALLOC_ZERO) != 0 {
        ptr::write_bytes(p, 0, size);
    }
    p
}

// ─── MemoryContextReset / MemoryContextDelete ─────────────────────────────

/// `void MemoryContextReset(MemoryContext context)`.
///
/// # Safety
/// `context` must be a live `MemoryContext`. All pointers previously
/// returned from this context are invalidated.
#[no_mangle]
pub unsafe extern "C" fn MemoryContextReset(context: *mut MemoryContext) {
    if context.is_null() {
        return;
    }
    (*context).reset();
}

/// `void MemoryContextDelete(MemoryContext context)`.
///
/// Drops the context and its underlying Bump entirely. The pointer must
/// have been obtained from `MemoryContextCreate` (i.e., be heap-owned).
///
/// # Safety
/// `context` must be a pointer that was `Box::leak`ed by
/// `MemoryContextCreate` (or equivalent — `with_memory_context` / tests
/// use Box::new internally, which is also fine here).
#[no_mangle]
pub unsafe extern "C" fn MemoryContextDelete(context: *mut MemoryContext) {
    if context.is_null() {
        return;
    }
    // If `context` is the current TLS pointer, clear it first so a
    // post-delete palloc fails the null assertion rather than UAFing.
    CURRENT_MEMORY_CONTEXT.with(|c| {
        if c.get() == context {
            c.set(ptr::null_mut());
        }
    });
    drop(Box::from_raw(context));
}

/// `MemoryContext MemoryContextCreate(...)` — simplified to just create
/// a fresh context. Postgres' real signature takes parent/identifier
/// strings; we ignore those for the V0 adapter since we don't model the
/// hierarchy. Returns a heap-owned pointer to be released via
/// `MemoryContextDelete`.
#[no_mangle]
pub unsafe extern "C" fn MemoryContextCreate() -> *mut MemoryContext {
    Box::into_raw(Box::new(MemoryContext::new()))
}

// ─── Safe Rust helper for tests / Rust-only callers ────────────────────────

/// Run `f` under a freshly-created MemoryContext set as
/// `CurrentMemoryContext`. Restores the previous context (and drops the
/// new arena) after.
pub fn with_memory_context<R>(f: impl FnOnce(*mut MemoryContext) -> R) -> R {
    let raw = unsafe { MemoryContextCreate() };
    let prev = unsafe { MemoryContextSwitchTo(raw) };
    let r = f(raw);
    unsafe {
        MemoryContextSwitchTo(prev);
        MemoryContextDelete(raw);
    }
    r
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palloc_then_no_pfree_is_fine() {
        with_memory_context(|_| unsafe {
            let p = palloc(16);
            assert!(!p.is_null());
        });
    }

    #[test]
    fn palloc0_is_zeroed() {
        with_memory_context(|_| unsafe {
            let p = palloc0(32);
            for i in 0..32 {
                assert_eq!(*p.add(i), 0, "byte {i} not zeroed");
            }
        });
    }

    #[test]
    fn palloc_extended_zero_flag() {
        with_memory_context(|_| unsafe {
            let p = palloc_extended(16, MCXT_ALLOC_ZERO);
            for i in 0..16 {
                assert_eq!(*p.add(i), 0);
            }
        });
    }

    #[test]
    fn chunk_header_round_trips() {
        with_memory_context(|_| unsafe {
            let p = palloc(123);
            assert_eq!(read_chunk_size(p), 123);
        });
    }

    #[test]
    fn pstrdup_round_trip() {
        with_memory_context(|_| unsafe {
            let src = b"hello\0";
            let dup = pstrdup(src.as_ptr().cast());
            let back = CStr::from_ptr(dup);
            assert_eq!(back.to_bytes(), b"hello");
        });
    }

    #[test]
    fn pnstrdup_truncates_and_terminates() {
        with_memory_context(|_| unsafe {
            let src = b"hello world\0";
            let dup = pnstrdup(src.as_ptr().cast(), 5);
            let back = CStr::from_ptr(dup);
            assert_eq!(back.to_bytes(), b"hello");
        });
    }

    #[test]
    fn pnstrdup_short_string_passes_through() {
        with_memory_context(|_| unsafe {
            let src = b"hi\0";
            let dup = pnstrdup(src.as_ptr().cast(), 100);
            let back = CStr::from_ptr(dup);
            assert_eq!(back.to_bytes(), b"hi");
        });
    }

    #[test]
    fn repalloc_grow_preserves_payload() {
        with_memory_context(|_| unsafe {
            let p = palloc(4);
            *p.add(0) = b'a';
            *p.add(1) = b'b';
            *p.add(2) = b'c';
            *p.add(3) = b'd';
            let p2 = repalloc(p, 16);
            assert_eq!(*p2.add(0), b'a');
            assert_eq!(*p2.add(3), b'd');
        });
    }

    #[test]
    fn repalloc_shrink_no_uninit_read() {
        // This is the case the spike documented as a sharp edge.
        // With the chunk header in place, repalloc shrink reads only
        // min(old=8, new=4) = 4 bytes — no over-read.
        with_memory_context(|_| unsafe {
            let p = palloc(8);
            for i in 0..8 { *p.add(i) = (i + 1) as u8; }
            let p2 = repalloc(p, 4);
            assert_eq!(read_chunk_size(p2), 4);
            for i in 0..4 {
                assert_eq!(*p2.add(i), (i + 1) as u8);
            }
        });
    }

    #[test]
    fn repalloc_null_acts_as_palloc() {
        with_memory_context(|_| unsafe {
            let p = repalloc(ptr::null_mut(), 8);
            assert!(!p.is_null());
            assert_eq!(read_chunk_size(p), 8);
        });
    }

    #[test]
    fn memory_context_alloc_explicit() {
        unsafe {
            let ctx_a = MemoryContextCreate();
            let ctx_b = MemoryContextCreate();
            let pa = MemoryContextAlloc(ctx_a, 16);
            let pb = MemoryContextAllocZero(ctx_b, 16);
            assert!(!pa.is_null() && !pb.is_null());
            for i in 0..16 { assert_eq!(*pb.add(i), 0); }
            MemoryContextDelete(ctx_a);
            MemoryContextDelete(ctx_b);
        }
    }

    #[test]
    fn memory_context_reset_keeps_context_alive() {
        unsafe {
            let ctx = MemoryContextCreate();
            let prev = MemoryContextSwitchTo(ctx);
            let _p1 = palloc(64);
            MemoryContextReset(ctx);
            // After reset, the context is still usable.
            let _p2 = palloc(64);
            MemoryContextSwitchTo(prev);
            MemoryContextDelete(ctx);
        }
    }

    #[test]
    fn memory_context_delete_clears_current_if_match() {
        unsafe {
            let ctx = MemoryContextCreate();
            let prev = MemoryContextSwitchTo(ctx);
            assert_eq!(GetCurrentMemoryContext(), ctx);
            MemoryContextDelete(ctx);
            // After deleting the current context, GetCurrentMemoryContext
            // must return NULL (otherwise a subsequent palloc would UAF).
            assert!(GetCurrentMemoryContext().is_null());
            MemoryContextSwitchTo(prev);
        }
    }

    #[test]
    fn maxalign_satisfied() {
        with_memory_context(|_| unsafe {
            for sz in [1, 7, 8, 9, 15, 16, 100, 1000] {
                let p = palloc(sz);
                assert_eq!(
                    (p as usize) % MAXALIGN,
                    0,
                    "palloc({sz}) returned non-MAXALIGN-aligned pointer {p:p}"
                );
            }
        });
    }
}

// ─── Internal helpers re-exported for paired Rust callers ──────────────────

/// Re-export `c_void` so future Pg.Ir-emitted Rust can write
/// `*mut c_void` arguments without an additional std import.
pub use std::ffi::c_void as PalloccVoid;

// Tag this symbol so the unused-import is suppressed.
#[doc(hidden)]
pub const fn __palloc_force_keep_c_void() -> Option<*mut c_void> { None }
