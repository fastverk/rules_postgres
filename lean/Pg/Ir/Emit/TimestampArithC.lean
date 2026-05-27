import Pg.Ir.Emit.TimestampArithCommon

/-!
# Pg.Ir.Emit.TimestampArithC — codegen real-style Postgres C source for the
timestamp-arithmetic V1 fmgr cluster.

Sister to `Pg.Ir.Emit.TimestampArith` (which emits Rust). Body shapes follow
Postgres' real source (`src/backend/utils/adt/timestamp.c`) to enable
AST-diff grounding.

Vendored functions:
1. `timestamp_pl_interval` (timestamp + interval → timestamp)
2. `timestamp_mi_interval` (timestamp - interval → timestamp)
3. `timestamp_mi` (timestamp - timestamp → interval)

All handle infinity sentinels; timestamp_mi allocates the result via palloc.
-/

namespace Pg.Ir.Emit.TimestampArithC

open Pg.Ir.Emit.TimestampArith (TimestampOp TimestampFamily families)

def renderCPlIntervalBody : String :=
  "\tTimestamp\ttimestamp = PG_GETARG_TIMESTAMP(0);\n" ++
  "\tInterval   *span = PG_GETARG_INTERVAL_P(1);\n" ++
  "\tTimestamp\tresult;\n\n" ++
  "\t/*\n" ++
  "\t * Handle infinities.\n" ++
  "\t *\n" ++
  "\t * We treat anything that amounts to \"infinity - infinity\" as an error,\n" ++
  "\t * since the timestamp type has nothing equivalent to NaN.\n" ++
  "\t */\n" ++
  "\tif (INTERVAL_IS_NOBEGIN(span))\n" ++
  "\t{\n" ++
  "\t\tif (TIMESTAMP_IS_NOEND(timestamp))\n" ++
  "\t\t\tereport(ERROR,\n" ++
  "\t\t\t\t\t(errcode(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE),\n" ++
  "\t\t\t\t\t errmsg(\"timestamp out of range\")));\n" ++
  "\t\telse\n" ++
  "\t\t\tTIMESTAMP_NOBEGIN(result);\n" ++
  "\t}\n" ++
  "\telse if (INTERVAL_IS_NOEND(span))\n" ++
  "\t{\n" ++
  "\t\tif (TIMESTAMP_IS_NOBEGIN(timestamp))\n" ++
  "\t\t\tereport(ERROR,\n" ++
  "\t\t\t\t\t(errcode(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE),\n" ++
  "\t\t\t\t\t errmsg(\"timestamp out of range\")));\n" ++
  "\t\telse\n" ++
  "\t\t\tTIMESTAMP_NOEND(result);\n" ++
  "\t}\n" ++
  "\telse if (TIMESTAMP_NOT_FINITE(timestamp))\n" ++
  "\t\tresult = timestamp;\n" ++
  "\telse\n" ++
  "\t{\n" ++
  "\t\tif (span->month != 0)\n" ++
  "\t\t{\n" ++
  "\t\t\tstruct pg_tm tt,\n" ++
  "\t\t\t\t\t   *tm = &tt;\n" ++
  "\t\t\tfsec_t\t\tfsec;\n\n" ++
  "\t\t\tif (timestamp2tm(timestamp, NULL, tm, &fsec, NULL, NULL) != 0)\n" ++
  "\t\t\t\tereport(ERROR,\n" ++
  "\t\t\t\t\t\t(errcode(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE),\n" ++
  "\t\t\t\t\t\t errmsg(\"timestamp out of range\")));\n\n" ++
  "\t\t\tif (pg_add_s32_overflow(tm->tm_mon, span->month, &tm->tm_mon))\n" ++
  "\t\t\t\tereport(ERROR,\n" ++
  "\t\t\t\t\t\t(errcode(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE),\n" ++
  "\t\t\t\t\t\t errmsg(\"timestamp out of range\")));\n" ++
  "\t\t\tif (tm->tm_mon > MONTHS_PER_YEAR)\n" ++
  "\t\t\t{\n" ++
  "\t\t\t\ttm->tm_year += (tm->tm_mon - 1) / MONTHS_PER_YEAR;\n" ++
  "\t\t\t\ttm->tm_mon = ((tm->tm_mon - 1) % MONTHS_PER_YEAR) + 1;\n" ++
  "\t\t\t}\n" ++
  "\t\t\telse if (tm->tm_mon < 1)\n" ++
  "\t\t\t{\n" ++
  "\t\t\t\ttm->tm_year += tm->tm_mon / MONTHS_PER_YEAR - 1;\n" ++
  "\t\t\t\ttm->tm_mon = tm->tm_mon % MONTHS_PER_YEAR + MONTHS_PER_YEAR;\n" ++
  "\t\t\t}\n\n" ++
  "\t\t\t/* adjust for end of month boundary problems... */\n" ++
  "\t\t\tif (tm->tm_mday > day_tab[isleap(tm->tm_year)][tm->tm_mon - 1])\n" ++
  "\t\t\t\ttm->tm_mday = (day_tab[isleap(tm->tm_year)][tm->tm_mon - 1]);\n\n" ++
  "\t\t\tif (tm2timestamp(tm, fsec, NULL, &timestamp) != 0)\n" ++
  "\t\t\t\tereport(ERROR,\n" ++
  "\t\t\t\t\t\t(errcode(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE),\n" ++
  "\t\t\t\t\t\t errmsg(\"timestamp out of range\")));\n" ++
  "\t\t}\n\n" ++
  "\t\tif (span->day != 0)\n" ++
  "\t\t{\n" ++
  "\t\t\tstruct pg_tm tt,\n" ++
  "\t\t\t\t\t   *tm = &tt;\n" ++
  "\t\t\tfsec_t\t\tfsec;\n" ++
  "\t\t\tint\t\t\tjulian;\n\n" ++
  "\t\t\tif (timestamp2tm(timestamp, NULL, tm, &fsec, NULL, NULL) != 0)\n" ++
  "\t\t\t\tereport(ERROR,\n" ++
  "\t\t\t\t\t\t(errcode(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE),\n" ++
  "\t\t\t\t\t\t errmsg(\"timestamp out of range\")));\n\n" ++
  "\t\t\t/*\n" ++
  "\t\t\t * Add days by converting to and from Julian.  We need an overflow\n" ++
  "\t\t\t * check here since j2date expects a non-negative integer input.\n" ++
  "\t\t\t */\n" ++
  "\t\t\tjulian = date2j(tm->tm_year, tm->tm_mon, tm->tm_mday);\n" ++
  "\t\t\tif (pg_add_s32_overflow(julian, span->day, &julian) ||\n" ++
  "\t\t\t\tjulian < 0)\n" ++
  "\t\t\t\tereport(ERROR,\n" ++
  "\t\t\t\t\t\t(errcode(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE),\n" ++
  "\t\t\t\t\t\t errmsg(\"timestamp out of range\")));\n" ++
  "\t\t\tj2date(julian, &tm->tm_year, &tm->tm_mon, &tm->tm_mday);\n\n" ++
  "\t\t\tif (tm2timestamp(tm, fsec, NULL, &timestamp) != 0)\n" ++
  "\t\t\t\tereport(ERROR,\n" ++
  "\t\t\t\t\t\t(errcode(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE),\n" ++
  "\t\t\t\t\t\t errmsg(\"timestamp out of range\")));\n" ++
  "\t\t}\n\n" ++
  "\t\tif (pg_add_s64_overflow(timestamp, span->time, &timestamp))\n" ++
  "\t\t\tereport(ERROR,\n" ++
  "\t\t\t\t\t(errcode(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE),\n" ++
  "\t\t\t\t\t errmsg(\"timestamp out of range\")));\n\n" ++
  "\t\tif (!IS_VALID_TIMESTAMP(timestamp))\n" ++
  "\t\t\tereport(ERROR,\n" ++
  "\t\t\t\t\t(errcode(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE),\n" ++
  "\t\t\t\t\t errmsg(\"timestamp out of range\")));\n\n" ++
  "\t\tresult = timestamp;\n" ++
  "\t}\n\n" ++
  "\tPG_RETURN_TIMESTAMP(result);\n"

def renderCMiIntervalBody : String :=
  "\tTimestamp\ttimestamp = PG_GETARG_TIMESTAMP(0);\n" ++
  "\tInterval   *span = PG_GETARG_INTERVAL_P(1);\n" ++
  "\tInterval\ttspan;\n\n" ++
  "\tinterval_um_internal(span, &tspan);\n\n" ++
  "\treturn DirectFunctionCall2(timestamp_pl_interval,\n" ++
  "\t\t\t\t\t   TimestampGetDatum(timestamp),\n" ++
  "\t\t\t\t\t   PointerGetDatum(&tspan));\n"

def renderCMiBody : String :=
  "\tTimestamp\tdt1 = PG_GETARG_TIMESTAMP(0);\n" ++
  "\tTimestamp\tdt2 = PG_GETARG_TIMESTAMP(1);\n" ++
  "\tInterval   *result;\n\n" ++
  "\tresult = (Interval *) palloc(sizeof(Interval));\n\n" ++
  "\t/*\n" ++
  "\t * Handle infinities.\n" ++
  "\t *\n" ++
  "\t * We treat anything that amounts to \"infinity - infinity\" as an error,\n" ++
  "\t * since the interval type has nothing equivalent to NaN.\n" ++
  "\t */\n" ++
  "\tif (TIMESTAMP_NOT_FINITE(dt1) || TIMESTAMP_NOT_FINITE(dt2))\n" ++
  "\t{\n" ++
  "\t\tif (TIMESTAMP_IS_NOBEGIN(dt1))\n" ++
  "\t\t{\n" ++
  "\t\t\tif (TIMESTAMP_IS_NOBEGIN(dt2))\n" ++
  "\t\t\t\tereport(ERROR,\n" ++
  "\t\t\t\t\t\t(errcode(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE),\n" ++
  "\t\t\t\t\t\t errmsg(\"interval out of range\")));\n" ++
  "\t\t\telse\n" ++
  "\t\t\t\tINTERVAL_NOBEGIN(result);\n" ++
  "\t\t}\n" ++
  "\t\telse if (TIMESTAMP_IS_NOEND(dt1))\n" ++
  "\t\t{\n" ++
  "\t\t\tif (TIMESTAMP_IS_NOEND(dt2))\n" ++
  "\t\t\t\tereport(ERROR,\n" ++
  "\t\t\t\t\t\t(errcode(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE),\n" ++
  "\t\t\t\t\t\t errmsg(\"interval out of range\")));\n" ++
  "\t\t\telse\n" ++
  "\t\t\t\tINTERVAL_NOEND(result);\n" ++
  "\t\t}\n" ++
  "\t\telse if (TIMESTAMP_IS_NOBEGIN(dt2))\n" ++
  "\t\t\tINTERVAL_NOEND(result);\n" ++
  "\t\telse\t\t\t\t/* TIMESTAMP_IS_NOEND(dt2) */\n" ++
  "\t\t\tINTERVAL_NOBEGIN(result);\n\n" ++
  "\t\tPG_RETURN_INTERVAL_P(result);\n" ++
  "\t}\n\n" ++
  "\tif (unlikely(pg_sub_s64_overflow(dt1, dt2, &result->time)))\n" ++
  "\t\tereport(ERROR,\n" ++
  "\t\t\t\t(errcode(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE),\n" ++
  "\t\t\t\t errmsg(\"interval out of range\")));\n\n" ++
  "\tresult->month = 0;\n" ++
  "\tresult->day = 0;\n\n" ++
  "\t/*----------\n" ++
  "\t *\tThis is wrong, but removing it breaks a lot of regression tests.\n" ++
  "\t *\tFor example:\n" ++
  "\t *\n" ++
  "\t *\ttest=> SET timezone = 'EST5EDT';\n" ++
  "\t *\ttest=> SELECT\n" ++
  "\t *\ttest-> ('2005-10-30 13:22:00-05'::timestamptz -\n" ++
  "\t *\ttest(> '2005-10-29 13:22:00-04'::timestamptz);\n" ++
  "\t *\t?column?\n" ++
  "\t *\t----------------\n" ++
  "\t *\t 1 day 01:00:00\n" ++
  "\t *\t (1 row)\n" ++
  "\t *\n" ++
  "\t *\tso adding that to the first timestamp gets:\n" ++
  "\t *\n" ++
  "\t *\t test=> SELECT\n" ++
  "\t *\t test-> ('2005-10-29 13:22:00-04'::timestamptz +\n" ++
  "\t *\t test(> ('2005-10-30 13:22:00-05'::timestamptz -\n" ++
  "\t *\t test(>  '2005-10-29 13:22:00-04'::timestamptz)) at time zone 'EST';\n" ++
  "\t *\t  timezone\n" ++
  "\t *\t--------------------\n" ++
  "\t *\t2005-10-30 14:22:00\n" ++
  "\t *\t(1 row)\n" ++
  "\t *\t----------\n" ++
  "\t */\n" ++
  "\tresult = DatumGetIntervalP(DirectFunctionCall1(interval_justify_hours,\n" ++
  "\t\t\t\t\t\t\t   IntervalPGetDatum(result)));\n\n" ++
  "\tPG_RETURN_INTERVAL_P(result);\n"

def renderCFn (fam : TimestampFamily) : String :=
  let body := match fam.op with
    | .pl_interval => renderCPlIntervalBody
    | .mi_interval => renderCMiIntervalBody
    | .mi          => renderCMiBody
  "Datum\n" ++
  fam.fnName ++ "(PG_FUNCTION_ARGS)\n" ++
  "{\n" ++
  body ++
  "}\n"

def cSource : String :=
  let header :=
    "/*\n" ++
    " * Generated by `rules_postgres/lean/Pg/Ir/Emit/TimestampArithC.lean`.\n" ++
    " * DO NOT EDIT BY HAND. Regenerate via\n" ++
    " * `rules_postgres/tools/regen/check-timestamp-arith-grounding.sh`.\n" ++
    " *\n" ++
    " * This file exists to be `clang -ast-dump=json`-ed alongside\n" ++
    " * real `src/backend/utils/adt/timestamp.c` and structurally diffed\n" ++
    " * function-by-function. A passing diff means the Lean spec\n" ++
    " * captures the same algorithm Postgres wrote.\n" ++
    " */\n" ++
    "\n" ++
    "#include \"postgres.h\"\n" ++
    "#include \"utils/timestamp.h\"\n" ++
    "#include \"utils/fmgrprotos.h\"\n" ++
    "#include \"utils/datetime.h\"\n" ++
    "\n" ++
    "/* Forward declarations for datetime helper functions from timestamp.c */\n" ++
    "extern int timestamp2tm(Timestamp timestamp, void *tz, struct pg_tm *tm, int32 *fsec, void *tzn, void *attimezone);\n" ++
    "extern int tm2timestamp(struct pg_tm *tm, int32 fsec, void *tz, Timestamp *result);\n" ++
    "extern int date2j(int y, int m, int d);\n" ++
    "extern void j2date(int jd, int *y, int *m, int *d);\n" ++
    "extern Datum interval_um_internal(PG_FUNCTION_ARGS);\n" ++
    "extern Datum interval_justify_hours(PG_FUNCTION_ARGS);\n"
  Id.run do
    let mut s := header
    for fam in families do
      s := s ++ "\n" ++ renderCFn fam
    return s

end Pg.Ir.Emit.TimestampArithC

def main : IO Unit := IO.print Pg.Ir.Emit.TimestampArithC.cSource
