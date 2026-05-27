//! Contract test: every emitted `pg_finfo_NAME` stub must return a
//! pointer to a `PgFinfoRecord` whose `api_version` field is `1`.
//!
//! This matches the C macro's contract verbatim (fmgr.h:415):
//!   `static const Pg_finfo_record my_finfo = { 1 };`
//! and is the only behavioral guarantee the V1 fmgr loader relies on.
//!
//! Because the C-side `pg_finfo_NAME` symbols would collide with the
//! Rust-side ones at link time, this test verifies the Rust side
//! against the macro's well-defined contract rather than against a
//! live C oracle. The companion crate that lifts the actual
//! `Datum fn(FunctionCallInfo)` bodies will do FFI diff-testing
//! because those symbols don't have a name collision when both
//! sides are linked.

use pg_finfo_v1::{PgFinfoRecord, PG_FINFO_TABLE};

#[test]
fn all_stubs_return_api_version_1() {
    assert!(
        !PG_FINFO_TABLE.is_empty(),
        "generated table should be non-empty; build.rs may have failed"
    );

    let mut failures: Vec<String> = Vec::new();
    for (fmgr_name, finfo_fn) in PG_FINFO_TABLE.iter() {
        // SAFETY: each `pg_finfo_NAME` returns a pointer to its own
        // 'static FINFO. Always non-null, always dereferenceable.
        let p: *const PgFinfoRecord = unsafe { finfo_fn() };
        if p.is_null() {
            failures.push(format!("{fmgr_name}: returned null"));
            continue;
        }
        let v = unsafe { (*p).api_version };
        if v != 1 {
            failures.push(format!(
                "{fmgr_name}: api_version = {v}, expected 1"
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} stubs violated the v1 contract:\n  {}",
        failures.len(),
        PG_FINFO_TABLE.len(),
        failures.join("\n  ")
    );
    eprintln!(
        "[pg_finfo_v1::contract] {} stubs verified",
        PG_FINFO_TABLE.len()
    );
}

#[test]
fn table_matches_manifest_count() {
    // Sanity: the manifest had 871 entries when this crate was
    // scaffolded. If that number drifts (regenerated manifest, etc.)
    // update this assertion intentionally rather than silently.
    assert_eq!(
        PG_FINFO_TABLE.len(),
        871,
        "PG_FINFO_TABLE count drifted from the 2026-05-25 manifest snapshot. \
         If the manifest was regenerated, update this test and the project memo."
    );
}

#[test]
fn no_duplicate_fmgr_names() {
    let mut names: Vec<&str> = PG_FINFO_TABLE.iter().map(|(n, _)| *n).collect();
    names.sort();
    let len_before = names.len();
    names.dedup();
    assert_eq!(
        names.len(),
        len_before,
        "duplicate fmgr_name in PG_FINFO_TABLE — manifest needs dedup"
    );
}

#[test]
fn pointers_are_stable() {
    // Each stub returns a pointer to its own 'static FINFO. Calling
    // a stub twice must return the same pointer (otherwise the
    // ABI contract — single record per fmgr function — is broken).
    for (fmgr_name, finfo_fn) in PG_FINFO_TABLE.iter().take(8) {
        let p1: *const PgFinfoRecord = unsafe { finfo_fn() };
        let p2: *const PgFinfoRecord = unsafe { finfo_fn() };
        assert_eq!(
            p1, p2,
            "{fmgr_name}: stub returned different pointers across calls"
        );
    }
}
