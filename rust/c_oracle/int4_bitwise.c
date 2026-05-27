/*
 * int4_bitwise.c — vendored int bitwise V1 fmgr bodies from Postgres
 * 17.6. Byte-identical to real PG source modulo the standalone
 * preamble. No ereport, no palloc.
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
#define PG_GETARG_INT16(n)       ((int16) fcinfo->args[n].value)
#define PG_GETARG_INT32(n)       ((int32) fcinfo->args[n].value)
#define PG_GETARG_INT64(n)       ((int64) fcinfo->args[n].value)
#define PG_RETURN_INT16(x)       return (Datum) (uint16_t) (x)
#define PG_RETURN_INT32(x)       return (Datum) (uint32) (x)
#define PG_RETURN_INT64(x)       return (Datum) (uint64_t) (x)

/* ──────────────────────────────────────────────────────────────── */
/* Bodies below are byte-identical to real Postgres source.         */
/* ──────────────────────────────────────────────────────────────── */

Datum int2and(PG_FUNCTION_ARGS) {
	int16		arg1 = PG_GETARG_INT16(0);
	int16		arg2 = PG_GETARG_INT16(1);
	PG_RETURN_INT16(arg1 & arg2);
}
Datum int2or(PG_FUNCTION_ARGS) {
	int16		arg1 = PG_GETARG_INT16(0);
	int16		arg2 = PG_GETARG_INT16(1);
	PG_RETURN_INT16(arg1 | arg2);
}
Datum int2xor(PG_FUNCTION_ARGS) {
	int16		arg1 = PG_GETARG_INT16(0);
	int16		arg2 = PG_GETARG_INT16(1);
	PG_RETURN_INT16(arg1 ^ arg2);
}
Datum int2not(PG_FUNCTION_ARGS) {
	int16		arg1 = PG_GETARG_INT16(0);
	PG_RETURN_INT16(~arg1);
}
Datum int2shl(PG_FUNCTION_ARGS) {
	int16		arg1 = PG_GETARG_INT16(0);
	int32		arg2 = PG_GETARG_INT32(1);
	PG_RETURN_INT16(arg1 << arg2);
}
Datum int2shr(PG_FUNCTION_ARGS) {
	int16		arg1 = PG_GETARG_INT16(0);
	int32		arg2 = PG_GETARG_INT32(1);
	PG_RETURN_INT16(arg1 >> arg2);
}

Datum int4and(PG_FUNCTION_ARGS) {
	int32		arg1 = PG_GETARG_INT32(0);
	int32		arg2 = PG_GETARG_INT32(1);
	PG_RETURN_INT32(arg1 & arg2);
}
Datum int4or(PG_FUNCTION_ARGS) {
	int32		arg1 = PG_GETARG_INT32(0);
	int32		arg2 = PG_GETARG_INT32(1);
	PG_RETURN_INT32(arg1 | arg2);
}
Datum int4xor(PG_FUNCTION_ARGS) {
	int32		arg1 = PG_GETARG_INT32(0);
	int32		arg2 = PG_GETARG_INT32(1);
	PG_RETURN_INT32(arg1 ^ arg2);
}
Datum int4not(PG_FUNCTION_ARGS) {
	int32		arg1 = PG_GETARG_INT32(0);
	PG_RETURN_INT32(~arg1);
}
Datum int4shl(PG_FUNCTION_ARGS) {
	int32		arg1 = PG_GETARG_INT32(0);
	int32		arg2 = PG_GETARG_INT32(1);
	PG_RETURN_INT32(arg1 << arg2);
}
Datum int4shr(PG_FUNCTION_ARGS) {
	int32		arg1 = PG_GETARG_INT32(0);
	int32		arg2 = PG_GETARG_INT32(1);
	PG_RETURN_INT32(arg1 >> arg2);
}

Datum int8and(PG_FUNCTION_ARGS) {
	int64		arg1 = PG_GETARG_INT64(0);
	int64		arg2 = PG_GETARG_INT64(1);
	PG_RETURN_INT64(arg1 & arg2);
}
Datum int8or(PG_FUNCTION_ARGS) {
	int64		arg1 = PG_GETARG_INT64(0);
	int64		arg2 = PG_GETARG_INT64(1);
	PG_RETURN_INT64(arg1 | arg2);
}
Datum int8xor(PG_FUNCTION_ARGS) {
	int64		arg1 = PG_GETARG_INT64(0);
	int64		arg2 = PG_GETARG_INT64(1);
	PG_RETURN_INT64(arg1 ^ arg2);
}
Datum int8not(PG_FUNCTION_ARGS) {
	int64		arg1 = PG_GETARG_INT64(0);
	PG_RETURN_INT64(~arg1);
}
Datum int8shl(PG_FUNCTION_ARGS) {
	int64		arg1 = PG_GETARG_INT64(0);
	int32		arg2 = PG_GETARG_INT32(1);
	PG_RETURN_INT64(arg1 << arg2);
}
Datum int8shr(PG_FUNCTION_ARGS) {
	int64		arg1 = PG_GETARG_INT64(0);
	int32		arg2 = PG_GETARG_INT32(1);
	PG_RETURN_INT64(arg1 >> arg2);
}
