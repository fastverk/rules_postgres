<!-- Generated with Stardoc: http://skydoc.bazel.build -->

User-facing rules for rules_postgres.

`pg_parse_valid_test` wraps the `parse_check` C binary as a `sh_test`
that gates a `.sql` file against PostgreSQL's own parser (via
libpg_query). The test passes iff parse_check exits 0 (clean parse),
fails with the parser's error + cursor position on stderr otherwise.

Use this to keep emitted-SQL or hand-written-DDL in sync with what
PostgreSQL accepts — catches typos, missing keywords, version drift in
features like INCLUDE columns or generated-as-identity.

<a id="pg_parse_valid_test"></a>

## pg_parse_valid_test

<pre>
load("@rules_postgres//postgres:defs.bzl", "pg_parse_valid_test")

pg_parse_valid_test(<a href="#pg_parse_valid_test-name">name</a>, <a href="#pg_parse_valid_test-sql">sql</a>, <a href="#pg_parse_valid_test-kwargs">**kwargs</a>)
</pre>

Assert that a SQL file parses cleanly under PostgreSQL's parser.

**PARAMETERS**


| Name  | Description | Default Value |
| :------------- | :------------- | :------------- |
| <a id="pg_parse_valid_test-name"></a>name |  test target name.   |  none |
| <a id="pg_parse_valid_test-sql"></a>sql |  label of the .sql file to validate (source, genrule output, or filegroup member — anything with a single-file location).   |  none |
| <a id="pg_parse_valid_test-kwargs"></a>kwargs |  forwarded to the underlying sh_test (e.g. `tags`, `size`, `timeout`).   |  none |


