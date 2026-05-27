fn main() {
    cc::Build::new()
        .file("c_oracle/renamed_cash_arith.c")
        .file("c_oracle/wrappers.c")
        .include("c_oracle")
        .include("../pg_fcinfo/include") // shared c_oracle_ereport.h
        .warnings(false)
        .opt_level(2)
        .compile("pg_cash_arith_c");

    println!("cargo:rerun-if-changed=c_oracle/cash_arith.c");
    println!("cargo:rerun-if-changed=c_oracle/renamed_cash_arith.c");
    println!("cargo:rerun-if-changed=c_oracle/wrappers.c");
    println!("cargo:rerun-if-changed=../pg_fcinfo/include/c_oracle_ereport.h");
}
