#include <stdint.h>
#include <stdbool.h>
#include <setjmp.h>

typedef uintptr_t Datum;
typedef struct FunctionCallInfoBaseData FunctionCallInfoBaseData;

__thread jmp_buf  fmgr_oracle_jmp;
__thread uint32_t fmgr_oracle_last_errcode;

#define WRAP(name)                                                            \
    extern Datum name##_orig(FunctionCallInfoBaseData *fcinfo);              \
    Datum c_##name(FunctionCallInfoBaseData *fcinfo) {                       \
        if (setjmp(fmgr_oracle_jmp) != 0) return (Datum) 0;                  \
        return name##_orig(fcinfo);                                          \
    }

WRAP(i2toi4)  WRAP(i4toi2)
WRAP(int28)   WRAP(int48)
WRAP(int82)   WRAP(int84)

#undef WRAP
