fn main() {
    cc::Build::new()
        .file("c_oracle/renamed_int4_bitwise.c")
        .file("c_oracle/wrappers.c")
        .include("c_oracle")
        .warnings(false)
        .opt_level(2)
        .compile("pg_int4_bitwise_c");

    println!("cargo:rerun-if-changed=c_oracle/int4_bitwise.c");
    println!("cargo:rerun-if-changed=c_oracle/renamed_int4_bitwise.c");
    println!("cargo:rerun-if-changed=c_oracle/wrappers.c");
}
