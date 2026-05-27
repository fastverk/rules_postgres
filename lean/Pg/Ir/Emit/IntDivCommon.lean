/-!
# Pg.Ir.Emit.IntDivCommon — shared family table for the integer
division emit modules (`IntDiv` for Rust, `IntDivC` for the
AST-grounding C).

Same factoring pattern as `IntArithCommon.lean`: keeps each emit module
a single compilation unit with its own `main`.

v1 scope: 3 functions = {int2div, int4div, int8div}. Division has
distinct overflow semantics compared to arithmetic (INT_MIN / -1
special-case) and requires TWO different errcodes:
  - ERRCODE_DIVISION_BY_ZERO: when arg2 == 0
  - ERRCODE_NUMERIC_VALUE_OUT_OF_RANGE: when arg1 == INT_MIN and arg2 == -1

Note: int2div returns int16 (not widened), per Postgres int.c source.

The Postgres sources follow this shape per body:

```
T1  arg1 = PG_GETARG_T1(0);
T2  arg2 = PG_GETARG_T2(1);
RT  result;

if (unlikely(arg2 == 0))
    ereport(ERROR,
            (errcode(ERRCODE_DIVISION_BY_ZERO),
             errmsg("division by zero")));

if (arg2 == -1)
{
    if (unlikely(arg1 == PG_INT<W>_MIN))
        ereport(ERROR,
                (errcode(ERRCODE_NUMERIC_VALUE_OUT_OF_RANGE),
                 errmsg("<word> out of range")));
    result = -arg1;
    PG_RETURN_T(result);
}

result = arg1 / arg2;
PG_RETURN_T(result);
```

The key difference across type families is the width constants (INT16_MIN,
INT32_MIN, INT64_MIN) and the ereport message word ("smallint", "integer",
"bigint").
-/

namespace Pg.Ir.Emit.IntDiv

/-- The three integer division functions. -/
inductive DivFamily where
  | int2div
  | int4div
  | int8div
  deriving DecidableEq, Repr

/-- Postgres function name. -/
def DivFamily.fnName : DivFamily → String
  | .int2div => "int2div"
  | .int4div => "int4div"
  | .int8div => "int8div"

structure DivFamilySpec where
  /-- Postgres function-name (e.g., `int4div`). -/
  fnName      : String
  /-- The `pg_fcinfo::decode_*` helper name for arg[0] on the Rust side. -/
  decoder     : String
  /-- The Rust type the body computes in (result type). -/
  rustResultTy : String
  /-- The `pg_fcinfo::encode_*` helper name for the return value. -/
  rustEncoder  : String
  /-- The Rust MIN constant for overflow detection: `i16::MIN`, `i32::MIN`, or `i64::MIN`. -/
  rustMin     : String
  /-- The `PG_GETARG_*` macro name on the C emit side. -/
  pgGetArg    : String
  /-- The `PG_RETURN_*` macro name. -/
  pgReturn    : String
  /-- C type name (e.g., `int16`, `int32`, `int64`). -/
  cWidth      : String
  /-- C MIN constant for overflow detection: `PG_INT16_MIN`, etc. -/
  cMin        : String
  /-- Error-message width word for `errmsg("<word> out of range")`. -/
  errWord     : String

/-- The 3 division families. -/
def families : List DivFamilySpec := [
  { fnName := "int2div", decoder := "decode_i16",
    rustResultTy := "i16", rustEncoder := "encode_i16",
    rustMin := "i16::MIN",
    pgGetArg := "PG_GETARG_INT16", pgReturn := "PG_RETURN_INT16",
    cWidth := "int16", cMin := "PG_INT16_MIN",
    errWord := "smallint" },
  { fnName := "int4div", decoder := "decode_i32",
    rustResultTy := "i32", rustEncoder := "encode_i32",
    rustMin := "i32::MIN",
    pgGetArg := "PG_GETARG_INT32", pgReturn := "PG_RETURN_INT32",
    cWidth := "int32", cMin := "PG_INT32_MIN",
    errWord := "integer" },
  { fnName := "int8div", decoder := "decode_i64",
    rustResultTy := "i64", rustEncoder := "encode_i64",
    rustMin := "i64::MIN",
    pgGetArg := "PG_GETARG_INT64", pgReturn := "PG_RETURN_INT64",
    cWidth := "int64", cMin := "PG_INT64_MIN",
    errWord := "bigint" },
]

end Pg.Ir.Emit.IntDiv
