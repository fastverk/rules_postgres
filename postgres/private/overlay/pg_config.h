/* tools/postgres_src/overlay/pg_config.h
 *
 * Minimal hand-written replacement for the autoconf-generated
 * pg_config.h. Contains only the macros needed to compile a small
 * subset of PG (currently the feasibility probe `src/common/string.c`).
 *
 * Values are appropriate for a 64-bit POSIX system (macOS arm64 or
 * Linux x86_64). Not portable to Windows, 32-bit systems, or older
 * macOS versions.
 *
 * As we surface more compile errors, more macros land here. This is
 * intentionally not a complete reflection of what `configure` would
 * produce — it's a working subset, kept minimal so the experiment
 * stays inspectable.
 */
#ifndef PG_CONFIG_H
#define PG_CONFIG_H

/* Versioning. */
#define PACKAGE_NAME             "PostgreSQL"
#define PACKAGE_TARNAME          "postgresql"
#define PACKAGE_VERSION          "17.6"
#define PACKAGE_STRING           "PostgreSQL 17.6"
#define PACKAGE_BUGREPORT        "pgsql-bugs@lists.postgresql.org"
#define PACKAGE_URL              "https://www.postgresql.org/"
#define PG_VERSION               "17.6"
#define PG_VERSION_NUM           170006
#define PG_MAJORVERSION          "17"
#define PG_MAJORVERSION_NUM      17
#define PG_MINORVERSION_NUM      6
#define PG_VERSION_STR \
  "PostgreSQL 17.6 (Bazel experimental build) on host"
#define CONFIGURE_ARGS           "(hand-written overlay; not from configure)"

/* Type sizes — 64-bit POSIX. */
#define SIZEOF_BOOL              1
#define SIZEOF_SHORT             2
#define SIZEOF_INT               4
#define SIZEOF_LONG              8
#define SIZEOF_LONG_LONG         8
#define SIZEOF_SIZE_T            8
#define SIZEOF_VOID_P            8
#define SIZEOF_OFF_T             8

#define ALIGNOF_SHORT            2
#define ALIGNOF_INT              4
#define ALIGNOF_LONG             8
#define ALIGNOF_LONG_LONG_INT    8
#define ALIGNOF_DOUBLE           8
#define ALIGNOF_PG_INT128_TYPE   16
#define MAXIMUM_ALIGNOF          8

/* Standard headers we always have on POSIX. */
#define HAVE_STDBOOL_H           1
#define HAVE_STDINT_H            1
#define HAVE_STDLIB_H            1
#define HAVE_STDIO_H             1
#define HAVE_STRING_H            1
#define HAVE_STRINGS_H           1
#define HAVE_UNISTD_H            1
#define HAVE_INTTYPES_H          1
#define HAVE_SYS_TYPES_H         1
#define HAVE_SYS_STAT_H          1
#define HAVE_SYS_TIME_H          1
#define HAVE_SYS_WAIT_H          1
#define HAVE_SYS_RESOURCE_H      1
#define HAVE_SYS_SELECT_H        1
#define HAVE_SYS_SOCKET_H        1
#define HAVE_SYS_UN_H            1
#define HAVE_SYS_FILE_H          1
#define HAVE_SYS_IPC_H           1
#define HAVE_SYS_SHM_H           1
#define HAVE_SYS_PARAM_H         1
#define HAVE_FCNTL_H             1
#define HAVE_TIME_H              1
#define HAVE_SIGNAL_H            1
#define HAVE_ERRNO_H             1
#define HAVE_LOCALE_H            1
#define HAVE_LIMITS_H            1
#define HAVE_TERMIOS_H           1
#define HAVE_NETDB_H             1
#define HAVE_NETINET_IN_H        1
#define HAVE_NETINET_TCP_H       1
#define HAVE_ARPA_INET_H         1
#define HAVE_PWD_H               1
#define HAVE_GRP_H               1
#define HAVE_GETOPT_H            1
#define HAVE_LANGINFO_H          1
#define HAVE_EXECINFO_H          1
#define HAVE_POLL_H              1
#define HAVE_DLFCN_H             1
#define HAVE_MEMORY_H            1

/* Int-type detection.
 *
 * PG's c.h uses `#ifndef HAVE_INT8` to *gate its own typedef* of
 * the `int8` name. HAVE_INT8 / HAVE_UINT8 are defined by configure
 * only when the system already has these specific names (rare —
 * stdint.h provides `int8_t`, not `int8`). On macOS / Linux we let
 * PG do the typedef itself, so leave these undefined.
 *
 * Same logic for HAVE_INT64 / HAVE_UINT64.
 *
 * HAVE_LONG_INT_64 *is* set on 64-bit POSIX (long is 8 bytes), so
 * PG can `typedef long int64;` rather than `long long`. */
#undef HAVE_INT8
#undef HAVE_UINT8
#undef HAVE_INT64
#undef HAVE_UINT64
#define HAVE_LONG_INT_64         1
/* HAVE_INT128 is autoconf-detected as a compiler-supported __int128. */
#define HAVE_INT128              1

/* Function availability — assume modern POSIX. */
#define HAVE_STRTOLL             1
#define HAVE_STRTOULL            1
#define HAVE_SNPRINTF            1
#define HAVE_VSNPRINTF           1
#define HAVE_STRDUP              1
#define HAVE_STRNDUP             1
#define HAVE_STRERROR            1
#define HAVE_STRERROR_R          1
#define HAVE_DECL_STRERROR_R     1
#define HAVE_MEMSET_S            1
#define HAVE_EXPLICIT_BZERO      1
#define HAVE_GETIFADDRS          1
#define HAVE_GETOPT              1
#define HAVE_GETOPT_LONG         1
#define HAVE_STRSIGNAL           1
#define HAVE_DECL_STRLCPY        1
#define HAVE_DECL_STRLCAT        1
#define HAVE_DECL_STRNLEN        1

/* Optional features OFF unless we explicitly need them. */
/* No SSL, no readline, no zlib, no LZ4, no zstd, no Kerberos, no LDAP,
   no PAM, no Bonjour, no SystemTap, no liburing, no ICU, no Gettext. */

/* All `#ifdef` features below are *undefined* — PG's c.h pattern
 * checks `#ifdef FOO`, not the macro's value, so a `#define FOO 0`
 * would still enable that branch. Leave them unset entirely. */
#undef ENABLE_NLS
#undef USE_OPENSSL
#undef USE_LDAP

/* Block-size defaults from configure's --with-blocksize=8 default. */
#define BLCKSZ                   8192
#define RELSEG_SIZE              131072
#define XLOG_BLCKSZ              8192
#define DEF_PGPORT               5432
#define DEF_PGPORT_STR           "5432"

/* Locale memory model on macOS / Linux: USE_LIBC */
#define USE_VALGRIND             0

/* C standard version assumed. */
#define _LARGEFILE_SOURCE        1

/* `pg_restrict` is configure's portable spelling of C99 `restrict`.
 * Modern clang / gcc all support the keyword unconditionally. */
#define pg_restrict restrict

/* Use the standard `inline` keyword for force-inlining helpers
 * (configure normally picks `__attribute__((always_inline))` etc.). */
#define pg_attribute_always_inline inline __attribute__((always_inline))
#define pg_attribute_no_unused inline __attribute__((__unused__))
#define pg_attribute_aligned(n) __attribute__((aligned(n)))
#define pg_attribute_noreturn() __attribute__((noreturn))
#define pg_attribute_printf(f, a) __attribute__((format(printf, f, a)))
#define pg_noinline __attribute__((noinline))
#define HAVE_GCC__ATOMIC_INT32_CAS 1
#define HAVE_GCC__SYNC_INT32_CAS 1
#define HAVE_FUNCNAME__FUNC 1

#endif /* PG_CONFIG_H */
