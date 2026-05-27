"""Bazel-native Pg.Ir cluster gate wiring.

Generates per-cluster `lean_emit` + `diff_test` pairs that together
implement Gate 1 (regen idempotence) without shelling out to host
`lean`. Each cluster gets:

  - <name>_rs_emit       lean_emit producing the Rust emit to stdout
  - gate1_<name>         diff_test asserting the committed src/lib.rs
                         matches the lean_emit output
  - <name>_c_emit        (clusters with a C oracle) lean_emit for the
                         real-PG-headers form
  - gate1_<name>_c       (clusters with a C oracle) diff_test for the
                         committed c_oracle/<cluster>_emit.c

Important: this macro MUST be invoked from `lean/BUILD.bazel`. The
underlying `lean_emit` rule strips the calling package prefix from
each src's short_path, so the srcs must live in (or under) the same
package as the rule. The lean/ tree is where they live.
"""

load("@bazel_skylib//rules:diff_test.bzl", "diff_test")
load("@rules_lean//lean:lean.bzl", "lean_emit")

# Standard Pg.Ir prelude — the modules every Lean emit imports
# transitively before its cluster-specific files. Matches the
# `lean -o Pg/Ir/Types.olean ...; lean -o Pg/Ir/Datum.olean ...` pairs
# at the top of every regen-*.sh.
PG_IR_PRELUDE = [
    "Pg/Ir/Types.lean",
    "Pg/Ir/Datum.lean",
]

# Older clusters (everything except macaddr/tid/uuid) also import
# Pg/Ir/Cmp.lean + Pg/Ir/Emit/Common.lean before their cluster modules.
PG_IR_OLD_BASE = PG_IR_PRELUDE + [
    "Pg/Ir/Cmp.lean",
    "Pg/Ir/Emit/Common.lean",
]

def gate1_cluster(
        name,
        rust_entry,
        rust_target,
        srcs,
        c_entry = None,
        c_target = None,
        c_extra_srcs = None):
    """Wire Gate 1 (regen idempotence) for one Pg.Ir cluster.

    Args:
      name: stem for generated targets (e.g., `int_arith`).
      rust_entry: package-relative path of the Rust-emitting Lean main
        (e.g., `Pg/Ir/Emit/IntArith.lean`).
      rust_target: Bazel label of the committed Rust emit
        (e.g., `//rust/pg_int4_arith:lib_rs`).
      srcs: ordered list of Lean source paths (relative to `lean/`)
        compiled by the Rust-emitting lean_emit. Order matters —
        lean_emit compiles them sequentially. Must include the entry.
      c_entry: optional package-relative path of the C-emitting Lean
        main (e.g., `Pg/Ir/Emit/UuidC.lean`). Only set for clusters
        that also emit a real-PG-headers C file.
      c_target: optional Bazel label of the committed C emit
        (e.g., `//rust/pg_uuid:uuid_emit_c`). Required when c_entry is set.
      c_extra_srcs: optional additional Lean srcs needed by the C
        emit beyond `srcs` (typically just `[<Cluster>C.lean]`).
    """
    rust_emit = name + "_rs_emit"

    lean_emit(
        name = rust_emit,
        srcs = srcs,
        entry = rust_entry,
        out = rust_emit + ".rs",
    )

    diff_test(
        name = "gate1_" + name,
        file1 = ":" + rust_emit,
        file2 = rust_target,
    )

    if c_entry:
        if not c_target:
            fail("c_target is required when c_entry is set")
        c_emit = name + "_c_emit"
        c_srcs = list(srcs) + (list(c_extra_srcs) if c_extra_srcs else [])

        lean_emit(
            name = c_emit,
            srcs = c_srcs,
            entry = c_entry,
            out = c_emit + ".c",
        )

        diff_test(
            name = "gate1_" + name + "_c",
            file1 = ":" + c_emit,
            file2 = c_target,
        )
