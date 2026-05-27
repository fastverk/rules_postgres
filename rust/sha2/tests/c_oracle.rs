//! Rust FFI wrapper around the C-side oracle compiled by build.rs.
//!
//! Provides `shaXXX_via_c(input: &[u8]) -> [u8; N]` helpers for all
//! four SHA-2 variants (224, 256, 384, 512) that call Postgres' actual
//! `pg_shaXXX_*` functions (vendored at c_oracle/sha2.c, unmodified
//! from PG 17.6) via `extern "C"`.
//!
//! This module is only used by the diff-tests (`diff_shaXXX.rs`); it
//! lives in `tests/` to keep the FFI surface out of the crate's main
//! API (pg_sha2's public Rust API is its own implementation, not a
//! wrapper around the C one).
//!
//! C-side ABI notes (see c_oracle/include/sha2_int.h):
//!   - pg_sha256_ctx  = { uint32 state[8]; uint64 bitcount; uint8 buffer[64]; }
//!   - pg_sha512_ctx  = { uint64 state[8]; uint64 bitcount[2]; uint8 buffer[128]; }
//!   - pg_sha224_ctx is a typedef alias for pg_sha256_ctx
//!   - pg_sha384_ctx is a typedef alias for pg_sha512_ctx

#![allow(non_camel_case_types)]
#![allow(dead_code)]

use std::mem::MaybeUninit;

// -----------------------------------------------------------------------------
// SHA-224 / SHA-256: share pg_sha256_ctx.
// -----------------------------------------------------------------------------

/// Bit-compatible with the C-side `pg_sha256_ctx`:
///   struct { uint32 state[8]; uint64 bitcount; uint8 buffer[64]; }
///
/// Matches `Sha256Ctx` from the candidate (pg_sha2's hand-translated
/// Rust struct). The layout MUST be identical or the FFI breaks.
#[repr(C)]
pub struct CSha256Ctx {
    pub state: [u32; 8],
    pub bitcount: u64,
    pub buffer: [u8; 64],
}

/// pg_sha224_ctx is a C typedef alias for pg_sha256_ctx; same layout.
pub type CSha224Ctx = CSha256Ctx;

// `c_pg_*` symbols are wrappers around the real `pg_*` symbols (see
// `c_oracle/wrappers.c`). The rename exists ONLY for the diff-test
// link path: without it, the test linker silently picks between the
// Rust crate's `pg_sha*_init` (no_mangle export) and the C oracle's,
// resolving both sides to one impl. Bug caught 2026-05-26 during the
// llm_translate live validation.
extern "C" {
    fn c_pg_sha224_init(ctx: *mut CSha224Ctx);
    fn c_pg_sha224_update(ctx: *mut CSha224Ctx, input: *const u8, len: usize);
    fn c_pg_sha224_final(ctx: *mut CSha224Ctx, digest: *mut u8);

    fn c_pg_sha256_init(ctx: *mut CSha256Ctx);
    fn c_pg_sha256_update(ctx: *mut CSha256Ctx, input: *const u8, len: usize);
    fn c_pg_sha256_final(ctx: *mut CSha256Ctx, digest: *mut u8);
}

/// Compute SHA-224 via Postgres' actual C implementation.
pub fn sha224_via_c(input: &[u8]) -> [u8; 28] {
    let mut ctx: MaybeUninit<CSha224Ctx> = MaybeUninit::uninit();
    let mut digest = [0u8; 28];
    unsafe {
        c_pg_sha224_init(ctx.as_mut_ptr());
        c_pg_sha224_update(ctx.as_mut_ptr(), input.as_ptr(), input.len());
        c_pg_sha224_final(ctx.as_mut_ptr(), digest.as_mut_ptr());
    }
    digest
}

/// Compute SHA-256 via Postgres' actual C implementation.
pub fn sha256_via_c(input: &[u8]) -> [u8; 32] {
    let mut ctx: MaybeUninit<CSha256Ctx> = MaybeUninit::uninit();
    let mut digest = [0u8; 32];
    unsafe {
        c_pg_sha256_init(ctx.as_mut_ptr());
        c_pg_sha256_update(ctx.as_mut_ptr(), input.as_ptr(), input.len());
        c_pg_sha256_final(ctx.as_mut_ptr(), digest.as_mut_ptr());
    }
    digest
}

// -----------------------------------------------------------------------------
// SHA-384 / SHA-512: share pg_sha512_ctx.
// -----------------------------------------------------------------------------

/// Bit-compatible with the C-side `pg_sha512_ctx`:
///   struct { uint64 state[8]; uint64 bitcount[2]; uint8 buffer[128]; }
///
/// Layout MUST match exactly or the FFI breaks.
#[repr(C)]
pub struct CSha512Ctx {
    pub state: [u64; 8],
    pub bitcount: [u64; 2],
    pub buffer: [u8; 128],
}

/// pg_sha384_ctx is a C typedef alias for pg_sha512_ctx; same layout.
pub type CSha384Ctx = CSha512Ctx;

// `c_pg_*` rename — see SHA-256 block above for the full rationale.
extern "C" {
    fn c_pg_sha384_init(ctx: *mut CSha384Ctx);
    fn c_pg_sha384_update(ctx: *mut CSha384Ctx, input: *const u8, len: usize);
    fn c_pg_sha384_final(ctx: *mut CSha384Ctx, digest: *mut u8);

    fn c_pg_sha512_init(ctx: *mut CSha512Ctx);
    fn c_pg_sha512_update(ctx: *mut CSha512Ctx, input: *const u8, len: usize);
    fn c_pg_sha512_final(ctx: *mut CSha512Ctx, digest: *mut u8);
}

/// Compute SHA-384 via Postgres' actual C implementation.
pub fn sha384_via_c(input: &[u8]) -> [u8; 48] {
    let mut ctx: MaybeUninit<CSha384Ctx> = MaybeUninit::uninit();
    let mut digest = [0u8; 48];
    unsafe {
        c_pg_sha384_init(ctx.as_mut_ptr());
        c_pg_sha384_update(ctx.as_mut_ptr(), input.as_ptr(), input.len());
        c_pg_sha384_final(ctx.as_mut_ptr(), digest.as_mut_ptr());
    }
    digest
}

/// Compute SHA-512 via Postgres' actual C implementation.
pub fn sha512_via_c(input: &[u8]) -> [u8; 64] {
    let mut ctx: MaybeUninit<CSha512Ctx> = MaybeUninit::uninit();
    let mut digest = [0u8; 64];
    unsafe {
        c_pg_sha512_init(ctx.as_mut_ptr());
        c_pg_sha512_update(ctx.as_mut_ptr(), input.as_ptr(), input.len());
        c_pg_sha512_final(ctx.as_mut_ptr(), digest.as_mut_ptr());
    }
    digest
}
