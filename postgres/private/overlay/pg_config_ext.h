/* tools/postgres_src/overlay/pg_config_ext.h
 *
 * Minimal hand-written replacement for the configure-generated
 * pg_config_ext.h. Contains the few macros configure sets that
 * are split out from pg_config.h proper.
 */
#ifndef PG_CONFIG_EXT_H
#define PG_CONFIG_EXT_H

/* The size in bytes of a typedef'ed PG_INT64_TYPE; assumed `long` on
 * 64-bit POSIX. */
#define PG_INT64_TYPE long

#endif /* PG_CONFIG_EXT_H */
