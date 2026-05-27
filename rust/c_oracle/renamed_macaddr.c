/*
 * renamed_macaddr.c — preprocessor-rename shim that compiles macaddr.c
 * with its public symbols renamed to `<name>_orig`.
 *
 * Prevents the test linker from silently merging the C oracle's functions
 * with the Rust crate's #[no_mangle] implementations.
 */

#define macaddr_eq macaddr_eq_orig
#define macaddr_ne macaddr_ne_orig
#define macaddr_lt macaddr_lt_orig
#define macaddr_le macaddr_le_orig
#define macaddr_gt macaddr_gt_orig
#define macaddr_ge macaddr_ge_orig
#define macaddr_cmp macaddr_cmp_orig
#define hashmacaddr hashmacaddr_orig
#define hashmacaddrextended hashmacaddrextended_orig

#define macaddr_cmp_internal macaddr_cmp_internal_orig

#include "macaddr.c"
