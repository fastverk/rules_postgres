//! Hand-translation of Postgres' `src/common/sha2.c` → Rust.
//!
//! First milestone of the cluster-driven C→Rust deterministic
//! translation spike. Methodology:
//!
//! 1. Take a tight semantic cluster (here: SHA-256, with SHA-224 /
//!    SHA-384 / SHA-512 as the family — clustered together in v4's
//!    stmt-level embeddings at d≈0.05).
//! 2. Hand-translate one canonical function set (SHA-256) to Rust.
//! 3. Verify behavioral equivalence against FIPS 180-4 test vectors
//!    + the (original) PG C implementation.
//! 4. Apply the cluster's transform rule to SHA-224 / SHA-384 / SHA-512
//!    by mechanical substitution (different constants + block widths,
//!    same shape).
//! 5. Compare LLVM IR: rustc emission against the
//!    `bazel-bin/examples/postgres_smoke/sha2_ll.llvm/.../sha2.c.ll`
//!    we already have on disk.
//!
//! Arithmetic policy: every `+` in the C source becomes
//! `wrapping_add` here. SHA-256 mod-2^32 arithmetic is the entire
//! point of the algorithm; we're not optimizing the math, we're
//! honoring the cryptographic specification. C's signed-overflow UB
//! is irrelevant — all SHA arithmetic is on `u32`/`u64`.
//!
//! Layout: `Sha256Ctx` is bit-compatible with `pg_sha256_ctx` (see
//! `src/include/common/sha2_int.h`): `[u32; 8] state`, `u64 bitcount`,
//! `[u8; 64] buffer`. Total 104 bytes.
//!
//! ABI: `#[no_mangle] extern "C"` on the public entry points so the
//! Rust .o can drop-in for the C .o at link time. Symbol names match
//! Postgres exactly (`pg_sha256_init`, `pg_sha256_update`,
//! `pg_sha256_final`).

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(clippy::needless_range_loop)]
// (`no_std` would be cleaner but requires a panic_handler for the
// staticlib/cdylib crate types — pure rlib + std is fine for the
// spike; SHA-256 itself doesn't touch any std-only APIs.)

// =============================================================================
// Constants — bit-identical to sha2.c lines 165-206.
// =============================================================================

const PG_SHA256_BLOCK_LENGTH: usize = 64;
const PG_SHA256_DIGEST_LENGTH: usize = 32;
const PG_SHA256_SHORT_BLOCK_LENGTH: usize = PG_SHA256_BLOCK_LENGTH - 8; // 56

/// Hash constant words K for SHA-256 (sha2.c:165).
const K256: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5,
    0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
    0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
    0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
    0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
    0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
    0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
    0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
    0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// Initial hash value H for SHA-256 (sha2.c:197).
const SHA256_INITIAL_HASH_VALUE: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
    0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

// =============================================================================
// Compression-function helpers (sha2.c:138-146).
//
// `Ch`, `Maj`, `Sigma0_256`, `Sigma1_256`, `sigma0_256`, `sigma1_256`
// translate 1:1 from the C macros. Bitwise ops are identical in C and
// Rust on unsigned types.
// =============================================================================

#[inline(always)]
fn Ch(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (!x & z)
}

#[inline(always)]
fn Maj(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (x & z) ^ (y & z)
}

// S32(b, x) in C = right-rotate x by b bits. Rust has it as an
// intrinsic.
#[inline(always)]
fn Sigma0_256(x: u32) -> u32 {
    x.rotate_right(2) ^ x.rotate_right(13) ^ x.rotate_right(22)
}

#[inline(always)]
fn Sigma1_256(x: u32) -> u32 {
    x.rotate_right(6) ^ x.rotate_right(11) ^ x.rotate_right(25)
}

#[inline(always)]
fn sigma0_256(x: u32) -> u32 {
    x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3)
}

#[inline(always)]
fn sigma1_256(x: u32) -> u32 {
    x.rotate_right(17) ^ x.rotate_right(19) ^ (x >> 10)
}

// =============================================================================
// Context struct — bit-compatible with pg_sha256_ctx.
//
// C declaration:
//   typedef struct pg_sha256_ctx {
//       uint32 state[8];
//       uint64 bitcount;
//       uint8  buffer[PG_SHA256_BLOCK_LENGTH];
//   } pg_sha256_ctx;
//
// On most 64-bit ABIs the layout is { 32 bytes state, 8 bytes
// bitcount, 64 bytes buffer } = 104 bytes total, naturally aligned to
// 8. We mark `repr(C)` so Rust uses the same field order + padding
// rules, making the struct ABI-compatible with the C version.
// =============================================================================

#[repr(C)]
pub struct Sha256Ctx {
    pub state: [u32; 8],
    pub bitcount: u64,
    pub buffer: [u8; PG_SHA256_BLOCK_LENGTH],
}

impl Default for Sha256Ctx {
    fn default() -> Self {
        Self {
            state: [0; 8],
            bitcount: 0,
            buffer: [0; PG_SHA256_BLOCK_LENGTH],
        }
    }
}

// =============================================================================
// pg_sha256_init — sha2.c:278-286.
//
// Rust translation: memcpy → array-assignment; memset → fill(0);
// bitcount = 0.
// =============================================================================

/// # Safety
/// `context` must be a valid, properly aligned pointer to a
/// `Sha256Ctx`. Null is tolerated (matches C behavior).
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

// =============================================================================
// SHA256_Transform — sha2.c:385-472 (non-unrolled version).
//
// This is the core compression function. Inner loop runs 64 rounds;
// the first 16 read 4 bytes at a time from `data` (big-endian u32),
// the remaining 48 expand W via sigma0_256/sigma1_256.
//
// Translation policy: every `+` here is `wrapping_add`. SHA arithmetic
// is mod-2^32; overflow is the algorithm.
// =============================================================================

fn sha256_transform(ctx: &mut Sha256Ctx, data: &[u8; PG_SHA256_BLOCK_LENGTH]) {
    let mut W: [u32; 16] = [0; 16];

    let mut a = ctx.state[0];
    let mut b = ctx.state[1];
    let mut c = ctx.state[2];
    let mut d = ctx.state[3];
    let mut e = ctx.state[4];
    let mut f = ctx.state[5];
    let mut g = ctx.state[6];
    let mut h = ctx.state[7];

    // Rounds 0..16 — load big-endian u32 from `data`.
    for j in 0..16 {
        let base = j * 4;
        let w = u32::from_be_bytes(data[base..base + 4].try_into().unwrap());
        W[j] = w;
        let t1 = h
            .wrapping_add(Sigma1_256(e))
            .wrapping_add(Ch(e, f, g))
            .wrapping_add(K256[j])
            .wrapping_add(W[j]);
        let t2 = Sigma0_256(a).wrapping_add(Maj(a, b, c));
        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(t1);
        d = c;
        c = b;
        b = a;
        a = t1.wrapping_add(t2);
    }

    // Rounds 16..64 — message-schedule expansion.
    for j in 16..64 {
        let s0 = sigma0_256(W[(j + 1) & 0x0f]);
        let s1 = sigma1_256(W[(j + 14) & 0x0f]);
        W[j & 0x0f] = W[j & 0x0f]
            .wrapping_add(s1)
            .wrapping_add(W[(j + 9) & 0x0f])
            .wrapping_add(s0);
        let t1 = h
            .wrapping_add(Sigma1_256(e))
            .wrapping_add(Ch(e, f, g))
            .wrapping_add(K256[j])
            .wrapping_add(W[j & 0x0f]);
        let t2 = Sigma0_256(a).wrapping_add(Maj(a, b, c));
        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(t1);
        d = c;
        c = b;
        b = a;
        a = t1.wrapping_add(t2);
    }

    ctx.state[0] = ctx.state[0].wrapping_add(a);
    ctx.state[1] = ctx.state[1].wrapping_add(b);
    ctx.state[2] = ctx.state[2].wrapping_add(c);
    ctx.state[3] = ctx.state[3].wrapping_add(d);
    ctx.state[4] = ctx.state[4].wrapping_add(e);
    ctx.state[5] = ctx.state[5].wrapping_add(f);
    ctx.state[6] = ctx.state[6].wrapping_add(g);
    ctx.state[7] = ctx.state[7].wrapping_add(h);
    // C version explicitly zeros the locals here for security. Rust
    // doesn't guarantee that — the optimizer may eliminate the
    // zeroing as dead stores. Use `zeroize` crate in production; for
    // the spike we accept that limitation.
}

// =============================================================================
// pg_sha256_update — sha2.c:475-526.
//
// Buffers incoming data into the context's 64-byte buffer; when the
// buffer is full, invokes SHA256_Transform on it. After the buffer
// is drained, processes as many complete 64-byte blocks as possible
// directly from `data`. Leftover bytes go back into the buffer for
// the next call.
// =============================================================================

/// # Safety
/// `context` must be a valid pointer; `data` must point to at least
/// `len` readable bytes.
#[no_mangle]
pub unsafe extern "C" fn pg_sha256_update(
    context: *mut Sha256Ctx,
    data: *const u8,
    len: usize,
) {
    if len == 0 || context.is_null() {
        return;
    }
    let ctx = &mut *context;
    let mut data_slice = core::slice::from_raw_parts(data, len);

    let usedspace = ((ctx.bitcount >> 3) % PG_SHA256_BLOCK_LENGTH as u64) as usize;
    if usedspace > 0 {
        let freespace = PG_SHA256_BLOCK_LENGTH - usedspace;
        if data_slice.len() >= freespace {
            ctx.buffer[usedspace..].copy_from_slice(&data_slice[..freespace]);
            ctx.bitcount = ctx.bitcount.wrapping_add((freespace as u64) << 3);
            data_slice = &data_slice[freespace..];
            let buf = ctx.buffer;
            sha256_transform(ctx, &buf);
        } else {
            ctx.buffer[usedspace..usedspace + data_slice.len()].copy_from_slice(data_slice);
            ctx.bitcount = ctx.bitcount.wrapping_add((data_slice.len() as u64) << 3);
            return;
        }
    }

    while data_slice.len() >= PG_SHA256_BLOCK_LENGTH {
        let block: [u8; PG_SHA256_BLOCK_LENGTH] =
            data_slice[..PG_SHA256_BLOCK_LENGTH].try_into().unwrap();
        sha256_transform(ctx, &block);
        ctx.bitcount = ctx.bitcount.wrapping_add((PG_SHA256_BLOCK_LENGTH as u64) << 3);
        data_slice = &data_slice[PG_SHA256_BLOCK_LENGTH..];
    }

    if !data_slice.is_empty() {
        ctx.buffer[..data_slice.len()].copy_from_slice(data_slice);
        ctx.bitcount = ctx.bitcount.wrapping_add((data_slice.len() as u64) << 3);
    }
}

// =============================================================================
// SHA256_Last — sha2.c:528-574.
//
// Final padding step: append 1 bit (0x80), zero-pad to the
// short-block length (56 bytes), then 8-byte big-endian bitcount, then
// run one or two final Transforms.
// =============================================================================

fn sha256_last(ctx: &mut Sha256Ctx) {
    let usedspace = ((ctx.bitcount >> 3) % PG_SHA256_BLOCK_LENGTH as u64) as usize;
    // C swaps bitcount FROM host order via REVERSE64 (little→big).
    // u64::to_be() is the idiomatic Rust equivalent. We then write it
    // as raw bytes at buffer[56..64].
    let bitcount_be = ctx.bitcount.to_be();

    if usedspace > 0 {
        ctx.buffer[usedspace] = 0x80;
        let mut next = usedspace + 1;

        if next <= PG_SHA256_SHORT_BLOCK_LENGTH {
            for byte in &mut ctx.buffer[next..PG_SHA256_SHORT_BLOCK_LENGTH] {
                *byte = 0;
            }
        } else {
            if next < PG_SHA256_BLOCK_LENGTH {
                for byte in &mut ctx.buffer[next..PG_SHA256_BLOCK_LENGTH] {
                    *byte = 0;
                }
            }
            // Do second-to-last transform.
            let buf = ctx.buffer;
            sha256_transform(ctx, &buf);

            // Reset for the last transform.
            for byte in &mut ctx.buffer[..PG_SHA256_SHORT_BLOCK_LENGTH] {
                *byte = 0;
            }
            next = 0;
        }
        let _ = next;
    } else {
        for byte in &mut ctx.buffer[..PG_SHA256_SHORT_BLOCK_LENGTH] {
            *byte = 0;
        }
        ctx.buffer[0] = 0x80;
    }

    // Write the big-endian bitcount into bytes 56..64.
    ctx.buffer[PG_SHA256_SHORT_BLOCK_LENGTH..PG_SHA256_BLOCK_LENGTH]
        .copy_from_slice(&bitcount_be.to_le_bytes());
    // ^ note: bitcount_be is already byte-swapped to big-endian via
    //   `to_be()`, so writing its little-endian byte representation
    //   yields the original big-endian bytes — matches the C code
    //   which writes the post-REVERSE64 u64 via direct memory store.

    let buf = ctx.buffer;
    sha256_transform(ctx, &buf);
}

// =============================================================================
// pg_sha256_final — sha2.c:576-600.
// =============================================================================

/// # Safety
/// `context` must be valid; `digest` must point to at least
/// `PG_SHA256_DIGEST_LENGTH` (32) writable bytes.
#[no_mangle]
pub unsafe extern "C" fn pg_sha256_final(context: *mut Sha256Ctx, digest: *mut u8) {
    if digest.is_null() || context.is_null() {
        return;
    }
    let ctx = &mut *context;
    sha256_last(ctx);

    // Convert state words to big-endian and write into digest.
    let out = core::slice::from_raw_parts_mut(digest, PG_SHA256_DIGEST_LENGTH);
    for (i, word) in ctx.state.iter().enumerate() {
        let be = word.to_be_bytes();
        out[i * 4..(i + 1) * 4].copy_from_slice(&be);
    }

    // Clean up state. Same as C's `memset(context, 0, sizeof(*context))`.
    // The optimizer may DCE this; see SHA256_Transform comment.
    ctx.state = [0; 8];
    ctx.bitcount = 0;
    ctx.buffer = [0; PG_SHA256_BLOCK_LENGTH];
}

// =============================================================================
// Safe Rust convenience wrapper (NOT in the C ABI surface; built atop
// the unsafe extern "C" entry points). For our own test harness.
// =============================================================================

/// Compute the SHA-256 of `input`. Returns a 32-byte digest.
pub fn sha256(input: &[u8]) -> [u8; PG_SHA256_DIGEST_LENGTH] {
    let mut ctx = Sha256Ctx::default();
    let mut digest = [0u8; PG_SHA256_DIGEST_LENGTH];
    unsafe {
        pg_sha256_init(&mut ctx);
        pg_sha256_update(&mut ctx, input.as_ptr(), input.len());
        pg_sha256_final(&mut ctx, digest.as_mut_ptr());
    }
    digest
}

// =============================================================================
// Cluster-template demonstration — sha224 / sha384 / sha512 init.
//
// The v4 embedder clustered `pg_sha256_init`, `pg_sha224_init`,
// `pg_sha384_init` together at cosine d≈0.05. This is the *novel*
// claim of cluster-driven translation: one written template + N
// mechanical applications.
//
// Translation rule extracted from `pg_sha256_init`:
//
//   pg_<HASH>_init(ctx) {
//       if ctx == NULL return;
//       memcpy(ctx->state, <HASH>_INITIAL_HASH_VALUE, <DIGEST_LENGTH>);
//       memset(ctx->buffer, 0, <BLOCK_LENGTH>);
//       ctx->bitcount = 0;
//   }
//
// SHA-224 reuses pg_sha256_ctx (same 8×u32 state, just different
// initial hash + a shorter output digest). SHA-384 uses pg_sha512_ctx
// (8×u64 state, 16-byte counter, 128-byte block). SHA-512 same ctx as
// SHA-384, different initial hash.
//
// What the template lift looks like in practice: rename the constant,
// rename the ctx type, rename the lengths. Everything else identical
// to pg_sha256_init.
// =============================================================================

const PG_SHA224_DIGEST_LENGTH: usize = 28;
const PG_SHA512_BLOCK_LENGTH: usize = 128;
const PG_SHA384_DIGEST_LENGTH: usize = 48;
const PG_SHA512_DIGEST_LENGTH: usize = 64;

const SHA224_INITIAL_HASH_VALUE: [u32; 8] = [
    0xc1059ed8, 0x367cd507, 0x3070dd17, 0xf70e5939,
    0xffc00b31, 0x68581511, 0x64f98fa7, 0xbefa4fa4,
];

const SHA384_INITIAL_HASH_VALUE: [u64; 8] = [
    0xcbbb9d5dc1059ed8, 0x629a292a367cd507,
    0x9159015a3070dd17, 0x152fecd8f70e5939,
    0x67332667ffc00b31, 0x8eb44a8768581511,
    0xdb0c2e0d64f98fa7, 0x47b5481dbefa4fa4,
];

const SHA512_INITIAL_HASH_VALUE: [u64; 8] = [
    0x6a09e667f3bcc908, 0xbb67ae8584caa73b,
    0x3c6ef372fe94f82b, 0xa54ff53a5f1d36f1,
    0x510e527fade682d1, 0x9b05688c2b3e6c1f,
    0x1f83d9abfb41bd6b, 0x5be0cd19137e2179,
];

/// pg_sha512_ctx — bit-compatible with the C struct:
///   typedef struct pg_sha512_ctx {
///       uint64 state[8];
///       uint64 bitcount[2];          // 128-bit message length
///       uint8  buffer[PG_SHA512_BLOCK_LENGTH];   // 128 bytes
///   } pg_sha512_ctx;
#[repr(C)]
pub struct Sha512Ctx {
    pub state: [u64; 8],
    pub bitcount: [u64; 2],
    pub buffer: [u8; PG_SHA512_BLOCK_LENGTH],
}

impl Default for Sha512Ctx {
    fn default() -> Self {
        Self {
            state: [0; 8],
            bitcount: [0; 2],
            buffer: [0; PG_SHA512_BLOCK_LENGTH],
        }
    }
}

// ─── Template instance 1: SHA-224 ────────────────────────────────────────────
//
// Diff from pg_sha256_init: HASH_VALUE constant + digest length only.
// Ctx type identical (SHA-224 truncates SHA-256's state on output).

/// # Safety
/// Same as `pg_sha256_init`.
#[no_mangle]
pub unsafe extern "C" fn pg_sha224_init(context: *mut Sha256Ctx) {
    if context.is_null() {
        return;
    }
    let ctx = &mut *context;
    ctx.state = SHA224_INITIAL_HASH_VALUE;
    ctx.buffer.fill(0);
    ctx.bitcount = 0;
}

// ─── Template instance 2: SHA-384 ────────────────────────────────────────────
//
// Diff from pg_sha256_init: Sha512Ctx type, 384's HASH_VALUE,
// PG_SHA512_BLOCK_LENGTH-sized buffer, bitcount is [u64; 2] not u64.

/// # Safety
/// Same as `pg_sha256_init`, but `context` must be a `Sha512Ctx`.
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

// ─── Template instance 3: SHA-512 ────────────────────────────────────────────
//
// Diff from pg_sha384_init: HASH_VALUE constant only.

/// # Safety
/// Same as `pg_sha256_init`, but `context` must be a `Sha512Ctx`.
#[no_mangle]
pub unsafe extern "C" fn pg_sha512_init(context: *mut Sha512Ctx) {
    if context.is_null() {
        return;
    }
    let ctx = &mut *context;
    ctx.state = SHA512_INITIAL_HASH_VALUE;
    ctx.buffer.fill(0);
    ctx.bitcount = [0, 0];
}

// =============================================================================
// SHA-512 full path — closes the SHA family.
//
// SHA-512 is structurally identical to SHA-256 (init/transform/update/
// last/final shape) but parameterized over u64 instead of u32:
//   - 80 rounds instead of 64
//   - 128-byte blocks instead of 64
//   - 128-bit bitcount (two u64s) instead of one u64
//   - Different Sigma/sigma rotation amounts
//   - Different K table (K512, 80 entries)
//
// The point of writing it out by hand is to give the AST-rewrite engine
// the *paired* canonical: pg_sha256_update ↔ pg_sha512_update sit in the
// same v4 cluster but with the u32↔u64 + 64↔80 + 8↔16 substitutions, so
// the engine can learn the lift from this single example.
// =============================================================================

const PG_SHA512_SHORT_BLOCK_LENGTH: usize = PG_SHA512_BLOCK_LENGTH - 16; // 112

const K512: [u64; 80] = [
    0x428a2f98d728ae22, 0x7137449123ef65cd, 0xb5c0fbcfec4d3b2f, 0xe9b5dba58189dbbc,
    0x3956c25bf348b538, 0x59f111f1b605d019, 0x923f82a4af194f9b, 0xab1c5ed5da6d8118,
    0xd807aa98a3030242, 0x12835b0145706fbe, 0x243185be4ee4b28c, 0x550c7dc3d5ffb4e2,
    0x72be5d74f27b896f, 0x80deb1fe3b1696b1, 0x9bdc06a725c71235, 0xc19bf174cf692694,
    0xe49b69c19ef14ad2, 0xefbe4786384f25e3, 0x0fc19dc68b8cd5b5, 0x240ca1cc77ac9c65,
    0x2de92c6f592b0275, 0x4a7484aa6ea6e483, 0x5cb0a9dcbd41fbd4, 0x76f988da831153b5,
    0x983e5152ee66dfab, 0xa831c66d2db43210, 0xb00327c898fb213f, 0xbf597fc7beef0ee4,
    0xc6e00bf33da88fc2, 0xd5a79147930aa725, 0x06ca6351e003826f, 0x142929670a0e6e70,
    0x27b70a8546d22ffc, 0x2e1b21385c26c926, 0x4d2c6dfc5ac42aed, 0x53380d139d95b3df,
    0x650a73548baf63de, 0x766a0abb3c77b2a8, 0x81c2c92e47edaee6, 0x92722c851482353b,
    0xa2bfe8a14cf10364, 0xa81a664bbc423001, 0xc24b8b70d0f89791, 0xc76c51a30654be30,
    0xd192e819d6ef5218, 0xd69906245565a910, 0xf40e35855771202a, 0x106aa07032bbd1b8,
    0x19a4c116b8d2d0c8, 0x1e376c085141ab53, 0x2748774cdf8eeb99, 0x34b0bcb5e19b48a8,
    0x391c0cb3c5c95a63, 0x4ed8aa4ae3418acb, 0x5b9cca4f7763e373, 0x682e6ff3d6b2b8a3,
    0x748f82ee5defb2fc, 0x78a5636f43172f60, 0x84c87814a1f0ab72, 0x8cc702081a6439ec,
    0x90befffa23631e28, 0xa4506cebde82bde9, 0xbef9a3f7b2c67915, 0xc67178f2e372532b,
    0xca273eceea26619c, 0xd186b8c721c0c207, 0xeada7dd6cde0eb1e, 0xf57d4f7fee6ed178,
    0x06f067aa72176fba, 0x0a637dc5a2c898a6, 0x113f9804bef90dae, 0x1b710b35131c471b,
    0x28db77f523047d84, 0x32caab7b40c72493, 0x3c9ebe0a15c9bebc, 0x431d67c49c100d4c,
    0x4cc5d4becb3e42b6, 0x597f299cfc657e2a, 0x5fcb6fab3ad6faec, 0x6c44198c4a475817,
];

// Compression helpers for SHA-512. Same shape as Sigma0_256 et al but
// 64-bit rotation amounts. Translation: each S64(b, x) → x.rotate_right(b).
#[inline(always)]
fn Sigma0_512(x: u64) -> u64 {
    x.rotate_right(28) ^ x.rotate_right(34) ^ x.rotate_right(39)
}

#[inline(always)]
fn Sigma1_512(x: u64) -> u64 {
    x.rotate_right(14) ^ x.rotate_right(18) ^ x.rotate_right(41)
}

#[inline(always)]
fn sigma0_512(x: u64) -> u64 {
    x.rotate_right(1) ^ x.rotate_right(8) ^ (x >> 7)
}

#[inline(always)]
fn sigma1_512(x: u64) -> u64 {
    x.rotate_right(19) ^ x.rotate_right(61) ^ (x >> 6)
}

// Add `n` bits (a u64) to the 128-bit bitcount stored as [u64; 2].
// Translation of the ADDINC128 macro. C uses unsigned wraparound +
// carry detection ("if (w[0] < n) w[1]++"). Rust: `overflowing_add`
// gives the carry bit directly.
#[inline(always)]
fn addinc128(bitcount: &mut [u64; 2], n: u64) {
    let (lo, carry) = bitcount[0].overflowing_add(n);
    bitcount[0] = lo;
    if carry {
        bitcount[1] = bitcount[1].wrapping_add(1);
    }
}

// SHA512_Transform — sha2.c:712-798 (non-unrolled).
fn sha512_transform(ctx: &mut Sha512Ctx, data: &[u8; PG_SHA512_BLOCK_LENGTH]) {
    let mut W: [u64; 16] = [0; 16];

    let mut a = ctx.state[0];
    let mut b = ctx.state[1];
    let mut c = ctx.state[2];
    let mut d = ctx.state[3];
    let mut e = ctx.state[4];
    let mut f = ctx.state[5];
    let mut g = ctx.state[6];
    let mut h = ctx.state[7];

    // Rounds 0..16 — load big-endian u64 from `data`.
    for j in 0..16 {
        let base = j * 8;
        let w = u64::from_be_bytes(data[base..base + 8].try_into().unwrap());
        W[j] = w;
        let t1 = h
            .wrapping_add(Sigma1_512(e))
            .wrapping_add(Ch64(e, f, g))
            .wrapping_add(K512[j])
            .wrapping_add(W[j]);
        let t2 = Sigma0_512(a).wrapping_add(Maj64(a, b, c));
        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(t1);
        d = c;
        c = b;
        b = a;
        a = t1.wrapping_add(t2);
    }

    // Rounds 16..80 — message-schedule expansion.
    for j in 16..80 {
        let s0 = sigma0_512(W[(j + 1) & 0x0f]);
        let s1 = sigma1_512(W[(j + 14) & 0x0f]);
        W[j & 0x0f] = W[j & 0x0f]
            .wrapping_add(s1)
            .wrapping_add(W[(j + 9) & 0x0f])
            .wrapping_add(s0);
        let t1 = h
            .wrapping_add(Sigma1_512(e))
            .wrapping_add(Ch64(e, f, g))
            .wrapping_add(K512[j])
            .wrapping_add(W[j & 0x0f]);
        let t2 = Sigma0_512(a).wrapping_add(Maj64(a, b, c));
        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(t1);
        d = c;
        c = b;
        b = a;
        a = t1.wrapping_add(t2);
    }

    ctx.state[0] = ctx.state[0].wrapping_add(a);
    ctx.state[1] = ctx.state[1].wrapping_add(b);
    ctx.state[2] = ctx.state[2].wrapping_add(c);
    ctx.state[3] = ctx.state[3].wrapping_add(d);
    ctx.state[4] = ctx.state[4].wrapping_add(e);
    ctx.state[5] = ctx.state[5].wrapping_add(f);
    ctx.state[6] = ctx.state[6].wrapping_add(g);
    ctx.state[7] = ctx.state[7].wrapping_add(h);
}

// 64-bit variants of Ch/Maj. C reuses the same macro by virtue of
// integer-type genericity; Rust needs separate fns (or a generic).
// Kept separate for clarity at IR level.
#[inline(always)]
fn Ch64(x: u64, y: u64, z: u64) -> u64 {
    (x & y) ^ (!x & z)
}

#[inline(always)]
fn Maj64(x: u64, y: u64, z: u64) -> u64 {
    (x & y) ^ (x & z) ^ (y & z)
}

/// # Safety
/// `context` must be a valid pointer; `data` must point to at least
/// `len` readable bytes.
#[no_mangle]
pub unsafe extern "C" fn pg_sha512_update(
    context: *mut Sha512Ctx,
    data: *const u8,
    len: usize,
) {
    if len == 0 || context.is_null() {
        return;
    }
    let ctx = &mut *context;
    let mut data_slice = core::slice::from_raw_parts(data, len);

    let usedspace = ((ctx.bitcount[0] >> 3) % PG_SHA512_BLOCK_LENGTH as u64) as usize;
    if usedspace > 0 {
        let freespace = PG_SHA512_BLOCK_LENGTH - usedspace;
        if data_slice.len() >= freespace {
            ctx.buffer[usedspace..].copy_from_slice(&data_slice[..freespace]);
            addinc128(&mut ctx.bitcount, (freespace as u64) << 3);
            data_slice = &data_slice[freespace..];
            let buf = ctx.buffer;
            sha512_transform(ctx, &buf);
        } else {
            ctx.buffer[usedspace..usedspace + data_slice.len()].copy_from_slice(data_slice);
            addinc128(&mut ctx.bitcount, (data_slice.len() as u64) << 3);
            return;
        }
    }

    while data_slice.len() >= PG_SHA512_BLOCK_LENGTH {
        let block: [u8; PG_SHA512_BLOCK_LENGTH] =
            data_slice[..PG_SHA512_BLOCK_LENGTH].try_into().unwrap();
        sha512_transform(ctx, &block);
        addinc128(&mut ctx.bitcount, (PG_SHA512_BLOCK_LENGTH as u64) << 3);
        data_slice = &data_slice[PG_SHA512_BLOCK_LENGTH..];
    }

    if !data_slice.is_empty() {
        ctx.buffer[..data_slice.len()].copy_from_slice(data_slice);
        addinc128(&mut ctx.bitcount, (data_slice.len() as u64) << 3);
    }
}

// SHA512_Last — sha2.c:854-902. Final padding for the 128-byte block,
// 128-bit big-endian bitcount in the last 16 bytes.
fn sha512_last(ctx: &mut Sha512Ctx) {
    let usedspace = ((ctx.bitcount[0] >> 3) % PG_SHA512_BLOCK_LENGTH as u64) as usize;
    // C swaps host→big-endian via REVERSE64 before the byte store.
    let bitcount_hi_be = ctx.bitcount[1].to_be();
    let bitcount_lo_be = ctx.bitcount[0].to_be();

    if usedspace > 0 {
        ctx.buffer[usedspace] = 0x80;
        let next = usedspace + 1;

        if next <= PG_SHA512_SHORT_BLOCK_LENGTH {
            for byte in &mut ctx.buffer[next..PG_SHA512_SHORT_BLOCK_LENGTH] {
                *byte = 0;
            }
        } else {
            if next < PG_SHA512_BLOCK_LENGTH {
                for byte in &mut ctx.buffer[next..PG_SHA512_BLOCK_LENGTH] {
                    *byte = 0;
                }
            }
            // Do second-to-last transform.
            let buf = ctx.buffer;
            sha512_transform(ctx, &buf);

            // Reset for the last transform (zero out the bytes that will
            // hold the bitcount; the rest were just consumed by the
            // second-to-last transform but we'll overwrite buffer[0..112]
            // for the next padding step).
            for byte in &mut ctx.buffer[..PG_SHA512_SHORT_BLOCK_LENGTH] {
                *byte = 0;
            }
        }
    } else {
        for byte in &mut ctx.buffer[..PG_SHA512_SHORT_BLOCK_LENGTH] {
            *byte = 0;
        }
        ctx.buffer[0] = 0x80;
    }

    // Store the 128-bit length, big-endian. C writes bitcount[1] then
    // bitcount[0] (already byte-swapped above) via two u64 stores.
    // Rust: write the byte-swapped values' little-endian representation
    // = the original values' big-endian representation.
    ctx.buffer[PG_SHA512_SHORT_BLOCK_LENGTH..PG_SHA512_SHORT_BLOCK_LENGTH + 8]
        .copy_from_slice(&bitcount_hi_be.to_le_bytes());
    ctx.buffer[PG_SHA512_SHORT_BLOCK_LENGTH + 8..PG_SHA512_BLOCK_LENGTH]
        .copy_from_slice(&bitcount_lo_be.to_le_bytes());

    let buf = ctx.buffer;
    sha512_transform(ctx, &buf);
}

/// # Safety
/// `context` must be valid; `digest` must point to at least
/// `PG_SHA512_DIGEST_LENGTH` (64) writable bytes.
#[no_mangle]
pub unsafe extern "C" fn pg_sha512_final(context: *mut Sha512Ctx, digest: *mut u8) {
    if digest.is_null() || context.is_null() {
        return;
    }
    let ctx = &mut *context;
    sha512_last(ctx);

    let out = core::slice::from_raw_parts_mut(digest, PG_SHA512_DIGEST_LENGTH);
    for (i, word) in ctx.state.iter().enumerate() {
        let be = word.to_be_bytes();
        out[i * 8..(i + 1) * 8].copy_from_slice(&be);
    }

    ctx.state = [0; 8];
    ctx.bitcount = [0, 0];
    ctx.buffer = [0; PG_SHA512_BLOCK_LENGTH];
}

// ─── Template instance: SHA-384 update/final ─────────────────────────────────
//
// sha2.c reuses pg_sha512_update verbatim for SHA-384 (the algorithm is
// SHA-512 with a different initial state, truncated to 48 bytes on
// output). Our lift mirrors that — pg_sha384_update is a one-line
// delegation, and pg_sha384_final differs from pg_sha512_final only in
// the digest length (48 bytes = state[0..6]).

/// # Safety
/// Same as `pg_sha512_update`.
#[no_mangle]
pub unsafe extern "C" fn pg_sha384_update(
    context: *mut Sha512Ctx,
    data: *const u8,
    len: usize,
) {
    pg_sha512_update(context, data, len);
}

/// # Safety
/// `context` must be valid; `digest` must point to at least
/// `PG_SHA384_DIGEST_LENGTH` (48) writable bytes.
#[no_mangle]
pub unsafe extern "C" fn pg_sha384_final(context: *mut Sha512Ctx, digest: *mut u8) {
    if digest.is_null() || context.is_null() {
        return;
    }
    let ctx = &mut *context;
    sha512_last(ctx);

    let out = core::slice::from_raw_parts_mut(digest, PG_SHA384_DIGEST_LENGTH);
    // 48 bytes = state[0..6] big-endian.
    for (i, word) in ctx.state.iter().take(6).enumerate() {
        let be = word.to_be_bytes();
        out[i * 8..(i + 1) * 8].copy_from_slice(&be);
    }

    ctx.state = [0; 8];
    ctx.bitcount = [0, 0];
    ctx.buffer = [0; PG_SHA512_BLOCK_LENGTH];
}

// ─── Template instance: SHA-224 update/final ─────────────────────────────────
//
// Same shape: SHA-224 is SHA-256 with a different initial state,
// truncated to 28 bytes on output. update is a verbatim delegation;
// final is pg_sha256_final but with PG_SHA224_DIGEST_LENGTH and the
// digest loop stops after 7 words (28 bytes = state[0..7]).

/// # Safety
/// Same as `pg_sha256_update`.
#[no_mangle]
pub unsafe extern "C" fn pg_sha224_update(
    context: *mut Sha256Ctx,
    data: *const u8,
    len: usize,
) {
    pg_sha256_update(context, data, len);
}

/// # Safety
/// `context` must be valid; `digest` must point to at least
/// `PG_SHA224_DIGEST_LENGTH` (28) writable bytes.
#[no_mangle]
pub unsafe extern "C" fn pg_sha224_final(context: *mut Sha256Ctx, digest: *mut u8) {
    if digest.is_null() || context.is_null() {
        return;
    }
    let ctx = &mut *context;
    sha256_last(ctx);

    let out = core::slice::from_raw_parts_mut(digest, PG_SHA224_DIGEST_LENGTH);
    // 28 bytes = state[0..7] big-endian.
    for (i, word) in ctx.state.iter().take(7).enumerate() {
        let be = word.to_be_bytes();
        out[i * 4..(i + 1) * 4].copy_from_slice(&be);
    }

    ctx.state = [0; 8];
    ctx.bitcount = 0;
    ctx.buffer = [0; PG_SHA256_BLOCK_LENGTH];
}

// =============================================================================
// Safe Rust convenience wrappers for SHA-224 / SHA-384 / SHA-512.
// =============================================================================

pub fn sha224(input: &[u8]) -> [u8; PG_SHA224_DIGEST_LENGTH] {
    let mut ctx = Sha256Ctx::default();
    let mut digest = [0u8; PG_SHA224_DIGEST_LENGTH];
    unsafe {
        pg_sha224_init(&mut ctx);
        pg_sha224_update(&mut ctx, input.as_ptr(), input.len());
        pg_sha224_final(&mut ctx, digest.as_mut_ptr());
    }
    digest
}

pub fn sha384(input: &[u8]) -> [u8; PG_SHA384_DIGEST_LENGTH] {
    let mut ctx = Sha512Ctx::default();
    let mut digest = [0u8; PG_SHA384_DIGEST_LENGTH];
    unsafe {
        pg_sha384_init(&mut ctx);
        pg_sha384_update(&mut ctx, input.as_ptr(), input.len());
        pg_sha384_final(&mut ctx, digest.as_mut_ptr());
    }
    digest
}

pub fn sha512(input: &[u8]) -> [u8; PG_SHA512_DIGEST_LENGTH] {
    let mut ctx = Sha512Ctx::default();
    let mut digest = [0u8; PG_SHA512_DIGEST_LENGTH];
    unsafe {
        pg_sha512_init(&mut ctx);
        pg_sha512_update(&mut ctx, input.as_ptr(), input.len());
        pg_sha512_final(&mut ctx, digest.as_mut_ptr());
    }
    digest
}

