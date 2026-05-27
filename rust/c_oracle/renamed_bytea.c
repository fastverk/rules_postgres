/*
 * renamed_bytea.c — rename byteacat → byteacat_orig before compiling,
 * so the test linker doesn't merge with the Rust crate's #[no_mangle].
 *
 * bytea_catenate is `static bytea *` — file-local linkage, no rename
 * needed.
 */

#define byteacat       byteacat_orig
#define byteaoctetlen  byteaoctetlen_orig
#define byteaeq        byteaeq_orig

#include "bytea.c"
