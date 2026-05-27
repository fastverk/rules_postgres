/-!
# Pg.Ir.Emit.FloatArithCommon — shared family table for the float
arithmetic V1 fmgr cluster (float{4,8}{pl,mi,mul,div}).

Sister to IntArithCommon, but simpler: no overflow primitives (IEEE-754
handles NaN and infinity). Instead, uses post-computation error checks via
helper functions from Postgres' float.h:
  - float_overflow_error() — for result isinf() when neither input was
  - float_underflow_error() — for result == 0.0 when neither input was
  - float_zero_divide_error() — for division by zero with non-NaN dividend

Four body shapes:
  - `plWithCheck` — float{4,8}pl — add, check overflow only
  - `miWithCheck` — float{4,8}mi — subtract, check overflow only
  - `mulWithCheck` — float{4,8}mul — multiply, check overflow AND underflow
  - `divWithCheck` — float{4,8}div — divide by zero check first, then compute

Matches real Postgres' source structure exactly so the AST grounding gate
stays clean.
-/

namespace Pg.Ir.Emit.FloatArith

inductive ArithBody where
  | plWithCheck   -- float4pl, float8pl: check overflow
  | miWithCheck   -- float4mi, float8mi: check overflow
  | mulWithCheck  -- float4mul, float8mul: check overflow AND underflow
  | divWithCheck  -- float4div, float8div: divide-by-zero check, then compute
  deriving DecidableEq, Repr

structure ArithFamily where
  /-- Postgres fmgr fn name (`float4pl`, `float8div`, …). -/
  fnName     : String
  /-- pg_fcinfo decoder for arg[0]. -/
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
  /-- C type suffix for is_infinite/isnan checks: `f` for float4, empty for float8. -/
  cCheckSuffix : String
  /-- The body shape. -/
  body       : ArithBody

def families : List ArithFamily := [
  -- Addition: overflow check only
  { fnName := "float4pl", decoder := "decode_f32", encoder := "encode_f32",
    rustTy := "f32", pgGetArg := "PG_GETARG_FLOAT4", pgReturn := "PG_RETURN_FLOAT4",
    cWidth := "float4", cCheckSuffix := "f", body := .plWithCheck },
  { fnName := "float8pl", decoder := "decode_f64", encoder := "encode_f64",
    rustTy := "f64", pgGetArg := "PG_GETARG_FLOAT8", pgReturn := "PG_RETURN_FLOAT8",
    cWidth := "float8", cCheckSuffix := "", body := .plWithCheck },

  -- Subtraction: overflow check only
  { fnName := "float4mi", decoder := "decode_f32", encoder := "encode_f32",
    rustTy := "f32", pgGetArg := "PG_GETARG_FLOAT4", pgReturn := "PG_RETURN_FLOAT4",
    cWidth := "float4", cCheckSuffix := "f", body := .miWithCheck },
  { fnName := "float8mi", decoder := "decode_f64", encoder := "encode_f64",
    rustTy := "f64", pgGetArg := "PG_GETARG_FLOAT8", pgReturn := "PG_RETURN_FLOAT8",
    cWidth := "float8", cCheckSuffix := "", body := .miWithCheck },

  -- Multiplication: overflow AND underflow checks
  { fnName := "float4mul", decoder := "decode_f32", encoder := "encode_f32",
    rustTy := "f32", pgGetArg := "PG_GETARG_FLOAT4", pgReturn := "PG_RETURN_FLOAT4",
    cWidth := "float4", cCheckSuffix := "f", body := .mulWithCheck },
  { fnName := "float8mul", decoder := "decode_f64", encoder := "encode_f64",
    rustTy := "f64", pgGetArg := "PG_GETARG_FLOAT8", pgReturn := "PG_RETURN_FLOAT8",
    cWidth := "float8", cCheckSuffix := "", body := .mulWithCheck },

  -- Division: divide-by-zero check, overflow check, underflow check
  { fnName := "float4div", decoder := "decode_f32", encoder := "encode_f32",
    rustTy := "f32", pgGetArg := "PG_GETARG_FLOAT4", pgReturn := "PG_RETURN_FLOAT4",
    cWidth := "float4", cCheckSuffix := "f", body := .divWithCheck },
  { fnName := "float8div", decoder := "decode_f64", encoder := "encode_f64",
    rustTy := "f64", pgGetArg := "PG_GETARG_FLOAT8", pgReturn := "PG_RETURN_FLOAT8",
    cWidth := "float8", cCheckSuffix := "", body := .divWithCheck },
]

end Pg.Ir.Emit.FloatArith
