/-!
# Pg.Ir.Emit.IntUnaryCommon — shared family table for the integer
unary V1 fmgr cluster (int{2,4,8}{um,abs}).

Sister to IntArithCommon. Same ereport mirror; simpler body shapes
(single arg, no overflow primitive needed — the only check is
`arg == <ty>::MIN` because negating it overflows).

Three body shapes:
  - `umFlat` — int2um / int4um — `PG_RETURN_<W>(-arg)` directly
  - `umWithResult` — int8um — declares `result = -arg` first
  - `abs` — int{2,4,8}abs — `result = (arg < 0) ? -arg : arg`

The shape split matches real Postgres' source structure exactly so
the AST grounding gate stays clean.
-/

namespace Pg.Ir.Emit.IntUnary

inductive UnaryBody where
  | umFlat       -- int2um, int4um
  | umWithResult -- int8um
  | abs          -- int2abs, int4abs, int8abs
  deriving DecidableEq, Repr

structure UnaryFamily where
  /-- Postgres fmgr fn name (`int4um`, `int4abs`, …). -/
  fnName     : String
  /-- pg_fcinfo decoder for the single arg. -/
  decoder    : String
  /-- pg_fcinfo encoder for the result. -/
  encoder    : String
  /-- Rust type name (`i16`/`i32`/`i64`). -/
  rustTy     : String
  /-- C `PG_GETARG_*` macro name. -/
  pgGetArg   : String
  /-- C `PG_RETURN_*` macro name. -/
  pgReturn   : String
  /-- C type name for local declarations (`int16`/`int32`/`int64`). -/
  cWidth     : String
  /-- Postgres' `PG_INT{16,32,64}_MIN` macro name. -/
  cIntMin    : String
  /-- Rust constant for the type's MIN value. -/
  rustIntMin : String
  /-- Postgres' errmsg text — one of "smallint out of range",
  "integer out of range", "bigint out of range". -/
  errmsg     : String
  /-- The body shape. -/
  body       : UnaryBody

def families : List UnaryFamily := [
  -- um: int2 (umFlat), int4 (umFlat), int8 (umWithResult)
  { fnName := "int2um", decoder := "decode_i16", encoder := "encode_i16",
    rustTy := "i16", pgGetArg := "PG_GETARG_INT16", pgReturn := "PG_RETURN_INT16",
    cWidth := "int16", cIntMin := "PG_INT16_MIN", rustIntMin := "i16::MIN",
    errmsg := "smallint out of range", body := .umFlat },
  { fnName := "int4um", decoder := "decode_i32", encoder := "encode_i32",
    rustTy := "i32", pgGetArg := "PG_GETARG_INT32", pgReturn := "PG_RETURN_INT32",
    cWidth := "int32", cIntMin := "PG_INT32_MIN", rustIntMin := "i32::MIN",
    errmsg := "integer out of range", body := .umFlat },
  { fnName := "int8um", decoder := "decode_i64", encoder := "encode_i64",
    rustTy := "i64", pgGetArg := "PG_GETARG_INT64", pgReturn := "PG_RETURN_INT64",
    cWidth := "int64", cIntMin := "PG_INT64_MIN", rustIntMin := "i64::MIN",
    errmsg := "bigint out of range", body := .umWithResult },
  -- abs: all three same shape
  { fnName := "int2abs", decoder := "decode_i16", encoder := "encode_i16",
    rustTy := "i16", pgGetArg := "PG_GETARG_INT16", pgReturn := "PG_RETURN_INT16",
    cWidth := "int16", cIntMin := "PG_INT16_MIN", rustIntMin := "i16::MIN",
    errmsg := "smallint out of range", body := .abs },
  { fnName := "int4abs", decoder := "decode_i32", encoder := "encode_i32",
    rustTy := "i32", pgGetArg := "PG_GETARG_INT32", pgReturn := "PG_RETURN_INT32",
    cWidth := "int32", cIntMin := "PG_INT32_MIN", rustIntMin := "i32::MIN",
    errmsg := "integer out of range", body := .abs },
  { fnName := "int8abs", decoder := "decode_i64", encoder := "encode_i64",
    rustTy := "i64", pgGetArg := "PG_GETARG_INT64", pgReturn := "PG_RETURN_INT64",
    cWidth := "int64", cIntMin := "PG_INT64_MIN", rustIntMin := "i64::MIN",
    errmsg := "bigint out of range", body := .abs },
]

end Pg.Ir.Emit.IntUnary
