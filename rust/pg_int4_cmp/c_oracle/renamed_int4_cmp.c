/*
 * renamed_int4_cmp.c — preprocessor-rename shim that compiles
 * int4_cmp.c with its public symbols renamed to `<name>_orig`,
 * matching the pattern from sha2/c_oracle/renamed_sha2.c.
 *
 * Why: prevents the test linker from silently merging the C oracle's
 * `int4eq` with the Rust crate's `#[no_mangle] pub extern "C" fn
 * int4eq`. Without this rename the diff-test compared Rust to itself
 * (bug class verified during the sha2 work — same fix applies here).
 *
 * The wrappers.c file calls the `_orig` versions and re-exports them
 * as `c_int4eq` etc. for the differential tests to link against.
 */

#define int4eq int4eq_orig
#define int4ne int4ne_orig
#define int4lt int4lt_orig
#define int4le int4le_orig
#define int4gt int4gt_orig
#define int4ge int4ge_orig

#define int2eq int2eq_orig
#define int2ne int2ne_orig
#define int2lt int2lt_orig
#define int2le int2le_orig
#define int2gt int2gt_orig
#define int2ge int2ge_orig

#define int8eq int8eq_orig
#define int8ne int8ne_orig
#define int8lt int8lt_orig
#define int8le int8le_orig
#define int8gt int8gt_orig
#define int8ge int8ge_orig

/* Cross-family lift (cmp shape reused across non-int integer types). */

#define date_eq date_eq_orig
#define date_ne date_ne_orig
#define date_lt date_lt_orig
#define date_le date_le_orig
#define date_gt date_gt_orig
#define date_ge date_ge_orig

#define timestamp_eq timestamp_eq_orig
#define timestamp_ne timestamp_ne_orig
#define timestamp_lt timestamp_lt_orig
#define timestamp_le timestamp_le_orig
#define timestamp_gt timestamp_gt_orig
#define timestamp_ge timestamp_ge_orig

#define cash_eq cash_eq_orig
#define cash_ne cash_ne_orig
#define cash_lt cash_lt_orig
#define cash_le cash_le_orig
#define cash_gt cash_gt_orig
#define cash_ge cash_ge_orig

#define pg_lsn_eq pg_lsn_eq_orig
#define pg_lsn_ne pg_lsn_ne_orig
#define pg_lsn_lt pg_lsn_lt_orig
#define pg_lsn_le pg_lsn_le_orig
#define pg_lsn_gt pg_lsn_gt_orig
#define pg_lsn_ge pg_lsn_ge_orig

#define oideq oideq_orig
#define oidne oidne_orig
#define oidlt oidlt_orig
#define oidle oidle_orig
#define oidgt oidgt_orig
#define oidge oidge_orig

#define booleq booleq_orig
#define boolne boolne_orig
#define boollt boollt_orig
#define boolle boolle_orig
#define boolgt boolgt_orig
#define boolge boolge_orig

#define float4eq float4eq_orig
#define float4ne float4ne_orig
#define float4lt float4lt_orig
#define float4le float4le_orig
#define float4gt float4gt_orig
#define float4ge float4ge_orig
#define float8eq float8eq_orig
#define float8ne float8ne_orig
#define float8lt float8lt_orig
#define float8le float8le_orig
#define float8gt float8gt_orig
#define float8ge float8ge_orig

#define chareq chareq_orig
#define charne charne_orig
#define charlt charlt_orig
#define charle charle_orig
#define chargt chargt_orig
#define charge charge_orig

#define int24eq int24eq_orig
#define int24ne int24ne_orig
#define int24lt int24lt_orig
#define int24le int24le_orig
#define int24gt int24gt_orig
#define int24ge int24ge_orig
#define int42eq int42eq_orig
#define int42ne int42ne_orig
#define int42lt int42lt_orig
#define int42le int42le_orig
#define int42gt int42gt_orig
#define int42ge int42ge_orig
#define int28eq int28eq_orig
#define int28ne int28ne_orig
#define int28lt int28lt_orig
#define int28le int28le_orig
#define int28gt int28gt_orig
#define int28ge int28ge_orig
#define int82eq int82eq_orig
#define int82ne int82ne_orig
#define int82lt int82lt_orig
#define int82le int82le_orig
#define int82gt int82gt_orig
#define int82ge int82ge_orig
#define int48eq int48eq_orig
#define int48ne int48ne_orig
#define int48lt int48lt_orig
#define int48le int48le_orig
#define int48gt int48gt_orig
#define int48ge int48ge_orig
#define int84eq int84eq_orig
#define int84ne int84ne_orig
#define int84lt int84lt_orig
#define int84le int84le_orig
#define int84gt int84gt_orig
#define int84ge int84ge_orig

#include "int4_cmp.c"
