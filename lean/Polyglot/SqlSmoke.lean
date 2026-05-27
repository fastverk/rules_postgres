/-
Polyglot.SqlSmoke — cross-repo wiring smoke test for the
`Polyglot.Sql.Ast` generic atlas published from rules_lang.

Phase 1b precondition: Pg.Ast (which lives in this repo's
`lean/Pg/Ast.lean`) will `import Polyglot.Sql.Ast` and define
`Pg.Ast.ExprExt` extending the generic 32-ctor Expr. Before
landing that factor work, this smoke test proves the import
resolves and a value of the generic `Expr Unit` type can be
constructed cross-repo.
-/

import Polyglot.Sql.Ast

namespace Polyglot.SqlSmoke

open Polyglot.Sql.Ast

/-- A trivial expression in the generic SQL atlas, instantiated
    with `Unit` as the dialect-extension type. Proves the module
    elaborates + the type is constructible. -/
def trivialBool : Expr Unit := Expr.litConst (LitConst.bool true)

/-- The `ext` hatch is what dialects use to extend the AST without
    forking it. With `Ext := Unit`, the hatch can only be filled
    with `()` — a degenerate dialect that adds nothing. Real
    dialects (Pg.Ast) parameterize this with a non-trivial
    `ExprExt` inductive. -/
def trivialExt : Expr Unit := Expr.ext ()

end Polyglot.SqlSmoke
