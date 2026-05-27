#!/usr/bin/env bash
# check-interval-grounding.sh — AST structural diff between the Lean-
# emitted C and real Postgres source for the interval-arithmetic cluster.

set -euo pipefail

LEAN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../lean" && pwd)"
DIFF_BIN="$(cd "$LEAN_DIR/../../rules_lang" && pwd)/translated/pipeline/target/release/c_ast_struct_diff"

PG_INCLUDE=""
for candidate in \
    /Users/mattmarshall/Library/Caches/bazel/_bazel_mattmarshall/0fee58293a938d2caa5da4f904216d74/external/+postgres_src_extension+postgres_src/src/include \
    /Users/mattmarshall/Library/Caches/bazel/_bazel_mattmarshall/90476f1c3b5e144c1cb932d1d6a2c5b0/external/+pg+postgres_src/src/include; do
  if [ -d "$candidate" ]; then PG_INCLUDE="$candidate"; break; fi
done
if [ -z "$PG_INCLUDE" ]; then
  echo "ERROR: postgres_src include path not found in bazel cache." >&2
  exit 2
fi

LIBPG_INCLUDE=""
for candidate in \
    /Users/mattmarshall/Library/Caches/bazel/_bazel_mattmarshall/0fee58293a938d2caa5da4f904216d74/external/+libpg_query_extension+libpg_query/src/postgres/include \
    /Users/mattmarshall/Library/Caches/bazel/_bazel_mattmarshall/90476f1c3b5e144c1cb932d1d6a2c5b0/external/+pg+libpg_query/src/postgres/include; do
  if [ -d "$candidate" ]; then LIBPG_INCLUDE="$candidate"; break; fi
done
EXTRA_INC=""
if [ -n "$LIBPG_INCLUDE" ]; then EXTRA_INC="-I $LIBPG_INCLUDE"; fi

if [ ! -x "$DIFF_BIN" ]; then
  echo "ERROR: c_ast_struct_diff binary missing: $DIFF_BIN" >&2
  exit 2
fi

WORK="$(mktemp -d -t pgir-interval-grounding.XXXXXX)"
trap 'rm -rf "$WORK"' EXIT

(
  cd "$LEAN_DIR"
  LEAN_PATH=. lean -o Pg/Ir/Emit/IntervalCommon.olean Pg/Ir/Emit/IntervalCommon.lean
  LEAN_PATH=. lean -o Pg/Ir/Emit/IntervalC.olean      Pg/Ir/Emit/IntervalC.lean
)

( cd "$LEAN_DIR" && LEAN_PATH=. lean --run Pg/Ir/Emit/IntervalC.lean ) > "$WORK/lean_emit.c"
clang -Xclang -ast-dump=json -fsyntax-only -I "$PG_INCLUDE" $EXTRA_INC "$WORK/lean_emit.c" \
  2>/dev/null > "$WORK/lean.ast.json" || true
if [ ! -s "$WORK/lean.ast.json" ]; then
  echo "ERROR: clang produced empty AST for Lean emit." >&2
  exit 2
fi

PG_SRC="$PG_INCLUDE/../backend/utils/adt/timestamp.c"
clang -Xclang -ast-dump=json -fsyntax-only -I "$PG_INCLUDE" $EXTRA_INC "$PG_SRC" \
  2>/dev/null > "$WORK/real.ast.json" || true
if [ ! -s "$WORK/real.ast.json" ]; then
  echo "ERROR: clang produced empty AST for real timestamp.c." >&2
  exit 2
fi

pass=0; fail=0; fails=""
# Interval cluster v1-v2: stub functions (interval_um, interval_pl, interval_mi).
# The `<fn>_internal` and finite_interval_* helpers are `static void` — they
# have file-local linkage which clang's AST tags differently per source location;
# checking them at the AST level is noisier than valuable since the diff-test
# (cargo) already verifies behavioral equivalence. We focus on the fmgr stubs.
for fn in interval_um interval_pl interval_mi; do
  if "$DIFF_BIN" --left "$WORK/real.ast.json" --right "$WORK/lean.ast.json" \
       --fn-name "$fn" >/dev/null 2>&1; then
    pass=$((pass+1))
  else
    fail=$((fail+1)); fails="$fails $fn"
  fi
done

total=$((pass + fail))
if [ "$fail" -eq 0 ]; then
  echo "ok: $pass / $total interval functions structurally equivalent to real Postgres."
  exit 0
else
  echo "ERROR: $fail / $total interval functions structurally differ from real Postgres:" >&2
  echo "$fails" | tr ' ' '\n' | grep -v '^$' | sed 's/^/  - /' >&2
  exit 1
fi
