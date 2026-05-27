//! FIPS 180-4 SHA-256 test vectors.
//!
//! These are the canonical SHA-256 test vectors from the FIPS standard
//! + NIST's CAVP examples. Any conformant SHA-256 implementation MUST
//! produce these outputs bit-for-bit. They serve as our behavioral
//! equivalence check between the C version (Postgres) and our Rust
//! translation: if the translation is correct, every input below
//! produces the listed output.
//!
//! Beyond FIPS vectors, we also include some pathological inputs:
//! empty, single-byte, exactly-block-sized (64 bytes), block+1, etc.
//! Those exercise the buffer-management branches in pg_sha256_update
//! that the FIPS-3-input set doesn't cover.

use pg_sha2::{sha224, sha256, sha384, sha512};

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

#[test]
fn fips_empty_string() {
    let d = sha256(b"");
    assert_eq!(
        hex(&d),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn fips_abc() {
    let d = sha256(b"abc");
    assert_eq!(
        hex(&d),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn fips_56_byte_block() {
    // Input is exactly 448 bits = 56 bytes, the short-block-length.
    // Exercises the path where a single padded block is enough
    // (no second-to-last transform).
    let d = sha256(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq");
    assert_eq!(
        hex(&d),
        "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
    );
}

#[test]
fn fips_one_million_a() {
    // The classic million-'a' vector. Exercises the multi-block
    // streaming path AND the final-padding path with usedspace > 0.
    let d = sha256(&[b'a'; 1_000_000]);
    assert_eq!(
        hex(&d),
        "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
    );
}

#[test]
fn block_aligned_64_bytes() {
    // 64 bytes = exactly one block. Exercises the path where the
    // buffer is filled cleanly, transform runs, and final-padding
    // operates on an empty buffer. Expected value verified via
    // openssl dgst -sha256.
    let d = sha256(&[0x55u8; 64]);
    assert_eq!(
        hex(&d),
        "3d9eae666b06b1a975071aca838b4bb5f27a8324eb2ddab0c8eccd71ceae6b50"
    );
}

#[test]
fn block_plus_one_byte() {
    // 65 bytes — exercises the "buffer fills + 1 leftover byte" path.
    // Expected value verified via openssl dgst -sha256.
    let d = sha256(&[0xa5u8; 65]);
    assert_eq!(
        hex(&d),
        "667f84020d981fcedce2816e4e9969a02d5c317a0aef56a6c588175820f82a81",
    );
}

#[test]
fn long_byte_pattern() {
    // 1024-byte input — exercises plenty of full-block transforms.
    let mut input = [0u8; 1024];
    for (i, b) in input.iter_mut().enumerate() {
        *b = (i & 0xff) as u8;
    }
    let d = sha256(&input);
    // Expected value verified via openssl dgst -sha256 on the same
    // byte pattern (0..255 repeated 4 times).
    assert_eq!(
        hex(&d),
        "785b0751fc2c53dc14a4ce3d800e69ef9ce1009eb327ccf458afe09c242c26c9"
    );
}

#[test]
fn incremental_equivalence() {
    // Updating in two halves should match updating in one go.
    use pg_sha2::{pg_sha256_final, pg_sha256_init, pg_sha256_update, Sha256Ctx};

    let input = b"the quick brown fox jumps over the lazy dog the quick brown fox jumps over the lazy dog";
    let split = 30;

    let full = sha256(input);

    let mut ctx = Sha256Ctx::default();
    let mut chunked = [0u8; 32];
    unsafe {
        pg_sha256_init(&mut ctx);
        pg_sha256_update(&mut ctx, input.as_ptr(), split);
        pg_sha256_update(
            &mut ctx,
            input.as_ptr().add(split),
            input.len() - split,
        );
        pg_sha256_final(&mut ctx, chunked.as_mut_ptr());
    }
    assert_eq!(full, chunked);
}

#[test]
fn template_lift_sha224_init_state() {
    // Smoke-test the cluster-template lift: confirm pg_sha224_init
    // produces the canonical SHA-224 initial state (FIPS 180-4).
    use pg_sha2::{pg_sha224_init, Sha256Ctx};
    let mut ctx = Sha256Ctx::default();
    unsafe { pg_sha224_init(&mut ctx) };
    assert_eq!(ctx.state[0], 0xc1059ed8);
    assert_eq!(ctx.state[7], 0xbefa4fa4);
    assert_eq!(ctx.bitcount, 0);
}

#[test]
fn template_lift_sha384_init_state() {
    use pg_sha2::{pg_sha384_init, Sha512Ctx};
    let mut ctx = Sha512Ctx::default();
    unsafe { pg_sha384_init(&mut ctx) };
    assert_eq!(ctx.state[0], 0xcbbb9d5dc1059ed8);
    assert_eq!(ctx.state[7], 0x47b5481dbefa4fa4);
    assert_eq!(ctx.bitcount, [0, 0]);
}

#[test]
fn template_lift_sha512_init_state() {
    use pg_sha2::{pg_sha512_init, Sha512Ctx};
    let mut ctx = Sha512Ctx::default();
    unsafe { pg_sha512_init(&mut ctx) };
    assert_eq!(ctx.state[0], 0x6a09e667f3bcc908);
    assert_eq!(ctx.state[7], 0x5be0cd19137e2179);
    assert_eq!(ctx.bitcount, [0, 0]);
}

// =============================================================================
// SHA-224 / SHA-384 / SHA-512 FIPS vectors — exercise the template-lifted
// _update + _final paths, not just _init.
// =============================================================================

#[test]
fn fips_sha224_empty() {
    assert_eq!(
        hex(&sha224(b"")),
        "d14a028c2a3a2bc9476102bb288234c415a2b01f828ea62ac5b3e42f"
    );
}

#[test]
fn fips_sha224_abc() {
    assert_eq!(
        hex(&sha224(b"abc")),
        "23097d223405d8228642a477bda255b32aadbce4bda0b3f7e36c9da7"
    );
}

#[test]
fn fips_sha224_56_byte_block() {
    assert_eq!(
        hex(&sha224(
            b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
        )),
        "75388b16512776cc5dba5da1fd890150b0c6455cb4f58b1952522525"
    );
}

#[test]
fn fips_sha384_empty() {
    assert_eq!(
        hex(&sha384(b"")),
        "38b060a751ac96384cd9327eb1b1e36a21fdb71114be07434c0cc7bf63f6e1da274edebfe76f65fbd51ad2f14898b95b"
    );
}

#[test]
fn fips_sha384_abc() {
    assert_eq!(
        hex(&sha384(b"abc")),
        "cb00753f45a35e8bb5a03d699ac65007272c32ab0eded1631a8b605a43ff5bed8086072ba1e7cc2358baeca134c825a7"
    );
}

#[test]
fn fips_sha384_112_byte_block() {
    // 112-byte input — exercises the SHA-384/512 short-block boundary
    // (PG_SHA512_SHORT_BLOCK_LENGTH = 112).
    assert_eq!(
        hex(&sha384(
            b"abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu"
        )),
        "09330c33f71147e83d192fc782cd1b4753111b173b3b05d22fa08086e3b0f712fcc7c71a557e2db966c3e9fa91746039"
    );
}

#[test]
fn fips_sha512_empty() {
    assert_eq!(
        hex(&sha512(b"")),
        "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e"
    );
}

#[test]
fn fips_sha512_abc() {
    assert_eq!(
        hex(&sha512(b"abc")),
        "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f"
    );
}

#[test]
fn fips_sha512_112_byte_block() {
    // 112-byte input — exercises the SHA-512 short-block boundary
    // exactly (begins padding in the same block).
    assert_eq!(
        hex(&sha512(
            b"abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu"
        )),
        "8e959b75dae313da8cf4f72814fc143f8f7779c6eb9f7fa17299aeadb6889018501d289e4900f7e4331b99dec4b5433ac7d329eeb6dd26545e96e55b874be909"
    );
}

#[test]
fn fips_sha512_one_million_a() {
    // Classic million-'a' for SHA-512: exercises multi-block streaming
    // through pg_sha512_update + final padding with usedspace > 0.
    let d = sha512(&[b'a'; 1_000_000]);
    assert_eq!(
        hex(&d),
        "e718483d0ce769644e2e42c7bc15b4638e1f98b13b2044285632a803afa973ebde0ff244877ea60a4cb0432ce577c31beb009c5c2c49aa2e4eadb217ad8cc09b"
    );
}

#[test]
fn sha512_incremental_equivalence() {
    use pg_sha2::{pg_sha512_final, pg_sha512_init, pg_sha512_update, Sha512Ctx};

    let input = b"the quick brown fox jumps over the lazy dog the quick brown fox jumps over the lazy dog the quick brown fox jumps over the lazy dog";
    let split = 50;

    let full = sha512(input);

    let mut ctx = Sha512Ctx::default();
    let mut chunked = [0u8; 64];
    unsafe {
        pg_sha512_init(&mut ctx);
        pg_sha512_update(&mut ctx, input.as_ptr(), split);
        pg_sha512_update(&mut ctx, input.as_ptr().add(split), input.len() - split);
        pg_sha512_final(&mut ctx, chunked.as_mut_ptr());
    }
    assert_eq!(full, chunked);
}

#[test]
fn many_small_updates() {
    // Feed one byte at a time — exercises usedspace > 0 path repeatedly.
    use pg_sha2::{pg_sha256_final, pg_sha256_init, pg_sha256_update, Sha256Ctx};

    let input: &[u8] = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
    let full = sha256(input);

    let mut ctx = Sha256Ctx::default();
    let mut chunked = [0u8; 32];
    unsafe {
        pg_sha256_init(&mut ctx);
        for byte in input {
            pg_sha256_update(&mut ctx, byte, 1);
        }
        pg_sha256_final(&mut ctx, chunked.as_mut_ptr());
    }
    assert_eq!(full, chunked);
}
