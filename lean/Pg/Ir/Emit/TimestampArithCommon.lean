/-!
# Pg.Ir.Emit.TimestampArithCommon — shared family table for the timestamp-
arithmetic emit modules (`TimestampArith` for Rust, `TimestampArithC` for
the AST-grounding C).

This cluster vendorizes three functions from Postgres'
`src/backend/utils/adt/timestamp.c`:

  `timestamp_pl_interval` (Timestamp + Interval → Timestamp)
  `timestamp_mi_interval` (Timestamp - Interval → Timestamp)
  `timestamp_mi` (Timestamp - Timestamp → Interval)

All three handle infinity sentinels and palloc their results (timestamp_mi
returns Interval, the other two take it as input). The bodies follow
Postgres' factoring: decode args, 4-way infinity dispatch, delegate to
finite-case helpers for compound arithmetic (month + day + time).

Family scope (v1):
  - timestamp_pl_interval: timestamp + interval → timestamp, 4-way
    infinity dispatch, then sequential month/day/time arithmetic via
    timestamp2tm / tm2timestamp cycles.
  - timestamp_mi_interval: timestamp - interval → timestamp; delegates to
    interval_um (negate the span) then calls timestamp_pl_interval.
  - timestamp_mi: timestamp - timestamp → interval, palloc result,
    4-way infinity dispatch, then direct time field subtraction (month/day = 0).
-/

namespace Pg.Ir.Emit.TimestampArith

/-- Per-function body shape and semantics. -/
inductive TimestampOp where
  | pl_interval   -- timestamp + interval → timestamp
  | mi_interval   -- timestamp - interval → timestamp (delegates to negate then pl)
  | mi            -- timestamp - timestamp → interval (palloc'd result)
  deriving DecidableEq, Repr

structure TimestampFamily where
  /-- Postgres fmgr function name. -/
  fnName : String
  /-- The operation shape. -/
  op     : TimestampOp

/-- v1 families: all three timestamp operations. -/
def families : List TimestampFamily := [
  { fnName := "timestamp_pl_interval", op := .pl_interval },
  { fnName := "timestamp_mi_interval", op := .mi_interval },
  { fnName := "timestamp_mi",          op := .mi }
]

end Pg.Ir.Emit.TimestampArith
