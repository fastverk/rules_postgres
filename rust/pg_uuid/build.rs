//! Compile the C oracle (renamed_uuid.c + wrappers.c) for the
//! differential tests. Mirrors the pattern used in pg_int4_cmp and pg_int4_hash.

fn main() {
    cc::Build::new()
        .file("c_oracle/renamed_uuid.c")
        .file("c_oracle/wrappers.c")
        .warnings(false)
        .opt_level(2)
        .compile("pg_uuid_c");

    println!("cargo:rerun-if-changed=c_oracle/uuid.c");
    println!("cargo:rerun-if-changed=c_oracle/renamed_uuid.c");
    println!("cargo:rerun-if-changed=c_oracle/wrappers.c");
}
