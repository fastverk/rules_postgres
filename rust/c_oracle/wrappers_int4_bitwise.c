/*
 * Bitwise functions are pure — no ereport, no longjmp needed. So
 * wrappers here are trivial passthroughs without setjmp.
 */

#include <stdint.h>
#include <stdbool.h>

typedef uintptr_t Datum;
typedef struct FunctionCallInfoBaseData FunctionCallInfoBaseData;

#define WRAP(name)                                                            \
    extern Datum name##_orig(FunctionCallInfoBaseData *fcinfo);              \
    Datum c_##name(FunctionCallInfoBaseData *fcinfo) { return name##_orig(fcinfo); }

WRAP(int2and) WRAP(int2or) WRAP(int2xor) WRAP(int2not) WRAP(int2shl) WRAP(int2shr)
WRAP(int4and) WRAP(int4or) WRAP(int4xor) WRAP(int4not) WRAP(int4shl) WRAP(int4shr)
WRAP(int8and) WRAP(int8or) WRAP(int8xor) WRAP(int8not) WRAP(int8shl) WRAP(int8shr)

#undef WRAP
