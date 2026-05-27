/*
 * wrappers.c — re-export the _orig-renamed C oracle symbols under the
 * `c_*` prefix the Rust diff-test extern-declares. Mirrors
 * pg_int4_cmp/c_oracle/wrappers.c.
 */

#include <stdint.h>
#include <stdbool.h>

typedef uintptr_t Datum;

typedef struct NullableDatum {
    Datum value;
    bool  isnull;
} NullableDatum;

typedef struct FunctionCallInfoBaseData FunctionCallInfoBaseData;

#define WRAP(name) \
    extern Datum name##_orig(FunctionCallInfoBaseData *fcinfo); \
    Datum c_##name(FunctionCallInfoBaseData *fcinfo) { return name##_orig(fcinfo); }

WRAP(hashchar)
WRAP(hashint2)
WRAP(hashint4)
WRAP(hashint8)
WRAP(hashoid)
WRAP(hashenum)

WRAP(hashcharextended)
WRAP(hashint2extended)
WRAP(hashint4extended)
WRAP(hashint8extended)
WRAP(hashoidextended)
WRAP(hashenumextended)

#undef WRAP
