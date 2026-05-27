//! Differential test for `pg_sha224_init` — hand-wired from the
//! emitted artifacts that `auto_cluster --emit-difftest` produced.
//!
//! Walks the user-facing flow end-to-end:
//!   1. auto_cluster emitted pg_sha224_init.spec.toml + .c_oracle.rs
//!      stubs (state-mutator → TODOs).
//!   2. The TODOs were filled in by hand: both sides allocate a
//!      context, call `pg_sha224_init`, and return the state[0..8]
//!      array as the comparable post-call value.
//!   3. proptest exercises 64 cases. Inputs are unused — `_init` takes
//!      a ctx and writes constants. We keep the proptest! shell so
//!      the diff-test scaffold matches `diff_sha256.rs`.
//!
//! This proves the emitted spec is wireable into a working diff-test
//! after a small, mechanical edit.

#[path = "c_oracle.rs"]
mod c_oracle;

use proptest::prelude::*;
use std::mem::MaybeUninit;

// `c_pg_sha224_init` is the rename wrapper from `c_oracle/wrappers.c`
// that forwards to the upstream `pg_sha224_init`. The rename ensures
// the test linker can't silently merge the Rust crate's no_mangle
// export with the C oracle's symbol — without it this diff-test was
// effectively comparing the Rust impl to itself (bug caught
// 2026-05-26 during the llm_translate live validation).
extern "C" {
    fn c_pg_sha224_init(ctx: *mut c_oracle::CSha256Ctx);
}

/// Reference: Postgres' actual `pg_sha224_init` via FFI (through the
/// `c_pg_*` rename wrapper). Returns the post-call `state` array.
fn pg_sha224_init_state_via_c() -> [u32; 8] {
    let mut ctx: MaybeUninit<c_oracle::CSha256Ctx> = MaybeUninit::uninit();
    unsafe {
        c_pg_sha224_init(ctx.as_mut_ptr());
        let ctx = ctx.assume_init();
        ctx.state
    }
}

/// Candidate: pg_sha2's hand-translated Rust `pg_sha224_init`.
fn pg_sha224_init_state_via_rust() -> [u32; 8] {
    let mut ctx = pg_sha2::Sha256Ctx::default();
    unsafe {
        pg_sha2::pg_sha224_init(&mut ctx);
    }
    ctx.state
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]
    #[test]
    fn diff_pg_sha224_init(_seed in any::<u8>()) {
        let reference_output: [u32; 8] = pg_sha224_init_state_via_c();
        let candidate_output: [u32; 8] = pg_sha224_init_state_via_rust();
        prop_assert_eq!(
            reference_output, candidate_output,
            "reference and candidate state arrays disagree"
        );
    }
}
