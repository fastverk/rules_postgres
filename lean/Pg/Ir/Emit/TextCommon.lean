/-!
# Pg.Ir.Emit.TextCommon — second varlena-using Pg.Ir cluster.

Sister to ByteaCommon. Text and bytea share the same varlena layout
(4-byte header + payload), but text is logically UTF-8-sequence whereas
bytea is binary. For concatenation purposes, the byte-level operation is
identical.

v0 scope: textcat (1 fn). Both text and bytea use identical varlena-level
helpers (decode_bytea_p / encode_bytea_p reused; text pointers are
equivalent at the Datum level).
-/

namespace Pg.Ir.Emit.Text

inductive TextBody where
  /-- `text_catenate(t1, t2)` — palloc concat. -/
  | catenate
  deriving DecidableEq, Repr

structure TextFamily where
  fnName : String
  arity  : Nat
  body   : TextBody

def families : List TextFamily := [
  { fnName := "textcat", arity := 2, body := .catenate },
]

end Pg.Ir.Emit.Text
