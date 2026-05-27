import Pg.Ir.Emit.CashArithCommon

/-!
# Pg.Ir.Emit.CashArithC — codegen real-style Postgres C source for the
cash-arithmetic V1 fmgr cluster.

Sister to `Pg.Ir.Emit.CashArith` (which emits Rust). Same
`CashFamily` table, but renders Postgres-idiomatic C so the result
AST-diffs cleanly against real `src/backend/utils/adt/cash.c`.

Body shapes:

**Overflow-checked** (cash_pl, cash_mi, cash_mul_int4):
```
Datum
<fn>(PG_FUNCTION_ARGS)
{
	Cash		arg1 = <GETARG1>(0);
	<T2>		arg2 = <GETARG2>(1);
	Cash		result;

	if (unlikely(pg_<op>_s64_overflow(arg1, <cast2>arg2, &result)))
		ereport(ERROR,
				(errcode(ERRCODE_NUMERIC_VALUE_OUT_OF_RANGE),
				 errmsg("money out of range")));
	<RETURN>(result);
}
```

**Comparison** (cashlarger, cashsmaller):
```
Datum
<fn>(PG_FUNCTION_ARGS)
{
	Cash		c1 = <GETARG1>(0);
	Cash		c2 = <GETARG2>(1);
	Cash		result;

	result = (c1 <op> c2) ? c1 : c2;
	<RETURN>(result);
}
```

**Division** (cash_div_int4):
```
Datum
<fn>(PG_FUNCTION_ARGS)
{
	Cash		c = <GETARG1>(0);
	int32		i = <GETARG2>(1);

	if (unlikely(i == 0))
		ereport(ERROR,
				(errcode(ERRCODE_DIVISION_BY_ZERO),
				 errmsg("division by zero")));
	<RETURN>(c / (int64) i);
}
```
-/

namespace Pg.Ir.Emit.CashArithC

open Pg.Ir.Emit.CashArith (BodyShape CashFamily families)

/-- C `pg_<op>_s64_overflow` primitive name. -/
def overflowPrim (op : Pg.Ir.Emit.CashArith.CashOp) : String :=
  "pg_" ++ op.primitive ++ "_s64_overflow"

/-- Render the static inline helper for overflow-checked ops. -/
def renderCHelperOverflow (fam : CashFamily) : String :=
  let prim := overflowPrim fam.op
  "static inline " ++ fam.cResultTy ++ "\n" ++
  fam.helperName ++ "(" ++ fam.cWidth1 ++ " c1, " ++ fam.helperArg2Type ++ " c2)\n" ++
  "{\n" ++
  "\t" ++ fam.cResultTy ++ "\t\tres;\n\n" ++
  "\tif (unlikely(" ++ prim ++ "(c1, " ++ fam.cCast2 ++ "c2, &res)))\n" ++
  "\t\tereport(ERROR,\n" ++
  "\t\t\t\t(errcode(ERRCODE_NUMERIC_VALUE_OUT_OF_RANGE),\n" ++
  "\t\t\t\t errmsg(\"money out of range\")));\n\n" ++
  "\treturn res;\n" ++
  "}\n"

/-- Render the static inline helper for division. -/
def renderCHelperDiv (fam : CashFamily) : String :=
  "static inline " ++ fam.cResultTy ++ "\n" ++
  fam.helperName ++ "(" ++ fam.cWidth1 ++ " c, " ++ fam.helperArg2Type ++ " i)\n" ++
  "{\n" ++
  "\tif (unlikely(i == 0))\n" ++
  "\t\tereport(ERROR,\n" ++
  "\t\t\t\t(errcode(ERRCODE_DIVISION_BY_ZERO),\n" ++
  "\t\t\t\t errmsg(\"division by zero\")));\n\n" ++
  "\treturn c / (int64) i;\n" ++
  "}\n"

/-- Render the helper based on body shape (if needed). -/
def renderCHelper (fam : CashFamily) : String :=
  if fam.helperName == "" then ""
  else
    match fam.op.bodyShape with
    | .binaryAddSub => renderCHelperOverflow fam
    | .binaryMulInt => renderCHelperOverflow fam
    | .binaryDivInt => renderCHelperDiv fam
    | .binaryCompare => ""

/-- Render overflow-checked body (now a stub that calls helper). -/
def renderCBodyOverflow (fam : CashFamily) : String :=
  if fam.helperName == "" then
    let prim := overflowPrim fam.op
    "\t" ++ fam.cWidth1   ++ "\t\targ1 = " ++ fam.pgGetArg1 ++ "(0);\n" ++
    "\t" ++ fam.cWidth2   ++ "\t\targ2 = " ++ fam.pgGetArg2 ++ "(1);\n" ++
    "\t" ++ fam.cResultTy ++ "\t\tresult;\n\n" ++
    "\tif (unlikely(" ++ prim ++ "(arg1, " ++ fam.cCast2 ++ "arg2, &result)))\n" ++
    "\t\tereport(ERROR,\n" ++
    "\t\t\t\t(errcode(ERRCODE_NUMERIC_VALUE_OUT_OF_RANGE),\n" ++
    "\t\t\t\t errmsg(\"money out of range\")));\n" ++
    "\t" ++ fam.pgReturn ++ "(result);\n"
  else
    "\tCash\t\targ1 = " ++ fam.pgGetArg1 ++ "(0);\n" ++
    "\t" ++ fam.cWidth2 ++ "\t\targ2 = " ++ fam.pgGetArg2 ++ "(1);\n\n" ++
    "\t" ++ fam.pgReturn ++ "(" ++ fam.helperName ++ "(arg1, " ++ fam.cCast2 ++ "arg2));\n"

/-- Render comparison body. -/
def renderCBodyCompare (fam : CashFamily) : String :=
  let cmp := if fam.fnName == "cashlarger" then ">" else "<"
  "\tCash\t\tc1 = " ++ fam.pgGetArg1 ++ "(0);\n" ++
  "\tCash\t\tc2 = " ++ fam.pgGetArg2 ++ "(1);\n" ++
  "\tCash\t\tresult;\n\n" ++
  "\tresult = (c1 " ++ cmp ++ " c2) ? c1 : c2;\n\n" ++
  "\t" ++ fam.pgReturn ++ "(result);\n"

/-- Render division body (stub that calls helper if one exists). -/
def renderCBodyDivInt (fam : CashFamily) : String :=
  if fam.helperName == "" then
    "\tCash\t\tc = " ++ fam.pgGetArg1 ++ "(0);\n" ++
    "\tint32\t\ti = " ++ fam.pgGetArg2 ++ "(1);\n\n" ++
    "\tif (unlikely(i == 0))\n" ++
    "\t\tereport(ERROR,\n" ++
    "\t\t\t\t(errcode(ERRCODE_DIVISION_BY_ZERO),\n" ++
    "\t\t\t\t errmsg(\"division by zero\")));\n" ++
    "\t" ++ fam.pgReturn ++ "(c / (int64) i);\n"
  else
    "\tCash\t\tc = " ++ fam.pgGetArg1 ++ "(0);\n" ++
    "\t" ++ fam.cWidth2 ++ "\t\ti = " ++ fam.pgGetArg2 ++ "(1);\n\n" ++
    "\t" ++ fam.pgReturn ++ "(" ++ fam.helperName ++ "(c, (int64) i));\n"

/-- Render the appropriate C body based on body shape. -/
def renderCBody (fam : CashFamily) : String :=
  match fam.op.bodyShape with
  | .binaryAddSub => renderCBodyOverflow fam
  | .binaryMulInt => renderCBodyOverflow fam
  | .binaryCompare => renderCBodyCompare fam
  | .binaryDivInt => renderCBodyDivInt fam

def renderCFn (fam : CashFamily) : String :=
  let helper := renderCHelper fam
  let helper_sep := if helper == "" then "" else "\n"
  helper ++ helper_sep ++
  "Datum\n" ++
  fam.fnName ++ "(PG_FUNCTION_ARGS)\n" ++
  "{\n" ++
  renderCBody fam ++
  "}\n"

def cSource : String :=
  let header :=
    "/*\n" ++
    " * Generated by `rules_postgres/lean/Pg/Ir/Emit/CashArithC.lean`.\n" ++
    " * DO NOT EDIT BY HAND. Regenerate via\n" ++
    " * `rules_postgres/tools/regen/check-cash-arith-grounding.sh`.\n" ++
    " *\n" ++
    " * This file exists to be `clang -ast-dump=json`-ed alongside\n" ++
    " * real `src/backend/utils/adt/cash.c` and structurally\n" ++
    " * diffed function-by-function. A passing diff means the Lean spec\n" ++
    " * captures the same algorithm Postgres wrote.\n" ++
    " */\n" ++
    "\n" ++
    "#include \"postgres.h\"\n" ++
    "\n" ++
    "#include <limits.h>\n" ++
    "#include <ctype.h>\n" ++
    "#include <math.h>\n" ++
    "\n" ++
    "#include \"common/int.h\"\n" ++
    "#include \"libpq/pqformat.h\"\n" ++
    "#include \"utils/builtins.h\"\n" ++
    "#include \"utils/cash.h\"\n" ++
    "#include \"utils/float.h\"\n" ++
    "#include \"utils/numeric.h\"\n" ++
    "#include \"utils/pg_locale.h\"\n"
  Id.run do
    let mut s := header
    for fam in families do
      s := s ++ "\n" ++ renderCFn fam
    return s

end Pg.Ir.Emit.CashArithC

def main : IO Unit := IO.print Pg.Ir.Emit.CashArithC.cSource
