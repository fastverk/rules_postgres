fn main() {
    cc::Build::new()
        .file("c_oracle/renamed_float_arith.c")
        .file("c_oracle/wrappers.c")
        .include("c_oracle")
        .include("../pg_fcinfo/include")
        .warnings(false)
        .opt_level(2)
        .compile("pg_float_arith_c");

    println!("cargo:rerun-if-changed=c_oracle/float_arith.c");
    println!("cargo:rerun-if-changed=c_oracle/renamed_float_arith.c");
    println!("cargo:rerun-if-changed=c_oracle/wrappers.c");
}
