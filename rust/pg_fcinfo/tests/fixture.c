/*
 * fixture.c — synthetic V1 fmgr fixtures for pg_fcinfo's integration test.
 *
 * These are tiny, self-contained C functions that mimic the Postgres V1
 * fmgr ABI (Datum fn(FunctionCallInfo)) without any Postgres backend
 * dependency. They let us validate end-to-end that:
 *
 *   1. `build_fcinfo!` produces a `FunctionCallInfoBaseData` whose layout
 *      matches the C side bit-for-bit (verified by `verify_abi`, also
 *      exercised here at runtime).
 *   2. `encode_*` helpers produce Datum values that the C side decodes
 *      back to the original Rust value.
 *   3. The full Rust→C→Rust round-trip preserves semantics.
 *
 * When Stream C lifts a real Postgres V1 function (e.g., gbt_bool_compress),
 * the only structural difference is the C function comes from Postgres'
 * source tree + the rename shim. The pg_fcinfo + build_fcinfo! +
 * decode_* surface stays the same.
 *
 * Self-contained: same vendored struct definitions as abi_probe.c so we
 * don't depend on the postgres_src include path at build time.
 */

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

typedef unsigned int Oid;
typedef int16_t  int16;
typedef int32_t  int32;
typedef int64_t  int64;
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
    Oid              fncollation;
    bool             isnull;
    short            nargs;
    NullableDatum    args[FLEXIBLE_ARRAY_MEMBER];
} FunctionCallInfoBaseData;

/* PG_GETARG_INT32(N) expands roughly to (int32) fcinfo->args[N].value */
#define PG_GETARG_INT32(fcinfo, n)  ((int32) (fcinfo)->args[n].value)
#define PG_GETARG_INT64(fcinfo, n)  ((int64) (fcinfo)->args[n].value)
#define PG_GETARG_BOOL(fcinfo, n)   (((fcinfo)->args[n].value & 1) != 0)

#define PG_RETURN_INT32(x)  return (Datum)(uint32_t)(x)
#define PG_RETURN_INT64(x)  return (Datum)(uint64_t)(x)
#define PG_RETURN_BOOL(x)   return (Datum)((x) ? 1 : 0)

/* ── Fixtures ─────────────────────────────────────────────────────────── */

/* Returns input * 2 — exercises i32 arg encode/decode round-trip. */
Datum c_test_double_i32(FunctionCallInfoBaseData *fcinfo) {
    int32 v = PG_GETARG_INT32(fcinfo, 0);
    PG_RETURN_INT32(v * 2);
}

/* Returns sum of two i32 args. */
Datum c_test_add_i32(FunctionCallInfoBaseData *fcinfo) {
    int32 a = PG_GETARG_INT32(fcinfo, 0);
    int32 b = PG_GETARG_INT32(fcinfo, 1);
    PG_RETURN_INT32(a + b);
}

/* Returns NOT bool. */
Datum c_test_not_bool(FunctionCallInfoBaseData *fcinfo) {
    bool v = PG_GETARG_BOOL(fcinfo, 0);
    PG_RETURN_BOOL(!v);
}

/* Returns nargs as i64 — exercises reading the nargs field. */
Datum c_test_echo_nargs(FunctionCallInfoBaseData *fcinfo) {
    PG_RETURN_INT64((int64) fcinfo->nargs);
}

/* Sets isnull = true, returns 0. Exercises the result-nullness path. */
Datum c_test_return_null(FunctionCallInfoBaseData *fcinfo) {
    fcinfo->isnull = true;
    PG_RETURN_INT32(0);
}
