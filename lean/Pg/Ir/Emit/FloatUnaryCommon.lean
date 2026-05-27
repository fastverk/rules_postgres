/-!
# Pg.Ir.Emit.FloatUnaryCommon — shared family table for the float
unary V1 fmgr cluster (float{4,8}{um,abs}).

Sister to IntUnaryCommon. Simpler than integers: no ereport needed,
no overflow check (IEEE-754 handles NaN and infinity cleanly).

Two body shapes:
  - `absFlat` — float4abs / float8abs — `PG_RETURN_<W>(fabsf/fabs(arg))`
  - `umWithResult` — float4um / float8um — declares `result = -arg` first

The shape split matches real Postgres' source structure exactly so
the AST grounding gate stays clean.
-/

namespace Pg.Ir.Emit.FloatUnary

inductive UnaryBody where
  | absFlat      -- float4abs, float8abs
  | umWithResult -- float4um, float8um
  deriving DecidableEq, Repr

structure UnaryFamily where
  /-- Postgres fmgr fn name (`float4um`, `float4abs`, …). -/
  fnName     : String
  /-- pg_fcinfo decoder for the single arg. -/
  decoder    : String
  /-- pg_fcinfo encoder for the result. -/
  encoder    : String
  /-- Rust type name (`f32`/`f64`). -/
  rustTy     : String
  /-- C `PG_GETARG_*` macro name. -/
  pgGetArg   : String
  /-- C `PG_RETURN_*` macro name. -/
  pgReturn   : String
  /-- C type name for local declarations (`float4`/`float8`). -/
  cWidth     : String
  /-- C math function for abs: `fabsf` / `fabs`. -/
  cAbsFunc   : String
  /-- The body shape. -/
  body       : UnaryBody

def families : List UnaryFamily := [
  -- abs: both same shape
  { fnName := "float4abs", decoder := "decode_f32", encoder := "encode_f32",
    rustTy := "f32", pgGetArg := "PG_GETARG_FLOAT4", pgReturn := "PG_RETURN_FLOAT4",
    cWidth := "float4", cAbsFunc := "fabsf", body := .absFlat },
  { fnName := "float8abs", decoder := "decode_f64", encoder := "encode_f64",
    rustTy := "f64", pgGetArg := "PG_GETARG_FLOAT8", pgReturn := "PG_RETURN_FLOAT8",
    cWidth := "float8", cAbsFunc := "fabs", body := .absFlat },
  -- um: both same shape
  { fnName := "float4um", decoder := "decode_f32", encoder := "encode_f32",
    rustTy := "f32", pgGetArg := "PG_GETARG_FLOAT4", pgReturn := "PG_RETURN_FLOAT4",
    cWidth := "float4", cAbsFunc := "fabsf", body := .umWithResult },
  { fnName := "float8um", decoder := "decode_f64", encoder := "encode_f64",
    rustTy := "f64", pgGetArg := "PG_GETARG_FLOAT8", pgReturn := "PG_RETURN_FLOAT8",
    cWidth := "float8", cAbsFunc := "fabs", body := .umWithResult },
]

end Pg.Ir.Emit.FloatUnary
