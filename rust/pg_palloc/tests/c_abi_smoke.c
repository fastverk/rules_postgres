/*
 * c_abi_smoke.c — C harness exercising pg_palloc's C ABI through
 * pg_palloc.h. Linked into the c_abi_smoke.rs integration test.
 *
 * Each c_smoke_* function returns 0 on success, a negative error code
 * matching the assertion that failed on failure.
 */

#include <stddef.h>
#include <stdint.h>
#include <string.h>
#include "pg_palloc.h"

/*
 * pstrdup round trip:
 *   - The Rust side has already set CurrentMemoryContext via
 *     with_memory_context.
 *   - pstrdup must give us a NUL-terminated copy in the current arena.
 */
int c_smoke_pstrdup_round_trip(void)
{
    const char *src = "hello pg_palloc";
    char *dup = pstrdup(src);
    if (!dup)                return -1;
    if (strcmp(dup, src))    return -2;
    /* No pfree: the arena reclaims at MemoryContextReset/Delete. */
    return 0;
}

/*
 * Grow a buffer, then shrink it. With the chunk header in place,
 * repalloc(shrink) must NOT read past `min(old, new)` — i.e., the
 * uninitialized tail of the shrunk-to buffer must remain whatever
 * fresh-allocation gives us, not stale grown-buffer contents.
 *
 * We assert payload preservation up to the smaller size and confirm
 * the call doesn't crash. (The chunk_header_round_trips Rust unit test
 * already covers the size-readback; this is the C ABI sanity gate.)
 */
int c_smoke_repalloc_grow_then_shrink(void)
{
    char *p = (char *) palloc(8);
    for (int i = 0; i < 8; i++) p[i] = (char) ('a' + i);
    char *p2 = (char *) repalloc(p, 32);
    for (int i = 0; i < 8; i++) {
        if (p2[i] != (char) ('a' + i)) return -10 - i;
    }
    /* Write into the freshly-grown tail. */
    for (int i = 8; i < 32; i++) p2[i] = 'Z';
    char *p3 = (char *) repalloc(p2, 4);
    for (int i = 0; i < 4; i++) {
        if (p3[i] != (char) ('a' + i)) return -20 - i;
    }
    return 0;
}

/*
 * Two explicit contexts hold independent arenas. Allocate in each,
 * Reset one, the other still owns its data.
 *
 * We can't actually verify "memory was reclaimed" portably; we can
 * verify reset returns and subsequent palloc keeps working.
 */
int c_smoke_explicit_context_independent_arenas(void)
{
    MemoryContext *ctx_a = MemoryContextCreate();
    MemoryContext *ctx_b = MemoryContextCreate();
    if (!ctx_a || !ctx_b) return -1;

    char *pa = (char *) MemoryContextAlloc(ctx_a, 16);
    char *pb = (char *) MemoryContextAllocZero(ctx_b, 16);
    if (!pa || !pb) return -2;

    /* MemoryContextAllocZero must zero. */
    for (int i = 0; i < 16; i++) {
        if (pb[i] != 0) return -10 - i;
    }

    /* Reset A. After reset, allocate again from A and confirm it works. */
    MemoryContextReset(ctx_a);
    char *pa2 = (char *) MemoryContextAlloc(ctx_a, 32);
    if (!pa2) return -30;

    /* B is independent — its prior allocation pointer is still in its
     * arena (we can't dereference safely because reset would have
     * been catastrophic, but B wasn't reset, so we just verify the
     * fresh alloc still works.) */
    char *pb2 = (char *) MemoryContextAlloc(ctx_b, 32);
    if (!pb2) return -40;

    MemoryContextDelete(ctx_a);
    MemoryContextDelete(ctx_b);
    return 0;
}

/*
 * Switching to a context, allocating, resetting, and allocating again
 * must keep the context-as-current usable across reset boundaries.
 */
int c_smoke_reset_keeps_context_usable(void)
{
    MemoryContext *ctx = MemoryContextCreate();
    if (!ctx) return -1;
    MemoryContext *prev = MemoryContextSwitchTo(ctx);

    char *p = (char *) palloc(100);
    if (!p) { MemoryContextSwitchTo(prev); MemoryContextDelete(ctx); return -2; }

    MemoryContextReset(ctx);
    char *p2 = (char *) palloc(100);
    if (!p2) { MemoryContextSwitchTo(prev); MemoryContextDelete(ctx); return -3; }

    MemoryContextSwitchTo(prev);
    MemoryContextDelete(ctx);
    return 0;
}
