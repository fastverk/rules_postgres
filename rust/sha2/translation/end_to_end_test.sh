#!/usr/bin/env bash
# End-to-end pipeline smoke test.
#
#   tensor (rules_lang)
#       │
#       ▼  c_cluster_diff.py            → C-side bindings
#       ▼  c_to_rust_bindings.py        → Rust-side bindings
#       ▼  translate-engine emit        → emitted Rust source
#       ▼  diff vs hand-written sibling
#
# Exit 0 if the emitted source byte-matches the hand-written one
# (after prettyplease normalization).

set -euo pipefail

REPO=/Volumes/Workspace/rules_lang
TENSOR="$REPO/bazel-bin/examples/postgres_smoke/postgres_full_tensor.tensor"
CANONICAL_RS="$REPO/translated/sha2/canonicals/pg_sha256_init.rs"
SIBLING_RS="$REPO/translated/sha2/canonicals/pg_sha384_init.rs"
OVERRIDES="$REPO/translated/sha2/translation/sha384_from_sha256.overrides.toml"
ENGINE="$REPO/translated/translator/target/debug/translate-engine"

WORK=$(mktemp -d)
trap "rm -rf $WORK" EXIT

echo "[1/4] tensor → C bindings" >&2
python3 "$REPO/c/cli/c_cluster_diff.py" \
    --tensor-dir "$TENSOR" \
    --canonical pg_sha256_init \
    --sibling pg_sha384_init \
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
# Reuse the engine to normalize the expected source the same way the
# emitter does — keeps whitespace-equivalence the gate, not raw bytes.
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
