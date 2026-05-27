/*
 * renamed_uuid.c — preprocessor-rename shim that compiles uuid.c
 * with its public symbols renamed to `<name>_orig`.
 *
 * Prevents the test linker from silently merging the C oracle's functions
 * with the Rust crate's #[no_mangle] implementations.
 */

#define uuid_eq uuid_eq_orig
#define uuid_ne uuid_ne_orig
#define uuid_lt uuid_lt_orig
#define uuid_le uuid_le_orig
#define uuid_gt uuid_gt_orig
#define uuid_ge uuid_ge_orig
#define uuid_cmp uuid_cmp_orig
#define uuid_hash uuid_hash_orig
#define uuid_hash_extended uuid_hash_extended_orig

#define uuid_internal_cmp uuid_internal_cmp_orig

#include "uuid.c"
