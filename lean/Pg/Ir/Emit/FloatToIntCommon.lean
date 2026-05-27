/-!
# Pg.Ir.Emit.FloatToIntCommon — shared family table for the float-to-int
cast V1 fmgr cluster (ftoi2, ftoi4, dtoi2, dtoi4).

Each function performs a floating-point-to-integer narrowing cast with
overflow and NaN checks:
  1. Apply `rint()` to remove fractional part (so just-out-of-range values
     that would round in-range don't fail).
  2. Check for NaN via `isnan()`.
  3. Range-check via `FLOAT{4,8}_FITS_IN_INT{16,32}(num)` macros.
  4. If both checks pass, truncate (cast) and return.

This matches real Postgres' source structure exactly so AST grounding stays
clean.
-/

namespace Pg.Ir.Emit.FloatToInt

inductive CastBody where
  | toInt16  -- ftoi2, dtoi2: narrowing cast to int16
  | toInt32  -- ftoi4, dtoi4: narrowing cast to int32
  deriving DecidableEq, Repr

structure CastFamily where
  /-- Postgres fmgr fn name (`ftoi2`, `dtoi4`, …). -/
  fnName     : String
  /-- pg_fcinfo decoder for arg[0]. -/
  decoder    : String
  /-- pg_fcinfo encoder for the result. -/
  encoder    : String
  /-- Rust type name for the float source (`f32`/`f64`). -/
  rustFloatTy : String
  /-- Rust type name for the int result (`i16`/`i32`). -/
  rustIntTy   : String
  /-- C `PG_GETARG_*` macro name for the float arg. -/
  pgGetArg   : String
  /-- C `PG_RETURN_*` macro name for the int result. -/
  pgReturn   : String
  /-- C type name for the float arg (`float4`/`float8`). -/
  cFloatWidth : String
  /-- C type name for the int result (`int16`/`int32`). -/
  cIntWidth   : String
  /-- C type suffix for isnan/FLOAT*_FITS_IN_INT* checks: `f` for float4, empty for float8. -/
  cCheckSuffix : String
  /-- Error-message width word for `errmsg("<word> out of range")`.
  Postgres uses "smallint" for int16, "integer" for int32. -/
  errWord    : String
  /-- The target int width. -/
  body       : CastBody

def families : List CastFamily := [
  -- float4 (float) to int16
  { fnName := "ftoi2", decoder := "decode_f32", encoder := "encode_i16",
    rustFloatTy := "f32", rustIntTy := "i16",
    pgGetArg := "PG_GETARG_FLOAT4", pgReturn := "PG_RETURN_INT16",
    cFloatWidth := "float4", cIntWidth := "int16", cCheckSuffix := "f",
    errWord := "smallint", body := .toInt16 },

  -- float4 (float) to int32
  { fnName := "ftoi4", decoder := "decode_f32", encoder := "encode_i32",
    rustFloatTy := "f32", rustIntTy := "i32",
    pgGetArg := "PG_GETARG_FLOAT4", pgReturn := "PG_RETURN_INT32",
    cFloatWidth := "float4", cIntWidth := "int32", cCheckSuffix := "f",
    errWord := "integer", body := .toInt32 },

  -- float8 (double) to int16
  { fnName := "dtoi2", decoder := "decode_f64", encoder := "encode_i16",
    rustFloatTy := "f64", rustIntTy := "i16",
    pgGetArg := "PG_GETARG_FLOAT8", pgReturn := "PG_RETURN_INT16",
    cFloatWidth := "float8", cIntWidth := "int16", cCheckSuffix := "",
    errWord := "smallint", body := .toInt16 },

  -- float8 (double) to int32
  { fnName := "dtoi4", decoder := "decode_f64", encoder := "encode_i32",
    rustFloatTy := "f64", rustIntTy := "i32",
    pgGetArg := "PG_GETARG_FLOAT8", pgReturn := "PG_RETURN_INT32",
    cFloatWidth := "float8", cIntWidth := "int32", cCheckSuffix := "",
    errWord := "integer", body := .toInt32 },
]

end Pg.Ir.Emit.FloatToInt
