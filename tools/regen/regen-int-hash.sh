#!/usr/bin/env bash
# regen-int-hash.sh — emit rules_postgres/rust/pg_int4_hash/src/lib.rs
# from the Lean spec at rules_postgres/lean/Pg/Ir/Emit/IntHash.lean.
#
# Companion to regen-int-cmp.sh. Same pipeline shape: pre-compile Lean
# deps, run the emit, write to the in-repo Rust target. With --check,
# verify the existing emit matches.

set -euo pipefail

LEAN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../lean" && pwd)"
REPO_ROOT="$(cd "$LEAN_DIR/.." && pwd)"
OUT_RS="$REPO_ROOT/rust/pg_int4_hash/src/lib.rs"

check_only=0
if [ "${1-}" = "--check" ]; then check_only=1; fi

(
  cd "$LEAN_DIR"
  LEAN_PATH=. lean -o Pg/Ir/Types.olean        Pg/Ir/Types.lean
  LEAN_PATH=. lean -o Pg/Ir/Datum.olean        Pg/Ir/Datum.lean
  LEAN_PATH=. lean -o Pg/Ir/Cmp.olean          Pg/Ir/Cmp.lean
  LEAN_PATH=. lean -o Pg/Ir/Emit/Common.olean         Pg/Ir/Emit/Common.lean
  LEAN_PATH=. lean -o Pg/Ir/Emit/IntHashCommon.olean   Pg/Ir/Emit/IntHashCommon.lean
  LEAN_PATH=. lean -o Pg/Ir/Emit/IntHash.olean         Pg/Ir/Emit/IntHash.lean
)

TMP="$(mktemp -t int_hash.rs.XXXXXX)"
trap 'rm -f "$TMP"' EXIT
( cd "$LEAN_DIR" && LEAN_PATH=. lean --run Pg/Ir/Emit/IntHash.lean ) > "$TMP"

if [ "$check_only" -eq 1 ]; then
  if diff -u "$OUT_RS" "$TMP"; then
    echo "ok: $OUT_RS matches the Lean emit."
  else
    echo "ERROR: $OUT_RS is stale relative to the Lean source." >&2
    echo "       Run rules_postgres/tools/regen/regen-int-hash.sh to update it." >&2
    exit 1
  fi
else
  cp "$TMP" "$OUT_RS"
  echo "wrote $OUT_RS"
fi
