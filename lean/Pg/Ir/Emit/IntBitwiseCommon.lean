/-!
# Pg.Ir.Emit.IntBitwiseCommon — int bitwise V1 fmgr cluster (18 fns =
6 ops × 3 widths). Pure: no ereport, no palloc.

Three body shapes:
  - binarySameType — and / or / xor, args both same width, op symbol
  - unary           — not (~arg1)
  - shift           — shl / shr, arg2 is always int32 regardless of
                     arg1's width. Rust emit uses `wrapping_shl/_shr`
                     to match C's x86_64 mod-width semantics across
                     all arg2 values (including arg2 ≥ width).
-/

namespace Pg.Ir.Emit.IntBitwise

inductive BitwiseBody where
  | binarySameType
  | unary
  | shift
  deriving DecidableEq, Repr

inductive BitwiseOp where
  | And | Or | Xor | Not | Shl | Shr
  deriving DecidableEq, Repr

structure BitwiseFamily where
  fnName   : String
  rustTy   : String
  cType    : String
  decoder  : String
  encoder  : String
  pgGetArg : String
  pgReturn : String
  body     : BitwiseBody
  op       : BitwiseOp

def opRustSymbol : BitwiseOp → String
  | .And => "&"
  | .Or  => "|"
  | .Xor => "^"
  | .Not => "!"        -- prefix in Rust
  | .Shl => "wrapping_shl"
  | .Shr => "wrapping_shr"

def opCSymbol : BitwiseOp → String
  | .And => "&"
  | .Or  => "|"
  | .Xor => "^"
  | .Not => "~"        -- prefix in C
  | .Shl => "<<"
  | .Shr => ">>"

/-- The 18 families. -/
def families : List BitwiseFamily :=
  let widths := [
    ("int2", "i16", "int16", "decode_i16", "encode_i16", "PG_GETARG_INT16", "PG_RETURN_INT16"),
    ("int4", "i32", "int32", "decode_i32", "encode_i32", "PG_GETARG_INT32", "PG_RETURN_INT32"),
    ("int8", "i64", "int64", "decode_i64", "encode_i64", "PG_GETARG_INT64", "PG_RETURN_INT64"),
  ]
  let ops : List (String × BitwiseBody × BitwiseOp) := [
    ("and", .binarySameType, .And),
    ("or",  .binarySameType, .Or),
    ("xor", .binarySameType, .Xor),
    ("not", .unary,          .Not),
    ("shl", .shift,          .Shl),
    ("shr", .shift,          .Shr),
  ]
  Id.run do
    let mut acc : List BitwiseFamily := []
    for (prefix_, rust, c, dec, enc, gg, pr) in widths do
      for (suffix, body, op) in ops do
        acc := acc ++ [{
          fnName := prefix_ ++ suffix,
          rustTy := rust, cType := c,
          decoder := dec, encoder := enc,
          pgGetArg := gg, pgReturn := pr,
          body := body, op := op,
        }]
    return acc

end Pg.Ir.Emit.IntBitwise
