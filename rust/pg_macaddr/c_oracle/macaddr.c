/*
 * macaddr.c — standalone C oracle for macaddr comparison and hash functions.
 *
 * This file is vendored from Postgres' src/backend/utils/adt/mac.c.
 * It compiles standalone with self-contained typedefs and macros for
 * differential testing against the Rust implementations.
 *
 * Functions:
 *   macaddr_eq, macaddr_ne, macaddr_lt, macaddr_le, macaddr_gt, macaddr_ge
 *   macaddr_cmp
 *   hashmacaddr, hashmacaddrextended
 */

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>
#include <string.h>

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

#define PG_FUNCTION_ARGS FunctionCallInfoBaseData *fcinfo

#define PG_GETARG_MACADDR_P(n) \
    ((macaddr *) DatumGetPointer(fcinfo->args[n].value))

#define PG_GETARG_INT64(n) \
    ((int64) fcinfo->args[n].value)

#define DatumGetPointer(X) ((void *) (X))

#define PG_RETURN_BOOL(x) \
    do { fcinfo->isnull = false; return (Datum)((x) ? 1 : 0); } while(0)

#define PG_RETURN_INT32(x) \
    do { fcinfo->isnull = false; return (Datum)((uint32)(x)); } while(0)

/* macaddr type definition */
typedef struct macaddr {
    unsigned char a;
    unsigned char b;
    unsigned char c;
    unsigned char d;
    unsigned char e;
    unsigned char f;
} macaddr;

/*
 * Utility macros used for sorting and comparing:
 */
#define hibits(addr) \
    ((unsigned long)(((addr)->a<<16)|((addr)->b<<8)|((addr)->c)))

#define lobits(addr) \
    ((unsigned long)(((addr)->d<<16)|((addr)->e<<8)|((addr)->f)))

/*
 * Comparison function for sorting:
 */

static int
macaddr_cmp_internal(macaddr *a1, macaddr *a2)
{
    if (hibits(a1) < hibits(a2))
        return -1;
    else if (hibits(a1) > hibits(a2))
        return 1;
    else if (lobits(a1) < lobits(a2))
        return -1;
    else if (lobits(a1) > lobits(a2))
        return 1;
    else
        return 0;
}

Datum
macaddr_cmp(PG_FUNCTION_ARGS)
{
    macaddr    *a1 = PG_GETARG_MACADDR_P(0);
    macaddr    *a2 = PG_GETARG_MACADDR_P(1);

    PG_RETURN_INT32(macaddr_cmp_internal(a1, a2));
}

/*
 * Boolean comparisons.
 */

Datum
macaddr_lt(PG_FUNCTION_ARGS)
{
    macaddr    *a1 = PG_GETARG_MACADDR_P(0);
    macaddr    *a2 = PG_GETARG_MACADDR_P(1);

    PG_RETURN_BOOL(macaddr_cmp_internal(a1, a2) < 0);
}

Datum
macaddr_le(PG_FUNCTION_ARGS)
{
    macaddr    *a1 = PG_GETARG_MACADDR_P(0);
    macaddr    *a2 = PG_GETARG_MACADDR_P(1);

    PG_RETURN_BOOL(macaddr_cmp_internal(a1, a2) <= 0);
}

Datum
macaddr_eq(PG_FUNCTION_ARGS)
{
    macaddr    *a1 = PG_GETARG_MACADDR_P(0);
    macaddr    *a2 = PG_GETARG_MACADDR_P(1);

    PG_RETURN_BOOL(macaddr_cmp_internal(a1, a2) == 0);
}

Datum
macaddr_ge(PG_FUNCTION_ARGS)
{
    macaddr    *a1 = PG_GETARG_MACADDR_P(0);
    macaddr    *a2 = PG_GETARG_MACADDR_P(1);

    PG_RETURN_BOOL(macaddr_cmp_internal(a1, a2) >= 0);
}

Datum
macaddr_gt(PG_FUNCTION_ARGS)
{
    macaddr    *a1 = PG_GETARG_MACADDR_P(0);
    macaddr    *a2 = PG_GETARG_MACADDR_P(1);

    PG_RETURN_BOOL(macaddr_cmp_internal(a1, a2) > 0);
}

Datum
macaddr_ne(PG_FUNCTION_ARGS)
{
    macaddr    *a1 = PG_GETARG_MACADDR_P(0);
    macaddr    *a2 = PG_GETARG_MACADDR_P(1);

    PG_RETURN_BOOL(macaddr_cmp_internal(a1, a2) != 0);
}

/*
 * Support function for hash indexes on macaddr.
 */

/* Forward declarations */
static Datum hash_any(const unsigned char *k, int keylen);
static Datum hash_any_extended(const unsigned char *k, int keylen, uint64 seed);

static inline uint32_t
read_le32(const unsigned char *k)
{
    return k[0] | (k[1] << 8) | (k[2] << 16) | (k[3] << 24);
}

Datum
hashmacaddr(PG_FUNCTION_ARGS)
{
    macaddr    *key = PG_GETARG_MACADDR_P(0);

    return hash_any((unsigned char *) key, sizeof(macaddr));
}

Datum
hashmacaddrextended(PG_FUNCTION_ARGS)
{
    macaddr    *key = PG_GETARG_MACADDR_P(0);

    return hash_any_extended((unsigned char *) key, sizeof(macaddr),
                             PG_GETARG_INT64(1));
}

/* Jenkins/lookup3 hash implementations. */
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

Datum
hash_any(const unsigned char *k, int keylen)
{
    uint32_t a = 0x9e3779b9 + keylen + 3923095;
    uint32_t b = a;
    uint32_t c = a;

    /* For 6-byte macaddr: process 4 bytes into a, 2 bytes into b. */
    if (keylen == 6) {
        a += read_le32(k + 0);
        b += k[4] | (k[5] << 8);
        final(a, b, c);
        return (Datum) c;
    }

    /* Generic path for other sizes (not used by macaddr). */
    if (keylen >= 4) {
        a += read_le32(k + 0);
        if (keylen >= 8) {
            b += read_le32(k + 4);
            if (keylen >= 12) {
                c += read_le32(k + 8);
                mix(a, b, c);
                if (keylen >= 16) {
                    a += read_le32(k + 12);
                    final(a, b, c);
                }
            }
        }
    }

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

    /* For 6-byte macaddr: process 4 bytes into a, 2 bytes into b. */
    if (keylen == 6) {
        a += read_le32(k + 0);
        b += k[4] | (k[5] << 8);
        final(a, b, c);
        return (Datum) (((uint64) b << 32) | (uint64) c);
    }

    /* Generic path for other sizes (not used by macaddr). */
    if (keylen >= 4) {
        a += read_le32(k + 0);
        if (keylen >= 8) {
            b += read_le32(k + 4);
            if (keylen >= 12) {
                c += read_le32(k + 8);
                mix(a, b, c);
                if (keylen >= 16) {
                    a += read_le32(k + 12);
                    final(a, b, c);
                }
            }
        }
    }

    return (Datum) (((uint64) b << 32) | (uint64) c);
}
