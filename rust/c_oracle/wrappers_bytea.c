/*
 * byteacat doesn't ereport — bytea_catenate is pure (modulo OOM in
 * palloc which we don't model). No setjmp needed.
 */

#include <stdint.h>
#include <stdbool.h>

typedef uintptr_t Datum;
typedef struct FunctionCallInfoBaseData FunctionCallInfoBaseData;

#define WRAP(name)                                                            \
    extern Datum name##_orig(FunctionCallInfoBaseData *fcinfo);              \
    Datum c_##name(FunctionCallInfoBaseData *fcinfo) { return name##_orig(fcinfo); }

WRAP(byteacat)
WRAP(byteaoctetlen)
WRAP(byteaeq)

#undef WRAP
