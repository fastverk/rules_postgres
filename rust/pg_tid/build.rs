//! Compile the C oracle (renamed_tid.c + wrappers.c) for the
//! differential tests. Mirrors the pattern used in pg_macaddr/pg_uuid.

fn main() {
    cc::Build::new()
        .file("c_oracle/renamed_tid.c")
        .file("c_oracle/wrappers.c")
        .warnings(false)
        .opt_level(2)
        .compile("pg_tid_c");

    println!("cargo:rerun-if-changed=c_oracle/tid.c");
    println!("cargo:rerun-if-changed=c_oracle/renamed_tid.c");
    println!("cargo:rerun-if-changed=c_oracle/wrappers.c");
}
