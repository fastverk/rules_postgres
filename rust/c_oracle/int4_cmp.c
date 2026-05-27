/*
 * int4_cmp.c — vendored verbatim from Postgres 17.6
 * `src/backend/utils/adt/int.c`. Just the six int4 comparison operators
 * (int4eq, int4ne, int4lt, int4le, int4gt, int4ge), with their
 * dependencies inlined into a self-contained .c that compiles without
 * the full Postgres backend.
 *
 * Why this works standalone: comparisons don't call into any backend
 * service — no palloc, no ereport, no syscache. Just read two i32s
 * from fcinfo, compare, return bool. The functions are in the strict-
 * pure-transitive tier per the purity analysis.
 *
 * The rename shim (`#define int4eq int4eq_orig`, etc.) lives in
 * renamed_int4_cmp.c — same pattern as sha2/c_oracle/renamed_sha2.c.
 * Without the rename, the test linker silently merges the Rust crate's
 * `#[no_mangle] pub extern "C" fn int4eq` with the C oracle's, making
 * the diff-test self-equal.
 *
 * Attribution: function bodies are PostgreSQL Global Development Group,
 * released under the PostgreSQL License (BSD-style). See
 * https://www.postgresql.org/about/licence/.
 */

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

/* Minimum typedefs from postgres_ext.h / c.h / postgres.h, matching
 * what `int4eq` etc. transitively need. */
typedef unsigned int Oid;
typedef int16_t   int16;
typedef int32_t   int32;
typedef int64_t   int64;
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
typedef Datum (*PGFunction)(FunctionCallInfo fcinfo);

typedef int32     DateADT;     /* src/include/utils/date.h */
typedef int64     Timestamp;   /* src/include/datatype/timestamp.h */
typedef int64     Cash;        /* src/include/utils/cash.h */
typedef uint64_t  XLogRecPtr;  /* src/include/access/xlogdefs.h */

#define PG_FUNCTION_ARGS         FunctionCallInfo fcinfo
#define PG_GETARG_INT16(n)       ((int16) fcinfo->args[n].value)
#define PG_GETARG_INT32(n)       ((int32) fcinfo->args[n].value)
#define PG_GETARG_INT64(n)       ((int64) fcinfo->args[n].value)
#define PG_GETARG_UINT32(n)      ((uint32_t) fcinfo->args[n].value)
#define PG_GETARG_DATEADT(n)     ((DateADT) PG_GETARG_INT32(n))
#define PG_GETARG_TIMESTAMP(n)   ((Timestamp) PG_GETARG_INT64(n))
#define PG_GETARG_CASH(n)        ((Cash) PG_GETARG_INT64(n))
#define PG_GETARG_LSN(n)         ((XLogRecPtr) fcinfo->args[n].value)
#define PG_GETARG_OID(n)         ((uint32_t) fcinfo->args[n].value)
#define PG_GETARG_BOOL(n)        (((fcinfo->args[n].value) & 1) != 0)
#define PG_GETARG_CHAR(n)        ((char) fcinfo->args[n].value)
typedef uint8_t uint8;
#define PG_RETURN_BOOL(x)        return (Datum)((x) ? 1 : 0)

/* Float family. Postgres encodes f32/f64 by reinterpreting their bits
 * as the matching unsigned integer width stored in the Datum. The
 * comparison helpers (float{4,8}_<op>) are NaN-aware per Postgres
 * semantics (NaN sorts greatest, NaN == NaN). All vendored verbatim
 * from src/include/utils/float.h. */
#include <math.h>      /* isnan */
typedef float   float4;
typedef double  float8;

#define PG_GETARG_FLOAT4(n) ({ union { uint32_t u; float v; } _b; _b.u = (uint32_t) fcinfo->args[n].value; _b.v; })
#define PG_GETARG_FLOAT8(n) ({ union { uint64_t u; double v; } _b; _b.u = (uint64_t) fcinfo->args[n].value; _b.v; })

static inline bool float4_eq(float4 a, float4 b) { return isnan(a) ? isnan(b) : !isnan(b) && a == b; }
static inline bool float4_ne(float4 a, float4 b) { return isnan(a) ? !isnan(b) : isnan(b) || a != b; }
static inline bool float4_lt(float4 a, float4 b) { return !isnan(a) && (isnan(b) || a < b); }
static inline bool float4_le(float4 a, float4 b) { return isnan(b) || (!isnan(a) && a <= b); }
static inline bool float4_gt(float4 a, float4 b) { return isnan(a) ? !isnan(b) : !isnan(b) && a > b; }
static inline bool float4_ge(float4 a, float4 b) { return isnan(a) || (!isnan(b) && a >= b); }
static inline bool float8_eq(float8 a, float8 b) { return isnan(a) ? isnan(b) : !isnan(b) && a == b; }
static inline bool float8_ne(float8 a, float8 b) { return isnan(a) ? !isnan(b) : isnan(b) || a != b; }
static inline bool float8_lt(float8 a, float8 b) { return !isnan(a) && (isnan(b) || a < b); }
static inline bool float8_le(float8 a, float8 b) { return isnan(b) || (!isnan(a) && a <= b); }
static inline bool float8_gt(float8 a, float8 b) { return isnan(a) ? !isnan(b) : !isnan(b) && a > b; }
static inline bool float8_ge(float8 a, float8 b) { return isnan(a) || (!isnan(b) && a >= b); }

/* ── Vendored from src/backend/utils/adt/int.c (PG 17.6) ─────────────── */

Datum
int4eq(PG_FUNCTION_ARGS)
{
	int32		arg1 = PG_GETARG_INT32(0);
	int32		arg2 = PG_GETARG_INT32(1);

	PG_RETURN_BOOL(arg1 == arg2);
}

Datum
int4ne(PG_FUNCTION_ARGS)
{
	int32		arg1 = PG_GETARG_INT32(0);
	int32		arg2 = PG_GETARG_INT32(1);

	PG_RETURN_BOOL(arg1 != arg2);
}

Datum
int4lt(PG_FUNCTION_ARGS)
{
	int32		arg1 = PG_GETARG_INT32(0);
	int32		arg2 = PG_GETARG_INT32(1);

	PG_RETURN_BOOL(arg1 < arg2);
}

Datum
int4le(PG_FUNCTION_ARGS)
{
	int32		arg1 = PG_GETARG_INT32(0);
	int32		arg2 = PG_GETARG_INT32(1);

	PG_RETURN_BOOL(arg1 <= arg2);
}

Datum
int4gt(PG_FUNCTION_ARGS)
{
	int32		arg1 = PG_GETARG_INT32(0);
	int32		arg2 = PG_GETARG_INT32(1);

	PG_RETURN_BOOL(arg1 > arg2);
}

Datum
int4ge(PG_FUNCTION_ARGS)
{
	int32		arg1 = PG_GETARG_INT32(0);
	int32		arg2 = PG_GETARG_INT32(1);

	PG_RETURN_BOOL(arg1 >= arg2);
}

/* ── int2 cmp ops (verbatim from src/backend/utils/adt/int.c) ──────── */

Datum
int2eq(PG_FUNCTION_ARGS)
{
	int16		arg1 = PG_GETARG_INT16(0);
	int16		arg2 = PG_GETARG_INT16(1);

	PG_RETURN_BOOL(arg1 == arg2);
}

Datum
int2ne(PG_FUNCTION_ARGS)
{
	int16		arg1 = PG_GETARG_INT16(0);
	int16		arg2 = PG_GETARG_INT16(1);

	PG_RETURN_BOOL(arg1 != arg2);
}

Datum
int2lt(PG_FUNCTION_ARGS)
{
	int16		arg1 = PG_GETARG_INT16(0);
	int16		arg2 = PG_GETARG_INT16(1);

	PG_RETURN_BOOL(arg1 < arg2);
}

Datum
int2le(PG_FUNCTION_ARGS)
{
	int16		arg1 = PG_GETARG_INT16(0);
	int16		arg2 = PG_GETARG_INT16(1);

	PG_RETURN_BOOL(arg1 <= arg2);
}

Datum
int2gt(PG_FUNCTION_ARGS)
{
	int16		arg1 = PG_GETARG_INT16(0);
	int16		arg2 = PG_GETARG_INT16(1);

	PG_RETURN_BOOL(arg1 > arg2);
}

Datum
int2ge(PG_FUNCTION_ARGS)
{
	int16		arg1 = PG_GETARG_INT16(0);
	int16		arg2 = PG_GETARG_INT16(1);

	PG_RETURN_BOOL(arg1 >= arg2);
}

/* ── int8 cmp ops (verbatim from src/backend/utils/adt/int8.c) ─────── */

Datum
int8eq(PG_FUNCTION_ARGS)
{
	int64		val1 = PG_GETARG_INT64(0);
	int64		val2 = PG_GETARG_INT64(1);

	PG_RETURN_BOOL(val1 == val2);
}

Datum
int8ne(PG_FUNCTION_ARGS)
{
	int64		val1 = PG_GETARG_INT64(0);
	int64		val2 = PG_GETARG_INT64(1);

	PG_RETURN_BOOL(val1 != val2);
}

Datum
int8lt(PG_FUNCTION_ARGS)
{
	int64		val1 = PG_GETARG_INT64(0);
	int64		val2 = PG_GETARG_INT64(1);

	PG_RETURN_BOOL(val1 < val2);
}

Datum
int8le(PG_FUNCTION_ARGS)
{
	int64		val1 = PG_GETARG_INT64(0);
	int64		val2 = PG_GETARG_INT64(1);

	PG_RETURN_BOOL(val1 <= val2);
}

Datum
int8gt(PG_FUNCTION_ARGS)
{
	int64		val1 = PG_GETARG_INT64(0);
	int64		val2 = PG_GETARG_INT64(1);

	PG_RETURN_BOOL(val1 > val2);
}

Datum
int8ge(PG_FUNCTION_ARGS)
{
	int64		val1 = PG_GETARG_INT64(0);
	int64		val2 = PG_GETARG_INT64(1);

	PG_RETURN_BOOL(val1 >= val2);
}

/* ── date cmp ops (verbatim from src/backend/utils/adt/date.c) ─────── */

Datum date_eq(PG_FUNCTION_ARGS) {
	DateADT a = PG_GETARG_DATEADT(0);
	DateADT b = PG_GETARG_DATEADT(1);
	PG_RETURN_BOOL(a == b);
}
Datum date_ne(PG_FUNCTION_ARGS) {
	DateADT a = PG_GETARG_DATEADT(0);
	DateADT b = PG_GETARG_DATEADT(1);
	PG_RETURN_BOOL(a != b);
}
Datum date_lt(PG_FUNCTION_ARGS) {
	DateADT a = PG_GETARG_DATEADT(0);
	DateADT b = PG_GETARG_DATEADT(1);
	PG_RETURN_BOOL(a < b);
}
Datum date_le(PG_FUNCTION_ARGS) {
	DateADT a = PG_GETARG_DATEADT(0);
	DateADT b = PG_GETARG_DATEADT(1);
	PG_RETURN_BOOL(a <= b);
}
Datum date_gt(PG_FUNCTION_ARGS) {
	DateADT a = PG_GETARG_DATEADT(0);
	DateADT b = PG_GETARG_DATEADT(1);
	PG_RETURN_BOOL(a > b);
}
Datum date_ge(PG_FUNCTION_ARGS) {
	DateADT a = PG_GETARG_DATEADT(0);
	DateADT b = PG_GETARG_DATEADT(1);
	PG_RETURN_BOOL(a >= b);
}

/* ── timestamp cmp ops (semantics from src/backend/utils/adt/timestamp.c)
 * The real impl routes through timestamp_cmp_internal which is just
 * (dt1 < dt2) ? -1 : ((dt1 > dt2) ? 1 : 0); we inline the equivalent
 * direct comparison here. Datum-level behavior is bit-identical. */

Datum timestamp_eq(PG_FUNCTION_ARGS) {
	Timestamp a = PG_GETARG_TIMESTAMP(0);
	Timestamp b = PG_GETARG_TIMESTAMP(1);
	PG_RETURN_BOOL(a == b);
}
Datum timestamp_ne(PG_FUNCTION_ARGS) {
	Timestamp a = PG_GETARG_TIMESTAMP(0);
	Timestamp b = PG_GETARG_TIMESTAMP(1);
	PG_RETURN_BOOL(a != b);
}
Datum timestamp_lt(PG_FUNCTION_ARGS) {
	Timestamp a = PG_GETARG_TIMESTAMP(0);
	Timestamp b = PG_GETARG_TIMESTAMP(1);
	PG_RETURN_BOOL(a < b);
}
Datum timestamp_le(PG_FUNCTION_ARGS) {
	Timestamp a = PG_GETARG_TIMESTAMP(0);
	Timestamp b = PG_GETARG_TIMESTAMP(1);
	PG_RETURN_BOOL(a <= b);
}
Datum timestamp_gt(PG_FUNCTION_ARGS) {
	Timestamp a = PG_GETARG_TIMESTAMP(0);
	Timestamp b = PG_GETARG_TIMESTAMP(1);
	PG_RETURN_BOOL(a > b);
}
Datum timestamp_ge(PG_FUNCTION_ARGS) {
	Timestamp a = PG_GETARG_TIMESTAMP(0);
	Timestamp b = PG_GETARG_TIMESTAMP(1);
	PG_RETURN_BOOL(a >= b);
}

/* ── cash cmp ops (verbatim from src/backend/utils/adt/cash.c) ─────── */

Datum cash_eq(PG_FUNCTION_ARGS) {
	Cash c1 = PG_GETARG_CASH(0);
	Cash c2 = PG_GETARG_CASH(1);
	PG_RETURN_BOOL(c1 == c2);
}
Datum cash_ne(PG_FUNCTION_ARGS) {
	Cash c1 = PG_GETARG_CASH(0);
	Cash c2 = PG_GETARG_CASH(1);
	PG_RETURN_BOOL(c1 != c2);
}
Datum cash_lt(PG_FUNCTION_ARGS) {
	Cash c1 = PG_GETARG_CASH(0);
	Cash c2 = PG_GETARG_CASH(1);
	PG_RETURN_BOOL(c1 < c2);
}
Datum cash_le(PG_FUNCTION_ARGS) {
	Cash c1 = PG_GETARG_CASH(0);
	Cash c2 = PG_GETARG_CASH(1);
	PG_RETURN_BOOL(c1 <= c2);
}
Datum cash_gt(PG_FUNCTION_ARGS) {
	Cash c1 = PG_GETARG_CASH(0);
	Cash c2 = PG_GETARG_CASH(1);
	PG_RETURN_BOOL(c1 > c2);
}
Datum cash_ge(PG_FUNCTION_ARGS) {
	Cash c1 = PG_GETARG_CASH(0);
	Cash c2 = PG_GETARG_CASH(1);
	PG_RETURN_BOOL(c1 >= c2);
}

/* ── pg_lsn cmp ops (verbatim from src/backend/utils/adt/pg_lsn.c) ─── */

Datum pg_lsn_eq(PG_FUNCTION_ARGS) {
	XLogRecPtr lsn1 = PG_GETARG_LSN(0);
	XLogRecPtr lsn2 = PG_GETARG_LSN(1);
	PG_RETURN_BOOL(lsn1 == lsn2);
}
Datum pg_lsn_ne(PG_FUNCTION_ARGS) {
	XLogRecPtr lsn1 = PG_GETARG_LSN(0);
	XLogRecPtr lsn2 = PG_GETARG_LSN(1);
	PG_RETURN_BOOL(lsn1 != lsn2);
}
Datum pg_lsn_lt(PG_FUNCTION_ARGS) {
	XLogRecPtr lsn1 = PG_GETARG_LSN(0);
	XLogRecPtr lsn2 = PG_GETARG_LSN(1);
	PG_RETURN_BOOL(lsn1 < lsn2);
}
Datum pg_lsn_le(PG_FUNCTION_ARGS) {
	XLogRecPtr lsn1 = PG_GETARG_LSN(0);
	XLogRecPtr lsn2 = PG_GETARG_LSN(1);
	PG_RETURN_BOOL(lsn1 <= lsn2);
}
Datum pg_lsn_gt(PG_FUNCTION_ARGS) {
	XLogRecPtr lsn1 = PG_GETARG_LSN(0);
	XLogRecPtr lsn2 = PG_GETARG_LSN(1);
	PG_RETURN_BOOL(lsn1 > lsn2);
}
Datum pg_lsn_ge(PG_FUNCTION_ARGS) {
	XLogRecPtr lsn1 = PG_GETARG_LSN(0);
	XLogRecPtr lsn2 = PG_GETARG_LSN(1);
	PG_RETURN_BOOL(lsn1 >= lsn2);
}

/* ── oid cmp ops (verbatim from src/backend/utils/adt/oid.c) ───────── */

Datum oideq(PG_FUNCTION_ARGS) {
	uint32_t arg1 = PG_GETARG_OID(0);
	uint32_t arg2 = PG_GETARG_OID(1);
	PG_RETURN_BOOL(arg1 == arg2);
}
Datum oidne(PG_FUNCTION_ARGS) {
	uint32_t arg1 = PG_GETARG_OID(0);
	uint32_t arg2 = PG_GETARG_OID(1);
	PG_RETURN_BOOL(arg1 != arg2);
}
Datum oidlt(PG_FUNCTION_ARGS) {
	uint32_t arg1 = PG_GETARG_OID(0);
	uint32_t arg2 = PG_GETARG_OID(1);
	PG_RETURN_BOOL(arg1 < arg2);
}
Datum oidle(PG_FUNCTION_ARGS) {
	uint32_t arg1 = PG_GETARG_OID(0);
	uint32_t arg2 = PG_GETARG_OID(1);
	PG_RETURN_BOOL(arg1 <= arg2);
}
Datum oidgt(PG_FUNCTION_ARGS) {
	uint32_t arg1 = PG_GETARG_OID(0);
	uint32_t arg2 = PG_GETARG_OID(1);
	PG_RETURN_BOOL(arg1 > arg2);
}
Datum oidge(PG_FUNCTION_ARGS) {
	uint32_t arg1 = PG_GETARG_OID(0);
	uint32_t arg2 = PG_GETARG_OID(1);
	PG_RETURN_BOOL(arg1 >= arg2);
}

/* ── bool cmp ops (verbatim from src/backend/utils/adt/bool.c) ─────── */

Datum booleq(PG_FUNCTION_ARGS) {
	bool arg1 = PG_GETARG_BOOL(0);
	bool arg2 = PG_GETARG_BOOL(1);
	PG_RETURN_BOOL(arg1 == arg2);
}
Datum boolne(PG_FUNCTION_ARGS) {
	bool arg1 = PG_GETARG_BOOL(0);
	bool arg2 = PG_GETARG_BOOL(1);
	PG_RETURN_BOOL(arg1 != arg2);
}
Datum boollt(PG_FUNCTION_ARGS) {
	bool arg1 = PG_GETARG_BOOL(0);
	bool arg2 = PG_GETARG_BOOL(1);
	PG_RETURN_BOOL(arg1 < arg2);
}
Datum boolle(PG_FUNCTION_ARGS) {
	bool arg1 = PG_GETARG_BOOL(0);
	bool arg2 = PG_GETARG_BOOL(1);
	PG_RETURN_BOOL(arg1 <= arg2);
}
Datum boolgt(PG_FUNCTION_ARGS) {
	bool arg1 = PG_GETARG_BOOL(0);
	bool arg2 = PG_GETARG_BOOL(1);
	PG_RETURN_BOOL(arg1 > arg2);
}
Datum boolge(PG_FUNCTION_ARGS) {
	bool arg1 = PG_GETARG_BOOL(0);
	bool arg2 = PG_GETARG_BOOL(1);
	PG_RETURN_BOOL(arg1 >= arg2);
}

/* ── float4 V1 fmgr cmp ops (vendored shape from src/backend/utils/adt/float.c) ── */

Datum float4eq(PG_FUNCTION_ARGS) { float4 a = PG_GETARG_FLOAT4(0); float4 b = PG_GETARG_FLOAT4(1); PG_RETURN_BOOL(float4_eq(a, b)); }
Datum float4ne(PG_FUNCTION_ARGS) { float4 a = PG_GETARG_FLOAT4(0); float4 b = PG_GETARG_FLOAT4(1); PG_RETURN_BOOL(float4_ne(a, b)); }
Datum float4lt(PG_FUNCTION_ARGS) { float4 a = PG_GETARG_FLOAT4(0); float4 b = PG_GETARG_FLOAT4(1); PG_RETURN_BOOL(float4_lt(a, b)); }
Datum float4le(PG_FUNCTION_ARGS) { float4 a = PG_GETARG_FLOAT4(0); float4 b = PG_GETARG_FLOAT4(1); PG_RETURN_BOOL(float4_le(a, b)); }
Datum float4gt(PG_FUNCTION_ARGS) { float4 a = PG_GETARG_FLOAT4(0); float4 b = PG_GETARG_FLOAT4(1); PG_RETURN_BOOL(float4_gt(a, b)); }
Datum float4ge(PG_FUNCTION_ARGS) { float4 a = PG_GETARG_FLOAT4(0); float4 b = PG_GETARG_FLOAT4(1); PG_RETURN_BOOL(float4_ge(a, b)); }

Datum float8eq(PG_FUNCTION_ARGS) { float8 a = PG_GETARG_FLOAT8(0); float8 b = PG_GETARG_FLOAT8(1); PG_RETURN_BOOL(float8_eq(a, b)); }
Datum float8ne(PG_FUNCTION_ARGS) { float8 a = PG_GETARG_FLOAT8(0); float8 b = PG_GETARG_FLOAT8(1); PG_RETURN_BOOL(float8_ne(a, b)); }
Datum float8lt(PG_FUNCTION_ARGS) { float8 a = PG_GETARG_FLOAT8(0); float8 b = PG_GETARG_FLOAT8(1); PG_RETURN_BOOL(float8_lt(a, b)); }
Datum float8le(PG_FUNCTION_ARGS) { float8 a = PG_GETARG_FLOAT8(0); float8 b = PG_GETARG_FLOAT8(1); PG_RETURN_BOOL(float8_le(a, b)); }
Datum float8gt(PG_FUNCTION_ARGS) { float8 a = PG_GETARG_FLOAT8(0); float8 b = PG_GETARG_FLOAT8(1); PG_RETURN_BOOL(float8_gt(a, b)); }
Datum float8ge(PG_FUNCTION_ARGS) { float8 a = PG_GETARG_FLOAT8(0); float8 b = PG_GETARG_FLOAT8(1); PG_RETURN_BOOL(float8_ge(a, b)); }

/* ── char cmp ops (verbatim from src/backend/utils/adt/char.c) ─────── */

Datum chareq(PG_FUNCTION_ARGS) { char a = PG_GETARG_CHAR(0); char b = PG_GETARG_CHAR(1); PG_RETURN_BOOL(a == b); }
Datum charne(PG_FUNCTION_ARGS) { char a = PG_GETARG_CHAR(0); char b = PG_GETARG_CHAR(1); PG_RETURN_BOOL(a != b); }
Datum charlt(PG_FUNCTION_ARGS) { char a = PG_GETARG_CHAR(0); char b = PG_GETARG_CHAR(1); PG_RETURN_BOOL((uint8) a < (uint8) b); }
Datum charle(PG_FUNCTION_ARGS) { char a = PG_GETARG_CHAR(0); char b = PG_GETARG_CHAR(1); PG_RETURN_BOOL((uint8) a <= (uint8) b); }
Datum chargt(PG_FUNCTION_ARGS) { char a = PG_GETARG_CHAR(0); char b = PG_GETARG_CHAR(1); PG_RETURN_BOOL((uint8) a > (uint8) b); }
Datum charge(PG_FUNCTION_ARGS) { char a = PG_GETARG_CHAR(0); char b = PG_GETARG_CHAR(1); PG_RETURN_BOOL((uint8) a >= (uint8) b); }

/* ── cross-type int cmp ops (verbatim from src/backend/utils/adt/int.c, int8.c) ── */

Datum int24eq(PG_FUNCTION_ARGS) { int16 a = PG_GETARG_INT16(0); int32 b = PG_GETARG_INT32(1); PG_RETURN_BOOL(a == b); }
Datum int24ne(PG_FUNCTION_ARGS) { int16 a = PG_GETARG_INT16(0); int32 b = PG_GETARG_INT32(1); PG_RETURN_BOOL(a != b); }
Datum int24lt(PG_FUNCTION_ARGS) { int16 a = PG_GETARG_INT16(0); int32 b = PG_GETARG_INT32(1); PG_RETURN_BOOL(a < b); }
Datum int24le(PG_FUNCTION_ARGS) { int16 a = PG_GETARG_INT16(0); int32 b = PG_GETARG_INT32(1); PG_RETURN_BOOL(a <= b); }
Datum int24gt(PG_FUNCTION_ARGS) { int16 a = PG_GETARG_INT16(0); int32 b = PG_GETARG_INT32(1); PG_RETURN_BOOL(a > b); }
Datum int24ge(PG_FUNCTION_ARGS) { int16 a = PG_GETARG_INT16(0); int32 b = PG_GETARG_INT32(1); PG_RETURN_BOOL(a >= b); }

Datum int42eq(PG_FUNCTION_ARGS) { int32 a = PG_GETARG_INT32(0); int16 b = PG_GETARG_INT16(1); PG_RETURN_BOOL(a == b); }
Datum int42ne(PG_FUNCTION_ARGS) { int32 a = PG_GETARG_INT32(0); int16 b = PG_GETARG_INT16(1); PG_RETURN_BOOL(a != b); }
Datum int42lt(PG_FUNCTION_ARGS) { int32 a = PG_GETARG_INT32(0); int16 b = PG_GETARG_INT16(1); PG_RETURN_BOOL(a < b); }
Datum int42le(PG_FUNCTION_ARGS) { int32 a = PG_GETARG_INT32(0); int16 b = PG_GETARG_INT16(1); PG_RETURN_BOOL(a <= b); }
Datum int42gt(PG_FUNCTION_ARGS) { int32 a = PG_GETARG_INT32(0); int16 b = PG_GETARG_INT16(1); PG_RETURN_BOOL(a > b); }
Datum int42ge(PG_FUNCTION_ARGS) { int32 a = PG_GETARG_INT32(0); int16 b = PG_GETARG_INT16(1); PG_RETURN_BOOL(a >= b); }

Datum int28eq(PG_FUNCTION_ARGS) { int16 a = PG_GETARG_INT16(0); int64 b = PG_GETARG_INT64(1); PG_RETURN_BOOL(a == b); }
Datum int28ne(PG_FUNCTION_ARGS) { int16 a = PG_GETARG_INT16(0); int64 b = PG_GETARG_INT64(1); PG_RETURN_BOOL(a != b); }
Datum int28lt(PG_FUNCTION_ARGS) { int16 a = PG_GETARG_INT16(0); int64 b = PG_GETARG_INT64(1); PG_RETURN_BOOL(a < b); }
Datum int28le(PG_FUNCTION_ARGS) { int16 a = PG_GETARG_INT16(0); int64 b = PG_GETARG_INT64(1); PG_RETURN_BOOL(a <= b); }
Datum int28gt(PG_FUNCTION_ARGS) { int16 a = PG_GETARG_INT16(0); int64 b = PG_GETARG_INT64(1); PG_RETURN_BOOL(a > b); }
Datum int28ge(PG_FUNCTION_ARGS) { int16 a = PG_GETARG_INT16(0); int64 b = PG_GETARG_INT64(1); PG_RETURN_BOOL(a >= b); }

Datum int82eq(PG_FUNCTION_ARGS) { int64 a = PG_GETARG_INT64(0); int16 b = PG_GETARG_INT16(1); PG_RETURN_BOOL(a == b); }
Datum int82ne(PG_FUNCTION_ARGS) { int64 a = PG_GETARG_INT64(0); int16 b = PG_GETARG_INT16(1); PG_RETURN_BOOL(a != b); }
Datum int82lt(PG_FUNCTION_ARGS) { int64 a = PG_GETARG_INT64(0); int16 b = PG_GETARG_INT16(1); PG_RETURN_BOOL(a < b); }
Datum int82le(PG_FUNCTION_ARGS) { int64 a = PG_GETARG_INT64(0); int16 b = PG_GETARG_INT16(1); PG_RETURN_BOOL(a <= b); }
Datum int82gt(PG_FUNCTION_ARGS) { int64 a = PG_GETARG_INT64(0); int16 b = PG_GETARG_INT16(1); PG_RETURN_BOOL(a > b); }
Datum int82ge(PG_FUNCTION_ARGS) { int64 a = PG_GETARG_INT64(0); int16 b = PG_GETARG_INT16(1); PG_RETURN_BOOL(a >= b); }

Datum int48eq(PG_FUNCTION_ARGS) { int32 a = PG_GETARG_INT32(0); int64 b = PG_GETARG_INT64(1); PG_RETURN_BOOL(a == b); }
Datum int48ne(PG_FUNCTION_ARGS) { int32 a = PG_GETARG_INT32(0); int64 b = PG_GETARG_INT64(1); PG_RETURN_BOOL(a != b); }
Datum int48lt(PG_FUNCTION_ARGS) { int32 a = PG_GETARG_INT32(0); int64 b = PG_GETARG_INT64(1); PG_RETURN_BOOL(a < b); }
Datum int48le(PG_FUNCTION_ARGS) { int32 a = PG_GETARG_INT32(0); int64 b = PG_GETARG_INT64(1); PG_RETURN_BOOL(a <= b); }
Datum int48gt(PG_FUNCTION_ARGS) { int32 a = PG_GETARG_INT32(0); int64 b = PG_GETARG_INT64(1); PG_RETURN_BOOL(a > b); }
Datum int48ge(PG_FUNCTION_ARGS) { int32 a = PG_GETARG_INT32(0); int64 b = PG_GETARG_INT64(1); PG_RETURN_BOOL(a >= b); }

Datum int84eq(PG_FUNCTION_ARGS) { int64 a = PG_GETARG_INT64(0); int32 b = PG_GETARG_INT32(1); PG_RETURN_BOOL(a == b); }
Datum int84ne(PG_FUNCTION_ARGS) { int64 a = PG_GETARG_INT64(0); int32 b = PG_GETARG_INT32(1); PG_RETURN_BOOL(a != b); }
Datum int84lt(PG_FUNCTION_ARGS) { int64 a = PG_GETARG_INT64(0); int32 b = PG_GETARG_INT32(1); PG_RETURN_BOOL(a < b); }
Datum int84le(PG_FUNCTION_ARGS) { int64 a = PG_GETARG_INT64(0); int32 b = PG_GETARG_INT32(1); PG_RETURN_BOOL(a <= b); }
Datum int84gt(PG_FUNCTION_ARGS) { int64 a = PG_GETARG_INT64(0); int32 b = PG_GETARG_INT32(1); PG_RETURN_BOOL(a > b); }
Datum int84ge(PG_FUNCTION_ARGS) { int64 a = PG_GETARG_INT64(0); int32 b = PG_GETARG_INT32(1); PG_RETURN_BOOL(a >= b); }
