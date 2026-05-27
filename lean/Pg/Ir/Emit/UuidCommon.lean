/-!
# Pg.Ir.Emit.UuidCommon — shared data tables for UUID cmp + hash cluster.

UUID in Postgres is a 16-byte fixed-size struct (similar to Interval), passed
by pointer. The comparison operators all delegate to a static `uuid_internal_cmp`
helper (memcmp-based); the hash operators use Jenkins/lookup3 hashing.

Families:
  - 6 comparison ops (eq, ne, lt, le, gt, ge) all via `uuid_internal_cmp`
  - 2 hash ops (hash, hash_extended) via Jenkins/lookup3

This module factors the shared Family and op definitions so each emit module
(Rust and C) can have its own `main` entrypoint.
-/

namespace Pg.Ir.Emit

/-- How the UUID hash body is constructed. -/
inductive UuidHashBody where
  | fixedHash16
  deriving DecidableEq, Repr

structure UuidCmpFamily where
  /-- Postgres function-name suffix (e.g., "eq", "ne", "lt", "le", "gt", "ge"). -/
  suffix : String
  /-- The comparison operator on the result of uuid_internal_cmp.
  "< 0" for lt, "<= 0" for le, "== 0" for eq, etc. -/
  cmpOp : String
  deriving DecidableEq

structure UuidHashFamily where
  /-- Postgres fmgr function name (e.g., "uuid_hash"). -/
  fnName : String
  /-- The `pg_fcinfo::decode_uuid_p` or similar helper on Rust side. -/
  decoderName : String := "decode_uuid_p"
  /-- How the hash is constructed from the UUID data. -/
  body : UuidHashBody
  /-- If true, this is a `*extended` variant: takes a second int64 seed and
  returns u64. Otherwise returns u32. -/
  extended : Bool := false

/-- The 6 comparison operators for UUID. -/
def uuidCmpOps : List UuidCmpFamily := [
  { suffix := "eq", cmpOp := "== 0" },
  { suffix := "ne", cmpOp := "!= 0" },
  { suffix := "lt", cmpOp := "< 0" },
  { suffix := "le", cmpOp := "<= 0" },
  { suffix := "gt", cmpOp := "> 0" },
  { suffix := "ge", cmpOp := ">= 0" },
]

/-- The 2 hash families (hash, hash_extended). -/
def uuidHashFamilies : List UuidHashFamily := [
  { fnName := "uuid_hash", body := .fixedHash16, extended := false },
  { fnName := "uuid_hash_extended", body := .fixedHash16, extended := true },
]

end Pg.Ir.Emit
