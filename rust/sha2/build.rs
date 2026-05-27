//! Compile Postgres' actual `src/common/sha2.c` into a static lib so
//! Rust tests can link it as the C-side oracle for differential
//! testing.
//!
//! The .c file is vendored at `c_oracle/sha2.c` (copied unmodified
//! from Postgres 17.6's source tree). The headers Postgres-side sha2.c
//! transitively needs (`postgres.h`, `postgres_fe.h`, `common/sha2.h`,
//! `common/sha2_int.h`) are provided as minimal shims in
//! `c_oracle/include/` — just the typedefs + constants sha2.c actually
//! references, none of the runtime Postgres machinery.
//!
//! Output: `libpg_sha256_c.a` linked into all test binaries via
//! cargo's automatic `rustc-link-lib` directive emitted by cc::Build.

fn main() {
    cc::Build::new()
        // renamed_sha2.c #includes sha2.c with each public symbol
        // renamed to `<name>_orig` via the preprocessor. This stops
        // the test linker from silently merging the C-oracle's
        // pg_sha224_init with the Rust crate's no_mangle pg_sha224_init
        // export (which made the diff-test compare Rust to itself —
        // caught 2026-05-26 during llm_translate live validation).
        .file("c_oracle/renamed_sha2.c")
        // wrappers.c calls the renamed pg_sha*_orig functions and
        // re-exports them under c_pg_* names that the differential
        // tests link against on the reference side.
        .file("c_oracle/wrappers.c")
        .include("c_oracle/include")
        // sha2.c is OpenBSD-derived legacy code with a few benign
        // -Wall warnings; silence them so the build output is clean.
        .warnings(false)
        // Optimize: matches what Postgres ships as production. The
        // diff-test wants behavioral equivalence, not IR equivalence,
        // so we don't need to match a specific -O level.
        .opt_level(2)
        .compile("pg_sha256_c");

    println!("cargo:rerun-if-changed=c_oracle/sha2.c");
    println!("cargo:rerun-if-changed=c_oracle/renamed_sha2.c");
    println!("cargo:rerun-if-changed=c_oracle/wrappers.c");
    println!("cargo:rerun-if-changed=c_oracle/include/postgres.h");
    println!("cargo:rerun-if-changed=c_oracle/include/postgres_fe.h");
    println!("cargo:rerun-if-changed=c_oracle/include/common/sha2.h");
    println!("cargo:rerun-if-changed=c_oracle/include/common/sha2_int.h");
}
