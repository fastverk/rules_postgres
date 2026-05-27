/*
 * renamed_float_arith.c — link-time renaming shim for the float arithmetic
 * C oracle. Includes the vendored body with all public symbols prefixed
 * with `_orig` so they don't collide with the Rust-emitted stubs during
 * linking. The wrappers.c file provides the wrapper entry points.
 */

#define float4pl   float4pl_orig
#define float4mi   float4mi_orig
#define float4mul  float4mul_orig
#define float4div  float4div_orig
#define float8pl   float8pl_orig
#define float8mi   float8mi_orig
#define float8mul  float8mul_orig
#define float8div  float8div_orig

#include "float_arith.c"
