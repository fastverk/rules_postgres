#!/usr/bin/env bash
set -euo pipefail

LEAN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../lean" && pwd)"
DIFF_BIN="$(cd "$LEAN_DIR/../../rules_lang" && pwd)/translated/pipeline/target/release/c_ast_struct_diff"

PG_INCLUDE=""
for c in \
    /Users/mattmarshall/Library/Caches/bazel/_bazel_mattmarshall/0fee58293a938d2caa5da4f904216d74/external/+postgres_src_extension+postgres_src/src/include \
    /Users/mattmarshall/Library/Caches/bazel/_bazel_mattmarshall/90476f1c3b5e144c1cb932d1d6a2c5b0/external/+pg+postgres_src/src/include; do
  if [ -d "$c" ]; then PG_INCLUDE="$c"; break; fi
done
[ -n "$PG_INCLUDE" ] || { echo "ERROR: postgres_src not found." >&2; exit 2; }

LIBPG_INCLUDE=""
for c in \
    /Users/mattmarshall/Library/Caches/bazel/_bazel_mattmarshall/0fee58293a938d2caa5da4f904216d74/external/+libpg_query_extension+libpg_query/src/postgres/include \
    /Users/mattmarshall/Library/Caches/bazel/_bazel_mattmarshall/90476f1c3b5e144c1cb932d1d6a2c5b0/external/+pg+libpg_query/src/postgres/include; do
  if [ -d "$c" ]; then LIBPG_INCLUDE="$c"; break; fi
done
EXTRA_INC=""
[ -n "$LIBPG_INCLUDE" ] && EXTRA_INC="-I $LIBPG_INCLUDE"

[ -x "$DIFF_BIN" ] || { echo "ERROR: c_ast_struct_diff missing." >&2; exit 2; }

WORK="$(mktemp -d -t pgir-text-grounding.XXXXXX)"
trap 'rm -rf "$WORK"' EXIT

(
  cd "$LEAN_DIR"
  LEAN_PATH=. lean -o Pg/Ir/Emit/TextCommon.olean Pg/Ir/Emit/TextCommon.lean
  LEAN_PATH=. lean -o Pg/Ir/Emit/TextC.olean      Pg/Ir/Emit/TextC.lean
)

( cd "$LEAN_DIR" && LEAN_PATH=. lean --run Pg/Ir/Emit/TextC.lean ) > "$WORK/lean_emit.c"
clang -Xclang -ast-dump=json -fsyntax-only -I "$PG_INCLUDE" $EXTRA_INC "$WORK/lean_emit.c" \
  2>/dev/null > "$WORK/lean.ast.json" || true
[ -s "$WORK/lean.ast.json" ] || { echo "ERROR: empty Lean AST." >&2; exit 2; }

PG_SRC="$PG_INCLUDE/../backend/utils/adt/varlena.c"
clang -Xclang -ast-dump=json -fsyntax-only -I "$PG_INCLUDE" $EXTRA_INC "$PG_SRC" \
  2>/dev/null > "$WORK/real.ast.json" || true
[ -s "$WORK/real.ast.json" ] || { echo "ERROR: empty real varlena.c AST." >&2; exit 2; }

pass=0; fail=0; fails=""
# Check textcat only (text_catenate is static, not worth AST-diffing).
for fn in textcat; do
  if "$DIFF_BIN" --left "$WORK/real.ast.json" --right "$WORK/lean.ast.json" \
       --fn-name "$fn" >/dev/null 2>&1; then
    pass=$((pass+1))
  else
    fail=$((fail+1)); fails="$fails $fn"
  fi
done

total=$((pass + fail))
if [ "$fail" -eq 0 ]; then
  echo "ok: $pass / $total text functions structurally equivalent to real Postgres."
  exit 0
else
  echo "ERROR: $fail / $total text functions structurally differ from real Postgres:" >&2
  echo "$fails" | tr ' ' '\n' | grep -v '^$' | sed 's/^/  - /' >&2
  exit 1
fi
