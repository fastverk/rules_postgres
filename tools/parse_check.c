/*
 * parse_check — minimal CLI around libpg_query's pg_query_parse.
 *
 * Usage: parse_check <path-to-sql-file>
 *
 * Reads the file as a single SQL string, hands it to pg_query_parse,
 * and exits 0 on parse success, 1 on parse failure (with the error
 * message + cursorpos printed to stderr). Used by the
 * sql_parse_valid_test rule to gate emitted SQL against postgres's
 * own parser — a syntactic CI check, not a semantic one.
 */
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "pg_query.h"

static char *slurp(const char *path, size_t *out_len) {
    FILE *f = fopen(path, "rb");
    if (!f) {
        fprintf(stderr, "parse_check: cannot open %s: %s\n", path, strerror(errno));
        return NULL;
    }
    if (fseek(f, 0, SEEK_END) != 0) {
        fclose(f);
        return NULL;
    }
    long sz = ftell(f);
    if (sz < 0) {
        fclose(f);
        return NULL;
    }
    if (fseek(f, 0, SEEK_SET) != 0) {
        fclose(f);
        return NULL;
    }
    char *buf = (char *)malloc((size_t)sz + 1);
    if (!buf) {
        fclose(f);
        return NULL;
    }
    size_t n = fread(buf, 1, (size_t)sz, f);
    fclose(f);
    buf[n] = '\0';
    *out_len = n;
    return buf;
}

int main(int argc, char **argv) {
    if (argc != 2) {
        fprintf(stderr, "usage: parse_check <path-to-sql-file>\n");
        return 2;
    }
    size_t len = 0;
    char *sql = slurp(argv[1], &len);
    if (!sql) {
        return 2;
    }

    PgQueryParseResult result = pg_query_parse(sql);
    int rc = 0;
    if (result.error) {
        fprintf(stderr, "parse_check: %s at cursorpos %d in %s\n",
                result.error->message, result.error->cursorpos, argv[1]);
        rc = 1;
    }
    pg_query_free_parse_result(result);
    free(sql);
    pg_query_exit();
    return rc;
}
