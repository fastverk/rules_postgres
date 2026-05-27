/*
 * date_arith.c — vendored date-arithmetic bodies from Postgres 17.6's
 * `src/backend/utils/adt/date.c`.
 *
 * Function bodies are BYTE-IDENTICAL to real Postgres source (modulo
 * the standalone preamble). The `ereport(ERROR, ...)` call uses the
 * macro override in c_oracle_ereport.h, which:
 *   - captures the errcode via `errcode(...) = (fmgr_oracle_last_errcode
 *     = code, 0)`
 *   - longjmp's out via `fmgr_oracle_route_error`
 *   - the c_<fn> wrappers in wrappers.c catch with setjmp and return 0
 *
 * v1 cluster scope: 3 functions = {pli, mii, mi}.
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

#define PG_FUNCTION_ARGS         FunctionCallInfo fcinfo

/*
 * DateADT is an int32 encoding of days since epoch.
 */
typedef int32 DateADT;

/*
 * PG_GETARG_* / PG_RETURN_* — Postgres' bit-packed Datum encoding.
 * DateADT is stored as a signed int32; PG_GETARG_DATEADT is equivalent
 * to PG_GETARG_INT32.
 *
 * CRITICAL: PG_RETURN_* MUST INCLUDE THE return KEYWORD.
 * See rules_postgres/tools/stream_a/PROMPT_TEMPLATE.md for the cautionary
 * tale of int-div's omitted return statement.
 */
#define PG_GETARG_INT32(n)       ((int32) fcinfo->args[n].value)
#define PG_GETARG_DATEADT(n)     ((DateADT) fcinfo->args[n].value)
#define PG_RETURN_INT32(x)       return (Datum) (uint32) (x)
#define PG_RETURN_DATEADT(x)     return (Datum) (uint32) (x)

/*
 * ERRCODE_* macros expand to MAKE_SQLSTATE(...) per errcodes.h; our
 * c_oracle_ereport.h redefines MAKE_SQLSTATE to capture them.
 */
#define ERRCODE_DATETIME_VALUE_OUT_OF_RANGE \
    MAKE_SQLSTATE('2','2','0','0','8')

/*
 * Date sentinels: DATEVAL_NOBEGIN = INT_MIN, DATEVAL_NOEND = INT_MAX.
 * These check if a date is infinite.
 */
#define DATEVAL_NOBEGIN   (-2147483647 - 1)  /* INT_MIN */
#define DATEVAL_NOEND     2147483647         /* INT_MAX */
#define DATE_NOT_FINITE(d) \
    ((d) == DATEVAL_NOBEGIN || (d) == DATEVAL_NOEND)

/*
 * Valid date range: excludes the sentinel values.
 */
#define IS_VALID_DATE(d) \
    ((d) != DATEVAL_NOBEGIN && (d) != DATEVAL_NOEND)

#include "c_oracle_ereport.h"

/* ──────────────────────────────────────────────────────────────── */
/* Bodies below are byte-identical to real Postgres source.         */
/* date.c: date_pli, date_mii, date_mi.                             */
/* ──────────────────────────────────────────────────────────────── */

Datum
date_pli(PG_FUNCTION_ARGS)
{
	DateADT		dateVal = PG_GETARG_DATEADT(0);
	int32		days = PG_GETARG_INT32(1);
	DateADT		result;

	if (DATE_NOT_FINITE(dateVal))
		PG_RETURN_DATEADT(dateVal);

	result = dateVal + days;

	/* Check for integer overflow and out-of-allowed-range */
	if ((days >= 0 ? (result < dateVal) : (result > dateVal)) ||
		!IS_VALID_DATE(result))
		ereport(ERROR,
				(errcode(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE),
				 errmsg("date out of range")));

	PG_RETURN_DATEADT(result);
}

Datum
date_mii(PG_FUNCTION_ARGS)
{
	DateADT		dateVal = PG_GETARG_DATEADT(0);
	int32		days = PG_GETARG_INT32(1);
	DateADT		result;

	if (DATE_NOT_FINITE(dateVal))
		PG_RETURN_DATEADT(dateVal);

	result = dateVal - days;

	/* Check for integer overflow and out-of-allowed-range */
	if ((days >= 0 ? (result > dateVal) : (result < dateVal)) ||
		!IS_VALID_DATE(result))
		ereport(ERROR,
				(errcode(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE),
				 errmsg("date out of range")));

	PG_RETURN_DATEADT(result);
}

Datum
date_mi(PG_FUNCTION_ARGS)
{
	DateADT		dateVal1 = PG_GETARG_DATEADT(0);
	DateADT		dateVal2 = PG_GETARG_DATEADT(1);

	if (DATE_NOT_FINITE(dateVal1) || DATE_NOT_FINITE(dateVal2))
		ereport(ERROR,
				(errcode(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE),
				 errmsg("cannot subtract infinite dates")));

	PG_RETURN_INT32((int32) (dateVal1 - dateVal2));
}
