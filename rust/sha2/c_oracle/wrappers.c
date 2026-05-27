/*
 * wrappers.c — re-export Postgres' SHA-2 functions under `c_pg_*` names.
 *
 * Why this exists:
 *
 *   The Rust crate `pg_sha2` exports `pg_sha224_init`, `pg_sha256_init`,
 *   etc. via `#[no_mangle] pub extern "C"` so it can drop-in as the
 *   .o for Postgres' real sha2.c. That's load-bearing for the
 *   eventual "Rust source tree compilable as a Postgres clone" goal.
 *
 *   For differential testing we link BOTH the Rust crate AND the C
 *   oracle (renamed_sha2.c via sha2.c) into the test binary. To
 *   prevent linker collision on the `pg_sha*` names:
 *     - renamed_sha2.c uses #define to rename the C-oracle-internal
 *       symbols to `pg_sha*_orig` before #include "sha2.c".
 *     - This file re-exports them under `c_pg_*` names that the
 *       differential tests link against on the "reference" side.
 *
 *   Bug history: 2026-05-26 caught the original symbol-clash bug
 *   when an obviously-wrong placeholder (zeroed state) passed the
 *   diff-test. First attempted fix (this file calling pg_sha*
 *   directly) didn't work because the linker still resolved the
 *   wrappers.c calls to the Rust crate's symbols. The renamed_sha2.c
 *   #define dance was the actual fix.
 *
 *   When pg_sha2 is dropped into actual Postgres, this wrappers.c
 *   and renamed_sha2.c are NOT linked — they're test-only.
 *   Production link uses sha2.c (or pg_sha2's no_mangle exports)
 *   under their real Postgres names.
 */

#include "postgres.h"     /* uint8/uint32/uint64 typedefs (shim) */
#include "common/sha2.h"
#include "sha2_int.h"

/* The renamed C-oracle-internal entry points (see renamed_sha2.c). */
extern void pg_sha224_init_orig(pg_sha224_ctx *ctx);
extern void pg_sha224_update_orig(pg_sha224_ctx *ctx, const uint8 *input, size_t len);
extern void pg_sha224_final_orig(pg_sha224_ctx *ctx, uint8 *dest);

extern void pg_sha256_init_orig(pg_sha256_ctx *ctx);
extern void pg_sha256_update_orig(pg_sha256_ctx *ctx, const uint8 *input, size_t len);
extern void pg_sha256_final_orig(pg_sha256_ctx *ctx, uint8 *dest);

extern void pg_sha384_init_orig(pg_sha384_ctx *ctx);
extern void pg_sha384_update_orig(pg_sha384_ctx *ctx, const uint8 *input, size_t len);
extern void pg_sha384_final_orig(pg_sha384_ctx *ctx, uint8 *dest);

extern void pg_sha512_init_orig(pg_sha512_ctx *ctx);
extern void pg_sha512_update_orig(pg_sha512_ctx *ctx, const uint8 *input, size_t len);
extern void pg_sha512_final_orig(pg_sha512_ctx *ctx, uint8 *dest);

/* ---- SHA-224 ---- */

void c_pg_sha224_init(pg_sha224_ctx *ctx)
{
	pg_sha224_init_orig(ctx);
}

void c_pg_sha224_update(pg_sha224_ctx *ctx, const uint8 *input, size_t len)
{
	pg_sha224_update_orig(ctx, input, len);
}

void c_pg_sha224_final(pg_sha224_ctx *ctx, uint8 *dest)
{
	pg_sha224_final_orig(ctx, dest);
}

/* ---- SHA-256 ---- */

void c_pg_sha256_init(pg_sha256_ctx *ctx)
{
	pg_sha256_init_orig(ctx);
}

void c_pg_sha256_update(pg_sha256_ctx *ctx, const uint8 *input, size_t len)
{
	pg_sha256_update_orig(ctx, input, len);
}

void c_pg_sha256_final(pg_sha256_ctx *ctx, uint8 *dest)
{
	pg_sha256_final_orig(ctx, dest);
}

/* ---- SHA-384 ---- */

void c_pg_sha384_init(pg_sha384_ctx *ctx)
{
	pg_sha384_init_orig(ctx);
}

void c_pg_sha384_update(pg_sha384_ctx *ctx, const uint8 *input, size_t len)
{
	pg_sha384_update_orig(ctx, input, len);
}

void c_pg_sha384_final(pg_sha384_ctx *ctx, uint8 *dest)
{
	pg_sha384_final_orig(ctx, dest);
}

/* ---- SHA-512 ---- */

void c_pg_sha512_init(pg_sha512_ctx *ctx)
{
	pg_sha512_init_orig(ctx);
}

void c_pg_sha512_update(pg_sha512_ctx *ctx, const uint8 *input, size_t len)
{
	pg_sha512_update_orig(ctx, input, len);
}

void c_pg_sha512_final(pg_sha512_ctx *ctx, uint8 *dest)
{
	pg_sha512_final_orig(ctx, dest);
}
