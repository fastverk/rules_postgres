/*
 * wrappers.c — re-export tid ops under `c_tid*` names.
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

/* The renamed (_orig) entry points from renamed_tid.c. */
extern Datum tideq_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum tidne_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum tidlt_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum tidle_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum tidgt_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum tidge_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum bttidcmp_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum tidlarger_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum tidsmaller_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum hashtid_orig(FunctionCallInfoBaseData *fcinfo);
extern Datum hashtidextended_orig(FunctionCallInfoBaseData *fcinfo);

/* Re-export under c_tid* names. */
Datum c_tideq(FunctionCallInfoBaseData *fcinfo) { return tideq_orig(fcinfo); }
Datum c_tidne(FunctionCallInfoBaseData *fcinfo) { return tidne_orig(fcinfo); }
Datum c_tidlt(FunctionCallInfoBaseData *fcinfo) { return tidlt_orig(fcinfo); }
Datum c_tidle(FunctionCallInfoBaseData *fcinfo) { return tidle_orig(fcinfo); }
Datum c_tidgt(FunctionCallInfoBaseData *fcinfo) { return tidgt_orig(fcinfo); }
Datum c_tidge(FunctionCallInfoBaseData *fcinfo) { return tidge_orig(fcinfo); }
Datum c_bttidcmp(FunctionCallInfoBaseData *fcinfo) { return bttidcmp_orig(fcinfo); }
Datum c_tidlarger(FunctionCallInfoBaseData *fcinfo) { return tidlarger_orig(fcinfo); }
Datum c_tidsmaller(FunctionCallInfoBaseData *fcinfo) { return tidsmaller_orig(fcinfo); }
Datum c_hashtid(FunctionCallInfoBaseData *fcinfo) { return hashtid_orig(fcinfo); }
Datum c_hashtidextended(FunctionCallInfoBaseData *fcinfo) { return hashtidextended_orig(fcinfo); }
