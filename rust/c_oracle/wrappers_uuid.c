/*
 * wrappers.c — re-export UUID ops under `c_uuid*` names.
 *
 * Allows the test binary to link BOTH the Rust crate implementations
 * AND the C oracle implementations under distinct names for differential
 * testing.
 */

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

typedef unsigned int Oid;
typedef int16_t   int16;
typedef int32_t   int32;
typedef int64_t   int64;
typedef uint32_t  uint32;
typedef uint64_t  uint64;
typedef uintptr_t Datum;
typedef void *fmNodePtr;
#define FLEXIBLE_ARRAY_MEMBER

typedef struct NullableDatum {
    Datum value;
    bool  isnull;
} NullableDatum;

struct FmgrInfo;

typedef struct FunctionCallInfoBaseData {
    struct FmgrInfo *flinfo;
    fmNodePtr        context;
    fmNodePtr        resultinfo;
    Oid              fncollation;
    bool             isnull;
    short            nargs;
    NullableDatum    args[FLEXIBLE_ARRAY_MEMBER];
} FunctionCallInfoBaseData;

/* The renamed (_orig) entry points from renamed_uuid.c. */
extern Datum uuid_eq_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum uuid_ne_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum uuid_lt_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum uuid_le_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum uuid_gt_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum uuid_ge_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum uuid_cmp_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum uuid_hash_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum uuid_hash_extended_orig(FunctionCallInfoBaseData *fcinfo);

/* Re-export under c_uuid* names. */
Datum c_uuid_eq(FunctionCallInfoBaseData *fcinfo) { return uuid_eq_orig(fcinfo); }
Datum c_uuid_ne(FunctionCallInfoBaseData *fcinfo) { return uuid_ne_orig(fcinfo); }
Datum c_uuid_lt(FunctionCallInfoBaseData *fcinfo) { return uuid_lt_orig(fcinfo); }
Datum c_uuid_le(FunctionCallInfoBaseData *fcinfo) { return uuid_le_orig(fcinfo); }
Datum c_uuid_gt(FunctionCallInfoBaseData *fcinfo) { return uuid_gt_orig(fcinfo); }
Datum c_uuid_ge(FunctionCallInfoBaseData *fcinfo) { return uuid_ge_orig(fcinfo); }
Datum c_uuid_cmp(FunctionCallInfoBaseData *fcinfo) { return uuid_cmp_orig(fcinfo); }
Datum c_uuid_hash(FunctionCallInfoBaseData *fcinfo) { return uuid_hash_orig(fcinfo); }
Datum c_uuid_hash_extended(FunctionCallInfoBaseData *fcinfo) { return uuid_hash_extended_orig(fcinfo); }

/* Jenkins/lookup3 hash implementations for the Oracle (matching pg_fcinfo). */
#define rot(x, k) (((x) << (k)) | ((x) >> (32 - (k))))
#define mix(a, b, c) { \
    a -= c; a ^= rot(c, 4); c += b; \
    b -= a; b ^= rot(a, 6); a += c; \
    c -= b; c ^= rot(b, 8); b += a; \
    a -= c; a ^= rot(c, 16); c += b; \
    b -= a; b ^= rot(a, 19); a += c; \
    c -= b; c ^= rot(b, 4); b += a; }
#define final(a, b, c) { \
    c ^= b; c -= rot(b, 14); \
    a ^= c; a -= rot(c, 11); \
    b ^= a; b -= rot(a, 25); \
    c ^= b; c -= rot(b, 16); \
    a ^= c; a -= rot(c, 4); \
    b ^= a; b -= rot(a, 14); \
    c ^= b; c -= rot(b, 24); }

static inline uint32_t
read_le32(const unsigned char *k)
{
    return k[0] | (k[1] << 8) | (k[2] << 16) | (k[3] << 24);
}

Datum
hash_any(const unsigned char *k, int keylen)
{
    uint32_t a = 0x9e3779b9 + keylen + 3923095;
    uint32_t b = a;
    uint32_t c = a;

    /* Process first 12 bytes. */
    a += read_le32(k + 0);
    b += read_le32(k + 4);
    c += read_le32(k + 8);
    mix(a, b, c);

    /* Process last 4 bytes. */
    a += read_le32(k + 12);
    final(a, b, c);

    return (Datum) c;
}

Datum
hash_any_extended(const unsigned char *k, int keylen, uint64 seed)
{
    uint32_t a = 0x9e3779b9 + keylen + 3923095;
    uint32_t b = a;
    uint32_t c = a;

    /* Apply seed if non-zero. */
    if (seed != 0) {
        a += (uint32_t)(seed >> 32);
        b += (uint32_t)seed;
        mix(a, b, c);
    }

    /* Process first 12 bytes. */
    a += read_le32(k + 0);
    b += read_le32(k + 4);
    c += read_le32(k + 8);
    mix(a, b, c);

    /* Process last 4 bytes. */
    a += read_le32(k + 12);
    final(a, b, c);

    return (Datum) (((uint64) b << 32) | (uint64) c);
}
