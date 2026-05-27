/-!
# Pg.Ir.Emit.ByteaCommon — first varlena-using Pg.Ir cluster.

Sister to IntervalCommon. Body shapes differ — interval is fixed-size
(16 bytes) palloc; bytea is variable-size (computed from input sizes).
Both share "palloc-result + delegate-to-static-helper + return pointer"
at the fmgr-stub level.

v0 scope: byteacat (1 fn). v1+ candidates: bytea_substr, byteaeq /
byteaoctetlen (read-only, no palloc — different shape; defer until the
read-only-varlena body template lands).
-/

namespace Pg.Ir.Emit.Bytea

inductive ByteaBody where
  /-- `bytea_catenate(t1, t2)` — palloc concat. -/
  | catenate
  /-- `byteaoctetlen` — `toast_raw_datum_size(arg0) - VARHDRSZ`, no
  detoast, no palloc. Returns int4 (payload byte count). -/
  | octetLen
  /-- `byteaeq` — fast-path length check, then memcmp. No palloc.
  Returns bool. -/
  | eq
  deriving DecidableEq, Repr

structure ByteaFamily where
  fnName : String
  arity  : Nat
  body   : ByteaBody

def families : List ByteaFamily := [
  { fnName := "byteacat",      arity := 2, body := .catenate },
  { fnName := "byteaoctetlen", arity := 1, body := .octetLen },
  { fnName := "byteaeq",       arity := 2, body := .eq       },
]

end Pg.Ir.Emit.Bytea
