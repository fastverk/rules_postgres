fn main() {
    cc::Build::new()
        .file("c_oracle/renamed_text.c")
        .file("c_oracle/wrappers.c")
        .include("c_oracle")
        .include("../pg_palloc/include")
        .warnings(false)
        .opt_level(2)
        .compile("pg_text_c");

    println!("cargo:rerun-if-changed=c_oracle/text.c");
    println!("cargo:rerun-if-changed=c_oracle/renamed_text.c");
    println!("cargo:rerun-if-changed=c_oracle/wrappers.c");
    println!("cargo:rerun-if-changed=../pg_palloc/include/pg_palloc.h");
}
