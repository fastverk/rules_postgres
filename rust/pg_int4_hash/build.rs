fn main() {
    cc::Build::new()
        .file("c_oracle/renamed_int4_hash.c")
        .file("c_oracle/wrappers.c")
        .warnings(false)
        .opt_level(2)
        .compile("pg_int4_hash_c");

    println!("cargo:rerun-if-changed=c_oracle/int4_hash.c");
    println!("cargo:rerun-if-changed=c_oracle/renamed_int4_hash.c");
    println!("cargo:rerun-if-changed=c_oracle/wrappers.c");
}
