//! Hand-translation of Postgres' `pqCheckOutBufferSpace` /
//! `pqCheckInBufferSpace` from `src/interfaces/libpq/fe-misc.c` → Rust.
//!
//! Second milestone of the cluster-driven C→Rust translation spike.
//! The SHA family in `translated/sha2/` validated the pipeline on a
//! tight algorithmic cluster (4 hash variants, ~600 lines). This crate
//! validates it on a *different shape*: a two-member cluster where the
//! sibling has extra prelude logic the canonical lacks (left-justify
//! of `inBuffer` contents before the grow loop). The cluster surface
//! is the inner grow loop; the prelude is hand-written.
//!
//! ## Layout
//!
//! - `PgConn`: `#[repr(C)]` newtype matching the libpq `PGconn`
//!   fields we touch (`inBuffer`, `inBufSize`, `inStart`, `inCursor`,
//!   `inEnd`, `outBuffer`, `outBufSize`). The real `PGconn` has 50+
//!   fields; we model only the buffer-management subset and accept
//!   that the struct is layout-incompatible with the full C version.
//!   ABI-compat would require declaring every field in the original
//!   order, which is out of scope for the cluster-validation spike.
//!
//! - `pqCheckOutBufferSpace`, `pqCheckInBufferSpace`: faithful
//!   translations of the C originals (fe-misc.c lines 287-342 and
//!   351-441 in PG17). Both `#[no_mangle] extern "C"` for ABI
//!   parity with libpq's exports.
//!
//! - `try_grow_out_buffer`, `try_grow_in_buffer`: the cluster's
//!   *inner* grow loop, factored out so the translator engine has a
//!   shape-identical canonical/sibling pair to lift between. The
//!   outer functions call these helpers.
//!
//! ## What this crate proves about the pipeline
//!
//! 1. **Tensor-side cluster diff generalizes.** `c_cluster_diff.py`
//!    runs on the real libpq functions and produces a reasonable
//!    report (one ident binding + one structural hole).
//! 2. **Engine handles non-SHA token-tree shapes.** Loops, ifs,
//!    pointer arithmetic, ABI annotations — all round-trip cleanly.
//! 3. **Overrides cover the gap when tensor diff hits a structural
//!    hole.** Field renames that the diff can't see (because they
//!    live inside a divergent subtree) flow in via `ident_overrides`.
//!
//! See the project's end-of-task report for the full pipeline grade.

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(clippy::missing_safety_doc)]

// =============================================================================
// PGconn model.
// =============================================================================

/// A minimal `#[repr(C)]` model of libpq's `PGconn` covering only the
/// fields touched by `pqCheckOutBufferSpace` / `pqCheckInBufferSpace`.
///
/// We deliberately omit the rest of the ~50-field struct. The crate
/// is exercised by Rust-side unit tests, not by linking against the
/// real libpq, so layout-compat with the full C `PGconn` isn't
/// required. A future hardening pass could mirror the full layout.
#[repr(C)]
pub struct PgConn {
    pub inBuffer: *mut u8,
    pub inBufSize: i32,
    pub inStart: i32,
    pub inCursor: i32,
    pub inEnd: i32,

    pub outBuffer: *mut u8,
    pub outBufSize: i32,
}

impl PgConn {
    /// Construct a PgConn with both buffers allocated to `initial_size`
    /// via `libc::malloc`. Caller owns the returned struct and is
    /// responsible for freeing the buffers via `drop_pg_conn`.
    pub fn new_for_test(initial_size: i32) -> Self {
        unsafe {
            let inbuf = if initial_size > 0 {
                libc::malloc(initial_size as usize) as *mut u8
            } else {
                core::ptr::null_mut()
            };
            let outbuf = if initial_size > 0 {
                libc::malloc(initial_size as usize) as *mut u8
            } else {
                core::ptr::null_mut()
            };
            PgConn {
                inBuffer: inbuf,
                inBufSize: initial_size,
                inStart: 0,
                inCursor: 0,
                inEnd: 0,
                outBuffer: outbuf,
                outBufSize: initial_size,
            }
        }
    }

    /// Free the buffers allocated by `new_for_test`.
    pub unsafe fn drop_buffers(&mut self) {
        if !self.inBuffer.is_null() {
            libc::free(self.inBuffer as *mut libc::c_void);
            self.inBuffer = core::ptr::null_mut();
        }
        if !self.outBuffer.is_null() {
            libc::free(self.outBuffer as *mut libc::c_void);
            self.outBuffer = core::ptr::null_mut();
        }
    }
}

// =============================================================================
// Inner grow-loop helpers — the cluster's actual translation surface.
//
// These are kept byte-equivalent to `canonicals/pqCheckOutBufferSpace.rs`
// and `canonicals/pqCheckInBufferSpace.rs`. The translator engine
// emits `try_grow_in_buffer` from `try_grow_out_buffer` via the
// bindings in `translation/in_from_out.overrides.toml`.
// =============================================================================

/// Try to grow `conn->outBuffer` to hold `bytes_needed` bytes. Mirrors
/// the doubling + 8K-step strategy from fe-misc.c. Returns 0 on success,
/// -1 on alloc failure.
#[no_mangle]
pub unsafe extern "C" fn try_grow_out_buffer(bytes_needed: usize, conn: *mut PgConn) -> i32 {
    let conn = &mut *conn;
    let mut newsize: i32 = conn.outBufSize;
    loop {
        newsize = newsize.wrapping_mul(2);
        if !(newsize > 0 && bytes_needed > newsize as usize) {
            break;
        }
    }
    if newsize > 0 && bytes_needed <= newsize as usize {
        let newbuf = libc::realloc(conn.outBuffer as *mut libc::c_void, newsize as usize) as *mut u8;
        if !newbuf.is_null() {
            conn.outBuffer = newbuf;
            conn.outBufSize = newsize;
            return 0;
        }
    }
    newsize = conn.outBufSize;
    loop {
        newsize = newsize.wrapping_add(8192);
        if !(newsize > 0 && bytes_needed > newsize as usize) {
            break;
        }
    }
    if newsize > 0 && bytes_needed <= newsize as usize {
        let newbuf = libc::realloc(conn.outBuffer as *mut libc::c_void, newsize as usize) as *mut u8;
        if !newbuf.is_null() {
            conn.outBuffer = newbuf;
            conn.outBufSize = newsize;
            return 0;
        }
    }
    -1
}

/// Try to grow `conn->inBuffer` to hold `bytes_needed` bytes. Sibling
/// of `try_grow_out_buffer` — the engine can synthesize this from the
/// canonical by ident-substitution.
#[no_mangle]
pub unsafe extern "C" fn try_grow_in_buffer(bytes_needed: usize, conn: *mut PgConn) -> i32 {
    let conn = &mut *conn;
    let mut newsize: i32 = conn.inBufSize;
    loop {
        newsize = newsize.wrapping_mul(2);
        if !(newsize > 0 && bytes_needed > newsize as usize) {
            break;
        }
    }
    if newsize > 0 && bytes_needed <= newsize as usize {
        let newbuf = libc::realloc(conn.inBuffer as *mut libc::c_void, newsize as usize) as *mut u8;
        if !newbuf.is_null() {
            conn.inBuffer = newbuf;
            conn.inBufSize = newsize;
            return 0;
        }
    }
    newsize = conn.inBufSize;
    loop {
        newsize = newsize.wrapping_add(8192);
        if !(newsize > 0 && bytes_needed > newsize as usize) {
            break;
        }
    }
    if newsize > 0 && bytes_needed <= newsize as usize {
        let newbuf = libc::realloc(conn.inBuffer as *mut libc::c_void, newsize as usize) as *mut u8;
        if !newbuf.is_null() {
            conn.inBuffer = newbuf;
            conn.inBufSize = newsize;
            return 0;
        }
    }
    -1
}

// =============================================================================
// pqCheckOutBufferSpace — fe-misc.c:287-342.
//
// Faithful 1:1 translation. The C function is the early-exit +
// inner grow loop fused; we keep the early-exit inline and delegate
// the grow loop to `try_grow_out_buffer` (the cluster canonical).
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn pqCheckOutBufferSpace(bytes_needed: usize, conn: *mut PgConn) -> i32 {
    if conn.is_null() {
        return -1;
    }
    let cur = (*conn).outBufSize;
    if bytes_needed <= cur as usize {
        return 0;
    }
    try_grow_out_buffer(bytes_needed, conn)
}

// =============================================================================
// pqCheckInBufferSpace — fe-misc.c:351-441.
//
// Distinct shape: the In side has a left-justify prelude (memmove
// inBuffer[inStart..inEnd] → inBuffer[0..inEnd-inStart], adjust
// inStart/inCursor/inEnd) before the grow loop, plus an inStart
// subtraction from bytes_needed. The cluster diff at the AST level
// reports this as one structural hole; we hand-code it here.
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn pqCheckInBufferSpace(mut bytes_needed: usize, conn: *mut PgConn) -> i32 {
    if conn.is_null() {
        return -1;
    }
    let conn_ref = &mut *conn;

    // Quick exit if we have enough space.
    if bytes_needed <= conn_ref.inBufSize as usize {
        return 0;
    }

    // Left-justify: the caller's bytes_needed counts data to the left
    // of inStart, which we can delete by sliding the buffer.
    bytes_needed = bytes_needed.saturating_sub(conn_ref.inStart as usize);

    if conn_ref.inStart < conn_ref.inEnd {
        if conn_ref.inStart > 0 {
            let len = (conn_ref.inEnd - conn_ref.inStart) as usize;
            libc::memmove(
                conn_ref.inBuffer as *mut libc::c_void,
                conn_ref.inBuffer.add(conn_ref.inStart as usize) as *const libc::c_void,
                len,
            );
            conn_ref.inEnd -= conn_ref.inStart;
            conn_ref.inCursor -= conn_ref.inStart;
            conn_ref.inStart = 0;
        }
    } else {
        // Buffer is logically empty, reset it.
        conn_ref.inStart = 0;
        conn_ref.inCursor = 0;
        conn_ref.inEnd = 0;
    }

    // Recheck after left-justify.
    if bytes_needed <= conn_ref.inBufSize as usize {
        return 0;
    }

    try_grow_in_buffer(bytes_needed, conn)
}
