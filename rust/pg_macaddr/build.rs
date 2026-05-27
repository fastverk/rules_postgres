//! Compile the C oracle (renamed_macaddr.c + wrappers.c) for the
//! differential tests. Mirrors the pattern used in pg_uuid.

fn main() {
    cc::Build::new()
        .file("c_oracle/renamed_macaddr.c")
        .file("c_oracle/wrappers.c")
        .warnings(false)
        .opt_level(2)
        .compile("pg_macaddr_c");

    println!("cargo:rerun-if-changed=c_oracle/macaddr.c");
    println!("cargo:rerun-if-changed=c_oracle/renamed_macaddr.c");
    println!("cargo:rerun-if-changed=c_oracle/wrappers.c");
}
