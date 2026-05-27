//! Behavioral tests for `pg_pgfnames`.
//!
//! These exercise the public surface as a C caller would:
//! - set up a MemoryContext (via the `with_memory_context` helper)
//! - call `pgfnames` over a real temp directory
//! - verify the returned NULL-terminated array contains the expected
//!   entries (sans "." and "..")
//! - call `pgfnames_cleanup` (no-op under bump-arena but must not crash)
//!
//! We also include the safer Rust-only `pgfnames_safe` API for
//! comparison.

use pg_palloc::PgBump as Bump;
use pg_pgfnames::{pgfnames, pgfnames_cleanup, pgfnames_safe};
use pg_palloc::with_memory_context;
use std::ffi::{CStr, CString};
use std::fs;
use std::os::raw::c_char;
use std::path::PathBuf;

/// Create a fresh temp dir for the test, fill it with given file names.
fn make_tmpdir(label: &str, files: &[&str]) -> PathBuf {
    let mut p = std::env::temp_dir();
    // Disambiguate per test + per run.
    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    p.push(format!("pg_pgfnames_{}_{}_{}", label, pid, nanos));
    fs::create_dir_all(&p).expect("mkdir");
    for f in files {
        fs::write(p.join(f), b"x").expect("write");
    }
    p
}

unsafe fn collect_c(arr: *mut *mut c_char) -> Vec<String> {
    if arr.is_null() {
        return vec![];
    }
    let mut out = Vec::new();
    let mut i = 0isize;
    loop {
        let entry = *arr.offset(i);
        if entry.is_null() {
            break;
        }
        let s = CStr::from_ptr(entry).to_string_lossy().into_owned();
        out.push(s);
        i += 1;
    }
    out
}

// =============================================================================
// C-ABI surface tests.
// =============================================================================

#[test]
fn pgfnames_empty_dir() {
    let dir = make_tmpdir("empty", &[]);
    let c_path = CString::new(dir.to_str().unwrap()).unwrap();

    with_memory_context(|_| unsafe {
        let arr = pgfnames(c_path.as_ptr());
        assert!(!arr.is_null());
        let entries = collect_c(arr);
        assert_eq!(entries.len(), 0, "empty dir should yield 0 entries");
        pgfnames_cleanup(arr);
    });

    fs::remove_dir_all(&dir).ok();
}

#[test]
fn pgfnames_small_dir() {
    let dir = make_tmpdir("small", &["a.txt", "b.txt", "c.txt"]);
    let c_path = CString::new(dir.to_str().unwrap()).unwrap();

    with_memory_context(|_| unsafe {
        let arr = pgfnames(c_path.as_ptr());
        assert!(!arr.is_null());
        let mut entries = collect_c(arr);
        entries.sort();
        assert_eq!(entries, vec!["a.txt", "b.txt", "c.txt"]);
        // Sanity: . and .. omitted.
        assert!(!entries.iter().any(|s| s == "." || s == ".."));
        pgfnames_cleanup(arr);
    });

    fs::remove_dir_all(&dir).ok();
}

#[test]
fn pgfnames_triggers_repalloc() {
    // The C code starts with fnsize=200; create > 200 files to force
    // at least one repalloc cycle. 250 is enough.
    let names: Vec<String> = (0..250).map(|i| format!("f{:04}", i)).collect();
    let names_ref: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    let dir = make_tmpdir("repalloc", &names_ref);
    let c_path = CString::new(dir.to_str().unwrap()).unwrap();

    with_memory_context(|_| unsafe {
        let arr = pgfnames(c_path.as_ptr());
        assert!(!arr.is_null());
        let entries = collect_c(arr);
        assert_eq!(entries.len(), 250, "should have 250 entries after repalloc");
        // All entries match the f{nnnn} pattern.
        for e in &entries {
            assert!(e.starts_with('f'), "unexpected entry: {}", e);
        }
        pgfnames_cleanup(arr);
    });

    fs::remove_dir_all(&dir).ok();
}

#[test]
fn pgfnames_nonexistent_path_returns_null() {
    let bogus = CString::new("/this/path/should/not/exist/zzz_pg_pgfnames").unwrap();
    with_memory_context(|_| unsafe {
        let arr = pgfnames(bogus.as_ptr());
        assert!(arr.is_null(), "non-existent dir should return NULL");
    });
}

#[test]
fn pgfnames_null_path_returns_null() {
    with_memory_context(|_| unsafe {
        let arr = pgfnames(std::ptr::null());
        assert!(arr.is_null(), "NULL path should return NULL");
    });
}

#[test]
fn pgfnames_cleanup_on_null_is_safe() {
    // No context needed — cleanup is a no-op on NULL.
    unsafe {
        pgfnames_cleanup(std::ptr::null_mut());
    }
}

// =============================================================================
// Safer Rust-only API tests.
// =============================================================================

#[test]
fn pgfnames_safe_basic() {
    let dir = make_tmpdir("safe", &["x", "y", "z"]);
    let bump = Bump::new();
    let entries = pgfnames_safe(&bump, &dir).expect("read");
    let mut names: Vec<&str> = entries
        .iter()
        .map(|cs| cs.to_str().unwrap())
        .collect();
    names.sort();
    assert_eq!(names, vec!["x", "y", "z"]);
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn pgfnames_safe_arena_lifetimes_compile() {
    // Compile-time check: borrowed entries outlive a Vec collect.
    let dir = make_tmpdir("lifetimes", &["one", "two"]);
    let bump = Bump::new();
    let collected: Vec<String> = {
        let entries = pgfnames_safe(&bump, &dir).expect("read");
        entries.iter().map(|c| c.to_string_lossy().into_owned()).collect()
    };
    assert_eq!(collected.len(), 2);
    fs::remove_dir_all(&dir).ok();
}

// =============================================================================
// MemoryContext behavior tests.
// =============================================================================

#[test]
fn reset_reclaims_arena() {
    // Allocate, reset, allocate again. The second allocation should
    // succeed (the arena's still alive) and produce a new pointer.
    use pg_palloc::{memory_context_switch_to, MemoryContext, palloc};

    let mut ctx = Box::new(MemoryContext::new());
    let raw = &mut *ctx as *mut MemoryContext;
    unsafe {
        let prev = memory_context_switch_to(raw);

        let p1 = palloc(64);
        assert!(!p1.is_null());

        (*raw).reset();

        let p2 = palloc(64);
        assert!(!p2.is_null());
        // Pointers might or might not coincide post-reset; bumpalo
        // reuses the same chunks. We just assert both are valid.

        memory_context_switch_to(prev);
    }
}

#[test]
fn switching_contexts_isolates_allocations() {
    use pg_palloc::{memory_context_switch_to, palloc, MemoryContext};

    let mut a = Box::new(MemoryContext::new());
    let mut b = Box::new(MemoryContext::new());
    let ra = &mut *a as *mut MemoryContext;
    let rb = &mut *b as *mut MemoryContext;

    unsafe {
        let prev = memory_context_switch_to(ra);
        let p_in_a = palloc(16);
        assert!(!p_in_a.is_null());

        memory_context_switch_to(rb);
        let p_in_b = palloc(16);
        assert!(!p_in_b.is_null());
        // Two distinct arenas → distinct chunks → distinct pointers.
        assert_ne!(p_in_a, p_in_b);

        memory_context_switch_to(prev);
    }
}
