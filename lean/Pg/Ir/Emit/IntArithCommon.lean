/-!
# Pg.Ir.Emit.IntArithCommon — shared family table for the integer
arithmetic emit modules (`IntArith` for Rust, `IntArithC` for the
AST-grounding C).

Same factoring pattern as `IntHashCommon.lean`: keeps each emit module
a single compilation unit with its own `main`.

v1 scope: 21 functions = {pl, mi, mul} × {int2, int4, int8, int24,
int42, int48, int84, int28, int82} − one redundancy. Division is NOT
in v1 — it has different overflow semantics (INT_MIN / -1 special-case)
and needs a `pg_ereport_division_by_zero` branch; lifted in a separate
cluster.

The Postgres sources follow a single shape per body:

```
T1 arg1 = PG_GETARG_T1(0);
T2 arg2 = PG_GETARG_T2(1);
RT  result;
if (unlikely(pg_<op>_s<W>_overflow(<cast>arg1, <cast>arg2, &result)))
    ereport(ERROR,
            (errcode(ERRCODE_NUMERIC_VALUE_OUT_OF_RANGE),
             errmsg("<width-word> out of range")));
PG_RETURN_T(result);
```

The single difference across cross-type families is which operands
get the explicit `(int32)` / `(int64)` widening cast and which
overflow primitive width is used (always the result type's width).
-/

namespace Pg.Ir.Emit.IntArith

/-- The three arithmetic ops. -/
inductive ArithOp where
  | pl
  | mi
  | mul
  deriving DecidableEq, Repr

/-- Postgres suffix on the function name. -/
def ArithOp.suffix : ArithOp → String
  | .pl  => "pl"
  | .mi  => "mi"
  | .mul => "mul"

/-- Lowercase op name used to pick the `pg_<add|sub|mul>_s<W>_overflow`
primitive. -/
def ArithOp.primitive : ArithOp → String
  | .pl  => "add"
  | .mi  => "sub"
  | .mul => "mul"

structure ArithFamily where
  /-- Postgres function-name prefix (e.g., `int4` produces int4pl /
  int4mi / int4mul). -/
  namePrefix  : String
  /-- The `pg_fcinfo::decode_*` helper name for arg[0] on the Rust side. -/
  decoder1    : String
  /-- The `pg_fcinfo::decode_*` helper name for arg[1] on the Rust side.
  Empty = same as `decoder1` (same-type families). -/
  decoder2    : String := ""
  /-- The Rust type the body computes in (also the overflow primitive's
  width and the result type). E.g., `i32` for int4*, `i64` for any
  cross-type family touching int8. -/
  rustResultTy : String
  /-- The `pg_fcinfo::encode_*` helper name for the return value. -/
  rustEncoder  : String
  /-- Overflow primitive width suffix on the Rust side: `s16`, `s32`,
  or `s64`. Combined with the op: `pg_add_s32_overflow`, etc. -/
  overflowWidth : String
  /-- Non-empty iff arg1 needs an explicit `as <rustResultTy>` widening
  cast on the Rust side before the overflow primitive. Used as a flag;
  the cast target is always `rustResultTy`. -/
  rustCast1   : String := ""
  /-- Non-empty iff arg2 needs an explicit `as <rustResultTy>` cast. -/
  rustCast2   : String := ""
  /-- The `PG_GETARG_*` macro name for arg[0] on the C emit side. -/
  pgGetArg1   : String
  /-- The `PG_GETARG_*` macro name for arg[1]. -/
  pgGetArg2   : String
  /-- The `PG_RETURN_*` macro name. -/
  pgReturn    : String
  /-- C type name for arg1 (e.g., `int16`, `int32`, `int64`). -/
  cWidth1     : String
  /-- C type name for arg2. -/
  cWidth2     : String
  /-- C result type / overflow primitive width: `int16`, `int32`, `int64`. -/
  cResultTy   : String
  /-- C-side cast string (with trailing space) applied to arg1 before
  the overflow primitive — e.g., `"(int32) "` for int24*. Empty when
  arg1 already matches `cResultTy`. -/
  cCast1      : String := ""
  /-- C-side cast string (with trailing space) applied to arg2. -/
  cCast2      : String := ""
  /-- Error-message width word for `errmsg("<word> out of range")`.
  Postgres uses "smallint" for int16 results, "integer" for int32,
  "bigint" for int64 — matches `cResultTy`. -/
  errWord     : String

/-- Width string for the C `pg_<op>_s<W>_overflow` primitive — same as
the Rust width-suffix on this family. -/
def ArithFamily.cOverflowWidth (fam : ArithFamily) : String := fam.overflowWidth

/-- True iff the family is cross-type (operands differ in width). -/
def ArithFamily.isCrossType (fam : ArithFamily) : Bool := fam.decoder2 != ""

/-- The 9 type families (3 same-type + 6 cross-type). Each renders 3
functions (pl/mi/mul) for 21 total. -/
def families : List ArithFamily := [
  -- Same-type
  { namePrefix := "int2", decoder1 := "decode_i16",
    rustResultTy := "i16", rustEncoder := "encode_i16",
    overflowWidth := "s16",
    pgGetArg1 := "PG_GETARG_INT16", pgGetArg2 := "PG_GETARG_INT16",
    pgReturn  := "PG_RETURN_INT16",
    cWidth1   := "int16", cWidth2 := "int16", cResultTy := "int16",
    errWord   := "smallint" },
  { namePrefix := "int4", decoder1 := "decode_i32",
    rustResultTy := "i32", rustEncoder := "encode_i32",
    overflowWidth := "s32",
    pgGetArg1 := "PG_GETARG_INT32", pgGetArg2 := "PG_GETARG_INT32",
    pgReturn  := "PG_RETURN_INT32",
    cWidth1   := "int32", cWidth2 := "int32", cResultTy := "int32",
    errWord   := "integer" },
  { namePrefix := "int8", decoder1 := "decode_i64",
    rustResultTy := "i64", rustEncoder := "encode_i64",
    overflowWidth := "s64",
    pgGetArg1 := "PG_GETARG_INT64", pgGetArg2 := "PG_GETARG_INT64",
    pgReturn  := "PG_RETURN_INT64",
    cWidth1   := "int64", cWidth2 := "int64", cResultTy := "int64",
    errWord   := "bigint" },
  -- int2 × int4 → int4
  { namePrefix := "int24", decoder1 := "decode_i16", decoder2 := "decode_i32",
    rustResultTy := "i32", rustEncoder := "encode_i32",
    overflowWidth := "s32",
    rustCast1 := "yes",
    pgGetArg1 := "PG_GETARG_INT16", pgGetArg2 := "PG_GETARG_INT32",
    pgReturn  := "PG_RETURN_INT32",
    cWidth1   := "int16", cWidth2 := "int32", cResultTy := "int32",
    cCast1    := "(int32) ",
    errWord   := "integer" },
  { namePrefix := "int42", decoder1 := "decode_i32", decoder2 := "decode_i16",
    rustResultTy := "i32", rustEncoder := "encode_i32",
    overflowWidth := "s32",
    rustCast2 := "yes",
    pgGetArg1 := "PG_GETARG_INT32", pgGetArg2 := "PG_GETARG_INT16",
    pgReturn  := "PG_RETURN_INT32",
    cWidth1   := "int32", cWidth2 := "int16", cResultTy := "int32",
    cCast2    := "(int32) ",
    errWord   := "integer" },
  -- int4 × int8 → int8
  { namePrefix := "int48", decoder1 := "decode_i32", decoder2 := "decode_i64",
    rustResultTy := "i64", rustEncoder := "encode_i64",
    overflowWidth := "s64",
    rustCast1 := "yes",
    pgGetArg1 := "PG_GETARG_INT32", pgGetArg2 := "PG_GETARG_INT64",
    pgReturn  := "PG_RETURN_INT64",
    cWidth1   := "int32", cWidth2 := "int64", cResultTy := "int64",
    cCast1    := "(int64) ",
    errWord   := "bigint" },
  { namePrefix := "int84", decoder1 := "decode_i64", decoder2 := "decode_i32",
    rustResultTy := "i64", rustEncoder := "encode_i64",
    overflowWidth := "s64",
    rustCast2 := "yes",
    pgGetArg1 := "PG_GETARG_INT64", pgGetArg2 := "PG_GETARG_INT32",
    pgReturn  := "PG_RETURN_INT64",
    cWidth1   := "int64", cWidth2 := "int32", cResultTy := "int64",
    cCast2    := "(int64) ",
    errWord   := "bigint" },
  -- int2 × int8 → int8
  { namePrefix := "int28", decoder1 := "decode_i16", decoder2 := "decode_i64",
    rustResultTy := "i64", rustEncoder := "encode_i64",
    overflowWidth := "s64",
    rustCast1 := "yes",
    pgGetArg1 := "PG_GETARG_INT16", pgGetArg2 := "PG_GETARG_INT64",
    pgReturn  := "PG_RETURN_INT64",
    cWidth1   := "int16", cWidth2 := "int64", cResultTy := "int64",
    cCast1    := "(int64) ",
    errWord   := "bigint" },
  { namePrefix := "int82", decoder1 := "decode_i64", decoder2 := "decode_i16",
    rustResultTy := "i64", rustEncoder := "encode_i64",
    overflowWidth := "s64",
    rustCast2 := "yes",
    pgGetArg1 := "PG_GETARG_INT64", pgGetArg2 := "PG_GETARG_INT16",
    pgReturn  := "PG_RETURN_INT64",
    cWidth1   := "int64", cWidth2 := "int16", cResultTy := "int64",
    cCast2    := "(int64) ",
    errWord   := "bigint" },
]

/-- All three ops, in source order. -/
def ops : List ArithOp := [.pl, .mi, .mul]

end Pg.Ir.Emit.IntArith
