import Pg.Ir.Emit.IntervalCommon

/-!
# Pg.Ir.Emit.IntervalC — emit real-style Postgres C for the interval
arithmetic cluster (AST grounding gate companion to `Interval.lean`).

Mirrors the IntCmpC / IntHashC / IntArithC pattern: emit C source
byte-equivalent (modulo whitespace clang normalizes away) to real
Postgres source at `src/backend/utils/adt/timestamp.c`, then
`clang -ast-dump=json` both sides and diff per-function.
-/

namespace Pg.Ir.Emit.Interval.C

open Pg.Ir.Emit.Interval (IntervalBody IntervalFamily families)

/-- Render the `<fn>_internal` body for unaryNegate. -/
def renderUnaryNegateInternalBody : String :=
  "\tif (INTERVAL_IS_NOBEGIN(interval))\n" ++
  "\t\tINTERVAL_NOEND(result);\n" ++
  "\telse if (INTERVAL_IS_NOEND(interval))\n" ++
  "\t\tINTERVAL_NOBEGIN(result);\n" ++
  "\telse\n" ++
  "\t{\n" ++
  "\t\t/* Negate each field, guarding against overflow */\n" ++
  "\t\tif (pg_sub_s64_overflow(INT64CONST(0), interval->time, &result->time) ||\n" ++
  "\t\t\tpg_sub_s32_overflow(0, interval->day, &result->day) ||\n" ++
  "\t\t\tpg_sub_s32_overflow(0, interval->month, &result->month) ||\n" ++
  "\t\t\tINTERVAL_NOT_FINITE(result))\n" ++
  "\t\t\tereport(ERROR,\n" ++
  "\t\t\t\t\t(errcode(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE),\n" ++
  "\t\t\t\t\t errmsg(\"interval out of range\")));\n" ++
  "\t}\n"

/-- Render the `<fn>_internal` C function for unaryNegate. -/
def renderCUnaryNegateInternal (fam : IntervalFamily) : String :=
  "static void\n" ++
  fam.fnName ++ "_internal(const Interval *interval, Interval *result)\n" ++
  "{\n" ++
  renderUnaryNegateInternalBody ++
  "}\n"

/-- Render the `<fn>_internal` C function if needed. -/
def renderCInternal (fam : IntervalFamily) : String :=
  match fam.body with
  | .unaryNegate => renderCUnaryNegateInternal fam
  | _ => "" -- binaryPl and binaryMi don't have separate internal helpers

/-- Render the fmgr stub for unaryNegate. -/
def renderCUnaryNegateFmgrStub (fam : IntervalFamily) : String :=
  "Datum\n" ++
  fam.fnName ++ "(PG_FUNCTION_ARGS)\n" ++
  "{\n" ++
  "\tInterval   *interval = PG_GETARG_INTERVAL_P(0);\n" ++
  "\tInterval   *result;\n" ++
  "\n" ++
  "\tresult = (Interval *) palloc(sizeof(Interval));\n" ++
  "\t" ++ fam.fnName ++ "_internal(interval, result);\n" ++
  "\n" ++
  "\tPG_RETURN_INTERVAL_P(result);\n" ++
  "}\n"

/-- Render the fmgr stub for binaryPl. -/
def renderCBinaryPlFmgrStub (fam : IntervalFamily) : String :=
  "Datum\n" ++
  fam.fnName ++ "(PG_FUNCTION_ARGS)\n" ++
  "{\n" ++
  "\tInterval   *span1 = PG_GETARG_INTERVAL_P(0);\n" ++
  "\tInterval   *span2 = PG_GETARG_INTERVAL_P(1);\n" ++
  "\tInterval   *result;\n" ++
  "\n" ++
  "\tresult = (Interval *) palloc(sizeof(Interval));\n" ++
  "\n" ++
  "\t/*\n" ++
  "\t * Handle infinities.\n" ++
  "\t *\n" ++
  "\t * We treat anything that amounts to \"infinity - infinity\" as an error,\n" ++
  "\t * since the interval type has nothing equivalent to NaN.\n" ++
  "\t */\n" ++
  "\tif (INTERVAL_IS_NOBEGIN(span1))\n" ++
  "\t{\n" ++
  "\t\tif (INTERVAL_IS_NOEND(span2))\n" ++
  "\t\t\tereport(ERROR,\n" ++
  "\t\t\t\t\t(errcode(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE),\n" ++
  "\t\t\t\t\t errmsg(\"interval out of range\")));\n" ++
  "\t\telse\n" ++
  "\t\t\tINTERVAL_NOBEGIN(result);\n" ++
  "\t}\n" ++
  "\telse if (INTERVAL_IS_NOEND(span1))\n" ++
  "\t{\n" ++
  "\t\tif (INTERVAL_IS_NOBEGIN(span2))\n" ++
  "\t\t\tereport(ERROR,\n" ++
  "\t\t\t\t\t(errcode(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE),\n" ++
  "\t\t\t\t\t errmsg(\"interval out of range\")));\n" ++
  "\t\telse\n" ++
  "\t\t\tINTERVAL_NOEND(result);\n" ++
  "\t}\n" ++
  "\telse if (INTERVAL_NOT_FINITE(span2))\n" ++
  "\t\tmemcpy(result, span2, sizeof(Interval));\n" ++
  "\telse\n" ++
  "\t\tfinite_interval_pl(span1, span2, result);\n" ++
  "\n" ++
  "\tPG_RETURN_INTERVAL_P(result);\n" ++
  "}\n"

/-- Render the fmgr stub for binaryMi. -/
def renderCBinaryMiFmgrStub (fam : IntervalFamily) : String :=
  "Datum\n" ++
  fam.fnName ++ "(PG_FUNCTION_ARGS)\n" ++
  "{\n" ++
  "\tInterval   *span1 = PG_GETARG_INTERVAL_P(0);\n" ++
  "\tInterval   *span2 = PG_GETARG_INTERVAL_P(1);\n" ++
  "\tInterval   *result;\n" ++
  "\n" ++
  "\tresult = (Interval *) palloc(sizeof(Interval));\n" ++
  "\n" ++
  "\t/*\n" ++
  "\t * Handle infinities.\n" ++
  "\t *\n" ++
  "\t * We treat anything that amounts to \"infinity - infinity\" as an error,\n" ++
  "\t * since the interval type has nothing equivalent to NaN.\n" ++
  "\t */\n" ++
  "\tif (INTERVAL_IS_NOBEGIN(span1))\n" ++
  "\t{\n" ++
  "\t\tif (INTERVAL_IS_NOBEGIN(span2))\n" ++
  "\t\t\tereport(ERROR,\n" ++
  "\t\t\t\t\t(errcode(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE),\n" ++
  "\t\t\t\t\t errmsg(\"interval out of range\")));\n" ++
  "\t\telse\n" ++
  "\t\t\tINTERVAL_NOBEGIN(result);\n" ++
  "\t}\n" ++
  "\telse if (INTERVAL_IS_NOEND(span1))\n" ++
  "\t{\n" ++
  "\t\tif (INTERVAL_IS_NOEND(span2))\n" ++
  "\t\t\tereport(ERROR,\n" ++
  "\t\t\t\t\t(errcode(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE),\n" ++
  "\t\t\t\t\t errmsg(\"interval out of range\")));\n" ++
  "\t\telse\n" ++
  "\t\t\tINTERVAL_NOEND(result);\n" ++
  "\t}\n" ++
  "\telse if (INTERVAL_IS_NOBEGIN(span2))\n" ++
  "\t\tINTERVAL_NOEND(result);\n" ++
  "\telse if (INTERVAL_IS_NOEND(span2))\n" ++
  "\t\tINTERVAL_NOBEGIN(result);\n" ++
  "\telse\n" ++
  "\t\tfinite_interval_mi(span1, span2, result);\n" ++
  "\n" ++
  "\tPG_RETURN_INTERVAL_P(result);\n" ++
  "}\n"

/-- Render the fmgr stub for the given family. -/
def renderCFmgrStub (fam : IntervalFamily) : String :=
  match fam.body with
  | .unaryNegate => renderCUnaryNegateFmgrStub fam
  | .binaryPl => renderCBinaryPlFmgrStub fam
  | .binaryMi => renderCBinaryMiFmgrStub fam

def cSource : String :=
  let header :=
    "/*\n" ++
    " * Generated by `rules_postgres/lean/Pg/Ir/Emit/IntervalC.lean`.\n" ++
    " * DO NOT EDIT BY HAND. Regenerate via\n" ++
    " * `rules_postgres/tools/regen/check-interval-grounding.sh`.\n" ++
    " *\n" ++
    " * Real-style C emit for AST-grounding against\n" ++
    " * `src/backend/utils/adt/timestamp.c`.\n" ++
    " */\n" ++
    "\n" ++
    "#include \"postgres.h\"\n" ++
    "#include \"common/int.h\"\n" ++
    "#include \"datatype/timestamp.h\"\n" ++
    "#include \"utils/fmgrprotos.h\"\n" ++
    "#include \"utils/timestamp.h\"\n"
  let helpers :=
    "\n/* Finite-interval helpers (called from pl/mi dispatchers). */\n" ++
    "\nstatic void\n" ++
    "finite_interval_pl(const Interval *span1, const Interval *span2, Interval *result)\n" ++
    "{\n" ++
    "\tAssert(!INTERVAL_NOT_FINITE(span1));\n" ++
    "\tAssert(!INTERVAL_NOT_FINITE(span2));\n" ++
    "\n" ++
    "\tif (pg_add_s32_overflow(span1->month, span2->month, &result->month) ||\n" ++
    "\t\tpg_add_s32_overflow(span1->day, span2->day, &result->day) ||\n" ++
    "\t\tpg_add_s64_overflow(span1->time, span2->time, &result->time) ||\n" ++
    "\t\tINTERVAL_NOT_FINITE(result))\n" ++
    "\t\tereport(ERROR,\n" ++
    "\t\t\t\t(errcode(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE),\n" ++
    "\t\t\t\t errmsg(\"interval out of range\")));\n" ++
    "}\n" ++
    "\nstatic void\n" ++
    "finite_interval_mi(const Interval *span1, const Interval *span2, Interval *result)\n" ++
    "{\n" ++
    "\tAssert(!INTERVAL_NOT_FINITE(span1));\n" ++
    "\tAssert(!INTERVAL_NOT_FINITE(span2));\n" ++
    "\n" ++
    "\tif (pg_sub_s32_overflow(span1->month, span2->month, &result->month) ||\n" ++
    "\t\tpg_sub_s32_overflow(span1->day, span2->day, &result->day) ||\n" ++
    "\t\tpg_sub_s64_overflow(span1->time, span2->time, &result->time) ||\n" ++
    "\t\tINTERVAL_NOT_FINITE(result))\n" ++
    "\t\tereport(ERROR,\n" ++
    "\t\t\t\t(errcode(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE),\n" ++
    "\t\t\t\t errmsg(\"interval out of range\")));\n" ++
    "}\n"
  let body :=
    Id.run do
      let mut s := ""
      for fam in families do
        s := s ++ "\n" ++ renderCInternal fam ++ "\n" ++ renderCFmgrStub fam
      return s
  header ++ helpers ++ body

end Pg.Ir.Emit.Interval.C

def main : IO Unit := IO.print Pg.Ir.Emit.Interval.C.cSource
