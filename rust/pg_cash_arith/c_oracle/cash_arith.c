/*
 * cash_arith.c — vendored cash-arithmetic bodies from Postgres
 * 17.6's `src/backend/utils/adt/cash.c`.
 *
 * Function bodies are BYTE-IDENTICAL to real Postgres source (modulo
 * the standalone preamble). The `ereport(ERROR, ...)` call uses the
 * macro override in c_oracle_ereport.h, which:
 *   - captures the errcode via `errcode(...) = (fmgr_oracle_last_errcode
 *     = code, 0)`
 *   - longjmp's out via `fmgr_oracle_route_error`
 *   - the c_<fn> wrappers in wrappers.c catch with setjmp and return 0
 *
 * v1 cluster scope: 6 functions = {pl, mi, mul_int4, div_int4, larger,
 * smaller}. Cash is typedef'd as int64 throughout.
 *
 * Attribution: function bodies are PostgreSQL Global Development Group,
 * released under the PostgreSQL License (BSD-style).
 */

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

typedef int16_t  int16;
typedef int32_t  int32;
typedef int64_t  int64;
typedef int64_t  Cash;
typedef uint32_t uint32;
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

#define PG_FUNCTION_ARGS         FunctionCallInfo fcinfo

/*
 * PG_GETARG_* / PG_RETURN_* — Postgres' bit-packed Datum encoding.
 * Cash is int64 under the hood; all values stored in low bits of 64-bit
 * Datum. Mirrors pg_fcinfo's decode/encode helpers.
 */
#define PG_GETARG_CASH(n)        ((Cash) fcinfo->args[n].value)
#define PG_GETARG_INT32(n)       ((int32) fcinfo->args[n].value)
#define PG_RETURN_CASH(x)        return (Datum) (uint64_t) (x)

/* ERRCODE_* macros expand to MAKE_SQLSTATE(...) per errcodes.h; our
 * c_oracle_ereport.h redefines MAKE_SQLSTATE to capture them. */
#define ERRCODE_NUMERIC_VALUE_OUT_OF_RANGE \
    MAKE_SQLSTATE('2','2','0','0','3')
#define ERRCODE_DIVISION_BY_ZERO \
    MAKE_SQLSTATE('2','2','0','1','2')

/*
 * Overflow primitives. Postgres uses __builtin_*_overflow when
 * available, which is identical to what Rust's `checked_*` lower to.
 * Both clang and gcc define HAVE__BUILTIN_OP_OVERFLOW on every modern
 * platform, so this matches what real Postgres compiles to.
 */
static inline bool pg_add_s64_overflow(int64 a, int64 b, int64 *result) { return __builtin_add_overflow(a, b, result); }
static inline bool pg_sub_s64_overflow(int64 a, int64 b, int64 *result) { return __builtin_sub_overflow(a, b, result); }
static inline bool pg_mul_s64_overflow(int64 a, int64 b, int64 *result) { return __builtin_mul_overflow(a, b, result); }

#include "c_oracle_ereport.h"

/* ──────────────────────────────────────────────────────────────── */
/* Bodies below are byte-identical to real Postgres source.         */
/* src/backend/utils/adt/cash.c: all 6 functions.                   */
/* ──────────────────────────────────────────────────────────────── */

Datum
cash_pl(PG_FUNCTION_ARGS)
{
	Cash		c1 = PG_GETARG_CASH(0);
	Cash		c2 = PG_GETARG_CASH(1);
	Cash		result;

	if (unlikely(pg_add_s64_overflow(c1, c2, &result)))
		ereport(ERROR,
				(errcode(ERRCODE_NUMERIC_VALUE_OUT_OF_RANGE),
				 errmsg("money out of range")));
	PG_RETURN_CASH(result);
}

Datum
cash_mi(PG_FUNCTION_ARGS)
{
	Cash		c1 = PG_GETARG_CASH(0);
	Cash		c2 = PG_GETARG_CASH(1);
	Cash		result;

	if (unlikely(pg_sub_s64_overflow(c1, c2, &result)))
		ereport(ERROR,
				(errcode(ERRCODE_NUMERIC_VALUE_OUT_OF_RANGE),
				 errmsg("money out of range")));
	PG_RETURN_CASH(result);
}

Datum
cash_mul_int4(PG_FUNCTION_ARGS)
{
	Cash		c = PG_GETARG_CASH(0);
	int32		i = PG_GETARG_INT32(1);
	Cash		result;

	if (unlikely(pg_mul_s64_overflow(c, (int64) i, &result)))
		ereport(ERROR,
				(errcode(ERRCODE_NUMERIC_VALUE_OUT_OF_RANGE),
				 errmsg("money out of range")));
	PG_RETURN_CASH(result);
}

Datum
cash_div_int4(PG_FUNCTION_ARGS)
{
	Cash		c = PG_GETARG_CASH(0);
	int32		i = PG_GETARG_INT32(1);

	if (unlikely(i == 0))
		ereport(ERROR,
				(errcode(ERRCODE_DIVISION_BY_ZERO),
				 errmsg("division by zero")));
	PG_RETURN_CASH(c / (int64) i);
}

Datum
cashlarger(PG_FUNCTION_ARGS)
{
	Cash		c1 = PG_GETARG_CASH(0);
	Cash		c2 = PG_GETARG_CASH(1);
	Cash		result;

	result = (c1 > c2) ? c1 : c2;

	PG_RETURN_CASH(result);
}

Datum
cashsmaller(PG_FUNCTION_ARGS)
{
	Cash		c1 = PG_GETARG_CASH(0);
	Cash		c2 = PG_GETARG_CASH(1);
	Cash		result;

	result = (c1 < c2) ? c1 : c2;

	PG_RETURN_CASH(result);
}
