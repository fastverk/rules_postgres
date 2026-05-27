// Build script: compile the C ABI smoke-test harness (tests/c_abi_smoke.c)
// into a static lib that the integration test links against. Mirrors
// the c-oracle compilation pattern used by pg_int4_cmp / pg_int4_hash /
// pg_int4_arith.

fn main() {
    println!("cargo:rerun-if-changed=tests/c_abi_smoke.c");
    println!("cargo:rerun-if-changed=include/pg_palloc.h");

    cc::Build::new()
        .file("tests/c_abi_smoke.c")
        .include("include")
        .warnings(true)
        .opt_level(2)
        .compile("pg_palloc_c_smoke");
}
