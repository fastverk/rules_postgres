/*
 * abi_probe.c — emit Postgres' canonical sizeof / offsetof for the
 * structs that pg_fcinfo's Rust mirrors must match.
 *
 * Self-contained: the struct definitions below are copied verbatim
 * from Postgres 17.6's src/include/postgres.h and src/include/fmgr.h,
 * along with the minimum typedefs they need. This keeps the probe
 * hermetic (no postgres_src include-path dependency at build time).
 *
 * If you bump the upstream Postgres version, re-paste the structs
 * from the new headers and re-run `cargo test`. Mismatches surface
 * via `verify_abi()` on the Rust side.
 *
 * Attribution: structs and field comments are PostgreSQL Global
 * Development Group, released under the PostgreSQL License
 * (BSD-style). See https://www.postgresql.org/about/licence/.
 */

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

/* ── Minimum typedefs from postgres_ext.h + c.h + postgres.h ─────────── */

typedef unsigned int Oid;                  /* postgres_ext.h */
typedef int16_t   int16;                   /* c.h */
typedef uintptr_t Datum;                   /* postgres.h */
typedef void *fmNodePtr;                   /* fmgr.h:46 — opaque Node * */

/* FLEXIBLE_ARRAY_MEMBER expands to nothing per Postgres c.h:330. */
#define FLEXIBLE_ARRAY_MEMBER

/* ── NullableDatum (postgres.h:368-379, verbatim) ────────────────────── */

typedef struct NullableDatum
{
#define FIELDNO_NULLABLE_DATUM_DATUM 0
	Datum		value;
#define FIELDNO_NULLABLE_DATUM_ISNULL 1
	bool		isnull;
	/* due to alignment padding this could be used for flags for free */
} NullableDatum;

/* FmgrInfo is referenced only by pointer; we don't need its layout
 * for the probe. Declare it as an opaque tag. */
struct FmgrInfo;

/* ── FunctionCallInfoBaseData (fmgr.h:90-103, verbatim) ──────────────── */

typedef struct FunctionCallInfoBaseData
{
	struct FmgrInfo *flinfo;       /* ptr to lookup info used for this call */
	fmNodePtr	context;           /* pass info about context of call */
	fmNodePtr	resultinfo;        /* pass or return extra info about result */
	Oid			fncollation;       /* collation for function to use */
#define FIELDNO_FUNCTIONCALLINFODATA_ISNULL 4
	bool		isnull;            /* function must set true if result is NULL */
	short		nargs;             /* # arguments actually passed */
#define FIELDNO_FUNCTIONCALLINFODATA_ARGS 6
	NullableDatum args[FLEXIBLE_ARRAY_MEMBER];
} FunctionCallInfoBaseData;

/* ── Probe exports ──────────────────────────────────────────────────── */

size_t pg_fcinfo_probe_sizeof_nullable_datum(void) {
    return sizeof(NullableDatum);
}

size_t pg_fcinfo_probe_sizeof_fcinfo_base(void) {
    /* The flexible-array base = offsetof(args). */
    return offsetof(FunctionCallInfoBaseData, args);
}

size_t pg_fcinfo_probe_offsetof_nullable_value(void) {
    return offsetof(NullableDatum, value);
}

size_t pg_fcinfo_probe_offsetof_nullable_isnull(void) {
    return offsetof(NullableDatum, isnull);
}

size_t pg_fcinfo_probe_offsetof_fcinfo_args(void) {
    return offsetof(FunctionCallInfoBaseData, args);
}

size_t pg_fcinfo_probe_offsetof_fcinfo_nargs(void) {
    return offsetof(FunctionCallInfoBaseData, nargs);
}

size_t pg_fcinfo_probe_offsetof_fcinfo_isnull(void) {
    return offsetof(FunctionCallInfoBaseData, isnull);
}
