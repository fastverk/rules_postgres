#!/usr/bin/env bash
# regen-interval.sh — emit rules_postgres/rust/pg_interval/src/lib.rs
# from the Lean spec at rules_postgres/lean/Pg/Ir/Emit/Interval.lean.
#
# Sister to regen-int-cmp.sh / regen-int-hash.sh / regen-int-arith.sh.

set -euo pipefail

LEAN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../lean" && pwd)"
REPO_ROOT="$(cd "$LEAN_DIR/.." && pwd)"
OUT_RS="$REPO_ROOT/rust/pg_interval/src/lib.rs"

check_only=0
if [ "${1-}" = "--check" ]; then check_only=1; fi

(
  cd "$LEAN_DIR"
  LEAN_PATH=. lean -o Pg/Ir/Emit/IntervalCommon.olean Pg/Ir/Emit/IntervalCommon.lean
  LEAN_PATH=. lean -o Pg/Ir/Emit/Interval.olean       Pg/Ir/Emit/Interval.lean
)

TMP="$(mktemp -t interval.rs.XXXXXX)"
trap 'rm -f "$TMP"' EXIT
( cd "$LEAN_DIR" && LEAN_PATH=. lean --run Pg/Ir/Emit/Interval.lean ) > "$TMP"

if [ "$check_only" -eq 1 ]; then
  if diff -u "$OUT_RS" "$TMP"; then
    echo "ok: $OUT_RS matches the Lean emit."
  else
    echo "ERROR: $OUT_RS is stale relative to the Lean source." >&2
    echo "       Run rules_postgres/tools/regen/regen-interval.sh to update it." >&2
    exit 1
  fi
else
  cp "$TMP" "$OUT_RS"
  echo "wrote $OUT_RS"
fi
