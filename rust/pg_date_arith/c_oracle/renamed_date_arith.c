/*
 * renamed_date_arith.c — include the original date_arith.c but define
 * RENAME_PG_SYMBOL to prefix all function definitions with _orig.
 * This prevents symbol collisions when wrappers.c re-exports them under
 * c_* names.
 */

#define RENAME_PG_SYMBOL(fn) fn##_orig
#define date_pli date_pli_orig
#define date_mii date_mii_orig
#define date_mi  date_mi_orig

#include "date_arith.c"
