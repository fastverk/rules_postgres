/*
 * renamed_sha2.c — preprocessor-rename shim that compiles sha2.c
 * with its public symbols renamed to `<name>_orig`.
 *
 * Why:
 *
 *   At test-link time, both the Rust crate (`pg_sha2`) and the C
 *   oracle (libpg_sha256_c.a) export `pg_sha224_init`, etc. The
 *   linker silently picks one. With wrappers.c calling
 *   `pg_sha224_init`, that call resolved to the Rust crate's symbol
 *   even though the wrapper lives in the C archive — so the
 *   reference side of the diff-test was calling the Rust impl,
 *   making the test compare Rust to itself.
 *
 *   Concrete evidence (2026-05-26): a sabotaged `pg_sha224_init`
 *   that zeros the state silently passed the diff-test even after
 *   the wrappers.c approach was added.
 *
 * The fix: physically rename the C-oracle-internal public symbols
 * via #define before #include "sha2.c". The C archive then has
 * `pg_sha224_init_orig` (no clash with Rust), and wrappers.c calls
 * those instead. Two genuinely separate impls compared.
 */

#define pg_sha224_init   pg_sha224_init_orig
#define pg_sha224_update pg_sha224_update_orig
#define pg_sha224_final  pg_sha224_final_orig

#define pg_sha256_init   pg_sha256_init_orig
#define pg_sha256_update pg_sha256_update_orig
#define pg_sha256_final  pg_sha256_final_orig

#define pg_sha384_init   pg_sha384_init_orig
#define pg_sha384_update pg_sha384_update_orig
#define pg_sha384_final  pg_sha384_final_orig

#define pg_sha512_init   pg_sha512_init_orig
#define pg_sha512_update pg_sha512_update_orig
#define pg_sha512_final  pg_sha512_final_orig

#include "sha2.c"
