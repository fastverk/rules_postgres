#!/usr/bin/env bash
# regen-int-hash.sh — regen the Rust + C-oracle emits for the int hash
# V1 fmgr cluster from the Lean spec at
# `rules_postgres/lean/Pg/Ir/Emit/IntHash{,Common,C}.lean`.
#
# Run when:
#   - Pg/Ir/Emit/IntHashCommon.lean changes (the family table)
#   - Pg/Ir/Emit/IntHash.lean changes (the Rust codegen template)
#   - Pg/Ir/Emit/IntHashC.lean changes (the C codegen template — Gate 3)
#
# CI gates the committed files via Bazel-native Gate 1
# (//lean:gate1_int4_hash{,_c}), which compares each lean_emit's
# stdout to the file written here.

set -euo pipefail

LEAN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../lean" && pwd)"
REPO_ROOT="$(cd "$LEAN_DIR/.." && pwd)"
RUST_TARGET="$REPO_ROOT/rust/pg_int4_hash/src/lib.rs"
# Real-PG-headers C emit; Gate 3 (clang AST struct diff) compares this
# against `@postgres_src//:src/backend/access/hash/hashfunc.c`.
C_TARGET="$REPO_ROOT/rust/pg_int4_hash/c_oracle/int4_hash_emit.c"

(
  cd "$LEAN_DIR"
  LEAN_PATH=. lean -o Pg/Ir/Types.olean                Pg/Ir/Types.lean
  LEAN_PATH=. lean -o Pg/Ir/Datum.olean                Pg/Ir/Datum.lean
  LEAN_PATH=. lean -o Pg/Ir/Cmp.olean                  Pg/Ir/Cmp.lean
  LEAN_PATH=. lean -o Pg/Ir/Emit/Common.olean          Pg/Ir/Emit/Common.lean
  LEAN_PATH=. lean -o Pg/Ir/Emit/IntHashCommon.olean   Pg/Ir/Emit/IntHashCommon.lean
  LEAN_PATH=. lean -o Pg/Ir/Emit/IntHash.olean         Pg/Ir/Emit/IntHash.lean
  LEAN_PATH=. lean -o Pg/Ir/Emit/IntHashC.olean        Pg/Ir/Emit/IntHashC.lean
)

TMP_RUST="$(mktemp -t int_hash_rust.rs.XXXXXX)"
TMP_C="$(mktemp -t int_hash_c.c.XXXXXX)"
trap 'rm -f "$TMP_RUST" "$TMP_C"' EXIT

( cd "$LEAN_DIR" && LEAN_PATH=. lean --run Pg/Ir/Emit/IntHash.lean )  > "$TMP_RUST"
( cd "$LEAN_DIR" && LEAN_PATH=. lean --run Pg/Ir/Emit/IntHashC.lean ) > "$TMP_C"

if [[ "${1:-}" == "--check" ]]; then
  if ! diff -u "$RUST_TARGET" "$TMP_RUST"; then
    echo "ERROR: $RUST_TARGET is stale relative to the Lean source." >&2
    echo "       Run rules_postgres/tools/regen/regen-int-hash.sh to update it." >&2
    exit 1
  fi
  if ! diff -u "$C_TARGET" "$TMP_C"; then
    echo "ERROR: $C_TARGET is stale relative to the Lean source." >&2
    echo "       Run rules_postgres/tools/regen/regen-int-hash.sh to update it." >&2
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
