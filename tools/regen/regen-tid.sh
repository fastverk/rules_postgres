#!/usr/bin/env bash
# Regenerate rules_postgres/rust/pg_tid/src/lib.rs from the
# Lean source of truth in rules_postgres/lean/Pg/Ir/.
#
# This script also regenerates the C emit (tid_emit.c) for AST grounding.
#
# Run when:
#   - Pg/Ir/Emit/TidCommon.lean changes (the family table)
#   - Pg/Ir/Emit/Tid.lean changes (the Rust codegen template)
#   - Pg/Ir/Emit/TidC.lean changes (the C codegen template)
#
# CI uses the `--check` flag to enforce "Lean source-of-truth and generated
# files agree." Local devs run without --check to update the files.

set -euo pipefail

LEAN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../lean" && pwd)"
REPO_ROOT="$(cd "$LEAN_DIR/.." && pwd)"
RUST_TARGET="$REPO_ROOT/rust/pg_tid/src/lib.rs"
C_TARGET="$REPO_ROOT/rust/pg_tid/c_oracle/tid_emit.c"

# Pre-compile Lean dependencies.
(
  cd "$LEAN_DIR"
  LEAN_PATH=. lean -o Pg/Ir/Types.olean             Pg/Ir/Types.lean
  LEAN_PATH=. lean -o Pg/Ir/Datum.olean             Pg/Ir/Datum.lean
  LEAN_PATH=. lean -o Pg/Ir/Emit/TidCommon.olean    Pg/Ir/Emit/TidCommon.lean
  LEAN_PATH=. lean -o Pg/Ir/Emit/Tid.olean          Pg/Ir/Emit/Tid.lean
  LEAN_PATH=. lean -o Pg/Ir/Emit/TidC.olean         Pg/Ir/Emit/TidC.lean
)

# Generate Rust source.
TMP_RUST="$(mktemp -t tid_rust.rs.XXXXXX)"
trap 'rm -f "$TMP_RUST"' EXIT
( cd "$LEAN_DIR" && LEAN_PATH=. lean --run Pg/Ir/Emit/Tid.lean ) > "$TMP_RUST"

# Generate C emit.
TMP_C="$(mktemp -t tid_c.c.XXXXXX)"
trap 'rm -f "$TMP_C" "$TMP_RUST"' EXIT
( cd "$LEAN_DIR" && LEAN_PATH=. lean --run Pg/Ir/Emit/TidC.lean ) > "$TMP_C"

if [[ "${1:-}" == "--check" ]]; then
  if ! diff -u "$RUST_TARGET" "$TMP_RUST"; then
    echo "ERROR: $RUST_TARGET is stale relative to the Lean source." >&2
    echo "       Run rules_postgres/tools/regen/regen-tid.sh to update it." >&2
    exit 1
  fi
  if ! diff -u "$C_TARGET" "$TMP_C"; then
    echo "ERROR: $C_TARGET is stale relative to the Lean source." >&2
    echo "       Run rules_postgres/tools/regen/regen-tid.sh to update it." >&2
    exit 1
  fi
  echo "ok: $RUST_TARGET and $C_TARGET match the Lean emit."
else
  mv "$TMP_RUST" "$RUST_TARGET"
  mv "$TMP_C" "$C_TARGET"
  trap - EXIT
  echo "wrote $RUST_TARGET"
  echo "wrote $C_TARGET"
fi
