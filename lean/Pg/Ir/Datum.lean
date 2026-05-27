import Pg.Ir.Types

/-!
# Pg.Ir.Datum — `Datum` encoding bridge.

Postgres' V1 fmgr ABI funnels every value through `Datum` (= `UInt64`
on 64-bit builds). Each SQL type has a canonical encoding to/from
`Datum`; the macros `Int32GetDatum`/`DatumGetInt32`/etc. wrap those
encodings on the C side.

This module mirrors them in Lean. The bit-encoding rules below
match Postgres 17.6's `src/include/postgres.h` and the pg_fcinfo
Rust crate's `encode_*` / `decode_*` helpers verbatim:

  Int16   → low 16 bits of UInt64, zero-extended via UInt16 cast
  Int32   → low 32 bits via UInt32
  Int64   → full 64 bits
  UInt32  → low 32 bits
  UInt64  → full 64 bits
  Bool    → 0 or 1

The casts go through the matching unsigned width first (preserving
the two's-complement bit pattern), then widen to `UInt64`. The Lean
definitions below mirror that.

**Round-trip theorems** are deferred to a v1 of this file; they
require Lean-core bit-vector lemmas about `UInt32.toUInt64.toUInt32 = id`
that may need explicit derivation. The empirical round-trip is
already validated by the pg_int4_cmp diff-test (~98K cases passing
against real Postgres).
-/

namespace Pg.Ir.Datum

open Pg.Ir.Types

/-! ### Encoders — `<T>GetDatum` analogues -/

@[inline] def encodeI16 (v : Int16)  : Datum := v.toUInt16.toUInt64
@[inline] def encodeI32 (v : Int32)  : Datum := v.toUInt32.toUInt64
@[inline] def encodeI64 (v : Int64)  : Datum := v.toUInt64
@[inline] def encodeU32 (v : UInt32) : Datum := v.toUInt64
@[inline] def encodeU64 (v : UInt64) : Datum := v
@[inline] def encodeBool (b : Bool)  : Datum := if b then 1 else 0

/-! ### Decoders — `DatumGet<T>` analogues -/

@[inline] def decodeI16 (d : Datum) : Int16  := d.toUInt16.toInt16
@[inline] def decodeI32 (d : Datum) : Int32  := d.toUInt32.toInt32
@[inline] def decodeI64 (d : Datum) : Int64  := d.toInt64
@[inline] def decodeU32 (d : Datum) : UInt32 := d.toUInt32
@[inline] def decodeU64 (d : Datum) : UInt64 := d
@[inline] def decodeBool (d : Datum) : Bool  := (d &&& 1) != 0

end Pg.Ir.Datum
