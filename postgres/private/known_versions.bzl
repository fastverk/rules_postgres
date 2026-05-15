"""SHA-256 pins for upstream PostgreSQL + libpg_query tarballs.

Bumping a version requires adding an entry here. Compute with:

    curl -fsSL <url> | shasum -a 256

Unpinned versions still build (warning emitted) but lose hermeticity.
Always prefer pinning.
"""

# libpg_query tags follow `<pg-major>-<libpg-version>` (e.g. `17-6.2.2`).
# See https://github.com/pganalyze/libpg_query/tags.
LIBPG_QUERY_VERSIONS = {
    "17-6.2.2": "e68962c18dbf5890821511be6c5c42261170bf8bfd51a82ea9176069f3d0df8b",
}

# PostgreSQL upstream tarballs from https://ftp.postgresql.org/pub/source/.
POSTGRES_SOURCE_VERSIONS = {
    "17.6": "e0630a3600aea27511715563259ec2111cd5f4353a4b040e0be827f94cd7a8b0",
}

# libpg_query fetching is delegated to
# @rules_github//github:repositories.bzl%github_source_repository — the
# URL template lives there as a fixed `https://github.com/<repo>/archive/
# refs/tags/<tag>.tar.gz` shape, so we just supply the repo + tag_format.

POSTGRES_SOURCE_URL_TEMPLATE = (
    "https://ftp.postgresql.org/pub/source/v{version}/postgresql-{version}.tar.bz2"
)
