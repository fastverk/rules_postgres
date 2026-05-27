#include <stdint.h>
#include <stdbool.h>
#include <setjmp.h>

typedef uintptr_t Datum;
typedef struct FunctionCallInfoBaseData FunctionCallInfoBaseData;

/* Thread-local jmp_buf for the C oracle's ereport/longjmp. */
__thread jmp_buf fmgr_oracle_jmp;

/* Thread-local errcode captured by MAKE_SQLSTATE override. */
__thread uint32_t fmgr_oracle_last_errcode;

/* Each wrapper function:
 *   1. Sets up a setjmp/longjmp pair to catch ereport(ERROR, ...).
 *   2. Calls the renamed _orig version of the body.
 *   3. On error, the longjmp lands here and we return 0 (error Datum).
 *   4. On success, return the computed Datum.
 */

#define WRAP(name)                                                            \
    extern Datum name##_orig(FunctionCallInfoBaseData *fcinfo);              \
    Datum c_##name(FunctionCallInfoBaseData *fcinfo) {                       \
        if (setjmp(fmgr_oracle_jmp)) {                                        \
            return 0;  /* error: longjmp landed here */                       \
        }                                                                    \
        return name##_orig(fcinfo);                                          \
    }

WRAP(ftoi2)  WRAP(ftoi4)  WRAP(dtoi2)  WRAP(dtoi4)

#undef WRAP
