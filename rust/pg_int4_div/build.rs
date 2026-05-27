fn main() {
    // Find Postgres include paths in bazel cache (same strategy as check-int-arith-grounding.sh)
    let pg_include = find_postgres_include();

    let mut builder = cc::Build::new();
    builder
        .file("c_oracle/renamed_int_div.c")
        .file("c_oracle/wrappers.c")
        .include("c_oracle")
        .include("../pg_fcinfo/include") // shared c_oracle_ereport.h
        .include(&pg_include)
        .warnings(false)
        .opt_level(2);

    if let Some(libpg_include) = find_libpg_include() {
        builder.include(&libpg_include);
    }

    builder.compile("pg_int4_div_c");

    println!("cargo:rerun-if-changed=c_oracle/int_div.c");
    println!("cargo:rerun-if-changed=c_oracle/renamed_int_div.c");
    println!("cargo:rerun-if-changed=c_oracle/wrappers.c");
    println!("cargo:rerun-if-changed=../pg_fcinfo/include/c_oracle_ereport.h");
}

fn find_postgres_include() -> String {
    let candidates = [
        "/Users/mattmarshall/Library/Caches/bazel/_bazel_mattmarshall/0fee58293a938d2caa5da4f904216d74/external/+postgres_src_extension+postgres_src/src/include",
        "/Users/mattmarshall/Library/Caches/bazel/_bazel_mattmarshall/90476f1c3b5e144c1cb932d1d6a2c5b0/external/+pg+postgres_src/src/include",
    ];
    for candidate in &candidates {
        if std::path::Path::new(candidate).exists() {
            return candidate.to_string();
        }
    }
    panic!("postgres_src include path not found in bazel cache");
}

fn find_libpg_include() -> Option<String> {
    let candidates = [
        "/Users/mattmarshall/Library/Caches/bazel/_bazel_mattmarshall/0fee58293a938d2caa5da4f904216d74/external/+libpg_query_extension+libpg_query/src/postgres/include",
        "/Users/mattmarshall/Library/Caches/bazel/_bazel_mattmarshall/90476f1c3b5e144c1cb932d1d6a2c5b0/external/+pg+libpg_query/src/postgres/include",
    ];
    for candidate in &candidates {
        if std::path::Path::new(candidate).exists() {
            return Some(candidate.to_string());
        }
    }
    None
}
