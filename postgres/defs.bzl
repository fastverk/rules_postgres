"""User-facing rules for rules_postgres.

* `pg_parse_valid_test` wraps the `parse_check` C binary as a `sh_test`
  that gates a `.sql` file against PostgreSQL's own parser (via
  libpg_query). Passes iff parse_check exits 0, fails with the parser's
  error + cursor position on stderr otherwise. Use this to keep
  emitted-SQL or hand-written-DDL in sync with what PostgreSQL accepts.

* `pg_parse_tree` runs the `sql_to_protobuf` C binary on a `.sql` file
  and captures the marshalled `pg_query.ParseResult` protobuf bytes as
  a `.pgpb` artifact. This is the single-file convenience macro;
  multi-file pipelines should use `sql_library` + `sql_ast_library` from
  `@rules_lang//polyglot:sql.bzl` instead.
"""

load("@rules_shell//shell:sh_test.bzl", _sh_test = "sh_test")

def pg_parse_valid_test(name, sql, **kwargs):
    """Assert that a SQL file parses cleanly under PostgreSQL's parser.

    Args:
      name: test target name.
      sql: label of the .sql file to validate (source, genrule output,
        or filegroup member — anything with a single-file location).
      **kwargs: forwarded to the underlying sh_test (e.g. `tags`,
        `size`, `timeout`).
    """
    _sh_test(
        name = name,
        srcs = ["@rules_postgres//tools:run_parse_check.sh"],
        data = [
            "@rules_postgres//tools:parse_check",
            sql,
        ],
        args = [
            "$(location @rules_postgres//tools:parse_check)",
            "$(location " + sql + ")",
        ],
        **kwargs
    )

def pg_parse_tree(name, sql, out = None, **kwargs):
    """Run libpg_query over a `.sql` file, capture the protobuf AST.

    Single-file convenience around `@rules_postgres//tools:sql_to_protobuf`.
    For multi-file pipelines, prefer `sql_library` + `sql_ast_library`
    from `@rules_lang//polyglot:sql.bzl`, which use the same C tool via
    `pg_sql_toolchain` and propagate `SqlAstInfo` so downstream
    projections (json, lean, catalog) compose cleanly.

    Args:
      name: genrule target name.
      sql: label of the .sql file to parse.
      out: output filename. Defaults to `name + ".pgpb"`.
      **kwargs: forwarded to the underlying `genrule`.

    Returns:
      A `.pgpb` file whose bytes are exactly the marshalled
      `pg_query.ParseResult` (see `@libpg_query//:pg_query.proto`).
    """
    if out == None:
        out = name + ".pgpb"
    native.genrule(
        name = name,
        srcs = [sql],
        outs = [out],
        tools = ["@rules_postgres//tools:sql_to_protobuf"],
        cmd = "$(location @rules_postgres//tools:sql_to_protobuf) $< > $@",
        **kwargs
    )
