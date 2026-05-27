/*
 * wrappers.c — re-export the _orig-renamed C oracle symbols under the
 * `c_*` prefix the Rust diff-test extern-declares. Each c_<fn> wrapper
 * sets up a setjmp() so that ereport() in the body longjmps cleanly out
 * and the wrapper returns Datum 0.
 *
 * The TLS jmp_buf and last-errcode slot are owned by this file (one
 * per per-crate-linker-unit; sharing across crates isn't needed since
 * each crate's renamed body links its own copy of c_oracle_ereport.h's
 * static-inline route helper).
 */

#include <stdint.h>
#include <stdbool.h>
#include <setjmp.h>

typedef uintptr_t Datum;
typedef struct FunctionCallInfoBaseData FunctionCallInfoBaseData;

/* TLS slots referenced by c_oracle_ereport.h. */
__thread jmp_buf fmgr_oracle_jmp;
__thread uint32_t fmgr_oracle_last_errcode;

#define WRAP(name)                                                            \
    extern Datum name##_orig(FunctionCallInfoBaseData *fcinfo);              \
    Datum c_##name(FunctionCallInfoBaseData *fcinfo) {                       \
        if (setjmp(fmgr_oracle_jmp) != 0) {                                  \
            return (Datum) 0;                                                \
        }                                                                    \
        return name##_orig(fcinfo);                                          \
    }

WRAP(interval_um)
WRAP(interval_pl)
WRAP(interval_mi)

#undef WRAP
