/*
 * renamed_float_to_int.c — link-time renaming shim for the float-to-int
 * cast C oracle. Includes the vendored body with all public symbols prefixed
 * with `_orig` so they don't collide with the Rust-emitted stubs during
 * linking. The wrappers.c file provides the wrapper entry points.
 */

#define ftoi2  ftoi2_orig
#define ftoi4  ftoi4_orig
#define dtoi2  dtoi2_orig
#define dtoi4  dtoi4_orig

#include "float_to_int.c"
