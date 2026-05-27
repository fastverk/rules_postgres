#!/usr/bin/env bash
# End-to-end pipeline smoke test for the libpq buffer-grow cluster.
#
# Mirrors translated/sha2/translation/end_to_end_test.sh — same four
# stages, different cluster pair.
#
#   tensor (rules_lang)
#       │
#       ▼  c_cluster_diff.py            → C-side bindings
#       ▼  c_to_rust_bindings.py        → Rust-side bindings
#       ▼  translate-engine emit        → emitted Rust source
#       ▼  diff vs hand-written sibling
#
# Exit 0 if the emitted source byte-matches the hand-written sibling
# (after prettyplease normalization).

set -euo pipefail

REPO=/Volumes/Workspace/rules_lang
TENSOR="$REPO/bazel-bin/examples/postgres_smoke/postgres_full_tensor.tensor"
CANONICAL_RS="$REPO/translated/libpq_buf/canonicals/pqCheckOutBufferSpace.rs"
SIBLING_RS="$REPO/translated/libpq_buf/canonicals/pqCheckInBufferSpace.rs"
OVERRIDES="$REPO/translated/libpq_buf/translation/in_from_out.overrides.toml"
ENGINE="$REPO/translated/translator/target/debug/translate-engine"

WORK=$(mktemp -d)
trap "rm -rf $WORK" EXIT

echo "[1/4] tensor → C bindings" >&2
# c_cluster_diff.py on the libpq pair surfaces:
#   - the function-name ident binding
#   - one structural hole at the top-level CompoundStmt (the In version
#     has 3 extra stmts for the left-justify prelude)
# We tee the C bindings to stderr for visibility; the structural hole
# is an *expected* finding for this cluster, documented in the
# overrides file.
python3 "$REPO/c/cli/c_cluster_diff.py" \
    --tensor-dir "$TENSOR" \
    --canonical pqCheckOutBufferSpace \
    --sibling pqCheckInBufferSpace \
    > "$WORK/c_bindings.toml"

echo "[2/4] C bindings → Rust bindings (via naming policy + overrides)" >&2
python3 "$REPO/c/cli/c_to_rust_bindings.py" \
    --input "$WORK/c_bindings.toml" \
    --overrides "$OVERRIDES" \
    > "$WORK/rust_bindings.toml"

echo "[3/4] translate-engine emit" >&2
"$ENGINE" emit \
    --canonical "$CANONICAL_RS" \
    --sibling "$SIBLING_RS" \
    --bindings "$WORK/rust_bindings.toml" \
    --out "$WORK/emitted.rs"

echo "[4/4] compare to hand-written sibling (prettyplease-normalized)" >&2
"$ENGINE" emit \
    --canonical "$SIBLING_RS" \
    --sibling "$SIBLING_RS" \
    --bindings "$WORK/rust_bindings.toml" \
    --out "$WORK/expected.rs" 2>/dev/null || cp "$SIBLING_RS" "$WORK/expected.rs"

if diff -u "$WORK/expected.rs" "$WORK/emitted.rs"; then
    echo "PASS: emitted source matches hand-written sibling" >&2
else
    echo "FAIL: emitted source differs from hand-written sibling" >&2
    exit 1
fi
