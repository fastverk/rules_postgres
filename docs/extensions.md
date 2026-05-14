<!-- Generated with Stardoc: http://skydoc.bazel.build -->

Module extension for rules_postgres.

Exposes two tag classes:

  pg.query(version = ...)  — fetches libpg_query and builds it as a
                             `cc_library`. Creates @libpg_query.

  pg.source(version = ...) — fetches the full PostgreSQL source tarball
                             and lays a minimal BUILD overlay on top
                             (filegroups for source dirs + a probe
                             `pg_common_string` cc_library). Creates
                             @postgres_src. Experimental — gives Bazel
                             access to raw PG source for tooling that
                             needs to compile pieces of the backend.

The two paths are independent. Most consumers want only `pg.query` for
SQL parse-validation gates; `pg.source` is for advanced tooling that
needs the full PG codebase under Bazel.

Default usage:

    pg = use_extension("@rules_postgres//postgres:extensions.bzl", "pg")
    pg.query(version = "17-6.2.2")
    use_repo(pg, "libpg_query")

With full PG source as well:

    pg.source(version = "17.6")
    use_repo(pg, "libpg_query", "postgres_src")

<a id="pg"></a>

## pg

<pre>
pg = use_extension("@rules_postgres//postgres:extensions.bzl", "pg")
pg.query(<a href="#pg.query-version">version</a>)
pg.source(<a href="#pg.source-version">version</a>)
</pre>

Module extension fetching libpg_query and/or the full PostgreSQL source tree.


**TAG CLASSES**

<a id="pg.query"></a>

### query

Pull libpg_query as @libpg_query.

**Attributes**

| Name  | Description | Type | Mandatory | Default |
| :------------- | :------------- | :------------- | :------------- | :------------- |
| <a id="pg.query-version"></a>version |  libpg_query release tag (e.g. "17-6.2.2").   | String | required |  |

<a id="pg.source"></a>

### source

Pull the PostgreSQL source tarball as @postgres_src.

**Attributes**

| Name  | Description | Type | Mandatory | Default |
| :------------- | :------------- | :------------- | :------------- | :------------- |
| <a id="pg.source-version"></a>version |  PostgreSQL release version (e.g. "17.6").   | String | required |  |


