/*
 * Minimal shim of Postgres' postgres.h sufficient to compile
 * src/common/sha2.c standalone. This is NOT a substitute for the real
 * postgres.h — it only provides the typedefs sha2.c actually uses:
 * uint8 / uint16 / uint32 / uint64.
 *
 * Used by rules_difftest as the C-side oracle for the SHA-256 / SHA-512
 * diff-test (see ../../tests/diff_sha256.rs, build.rs).
 */

#ifndef PG_SHIM_POSTGRES_H
#define PG_SHIM_POSTGRES_H

#include <stddef.h>
#include <stdint.h>
#include <string.h>

typedef uint8_t  uint8;
typedef uint16_t uint16;
typedef uint32_t uint32;
typedef uint64_t uint64;

#endif /* PG_SHIM_POSTGRES_H */
