#!/usr/bin/env bash
# check-tid-grounding.sh — verify Lean spec ≡ real Postgres C at
# the AST level for the tid cluster.
#
# Companion to regen-tid.sh (which handles Lean → Rust and Lean → C).
# This script structurally diffs the Lean-emitted C against the real
# Postgres source. A passing run proves the Lean spec captures the
# same algorithm Postgres wrote, not just its observable behavior.
#
# Pipeline:
#   Lean.Pg.Ir.Emit.TidC.main → /tmp/.../lean_emit.c
#   clang -ast-dump=json on the Lean emit AND the real PG source file
#   c_ast_struct_diff per function: real C ↔ Lean-emitted C

set -euo pipefail

LEAN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../lean" && pwd)"
DIFF_BIN="$(cd "$LEAN_DIR/../../rules_lang" && pwd)/translated/pipeline/target/release/c_ast_struct_diff"

# Locate postgres_src in the bazel cache.
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

WORK="$(mktemp -d -t pgir-tid-grounding.XXXXXX)"
trap 'rm -rf "$WORK"' EXIT

# Pre-compile Lean dependencies.
(
  cd "$LEAN_DIR"
  LEAN_PATH=. lean -o Pg/Ir/Types.olean             Pg/Ir/Types.lean
  LEAN_PATH=. lean -o Pg/Ir/Datum.olean             Pg/Ir/Datum.lean
  LEAN_PATH=. lean -o Pg/Ir/Emit/TidCommon.olean    Pg/Ir/Emit/TidCommon.lean
  LEAN_PATH=. lean -o Pg/Ir/Emit/TidC.olean         Pg/Ir/Emit/TidC.lean
)

# Emit Lean's C → tmp file, dump its AST.
LEAN_C="$WORK/lean_emit.c"
( cd "$LEAN_DIR" && LEAN_PATH=. lean --run Pg/Ir/Emit/TidC.lean ) > "$LEAN_C"
LEAN_AST="$WORK/lean_emit.json"
clang -Xclang -ast-dump=json -fsyntax-only -I "$PG_INCLUDE" $EXTRA_INC "$LEAN_C" > "$LEAN_AST" 2>/dev/null || true

# Locate the real postgres tid.c source.
PG_BACKEND=""
for candidate in \
    /Users/mattmarshall/Library/Caches/bazel/_bazel_mattmarshall/0fee58293a938d2caa5da4f904216d74/external/+postgres_src_extension+postgres_src/src/backend/utils/adt/tid.c \
    /Users/mattmarshall/Library/Caches/bazel/_bazel_mattmarshall/90476f1c3b5e144c1cb932d1d6a2c5b0/external/+pg+postgres_src/src/backend/utils/adt/tid.c; do
  if [ -f "$candidate" ]; then PG_BACKEND="$candidate"; break; fi
done
if [ -z "$PG_BACKEND" ]; then
  echo "ERROR: postgres tid.c not found in bazel cache." >&2
  exit 2
fi

PG_AST="$WORK/pg_real.json"
clang -Xclang -ast-dump=json -fsyntax-only -I "$PG_INCLUDE" $EXTRA_INC "$PG_BACKEND" > "$PG_AST" 2>/dev/null || true

# Run the structural diff for each function.
# Note: tid hash functions (hashtid, hashtidextended) call hash_any/hash_any_extended
# which are defined in src/common/hashfn.c, not tid.c. For v0, we verify only the
# comparison and clamping operators which are self-contained within tid.c.
# The hash operators are validated through differential tests in pg_tid's cargo tests.
FUNCTIONS=(
  "tideq"
  "tidne"
  "tidlt"
  "tidle"
  "tidgt"
  "tidge"
  "bttidcmp"
  "tidlarger"
  "tidsmaller"
)

# Include the hash functions too — they're trivially AST-comparable
# now that we use real Postgres headers.
FUNCTIONS+=("hashtid" "hashtidextended")

PASS=0
FAIL=0

for fn in "${FUNCTIONS[@]}"; do
  if "$DIFF_BIN" --left "$PG_AST" --right "$LEAN_AST" --fn-name "$fn" >/dev/null 2>&1; then
    PASS=$((PASS + 1))
  else
    FAIL=$((FAIL + 1))
    echo "MISMATCH: $fn" >&2
  fi
done

TOTAL=${#FUNCTIONS[@]}
if [ $FAIL -eq 0 ]; then
  echo "ok: $PASS / $TOTAL tid functions structurally equivalent to real Postgres."
  exit 0
else
  echo "FAILED: $PASS / $TOTAL functions matched; $FAIL diverged." >&2
  exit 1
fi
