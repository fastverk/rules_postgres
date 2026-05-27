#!/usr/bin/env bash
# check-int-unary-grounding.sh — AST structural diff for the int unary
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

WORK="$(mktemp -d -t pgir-int-unary-grounding.XXXXXX)"
trap 'rm -rf "$WORK"' EXIT

(
  cd "$LEAN_DIR"
  LEAN_PATH=. lean -o Pg/Ir/Emit/IntUnaryCommon.olean Pg/Ir/Emit/IntUnaryCommon.lean
  LEAN_PATH=. lean -o Pg/Ir/Emit/IntUnaryC.olean      Pg/Ir/Emit/IntUnaryC.lean
)

( cd "$LEAN_DIR" && LEAN_PATH=. lean --run Pg/Ir/Emit/IntUnaryC.lean ) > "$WORK/lean_emit.c"
clang -Xclang -ast-dump=json -fsyntax-only -I "$PG_INCLUDE" $EXTRA_INC "$WORK/lean_emit.c" \
  2>/dev/null > "$WORK/lean.ast.json" || true
if [ ! -s "$WORK/lean.ast.json" ]; then
  echo "ERROR: clang produced empty AST for Lean emit." >&2
  exit 2
fi

# int unary fns: int2um/abs + int4um/abs live in int.c; int8um/abs in int8.c.
for srcfile in int int8; do
  PG_SRC="$PG_INCLUDE/../backend/utils/adt/${srcfile}.c"
  clang -Xclang -ast-dump=json -fsyntax-only -I "$PG_INCLUDE" $EXTRA_INC "$PG_SRC" \
    2>/dev/null > "$WORK/real_${srcfile}.ast.json" || true
done

pass=0; fail=0; fails=""
check() {
  local src=$1 fn=$2
  if [ ! -f "$WORK/real_${src}.ast.json" ]; then return; fi
  if "$DIFF_BIN" --left "$WORK/real_${src}.ast.json" --right "$WORK/lean.ast.json" \
       --fn-name "$fn" >/dev/null 2>&1; then
    pass=$((pass+1))
  else
    fail=$((fail+1)); fails="$fails ${src}/${fn}"
  fi
}
check int int2um; check int int2abs
check int int4um; check int int4abs
check int8 int8um; check int8 int8abs

total=$((pass + fail))
if [ "$fail" -eq 0 ]; then
  echo "ok: $pass / $total int unary functions structurally equivalent to real Postgres."
  exit 0
else
  echo "ERROR: $fail / $total int unary functions structurally differ from real Postgres:" >&2
  echo "$fails" | tr ' ' '\n' | grep -v '^$' | sed 's/^/  - /' >&2
  exit 1
fi
