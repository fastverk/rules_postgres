/-
Pg.Ir — Lean IR for the Postgres C→Rust translation pipeline.

Sibling namespace to `Pg.Ast` (libpg_query parse-tree AST) and
`Pg.Ty` (typed schema). `Pg.Ir` covers the *runtime semantics*
of Postgres' V1 fmgr functions — the layer between an AST and an
emitted Rust source file.

Layout:
- `Pg.Ir.Types` — Postgres type abbreviations (DateADT = Int32, etc.).
- `Pg.Ir.Datum` — Datum encoding/decoding bridge (mirrors the
  pg_fcinfo Rust crate; matches Postgres 17.6's macros bit-for-bit).
- `Pg.Ir.Cmp`   — polymorphic comparison primitives with `rfl`-grade
  spec theorems.
- `Pg.Ir.Emit.IntCmp` — codegens the 48-function int/date/timestamp/
  cash/pg_lsn/oid comparison cluster as Rust source for
  `rules_lang/translated/pg_int4_cmp/`.

Source-of-truth status:
- These Lean files are the spec.
- The Rust source under `rules_lang/translated/` is regenerated from
  them via `tools/regen/regen-int-cmp.sh`.
- The diff-test gate in pg_int4_cmp tests behavioral equivalence
  against real Postgres 17.6 (~98K cases passing).

Together: C ↔ Lean spec ↔ Rust, three-way closed.
-/

import Pg.Ir.Types
import Pg.Ir.Datum
import Pg.Ir.Cmp
import Pg.Ir.Emit.Common
import Pg.Ir.Emit.IntCmp
import Pg.Ir.Emit.IntCmpC
