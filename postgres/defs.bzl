"""User-facing rules for rules_postgres.

`pg_parse_valid_test` wraps the `parse_check` C binary as a `sh_test`
that gates a `.sql` file against PostgreSQL's own parser (via
libpg_query). The test passes iff parse_check exits 0 (clean parse),
fails with the parser's error + cursor position on stderr otherwise.

Use this to keep emitted-SQL or hand-written-DDL in sync with what
PostgreSQL accepts — catches typos, missing keywords, version drift in
features like INCLUDE columns or generated-as-identity.
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
