fn main() {
    cc::Build::new()
        .file("c_oracle/timestamp_arith.c")
        .file("c_oracle/wrappers.c")
        .include("c_oracle")
        .include("../pg_fcinfo/include") // shared c_oracle_ereport.h
        .include("../pg_palloc/include") // pg_palloc.h (palloc declarations)
        .warnings(false)
        .opt_level(2)
        .compile("pg_timestamp_arith_c");

    println!("cargo:rerun-if-changed=c_oracle/timestamp_arith.c");
    println!("cargo:rerun-if-changed=c_oracle/wrappers.c");
    println!("cargo:rerun-if-changed=../pg_fcinfo/include/c_oracle_ereport.h");
    println!("cargo:rerun-if-changed=../pg_palloc/include/pg_palloc.h");
}
