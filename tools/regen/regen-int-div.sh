#!/usr/bin/env bash
# regen-int-div.sh — emit rules_postgres/rust/pg_int4_div/src/lib.rs from
# the Lean spec at rules_postgres/lean/Pg/Ir/Emit/IntDiv.lean.
#
# Companion to regen-int-arith.sh. Same pipeline shape: pre-compile Lean
# deps, run the emit, write to the Rust crate. With --check, verify the
# existing emit matches.

set -euo pipefail

LEAN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../lean" && pwd)"
REPO_ROOT="$(cd "$LEAN_DIR/.." && pwd)"
OUT_RS="$REPO_ROOT/rust/pg_int4_div/src/lib.rs"

check_only=0
if [ "${1-}" = "--check" ]; then check_only=1; fi

(
  cd "$LEAN_DIR"
  LEAN_PATH=. lean -o Pg/Ir/Types.olean                Pg/Ir/Types.lean
  LEAN_PATH=. lean -o Pg/Ir/Datum.olean                Pg/Ir/Datum.lean
  LEAN_PATH=. lean -o Pg/Ir/Emit/Common.olean          Pg/Ir/Emit/Common.lean
  LEAN_PATH=. lean -o Pg/Ir/Emit/IntDivCommon.olean    Pg/Ir/Emit/IntDivCommon.lean
  LEAN_PATH=. lean -o Pg/Ir/Emit/IntDiv.olean          Pg/Ir/Emit/IntDiv.lean
)

WORK="$(mktemp -t pgir-div-regen.XXXXXX)"
trap 'rm -f "$WORK"' EXIT

( cd "$LEAN_DIR" && LEAN_PATH=. lean --run Pg/Ir/Emit/IntDiv.lean ) > "$WORK"

if [ "$check_only" = 1 ]; then
  if diff -q "$OUT_RS" "$WORK" >/dev/null 2>&1; then
    echo "ok: $OUT_RS matches Lean emit"
    exit 0
  else
    echo "ERROR: $OUT_RS differs from Lean emit. Run without --check to regenerate." >&2
    diff -u "$OUT_RS" "$WORK" || true
    exit 1
  fi
else
  cp "$WORK" "$OUT_RS"
  echo "ok: regenerated $OUT_RS"
fi
