"""pg_sql_toolchain — registers libpg_query as the postgres SQL parser
for the generic `sql_*_library` rules in `@rules_lang//polyglot:sql.bzl`.

Usage in the consuming module's BUILD:

    load("@rules_postgres//postgres:sql_toolchain.bzl", "pg_sql_toolchain")

    pg_sql_toolchain(
        name = "pg17_libpg_query",
        version = "17-6.2.2",
    )

    toolchain(
        name = "pg_sql_toolchain",
        toolchain = ":pg17_libpg_query",
        toolchain_type = "@rules_lang//polyglot/sql:postgres_toolchain_type",
    )

(register the `toolchain` target via `register_toolchains(...)` in
MODULE.bazel or `--extra_toolchains` on the command line.)

Also exposes `pg_sql_catalog_library` — a thin wrapper around
`sql_catalog_library` that pre-fills `folder` with
`@rules_postgres//tools:pgpb_to_snapshot`, so consumers don't have to
hand-thread the postgres-specific catalog folder."""

load("@rules_lang//polyglot:sql.bzl", "SqlAstInfo", "SqlToolchainInfo", "sql_catalog_library")

def _pg_sql_toolchain_impl(ctx):
    return [
        platform_common.ToolchainInfo(
            sqltoolchaininfo = SqlToolchainInfo(
                parser = ctx.executable.parser,
                parser_format = "libpg_query_protobuf",
                proto_descriptor = ctx.file.proto_descriptor,
                version = ctx.attr.version,
                dialect = "postgres",
            ),
        ),
    ]

pg_sql_toolchain = rule(
    implementation = _pg_sql_toolchain_impl,
    attrs = {
        "parser": attr.label(
            executable = True,
            cfg = "exec",
            default = "@rules_postgres//tools:sql_to_protobuf",
            doc = "Parser binary — `.sql` arg, writes protobuf bytes to stdout.",
        ),
        "proto_descriptor": attr.label(
            allow_single_file = [".proto"],
            default = "@libpg_query//:protobuf/pg_query.proto",
            doc = "The `pg_query.proto` schema describing the parser's output.",
        ),
        "version": attr.string(
            mandatory = True,
            doc = "libpg_query release tag, e.g. '17-6.2.2'.",
        ),
    },
    doc = """Postgres dialect toolchain for the rules_lang SQL pipeline.

    Wraps `tools:sql_to_protobuf` (a `pg_query_parse_protobuf` CLI) and
    exposes its output format as `libpg_query_protobuf`. The same
    libpg_query release is used for the schema descriptor, so encoder
    and decoder versions are bound by Bazel resolution.""",
)

def pg_sql_catalog_library(name, deps, module_name = None, output_format = "lean", **kwargs):
    """Catalog projection wrapper that pre-fills the postgres folder.

    Identical to `sql_catalog_library` but binds `folder` to
    `@rules_postgres//tools/pgpb_to_snapshot:pgpb_to_snapshot` so
    consumers don't need to know the dialect-specific tool name.
    """
    sql_catalog_library(
        name = name,
        deps = deps,
        folder = "@rules_postgres//tools/pgpb_to_snapshot:pgpb_to_snapshot",
        module_name = module_name,
        output_format = output_format,
        **kwargs
    )

# ─── pg_sql_typed_library ──────────────────────────────────────────
#
# Produces a `Pg.Query.Top.TopParseResult` Lean module from a list
# of `sql_ast_library` deps, using `pgpb_to_lean_ast --typed`. The
# resulting `.lean` source can be fed into the kernel-checked Lean
# catalog fold (`Snapshot.ofTopParseResultAugmented`) — the
# Phase 0-7 path that's byte-equivalent to `pg_sql_catalog_library`'s
# C-tool output.
#
# Inputs are sorted by short_path for determinism (same convention
# `sql_catalog_library` uses), so the resulting `TopParseResult.stmts`
# list mirrors the C tool's stmt order exactly.

def _pg_sql_typed_library_impl(ctx):
    entries = []
    for d in ctx.attr.deps:
        entries.extend(d[SqlAstInfo].asts.to_list())
    entries = sorted(entries, key = lambda e: e.sql.short_path)
    ast_files = [e.ast for e in entries]

    out = ctx.actions.declare_file(ctx.attr.module_name + ".lean")
    args = ctx.actions.args()
    args.add("--typed")
    if ctx.attr.skip_other_bytes:
        args.add("--skip-other-bytes")
    args.add("--module", ctx.attr.module_name)
    args.add("--output", out.path)
    for ast in ast_files:
        args.add(ast.path)

    ctx.actions.run(
        inputs = ast_files,
        outputs = [out],
        executable = ctx.executable._tool,
        arguments = [args],
        mnemonic = "PgSqlTyped",
        progress_message = "pgpb_to_lean_ast --typed: %d AST(s) -> %s" %
                           (len(ast_files), out.short_path),
    )
    return [DefaultInfo(files = depset(direct = [out]))]

pg_sql_typed_library = rule(
    implementation = _pg_sql_typed_library_impl,
    attrs = {
        "deps": attr.label_list(
            providers = [SqlAstInfo],
            mandatory = True,
            doc = "`sql_ast_library` targets to combine.",
        ),
        "module_name": attr.string(
            mandatory = True,
            doc = "Lean module name AND output filename stem " +
                  "(no dots — Bazel target names can't have them).",
        ),
        "skip_other_bytes": attr.bool(
            default = False,
            doc = "Replace unrecognized stmts' opaque payload with " +
                  "`_root_.ByteArray.empty`. Saves a lot of Lean " +
                  "elaboration time on schemas heavy in PLpgSQL " +
                  "function bodies. The fold ignores `.other` Nodes " +
                  "entirely, so this never affects Snapshot semantics " +
                  "— only debug info that nothing currently consumes.",
        ),
        "_tool": attr.label(
            default = "@rules_postgres//tools/pgpb_to_lean_ast:pgpb_to_lean_ast",
            executable = True,
            cfg = "exec",
        ),
    },
    doc = """Run `pgpb_to_lean_ast --typed` over all `.pgpb` files in
    the dep closure, producing a single `<module_name>.lean` file
    with a `def parseResult : Pg.Query.Top.TopParseResult`.

    Downstream `lean_test` / `lean_emit` consumers depend on this
    rule and `import <module_name>` to pull in the parsed value.""",
)
