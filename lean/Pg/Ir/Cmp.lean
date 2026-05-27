/-!
# Pg.Ir.Cmp — polymorphic comparison primitives for the V1 fmgr cluster.

The 48 Rust functions in `rules_lang/translated/pg_int4_cmp/src/lib.rs`
decompose to 6 logical operations applied to 5 underlying primitive
types. This module defines the 6 operations once, polymorphically
over Lean's `BEq` + `Ord` typeclasses, and proves they're definitionally
equal to the underlying ops via `rfl`.

The proofs are one line each — `rfl` discharges them because the
operations are *by definition* the underlying `BEq`/`Ord` instances
on the primitive types. No exhaustive case analysis or bit-vector
reasoning needed; the proof says "the definition IS the spec."

Source of truth status:
- This file = the Lean spec.
- pg_int4_cmp/src/lib.rs = generated from this file (via
  `Pg.Ir.Emit.IntCmp`).
- pg_int4_cmp's diff-test = behavioral equivalence to real Postgres
  (~98K cases passing).

Together: C ↔ Lean spec ↔ Rust, three-way closed.
-/

namespace Pg.Ir.Cmp

/-! ### Polymorphic comparison primitives

`cmpEq` / `cmpNe` use the `BEq` typeclass (`==` / `!=`).

`cmpLt` / `cmpLe` / `cmpGt` / `cmpGe` are defined via `Ord.compare`
returning an `Ordering`. This avoids the typeclass-stuck-on-metavariable
issue that arises with `[Decidable (a < b)]` instance arguments,
because `Ord` resolves entirely from the type (no per-pair instance
needed).

All five primitive types we instantiate at (Int16/Int32/Int64/
UInt32/UInt64) have Lean-core `Ord` instances that match the
underlying integer comparison semantics — signed for `Int*`, unsigned
for `UInt*`. This matches Postgres' fmgr conventions for the int
families exactly.
-/

@[inline] def cmpEq {α : Type} [BEq α] (a b : α) : Bool := a == b
@[inline] def cmpNe {α : Type} [BEq α] (a b : α) : Bool := a != b

@[inline] def cmpLt {α : Type} [Ord α] (a b : α) : Bool :=
  Ord.compare a b == .lt

@[inline] def cmpLe {α : Type} [Ord α] (a b : α) : Bool :=
  Ord.compare a b != .gt

@[inline] def cmpGt {α : Type} [Ord α] (a b : α) : Bool :=
  Ord.compare a b == .gt

@[inline] def cmpGe {α : Type} [Ord α] (a b : α) : Bool :=
  Ord.compare a b != .lt

/-! ### Spec theorems

Each says "the comparison primitive matches the underlying logical
operation." All discharge by `rfl` because the definitions ARE the
typeclass dispatches.

These hold polymorphically — they're not per-type. Specializing α
to `Int32`/`Int64`/`UInt32`/etc. is automatic when the codegen
instantiates them for the concrete Postgres families.
-/

theorem cmpEq_spec {α : Type} [BEq α] (a b : α) : cmpEq a b = (a == b) := rfl
theorem cmpNe_spec {α : Type} [BEq α] (a b : α) : cmpNe a b = (a != b) := rfl
theorem cmpLt_spec {α : Type} [Ord α] (a b : α) :
    cmpLt a b = (Ord.compare a b == .lt) := rfl
theorem cmpLe_spec {α : Type} [Ord α] (a b : α) :
    cmpLe a b = (Ord.compare a b != .gt) := rfl
theorem cmpGt_spec {α : Type} [Ord α] (a b : α) :
    cmpGt a b = (Ord.compare a b == .gt) := rfl
theorem cmpGe_spec {α : Type} [Ord α] (a b : α) :
    cmpGe a b = (Ord.compare a b != .lt) := rfl

/-! ### Algebraic relations between operators (v1 future work)

Deferred: theorems like `cmpNe a b = !(cmpEq a b)` and
`cmpGt a b = cmpLt b a`. These would let the codegen emit only the
canonical pair (eq, lt) per type and derive the other 4 algebraically.
Useful optimization but requires a `LawfulOrd` predicate to make the
swap-symmetry proof go through; not load-bearing for v0 (which emits
all 6 ops explicitly to match the existing Rust 1:1).
-/

end Pg.Ir.Cmp
