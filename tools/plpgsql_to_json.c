/* tools/libpg_query/plpgsql_to_json.c
 *
 * CLI wrapper around libpg_query's `pg_query_parse_plpgsql()`. Reads
 * a CREATE FUNCTION text (whose body is PL/pgSQL) from a file path
 * given on argv, parses the body via the PG-grounded PL/pgSQL parser
 * embedded in libpg_query, and writes the resulting JSON AST to
 * stdout.
 *
 * Used as the C-side leg of the proposed BodyStmt round-trip bridge:
 *
 *   Lean BodyStmt
 *     │ PgPretty.printStmt
 *     ▼
 *   CREATE FUNCTION text (a .sql file)
 *     │ plpgsql_to_json (this tool)
 *     ▼
 *   JSON PL/pgSQL AST
 *     │ Lean JSON reader → BodyStmt'
 *     ▼
 *   structural_eq BodyStmt BodyStmt'
 *
 * Exit codes:
 *   0  — parse succeeded; JSON written to stdout
 *   1  — argv error or file-read failure
 *   2  — parse error (libpg_query reported); error message on stderr
 */

#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>

#include "pg_query.h"

static char *read_whole_file(const char *path) {
    FILE *fp = fopen(path, "rb");
    if (!fp) {
        fprintf(stderr, "plpgsql_to_json: cannot open %s: %s\n",
                path, strerror(errno));
        return NULL;
    }
    if (fseek(fp, 0, SEEK_END) != 0) {
        fprintf(stderr, "plpgsql_to_json: fseek failed on %s\n", path);
        fclose(fp);
        return NULL;
    }
    long size = ftell(fp);
    if (size < 0) {
        fclose(fp);
        return NULL;
    }
    rewind(fp);
    char *buf = (char *) malloc((size_t) size + 1);
    if (!buf) {
        fclose(fp);
        return NULL;
    }
    size_t n = fread(buf, 1, (size_t) size, fp);
    fclose(fp);
    if (n != (size_t) size) {
        free(buf);
        return NULL;
    }
    buf[size] = '\0';
    return buf;
}

int main(int argc, char **argv) {
    if (argc != 2) {
        fprintf(stderr, "usage: %s <create-function.sql>\n", argv[0]);
        return 1;
    }

    char *src = read_whole_file(argv[1]);
    if (!src) return 1;

    PgQueryPlpgsqlParseResult r = pg_query_parse_plpgsql(src);
    free(src);

    if (r.error != NULL) {
        fprintf(stderr, "plpgsql parse error: %s\n", r.error->message);
        pg_query_free_plpgsql_parse_result(r);
        return 2;
    }

    /* JSON output verbatim from libpg_query. */
    fputs(r.plpgsql_funcs, stdout);
    fputc('\n', stdout);

    pg_query_free_plpgsql_parse_result(r);
    return 0;
}
