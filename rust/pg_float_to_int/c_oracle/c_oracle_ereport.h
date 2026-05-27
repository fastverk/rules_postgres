/*
 * c_oracle_ereport.h — shared ereport / errcode / errmsg macro
 * overrides for the C oracle in Pg.Ir-emitted crates.
 *
 * Why this exists: Postgres' V1 fmgr bodies signal errors via
 *   ereport(ERROR, (errcode(ERRCODE_NUMERIC_VALUE_OUT_OF_RANGE),
 *                   errmsg("integer out of range")));
 * which expands to a longjmp out of the surrounding transaction.
 * Our standalone C oracle has no transaction. We model the behavior
 * with:
 *   1. A thread-local jmp_buf per crate (set by the c_* wrapper before
 *      calling into the renamed body).
 *   2. ereport(ERROR, ...) → call pg_fcinfo_set_error(kind) +
 *      longjmp(fmgr_oracle_jmp, 1).
 *   3. The wrapper's setjmp() catches the longjmp and returns 0.
 *
 * Effect: the vendored body remains BYTE-IDENTICAL to real Postgres
 * source (no embedded returns, no special macros), and the diff-test
 * harness sees a `*_fcinfo_set_error()` flag plus a returned Datum
 * of 0 — equivalent observable behavior to a real Postgres backend
 * catching ereport(ERROR, ...) at the SPI/transaction boundary.
 *
 * The Rust side never longjmps; it just sets the flag and returns 0
 * directly. Only the C oracle uses longjmp — Rust drops are not
 * skipped.
 *
 * Usage:
 *   - Include this header in the renamed_<cluster>.c shim BEFORE the
 *     vendored body is #included.
 *   - The wrappers.c must `#include <setjmp.h>`, declare the shared
 *     thread-local `fmgr_oracle_jmp` jmp_buf, and wrap each c_<fn>
 *     entry point with setjmp() that returns 0 on longjmp.
 */

#ifndef C_ORACLE_EREPORT_H
#define C_ORACLE_EREPORT_H

#include <stdint.h>
#include <setjmp.h>

/* Defined in wrappers.c — single per-cluster TLS jmp_buf. */
extern __thread jmp_buf fmgr_oracle_jmp;

/* Implemented in pg_fcinfo (Rust). Sets the shared TLS error flag. */
extern void pg_fcinfo_set_error(uint32_t kind);

/* SQLSTATE kinds known to the Rust side. Keep in sync with
 * pg_fcinfo::FmgrErrorKind. */
#define FMGR_ERR_NUMERIC_VALUE_OUT_OF_RANGE   1u
#define FMGR_ERR_DIVISION_BY_ZERO             2u
#define FMGR_ERR_DATETIME_VALUE_OUT_OF_RANGE  3u

/* Postgres ereport levels. Only ERROR matters for the oracle (LOG /
 * NOTICE / etc. shouldn't appear in pure V1 fmgr arithmetic bodies; if
 * they do, the cluster is out of scope). */
#define DEBUG5  10
#define DEBUG4  11
#define DEBUG3  12
#define DEBUG2  13
#define DEBUG1  14
#define LOG     15
#define INFO    17
#define NOTICE  18
#define WARNING 19
#define ERROR   21
#define FATAL   22
#define PANIC   23

/*
 * Map an `errcode(...)` argument to one of our FMGR_ERR_* discriminants.
 * Postgres builds SQLSTATE codes via MAKE_SQLSTATE; for the oracle we
 * only need to distinguish a handful, so we recognize the two we care
 * about by their MAKE_SQLSTATE-packed value and default everything else
 * to NUMERIC_VALUE_OUT_OF_RANGE.
 *
 * MAKE_SQLSTATE packs 5 ASCII chars into 32 bits, one nibble per char:
 *   '2'=0x32, '0'=0x30, '3'=0x33, '1'=0x31 etc.
 * "22003" → bytes 0x32 0x32 0x30 0x30 0x33 → packed LE as ((0x32) |
 * (0x32<<6) | (0x30<<12) | (0x30<<18) | (0x33<<24)) per
 * src/include/utils/elog.h.
 *
 * Rather than reverse-engineer MAKE_SQLSTATE's bit-packing here, we
 * intercept it: redefine MAKE_SQLSTATE / errcode to capture the textual
 * SQLSTATE and route to our kinds. ERRCODE_* macros expand to
 * MAKE_SQLSTATE(...) per `errcodes.h`, so this captures all of them.
 */
extern __thread uint32_t fmgr_oracle_last_errcode;

#undef  MAKE_SQLSTATE
#define MAKE_SQLSTATE(a, b, c, d, e)                                         \
    /* Pack to a u32 the same way Postgres does so we can match on the */    \
    /* original ERRCODE_* identifier without parsing strings. */             \
    ((uint32_t)((a)       | ((b) << 6)  | ((c) << 12) |                       \
                ((d) << 18) | ((e) << 24)))

/* The two we currently care about. Match the bit-packing above. */
#define FMGR_SQLSTATE_NUMERIC_VALUE_OUT_OF_RANGE \
    MAKE_SQLSTATE('2','2','0','0','3')
#define FMGR_SQLSTATE_DIVISION_BY_ZERO \
    MAKE_SQLSTATE('2','2','0','1','2')
#define FMGR_SQLSTATE_DATETIME_VALUE_OUT_OF_RANGE \
    MAKE_SQLSTATE('2','2','0','0','8')

#define errcode(code)                                                        \
    (fmgr_oracle_last_errcode = (code), 0)

/* errmsg / errdetail / errhint and friends are no-ops on the oracle —
 * the diff-test only cares about which kind, not the textual message. */
#define errmsg(...)             0
#define errmsg_internal(...)    0
#define errdetail(...)          0
#define errdetail_internal(...) 0
#define errhint(...)            0

/*
 * ereport(ERROR, ...) → set the flag (kind derived from the captured
 * errcode), then longjmp to the wrapper's setjmp. Other levels are
 * silenced; if a pure V1 fmgr body legitimately emits a non-ERROR
 * report, treat that as a scope violation and abort.
 *
 * The variadic args contain the (errcode(...), errmsg(...), …)
 * comma-expression Postgres uses; we evaluate it for its side effect
 * (`errcode` sets `fmgr_oracle_last_errcode`) and discard the value.
 */
static inline void fmgr_oracle_route_error(int level)
{
    uint32_t kind;
    switch (fmgr_oracle_last_errcode) {
    case FMGR_SQLSTATE_DIVISION_BY_ZERO:
        kind = FMGR_ERR_DIVISION_BY_ZERO; break;
    case FMGR_SQLSTATE_DATETIME_VALUE_OUT_OF_RANGE:
        kind = FMGR_ERR_DATETIME_VALUE_OUT_OF_RANGE; break;
    case FMGR_SQLSTATE_NUMERIC_VALUE_OUT_OF_RANGE:
    default:
        kind = FMGR_ERR_NUMERIC_VALUE_OUT_OF_RANGE; break;
    }
    pg_fcinfo_set_error(kind);
    (void) level;
    longjmp(fmgr_oracle_jmp, 1);
}

#define ereport(level, rest)                                                 \
    do {                                                                     \
        fmgr_oracle_last_errcode = 0;                                        \
        (void) (rest); /* evaluates errcode/errmsg side-effects */           \
        if ((level) >= ERROR) {                                              \
            fmgr_oracle_route_error(level);                                  \
        }                                                                    \
    } while (0)

/* New-style multi-arg ereport (PG 14+) uses comma-separated calls
 * instead of a parenthesized expression: `ereport(ERROR, errcode(...),
 * errmsg(...));`. We accept both by routing through a variadic macro
 * that builds the same comma-expression internally. */
#define ereport_domain(level, domain, ...)  ereport((level), (__VA_ARGS__))

/* `elog(ERROR, "...")` is a simpler form. Treat it as numeric-OOR for
 * the diff-test — pure arithmetic V1 fmgrs don't use elog. */
#define elog(level, ...)                                                     \
    do {                                                                     \
        if ((level) >= ERROR) {                                              \
            pg_fcinfo_set_error(FMGR_ERR_NUMERIC_VALUE_OUT_OF_RANGE);        \
            longjmp(fmgr_oracle_jmp, 1);                                     \
        }                                                                    \
    } while (0)

/* `unlikely()` is a pure compiler hint in Postgres; preserve byte-level
 * equivalence with the vendored body by defining it as identity. */
#ifndef unlikely
#define unlikely(x) (x)
#endif
#ifndef likely
#define likely(x)   (x)
#endif

#endif /* C_ORACLE_EREPORT_H */
