#!/usr/bin/env bash
# regen-date-arith.sh — emit rules_postgres/rust/pg_date_arith/src/lib.rs from
# the Lean spec at rules_postgres/lean/Pg/Ir/Emit/DateArith.lean.
#
# Companion to regen-int-arith.sh. Same pipeline shape: pre-compile Lean
# deps, run the emit, write to the sibling crate. With --check, verify the
# existing emit matches.
#
# The crate name is `pg_date_arith`; the emit covers the 3-function date
# arithmetic cluster.

set -euo pipefail

LEAN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../lean" && pwd)"
REPO_ROOT="$(cd "$LEAN_DIR/.." && pwd)"
OUT_RS="$REPO_ROOT/rust/pg_date_arith/src/lib.rs"

check_only=0
if [ "${1-}" = "--check" ]; then check_only=1; fi

(
  cd "$LEAN_DIR"
  LEAN_PATH=. lean -o Pg/Ir/Types.olean                Pg/Ir/Types.lean
  LEAN_PATH=. lean -o Pg/Ir/Datum.olean                Pg/Ir/Datum.lean
  LEAN_PATH=. lean -o Pg/Ir/Emit/Common.olean          Pg/Ir/Emit/Common.lean
  LEAN_PATH=. lean -o Pg/Ir/Emit/DateArithCommon.olean  Pg/Ir/Emit/DateArithCommon.lean
  LEAN_PATH=. lean -o Pg/Ir/Emit/DateArith.olean        Pg/Ir/Emit/DateArith.lean
)

TMP="$(mktemp -t date_arith.rs.XXXXXX)"
trap 'rm -f "$TMP"' EXIT
( cd "$LEAN_DIR" && LEAN_PATH=. lean --run Pg/Ir/Emit/DateArith.lean ) > "$TMP"

if [ "$check_only" -eq 1 ]; then
  if diff -u "$OUT_RS" "$TMP"; then
    echo "ok: $OUT_RS matches the Lean emit."
  else
    echo "ERROR: $OUT_RS is stale relative to the Lean source." >&2
    echo "       Run rules_postgres/tools/regen/regen-date-arith.sh to update it." >&2
    exit 1
  fi
else
  cp "$TMP" "$OUT_RS"
  echo "wrote $OUT_RS"
fi
