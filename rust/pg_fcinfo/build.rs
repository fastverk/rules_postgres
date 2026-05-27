//! Compile the ABI probe so `pg_fcinfo::verify_abi()` can compare
//! Postgres' canonical sizeof/offsetof against the Rust mirrors.

fn main() {
    cc::Build::new()
        .file("c_oracle/abi_probe.c")
        // Synthetic V1 fmgr fixtures for the integration test (see
        // tests/fixture.c). Self-contained — mimics the Postgres V1
        // ABI without any postgres_src dependency.
        .file("tests/fixture.c")
        .warnings(false)
        .opt_level(0)
        .compile("pg_fcinfo_abi_probe");

    println!("cargo:rerun-if-changed=c_oracle/abi_probe.c");
    println!("cargo:rerun-if-changed=tests/fixture.c");
}
