#[no_mangle]
pub unsafe extern "C" fn pg_sha256_init(context: *mut Sha256Ctx) {
    if context.is_null() {
        return;
    }
    let ctx = &mut *context;
    ctx.state = SHA256_INITIAL_HASH_VALUE;
    ctx.buffer.fill(0);
    ctx.bitcount = 0;
}
