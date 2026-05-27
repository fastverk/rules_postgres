/*
 * float_unary.c — vendored float unary V1 fmgr bodies from Postgres 17.6.
 * Function bodies are BYTE-IDENTICAL to real Postgres source modulo
 * the standalone preamble (matches pg_int4_unary's vendoring pattern).
 *
 * Attribution: function bodies are PostgreSQL Global Development Group,
 * released under the PostgreSQL License (BSD-style).
 */

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>
#include <math.h>

typedef float float4;
typedef double float8;
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
    uint32_t         fncollation;
    bool             isnull;
    short            nargs;
    NullableDatum    args[FLEXIBLE_ARRAY_MEMBER];
} FunctionCallInfoBaseData;

typedef FunctionCallInfoBaseData *FunctionCallInfo;

#define PG_FUNCTION_ARGS         FunctionCallInfo fcinfo
#define PG_GETARG_FLOAT4(n)      (*(float4 *)&fcinfo->args[n].value)
#define PG_GETARG_FLOAT8(n)      (*(float8 *)&fcinfo->args[n].value)
#define PG_RETURN_FLOAT4(x)      do { float4 _tmp = (x); return (Datum) (uint32_t) (*(uint32_t *) &_tmp); } while(0)
#define PG_RETURN_FLOAT8(x)      do { float8 _tmp = (x); return (Datum) (uint64_t) (*(uint64_t *) &_tmp); } while(0)

/* ──────────────────────────────────────────────────────────────── */
/* Bodies below are byte-identical to real Postgres source.         */
/* float.c: float4abs, float4um, float8abs, float8um.               */
/* ──────────────────────────────────────────────────────────────── */

Datum
float4abs(PG_FUNCTION_ARGS)
{
	float4		arg1 = PG_GETARG_FLOAT4(0);

	PG_RETURN_FLOAT4(fabsf(arg1));
}

Datum
float4um(PG_FUNCTION_ARGS)
{
	float4		arg1 = PG_GETARG_FLOAT4(0);
	float4		result;

	result = -arg1;
	PG_RETURN_FLOAT4(result);
}

Datum
float8abs(PG_FUNCTION_ARGS)
{
	float8		arg1 = PG_GETARG_FLOAT8(0);

	PG_RETURN_FLOAT8(fabs(arg1));
}

Datum
float8um(PG_FUNCTION_ARGS)
{
	float8		arg1 = PG_GETARG_FLOAT8(0);
	float8		result;

	result = -arg1;
	PG_RETURN_FLOAT8(result);
}
