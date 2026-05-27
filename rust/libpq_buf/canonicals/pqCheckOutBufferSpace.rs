// Cluster canonical: inner buffer-grow loop for the *output* side.
// The translator engine emits `try_grow_in_buffer` (the sibling) from
// this by ident-substitution of the field names (outBuffer→inBuffer,
// outBufSize→inBufSize) and the function name itself. Everything else
// is shape-identical between the two cluster members.

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
