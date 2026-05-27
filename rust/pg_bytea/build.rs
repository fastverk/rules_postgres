fn main() {
    cc::Build::new()
        .file("c_oracle/renamed_bytea.c")
        .file("c_oracle/wrappers.c")
        .include("c_oracle")
        .include("../pg_palloc/include")
        .warnings(false)
        .opt_level(2)
        .compile("pg_bytea_c");

    println!("cargo:rerun-if-changed=c_oracle/bytea.c");
    println!("cargo:rerun-if-changed=c_oracle/renamed_bytea.c");
    println!("cargo:rerun-if-changed=c_oracle/wrappers.c");
    println!("cargo:rerun-if-changed=../pg_palloc/include/pg_palloc.h");
}
