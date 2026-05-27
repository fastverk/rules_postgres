/*
 * renamed_interval.c — rename each vendored fmgr body to <fn>_orig
 * before compiling, so the test linker doesn't merge them with the
 * Rust crate's #[no_mangle] symbols.
 *
 * The static helper functions (interval_um_internal, finite_interval_pl,
 * finite_interval_mi) have no external linkage in real Postgres, so they
 * don't need renaming.
 */

#define interval_um  interval_um_orig
#define interval_pl  interval_pl_orig
#define interval_mi  interval_mi_orig

#include "interval.c"
