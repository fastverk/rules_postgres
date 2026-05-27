/*
 * tid.c — standalone C oracle for tid comparison, clamping and hash functions.
 *
 * This file is vendored from Postgres' src/backend/utils/adt/tid.c.
 * It compiles standalone with self-contained typedefs and macros for
 * differential testing against the Rust implementations.
 *
 * Functions:
 *   tideq, tidne, tidlt, tidle, tidgt, tidge (6 comparison ops)
 *   bttidcmp (3-way comparison)
 *   tidlarger, tidsmaller (2 clamping ops)
 *   hashtid, hashtidextended (2 hash ops)
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

#define PG_GETARG_ITEMPOINTER(n) \
    ((ItemPointer) DatumGetPointer(fcinfo->args[n].value))

#define PG_GETARG_INT64(n) \
    ((int64) fcinfo->args[n].value)

#define DatumGetPointer(X) ((void *) (X))

#define PG_RETURN_BOOL(x) \
    do { fcinfo->isnull = false; return (Datum)((x) ? 1 : 0); } while(0)

#define PG_RETURN_INT32(x) \
    do { fcinfo->isnull = false; return (Datum)((uint32)(x)); } while(0)

#define PG_RETURN_ITEMPOINTER(x) \
    do { fcinfo->isnull = false; return PointerGetDatum(x); } while(0)

#define PointerGetDatum(X) ((Datum) (uintptr_t) (X))

/* ItemPointer (tid) type definitions */
typedef uint32_t BlockNumber;
typedef uint16_t OffsetNumber;

typedef struct BlockIdData {
    uint16_t bi_hi;
    uint16_t bi_lo;
} BlockIdData;

typedef struct ItemPointerData {
    BlockIdData ip_blkid;
    OffsetNumber ip_posid;
} ItemPointerData;

typedef ItemPointerData *ItemPointer;

/* Helper to extract BlockNumber from BlockIdData */
static inline BlockNumber
BlockIdGetBlockNumber(BlockIdData *blockId)
{
    return (blockId->bi_hi << 16) | blockId->bi_lo;
}

/* Helper to extract OffsetNumber */
static inline OffsetNumber
ItemPointerGetOffsetNumberNoCheck(const ItemPointerData *pointer)
{
    return pointer->ip_posid;
}

/* Helper to extract BlockNumber */
static inline BlockNumber
ItemPointerGetBlockNumberNoCheck(const ItemPointerData *pointer)
{
    return BlockIdGetBlockNumber(&pointer->ip_blkid);
}

/*
 * ItemPointerCompare
 *      Generic btree-style comparison for item pointers.
 */
static int32_t
ItemPointerCompare(ItemPointer arg1, ItemPointer arg2)
{
    BlockNumber b1 = ItemPointerGetBlockNumberNoCheck(arg1);
    BlockNumber b2 = ItemPointerGetBlockNumberNoCheck(arg2);

    if (b1 < b2)
        return -1;
    else if (b1 > b2)
        return 1;
    else if (ItemPointerGetOffsetNumberNoCheck(arg1) <
             ItemPointerGetOffsetNumberNoCheck(arg2))
        return -1;
    else if (ItemPointerGetOffsetNumberNoCheck(arg1) >
             ItemPointerGetOffsetNumberNoCheck(arg2))
        return 1;
    else
        return 0;
}

/* Comparison functions */

Datum
tideq(PG_FUNCTION_ARGS)
{
    ItemPointer arg1 = PG_GETARG_ITEMPOINTER(0);
    ItemPointer arg2 = PG_GETARG_ITEMPOINTER(1);

    PG_RETURN_BOOL(ItemPointerCompare(arg1, arg2) == 0);
}

Datum
tidne(PG_FUNCTION_ARGS)
{
    ItemPointer arg1 = PG_GETARG_ITEMPOINTER(0);
    ItemPointer arg2 = PG_GETARG_ITEMPOINTER(1);

    PG_RETURN_BOOL(ItemPointerCompare(arg1, arg2) != 0);
}

Datum
tidlt(PG_FUNCTION_ARGS)
{
    ItemPointer arg1 = PG_GETARG_ITEMPOINTER(0);
    ItemPointer arg2 = PG_GETARG_ITEMPOINTER(1);

    PG_RETURN_BOOL(ItemPointerCompare(arg1, arg2) < 0);
}

Datum
tidle(PG_FUNCTION_ARGS)
{
    ItemPointer arg1 = PG_GETARG_ITEMPOINTER(0);
    ItemPointer arg2 = PG_GETARG_ITEMPOINTER(1);

    PG_RETURN_BOOL(ItemPointerCompare(arg1, arg2) <= 0);
}

Datum
tidgt(PG_FUNCTION_ARGS)
{
    ItemPointer arg1 = PG_GETARG_ITEMPOINTER(0);
    ItemPointer arg2 = PG_GETARG_ITEMPOINTER(1);

    PG_RETURN_BOOL(ItemPointerCompare(arg1, arg2) > 0);
}

Datum
tidge(PG_FUNCTION_ARGS)
{
    ItemPointer arg1 = PG_GETARG_ITEMPOINTER(0);
    ItemPointer arg2 = PG_GETARG_ITEMPOINTER(1);

    PG_RETURN_BOOL(ItemPointerCompare(arg1, arg2) >= 0);
}

Datum
bttidcmp(PG_FUNCTION_ARGS)
{
    ItemPointer arg1 = PG_GETARG_ITEMPOINTER(0);
    ItemPointer arg2 = PG_GETARG_ITEMPOINTER(1);

    PG_RETURN_INT32(ItemPointerCompare(arg1, arg2));
}

Datum
tidlarger(PG_FUNCTION_ARGS)
{
    ItemPointer arg1 = PG_GETARG_ITEMPOINTER(0);
    ItemPointer arg2 = PG_GETARG_ITEMPOINTER(1);

    PG_RETURN_ITEMPOINTER(ItemPointerCompare(arg1, arg2) >= 0 ? arg1 : arg2);
}

Datum
tidsmaller(PG_FUNCTION_ARGS)
{
    ItemPointer arg1 = PG_GETARG_ITEMPOINTER(0);
    ItemPointer arg2 = PG_GETARG_ITEMPOINTER(1);

    PG_RETURN_ITEMPOINTER(ItemPointerCompare(arg1, arg2) <= 0 ? arg1 : arg2);
}

/* Hash support functions */

/* Forward declarations */
static Datum hash_any(const unsigned char *k, int keylen);
static Datum hash_any_extended(const unsigned char *k, int keylen, uint64 seed);

Datum
hashtid(PG_FUNCTION_ARGS)
{
    ItemPointer key = PG_GETARG_ITEMPOINTER(0);

    return hash_any((unsigned char *) key,
                    sizeof(BlockIdData) + sizeof(OffsetNumber));
}

Datum
hashtidextended(PG_FUNCTION_ARGS)
{
    ItemPointer key = PG_GETARG_ITEMPOINTER(0);
    uint64 seed = PG_GETARG_INT64(1);

    return hash_any_extended((unsigned char *) key,
                             sizeof(BlockIdData) + sizeof(OffsetNumber),
                             seed);
}

/* Jenkins/lookup3 hash implementations. */
#define rot(x, k) (((x) << (k)) | ((x) >> (32 - (k))))

#define mix(a, b, c) \
do { \
    a -= c; a ^= rot(c, 4); c += b; \
    b -= a; b ^= rot(a, 6); a += c; \
    c -= b; c ^= rot(b, 8); b += a; \
    a -= c; a ^= rot(c, 16); c += b; \
    b -= a; b ^= rot(a, 19); a += c; \
    c -= b; c ^= rot(b, 4); b += a; \
} while(0)

#define final(a, b, c) \
do { \
    c ^= b; c -= rot(b, 14); \
    a ^= c; a -= rot(c, 11); \
    b ^= a; b -= rot(a, 25); \
    c ^= b; c -= rot(b, 16); \
    a ^= c; a -= rot(c, 4); \
    b ^= a; b -= rot(a, 14); \
    c ^= b; c -= rot(b, 24); \
} while(0)

static inline uint32_t
read_le32(const unsigned char *k)
{
    return k[0] | (k[1] << 8) | (k[2] << 16) | (k[3] << 24);
}

/*
 * hash_any() -- hash a variable-length key into a 32-bit value
 *      k       : the key (array of bytes)
 *      keylen  : the length of the key
 *
 * Returns a uint32 Datum containing the hash value.
 * It is implementation-dependent what happens if keylen is 0.
 */
static Datum
hash_any(const unsigned char *k, int keylen)
{
    uint32_t a, b, c, len;

    len = keylen;
    a = b = c = 0x9e3779b9 + len + 0;

    while (len >= 12)
    {
        a += read_le32(k);
        b += read_le32(k + 4);
        c += read_le32(k + 8);
        mix(a, b, c);
        k += 12;
        len -= 12;
    }

    switch (len)
    {
        case 11:
            c += ((uint32_t) k[10]) << 24;
        case 10:
            c += ((uint32_t) k[9]) << 16;
        case 9:
            c += ((uint32_t) k[8]) << 8;
        case 8:
            b += read_le32(k + 4);
            a += read_le32(k);
            break;
        case 7:
            b += ((uint32_t) k[6]) << 16;
        case 6:
            b += ((uint32_t) k[5]) << 8;
        case 5:
            b += k[4];
        case 4:
            a += read_le32(k);
            break;
        case 3:
            a += ((uint32_t) k[2]) << 16;
        case 2:
            a += ((uint32_t) k[1]) << 8;
        case 1:
            a += k[0];
            break;
        case 0:
            return 0;
    }

    final(a, b, c);

    return (Datum) c;
}

/*
 * hash_any_extended() -- hash a variable-length key into a 64-bit value,
 *      with a seed
 *      k       : the key (array of bytes)
 *      keylen  : the length of the key
 *      seed    : startup seed
 *
 * Returns a uint64 Datum containing the hash value.
 * It is implementation-dependent what happens if keylen is 0.
 */
static Datum
hash_any_extended(const unsigned char *k, int keylen, uint64 seed)
{
    uint32_t a, b, c, len;
    uint32_t seed1 = (uint32_t) seed;
    uint32_t seed2 = (uint32_t) (seed >> 32);

    len = keylen;
    a = b = c = 0x9e3779b9 + len + seed1;
    c += seed2;

    while (len >= 12)
    {
        a += read_le32(k);
        b += read_le32(k + 4);
        c += read_le32(k + 8);
        mix(a, b, c);
        k += 12;
        len -= 12;
    }

    switch (len)
    {
        case 11:
            c += ((uint32_t) k[10]) << 24;
        case 10:
            c += ((uint32_t) k[9]) << 16;
        case 9:
            c += ((uint32_t) k[8]) << 8;
        case 8:
            b += read_le32(k + 4);
            a += read_le32(k);
            break;
        case 7:
            b += ((uint32_t) k[6]) << 16;
        case 6:
            b += ((uint32_t) k[5]) << 8;
        case 5:
            b += k[4];
        case 4:
            a += read_le32(k);
            break;
        case 3:
            a += ((uint32_t) k[2]) << 16;
        case 2:
            a += ((uint32_t) k[1]) << 8;
        case 1:
            a += k[0];
            break;
        case 0:
            return seed;
    }

    final(a, b, c);

    return (Datum) (((uint64) seed2 << 32) | c);
}
