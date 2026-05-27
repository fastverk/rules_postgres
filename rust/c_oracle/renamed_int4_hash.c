/*
 * renamed_int4_hash.c — preprocessor-rename shim that compiles
 * int4_hash.c with its public symbols renamed to `<name>_orig`.
 * Mirrors pg_int4_cmp/c_oracle/renamed_int4_cmp.c.
 *
 * Why: prevent the test linker from merging the C oracle's `hashint4`
 * with the Rust crate's `#[no_mangle] pub extern "C" fn hashint4`
 * (would make the diff-test compare Rust to itself).
 */

#define hashchar  hashchar_orig
#define hashint2  hashint2_orig
#define hashint4  hashint4_orig
#define hashint8  hashint8_orig
#define hashoid   hashoid_orig
#define hashenum  hashenum_orig

#define hashcharextended  hashcharextended_orig
#define hashint2extended  hashint2extended_orig
#define hashint4extended  hashint4extended_orig
#define hashint8extended  hashint8extended_orig
#define hashoidextended   hashoidextended_orig
#define hashenumextended  hashenumextended_orig

/* Also rename the hashfn.c primitives so the C oracle owns its private
 * copy distinct from anything pg_fcinfo's mirror exposes. */
#define hash_bytes_uint32          hash_bytes_uint32_orig
#define hash_bytes_uint32_extended hash_bytes_uint32_extended_orig

#include "int4_hash.c"
