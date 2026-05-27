#!/usr/bin/env bash
# check-all-behavioral.sh — Gate 2: cluster behavioral diff-test.
#
# For every emit-Rust cluster crate with a `tests/diff_*.rs` harness,
# runs `cargo test --release`. Each harness links the cluster's
# `c_oracle/` (vendored real-PG C body + setjmp wrapper) and the
# Lean-emitted Rust impl, drives both through pg_fcinfo, and asserts
# bit-identical Datums + matching FmgrErrorKind on a corpus of inputs
# (proptest + explicit boundary cases).
#
# Local invocation:
#   tools/regen/check-all-behavioral.sh
#
# Bazel invocation (opt-in, requires `cargo` on PATH):
#   bazel test //tools/regen:gate2_behavioral_diff
#
# Requirements:
#   - Rust toolchain on PATH (cargo + rustc).
#   - C compiler reachable by `cc` crate (clang on macOS, gcc/clang on
#     Linux). The c_oracle/ build is self-contained — no PG checkout
#     required (each crate ships its renamed PG C body + minimal
#     headers under c_oracle/).

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUST_DIR="$(cd "$SCRIPT_DIR/../../rust" && pwd)"

shopt -s nullglob
clusters=()
for d in "$RUST_DIR"/pg_*/; do
  cluster="$(basename "$d")"
  diff_tests=( "$d"/tests/diff_*.rs )
  if [ ${#diff_tests[@]} -gt 0 ]; then
    clusters+=( "$cluster" )
  fi
done
shopt -u nullglob

if [ ${#clusters[@]} -eq 0 ]; then
  echo "ERROR: no clusters with tests/diff_*.rs found under $RUST_DIR" >&2
  exit 2
fi

failed=()
for cluster in "${clusters[@]}"; do
  printf '== %-30s ' "$cluster"
  log="$(mktemp -t "${cluster}-cargo.XXXXXX.log")"
  if (cd "$RUST_DIR/$cluster" && cargo test --release --quiet) >"$log" 2>&1; then
    # Extract the "test result: ok. N passed" line.
    summary=$(grep -E "test result: " "$log" | tail -1)
    echo "${summary:-ok}"
  else
    echo "FAIL"
    sed 's/^/    /' "$log"
    failed+=( "$cluster" )
  fi
  rm -f "$log"
done

echo
total=${#clusters[@]}
if [ ${#failed[@]} -eq 0 ]; then
  echo "Gate 2: behavioral diff-test — $total / $total clusters pass."
  exit 0
fi

echo "Gate 2: behavioral diff-test — ${#failed[@]} / $total cluster(s) failed:"
for c in "${failed[@]}"; do echo "  - $c"; done
exit 1
