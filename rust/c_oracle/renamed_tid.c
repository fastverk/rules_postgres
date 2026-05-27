/*
 * renamed_tid.c — preprocessor-rename shim that compiles tid.c
 * with its public symbols renamed to `<name>_orig`.
 *
 * Prevents the test linker from silently merging the C oracle's functions
 * with the Rust crate's #[no_mangle] implementations.
 */

#define tideq tideq_orig
#define tidne tidne_orig
#define tidlt tidlt_orig
#define tidle tidle_orig
#define tidgt tidgt_orig
#define tidge tidge_orig
#define bttidcmp bttidcmp_orig
#define tidlarger tidlarger_orig
#define tidsmaller tidsmaller_orig
#define hashtid hashtid_orig
#define hashtidextended hashtidextended_orig

#define ItemPointerCompare ItemPointerCompare_orig

#include "tid.c"
