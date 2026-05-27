#!/usr/bin/env bash
# check-float-to-int-grounding.sh — AST structural diff for the float-to-int
# cluster against real Postgres source.

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

WORK="$(mktemp -d -t pgir-float-to-int-grounding.XXXXXX)"
trap 'rm -rf "$WORK"' EXIT

(
  cd "$LEAN_DIR"
  LEAN_PATH=. lean -o Pg/Ir/Emit/FloatToIntCommon.olean Pg/Ir/Emit/FloatToIntCommon.lean
  LEAN_PATH=. lean -o Pg/Ir/Emit/FloatToIntC.olean      Pg/Ir/Emit/FloatToIntC.lean
)

( cd "$LEAN_DIR" && LEAN_PATH=. lean --run Pg/Ir/Emit/FloatToIntC.lean ) > "$WORK/lean_emit.c"
clang -Xclang -ast-dump=json -fsyntax-only -I "$PG_INCLUDE" $EXTRA_INC "$WORK/lean_emit.c" \
  2>/dev/null > "$WORK/lean.ast.json" || true
if [ ! -s "$WORK/lean.ast.json" ]; then
  echo "ERROR: clang produced empty AST for Lean emit." >&2
  exit 2
fi

# float-to-int fns: all four (ftoi2, ftoi4, dtoi2, dtoi4) live in float.c.
PG_SRC="$PG_INCLUDE/../backend/utils/adt/float.c"
clang -Xclang -ast-dump=json -fsyntax-only -I "$PG_INCLUDE" $EXTRA_INC "$PG_SRC" \
  2>/dev/null > "$WORK/real_float.ast.json" || true

pass=0; fail=0; fails=""
check() {
  local fn=$1
  if [ ! -f "$WORK/real_float.ast.json" ]; then return; fi
  if "$DIFF_BIN" --left "$WORK/real_float.ast.json" --right "$WORK/lean.ast.json" \
       --fn-name "$fn" >/dev/null 2>&1; then
    pass=$((pass+1))
  else
    fail=$((fail+1)); fails="$fails float/${fn}"
  fi
}
check ftoi2;  check ftoi4;  check dtoi2;  check dtoi4

total=$((pass + fail))
if [ "$fail" -eq 0 ]; then
  echo "ok: $pass / $total float-to-int cast functions structurally equivalent to real Postgres."
  exit 0
else
  echo "ERROR: $fail / $total float-to-int cast functions structurally differ from real Postgres:" >&2
  echo "$fails" | tr ' ' '\n' | grep -v '^$' | sed 's/^/  - /' >&2
  exit 1
fi
