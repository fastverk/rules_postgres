//! Behavioral smoke tests for the libpq buffer-grow translation.
//!
//! These mirror the canonical realloc-and-double logic that real
//! libpq exercises: a connection starts with some buffer size; data
//! comes in beyond that size; the grow routine should double until
//! big enough.
//!
//! NOT a substitute for differential testing against the real libpq C
//! source — we don't link against libpq here. The structural-IR
//! equivalence gate (`//examples/postgres_smoke:libpq_buf_translation_correct`)
//! is the cross-check that the Rust IR matches the C IR.

use libpq_buf::{pqCheckInBufferSpace, pqCheckOutBufferSpace, PgConn};

#[test]
fn out_buffer_quick_exit_when_already_large_enough() {
    let mut conn = PgConn::new_for_test(1024);
    unsafe {
        // bytes_needed <= current buffer size → no realloc, return 0.
        let rc = pqCheckOutBufferSpace(100, &mut conn);
        assert_eq!(rc, 0);
        assert_eq!(conn.outBufSize, 1024, "no resize expected");
        conn.drop_buffers();
    }
}

#[test]
fn out_buffer_doubles_when_outgrown() {
    let mut conn = PgConn::new_for_test(256);
    unsafe {
        // Need 300 bytes, current is 256 → doubles to 512.
        let rc = pqCheckOutBufferSpace(300, &mut conn);
        assert_eq!(rc, 0);
        assert_eq!(conn.outBufSize, 512);
        conn.drop_buffers();
    }
}

#[test]
fn out_buffer_doubles_multiple_times() {
    let mut conn = PgConn::new_for_test(256);
    unsafe {
        // Need ~5000 bytes → 256→512→1024→2048→4096→8192.
        let rc = pqCheckOutBufferSpace(5000, &mut conn);
        assert_eq!(rc, 0);
        assert!(conn.outBufSize >= 5000, "got {}", conn.outBufSize);
        // Specifically: 256 doubled 5 times = 8192.
        assert_eq!(conn.outBufSize, 8192);
        conn.drop_buffers();
    }
}

#[test]
fn in_buffer_quick_exit_when_already_large_enough() {
    let mut conn = PgConn::new_for_test(1024);
    unsafe {
        let rc = pqCheckInBufferSpace(100, &mut conn);
        assert_eq!(rc, 0);
        assert_eq!(conn.inBufSize, 1024);
        conn.drop_buffers();
    }
}

#[test]
fn in_buffer_left_justify_frees_space_without_realloc() {
    // Buffer size 1024 with 800 bytes already consumed (inStart=800,
    // inEnd=1000). After left-justify, 200 bytes of live data sit at
    // buffer[0..200], freeing 824 bytes at the tail.
    let mut conn = PgConn::new_for_test(1024);
    conn.inStart = 800;
    conn.inCursor = 900;
    conn.inEnd = 1000;
    unsafe {
        // bytes_needed=1000 > inBufSize=1024 is false → quick exit
        // path triggers. We need bytes_needed > inBufSize to hit the
        // left-justify path. Set bytes_needed=1100.
        let rc = pqCheckInBufferSpace(1100, &mut conn);
        // After left-justify: bytes_needed becomes 1100-800=300,
        // which fits in 1024 → no realloc.
        assert_eq!(rc, 0);
        assert_eq!(conn.inBufSize, 1024, "left-justify should avoid realloc");
        assert_eq!(conn.inStart, 0);
        assert_eq!(conn.inEnd, 200);
        assert_eq!(conn.inCursor, 100);
        conn.drop_buffers();
    }
}

#[test]
fn in_buffer_doubles_when_left_justify_insufficient() {
    // Tiny buffer with no headroom — left-justify yields nothing,
    // grow kicks in.
    let mut conn = PgConn::new_for_test(256);
    unsafe {
        let rc = pqCheckInBufferSpace(500, &mut conn);
        assert_eq!(rc, 0);
        assert!(conn.inBufSize >= 500, "got {}", conn.inBufSize);
        conn.drop_buffers();
    }
}

#[test]
fn null_conn_returns_error() {
    unsafe {
        assert_eq!(pqCheckOutBufferSpace(100, core::ptr::null_mut()), -1);
        assert_eq!(pqCheckInBufferSpace(100, core::ptr::null_mut()), -1);
    }
}
