/-!
# Pg.Ir.Emit.MacaddrCommon — shared data tables for macaddr cmp + hash cluster.

macaddr in Postgres is a 6-byte fixed-size struct (a, b, c, d, e, f as unsigned char),
passed by pointer. The comparison operators all delegate to a static `macaddr_cmp_internal`
helper which compares the high 3 bytes (hibits), then the low 3 bytes (lobits); the hash
operators use `hash_any` / `hash_any_extended` on the 6-byte payload.

Families:
  - 6 comparison ops (eq, ne, lt, le, gt, ge) all via `macaddr_cmp_internal`
  - 1 3-way cmp op via `macaddr_cmp_internal`
  - 2 hash ops (hash, hash_extended) via hash_any/hash_any_extended

This module factors the shared Family and op definitions so each emit module
(Rust and C) can have its own `main` entrypoint.
-/

namespace Pg.Ir.Emit

/-- How the macaddr hash body is constructed. -/
inductive MacaddrHashBody where
  | fixedHash6
  deriving DecidableEq, Repr

structure MacaddrCmpFamily where
  /-- Postgres function-name suffix (e.g., "eq", "ne", "lt", "le", "gt", "ge"). -/
  suffix : String
  /-- The comparison operator on the result of macaddr_cmp_internal.
  "< 0" for lt, "<= 0" for le, "== 0" for eq, etc. -/
  cmpOp : String
  deriving DecidableEq

structure MacaddrHashFamily where
  /-- Postgres fmgr function name (e.g., "hashmacaddr"). -/
  fnName : String
  /-- The `pg_fcinfo::decode_macaddr_p` or similar helper on Rust side. -/
  decoderName : String := "decode_macaddr_p"
  /-- How the hash is constructed from the macaddr data. -/
  body : MacaddrHashBody
  /-- If true, this is a `*extended` variant: takes a second int64 seed and
  returns u64. Otherwise returns u32. -/
  extended : Bool := false

/-- The 6 comparison operators for macaddr. -/
def macaddrCmpOps : List MacaddrCmpFamily := [
  { suffix := "eq", cmpOp := "== 0" },
  { suffix := "ne", cmpOp := "!= 0" },
  { suffix := "lt", cmpOp := "< 0" },
  { suffix := "le", cmpOp := "<= 0" },
  { suffix := "gt", cmpOp := "> 0" },
  { suffix := "ge", cmpOp := ">= 0" },
]

/-- The 2 hash families (hashmacaddr, hashmacaddrextended). -/
def macaddrHashFamilies : List MacaddrHashFamily := [
  { fnName := "hashmacaddr", body := .fixedHash6, extended := false },
  { fnName := "hashmacaddrextended", body := .fixedHash6, extended := true },
]

end Pg.Ir.Emit
