fn main() {
    cc::Build::new()
        .file("c_oracle/renamed_float_unary.c")
        .file("c_oracle/wrappers.c")
        .include("c_oracle")
        .include("../pg_fcinfo/include")
        .warnings(false)
        .opt_level(2)
        .compile("pg_float_unary_c");

    println!("cargo:rerun-if-changed=c_oracle/float_unary.c");
    println!("cargo:rerun-if-changed=c_oracle/renamed_float_unary.c");
    println!("cargo:rerun-if-changed=c_oracle/wrappers.c");
}
