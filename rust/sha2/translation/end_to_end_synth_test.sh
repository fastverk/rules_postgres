#!/usr/bin/env bash
# End-to-end pipeline smoke test — *synthesized sibling* path.
#
# Identical to end_to_end_test.sh except step 3a synthesizes the sibling
# Rust source from (canonical, Rust bindings, overrides). The sibling is
# no longer a hand-written input.
#
#   tensor (rules_lang)
#       │
#       ▼  c_cluster_diff.py            → C-side bindings
#       ▼  c_to_rust_bindings.py        → Rust-side bindings
#       ▼  translate-engine synthesize  → synthesized sibling.rs
#       ▼  translate-engine emit        → emitted Rust source
#       ▼  diff vs hand-written sibling
#
# Per-cluster human input set:
#   - canonical Rust source       (1 file)
#   - per-cluster overrides TOML  (1 file, ~10 lines)
# That's it. The hand-written sibling Rust file is no longer required.
#
# Exit 0 if the emitted source byte-matches the hand-written sibling
# (after prettyplease normalization).

set -euo pipefail

REPO=/Volumes/Workspace/rules_lang
TENSOR="$REPO/bazel-bin/examples/postgres_smoke/postgres_full_tensor.tensor"
CANONICAL_RS="$REPO/translated/sha2/canonicals/pg_sha256_init.rs"
HAND_SIBLING_RS="$REPO/translated/sha2/canonicals/pg_sha384_init.rs"   # only used as the regression oracle
OVERRIDES="$REPO/translated/sha2/translation/sha384_from_sha256.overrides.toml"
ENGINE="$REPO/translated/translator/target/debug/translate-engine"

if [[ ! -x "$ENGINE" ]]; then
    echo "[build] translate-engine not built yet — running cargo build" >&2
    (cd "$REPO/translated/translator" && cargo build --quiet)
fi

WORK=$(mktemp -d)
trap "rm -rf $WORK" EXIT

echo "[1/5] tensor → C bindings" >&2
python3 "$REPO/c/cli/c_cluster_diff.py" \
    --tensor-dir "$TENSOR" \
    --canonical pg_sha256_init \
    --sibling pg_sha384_init \
    > "$WORK/c_bindings.toml"

echo "[2/5] C bindings → Rust bindings (via naming policy + overrides)" >&2
python3 "$REPO/c/cli/c_to_rust_bindings.py" \
    --input "$WORK/c_bindings.toml" \
    --overrides "$OVERRIDES" \
    > "$WORK/rust_bindings.toml"

echo "[3/5] translate-engine synthesize — derive sibling from canonical + bindings" >&2
"$ENGINE" synthesize \
    --canonical "$CANONICAL_RS" \
    --bindings  "$WORK/rust_bindings.toml" \
    --overrides "$OVERRIDES" \
    --c-bindings "$WORK/c_bindings.toml" \
    --out "$WORK/synthesized_sibling.rs"

echo "    synthesized sibling at $WORK/synthesized_sibling.rs:" >&2
sed 's/^/      /' "$WORK/synthesized_sibling.rs" >&2

echo "[4/5] translate-engine emit — substitute bindings into canonical via synthesized sibling" >&2
"$ENGINE" emit \
    --canonical "$CANONICAL_RS" \
    --sibling "$WORK/synthesized_sibling.rs" \
    --bindings "$WORK/rust_bindings.toml" \
    --out "$WORK/emitted.rs"

echo "[5/5] compare emitted source to hand-written sibling (prettyplease-normalized)" >&2
# Normalize the hand-written sibling through the engine's own prettyplease pass
# so whitespace doesn't show up as a diff.
"$ENGINE" emit \
    --canonical "$HAND_SIBLING_RS" \
    --sibling "$HAND_SIBLING_RS" \
    --bindings "$WORK/rust_bindings.toml" \
    --out "$WORK/expected.rs" 2>/dev/null || cp "$HAND_SIBLING_RS" "$WORK/expected.rs"

if diff -u "$WORK/expected.rs" "$WORK/emitted.rs"; then
    echo "PASS: emitted source (via synthesized sibling) matches hand-written sibling" >&2
else
    echo "FAIL: emitted source differs from hand-written sibling" >&2
    exit 1
fi
