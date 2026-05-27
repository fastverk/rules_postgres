//! Compile the C oracle (renamed_int4_cmp.c + wrappers.c) for the
//! differential tests. Mirrors the sha2 crate's pattern:
//! `_orig`-renamed C-side public symbols + `c_*`-prefixed re-exports.

fn main() {
    cc::Build::new()
        .file("c_oracle/renamed_int4_cmp.c")
        .file("c_oracle/wrappers.c")
        .warnings(false)
        .opt_level(2)
        .compile("pg_int4_cmp_c");

    println!("cargo:rerun-if-changed=c_oracle/int4_cmp.c");
    println!("cargo:rerun-if-changed=c_oracle/renamed_int4_cmp.c");
    println!("cargo:rerun-if-changed=c_oracle/wrappers.c");
}
