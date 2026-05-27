/*
 * Vendored integer division functions from Postgres.
 * int2div, int4div from src/backend/utils/adt/int.c
 * int8div from src/backend/utils/adt/int8.c
 *
 * These are the canonical implementations against which the Lean-emitted
 * Rust bodies are diff-tested.
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

/* Macro definitions for V1 fmgr argument access. */
#define PG_FUNCTION_ARGS         FunctionCallInfo fcinfo

#define PG_GETARG_INT16(n)  ((int16) fcinfo->args[n].value)
#define PG_GETARG_INT32(n)  ((int32) fcinfo->args[n].value)
#define PG_GETARG_INT64(n)  ((int64) fcinfo->args[n].value)

#define PG_RETURN_INT16(x)  return (Datum) (uint16_t) (x)
#define PG_RETURN_INT32(x)  return (Datum) (uint32_t) (x)
#define PG_RETURN_INT64(x)  return (Datum) (uint64_t) (x)
#define PG_RETURN_NULL()    do { fcinfo->isnull = true; return (Datum) 0; } while (0)

/* Postgres manifest constants. */
#define PG_INT16_MIN  (-32768)
#define PG_INT16_MAX  (32767)
#define PG_INT32_MIN  (-2147483647L - 1)
#define PG_INT32_MAX  (2147483647L)
#define PG_INT64_MIN  (-9223372036854775807LL - 1)
#define PG_INT64_MAX  (9223372036854775807LL)

/* Include the oracle's ereport overrides which route errors to the Rust side. */
#include "c_oracle_ereport.h"

/* ERRCODE_* macros expand to MAKE_SQLSTATE(...) per errcodes.h; our
 * c_oracle_ereport.h redefines MAKE_SQLSTATE to capture them. */
#define ERRCODE_NUMERIC_VALUE_OUT_OF_RANGE \
    MAKE_SQLSTATE('2','2','0','0','3')
#define ERRCODE_DIVISION_BY_ZERO \
    MAKE_SQLSTATE('2','2','0','1','2')

Datum
int2div(PG_FUNCTION_ARGS)
{
	int16		arg1 = PG_GETARG_INT16(0);
	int16		arg2 = PG_GETARG_INT16(1);
	int16		result;

	if (arg2 == 0)
	{
		ereport(ERROR,
				(errcode(ERRCODE_DIVISION_BY_ZERO),
				 errmsg("division by zero")));
		/* ensure compiler realizes we mustn't reach the division (gcc bug) */
		PG_RETURN_NULL();
	}

	/*
	 * SHRT_MIN / -1 is problematic, since the result can't be represented on
	 * a two's-complement machine.  Some machines produce SHRT_MIN, some
	 * produce zero, some throw an exception.  We can dodge the problem by
	 * recognizing that division by -1 is the same as negation.
	 */
	if (arg2 == -1)
	{
		if (unlikely(arg1 == PG_INT16_MIN))
			ereport(ERROR,
					(errcode(ERRCODE_NUMERIC_VALUE_OUT_OF_RANGE),
					 errmsg("smallint out of range")));
		result = -arg1;
		PG_RETURN_INT16(result);
	}

	/* No overflow is possible */

	result = arg1 / arg2;

	PG_RETURN_INT16(result);
}

Datum
int4div(PG_FUNCTION_ARGS)
{
	int32		arg1 = PG_GETARG_INT32(0);
	int32		arg2 = PG_GETARG_INT32(1);
	int32		result;

	if (arg2 == 0)
	{
		ereport(ERROR,
				(errcode(ERRCODE_DIVISION_BY_ZERO),
				 errmsg("division by zero")));
		/* ensure compiler realizes we mustn't reach the division (gcc bug) */
		PG_RETURN_NULL();
	}

	/*
	 * INT_MIN / -1 is problematic, since the result can't be represented on a
	 * two's-complement machine.  Some machines produce INT_MIN, some produce
	 * zero, some throw an exception.  We can dodge the problem by recognizing
	 * that division by -1 is the same as negation.
	 */
	if (arg2 == -1)
	{
		if (unlikely(arg1 == PG_INT32_MIN))
			ereport(ERROR,
					(errcode(ERRCODE_NUMERIC_VALUE_OUT_OF_RANGE),
					 errmsg("integer out of range")));
		result = -arg1;
		PG_RETURN_INT32(result);
	}

	/* No overflow is possible */

	result = arg1 / arg2;

	PG_RETURN_INT32(result);
}

Datum
int8div(PG_FUNCTION_ARGS)
{
	int64		arg1 = PG_GETARG_INT64(0);
	int64		arg2 = PG_GETARG_INT64(1);
	int64		result;

	if (arg2 == 0)
	{
		ereport(ERROR,
				(errcode(ERRCODE_DIVISION_BY_ZERO),
				 errmsg("division by zero")));
		/* ensure compiler realizes we mustn't reach the division (gcc bug) */
		PG_RETURN_NULL();
	}

	/*
	 * INT64_MIN / -1 is problematic, since the result can't be represented on
	 * a two's-complement machine.  Some machines produce INT64_MIN, some
	 * produce zero, some throw an exception.  We can dodge the problem by
	 * recognizing that division by -1 is the same as negation.
	 */
	if (arg2 == -1)
	{
		if (unlikely(arg1 == PG_INT64_MIN))
			ereport(ERROR,
					(errcode(ERRCODE_NUMERIC_VALUE_OUT_OF_RANGE),
					 errmsg("bigint out of range")));
		result = -arg1;
		PG_RETURN_INT64(result);
	}

	/* No overflow is possible */

	result = arg1 / arg2;

	PG_RETURN_INT64(result);
}
