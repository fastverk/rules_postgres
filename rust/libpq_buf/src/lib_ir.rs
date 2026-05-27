//! Self-contained single-file Rust source for the LLVM IR
//! equivalence gate. Equivalent to `src/lib.rs` but without external
//! crate dependencies (no `libc` — we declare the three C functions
//! we need as `extern "C"` instead). The `rust_llvm_ir_single` Bazel
//! rule invokes host rustc with no `--extern` flags, so the IR target
//! has to be dep-free.
//!
//! This mirrors how `translated/sha2/src/lib.rs` is also single-file
//! (and `no_std`-friendly). We keep the public symbols `#[no_mangle]
//! extern "C"` so the LLVM IR exports match libpq's expected names.

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(clippy::missing_safety_doc)]

// Free-standing extern decls for the C runtime fns we call. The IR
// will reference `@realloc`, `@memmove` — matching what the C side
// emits for `pqCheckOutBufferSpace` / `pqCheckInBufferSpace`. Symbol
// equivalence at the LLVM IR level is the gate; no Rust-side libc
// crate needed.
extern "C" {
    fn realloc(ptr: *mut u8, size: usize) -> *mut u8;
    fn memmove(dst: *mut u8, src: *const u8, n: usize) -> *mut u8;
}

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
        let newbuf = realloc(conn.outBuffer, newsize as usize);
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
        let newbuf = realloc(conn.outBuffer, newsize as usize);
        if !newbuf.is_null() {
            conn.outBuffer = newbuf;
            conn.outBufSize = newsize;
            return 0;
        }
    }
    -1
}

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
        let newbuf = realloc(conn.inBuffer, newsize as usize);
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
        let newbuf = realloc(conn.inBuffer, newsize as usize);
        if !newbuf.is_null() {
            conn.inBuffer = newbuf;
            conn.inBufSize = newsize;
            return 0;
        }
    }
    -1
}

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

#[no_mangle]
pub unsafe extern "C" fn pqCheckInBufferSpace(mut bytes_needed: usize, conn: *mut PgConn) -> i32 {
    if conn.is_null() {
        return -1;
    }
    let conn_ref = &mut *conn;

    if bytes_needed <= conn_ref.inBufSize as usize {
        return 0;
    }

    bytes_needed = bytes_needed.saturating_sub(conn_ref.inStart as usize);

    if conn_ref.inStart < conn_ref.inEnd {
        if conn_ref.inStart > 0 {
            let len = (conn_ref.inEnd - conn_ref.inStart) as usize;
            memmove(
                conn_ref.inBuffer,
                conn_ref.inBuffer.add(conn_ref.inStart as usize),
                len,
            );
            conn_ref.inEnd -= conn_ref.inStart;
            conn_ref.inCursor -= conn_ref.inStart;
            conn_ref.inStart = 0;
        }
    } else {
        conn_ref.inStart = 0;
        conn_ref.inCursor = 0;
        conn_ref.inEnd = 0;
    }

    if bytes_needed <= conn_ref.inBufSize as usize {
        return 0;
    }

    try_grow_in_buffer(bytes_needed, conn)
}
