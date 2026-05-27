#[no_mangle]
pub unsafe extern "C" fn pg_sha384_init(context: *mut Sha512Ctx) {
    if context.is_null() {
        return;
    }
    let ctx = &mut *context;
    ctx.state = SHA384_INITIAL_HASH_VALUE;
    ctx.buffer.fill(0);
    ctx.bitcount = [0, 0];
}
