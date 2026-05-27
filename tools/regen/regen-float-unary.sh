#!/usr/bin/env bash
set -euo pipefail

LEAN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../lean" && pwd)"
REPO_ROOT="$(cd "$LEAN_DIR/.." && pwd)"
OUT_RS="$REPO_ROOT/rust/pg_float_unary/src/lib.rs"

check_only=0
if [ "${1-}" = "--check" ]; then check_only=1; fi

(
  cd "$LEAN_DIR"
  LEAN_PATH=. lean -o Pg/Ir/Emit/FloatUnaryCommon.olean Pg/Ir/Emit/FloatUnaryCommon.lean
  LEAN_PATH=. lean -o Pg/Ir/Emit/FloatUnary.olean       Pg/Ir/Emit/FloatUnary.lean
)

TMP="$(mktemp -t float_unary.rs.XXXXXX)"
trap 'rm -f "$TMP"' EXIT
( cd "$LEAN_DIR" && LEAN_PATH=. lean --run Pg/Ir/Emit/FloatUnary.lean ) > "$TMP"

if [ "$check_only" -eq 1 ]; then
  if diff -u "$OUT_RS" "$TMP"; then
    echo "ok: $OUT_RS matches the Lean emit."
  else
    echo "ERROR: $OUT_RS is stale relative to the Lean source." >&2
    echo "       Run rules_postgres/tools/regen/regen-float-unary.sh to update it." >&2
    exit 1
  fi
else
  cp "$TMP" "$OUT_RS"
  echo "wrote $OUT_RS"
fi
