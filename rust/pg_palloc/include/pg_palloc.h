/*
 * pg_palloc.h — C declarations for the pg_palloc crate.
 *
 * Include this in C oracle bodies that need palloc / pfree / repalloc
 * / MemoryContext* without dragging in the full Postgres headers.
 * Mirrors `src/include/utils/palloc.h` from PG 17.6, narrowed to the
 * API surface our adapter implements.
 *
 * Linkage: link against `libpg_palloc.a` (the `staticlib` crate-type)
 * or `libpg_palloc.dylib`.
 *
 * Companion documentation for the chunk-header layout, the
 * thread-local CurrentMemoryContext, and the "pfree is a no-op"
 * decision lives in src/lib.rs.
 */

#ifndef PG_PALLOC_H
#define PG_PALLOC_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Opaque to C; the Rust side owns the struct definition.
 * Matches Postgres' `typedef struct MemoryContextData *MemoryContext`. */
typedef struct MemoryContext MemoryContext;

/* MCXT_ALLOC_* flag bits for palloc_extended / MemoryContextAllocExtended. */
#define MCXT_ALLOC_HUGE   0x01u
#define MCXT_ALLOC_NO_OOM 0x02u
#define MCXT_ALLOC_ZERO   0x04u

/* ── Current-context allocation ─────────────────────────────────────── */

void *palloc(size_t size);
void *palloc0(size_t size);
void *palloc_extended(size_t size, uint32_t flags);

void  pfree(void *pointer);
void *repalloc(void *pointer, size_t size);
void *repalloc_huge(void *pointer, size_t size);

char *pstrdup(const char *s);
char *pnstrdup(const char *s, size_t n);

/* ── Explicit-context allocation ───────────────────────────────────── */

void *MemoryContextAlloc(MemoryContext *context, size_t size);
void *MemoryContextAllocZero(MemoryContext *context, size_t size);
void *MemoryContextAllocExtended(MemoryContext *context, size_t size, uint32_t flags);

/* ── Context lifecycle ─────────────────────────────────────────────── */

MemoryContext *MemoryContextCreate(void);
void           MemoryContextReset(MemoryContext *context);
void           MemoryContextDelete(MemoryContext *context);
MemoryContext *MemoryContextSwitchTo(MemoryContext *context);
MemoryContext *GetCurrentMemoryContext(void);

#ifdef __cplusplus
} /* extern "C" */
#endif

#endif /* PG_PALLOC_H */
