/-!
# Pg.Ir.Types — Postgres type abbreviations.

Postgres' V1 fmgr ABI passes args as `Datum` (= `uintptr_t` = `UInt64`
on 64-bit builds). Each SQL type has a typedef that resolves to a
primitive integer or float; the encoding/decoding to/from `Datum`
follows from those typedefs.

This module reifies the canonical 17.6 typedef chain so the Lean IR
can talk about `DateADT` directly rather than always saying
`Int32`. `abbrev` means it's a NAME for the underlying type, not a
new opaque type — `Int32` and `DateADT` are interchangeable in proofs
and in pattern-matching.

Sources (Postgres 17.6):
- DateADT     in src/include/utils/date.h
- Timestamp   in src/include/datatype/timestamp.h
- Cash        in src/include/utils/cash.h
- XLogRecPtr  in src/include/access/xlogdefs.h
- Oid         in src/include/postgres_ext.h
-/

namespace Pg.Ir.Types

/-- `Datum` is Postgres' uniform value type at the V1 fmgr boundary.
On 64-bit builds it's `uintptr_t`, i.e. `UInt64`. The pg_fcinfo Rust
crate uses `type Datum = u64;` for the same reason. -/
abbrev Datum := UInt64

/-- Postgres `DateADT`. Days since 2000-01-01, encoded as int32. -/
abbrev DateADT := Int32

/-- Postgres `Timestamp`. Microseconds since 2000-01-01 epoch, int64. -/
abbrev Timestamp := Int64

/-- Postgres `Cash`. Currency type, int64. -/
abbrev Cash := Int64

/-- Postgres `XLogRecPtr`. Pointer into the WAL, uint64. -/
abbrev XLogRecPtr := UInt64

/-- Postgres `Oid`. Object identifier, uint32. -/
abbrev Oid := UInt32

end Pg.Ir.Types
