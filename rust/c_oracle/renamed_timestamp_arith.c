/*
 * renamed_timestamp_arith.c — preprocessor-rename shim that compiles
 * timestamp_arith.c with its public symbols renamed to `<name>_orig`.
 * Prevents the test linker from silently merging the C oracle's
 * functions with the Rust crate's #[no_mangle] implementations.
 *
 * Mirrors the pattern in pg_uuid / pg_macaddr / pg_tid / pg_interval /
 * pg_int4_arith.
 */

#define timestamp_pl_interval  timestamp_pl_interval_orig
#define timestamp_mi_interval  timestamp_mi_interval_orig
#define timestamp_mi           timestamp_mi_orig

#include "timestamp_arith.c"
