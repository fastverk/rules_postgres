#!/usr/bin/env bash
# check-int-hash-grounding.sh — AST-level structural equivalence check
# for the hash V1 fmgr cluster.
#
# Sister to check-int-cmp-grounding.sh. Pipeline:
#   Lean.Pg.Ir.Emit.IntHashC.main → /tmp/.../lean_emit.c
#   clang -ast-dump=json on Lean emit AND real PG hashfunc.c
#   c_ast_struct_diff per function
#
# Exit 0 if all 6 functions structurally match; 1 otherwise.

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

WORK="$(mktemp -d -t pgir-hash-grounding.XXXXXX)"
trap 'rm -rf "$WORK"' EXIT

(
  cd "$LEAN_DIR"
  LEAN_PATH=. lean -o Pg/Ir/Types.olean         Pg/Ir/Types.lean
  LEAN_PATH=. lean -o Pg/Ir/Datum.olean         Pg/Ir/Datum.lean
  LEAN_PATH=. lean -o Pg/Ir/Cmp.olean           Pg/Ir/Cmp.lean
  LEAN_PATH=. lean -o Pg/Ir/Emit/Common.olean        Pg/Ir/Emit/Common.lean
  LEAN_PATH=. lean -o Pg/Ir/Emit/IntHashCommon.olean Pg/Ir/Emit/IntHashCommon.lean
  LEAN_PATH=. lean -o Pg/Ir/Emit/IntHash.olean       Pg/Ir/Emit/IntHash.lean
  LEAN_PATH=. lean -o Pg/Ir/Emit/IntHashC.olean      Pg/Ir/Emit/IntHashC.lean
)

( cd "$LEAN_DIR" && LEAN_PATH=. lean --run Pg/Ir/Emit/IntHashC.lean ) > "$WORK/lean_emit.c"
clang -Xclang -ast-dump=json -fsyntax-only -I "$PG_INCLUDE" $EXTRA_INC "$WORK/lean_emit.c" \
  2>/dev/null > "$WORK/lean.ast.json" || true
if [ ! -s "$WORK/lean.ast.json" ]; then
  echo "ERROR: clang produced empty AST for Lean emit." >&2
  exit 2
fi

PG_HASH="$PG_INCLUDE/../backend/access/hash/hashfunc.c"
clang -Xclang -ast-dump=json -fsyntax-only -I "$PG_INCLUDE" $EXTRA_INC "$PG_HASH" \
  2>/dev/null > "$WORK/real.ast.json" || true
if [ ! -s "$WORK/real.ast.json" ]; then
  echo "ERROR: clang produced empty AST for real hashfunc.c." >&2
  exit 2
fi

pass=0; fail=0; fails=""
for fn in hashchar hashint2 hashint4 hashint8 hashoid hashenum \
          hashcharextended hashint2extended hashint4extended \
          hashint8extended hashoidextended hashenumextended; do
  if "$DIFF_BIN" --left "$WORK/real.ast.json" --right "$WORK/lean.ast.json" \
       --fn-name "$fn" >/dev/null 2>&1; then
    pass=$((pass+1))
  else
    fail=$((fail+1)); fails="$fails $fn"
  fi
done

total=$((pass + fail))
if [ "$fail" -eq 0 ]; then
  echo "ok: $pass / $total hash functions structurally equivalent to real Postgres."
  exit 0
else
  echo "ERROR: $fail / $total hash functions structurally differ from real Postgres:" >&2
  echo "$fails" | tr ' ' '\n' | grep -v '^$' | sed 's/^/  - /' >&2
  exit 1
fi
