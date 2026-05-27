/*
 * wrappers.c — re-export int4 comparison ops under `c_int4*` names.
 *
 * Mirrors sha2/c_oracle/wrappers.c. Required because the test binary
 * links BOTH the Rust crate (with `#[no_mangle] pub extern "C" fn
 * int4eq`) AND the C oracle (`int4eq` from int4_cmp.c). The linker
 * would otherwise silently pick one — the renamed-symbol approach
 * (this file + renamed_int4_cmp.c) makes the two impls genuinely
 * distinct.
 */

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

typedef unsigned int Oid;
typedef int16_t   int16;
typedef int32_t   int32;
typedef int64_t   int64;
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

/* The renamed (_orig) entry points from renamed_int4_cmp.c. */
extern Datum int4eq_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum int4ne_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum int4lt_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum int4le_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum int4gt_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum int4ge_orig(FunctionCallInfoBaseData *fcinfo);

extern Datum int2eq_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum int2ne_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum int2lt_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum int2le_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum int2gt_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum int2ge_orig(FunctionCallInfoBaseData *fcinfo);

extern Datum int8eq_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum int8ne_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum int8lt_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum int8le_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum int8gt_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum int8ge_orig(FunctionCallInfoBaseData *fcinfo);

Datum c_int4eq(FunctionCallInfoBaseData *fcinfo) { return int4eq_orig(fcinfo); }
Datum c_int4ne(FunctionCallInfoBaseData *fcinfo) { return int4ne_orig(fcinfo); }
Datum c_int4lt(FunctionCallInfoBaseData *fcinfo) { return int4lt_orig(fcinfo); }
Datum c_int4le(FunctionCallInfoBaseData *fcinfo) { return int4le_orig(fcinfo); }
Datum c_int4gt(FunctionCallInfoBaseData *fcinfo) { return int4gt_orig(fcinfo); }
Datum c_int4ge(FunctionCallInfoBaseData *fcinfo) { return int4ge_orig(fcinfo); }

Datum c_int2eq(FunctionCallInfoBaseData *fcinfo) { return int2eq_orig(fcinfo); }
Datum c_int2ne(FunctionCallInfoBaseData *fcinfo) { return int2ne_orig(fcinfo); }
Datum c_int2lt(FunctionCallInfoBaseData *fcinfo) { return int2lt_orig(fcinfo); }
Datum c_int2le(FunctionCallInfoBaseData *fcinfo) { return int2le_orig(fcinfo); }
Datum c_int2gt(FunctionCallInfoBaseData *fcinfo) { return int2gt_orig(fcinfo); }
Datum c_int2ge(FunctionCallInfoBaseData *fcinfo) { return int2ge_orig(fcinfo); }

Datum c_int8eq(FunctionCallInfoBaseData *fcinfo) { return int8eq_orig(fcinfo); }
Datum c_int8ne(FunctionCallInfoBaseData *fcinfo) { return int8ne_orig(fcinfo); }
Datum c_int8lt(FunctionCallInfoBaseData *fcinfo) { return int8lt_orig(fcinfo); }
Datum c_int8le(FunctionCallInfoBaseData *fcinfo) { return int8le_orig(fcinfo); }
Datum c_int8gt(FunctionCallInfoBaseData *fcinfo) { return int8gt_orig(fcinfo); }
Datum c_int8ge(FunctionCallInfoBaseData *fcinfo) { return int8ge_orig(fcinfo); }

extern Datum date_eq_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum date_ne_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum date_lt_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum date_le_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum date_gt_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum date_ge_orig(FunctionCallInfoBaseData *fcinfo);

extern Datum timestamp_eq_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum timestamp_ne_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum timestamp_lt_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum timestamp_le_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum timestamp_gt_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum timestamp_ge_orig(FunctionCallInfoBaseData *fcinfo);

extern Datum cash_eq_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum cash_ne_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum cash_lt_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum cash_le_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum cash_gt_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum cash_ge_orig(FunctionCallInfoBaseData *fcinfo);

extern Datum pg_lsn_eq_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum pg_lsn_ne_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum pg_lsn_lt_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum pg_lsn_le_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum pg_lsn_gt_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum pg_lsn_ge_orig(FunctionCallInfoBaseData *fcinfo);

extern Datum oideq_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum oidne_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum oidlt_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum oidle_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum oidgt_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum oidge_orig(FunctionCallInfoBaseData *fcinfo);

Datum c_date_eq(FunctionCallInfoBaseData *fcinfo) { return date_eq_orig(fcinfo); }
Datum c_date_ne(FunctionCallInfoBaseData *fcinfo) { return date_ne_orig(fcinfo); }
Datum c_date_lt(FunctionCallInfoBaseData *fcinfo) { return date_lt_orig(fcinfo); }
Datum c_date_le(FunctionCallInfoBaseData *fcinfo) { return date_le_orig(fcinfo); }
Datum c_date_gt(FunctionCallInfoBaseData *fcinfo) { return date_gt_orig(fcinfo); }
Datum c_date_ge(FunctionCallInfoBaseData *fcinfo) { return date_ge_orig(fcinfo); }

Datum c_timestamp_eq(FunctionCallInfoBaseData *fcinfo) { return timestamp_eq_orig(fcinfo); }
Datum c_timestamp_ne(FunctionCallInfoBaseData *fcinfo) { return timestamp_ne_orig(fcinfo); }
Datum c_timestamp_lt(FunctionCallInfoBaseData *fcinfo) { return timestamp_lt_orig(fcinfo); }
Datum c_timestamp_le(FunctionCallInfoBaseData *fcinfo) { return timestamp_le_orig(fcinfo); }
Datum c_timestamp_gt(FunctionCallInfoBaseData *fcinfo) { return timestamp_gt_orig(fcinfo); }
Datum c_timestamp_ge(FunctionCallInfoBaseData *fcinfo) { return timestamp_ge_orig(fcinfo); }

Datum c_cash_eq(FunctionCallInfoBaseData *fcinfo) { return cash_eq_orig(fcinfo); }
Datum c_cash_ne(FunctionCallInfoBaseData *fcinfo) { return cash_ne_orig(fcinfo); }
Datum c_cash_lt(FunctionCallInfoBaseData *fcinfo) { return cash_lt_orig(fcinfo); }
Datum c_cash_le(FunctionCallInfoBaseData *fcinfo) { return cash_le_orig(fcinfo); }
Datum c_cash_gt(FunctionCallInfoBaseData *fcinfo) { return cash_gt_orig(fcinfo); }
Datum c_cash_ge(FunctionCallInfoBaseData *fcinfo) { return cash_ge_orig(fcinfo); }

Datum c_pg_lsn_eq(FunctionCallInfoBaseData *fcinfo) { return pg_lsn_eq_orig(fcinfo); }
Datum c_pg_lsn_ne(FunctionCallInfoBaseData *fcinfo) { return pg_lsn_ne_orig(fcinfo); }
Datum c_pg_lsn_lt(FunctionCallInfoBaseData *fcinfo) { return pg_lsn_lt_orig(fcinfo); }
Datum c_pg_lsn_le(FunctionCallInfoBaseData *fcinfo) { return pg_lsn_le_orig(fcinfo); }
Datum c_pg_lsn_gt(FunctionCallInfoBaseData *fcinfo) { return pg_lsn_gt_orig(fcinfo); }
Datum c_pg_lsn_ge(FunctionCallInfoBaseData *fcinfo) { return pg_lsn_ge_orig(fcinfo); }

Datum c_oideq(FunctionCallInfoBaseData *fcinfo) { return oideq_orig(fcinfo); }
Datum c_oidne(FunctionCallInfoBaseData *fcinfo) { return oidne_orig(fcinfo); }
Datum c_oidlt(FunctionCallInfoBaseData *fcinfo) { return oidlt_orig(fcinfo); }
Datum c_oidle(FunctionCallInfoBaseData *fcinfo) { return oidle_orig(fcinfo); }
Datum c_oidgt(FunctionCallInfoBaseData *fcinfo) { return oidgt_orig(fcinfo); }
Datum c_oidge(FunctionCallInfoBaseData *fcinfo) { return oidge_orig(fcinfo); }

extern Datum booleq_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum boolne_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum boollt_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum boolle_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum boolgt_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum boolge_orig(FunctionCallInfoBaseData *fcinfo);

Datum c_booleq(FunctionCallInfoBaseData *fcinfo) { return booleq_orig(fcinfo); }
Datum c_boolne(FunctionCallInfoBaseData *fcinfo) { return boolne_orig(fcinfo); }
Datum c_boollt(FunctionCallInfoBaseData *fcinfo) { return boollt_orig(fcinfo); }
Datum c_boolle(FunctionCallInfoBaseData *fcinfo) { return boolle_orig(fcinfo); }
Datum c_boolgt(FunctionCallInfoBaseData *fcinfo) { return boolgt_orig(fcinfo); }
Datum c_boolge(FunctionCallInfoBaseData *fcinfo) { return boolge_orig(fcinfo); }

extern Datum float4eq_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum float4ne_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum float4lt_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum float4le_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum float4gt_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum float4ge_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum float8eq_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum float8ne_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum float8lt_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum float8le_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum float8gt_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum float8ge_orig(FunctionCallInfoBaseData *fcinfo);

Datum c_float4eq(FunctionCallInfoBaseData *fcinfo) { return float4eq_orig(fcinfo); }
Datum c_float4ne(FunctionCallInfoBaseData *fcinfo) { return float4ne_orig(fcinfo); }
Datum c_float4lt(FunctionCallInfoBaseData *fcinfo) { return float4lt_orig(fcinfo); }
Datum c_float4le(FunctionCallInfoBaseData *fcinfo) { return float4le_orig(fcinfo); }
Datum c_float4gt(FunctionCallInfoBaseData *fcinfo) { return float4gt_orig(fcinfo); }
Datum c_float4ge(FunctionCallInfoBaseData *fcinfo) { return float4ge_orig(fcinfo); }
Datum c_float8eq(FunctionCallInfoBaseData *fcinfo) { return float8eq_orig(fcinfo); }
Datum c_float8ne(FunctionCallInfoBaseData *fcinfo) { return float8ne_orig(fcinfo); }
Datum c_float8lt(FunctionCallInfoBaseData *fcinfo) { return float8lt_orig(fcinfo); }
Datum c_float8le(FunctionCallInfoBaseData *fcinfo) { return float8le_orig(fcinfo); }
Datum c_float8gt(FunctionCallInfoBaseData *fcinfo) { return float8gt_orig(fcinfo); }
Datum c_float8ge(FunctionCallInfoBaseData *fcinfo) { return float8ge_orig(fcinfo); }

extern Datum chareq_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum charne_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum charlt_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum charle_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum chargt_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum charge_orig(FunctionCallInfoBaseData *fcinfo);

Datum c_chareq(FunctionCallInfoBaseData *fcinfo) { return chareq_orig(fcinfo); }
Datum c_charne(FunctionCallInfoBaseData *fcinfo) { return charne_orig(fcinfo); }
Datum c_charlt(FunctionCallInfoBaseData *fcinfo) { return charlt_orig(fcinfo); }
Datum c_charle(FunctionCallInfoBaseData *fcinfo) { return charle_orig(fcinfo); }
Datum c_chargt(FunctionCallInfoBaseData *fcinfo) { return chargt_orig(fcinfo); }
Datum c_charge(FunctionCallInfoBaseData *fcinfo) { return charge_orig(fcinfo); }

/* cross-type int comparators: int24/42/28/82/48/84 */
#define WRAP(name) \
    extern Datum name##_orig(FunctionCallInfoBaseData *fcinfo); \
    Datum c_##name(FunctionCallInfoBaseData *fcinfo) { return name##_orig(fcinfo); }
WRAP(int24eq) WRAP(int24ne) WRAP(int24lt) WRAP(int24le) WRAP(int24gt) WRAP(int24ge)
WRAP(int42eq) WRAP(int42ne) WRAP(int42lt) WRAP(int42le) WRAP(int42gt) WRAP(int42ge)
WRAP(int28eq) WRAP(int28ne) WRAP(int28lt) WRAP(int28le) WRAP(int28gt) WRAP(int28ge)
WRAP(int82eq) WRAP(int82ne) WRAP(int82lt) WRAP(int82le) WRAP(int82gt) WRAP(int82ge)
WRAP(int48eq) WRAP(int48ne) WRAP(int48lt) WRAP(int48le) WRAP(int48gt) WRAP(int48ge)
WRAP(int84eq) WRAP(int84ne) WRAP(int84lt) WRAP(int84le) WRAP(int84gt) WRAP(int84ge)
#undef WRAP
