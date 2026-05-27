import Pg.Ir.Types
import Pg.Ir.Datum
import Pg.Ir.Cmp
import Pg.Ir.Emit.Common

/-!
# Pg.Ir.Emit.IntCmpC — codegen real-style Postgres C source.

Sibling to `Pg.Ir.Emit.IntCmp` (which emits Rust). Same `Family` table,
same comparison ops, but renders **Postgres-idiomatic C** instead of
Rust. The point: produce C that's structurally identical to what's
in real `src/backend/utils/adt/int.c` after clang preprocessing, so
that an AST-level diff between the two reveals algorithmic
divergence (vs. mere style differences).

Bound for Lean ↔ C structural-equivalence proofs:

```
real Postgres C ──compiled──► .a  ─┐
                                   ├─ behavioral diff-test (98K cases)
Lean-emitted Rust ──compiled──► .a ┘

real Postgres C    ──ast-dump──► ast.json ─┐
                                            ├─ STRUCTURAL diff
Lean-emitted C     ──ast-dump──► ast.json ─┘
```

The Rust emit proves "Lean spec is behaviorally equivalent to PG C on
tested inputs." This C emit proves "Lean spec captures the C
algorithm itself, not just its outputs on a sample."

Together: Lean ≡ C structurally AND Lean ≡ Rust behaviorally ⇒
Rust is the same algorithm as the C, certified at both levels.

## Style requirements

The emit must use:
- `PG_FUNCTION_ARGS` macro (expands to `FunctionCallInfo fcinfo`)
- `PG_GETARG_<TYPE>(N)` macros for each arg
- `PG_RETURN_BOOL(...)` for the result
- Postgres' typedef'd type names (`int32`, `Cash`, `Oid`, etc.)
- Function naming + tabs matching Postgres style (so even a byte-level
  diff would only differ at locals, which clang normalizes anyway)

Differences from the Postgres vendored source that the AST diff will
need to tolerate:
- File/path/line info in `loc` and `range` fields (clang normalizes,
  but we strip in the diff tool)
- Internal `id` fields (per-parse unique; meaningless)
- Function-pointer ABI annotations (no equivalent in C; only Rust uses
  `#[no_mangle] extern "C"`)
-/

namespace Pg.Ir.Emit.IntCmpC

open Pg.Ir.Emit (CmpStyle Family families ops effectiveCmpStyle)

/-- The C type that maps to this family's decoded operand. Used in
the C source's local-variable declarations. -/
def familyCType (fam : Family) : String :=
  match fam.namePrefix with
  -- int families
  | "int2"       => "int16"
  | "int4"       => "int32"
  | "int8"       => "int64"
  -- date/time families
  | "date_"      => "DateADT"
  | "timestamp_" => "Timestamp"
  | "cash_"      => "Cash"
  | "pg_lsn_"    => "XLogRecPtr"
  -- OIDs and bool
  | "oid"        => "Oid"
  | "bool"       => "bool"
  -- float
  | "float4"     => "float4"
  | "float8"     => "float8"
  | "char"       => "char"     -- Postgres "char" type = signed char
  -- Cross-type comparators: cType is the FIRST arg's type. Use
  -- `familyCType2` for the second.
  | "int24"      => "int16"
  | "int42"      => "int32"
  | "int28"      => "int16"
  | "int82"      => "int64"
  | "int48"      => "int32"
  | "int84"      => "int64"
  | _            => "/* TODO: unhandled family */ int64"

/-- C type for arg[1] in cross-type comparators. For same-type families
this is identical to `familyCType`; for int24/int28/etc. it's the
DIFFERENT operand width. -/
def familyCType2 (fam : Family) : String :=
  match fam.namePrefix with
  | "int24" => "int32"   -- (int2, int4)
  | "int42" => "int16"   -- (int4, int2)
  | "int28" => "int64"   -- (int2, int8)
  | "int82" => "int16"   -- (int8, int2)
  | "int48" => "int64"   -- (int4, int8)
  | "int84" => "int32"   -- (int8, int4)
  | _       => familyCType fam

/-- The PG_GETARG_* macro for arg[0]. -/
def familyGetArg (fam : Family) : String :=
  match fam.namePrefix with
  | "int2"       => "PG_GETARG_INT16"
  | "int4"       => "PG_GETARG_INT32"
  | "int8"       => "PG_GETARG_INT64"
  | "date_"      => "PG_GETARG_DATEADT"
  | "timestamp_" => "PG_GETARG_TIMESTAMP"
  | "cash_"      => "PG_GETARG_CASH"
  | "pg_lsn_"    => "PG_GETARG_LSN"
  | "oid"        => "PG_GETARG_OID"
  | "bool"       => "PG_GETARG_BOOL"
  | "float4"     => "PG_GETARG_FLOAT4"
  | "float8"     => "PG_GETARG_FLOAT8"
  | "char"       => "PG_GETARG_CHAR"
  | "int24"      => "PG_GETARG_INT16"
  | "int42"      => "PG_GETARG_INT32"
  | "int28"      => "PG_GETARG_INT16"
  | "int82"      => "PG_GETARG_INT64"
  | "int48"      => "PG_GETARG_INT32"
  | "int84"      => "PG_GETARG_INT64"
  | _            => "/* TODO */ PG_GETARG_INT64"

/-- The PG_GETARG_* macro for arg[1]. -/
def familyGetArg2 (fam : Family) : String :=
  match fam.namePrefix with
  | "int24" => "PG_GETARG_INT32"
  | "int42" => "PG_GETARG_INT16"
  | "int28" => "PG_GETARG_INT64"
  | "int82" => "PG_GETARG_INT16"
  | "int48" => "PG_GETARG_INT64"
  | "int84" => "PG_GETARG_INT32"
  | _       => familyGetArg fam

/-- Render the comparison body for the C emit. -/
def renderCmpExpr (style : CmpStyle) (suffix opStr : String) : String :=
  match style with
  | .native                   => "arg1 " ++ opStr ++ " arg2"
  | .nativeCastBoth cCast _   =>
      "(" ++ cCast ++ ") arg1 " ++ opStr ++ " (" ++ cCast ++ ") arg2"
  | .pgFloat32                => "float4_" ++ suffix ++ "(arg1, arg2)"
  | .pgFloat64                => "float8_" ++ suffix ++ "(arg1, arg2)"
  | .pgCmpInternal name       => name ++ "(arg1, arg2) " ++ opStr ++ " 0"

/-- Render one C function. Naming + tabs match Postgres style so a
byte-level diff is dominated by clang-normalized-away cosmetics. -/
def renderCFn (fam : Family) (suffix opStr : String) : String :=
  let fnName  := fam.namePrefix ++ suffix
  let cType1  := familyCType fam
  let cType2  := familyCType2 fam
  let getArg1 := familyGetArg fam
  let getArg2 := familyGetArg2 fam
  let style   := effectiveCmpStyle fam suffix
  let cmp     := renderCmpExpr style suffix opStr
  "Datum\n" ++
  fnName ++ "(PG_FUNCTION_ARGS)\n" ++
  "{\n" ++
  "\t" ++ cType1 ++ "\t\targ1 = " ++ getArg1 ++ "(0);\n" ++
  "\t" ++ cType2 ++ "\t\targ2 = " ++ getArg2 ++ "(1);\n" ++
  "\n" ++
  "\tPG_RETURN_BOOL(" ++ cmp ++ ");\n" ++
  "}\n"

/-- Render all 6 ops for one family. -/
def renderCFamily (fam : Family) : String :=
  Id.run do
    let mut s := "\n/* ── " ++ fam.namePrefix ++ " family ── */\n"
    for (suffix, opStr) in ops do
      s := s ++ "\n" ++ renderCFn fam suffix opStr
    return s

/-- The full C source for all comparison families. -/
def cSource : String :=
  let header :=
    "/*\n" ++
    " * Generated by `rules_postgres/lean/Pg/Ir/Emit/IntCmpC.lean`.\n" ++
    " * DO NOT EDIT BY HAND. Regenerate via\n" ++
    " * `rules_postgres/tools/regen/regen-int-cmp.sh`.\n" ++
    " *\n" ++
    " * The point of this file is NOT to compile + link — Postgres'\n" ++
    " * own int.c / date.c / etc. handle that. Instead, this file\n" ++
    " * exists to be `clang -ast-dump=json`-ed alongside the vendored\n" ++
    " * Postgres source, and the two AST JSONs are structurally diffed.\n" ++
    " * A passing diff means the Lean spec captures the same algorithm\n" ++
    " * Postgres wrote, not just its observable behavior.\n" ++
    " *\n" ++
    " * Style: mirrors Postgres' int.c naming + tabs verbatim so the\n" ++
    " * AST trees should match modulo clang-normalized cosmetics.\n" ++
    " */\n" ++
    "\n" ++
    "#include \"postgres.h\"\n" ++
    "#include \"fmgr.h\"\n" ++
    "#include \"utils/date.h\"\n" ++
    "#include \"datatype/timestamp.h\"\n" ++
    "#include \"utils/timestamp.h\"\n" ++   /- PG_GETARG_TIMESTAMP + DatumGetTimestamp -/
    "#include \"utils/cash.h\"\n" ++
    "#include \"access/xlogdefs.h\"\n" ++
    "#include \"utils/pg_lsn.h\"\n" ++      /- PG_GETARG_LSN + DatumGetLSN -/
    "#include \"utils/float.h\"\n"
  Id.run do
    let mut s := header
    for fam in families do
      s := s ++ renderCFamily fam
    return s

end Pg.Ir.Emit.IntCmpC

/-- `main` entrypoint: print the C source to stdout. -/
def main : IO Unit := IO.print Pg.Ir.Emit.IntCmpC.cSource
