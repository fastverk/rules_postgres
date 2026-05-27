/*
 * float_arith.c — vendored float arithmetic V1 fmgr bodies from Postgres 17.6.
 * Function bodies are BYTE-IDENTICAL to real Postgres source modulo
 * the standalone preamble (matches pg_int4_arith's vendoring pattern).
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

#define ERRCODE_NUMERIC_VALUE_OUT_OF_RANGE \
    MAKE_SQLSTATE('2','2','0','0','3')
#define ERRCODE_DIVISION_BY_ZERO \
    MAKE_SQLSTATE('2','2','0','1','2')

#include "c_oracle_ereport.h"

/* Forward declarations of helper functions from utils/float.h */
static inline float4 float4_pl(const float4 val1, const float4 val2);
static inline float8 float8_pl(const float8 val1, const float8 val2);
static inline float4 float4_mi(const float4 val1, const float4 val2);
static inline float8 float8_mi(const float8 val1, const float8 val2);
static inline float4 float4_mul(const float4 val1, const float4 val2);
static inline float8 float8_mul(const float8 val1, const float8 val2);
static inline float4 float4_div(const float4 val1, const float4 val2);
static inline float8 float8_div(const float8 val1, const float8 val2);

/* Stubs for error functions — the ereport macros will route to pg_fcinfo_set_error */
static inline void float_overflow_error(void) {
    ereport(ERROR, (errcode(ERRCODE_NUMERIC_VALUE_OUT_OF_RANGE),
                    errmsg("value out of range: overflow")));
}
static inline void float_underflow_error(void) {
    ereport(ERROR, (errcode(ERRCODE_NUMERIC_VALUE_OUT_OF_RANGE),
                    errmsg("value out of range: underflow")));
}
static inline void float_zero_divide_error(void) {
    ereport(ERROR, (errcode(ERRCODE_DIVISION_BY_ZERO),
                    errmsg("division by zero")));
}

/* ──────────────────────────────────────────────────────────────── */
/* Bodies below are byte-identical to real Postgres source.         */
/* float.c: float4pl, float4mi, float4mul, float4div,              */
/*          float8pl, float8mi, float8mul, float8div.              */
/* ──────────────────────────────────────────────────────────────── */

Datum
float4pl(PG_FUNCTION_ARGS)
{
	float4		arg1 = PG_GETARG_FLOAT4(0);
	float4		arg2 = PG_GETARG_FLOAT4(1);

	PG_RETURN_FLOAT4(float4_pl(arg1, arg2));
}

Datum
float4mi(PG_FUNCTION_ARGS)
{
	float4		arg1 = PG_GETARG_FLOAT4(0);
	float4		arg2 = PG_GETARG_FLOAT4(1);

	PG_RETURN_FLOAT4(float4_mi(arg1, arg2));
}

Datum
float4mul(PG_FUNCTION_ARGS)
{
	float4		arg1 = PG_GETARG_FLOAT4(0);
	float4		arg2 = PG_GETARG_FLOAT4(1);

	PG_RETURN_FLOAT4(float4_mul(arg1, arg2));
}

Datum
float4div(PG_FUNCTION_ARGS)
{
	float4		arg1 = PG_GETARG_FLOAT4(0);
	float4		arg2 = PG_GETARG_FLOAT4(1);

	PG_RETURN_FLOAT4(float4_div(arg1, arg2));
}

Datum
float8pl(PG_FUNCTION_ARGS)
{
	float8		arg1 = PG_GETARG_FLOAT8(0);
	float8		arg2 = PG_GETARG_FLOAT8(1);

	PG_RETURN_FLOAT8(float8_pl(arg1, arg2));
}

Datum
float8mi(PG_FUNCTION_ARGS)
{
	float8		arg1 = PG_GETARG_FLOAT8(0);
	float8		arg2 = PG_GETARG_FLOAT8(1);

	PG_RETURN_FLOAT8(float8_mi(arg1, arg2));
}

Datum
float8mul(PG_FUNCTION_ARGS)
{
	float8		arg1 = PG_GETARG_FLOAT8(0);
	float8		arg2 = PG_GETARG_FLOAT8(1);

	PG_RETURN_FLOAT8(float8_mul(arg1, arg2));
}

Datum
float8div(PG_FUNCTION_ARGS)
{
	float8		arg1 = PG_GETARG_FLOAT8(0);
	float8		arg2 = PG_GETARG_FLOAT8(1);

	PG_RETURN_FLOAT8(float8_div(arg1, arg2));
}

/* Helper functions from utils/float.h */

static inline float4
float4_pl(const float4 val1, const float4 val2)
{
	float4		result;

	result = val1 + val2;
	if (unlikely(isinf(result)) && !isinf(val1) && !isinf(val2))
		float_overflow_error();

	return result;
}

static inline float8
float8_pl(const float8 val1, const float8 val2)
{
	float8		result;

	result = val1 + val2;
	if (unlikely(isinf(result)) && !isinf(val1) && !isinf(val2))
		float_overflow_error();

	return result;
}

static inline float4
float4_mi(const float4 val1, const float4 val2)
{
	float4		result;

	result = val1 - val2;
	if (unlikely(isinf(result)) && !isinf(val1) && !isinf(val2))
		float_overflow_error();

	return result;
}

static inline float8
float8_mi(const float8 val1, const float8 val2)
{
	float8		result;

	result = val1 - val2;
	if (unlikely(isinf(result)) && !isinf(val1) && !isinf(val2))
		float_overflow_error();

	return result;
}

static inline float4
float4_mul(const float4 val1, const float4 val2)
{
	float4		result;

	result = val1 * val2;
	if (unlikely(isinf(result)) && !isinf(val1) && !isinf(val2))
		float_overflow_error();
	if (unlikely(result == 0.0f) && val1 != 0.0f && val2 != 0.0f)
		float_underflow_error();

	return result;
}

static inline float8
float8_mul(const float8 val1, const float8 val2)
{
	float8		result;

	result = val1 * val2;
	if (unlikely(isinf(result)) && !isinf(val1) && !isinf(val2))
		float_overflow_error();
	if (unlikely(result == 0.0) && val1 != 0.0 && val2 != 0.0)
		float_underflow_error();

	return result;
}

static inline float4
float4_div(const float4 val1, const float4 val2)
{
	float4		result;

	if (unlikely(val2 == 0.0f) && !isnan(val1))
		float_zero_divide_error();
	result = val1 / val2;
	if (unlikely(isinf(result)) && !isinf(val1))
		float_overflow_error();
	if (unlikely(result == 0.0f) && val1 != 0.0f && !isinf(val2))
		float_underflow_error();

	return result;
}

static inline float8
float8_div(const float8 val1, const float8 val2)
{
	float8		result;

	if (unlikely(val2 == 0.0) && !isnan(val1))
		float_zero_divide_error();
	result = val1 / val2;
	if (unlikely(isinf(result)) && !isinf(val1))
		float_overflow_error();
	if (unlikely(result == 0.0) && val1 != 0.0 && !isinf(val2))
		float_underflow_error();

	return result;
}
