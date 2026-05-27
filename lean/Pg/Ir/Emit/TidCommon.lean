/-!
# Pg.Ir.Emit.TidCommon — shared data tables for tid cmp + hash cluster.

tid (tuple identifier) in Postgres is a 6-byte fixed-size struct:
  - BlockIdData (4 bytes): ip_blkid
  - OffsetNumber (2 bytes): ip_posid
Total: 6 bytes. Passed by pointer.

The comparison operators all delegate to a static `ItemPointerCompare`
helper which compares blocks first, then offsets. The hash operators
use `hash_any` / `hash_any_extended` on the 6-byte payload.

Families:
  - 6 comparison ops (tideq, tidne, tidlt, tidle, tidgt, tidge) all via ItemPointerCompare
  - 1 3-way cmp op (bttidcmp) via ItemPointerCompare
  - 2 clamping ops (tidlarger, tidsmaller) via ItemPointerCompare
  - 2 hash ops (hashtid, hashtidextended) via hash_any/hash_any_extended

This module factors the shared Family and op definitions so each emit module
(Rust and C) can have its own `main` entrypoint.
-/

namespace Pg.Ir.Emit

/-- How the tid hash body is constructed. -/
inductive TidHashBody where
  | fixedHash6
  deriving DecidableEq, Repr

structure TidCmpFamily where
  /-- Postgres function-name suffix (e.g., "eq", "ne", "lt", "le", "gt", "ge"). -/
  suffix : String
  /-- The comparison operator on the result of ItemPointerCompare.
  "< 0" for lt, "<= 0" for le, "== 0" for eq, etc. -/
  cmpOp : String
  deriving DecidableEq

structure TidClampFamily where
  /-- Postgres fmgr function name (e.g., "tidlarger", "tidsmaller"). -/
  fnName : String
  /-- The comparison operator (">= 0" for larger, "<= 0" for smaller). -/
  cmpOp : String
  deriving DecidableEq

structure TidHashFamily where
  /-- Postgres fmgr function name (e.g., "hashtid"). -/
  fnName : String
  /-- The `pg_fcinfo::decode_itempointer_p` or similar helper on Rust side. -/
  decoderName : String := "decode_itempointer_p"
  /-- How the hash is constructed from the tid data. -/
  body : TidHashBody
  /-- If true, this is a `*extended` variant: takes a second int64 seed and
  returns u64. Otherwise returns u32. -/
  extended : Bool := false

/-- The 6 comparison operators for tid. -/
def tidCmpOps : List TidCmpFamily := [
  { suffix := "eq", cmpOp := "== 0" },
  { suffix := "ne", cmpOp := "!= 0" },
  { suffix := "lt", cmpOp := "< 0" },
  { suffix := "le", cmpOp := "<= 0" },
  { suffix := "gt", cmpOp := "> 0" },
  { suffix := "ge", cmpOp := ">= 0" },
]

/-- The 2 clamping families (tidlarger, tidsmaller). -/
def tidClampFamilies : List TidClampFamily := [
  { fnName := "tidlarger", cmpOp := ">= 0" },
  { fnName := "tidsmaller", cmpOp := "<= 0" },
]

/-- The 2 hash families (hashtid, hashtidextended). -/
def tidHashFamilies : List TidHashFamily := [
  { fnName := "hashtid", body := .fixedHash6, extended := false },
  { fnName := "hashtidextended", body := .fixedHash6, extended := true },
]

end Pg.Ir.Emit
