/-!
# Pg.Ir.Emit.IntCastsCommon — shared family table for the integer
width-cast emit modules (`IntCasts` for Rust, `IntCastsC` for the
AST-grounding C).

v1 scope: 6 functions = {i2toi4, i4toi2, i2toi8, i8toi2, i4toi8, i8toi4}.

Widening casts (i2→i4, i2→i8, i4→i8) are pure — no overflow possible.
Narrowing casts (i4→i2, i8→i2, i8→i4) ereport on overflow.

Body shapes:
- Widening: `PG_RETURN_<W>(<cast> arg)` — direct cast
- Narrowing: `arg` checked against `[MIN, MAX]` bounds; ereport on overflow

Real Postgres sources (int.c / int8.c):
  - i2toi4: int2→int4 widening
  - i4toi2: int4→int2 narrowing (check SHRT_MIN/SHRT_MAX)
  - i2toi8 (int28): int2→int8 widening
  - i8toi2 (int82): int8→int2 narrowing (check PG_INT16_MIN/MAX)
  - i4toi8 (int48): int4→int8 widening
  - i8toi4 (int84): int8→int4 narrowing (check PG_INT32_MIN/MAX)
-/

namespace Pg.Ir.Emit.IntCasts

/-- The 6 cast functions. -/
inductive CastOp where
  | i2toi4 -- int2 → int4 (widening)
  | i4toi2 -- int4 → int2 (narrowing)
  | i2toi8 -- int2 → int8 (widening)
  | i8toi2 -- int8 → int2 (narrowing)
  | i4toi8 -- int4 → int8 (widening)
  | i8toi4 -- int8 → int4 (narrowing)
  deriving DecidableEq, Repr

def CastOp.fnName : CastOp → String
  | .i2toi4 => "i2toi4"
  | .i4toi2 => "i4toi2"
  | .i2toi8 => "int28"
  | .i8toi2 => "int82"
  | .i4toi8 => "int48"
  | .i8toi4 => "int84"

def CastOp.isWidening : CastOp → Bool
  | .i2toi4 | .i2toi8 | .i4toi8 => true
  | .i4toi2 | .i8toi2 | .i8toi4 => false

structure CastFamily where
  /-- Function name (e.g., "i2toi4", "int82"). -/
  fnName      : String
  /-- Source type name ("int2", "int4", "int8"). -/
  srcTy       : String
  /-- Source pg_fcinfo decoder (e.g., "decode_i16"). -/
  srcDecoder  : String
  /-- Result type name. -/
  resTy       : String
  /-- Result pg_fcinfo encoder. -/
  resEncoder  : String
  /-- Rust type for source ("i16", "i32", "i64"). -/
  rustSrcTy   : String
  /-- Rust type for result. -/
  rustResTy   : String
  /-- C type for source ("int16", "int32", "int64"). -/
  cSrcTy      : String
  /-- C type for result. -/
  cResTy      : String
  /-- PG_GETARG_* macro name. -/
  pgGetArg    : String
  /-- PG_RETURN_* macro name. -/
  pgReturn    : String
  /-- True if widening (no overflow check needed). -/
  isWidening  : Bool
  /-- For narrowing casts: MIN bound macro (e.g., "SHRT_MIN", "PG_INT32_MIN"). -/
  minBound    : String := ""
  /-- For narrowing casts: MAX bound macro. -/
  maxBound    : String := ""
  /-- For narrowing casts: errmsg text ("smallint out of range", etc). -/
  errmsg      : String := ""

/-- The 6 cast families. Ordered to match real Postgres: int.c functions
(i2toi4, i4toi2) adjacent, then int8.c functions (int28, int48, int82, int84). -/
def families : List CastFamily := [
  -- int.c: i2toi4, i4toi2
  { fnName := "i2toi4", srcTy := "int2", srcDecoder := "decode_i16",
    resTy := "int4", resEncoder := "encode_i32",
    rustSrcTy := "i16", rustResTy := "i32",
    cSrcTy := "int16", cResTy := "int32",
    pgGetArg := "PG_GETARG_INT16", pgReturn := "PG_RETURN_INT32",
    isWidening := true },
  { fnName := "i4toi2", srcTy := "int4", srcDecoder := "decode_i32",
    resTy := "int2", resEncoder := "encode_i16",
    rustSrcTy := "i32", rustResTy := "i16",
    cSrcTy := "int32", cResTy := "int16",
    pgGetArg := "PG_GETARG_INT32", pgReturn := "PG_RETURN_INT16",
    isWidening := false,
    minBound := "SHRT_MIN", maxBound := "SHRT_MAX",
    errmsg := "smallint out of range" },
  -- int8.c: int28, int48, int82, int84
  { fnName := "int28", srcTy := "int2", srcDecoder := "decode_i16",
    resTy := "int8", resEncoder := "encode_i64",
    rustSrcTy := "i16", rustResTy := "i64",
    cSrcTy := "int16", cResTy := "int64",
    pgGetArg := "PG_GETARG_INT16", pgReturn := "PG_RETURN_INT64",
    isWidening := true },
  { fnName := "int48", srcTy := "int4", srcDecoder := "decode_i32",
    resTy := "int8", resEncoder := "encode_i64",
    rustSrcTy := "i32", rustResTy := "i64",
    cSrcTy := "int32", cResTy := "int64",
    pgGetArg := "PG_GETARG_INT32", pgReturn := "PG_RETURN_INT64",
    isWidening := true },
  { fnName := "int82", srcTy := "int8", srcDecoder := "decode_i64",
    resTy := "int2", resEncoder := "encode_i16",
    rustSrcTy := "i64", rustResTy := "i16",
    cSrcTy := "int64", cResTy := "int16",
    pgGetArg := "PG_GETARG_INT64", pgReturn := "PG_RETURN_INT16",
    isWidening := false,
    minBound := "PG_INT16_MIN", maxBound := "PG_INT16_MAX",
    errmsg := "smallint out of range" },
  { fnName := "int84", srcTy := "int8", srcDecoder := "decode_i64",
    resTy := "int4", resEncoder := "encode_i32",
    rustSrcTy := "i64", rustResTy := "i32",
    cSrcTy := "int64", cResTy := "int32",
    pgGetArg := "PG_GETARG_INT64", pgReturn := "PG_RETURN_INT32",
    isWidening := false,
    minBound := "PG_INT32_MIN", maxBound := "PG_INT32_MAX",
    errmsg := "integer out of range" },
]

/-- All cast ops, in source order. -/
def ops : List CastOp := [.i2toi4, .i4toi2, .i2toi8, .i8toi2, .i4toi8, .i8toi4]

end Pg.Ir.Emit.IntCasts
