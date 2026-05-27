//! Canonical hand-translation of `pgfnames` from `src/common/pgfnames.c`.
//!
//! This file is kept as a *clean* canonical (no extra helpers, no
//! safety doc, no thread-local ceremony) so the cluster-template
//! engine can use it as a seed for any siblings discovered by the
//! embedder. The "live" version (with full doc/safety/unsafe-fn
//! handling) is in `src/lib.rs`.
//!
//! Sibling candidates (per the v4 embedder's clustering — to be
//! confirmed once we run klad LSH against this function):
//! - `ReadDir` in `src/bin/pg_basebackup/pg_basebackup.c` (directory
//!   walk + palloc'd array of strings).
//! - `find_my_exec` family — different output (single string) but
//!   same enclosing structure of "loop+grow+palloc'd array".
//!
//! These remain TODO; this file just establishes the canonical for
//! `pgfnames` itself.

use crate::{palloc, pstrdup, repalloc};
use std::ffi::CStr;
use std::os::raw::c_char;
use std::os::unix::ffi::OsStrExt;
use std::ptr;

/// `char **pgfnames(const char *path)` — canonical form.
///
/// # Safety
/// Same as `pg_pgfnames::pgfnames` — `CurrentMemoryContext` must be set,
/// `path` must be NUL-terminated.
#[allow(dead_code)]
pub unsafe fn pgfnames_canonical(path: *const c_char) -> *mut *mut c_char {
    if path.is_null() {
        return ptr::null_mut();
    }
    let path_cstr = CStr::from_ptr(path);
    let path_os = std::ffi::OsStr::from_bytes(path_cstr.to_bytes());

    let entries = match std::fs::read_dir(path_os) {
        Ok(it) => it,
        Err(_) => return ptr::null_mut(),
    };

    let mut fnsize: usize = 200;
    let mut numnames: usize = 0;
    let mut filenames = palloc(fnsize * std::mem::size_of::<*mut c_char>()) as *mut *mut c_char;

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_bytes = name.as_bytes();
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
        // pstrdup-equivalent: allocate +NUL, copy bytes.
        let needed = name_bytes.len() + 1;
        let dst = palloc(needed);
        ptr::copy_nonoverlapping(name_bytes.as_ptr(), dst, name_bytes.len());
        *dst.add(name_bytes.len()) = 0;
        *filenames.add(numnames) = dst as *mut c_char;
        numnames += 1;
    }

    *filenames.add(numnames) = ptr::null_mut();
    filenames
}

// pstrdup is referenced via the canonical form above through palloc
// only; the explicit `pstrdup` call in the C source folds into the
// raw palloc + copy_nonoverlapping pair above for clarity. If we
// later want to preserve the literal `pstrdup` call site for cluster
// matching, this is the line to revisit.
#[allow(dead_code)]
unsafe fn _unused_pstrdup_canonical_marker(s: *const c_char) -> *mut c_char {
    pstrdup(s)
}
