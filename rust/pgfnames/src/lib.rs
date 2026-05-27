//! Hand-translation of Postgres' `src/common/pgfnames.c` → Rust.
//!
//! Second milestone of the cluster-driven C→Rust translation spike.
//! Originally embedded its own `MemoryContext` / `palloc` / `pfree`
//! adapter; that adapter is now productized as `pg_palloc` and this
//! crate consumes it via the dependency.
//!
//! Refactor history:
//!   - v0.0.1 (May 2026): spike with embedded MemoryContext adapter.
//!   - v0.0.2 (May 2026): depends on `pg_palloc`; this file is now
//!     purely the two pgfnames functions + their tests. All
//!     palloc-family symbols (`palloc`, `pfree`, `repalloc`, `pstrdup`,
//!     `MemoryContextSwitchTo`, …) are re-exported into the linked
//!     staticlib transitively from `pg_palloc`.
//!
//! See `pg_palloc/src/lib.rs` for the MemoryContext semantics
//! (chunk-header layout, thread-local CurrentMemoryContext, `pfree`-as-
//! no-op rationale, repalloc shrink correctness via the chunk header).

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(clippy::missing_safety_doc)]

use pg_palloc::{palloc, pfree, repalloc, PgBump as Bump};
use std::ffi::{CStr, OsStr};
use std::os::raw::c_char;
use std::os::unix::ffi::OsStrExt;
use std::ptr;

// =============================================================================
// pgfnames — the actual translation target.
//
// C source: src/common/pgfnames.c. Both functions translated.
// =============================================================================

/// `char **pgfnames(const char *path)` — list entries in a directory.
///
/// Returns a NULL-terminated, palloc'd array of palloc'd
/// NUL-terminated C strings. Returns NULL on opendir() failure.
///
/// Caller must call `pgfnames_cleanup` to release (which under our
/// bump-arena story is a no-op for individual entries, but resets
/// nothing — the proper Postgres semantics is that the whole context
/// gets reset eventually).
///
/// # Safety
/// - `path` must be a valid NUL-terminated C string.
/// - `CurrentMemoryContext` must be set.
#[no_mangle]
pub unsafe extern "C" fn pgfnames(path: *const c_char) -> *mut *mut c_char {
    if path.is_null() {
        return ptr::null_mut();
    }
    let path_cstr = CStr::from_ptr(path);
    let path_os = OsStr::from_bytes(path_cstr.to_bytes());

    let entries = match std::fs::read_dir(path_os) {
        Ok(it) => it,
        Err(_) => {
            // C version pg_log_warning's here. We omit the log (no
            // ereport equivalent yet) — see "what's still not handled"
            // in the report.
            return ptr::null_mut();
        }
    };

    let mut fnsize: usize = 200;
    let mut numnames: usize = 0;
    // palloc(fnsize * sizeof(char *))
    let mut filenames = palloc(fnsize * std::mem::size_of::<*mut c_char>()) as *mut *mut c_char;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue, // C reads errno after readdir; warn-and-continue
        };
        let name = entry.file_name();
        let name_bytes = name.as_bytes();
        // C source skips "." and ".."
        if name_bytes == b"." || name_bytes == b".." {
            continue;
        }
        if numnames + 1 >= fnsize {
            fnsize *= 2;
            filenames = repalloc(
                filenames as *mut u8,
                fnsize * std::mem::size_of::<*mut c_char>(),
            ) as *mut *mut c_char;
        }
        // pstrdup: palloc + memcpy of a NUL-terminated string.
        let needed = name_bytes.len() + 1;
        let dst = palloc(needed);
        ptr::copy_nonoverlapping(name_bytes.as_ptr(), dst, name_bytes.len());
        *dst.add(name_bytes.len()) = 0; // NUL terminator
        *filenames.add(numnames) = dst as *mut c_char;
        numnames += 1;
    }

    // Sentinel NULL terminator.
    *filenames.add(numnames) = ptr::null_mut();

    filenames
}

/// `void pgfnames_cleanup(char **filenames)`.
///
/// In C-Postgres this `pfree`s each string then `pfree`s the array.
/// Under our bump-arena story, all pfree's are no-ops — actual
/// reclamation happens at `MemoryContextReset`. We still walk the
/// array (matching the C structure) so that this function is a faithful
/// transliteration; the loop is dead in terms of side-effects but
/// preserves the call shape for cluster-template lift.
///
/// # Safety
/// `filenames` must be the result of a prior `pgfnames` call, or NULL.
#[no_mangle]
pub unsafe extern "C" fn pgfnames_cleanup(filenames: *mut *mut c_char) {
    if filenames.is_null() {
        return;
    }
    let mut fnp = filenames;
    while !(*fnp).is_null() {
        pfree(*fnp as *mut u8);
        fnp = fnp.add(1);
    }
    pfree(filenames as *mut u8);
}

// =============================================================================
// Safer Rust-only API.
//
// `pgfnames_safe` lets Rust callers use the arena without going through
// the thread-local global, and returns `&'arena CStr` references whose
// lifetimes track the arena's. This is the recommended entry point for
// new Rust code; the C ABI above exists for drop-in linkage compat.
// =============================================================================

/// Safer variant of `pgfnames` for Rust callers. Returns each entry as
/// a `&'arena CStr` borrowed from the supplied bump arena. The
/// arena-allocated outer slice has the same lifetime.
pub fn pgfnames_safe<'arena, P: AsRef<OsStr>>(
    bump: &'arena Bump,
    path: P,
) -> Option<&'arena [&'arena CStr]> {
    let entries = std::fs::read_dir(path.as_ref()).ok()?;
    let mut tmp: Vec<&'arena CStr> = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let bytes = name.as_bytes();
        if bytes == b"." || bytes == b".." {
            continue;
        }
        let len = bytes.len() + 1;
        let with_nul = bump.alloc_slice_fill_copy(len, 0u8);
        with_nul[..bytes.len()].copy_from_slice(bytes);
        let cstr = unsafe { CStr::from_bytes_with_nul_unchecked(with_nul) };
        tmp.push(cstr);
    }
    Some(bump.alloc_slice_copy(&tmp))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pg_palloc::with_memory_context;

    #[test]
    fn pgfnames_lists_temp_directory() {
        // Smoke test: call pgfnames on a tmpdir we control, verify the
        // returned NUL-terminated array has at least one entry skipping
        // "." and "..".
        use std::fs;
        let dir = std::env::temp_dir().join(format!("pgfnames_test_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("a.txt"), "hello").unwrap();
        fs::write(dir.join("b.txt"), "world").unwrap();

        let dir_c = format!("{}\0", dir.to_str().unwrap());
        with_memory_context(|_| unsafe {
            let arr = pgfnames(dir_c.as_ptr() as *const c_char);
            assert!(!arr.is_null());
            let mut n = 0;
            let mut p = arr;
            while !(*p).is_null() {
                n += 1;
                p = p.add(1);
            }
            assert!(n >= 2, "expected ≥ 2 entries, saw {n}");
            pgfnames_cleanup(arr);
        });

        let _ = fs::remove_dir_all(&dir);
    }
}
