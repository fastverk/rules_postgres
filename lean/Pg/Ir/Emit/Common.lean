/-!
# Pg.Ir.Emit.Common — shared data tables for the int-cmp emit modules.

Both `Pg.Ir.Emit.IntCmp` (Rust emit) and `Pg.Ir.Emit.IntCmpC` (C emit
for AST-level structural verification) share the same `Family` table
and `ops` list. They differ only in the language they render to.

Factored out here so each emit module can have its own `main`
entrypoint (Lean rejects multiple `main` declarations in a single
compilation unit).
-/

namespace Pg.Ir.Emit

/-- How the comparison is rendered for a family.

- `native` uses the target language's built-in `==/<` operators —
  appropriate for types whose Postgres comparison semantics match
  the native semantics (all integer families + bool).

- `pgFloat32` / `pgFloat64` route through helper functions
  (`pgcmp_<op>_f{32,64}` on the Rust side; `float{4,8}_<op>` on the
  C side) — appropriate for float types where Postgres has NaN-aware
  semantics that differ from IEEE-754 (NaN sorts greatest, NaN ==
  NaN).
-/
inductive CmpStyle where
  /-- `a OP b` directly. Works for primitive integer types whose
  Postgres comparison semantics match native (int / date / cash /
  pg_lsn / oid / bool — same width compared with signed-or-unsigned
  per type). -/
  | native
  /-- `(<castTy>) a OP (<castTy>) b` — Rust emits
  `(a as <rustCastTy>) OP (b as <rustCastTy>)`. Used by `char`'s
  ordering ops (lt/le/gt/ge) which Postgres casts to `uint8` even
  though the storage is `int8`. The `cCastTy` and `rustCastTy`
  fields carry the target C and Rust type names respectively. -/
  | nativeCastBoth (cCastTy : String) (rustCastTy : String)
  /-- `pgcmp_<op>_f32(a, b)` — NaN-aware float-4 comparison. -/
  | pgFloat32
  /-- `pgcmp_<op>_f64(a, b)` — NaN-aware float-8 comparison. -/
  | pgFloat64
  /-- `<name>(a, b) <op> 0` — comparison via a Postgres 3-way
  cmp-internal helper. Used by families that route all 6 ops through
  a single `<type>_cmp_internal` helper (timestamp, uuid, interval,
  etc.). The `name` field carries the helper function name. The
  helper exists on BOTH sides: the C emit calls it directly (via the
  appropriate header include); the Rust emit calls a mirror helper
  in pg_fcinfo. -/
  | pgCmpInternal (name : String)
  deriving DecidableEq, Repr

structure Family where
  /-- Postgres function-name prefix (e.g., `int4`, `date_`). -/
  namePrefix : String
  /-- The `pg_fcinfo::decode_*` helper name for arg[0] on the Rust side. -/
  decoder    : String
  /-- The `pg_fcinfo::decode_*` helper name for arg[1] on the Rust side.
  Empty = same as `decoder` (the common case for same-type families).
  When non-empty, this family is cross-type (e.g., int24 with arg[0]:int2,
  arg[1]:int4). -/
  decoder2   : String := ""
  /-- Rust common type to widen both operands to before comparison.
  Empty = no promotion needed. For same-type families this is empty;
  for cross-type families it's the wider type so the comparison
  doesn't fail Rust's strict type-checking (C does this implicitly). -/
  rustPromote : String := ""
  /-- Default comparison style for ops not in `opOverrides`. -/
  cmpStyle   : CmpStyle
  /-- Per-op cmpStyle overrides. Useful for families like `char` where
  eq/ne use signed comparison but lt/le/gt/ge cast to uint8 first.
  Lookup is by Postgres op suffix ("eq", "ne", "lt", ...). -/
  opOverrides : List (String × CmpStyle) := []

def families : List Family := [
  { namePrefix :="int2",        decoder := "decode_i16",  cmpStyle := .native    },
  { namePrefix :="int4",        decoder := "decode_i32",  cmpStyle := .native    },
  { namePrefix :="int8",        decoder := "decode_i64",  cmpStyle := .native    },
  { namePrefix :="date_",       decoder := "decode_i32",  cmpStyle := .native    },  -- DateADT = Int32
  { namePrefix :="timestamp_",  decoder := "decode_i64",  cmpStyle := .pgCmpInternal "timestamp_cmp_internal" },
  { namePrefix :="cash_",       decoder := "decode_i64",  cmpStyle := .native    },  -- Cash = Int64
  { namePrefix :="pg_lsn_",     decoder := "decode_u64",  cmpStyle := .native    },  -- XLogRecPtr = UInt64
  { namePrefix :="oid",         decoder := "decode_u32",  cmpStyle := .native    },  -- Oid = UInt32
  { namePrefix :="bool",        decoder := "decode_bool", cmpStyle := .native    },  -- false < true
  { namePrefix :="float4",      decoder := "decode_f32",  cmpStyle := .pgFloat32 },  -- V1 fmgr: float4eq (no _); helper: float4_eq
  { namePrefix :="float8",      decoder := "decode_f64",  cmpStyle := .pgFloat64 },  -- V1 fmgr: float8eq (no _); helper: float8_eq
  -- char: signed Postgres "char" type (== uint8 storage). eq/ne use
  -- signed comparison; lt/le/gt/ge cast both args to uint8 first.
  { namePrefix :="char",        decoder := "decode_i8",   cmpStyle := .native,
    opOverrides := [
      ("lt", .nativeCastBoth "uint8" "u8"),
      ("le", .nativeCastBoth "uint8" "u8"),
      ("gt", .nativeCastBoth "uint8" "u8"),
      ("ge", .nativeCastBoth "uint8" "u8"),
    ]
  },
  -- Cross-type int comparators: int.c (int2/int4 cross), int8.c (int8 cross).
  -- Both args decoded as their native widths; Rust widens to the larger
  -- type via rustPromote so the comparison type-checks. C uses implicit
  -- widening so its body shape is identical to same-type comparison.
  { namePrefix :="int24",  decoder := "decode_i16", decoder2 := "decode_i32",
    rustPromote := "i32", cmpStyle := .native },
  { namePrefix :="int42",  decoder := "decode_i32", decoder2 := "decode_i16",
    rustPromote := "i32", cmpStyle := .native },
  { namePrefix :="int28",  decoder := "decode_i16", decoder2 := "decode_i64",
    rustPromote := "i64", cmpStyle := .native },
  { namePrefix :="int82",  decoder := "decode_i64", decoder2 := "decode_i16",
    rustPromote := "i64", cmpStyle := .native },
  { namePrefix :="int48",  decoder := "decode_i32", decoder2 := "decode_i64",
    rustPromote := "i64", cmpStyle := .native },
  { namePrefix :="int84",  decoder := "decode_i64", decoder2 := "decode_i32",
    rustPromote := "i64", cmpStyle := .native },
]

/-- The 6 comparison ops as (Postgres suffix, native Rust operator). -/
def ops : List (String × String) := [
  ("eq", "=="),
  ("ne", "!="),
  ("lt", "<"),
  ("le", "<="),
  ("gt", ">"),
  ("ge", ">="),
]

/-- Look up the effective cmpStyle for (family, op suffix). Returns
the override if set, else the family's default. -/
def effectiveCmpStyle (fam : Family) (opSuffix : String) : CmpStyle :=
  match fam.opOverrides.find? (fun (s, _) => s == opSuffix) with
  | some (_, style) => style
  | none            => fam.cmpStyle

/-- Effective decoder for arg[0]. -/
def decoderForArg1 (fam : Family) : String := fam.decoder

/-- Effective decoder for arg[1]. Falls back to `decoder` when
`decoder2` is empty (same-type families). -/
def decoderForArg2 (fam : Family) : String :=
  if fam.decoder2 == "" then fam.decoder else fam.decoder2

end Pg.Ir.Emit
