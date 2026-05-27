/*
 * Minimal shim. Provides the PG_SHA*_DIGEST_LENGTH /
 * PG_SHA*_BLOCK_LENGTH constants that sha2.c + sha2_int.h reference.
 * Values lifted from Postgres' src/include/common/sha2.h.
 */
#ifndef PG_SHIM_COMMON_SHA2_H
#define PG_SHIM_COMMON_SHA2_H

#define PG_SHA224_BLOCK_LENGTH   64
#define PG_SHA224_DIGEST_LENGTH  28
#define PG_SHA256_BLOCK_LENGTH   64
#define PG_SHA256_DIGEST_LENGTH  32
#define PG_SHA384_BLOCK_LENGTH  128
#define PG_SHA384_DIGEST_LENGTH  48
#define PG_SHA512_BLOCK_LENGTH  128
#define PG_SHA512_DIGEST_LENGTH  64

#endif /* PG_SHIM_COMMON_SHA2_H */
