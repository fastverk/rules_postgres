/*
 * sql_to_protobuf — CLI around libpg_query's pg_query_parse_protobuf().
 *
 * Usage: sql_to_protobuf <path-to-sql-file>
 *
 * Reads the file as a single SQL string, hands it to
 * `pg_query_parse_protobuf`, and writes the resulting `PgQueryProtobuf`
 * payload (raw bytes — the marshalled `pg_query.ParseResult`) to stdout.
 *
 * The output is exactly the wire-format bytes for the `ParseResult`
 * message defined in `@libpg_query//:pg_query.proto` — the canonical
 * AST format consumed by every downstream `sql_*_library` projection
 * (sql_json_library, sql_proto_library, sql_lean_library,
 * sql_catalog_library).
 *
 * Exit codes:
 *   0  — parse succeeded; protobuf bytes written to stdout
 *   1  — parse error reported by libpg_query (message on stderr)
 *   2  — argv error or file-read failure
 *
 * Companion to `parse_check` (which discards the parse tree) and
 * `plpgsql_to_json` (which handles the PL/pgSQL sub-grammar). Together
 * the three cover the public surface of libpg_query.
 */
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "pg_query.h"

static char *slurp(const char *path, size_t *out_len) {
    FILE *f = fopen(path, "rb");
    if (!f) {
        fprintf(stderr, "sql_to_protobuf: cannot open %s: %s\n", path, strerror(errno));
        return NULL;
    }
    if (fseek(f, 0, SEEK_END) != 0) { fclose(f); return NULL; }
    long sz = ftell(f);
    if (sz < 0) { fclose(f); return NULL; }
    if (fseek(f, 0, SEEK_SET) != 0) { fclose(f); return NULL; }
    char *buf = (char *)malloc((size_t)sz + 1);
    if (!buf) { fclose(f); return NULL; }
    size_t n = fread(buf, 1, (size_t)sz, f);
    fclose(f);
    buf[n] = '\0';
    *out_len = n;
    return buf;
}

int main(int argc, char **argv) {
    if (argc != 2) {
        fprintf(stderr, "usage: sql_to_protobuf <path-to-sql-file>\n");
        return 2;
    }
    size_t len = 0;
    char *sql = slurp(argv[1], &len);
    if (!sql) {
        return 2;
    }

    PgQueryProtobufParseResult result = pg_query_parse_protobuf(sql);
    int rc = 0;
    if (result.error) {
        fprintf(stderr, "sql_to_protobuf: %s at cursorpos %d in %s\n",
                result.error->message, result.error->cursorpos, argv[1]);
        rc = 1;
    } else {
        size_t written = fwrite(result.parse_tree.data, 1,
                                result.parse_tree.len, stdout);
        if (written != (size_t)result.parse_tree.len) {
            fprintf(stderr, "sql_to_protobuf: short write (%zu of %zu bytes)\n",
                    written, (size_t)result.parse_tree.len);
            rc = 2;
        }
    }
    pg_query_free_protobuf_parse_result(result);
    free(sql);
    pg_query_exit();
    return rc;
}
