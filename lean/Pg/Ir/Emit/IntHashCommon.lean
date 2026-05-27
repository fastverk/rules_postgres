/-!
# Pg.Ir.Emit.IntHashCommon — shared family table for the integer-hash
emit modules (`IntHash` for Rust, `IntHashC` for the AST-grounding C).

Same factoring pattern as `Common.lean` for the cmp cluster: keeps each
emit module a single compilation unit with its own `main`.
-/

namespace Pg.Ir.Emit.IntHash

/-- How to build the `u32` value that `hash_bytes_uint32` consumes
from the decoded fmgr argument. -/
inductive HashBody where
  | signedToU32
  | unsignedAsU32
  | int8Split
  deriving DecidableEq, Repr

structure HashFamily where
  /-- Postgres fmgr function name (e.g., `hashint4`). -/
  fnName  : String
  /-- The `pg_fcinfo::decode_*` helper name for arg[0]. -/
  decoder : String
  /-- How the body transforms the decoded value into the u32 hash input. -/
  body    : HashBody
  /-- The `PG_GETARG_*` macro name on the C emit side. -/
  pgGetArg : String
  /-- The explicit C-side cast on the operand. Empty when no cast is
  needed (hashint4: arg is already `int32`). Trailing space included. -/
  cCast    : String := ""
  /-- If true, this is an `*extended` variant: takes a second `int64`
  seed argument and routes through `hash_uint32_extended`, returning
  uint64 instead of uint32. The arg[0] u32 computation is identical
  to the non-extended sibling. -/
  extended : Bool := false

/-- The 12 families covered in v1 (6 standard + 6 *extended). -/
def families : List HashFamily := [
  { fnName := "hashchar", decoder := "decode_i8",  body := .signedToU32,
    pgGetArg := "PG_GETARG_CHAR",  cCast := "(int32) " },
  { fnName := "hashint2", decoder := "decode_i16", body := .signedToU32,
    pgGetArg := "PG_GETARG_INT16", cCast := "(int32) " },
  { fnName := "hashint4", decoder := "decode_i32", body := .signedToU32,
    pgGetArg := "PG_GETARG_INT32" },
  { fnName := "hashint8", decoder := "decode_i64", body := .int8Split,
    pgGetArg := "PG_GETARG_INT64" },
  { fnName := "hashoid",  decoder := "decode_u32", body := .unsignedAsU32,
    pgGetArg := "PG_GETARG_OID",   cCast := "(uint32) " },
  { fnName := "hashenum", decoder := "decode_u32", body := .unsignedAsU32,
    pgGetArg := "PG_GETARG_OID",   cCast := "(uint32) " },
  { fnName := "hashcharextended", decoder := "decode_i8",  body := .signedToU32,
    pgGetArg := "PG_GETARG_CHAR",  cCast := "(int32) ",  extended := true },
  { fnName := "hashint2extended", decoder := "decode_i16", body := .signedToU32,
    pgGetArg := "PG_GETARG_INT16", cCast := "(int32) ",  extended := true },
  { fnName := "hashint4extended", decoder := "decode_i32", body := .signedToU32,
    pgGetArg := "PG_GETARG_INT32",                       extended := true },
  { fnName := "hashint8extended", decoder := "decode_i64", body := .int8Split,
    pgGetArg := "PG_GETARG_INT64",                       extended := true },
  { fnName := "hashoidextended",  decoder := "decode_u32", body := .unsignedAsU32,
    pgGetArg := "PG_GETARG_OID",   cCast := "(uint32) ", extended := true },
  { fnName := "hashenumextended", decoder := "decode_u32", body := .unsignedAsU32,
    pgGetArg := "PG_GETARG_OID",   cCast := "(uint32) ", extended := true },
]

end Pg.Ir.Emit.IntHash
