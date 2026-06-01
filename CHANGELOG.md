# Changelog

All notable changes to rules_postgres. The format is loosely
[Keep a Changelog](https://keepachangelog.com/) — version headers
mirror the published bazel-registry entries.

## 0.4.3 — Pg.Catalog.PgProc: + proargnames + proretset + proargmodes

Extends `lean/Pg/Catalog/Tables.lean`'s `PgProc` structure with
three new fields + introduces an `ArgMode` enum:

- `proargnames : List String  := []` — per-argument source name.
  Empty means "fall back to positional `arg0` / `arg1` / …".
- `proretset   : Bool         := false` — set-returning flag
  (`SETOF X`, `RETURNS TABLE(...)`).
- `proargmodes : List ArgMode := []` — IN / OUT / INOUT /
  VARIADIC / TABLE_OUT per argument. Empty means "all IN".

`ArgMode` mirrors postgres's single-char encoding via a closed
inductive (`.in_` / `.out` / `.inout` / `.variadic` / `.tableOut`)
plus a `.toChar` projection for round-tripping with on-disk
representations.

All three new fields default to safe sentinels so existing
`PgProc` literal sites (notably `Pg.Catalog.Generated`'s 3314
procs) compile unchanged. Consumers wanting the precision
populate them explicitly.

Motivated by Aion's V0 codegen Slice 1 (catalog → FunctionSpec
derivation, replacing the hand-coded FunctionSpec list with one
mechanically built from a `Snapshot`'s `procs`). The new fields
are the data the deriver needs:

  * `proargnames` — for naming the TS wrapper's input fields
                    (`p_username` rather than `arg0`).
  * `proretset`   — drives `FunctionSpec.isSetOf`.
  * `proargmodes` — separates IN args (input schema) from OUT
                    args (composite return) and handles INOUT
                    (input AND output) properly.

The extension is additive — no breaking changes — so the
upgrade is a one-line bazel_dep bump.

## 0.4.2 — Pg.Catalog.PgType: + typbasetype + typelem

Extends `lean/Pg/Catalog/Tables.lean`'s `PgType` structure with two
new fields:

- `typbasetype : Oid .type` — for DOMAIN rows, the underlying type
  the domain wraps. `Oid.invalid .type` for non-domain rows.
- `typelem : Oid .type` — for ARRAY rows, the element type. Postgres
  encodes arrays as `typtype = .base` plus `typcategory = 'A'` plus
  a non-invalid `typelem`. `Oid.invalid .type` for non-array rows.

Both default to `Oid.invalid .type` so existing `PgType` literal
sites (in particular `Pg.Catalog.Generated.bootstrapSnapshot`'s 185
type entries) compile unchanged; consumers wanting the precision
populate the fields explicitly.

Motivated by Aion's V0 codegen Slice C, which needs to emit
`z.array(elementSchema)` for array types and walk domains down to
their underlying primitive. The extension is additive — no breaking
changes — so the upgrade is a one-line bazel_dep bump for consumers.

## 0.2.0 — delegate libpg_query fetch to rules_github

- Replace the in-tree libpg_query download logic with a dependency on
  `rules_github`'s `github_source_repository` so the parser source is
  pulled via the shared substrate.
- Update the install snippet to point at the `fastverk/bazel-registry`.

## 0.1.0 — initial release

- First cut of Bazel rules for PostgreSQL tooling: `pg.query` (fetches
  and builds [libpg_query](https://github.com/pganalyze/libpg_query) as
  a `cc_library`), experimental `pg.source` (full PG tarball with a
  minimal BUILD overlay), `pg_parse_valid_test` (sh_test wrapper that
  asserts a `.sql` parses cleanly), and the `parse_check` +
  `plpgsql_to_json` C tools.
