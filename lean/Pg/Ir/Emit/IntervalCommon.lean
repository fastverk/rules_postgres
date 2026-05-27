/-!
# Pg.Ir.Emit.IntervalCommon — shared family table for the interval-
arithmetic emit modules (`Interval` for Rust, `IntervalC` for the
AST-grounding C).

This is the first **palloc-result** cluster in Pg.Ir — the body shape
allocates an `Interval` and returns a pointer-valued Datum, instead of
encoding a primitive value into the low bits of a u64 (the existing
cmp/hash/arith pattern). The split between the "uniform fmgr stub"
and the "per-family `_internal` helper" mirrors Postgres' own
factoring in `src/backend/utils/adt/timestamp.c`.

Cluster scope grows in stages:
  v1 (this module's first landing): interval_um (1 fn)
  v2 (next): interval_pl, interval_mi (binary add/sub on time/day/month
            with 4-way infinity-sentinel dispatch)
  v3 (later): interval_mul, interval_div (float8 second arg)
  v4 (later): interval_um_internal-style helpers in the cmp pattern
            for date/timestamp arithmetic with infinity sentinels
-/

namespace Pg.Ir.Emit.Interval

/-- Per-family body shape. Each variant determines:
  - whether to emit a `<fn>_internal` helper (unary only)
  - the fmgr stub's dispatch structure
  - which `pg_*_overflow` primitives are called

`unaryNegate` — has `interval_um_internal` helper. Three-way dispatch:
  NOBEGIN → NOEND, NOEND → NOBEGIN, normal → per-field overflow-checked
  negate with post-check `INTERVAL_NOT_FINITE(result)`.

`binaryPl` — NO internal helper; dispatch directly in fmgr stub.
  Four-way infinity dispatch; finite case delegates to `finite_interval_pl`.
  Uses addition overflow checks on finite case.

`binaryMi` — NO internal helper; dispatch directly in fmgr stub.
  Four-way infinity dispatch (different than Pl); finite case delegates to
  `finite_interval_mi`. Uses subtraction overflow checks on finite case.
-/
inductive IntervalBody where
  | unaryNegate
  | binaryPl
  | binaryMi
  deriving DecidableEq, Repr

structure IntervalFamily where
  /-- Postgres fmgr function name (e.g., `interval_um`). -/
  fnName : String
  /-- Number of Interval args. 1 = unary (just interval_um); 2 =
  binary (interval_pl, interval_mi). For v1 only `1` is supported. -/
  arity  : Nat
  /-- The body shape, which selects the `_internal` helper template. -/
  body   : IntervalBody

/-- v1-v2 families. -/
def families : List IntervalFamily := [
  { fnName := "interval_um", arity := 1, body := .unaryNegate },
  { fnName := "interval_pl", arity := 2, body := .binaryPl },
  { fnName := "interval_mi", arity := 2, body := .binaryMi }
]

end Pg.Ir.Emit.Interval
