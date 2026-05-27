/*
 * wrappers.c — re-export the _orig-renamed C oracle symbols under the
 * `c_*` prefix the Rust diff-test extern-declares, wrapping each in a
 * setjmp() so that ereport() in the body longjmps cleanly out to here
 * and the wrapper returns Datum 0.
 */

#include <stdint.h>
#include <stdbool.h>
#include <setjmp.h>

typedef uintptr_t Datum;
typedef struct FunctionCallInfoBaseData FunctionCallInfoBaseData;

/* Shared TLS jmp_buf. The c_oracle_ereport.h `ereport()` macro
 * longjmps to this buf; the wrapper functions below set it up. */
__thread jmp_buf fmgr_oracle_jmp;
/* Capture target for the `errcode(...)` macro. */
__thread uint32_t fmgr_oracle_last_errcode;

#define WRAP(name)                                                            \
    extern Datum name##_orig(FunctionCallInfoBaseData *fcinfo);              \
    Datum c_##name(FunctionCallInfoBaseData *fcinfo) {                       \
        if (setjmp(fmgr_oracle_jmp) != 0) {                                  \
            /* ereport() in the body longjmp'd here; the Rust-side flag */   \
            /* was already set inside the macro via pg_fcinfo_set_error. */  \
            return (Datum) 0;                                                \
        }                                                                    \
        return name##_orig(fcinfo);                                          \
    }

WRAP(cash_pl)
WRAP(cash_mi)
WRAP(cash_mul_int4)
WRAP(cash_div_int4)
WRAP(cashlarger)
WRAP(cashsmaller)

#undef WRAP
