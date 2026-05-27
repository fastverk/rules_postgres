/*
 * int4_hash.c — vendored bodies of Postgres' integer-type hash V1 fmgr
 * functions from `src/backend/access/hash/hashfunc.c`, plus the
 * `hash_bytes_uint32` Jenkins/lookup3 hash from
 * `src/common/hashfn.c`. Self-contained — no backend dependencies.
 *
 * Sister to pg_int4_cmp's int4_cmp.c. Same pattern: vendored bodies,
 * minimum typedefs to satisfy PG_FUNCTION_ARGS / PG_GETARG_* macros,
 * compiled standalone so the diff-test can link against bit-identical
 * real-Postgres code.
 *
 * Attribution: function bodies are PostgreSQL Global Development Group,
 * released under the PostgreSQL License (BSD-style). See
 * https://www.postgresql.org/about/licence/.
 */

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

typedef unsigned int Oid;
typedef int16_t  int16;
typedef int32_t  int32;
typedef int64_t  int64;
typedef uint32_t uint32;
typedef uint64_t uint64;
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

typedef FunctionCallInfoBaseData *FunctionCallInfo;

#define PG_FUNCTION_ARGS    FunctionCallInfo fcinfo
#define PG_GETARG_CHAR(n)   ((char)  fcinfo->args[n].value)
#define PG_GETARG_INT16(n)  ((int16) fcinfo->args[n].value)
#define PG_GETARG_INT32(n)  ((int32) fcinfo->args[n].value)
#define PG_GETARG_INT64(n)  ((int64) fcinfo->args[n].value)
#define PG_GETARG_OID(n)    ((Oid)   fcinfo->args[n].value)

#define UInt32GetDatum(x)   ((Datum)(x))
#define UInt64GetDatum(x)   ((Datum)(x))

/* ── Jenkins/lookup3 building blocks (verbatim from hashfn.c) ── */

static inline uint32 pg_rotate_left32(uint32 word, int n)
{
    return (word << n) | (word >> (32 - n));
}

#define rot(x,k) pg_rotate_left32(x, k)

#define mix(a,b,c) \
{ \
  a -= c;  a ^= rot(c, 4);  c += b; \
  b -= a;  b ^= rot(a, 6);  a += c; \
  c -= b;  c ^= rot(b, 8);  b += a; \
  a -= c;  a ^= rot(c,16);  c += b; \
  b -= a;  b ^= rot(a,19);  a += c; \
  c -= b;  c ^= rot(b, 4);  b += a; \
}

#define final(a,b,c) \
{ \
  c ^= b; c -= rot(b,14); \
  a ^= c; a -= rot(c,11); \
  b ^= a; b -= rot(a,25); \
  c ^= b; c -= rot(b,16); \
  a ^= c; a -= rot(c, 4); \
  b ^= a; b -= rot(a,14); \
  c ^= b; c -= rot(b,24); \
}

/* hash_bytes_uint32 — verbatim from src/common/hashfn.c. */
uint32 hash_bytes_uint32(uint32 k)
{
    uint32 a, b, c;
    a = b = c = 0x9e3779b9 + (uint32) sizeof(uint32) + 3923095;
    a += k;
    final(a, b, c);
    return c;
}

uint64 hash_bytes_uint32_extended(uint32 k, uint64 seed)
{
    uint32 a, b, c;
    a = b = c = 0x9e3779b9 + (uint32) sizeof(uint32) + 3923095;
    if (seed != 0) {
        a += (uint32) (seed >> 32);
        b += (uint32) seed;
        mix(a, b, c);
    }
    a += k;
    final(a, b, c);
    return ((uint64) b << 32) | c;
}

/* hash_uint32 / hash_uint32_extended — inline wrappers from hashfn.h. */
static inline Datum hash_uint32(uint32 k) { return UInt32GetDatum(hash_bytes_uint32(k)); }
static inline Datum hash_uint32_extended(uint32 k, uint64 seed) { return UInt64GetDatum(hash_bytes_uint32_extended(k, seed)); }

/* ── Integer-type hash V1 fmgr bodies (verbatim from hashfunc.c) ── */

Datum hashchar(PG_FUNCTION_ARGS)
{
    return hash_uint32((int32) PG_GETARG_CHAR(0));
}

Datum hashint2(PG_FUNCTION_ARGS)
{
    return hash_uint32((int32) PG_GETARG_INT16(0));
}

Datum hashint4(PG_FUNCTION_ARGS)
{
    return hash_uint32(PG_GETARG_INT32(0));
}

Datum hashint8(PG_FUNCTION_ARGS)
{
    int64  val    = PG_GETARG_INT64(0);
    uint32 lohalf = (uint32) val;
    uint32 hihalf = (uint32) (val >> 32);

    lohalf ^= (val >= 0) ? hihalf : ~hihalf;

    return hash_uint32(lohalf);
}

Datum hashoid(PG_FUNCTION_ARGS)
{
    return hash_uint32((uint32) PG_GETARG_OID(0));
}

Datum hashenum(PG_FUNCTION_ARGS)
{
    return hash_uint32((uint32) PG_GETARG_OID(0));
}

/* ── *extended siblings (2-arg → uint64; verbatim from hashfunc.c) ── */

Datum hashcharextended(PG_FUNCTION_ARGS)
{
    return hash_uint32_extended((int32) PG_GETARG_CHAR(0), PG_GETARG_INT64(1));
}

Datum hashint2extended(PG_FUNCTION_ARGS)
{
    return hash_uint32_extended((int32) PG_GETARG_INT16(0), PG_GETARG_INT64(1));
}

Datum hashint4extended(PG_FUNCTION_ARGS)
{
    return hash_uint32_extended(PG_GETARG_INT32(0), PG_GETARG_INT64(1));
}

Datum hashint8extended(PG_FUNCTION_ARGS)
{
    int64  val    = PG_GETARG_INT64(0);
    uint32 lohalf = (uint32) val;
    uint32 hihalf = (uint32) (val >> 32);

    lohalf ^= (val >= 0) ? hihalf : ~hihalf;

    return hash_uint32_extended(lohalf, PG_GETARG_INT64(1));
}

Datum hashoidextended(PG_FUNCTION_ARGS)
{
    return hash_uint32_extended((uint32) PG_GETARG_OID(0), PG_GETARG_INT64(1));
}

Datum hashenumextended(PG_FUNCTION_ARGS)
{
    return hash_uint32_extended((uint32) PG_GETARG_OID(0), PG_GETARG_INT64(1));
}
