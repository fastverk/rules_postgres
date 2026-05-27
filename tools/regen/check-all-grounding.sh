#!/usr/bin/env bash
# check-all-grounding.sh — Gate 3: clang AST structural diff vs real PG.
#
# Runs every per-cluster `check-*-grounding.sh` script and reports any
# cluster whose Lean-emitted C body diverges structurally from the
# corresponding real Postgres source. Each per-cluster script:
#   1. Builds the cluster's `Pg.Ir.Emit.<Cluster>C` Lean spec,
#   2. Runs it to emit `lean_emit.c`,
#   3. `clang -ast-dump=json` on both Lean emit and the real PG file,
#   4. `c_ast_struct_diff` per function from the cluster's roster.
#
# Local-only by design: the per-cluster scripts resolve the postgres
# source tree from Bazel cache paths under
# `~/Library/Caches/bazel/.../external/+pg+postgres_src/src/include`,
# and require the `c_ast_struct_diff` binary built from
# `../rules_lang/translated/pipeline/`.
#
# Local invocation:
#   tools/regen/check-all-grounding.sh
#
# Requirements:
#   - `lean` on PATH (same as Gate 1).
#   - `clang` on PATH.
#   - Postgres source materialized in the Bazel cache. Trigger it via
#     any `bazel build` that pulls @postgres_src (e.g.
#     `bazel build //examples/meson_smoke/...`).
#   - `c_ast_struct_diff` binary built from sibling rules_lang repo:
#     `(cd ../rules_lang/translated/pipeline && cargo build --release)`.

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

shopt -s nullglob
scripts=( "$SCRIPT_DIR"/check-*-grounding.sh )
shopt -u nullglob

if [ ${#scripts[@]} -eq 0 ]; then
  echo "ERROR: no check-*-grounding.sh scripts found under $SCRIPT_DIR" >&2
  exit 2
fi

failed=()
skipped=()
for s in "${scripts[@]}"; do
  name="$(basename "$s")"
  printf '== %-45s ' "$name"
  log="$(mktemp -t "${name}.XXXXXX.log")"
  "$s" >"$log" 2>&1
  rc=$?
  case "$rc" in
    0)
      tail -1 "$log"
      ;;
    2)
      echo "SKIP (rc=2 — missing toolchain/cache)"
      sed 's/^/    /' "$log"
      skipped+=( "$name" )
      ;;
    *)
      echo "FAIL (rc=$rc)"
      sed 's/^/    /' "$log"
      failed+=( "$name" )
      ;;
  esac
  rm -f "$log"
done

echo
total=${#scripts[@]}
if [ ${#failed[@]} -eq 0 ] && [ ${#skipped[@]} -eq 0 ]; then
  echo "Gate 3: clang AST structural diff — $total / $total clusters match real PG."
  exit 0
fi

if [ ${#failed[@]} -eq 0 ]; then
  echo "Gate 3: clang AST structural diff — $(( total - ${#skipped[@]} )) / $total passed; ${#skipped[@]} skipped:"
  for s in "${skipped[@]}"; do echo "  - $s"; done
  # Skips are infra/cache issues, not divergence — exit 0 with a warning.
  exit 0
fi

echo "Gate 3: clang AST structural diff — ${#failed[@]} / $total cluster(s) failed:"
for f in "${failed[@]}"; do echo "  - $f"; done
if [ ${#skipped[@]} -gt 0 ]; then
  echo "Additionally skipped (toolchain/cache missing):"
  for s in "${skipped[@]}"; do echo "  - $s"; done
fi
exit 1
