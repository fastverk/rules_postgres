/*
 * text.c — vendored textcat + text_catenate from Postgres 17.6's
 * `src/backend/utils/adt/varlena.c`. Body shape:
 *
 *   textcat(fcinfo)        — V1 fmgr stub; reads two text args via
 *                             PG_GETARG_TEXT_PP, delegates to
 *                             text_catenate, returns the result.
 *
 *   text_catenate(t1, t2)  — static internal helper that does the
 *                             actual work: read sizes, palloc total,
 *                             SET_VARSIZE, memcpy both payloads.
 *
 * v0 scope assumes BOTH inputs are uncompressed 4-byte-header text
 * values (no TOAST, no SHORT-header). The Rust-side pg_fcinfo
 * varlena helpers panic on non-4-byte headers; the diff-test inputs
 * stay within that scope by construction.
 *
 * Text and bytea share the same varlena layout at the Datum level,
 * so text_catenate is structurally identical to bytea_catenate.
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

/* text is a varlena. We don't define a struct here — text pointers
 * point at a 4-byte length header followed by payload bytes. */
typedef unsigned char text;

#define PG_GETARG_TEXT_PP(n)  ((text *) fcinfo->args[n].value)
#define PG_RETURN_TEXT_P(p)   return (Datum) (uintptr_t) (p)

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

#include "pg_palloc.h"

/* ──────────────────────────────────────────────────────────────── */
/* Body below is byte-identical to real Postgres source             */
/* (src/backend/utils/adt/varlena.c, PG 17.6).                      */
/* ──────────────────────────────────────────────────────────────── */

static text *
text_catenate(text *t1, text *t2)
{
	text	   *result;
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
	result = (text *) palloc(len);

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
textcat(PG_FUNCTION_ARGS)
{
	text	   *t1 = PG_GETARG_TEXT_PP(0);
	text	   *t2 = PG_GETARG_TEXT_PP(1);

	PG_RETURN_TEXT_P(text_catenate(t1, t2));
}
