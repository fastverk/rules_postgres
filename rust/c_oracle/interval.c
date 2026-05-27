/*
 * interval.c — vendored interval-arithmetic bodies from Postgres 17.6's
 * `src/backend/utils/adt/timestamp.c`. Smoke-test C oracle for the
 * Pg.Ir interval cluster (first palloc-using cluster).
 *
 * v0 scope: just interval_um + interval_um_internal. Body is BYTE-
 * IDENTICAL to real Postgres source modulo:
 *   - standalone-build preamble (typedefs / macros / overflow primitives)
 *   - the `static void interval_um_internal(...)` linkage is kept exactly
 *
 * The body calls real `palloc(sizeof(Interval))`. The Rust-side
 * `pg_palloc` crate's static lib provides that symbol; both the C
 * oracle and the Rust translation allocate from the SAME
 * CurrentMemoryContext that the diff-test harness sets up.
 *
 * Attribution: function bodies are PostgreSQL Global Development Group,
 * released under the PostgreSQL License (BSD-style).
 */

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>
#include <string.h>
#include <assert.h>

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
    uint32           fncollation;
    bool             isnull;
    short            nargs;
    NullableDatum    args[FLEXIBLE_ARRAY_MEMBER];
} FunctionCallInfoBaseData;

typedef FunctionCallInfoBaseData *FunctionCallInfo;

#define PG_FUNCTION_ARGS  FunctionCallInfo fcinfo

/* Interval struct — must match pg_fcinfo::Interval layout exactly
 * (8-byte time, 4-byte day, 4-byte month; total 16 bytes). */
typedef struct {
    int64  time;
    int32  day;
    int32  month;
} Interval;

/* PG_GETARG / PG_RETURN macros for Interval — pointer-valued Datums. */
#define PG_GETARG_INTERVAL_P(n)  ((Interval *) fcinfo->args[n].value)
#define PG_RETURN_INTERVAL_P(p)  return (Datum) (uintptr_t) (p)

/* INT64CONST — Postgres' compile-time int64 literal macro. */
#define INT64CONST(x)  ((int64) x##LL)

/* Infinity sentinels and predicates — verbatim from
 * src/include/datatype/timestamp.h. */
#define PG_INT32_MIN  (-2147483647 - 1)
#define PG_INT32_MAX  2147483647
#define PG_INT64_MIN  INT64CONST(-9223372036854775807) - INT64CONST(1)
#define PG_INT64_MAX  INT64CONST(9223372036854775807)

#define INTERVAL_NOBEGIN(i)                  \
    do {                                     \
        (i)->time  = PG_INT64_MIN;           \
        (i)->day   = PG_INT32_MIN;           \
        (i)->month = PG_INT32_MIN;           \
    } while (0)

#define INTERVAL_IS_NOBEGIN(i)               \
    ((i)->month == PG_INT32_MIN && (i)->day == PG_INT32_MIN && (i)->time == PG_INT64_MIN)

#define INTERVAL_NOEND(i)                    \
    do {                                     \
        (i)->time  = PG_INT64_MAX;           \
        (i)->day   = PG_INT32_MAX;           \
        (i)->month = PG_INT32_MAX;           \
    } while (0)

#define INTERVAL_IS_NOEND(i)                 \
    ((i)->month == PG_INT32_MAX && (i)->day == PG_INT32_MAX && (i)->time == PG_INT64_MAX)

#define INTERVAL_NOT_FINITE(i) (INTERVAL_IS_NOBEGIN(i) || INTERVAL_IS_NOEND(i))

/* ERRCODE_* — the c_oracle_ereport.h shim captures these via the
 * redefined MAKE_SQLSTATE and routes to FMGR_ERR_DATETIME_VALUE_OUT_OF_RANGE. */
#define ERRCODE_DATETIME_VALUE_OUT_OF_RANGE \
    MAKE_SQLSTATE('2','2','0','0','8')

/* Assert — minimal implementation for the oracle. */
#ifndef Assert
#define Assert(condition) assert(condition)
#endif

/* Overflow primitives — same as int4_arith.c. */
static inline bool pg_add_s32_overflow(int32 a, int32 b, int32 *result) { return __builtin_add_overflow(a, b, result); }
static inline bool pg_add_s64_overflow(int64 a, int64 b, int64 *result) { return __builtin_add_overflow(a, b, result); }
static inline bool pg_sub_s32_overflow(int32 a, int32 b, int32 *result) { return __builtin_sub_overflow(a, b, result); }
static inline bool pg_sub_s64_overflow(int64 a, int64 b, int64 *result) { return __builtin_sub_overflow(a, b, result); }

/* palloc lives in the pg_palloc crate's staticlib — declared via
 * pg_palloc.h. */
#include "pg_palloc.h"

#include "c_oracle_ereport.h"

/* ──────────────────────────────────────────────────────────────── */
/* Body below is byte-identical to real Postgres source              */
/* (src/backend/utils/adt/timestamp.c, PG 17.6).                     */
/* ──────────────────────────────────────────────────────────────── */

static void
interval_um_internal(const Interval *interval, Interval *result)
{
	if (INTERVAL_IS_NOBEGIN(interval))
		INTERVAL_NOEND(result);
	else if (INTERVAL_IS_NOEND(interval))
		INTERVAL_NOBEGIN(result);
	else
	{
		/* Negate each field, guarding against overflow */
		if (pg_sub_s64_overflow(INT64CONST(0), interval->time, &result->time) ||
			pg_sub_s32_overflow(0, interval->day, &result->day) ||
			pg_sub_s32_overflow(0, interval->month, &result->month) ||
			INTERVAL_NOT_FINITE(result))
			ereport(ERROR,
					(errcode(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE),
					 errmsg("interval out of range")));
	}
}

Datum
interval_um(PG_FUNCTION_ARGS)
{
	Interval   *interval = PG_GETARG_INTERVAL_P(0);
	Interval   *result;

	result = (Interval *) palloc(sizeof(Interval));
	interval_um_internal(interval, result);

	PG_RETURN_INTERVAL_P(result);
}

/* Helpers for interval_pl and interval_mi (finite cases) */

static void
finite_interval_pl(const Interval *span1, const Interval *span2, Interval *result)
{
	Assert(!INTERVAL_NOT_FINITE(span1));
	Assert(!INTERVAL_NOT_FINITE(span2));

	if (pg_add_s32_overflow(span1->month, span2->month, &result->month) ||
		pg_add_s32_overflow(span1->day, span2->day, &result->day) ||
		pg_add_s64_overflow(span1->time, span2->time, &result->time) ||
		INTERVAL_NOT_FINITE(result))
		ereport(ERROR,
				(errcode(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE),
				 errmsg("interval out of range")));
}

static void
finite_interval_mi(const Interval *span1, const Interval *span2, Interval *result)
{
	Assert(!INTERVAL_NOT_FINITE(span1));
	Assert(!INTERVAL_NOT_FINITE(span2));

	if (pg_sub_s32_overflow(span1->month, span2->month, &result->month) ||
		pg_sub_s32_overflow(span1->day, span2->day, &result->day) ||
		pg_sub_s64_overflow(span1->time, span2->time, &result->time) ||
		INTERVAL_NOT_FINITE(result))
		ereport(ERROR,
				(errcode(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE),
				 errmsg("interval out of range")));
}

Datum
interval_pl(PG_FUNCTION_ARGS)
{
	Interval   *span1 = PG_GETARG_INTERVAL_P(0);
	Interval   *span2 = PG_GETARG_INTERVAL_P(1);
	Interval   *result;

	result = (Interval *) palloc(sizeof(Interval));

	/*
	 * Handle infinities.
	 *
	 * We treat anything that amounts to "infinity - infinity" as an error,
	 * since the interval type has nothing equivalent to NaN.
	 */
	if (INTERVAL_IS_NOBEGIN(span1))
	{
		if (INTERVAL_IS_NOEND(span2))
			ereport(ERROR,
					(errcode(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE),
					 errmsg("interval out of range")));
		else
			INTERVAL_NOBEGIN(result);
	}
	else if (INTERVAL_IS_NOEND(span1))
	{
		if (INTERVAL_IS_NOBEGIN(span2))
			ereport(ERROR,
					(errcode(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE),
					 errmsg("interval out of range")));
		else
			INTERVAL_NOEND(result);
	}
	else if (INTERVAL_NOT_FINITE(span2))
		memcpy(result, span2, sizeof(Interval));
	else
		finite_interval_pl(span1, span2, result);

	PG_RETURN_INTERVAL_P(result);
}

Datum
interval_mi(PG_FUNCTION_ARGS)
{
	Interval   *span1 = PG_GETARG_INTERVAL_P(0);
	Interval   *span2 = PG_GETARG_INTERVAL_P(1);
	Interval   *result;

	result = (Interval *) palloc(sizeof(Interval));

	/*
	 * Handle infinities.
	 *
	 * We treat anything that amounts to "infinity - infinity" as an error,
	 * since the interval type has nothing equivalent to NaN.
	 */
	if (INTERVAL_IS_NOBEGIN(span1))
	{
		if (INTERVAL_IS_NOBEGIN(span2))
			ereport(ERROR,
					(errcode(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE),
					 errmsg("interval out of range")));
		else
			INTERVAL_NOBEGIN(result);
	}
	else if (INTERVAL_IS_NOEND(span1))
	{
		if (INTERVAL_IS_NOEND(span2))
			ereport(ERROR,
					(errcode(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE),
					 errmsg("interval out of range")));
		else
			INTERVAL_NOEND(result);
	}
	else if (INTERVAL_IS_NOBEGIN(span2))
		INTERVAL_NOEND(result);
	else if (INTERVAL_IS_NOEND(span2))
		INTERVAL_NOBEGIN(result);
	else
		finite_interval_mi(span1, span2, result);

	PG_RETURN_INTERVAL_P(result);
}
