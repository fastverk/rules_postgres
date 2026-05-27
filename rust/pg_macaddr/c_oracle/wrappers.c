/*
 * wrappers.c — re-export macaddr ops under `c_macaddr*` names.
 *
 * Allows the test binary to link BOTH the Rust crate implementations
 * AND the C oracle implementations under distinct names for differential
 * testing.
 */

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

typedef unsigned int Oid;
typedef int16_t   int16;
typedef int32_t   int32;
typedef int64_t   int64;
typedef uint32_t  uint32;
typedef uint64_t  uint64;
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

/* The renamed (_orig) entry points from renamed_macaddr.c. */
extern Datum macaddr_eq_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum macaddr_ne_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum macaddr_lt_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum macaddr_le_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum macaddr_gt_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum macaddr_ge_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum macaddr_cmp_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum hashmacaddr_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum hashmacaddrextended_orig(FunctionCallInfoBaseData *fcinfo);

/* Re-export under c_macaddr* names. */
Datum c_macaddr_eq(FunctionCallInfoBaseData *fcinfo) { return macaddr_eq_orig(fcinfo); }
Datum c_macaddr_ne(FunctionCallInfoBaseData *fcinfo) { return macaddr_ne_orig(fcinfo); }
Datum c_macaddr_lt(FunctionCallInfoBaseData *fcinfo) { return macaddr_lt_orig(fcinfo); }
Datum c_macaddr_le(FunctionCallInfoBaseData *fcinfo) { return macaddr_le_orig(fcinfo); }
Datum c_macaddr_gt(FunctionCallInfoBaseData *fcinfo) { return macaddr_gt_orig(fcinfo); }
Datum c_macaddr_ge(FunctionCallInfoBaseData *fcinfo) { return macaddr_ge_orig(fcinfo); }
Datum c_macaddr_cmp(FunctionCallInfoBaseData *fcinfo) { return macaddr_cmp_orig(fcinfo); }
Datum c_hashmacaddr(FunctionCallInfoBaseData *fcinfo) { return hashmacaddr_orig(fcinfo); }
Datum c_hashmacaddrextended(FunctionCallInfoBaseData *fcinfo) { return hashmacaddrextended_orig(fcinfo); }
