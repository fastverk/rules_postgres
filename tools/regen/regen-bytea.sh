#!/usr/bin/env bash
set -euo pipefail

LEAN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../lean" && pwd)"
REPO_ROOT="$(cd "$LEAN_DIR/.." && pwd)"
OUT_RS="$REPO_ROOT/rust/pg_bytea/src/lib.rs"

check_only=0
if [ "${1-}" = "--check" ]; then check_only=1; fi

(
  cd "$LEAN_DIR"
  LEAN_PATH=. lean -o Pg/Ir/Emit/ByteaCommon.olean Pg/Ir/Emit/ByteaCommon.lean
  LEAN_PATH=. lean -o Pg/Ir/Emit/Bytea.olean       Pg/Ir/Emit/Bytea.lean
)

TMP="$(mktemp -t bytea.rs.XXXXXX)"
trap 'rm -f "$TMP"' EXIT
( cd "$LEAN_DIR" && LEAN_PATH=. lean --run Pg/Ir/Emit/Bytea.lean ) > "$TMP"

if [ "$check_only" -eq 1 ]; then
  if diff -u "$OUT_RS" "$TMP"; then
    echo "ok: $OUT_RS matches the Lean emit."
  else
    echo "ERROR: $OUT_RS is stale relative to the Lean source." >&2
    exit 1
  fi
else
  cp "$TMP" "$OUT_RS"
  echo "wrote $OUT_RS"
fi
