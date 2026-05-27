// Cluster sibling: inner buffer-grow loop for the *input* side.
// Hand-written counterpart to `try_grow_out_buffer`. The translator's
// AST diff against the canonical surfaces three holes: function name,
// outBuffer↔inBuffer (× 4 occurrences, deduped), outBufSize↔inBufSize
// (× 4 occurrences, deduped).

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
