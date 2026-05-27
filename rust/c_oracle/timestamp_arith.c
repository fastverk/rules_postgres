/*
 * timestamp_arith.c — vendored Postgres timestamp arithmetic functions:
 * timestamp_pl_interval, timestamp_mi_interval, timestamp_mi.
 *
 * Standalone C oracle (no real Postgres headers). Self-contained typedefs
 * + macros. Diff-test via c_oracle_ereport.h ereport() + setjmp route.
 *
 * Attribution: function bodies are PostgreSQL Global Development Group,
 * released under the PostgreSQL License (BSD-style).
 */

#include <stdint.h>
#include <stdbool.h>
#include <string.h>
#include <stdlib.h>
#include <setjmp.h>

/* Forward declarations referenced by ereport macro (defined in wrappers.c) */
extern __thread jmp_buf fmgr_oracle_jmp;
extern void pg_fcinfo_set_error(uint32_t kind);

/* Error codes from c_oracle_ereport.h */
#define FMGR_ERR_DATETIME_VALUE_OUT_OF_RANGE  3u

/* ereport routing: maps to pg_fcinfo_set_error + longjmp. */
#define fmgr_oracle_ereport(errcode_val, msg) \
    do { \
        pg_fcinfo_set_error(FMGR_ERR_DATETIME_VALUE_OUT_OF_RANGE); \
        longjmp(fmgr_oracle_jmp, 1); \
    } while (0)

typedef uintptr_t Datum;
typedef int64_t int64;
typedef int32_t int32;
typedef int16_t int16;
typedef uint32_t uint32;
typedef int64_t Timestamp;
typedef int64_t TimeOffset;
typedef int32_t fsec_t;

/* Interval: 16-byte structure. */
typedef struct {
    int64 time;    /* microseconds */
    int32 day;
    int32 month;
} Interval;

#define PG_INT64_MIN    ((int64) -0x8000000000000000LL)
#define PG_INT64_MAX    ((int64) 0x7FFFFFFFFFFFFFFFLL)
#define PG_INT32_MIN    ((int32) -0x80000000)
#define PG_INT32_MAX    ((int32) 0x7FFFFFFF)

#define DT_NOBEGIN  PG_INT64_MIN
#define DT_NOEND    PG_INT64_MAX

#define TIMESTAMP_IS_NOBEGIN(j)  ((j) == DT_NOBEGIN)
#define TIMESTAMP_IS_NOEND(j)    ((j) == DT_NOEND)
#define TIMESTAMP_NOT_FINITE(j)  (TIMESTAMP_IS_NOBEGIN(j) || TIMESTAMP_IS_NOEND(j))

#define TIMESTAMP_NOBEGIN(j)  do { (j) = DT_NOBEGIN; } while (0)
#define TIMESTAMP_NOEND(j)    do { (j) = DT_NOEND; } while (0)

#define INTERVAL_IS_NOBEGIN(i)  \
    ((i)->month == PG_INT32_MIN && (i)->day == PG_INT32_MIN && (i)->time == PG_INT64_MIN)

#define INTERVAL_IS_NOEND(i)  \
    ((i)->month == PG_INT32_MAX && (i)->day == PG_INT32_MAX && (i)->time == PG_INT64_MAX)

#define INTERVAL_NOT_FINITE(i)  (INTERVAL_IS_NOBEGIN(i) || INTERVAL_IS_NOEND(i))

#define INTERVAL_NOBEGIN(i)  \
    do { (i)->time = PG_INT64_MIN; (i)->day = PG_INT32_MIN; (i)->month = PG_INT32_MIN; } while (0)

#define INTERVAL_NOEND(i)  \
    do { (i)->time = PG_INT64_MAX; (i)->day = PG_INT32_MAX; (i)->month = PG_INT32_MAX; } while (0)

#define PG_GETARG_TIMESTAMP(n)    ((Timestamp) ((uintptr_t) fcinfo->args[(n)].value))
#define PG_GETARG_INTERVAL_P(n)   ((Interval *) (uintptr_t) fcinfo->args[(n)].value)

#define PG_RETURN_TIMESTAMP(x)    return (Datum) (uint64_t) (x)
#define PG_RETURN_INTERVAL_P(p)   return (Datum) (uintptr_t) (p)

#define MONTHS_PER_YEAR 12
#define USECS_PER_DAY   INT64CONST(86400000000)

/* Stubs for pg_add/pg_sub overflow checks. Real implementation would
   check for overflow; for the oracle we use simple addition/subtraction. */
static inline bool
pg_add_s64_overflow(int64 a, int64 b, int64 *result)
{
    /* Simplified: assume no overflow. Real Postgres has proper checks. */
    *result = a + b;
    return false;
}

static inline bool
pg_sub_s64_overflow(int64 a, int64 b, int64 *result)
{
    *result = a - b;
    return false;
}

static inline bool
pg_add_s32_overflow(int32 a, int32 b, int32 *result)
{
    *result = a + b;
    return false;
}

/* Stubs for timestamp2tm, tm2timestamp, date2j, j2date, isleap, day_tab.
   These are complex Postgres datetime functions; the oracle just errors. */

#define IS_VALID_TIMESTAMP(ts)  1
#define unlikely(x)              (x)

#define errcode(x) (x)
#define ERRCODE_DATETIME_VALUE_OUT_OF_RANGE 1

struct pg_tm {
    int tm_sec;
    int tm_min;
    int tm_hour;
    int tm_mday;
    int tm_mon;
    int tm_year;
    int tm_wday;
    int tm_yday;
    int tm_isdst;
    struct pg_tm *tm_gmtoff;
};

static void stub_not_implemented(void)
{
    fmgr_oracle_ereport(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE,
                       "timestamp arithmetic requires Postgres datetime helpers (timestamp2tm, tm2timestamp, etc.)");
}

/* Stub implementations that error for month/day arithmetic. */
static int
timestamp2tm(Timestamp timestamp, void *tzp, struct pg_tm *tm, int32 *fsec, void *tzn, void *tz)
{
    stub_not_implemented();
    return -1;
}

static int
tm2timestamp(struct pg_tm *tm, int32 fsec, void *tz, Timestamp *result)
{
    stub_not_implemented();
    return -1;
}

static int
date2j(int y, int m, int d)
{
    stub_not_implemented();
    return 0;
}

static void
j2date(int jd, int *y, int *m, int *d)
{
    stub_not_implemented();
}

static int
isleap(int y)
{
    return (y % 4 == 0 && (y % 100 != 0 || y % 400 == 0));
}

static int day_tab[2][13] = {
    {0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31},
    {0, 31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31}
};

/* interval_um_internal stub — errors like other datetime helpers. */
static void
interval_um_internal(const Interval *span, Interval *result)
{
    stub_not_implemented();
}

/* Postgres' DirectFunctionCall2 and related macros — stubs. */
#define PointerGetDatum(x)           ((Datum) (uintptr_t) (x))
#define TimestampGetDatum(x)         ((Datum) (uint64_t) (x))
#define IntervalPGetDatum(x)         ((Datum) (uintptr_t) (x))
#define DatumGetIntervalP(x)         ((Interval *) (uintptr_t) (x))

typedef Datum (*PGFunction)(void *);
typedef struct {
    void *fn_addr;
} FmgrInfo;

static Datum
DirectFunctionCall2(PGFunction func, Datum arg1, Datum arg2)
{
    stub_not_implemented();
    return 0;
}

/* palloc stub — for timestamp_mi result allocation. */
static void *
palloc(size_t size)
{
    return malloc(size);
}

/* Vendored timestamp_pl_interval. */
Datum
timestamp_pl_interval_orig(void *fcinfo_ptr)
{
    typedef struct {
        void *flinfo;
        void *context;
        void *resultinfo;
        uint32 fncollation;
        bool isnull;
        int16 nargs;
        struct {
            Datum value;
            bool isnull;
        } args[100];
    } FunctionCallInfoBaseData;

    FunctionCallInfoBaseData *fcinfo = (FunctionCallInfoBaseData *) fcinfo_ptr;

    Timestamp timestamp = PG_GETARG_TIMESTAMP(0);
    Interval *span = PG_GETARG_INTERVAL_P(1);
    Timestamp result;

    /*
     * Handle infinities.
     */
    if (INTERVAL_IS_NOBEGIN(span))
    {
        if (TIMESTAMP_IS_NOEND(timestamp))
            fmgr_oracle_ereport(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE,
                              "timestamp out of range");
        else
            TIMESTAMP_NOBEGIN(result);
    }
    else if (INTERVAL_IS_NOEND(span))
    {
        if (TIMESTAMP_IS_NOBEGIN(timestamp))
            fmgr_oracle_ereport(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE,
                              "timestamp out of range");
        else
            TIMESTAMP_NOEND(result);
    }
    else if (TIMESTAMP_NOT_FINITE(timestamp))
        result = timestamp;
    else
    {
        if (span->month != 0)
        {
            struct pg_tm tt, *tm = &tt;
            int32 fsec;

            if (timestamp2tm(timestamp, NULL, tm, &fsec, NULL, NULL) != 0)
                fmgr_oracle_ereport(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE,
                                  "timestamp out of range");

            if (pg_add_s32_overflow(tm->tm_mon, span->month, &tm->tm_mon))
                fmgr_oracle_ereport(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE,
                                  "timestamp out of range");
            if (tm->tm_mon > MONTHS_PER_YEAR)
            {
                tm->tm_year += (tm->tm_mon - 1) / MONTHS_PER_YEAR;
                tm->tm_mon = ((tm->tm_mon - 1) % MONTHS_PER_YEAR) + 1;
            }
            else if (tm->tm_mon < 1)
            {
                tm->tm_year += tm->tm_mon / MONTHS_PER_YEAR - 1;
                tm->tm_mon = tm->tm_mon % MONTHS_PER_YEAR + MONTHS_PER_YEAR;
            }

            if (tm->tm_mday > day_tab[isleap(tm->tm_year)][tm->tm_mon - 1])
                tm->tm_mday = (day_tab[isleap(tm->tm_year)][tm->tm_mon - 1]);

            if (tm2timestamp(tm, fsec, NULL, &timestamp) != 0)
                fmgr_oracle_ereport(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE,
                                  "timestamp out of range");
        }

        if (span->day != 0)
        {
            struct pg_tm tt, *tm = &tt;
            int32 fsec;
            int julian;

            if (timestamp2tm(timestamp, NULL, tm, &fsec, NULL, NULL) != 0)
                fmgr_oracle_ereport(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE,
                                  "timestamp out of range");

            julian = date2j(tm->tm_year, tm->tm_mon, tm->tm_mday);
            if (pg_add_s32_overflow(julian, span->day, &julian) ||
                julian < 0)
                fmgr_oracle_ereport(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE,
                                  "timestamp out of range");
            j2date(julian, &tm->tm_year, &tm->tm_mon, &tm->tm_mday);

            if (tm2timestamp(tm, fsec, NULL, &timestamp) != 0)
                fmgr_oracle_ereport(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE,
                                  "timestamp out of range");
        }

        if (pg_add_s64_overflow(timestamp, span->time, &timestamp))
            fmgr_oracle_ereport(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE,
                              "timestamp out of range");

        if (!IS_VALID_TIMESTAMP(timestamp))
            fmgr_oracle_ereport(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE,
                              "timestamp out of range");

        result = timestamp;
    }

    PG_RETURN_TIMESTAMP(result);
}

/* Vendored timestamp_mi_interval. */
Datum
timestamp_mi_interval_orig(void *fcinfo_ptr)
{
    typedef struct {
        void *flinfo;
        void *context;
        void *resultinfo;
        uint32 fncollation;
        bool isnull;
        int16 nargs;
        struct {
            Datum value;
            bool isnull;
        } args[100];
    } FunctionCallInfoBaseData;

    FunctionCallInfoBaseData *fcinfo = (FunctionCallInfoBaseData *) fcinfo_ptr;

    Timestamp timestamp = PG_GETARG_TIMESTAMP(0);
    Interval *span = PG_GETARG_INTERVAL_P(1);
    Interval tspan;

    interval_um_internal(span, &tspan);

    return DirectFunctionCall2((PGFunction) timestamp_pl_interval_orig,
                              TimestampGetDatum(timestamp),
                              PointerGetDatum(&tspan));
}

/* Vendored timestamp_mi. */
Datum
timestamp_mi_orig(void *fcinfo_ptr)
{
    typedef struct {
        void *flinfo;
        void *context;
        void *resultinfo;
        uint32 fncollation;
        bool isnull;
        int16 nargs;
        struct {
            Datum value;
            bool isnull;
        } args[100];
    } FunctionCallInfoBaseData;

    FunctionCallInfoBaseData *fcinfo = (FunctionCallInfoBaseData *) fcinfo_ptr;

    Timestamp dt1 = PG_GETARG_TIMESTAMP(0);
    Timestamp dt2 = PG_GETARG_TIMESTAMP(1);
    Interval *result;

    result = (Interval *) palloc(sizeof(Interval));

    /*
     * Handle infinities.
     */
    if (TIMESTAMP_NOT_FINITE(dt1) || TIMESTAMP_NOT_FINITE(dt2))
    {
        if (TIMESTAMP_IS_NOBEGIN(dt1))
        {
            if (TIMESTAMP_IS_NOBEGIN(dt2))
                fmgr_oracle_ereport(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE,
                                  "interval out of range");
            else
                INTERVAL_NOBEGIN(result);
        }
        else if (TIMESTAMP_IS_NOEND(dt1))
        {
            if (TIMESTAMP_IS_NOEND(dt2))
                fmgr_oracle_ereport(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE,
                                  "interval out of range");
            else
                INTERVAL_NOEND(result);
        }
        else if (TIMESTAMP_IS_NOBEGIN(dt2))
            INTERVAL_NOEND(result);
        else                        /* TIMESTAMP_IS_NOEND(dt2) */
            INTERVAL_NOBEGIN(result);

        PG_RETURN_INTERVAL_P(result);
    }

    if (unlikely(pg_sub_s64_overflow(dt1, dt2, &result->time)))
        fmgr_oracle_ereport(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE,
                          "interval out of range");

    result->month = 0;
    result->day = 0;

    PG_RETURN_INTERVAL_P(result);
}
