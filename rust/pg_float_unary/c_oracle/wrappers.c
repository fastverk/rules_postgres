#include <stdint.h>
#include <stdbool.h>

typedef uintptr_t Datum;
typedef struct FunctionCallInfoBaseData FunctionCallInfoBaseData;

#define WRAP(name)                                                            \
    extern Datum name##_orig(FunctionCallInfoBaseData *fcinfo);              \
    Datum c_##name(FunctionCallInfoBaseData *fcinfo) {                       \
        return name##_orig(fcinfo);                                          \
    }

WRAP(float4abs)  WRAP(float4um)
WRAP(float8abs)  WRAP(float8um)

#undef WRAP
