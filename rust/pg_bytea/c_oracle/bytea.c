/*
 * bytea.c — vendored byteacat + bytea_catenate from Postgres 17.6's
 * `src/backend/utils/adt/varlena.c`. Body shape:
 *
 *   byteacat(fcinfo)         — V1 fmgr stub; reads two bytea args via
 *                              PG_GETARG_BYTEA_PP, delegates to
 *                              bytea_catenate, returns the result.
 *
 *   bytea_catenate(t1, t2)   — static internal helper that does the
 *                              actual work: read sizes, palloc total,
 *                              SET_VARSIZE, memcpy both payloads.
 *
 * v0 scope assumes BOTH inputs are uncompressed 4-byte-header bytea
 * values (no TOAST, no SHORT-header). The Rust-side pg_fcinfo
 * varlena helpers panic on non-4-byte headers; the diff-test inputs
 * stay within that scope by construction.
 *
 * Attribution: function bodies are PostgreSQL Global Development Group,
 * released under the PostgreSQL License (BSD-style).
 */

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>
#include <string.h>

typedef int16_t  int16;
typedef int32_t  int32;
typedef int64_t  int64;
typedef uint32_t uint32;
typedef uintptr_t Datum;
typedef void *fmNodePtr;
typedef size_t Size;
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

/* bytea is a varlena. We don't define a struct here — bytea pointers
 * point at a 4-byte length header followed by payload bytes. */
typedef unsigned char bytea;

#define PG_GETARG_BYTEA_PP(n)  ((bytea *) fcinfo->args[n].value)
#define PG_RETURN_BYTEA_P(p)   return (Datum) (uintptr_t) (p)

/* 4-byte varlena macros, matching pg_fcinfo's helpers for the same
 * uncompressed-non-TOAST case. */
#define VARHDRSZ                4

#define VARSIZE_4B(PTR)         \
    ((((*(uint32_t *) (PTR)) >> 2) & 0x3FFFFFFFu))
#define VARDATA_4B(PTR)         ((char *) (PTR) + VARHDRSZ)
#define SET_VARSIZE_4B(PTR,len) \
    (*(uint32_t *) (PTR) = ((uint32_t) (len)) << 2)

/* The _ANY_ variants. For v0 we only support 4-byte headers. */
#define VARSIZE_ANY(PTR)        VARSIZE_4B(PTR)
#define VARSIZE_ANY_EXHDR(PTR)  (VARSIZE_4B(PTR) - VARHDRSZ)
#define VARDATA_ANY(PTR)        VARDATA_4B(PTR)
#define SET_VARSIZE(PTR,len)    SET_VARSIZE_4B(PTR, len)
#define VARDATA(PTR)            VARDATA_4B(PTR)

/* TOAST-light stubs. Real Postgres' toast_raw_datum_size peeks at the
 * varlena header / TOAST descriptor without detoasting. For our v0
 * scope (uncompressed, non-TOASTed inputs) it reduces to VARSIZE_4B.
 *
 * DatumGetByteaPP just casts the Datum to bytea* in the non-TOAST
 * case. PG_FREE_IF_COPY is a no-op when the input wasn't copied
 * during detoast.
 */
#define PG_GETARG_DATUM(n)         ((Datum) fcinfo->args[n].value)
#define PG_RETURN_INT32(x)         return (Datum) (uint32) (x)
#define PG_RETURN_BOOL(x)          return (Datum)((x) ? 1 : 0)
#define DatumGetByteaPP(d)         ((bytea *) (uintptr_t) (d))
#define PG_FREE_IF_COPY(p, n)      ((void) 0)

static inline Size toast_raw_datum_size(Datum d) {
    return VARSIZE_4B((void *)(uintptr_t)d);
}

#include "pg_palloc.h"

/* ──────────────────────────────────────────────────────────────── */
/* Body below is byte-identical to real Postgres source             */
/* (src/backend/utils/adt/varlena.c, PG 17.6).                      */
/* ──────────────────────────────────────────────────────────────── */

static bytea *
bytea_catenate(bytea *t1, bytea *t2)
{
	bytea	   *result;
	int			len1,
				len2,
				len;
	char	   *ptr;

	len1 = VARSIZE_ANY_EXHDR(t1);
	len2 = VARSIZE_ANY_EXHDR(t2);

	/* paranoia ... probably should throw error instead? */
	if (len1 < 0)
		len1 = 0;
	if (len2 < 0)
		len2 = 0;

	len = len1 + len2 + VARHDRSZ;
	result = (bytea *) palloc(len);

	/* Set size of result string... */
	SET_VARSIZE(result, len);

	/* Fill data field of result string... */
	ptr = VARDATA(result);
	if (len1 > 0)
		memcpy(ptr, VARDATA_ANY(t1), len1);
	if (len2 > 0)
		memcpy(ptr + len1, VARDATA_ANY(t2), len2);

	return result;
}

Datum
byteacat(PG_FUNCTION_ARGS)
{
	bytea	   *t1 = PG_GETARG_BYTEA_PP(0);
	bytea	   *t2 = PG_GETARG_BYTEA_PP(1);

	PG_RETURN_BYTEA_P(bytea_catenate(t1, t2));
}

Datum
byteaoctetlen(PG_FUNCTION_ARGS)
{
	Datum		str = PG_GETARG_DATUM(0);

	/* We need not detoast the input at all */
	PG_RETURN_INT32(toast_raw_datum_size(str) - VARHDRSZ);
}

Datum
byteaeq(PG_FUNCTION_ARGS)
{
	Datum		arg1 = PG_GETARG_DATUM(0);
	Datum		arg2 = PG_GETARG_DATUM(1);
	bool		result;
	Size		len1,
				len2;

	/*
	 * We can use a fast path for unequal lengths, which might save us from
	 * having to detoast one or both values.
	 */
	len1 = toast_raw_datum_size(arg1);
	len2 = toast_raw_datum_size(arg2);
	if (len1 != len2)
		result = false;
	else
	{
		bytea	   *barg1 = DatumGetByteaPP(arg1);
		bytea	   *barg2 = DatumGetByteaPP(arg2);

		result = (memcmp(VARDATA_ANY(barg1), VARDATA_ANY(barg2),
						 len1 - VARHDRSZ) == 0);

		PG_FREE_IF_COPY(barg1, 0);
		PG_FREE_IF_COPY(barg2, 1);
	}

	PG_RETURN_BOOL(result);
}
