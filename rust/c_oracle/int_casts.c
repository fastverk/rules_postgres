/*
 * int_casts.c — vendored integer width-cast V1 fmgr bodies from Postgres 17.6.
 * Function bodies are BYTE-IDENTICAL to real Postgres source modulo
 * the standalone preamble (matches pg_int4_arith's vendoring pattern).
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

#define PG_FUNCTION_ARGS         FunctionCallInfo fcinfo
#define PG_GETARG_INT16(n)       ((int16) fcinfo->args[n].value)
#define PG_GETARG_INT32(n)       ((int32) fcinfo->args[n].value)
#define PG_GETARG_INT64(n)       ((int64) fcinfo->args[n].value)
#define PG_RETURN_INT16(x)       return (Datum) (uint16_t) (x)
#define PG_RETURN_INT32(x)       return (Datum) (uint32_t) (x)
#define PG_RETURN_INT64(x)       return (Datum) (uint64_t) (x)

#define PG_INT16_MIN  ((int16) (-32767 - 1))
#define PG_INT16_MAX  ((int16) 32767)
#define PG_INT32_MIN  (-2147483647 - 1)
#define PG_INT32_MAX  2147483647
#define PG_INT64_MIN  ((int64) (-9223372036854775807LL - 1))
#define PG_INT64_MAX  ((int64) 9223372036854775807LL)

#define SHRT_MIN      PG_INT16_MIN
#define SHRT_MAX      PG_INT16_MAX

#define ERRCODE_NUMERIC_VALUE_OUT_OF_RANGE \
    MAKE_SQLSTATE('2','2','0','0','3')

#include "c_oracle_ereport.h"

/* ──────────────────────────────────────────────────────────────── */
/* Bodies below are byte-identical to real Postgres source.         */
/* int.c: i2toi4, i4toi2.                                            */
/* int8.c: int28, int48, int82, int84.                               */
/* ──────────────────────────────────────────────────────────────── */

Datum
i2toi4(PG_FUNCTION_ARGS)
{
	int16		arg1 = PG_GETARG_INT16(0);

	PG_RETURN_INT32((int32) arg1);
}

Datum
i4toi2(PG_FUNCTION_ARGS)
{
	int32		arg1 = PG_GETARG_INT32(0);

	if (unlikely(arg1 < SHRT_MIN) || unlikely(arg1 > SHRT_MAX))
		ereport(ERROR,
				(errcode(ERRCODE_NUMERIC_VALUE_OUT_OF_RANGE),
				 errmsg("smallint out of range")));

	PG_RETURN_INT16((int16) arg1);
}

Datum
int28(PG_FUNCTION_ARGS)
{
	int16		arg = PG_GETARG_INT16(0);

	PG_RETURN_INT64((int64) arg);
}

Datum
int48(PG_FUNCTION_ARGS)
{
	int32		arg = PG_GETARG_INT32(0);

	PG_RETURN_INT64((int64) arg);
}

Datum
int82(PG_FUNCTION_ARGS)
{
	int64		arg = PG_GETARG_INT64(0);

	if (unlikely(arg < PG_INT16_MIN) || unlikely(arg > PG_INT16_MAX))
		ereport(ERROR,
				(errcode(ERRCODE_NUMERIC_VALUE_OUT_OF_RANGE),
				 errmsg("smallint out of range")));

	PG_RETURN_INT16((int16) arg);
}

Datum
int84(PG_FUNCTION_ARGS)
{
	int64		arg = PG_GETARG_INT64(0);

	if (unlikely(arg < PG_INT32_MIN) || unlikely(arg > PG_INT32_MAX))
		ereport(ERROR,
				(errcode(ERRCODE_NUMERIC_VALUE_OUT_OF_RANGE),
				 errmsg("integer out of range")));

	PG_RETURN_INT32((int32) arg);
}
