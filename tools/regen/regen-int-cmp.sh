#!/usr/bin/env bash
# Regenerate rules_postgres/rust/pg_int4_cmp/src/lib.rs from the
# Lean source of truth in rules_postgres/lean/Pg/Ir/.
#
# Run when:
#   - Pg/Ir/Cmp.lean, Pg/Ir/Datum.lean, or Pg/Ir/Types.lean changes
#     (the algebraic spec or encoding bridge)
#   - Pg/Ir/Emit/IntCmp.lean changes (the codegen template or family
#     table — adding a new type family lands here)
#
# CI uses the `--check` flag to enforce "Lean source-of-truth and
# generated Rust agree." Local devs run without --check to actually
# update the file.
#
# In-repo: both the Lean spec and the Rust target live under
# rules_postgres now. Bazel-wiring is still out of scope; this script
# is the canonical regen entry point.

set -euo pipefail

LEAN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../lean" && pwd)"
REPO_ROOT="$(cd "$LEAN_DIR/.." && pwd)"
GENERATED="$REPO_ROOT/rust/pg_int4_cmp/src/lib.rs"

# Pre-compile dependent oleans so `--run` resolves imports.
# Order matters — each module must be compiled after its dependencies.
(
  cd "$LEAN_DIR"
  LEAN_PATH=. lean -o Pg/Ir/Types.olean         Pg/Ir/Types.lean
  LEAN_PATH=. lean -o Pg/Ir/Datum.olean         Pg/Ir/Datum.lean
  LEAN_PATH=. lean -o Pg/Ir/Cmp.olean           Pg/Ir/Cmp.lean
  LEAN_PATH=. lean -o Pg/Ir/Emit/Common.olean   Pg/Ir/Emit/Common.lean
  LEAN_PATH=. lean -o Pg/Ir/Emit/IntCmp.olean   Pg/Ir/Emit/IntCmp.lean
)

# Emit to a temp file so we never lose trailing newlines via shell
# command substitution (bash strips them).
TMP="$(mktemp -t int_cmp.rs.XXXXXX)"
trap 'rm -f "$TMP"' EXIT
( cd "$LEAN_DIR" && LEAN_PATH=. lean --run Pg/Ir/Emit/IntCmp.lean ) > "$TMP"

if [[ "${1:-}" == "--check" ]]; then
  if ! diff -u "$GENERATED" "$TMP"; then
    echo "ERROR: $GENERATED is stale relative to the Lean source." >&2
    echo "       Run rules_postgres/tools/regen/regen-int-cmp.sh to update it." >&2
    exit 1
  fi
  echo "ok: $GENERATED matches the Lean emit."
else
  mv "$TMP" "$GENERATED"
  trap - EXIT
  echo "wrote $GENERATED"
fi
