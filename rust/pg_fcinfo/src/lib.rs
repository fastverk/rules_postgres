//! Postgres V1 fmgr ABI bridge.
//!
//! Provides Rust mirrors of `Datum`, `NullableDatum`, and
//! `FunctionCallInfoBaseData` (the structs Postgres' V1 fmgr uses
//! to pass args + collect results), plus per-type encode/decode
//! helpers and a `build_fcinfo!` macro that constructs a valid
//! fcinfo from typed Rust args in one line.
//!
//! ## Why this exists
//!
//! `auto_cluster --emit-difftest` recognizes the V1 fmgr shape
//! (`Datum fn(PG_FUNCTION_ARGS)`) and emits scaffolded diff-tests
//! for it. Phase 1 of that work (2026-05-26 morning) emitted scaffolds
//! with `TODO(fcinfo)` markers requiring ~5 min of human fill per
//! function. This crate is phase 2: the scaffolds now use
//! `build_fcinfo!` and the per-type encoders here, dropping per-function
//! fill to zero for the common arg types (int32/int64/bool/float8/ptr).
//!
//! ## ABI guarantees
//!
//! `FunctionCallInfoBaseData` and `NullableDatum` are `#[repr(C)]` and
//! the field order mirrors Postgres 17.x's `src/include/fmgr.h` /
//! `src/include/postgres.h`. The `args` array is sized to
//! `FUNC_MAX_ARGS = 100` (Postgres' upper bound), but most V1 calls
//! use ≤ 4. `build.rs` compiles a small C probe to verify our
//! `sizeof` matches Postgres' at link time.
//!
//! ## Scope
//!
//! - Datum encoding for `i16` / `i32` / `i64` / `bool` / `f32` / `f64`
//!   / raw pointers. Sufficient for the leaf-pool subset of V1 fmgr
//!   functions (geo_ops, float, int, int8, date, timestamp, network,
//!   uuid, mac, oid, bool, char — covers most of the 983-fn cluster).
//! - Datum decoding for the same types.
//! - `build_fcinfo!(args = [ ... ])` macro: zero-allocation construction
//!   of a fcinfo on the caller's stack.
//! - NOT yet: text/varlena/numeric/array types — those require palloc
//!   integration which the leaf pool by definition doesn't have. The
//!   memory_context adapter from purity_detect's classification is the
//!   right home for those when we tackle them.

#![allow(clippy::missing_safety_doc)]

use std::ffi::c_void;

/// Postgres' `Datum` is `typedef uintptr_t`. On 64-bit Postgres builds
/// (the target for all current klad work) it's a `u64`. 32-bit Postgres
/// is unsupported by this crate.
pub type Datum = u64;

/// `Oid` is `typedef uint32` in Postgres. Used as the `fncollation`
/// field of fcinfo and various other places.
pub type Oid = u32;

/// Mirror of `NullableDatum` from `src/include/postgres.h`. Field order
/// + padding MUST match the C-side layout; verified by build.rs.
///
/// The C struct has a comment noting that due to alignment padding the
/// trailing byte after `isnull` could be used for flags. Our Rust mirror
/// leaves that padding implicit (it's just a `bool` followed by padding
/// up to the next 8-byte boundary).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NullableDatum {
    /// The `Datum` value. Encoding depends on the SQL type; use the
    /// `encode_*` helpers in this module to populate.
    pub value: Datum,
    /// True iff the value is SQL NULL.
    pub isnull: bool,
}

impl Default for NullableDatum {
    fn default() -> Self {
        Self { value: 0, isnull: false }
    }
}

impl NullableDatum {
    /// Construct a non-null datum with the given encoded value.
    pub const fn val(value: Datum) -> Self {
        Self { value, isnull: false }
    }
    /// Construct a SQL-NULL datum.
    pub const fn null() -> Self {
        Self { value: 0, isnull: true }
    }
}

/// Maximum number of args a V1 fmgr function can take. Matches
/// `FUNC_MAX_ARGS` from Postgres' `src/include/pg_config_manual.h`.
pub const FUNC_MAX_ARGS: usize = 100;

/// Mirror of `FunctionCallInfoBaseData` from `src/include/fmgr.h`.
///
/// The C struct ends with `NullableDatum args[FLEXIBLE_ARRAY_MEMBER];`
/// — a flexible array. C callers allocate enough storage for `nargs`
/// args (via the `LOCAL_FCINFO` macro or `SizeForFunctionCallInfo`).
/// Our Rust mirror uses a fixed-size `[NullableDatum; FUNC_MAX_ARGS]`
/// so the type has a static layout; this wastes some stack but keeps
/// the FFI surface ergonomic. V1 calls almost always use nargs ≤ 4.
#[repr(C)]
pub struct FunctionCallInfoBaseData {
    /// `FmgrInfo *` — system-catalog lookup info. Null is OK for
    /// pure leaf-pool functions that don't dispatch via fmgr_info.
    pub flinfo: *mut c_void,
    /// `fmNodePtr` — pass info about call context. Null is OK.
    pub context: *mut c_void,
    /// `fmNodePtr` — pass/return extra result info. Null is OK.
    pub resultinfo: *mut c_void,
    /// Collation OID for the function to use. 0 = default.
    pub fncollation: Oid,
    /// Function must set true if result is NULL. Caller reads this
    /// after the call to distinguish a NULL return from a 0 Datum.
    pub isnull: bool,
    /// Number of args actually passed (≤ FUNC_MAX_ARGS).
    pub nargs: i16,
    /// The args. Only the first `nargs` are read by the callee.
    pub args: [NullableDatum; FUNC_MAX_ARGS],
}

impl Default for FunctionCallInfoBaseData {
    fn default() -> Self {
        Self {
            flinfo: std::ptr::null_mut(),
            context: std::ptr::null_mut(),
            resultinfo: std::ptr::null_mut(),
            fncollation: 0,
            isnull: false,
            nargs: 0,
            args: [NullableDatum::default(); FUNC_MAX_ARGS],
        }
    }
}

// ─── Encoding helpers ─────────────────────────────────────────────────────────
//
// Postgres' Datum encoding (from `src/include/postgres.h`):
//   Int16   → low 16 bits, sign-extended via uint16 then cast
//   Int32   → low 32 bits via uint32
//   Int64   → full 64 bits
//   Bool    → 0 or 1
//   Float4  → bit pattern as uint32
//   Float8  → bit pattern as uint64
//   Pointer → cast to uintptr_t
//
// We match those conventions exactly so a Datum produced by an
// `encode_*` helper here is byte-identical to one produced by the
// corresponding C `Int32GetDatum`/`Float8GetDatum`/etc. macros.

#[inline]
pub fn encode_i8(v: i8) -> Datum {
    v as u8 as u64
}

#[inline]
pub fn encode_u8(v: u8) -> Datum {
    v as u64
}

#[inline]
pub fn encode_i16(v: i16) -> Datum {
    // C: Int16GetDatum(v) → ((Datum) (int16) (v))
    v as u16 as u64
}

#[inline]
pub fn encode_i32(v: i32) -> Datum {
    v as u32 as u64
}

#[inline]
pub fn encode_i64(v: i64) -> Datum {
    v as u64
}

#[inline]
pub fn encode_u32(v: u32) -> Datum {
    v as u64
}

#[inline]
pub fn encode_u64(v: u64) -> Datum {
    v
}

#[inline]
pub fn encode_bool(v: bool) -> Datum {
    if v { 1 } else { 0 }
}

#[inline]
pub fn encode_f32(v: f32) -> Datum {
    // C: Float4GetDatum boxes through a uint32 union; same bit pattern.
    v.to_bits() as u64
}

#[inline]
pub fn encode_f64(v: f64) -> Datum {
    v.to_bits()
}

#[inline]
pub fn encode_ptr<T>(p: *const T) -> Datum {
    p as usize as u64
}

#[inline]
pub fn encode_mut_ptr<T>(p: *mut T) -> Datum {
    p as usize as u64
}

// ─── Decoding helpers ─────────────────────────────────────────────────────────

#[inline]
pub fn decode_i8(d: Datum) -> i8 {
    d as u8 as i8
}

#[inline]
pub fn decode_u8(d: Datum) -> u8 {
    d as u8
}

#[inline]
pub fn decode_i16(d: Datum) -> i16 {
    d as u16 as i16
}

#[inline]
pub fn decode_i32(d: Datum) -> i32 {
    d as u32 as i32
}

#[inline]
pub fn decode_i64(d: Datum) -> i64 {
    d as i64
}

#[inline]
pub fn decode_u32(d: Datum) -> u32 {
    d as u32
}

#[inline]
pub fn decode_u64(d: Datum) -> u64 {
    d
}

#[inline]
pub fn decode_bool(d: Datum) -> bool {
    (d & 1) != 0
}

// ─── Postgres-NaN-aware float comparisons ──────────────────────────────────
//
// Postgres' `src/include/utils/float.h` defines float comparisons that
// treat NaN as the greatest value (NaN sorts AFTER +Infinity) AND equal
// to itself (NaN == NaN). Rust's native `==`/`<`/etc. use IEEE-754:
// NaN is unordered and NaN != NaN. So the V1 fmgr float comparison
// operators (`float4_eq`, `float8_lt`, etc.) need explicit NaN handling.
//
// These helpers wrap that logic per-op. The Pg.Ir codegen emits calls
// to them for the float families. For non-float types the Lean emit
// uses native Rust `==`/`<`/etc. directly (no helper).
//
// Canonical reference:
//   src/include/utils/float.h, PG 17.6, lines ~340-380.

#[inline] pub fn pgcmp_eq_f32(a: f32, b: f32) -> bool {
    if a.is_nan() { b.is_nan() } else { !b.is_nan() && a == b }
}
#[inline] pub fn pgcmp_ne_f32(a: f32, b: f32) -> bool {
    if a.is_nan() { !b.is_nan() } else { b.is_nan() || a != b }
}
#[inline] pub fn pgcmp_lt_f32(a: f32, b: f32) -> bool {
    !a.is_nan() && (b.is_nan() || a < b)
}
#[inline] pub fn pgcmp_le_f32(a: f32, b: f32) -> bool {
    b.is_nan() || (!a.is_nan() && a <= b)
}
#[inline] pub fn pgcmp_gt_f32(a: f32, b: f32) -> bool {
    a.is_nan() && !b.is_nan() || (!b.is_nan() && a > b)
}
#[inline] pub fn pgcmp_ge_f32(a: f32, b: f32) -> bool {
    a.is_nan() || (!b.is_nan() && a >= b)
}

#[inline] pub fn pgcmp_eq_f64(a: f64, b: f64) -> bool {
    if a.is_nan() { b.is_nan() } else { !b.is_nan() && a == b }
}
#[inline] pub fn pgcmp_ne_f64(a: f64, b: f64) -> bool {
    if a.is_nan() { !b.is_nan() } else { b.is_nan() || a != b }
}
#[inline] pub fn pgcmp_lt_f64(a: f64, b: f64) -> bool {
    !a.is_nan() && (b.is_nan() || a < b)
}
#[inline] pub fn pgcmp_le_f64(a: f64, b: f64) -> bool {
    b.is_nan() || (!a.is_nan() && a <= b)
}
#[inline] pub fn pgcmp_gt_f64(a: f64, b: f64) -> bool {
    a.is_nan() && !b.is_nan() || (!b.is_nan() && a > b)
}
#[inline] pub fn pgcmp_ge_f64(a: f64, b: f64) -> bool {
    a.is_nan() || (!b.is_nan() && a >= b)
}

// ─── Postgres 3-way `<type>_cmp_internal` helpers ──────────────────────────
//
// Postgres has a code-sharing pattern where each comparison family
// routes all 6 ops through a single `<type>_cmp_internal` 3-way
// compare that returns -1/0/1. The V1 fmgr eq/ne/lt/le/gt/ge functions
// then compare the cmp_internal result against 0.
//
// We mirror those helpers here so the Lean-emitted Rust matches
// Postgres' algorithmic structure (and thus the C-AST structural-diff
// gate passes).

/// Mirror of `timestamp_cmp_internal` from src/include/utils/timestamp.h.
/// `Timestamp` is `int64` in Postgres on 64-bit builds.
#[inline]
pub fn timestamp_cmp_internal(a: i64, b: i64) -> i32 {
    if a < b { -1 } else if a > b { 1 } else { 0 }
}

// ─── Pointer-valued Datum types (Postgres pass-by-reference types) ─────────
//
// Postgres' variable-width and pointer-valued types (Interval,
// ArrayType, varlena, text, …) pass their Datums as raw pointers
// `*mut T` cast to u64. The fmgr stub:
//   - Reads `args[N].value` as a pointer (`decode_*_p`).
//   - Computes a result.
//   - Allocates the result via palloc.
//   - Casts the result pointer back to u64 (`encode_*_p`).
//
// We don't enforce lifetime annotations here — that's all unsafe
// pointer math at the C ABI boundary. The pg_palloc adapter owns the
// arena lifetime invariants; callers must keep CurrentMemoryContext
// alive across the call.
//
// ## Interval
//
// `Interval` is Postgres' interval datatype — 16 bytes total: 8-byte
// microsecond count, 4-byte day count, 4-byte month count. The layout
// matches `src/include/datatype/timestamp.h::Interval` exactly.
//
// Infinite intervals use sentinel values: `+infinity` is all-MAX,
// `-infinity` is all-MIN. The `INTERVAL_IS_NOBEGIN` / `INTERVAL_IS_NOEND`
// helpers below mirror the macros of the same name in
// `datatype/timestamp.h`. Bodies that operate on intervals must
// check these BEFORE doing arithmetic, because the infinity sentinel
// values would otherwise overflow the per-field overflow checks.

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Interval {
    /// Microseconds (signed). All time units smaller than days.
    pub time:  i64,
    /// Day count, after `time` for alignment per Postgres'
    /// `pg_attribute_aligned(8)` declaration.
    pub day:   i32,
    /// Month count (years rolled in).
    pub month: i32,
}

/// `Interval` matches Postgres' layout (8-byte time, 4-byte day,
/// 4-byte month). The `pg_palloc::palloc(size_of::<Interval>())`
/// allocation is 16 bytes payload + 8-byte chunk header.
pub const SIZEOF_INTERVAL: usize = 16;

/// `INTERVAL_IS_NOBEGIN(i)` — true if the interval encodes -infinity.
#[inline]
pub fn interval_is_nobegin(i: &Interval) -> bool {
    i.month == i32::MIN && i.day == i32::MIN && i.time == i64::MIN
}

/// `INTERVAL_IS_NOEND(i)` — true if the interval encodes +infinity.
#[inline]
pub fn interval_is_noend(i: &Interval) -> bool {
    i.month == i32::MAX && i.day == i32::MAX && i.time == i64::MAX
}

/// `INTERVAL_NOBEGIN(i)` — write the -infinity sentinel into `i`.
#[inline]
pub fn interval_nobegin(i: &mut Interval) {
    i.time = i64::MIN; i.day = i32::MIN; i.month = i32::MIN;
}

/// `INTERVAL_NOEND(i)` — write the +infinity sentinel into `i`.
#[inline]
pub fn interval_noend(i: &mut Interval) {
    i.time = i64::MAX; i.day = i32::MAX; i.month = i32::MAX;
}

/// `INTERVAL_NOT_FINITE(i)` — true if i is either sentinel.
#[inline]
pub fn interval_not_finite(i: &Interval) -> bool {
    interval_is_nobegin(i) || interval_is_noend(i)
}

/// `PG_GETARG_INTERVAL_P(n)` analog — decode arg as `*const Interval`.
///
/// # Safety
/// `d` must be a Datum produced by a Postgres-compatible
/// `PG_RETURN_INTERVAL_P` call (i.e., a `*const Interval` cast to u64).
/// The pointee's lifetime is the caller's MemoryContext arena.
#[inline]
pub unsafe fn decode_interval_p(d: Datum) -> *const Interval {
    d as *const Interval
}

/// `PG_RETURN_INTERVAL_P(p)` analog — encode a `*mut Interval` as a
/// Datum. The caller must have allocated the pointee via `palloc` (or
/// equivalent) from the active MemoryContext.
#[inline]
pub fn encode_interval_p(p: *mut Interval) -> Datum {
    p as Datum
}

// ─── UUID (16-byte fixed-size struct) ──────────────────────────────────────────
//
// UUID in Postgres is a 16-byte fixed-size struct similar to Interval:
// just a pointer to the 16 bytes, no varlena header. Passed by pointer
// via `PG_GETARG_UUID_P` / `PG_RETURN_UUID_P`.

/// `PG_GETARG_UUID_P(n)` analog — decode arg as `*const u8` (the UUID data).
///
/// # Safety
/// `d` must be a Datum produced by a Postgres-compatible
/// `PG_RETURN_UUID_P` call (i.e., a `*const pg_uuid_t` cast to u64).
/// The pointee is 16 bytes of UUID data.
#[inline]
pub unsafe fn decode_uuid_p(d: Datum) -> *const u8 {
    d as *const u8
}

/// `PG_RETURN_UUID_P(p)` analog — encode a `*const u8` (UUID data) as a Datum.
///
/// # Safety
/// `p` must be a valid pointer to at least 16 bytes of UUID data.
#[inline]
pub fn encode_uuid_p(p: *const u8) -> Datum {
    p as Datum
}

// ─── macaddr (6-byte fixed-size struct) ──────────────────────────────────────
//
// macaddr in Postgres is a 6-byte fixed-size struct (unsigned char a,b,c,d,e,f),
// similar to UUID: just a pointer to the 6 bytes, no varlena header. Passed by
// pointer via `PG_GETARG_MACADDR_P` / `PG_RETURN_MACADDR_P`.

/// `PG_GETARG_MACADDR_P(n)` analog — decode arg as `*const u8` (the macaddr data).
///
/// # Safety
/// `d` must be a Datum produced by a Postgres-compatible
/// `PG_RETURN_MACADDR_P` call (i.e., a `*const macaddr` cast to u64).
/// The pointee is 6 bytes of macaddr data.
#[inline]
pub unsafe fn decode_macaddr_p(d: Datum) -> *const u8 {
    d as *const u8
}

/// `PG_RETURN_MACADDR_P(p)` analog — encode a `*const u8` (macaddr data) as a Datum.
///
/// # Safety
/// `p` must be a valid pointer to at least 6 bytes of macaddr data.
#[inline]
pub fn encode_macaddr_p(p: *const u8) -> Datum {
    p as Datum
}

// ─── ItemPointer / tid (6-byte fixed-size struct) ────────────────────────────
//
// ItemPointer (tid) in Postgres is a 6-byte fixed-size struct (BlockIdData + OffsetNumber),
// similar to macaddr: just a pointer to the 6 bytes, no varlena header. Passed by
// pointer via `PG_GETARG_ITEMPOINTER` / `PG_RETURN_ITEMPOINTER`.

/// `PG_GETARG_ITEMPOINTER(n)` analog — decode arg as `*const u8` (the ItemPointer data).
///
/// # Safety
/// `d` must be a Datum produced by a Postgres-compatible
/// `PG_RETURN_ITEMPOINTER` call (i.e., a `*const ItemPointerData` cast to u64).
/// The pointee is 6 bytes of ItemPointer data.
#[inline]
pub unsafe fn decode_itempointer_p(d: Datum) -> *const u8 {
    d as *const u8
}

/// `PG_RETURN_ITEMPOINTER(p)` analog — encode a `*const u8` (ItemPointer data) as a Datum.
///
/// # Safety
/// `p` must be a valid pointer to at least 6 bytes of ItemPointer data.
#[inline]
pub fn encode_itempointer(p: *const u8) -> Datum {
    p as Datum
}

// ─── Varlena (variable-length type) header helpers ────────────────────────
//
// Postgres' variable-width types — bytea, text, varchar, numeric,
// arrays, ranges — share a varlena header layout. The header is either
// 4 bytes (long form) or 1 byte (short form, lengths ≤ 126). The low
// 2 bits of the first byte distinguish:
//
//   - 00 — 4-byte uncompressed header (the common case our v0 supports)
//   - 01 — 1-byte short header
//   - 10 — external (TOAST pointer)
//   - 11 — 4-byte compressed-inline header
//
// v0 scope: we model ONLY the 4-byte uncompressed case (00). This is
// the format produced by `palloc(size + VARHDRSZ) + SET_VARSIZE` —
// i.e., every varlena value we construct in pg_bytea / pg_text /
// pg_numeric etc. v1+ will add short-header handling for
// VARSIZE_ANY-style read paths that need to accept both forms.
//
// Reference: `src/include/varatt.h` (the macros) and `src/include/c.h`
// (the `varattrib_*` structs).

pub const VARHDRSZ: usize = 4;

/// Read the total varlena size (header + payload) from a 4-byte-header
/// varlena pointer. Equivalent to Postgres' `VARSIZE_4B(PTR)`:
/// `((header >> 2) & 0x3FFFFFFF)`. The low 2 bits encode the format
/// (which we require to be 0 = 4-byte uncompressed).
///
/// # Safety
/// `p` must point to a 4-byte-header varlena (low 2 bits of first byte
/// are 00). Use `varsize_any` to accept both 4-byte and 1-byte forms.
#[inline]
pub unsafe fn varsize_4b(p: *const u8) -> usize {
    let header = (p as *const u32).read_unaligned();
    ((header >> 2) & 0x3FFFFFFF) as usize
}

/// `VARDATA_4B(PTR)` — pointer to the payload (header + 4 bytes).
///
/// # Safety
/// Same as `varsize_4b`.
#[inline]
pub unsafe fn vardata_4b(p: *const u8) -> *const u8 {
    p.add(VARHDRSZ)
}

#[inline]
pub unsafe fn vardata_4b_mut(p: *mut u8) -> *mut u8 {
    p.add(VARHDRSZ)
}

/// `SET_VARSIZE_4B(PTR, len)` — write a 4-byte header for a varlena of
/// total length `len` (header included). Postgres encodes the header
/// as `(len << 2)` with low 2 bits = 0 (4-byte uncompressed format).
///
/// # Safety
/// `p` must point to at least `VARHDRSZ` bytes of writable memory.
/// `len` must fit in 30 bits (< 1 GiB).
#[inline]
pub unsafe fn set_varsize_4b(p: *mut u8, len: usize) {
    let header = (len as u32) << 2;
    (p as *mut u32).write_unaligned(header);
}

/// `VARSIZE_ANY(PTR)` — total size (header + payload) for either
/// 4-byte or 1-byte header. v0: 4-byte only.
///
/// # Safety
/// `p` must point to a 4-byte-header varlena.
#[inline]
pub unsafe fn varsize_any(p: *const u8) -> usize {
    let first_byte = *p;
    if first_byte & 0x03 == 0x00 {
        varsize_4b(p)
    } else {
        panic!("varsize_any: unsupported header tag 0x{:x}", first_byte & 0x03);
    }
}

/// `toast_raw_datum_size(d)` — Postgres' helper that returns the
/// logical (post-detoast) total size of a varlena without actually
/// detoasting. For our v0 scope (uncompressed, non-TOASTed inputs),
/// this reduces to `varsize_4b(d)`. Real Postgres handles compressed
/// and TOAST-external cases via the TOAST descriptor.
///
/// # Safety
/// `d` must be a Datum encoding a `*const u8` to a 4-byte-header
/// varlena.
#[inline]
pub unsafe fn toast_raw_datum_size(d: Datum) -> usize {
    varsize_4b(d as *const u8)
}

/// `VARSIZE_ANY_EXHDR(PTR)` — payload size (excludes header) for either
/// 4-byte or 1-byte header. In v0 we only support 4-byte, so this is
/// equivalent to `varsize_4b(p) - VARHDRSZ`. The 1-byte form will be
/// added in v1 when we lift functions that read TOAST inputs.
///
/// # Safety
/// `p` must point to a 4-byte-header varlena.
#[inline]
pub unsafe fn varsize_any_exhdr(p: *const u8) -> usize {
    let first_byte = *p;
    if first_byte & 0x03 == 0x00 {
        // 4-byte uncompressed header.
        varsize_4b(p).saturating_sub(VARHDRSZ)
    } else {
        // Other forms not yet supported.
        panic!("varsize_any_exhdr: unsupported header tag 0x{:x}", first_byte & 0x03);
    }
}

/// `VARDATA_ANY(PTR)` — payload pointer for either 4-byte or 1-byte
/// header. v0: 4-byte only.
///
/// # Safety
/// Same as `varsize_any_exhdr`.
#[inline]
pub unsafe fn vardata_any(p: *const u8) -> *const u8 {
    let first_byte = *p;
    if first_byte & 0x03 == 0x00 {
        vardata_4b(p)
    } else {
        panic!("vardata_any: unsupported header tag 0x{:x}", first_byte & 0x03);
    }
}

/// `PG_GETARG_BYTEA_P(N)` / `PG_GETARG_BYTEA_PP(N)` analog — decode
/// arg as a `*const u8` pointing at a varlena header. The pointee's
/// lifetime is the caller's MemoryContext arena.
///
/// # Safety
/// `d` must be a Datum produced by `PG_RETURN_BYTEA_P` (i.e., a varlena
/// pointer cast to Datum) for an UNCOMPRESSED, NON-TOASTED bytea.
#[inline]
pub unsafe fn decode_bytea_p(d: Datum) -> *const u8 {
    d as *const u8
}

/// `PG_RETURN_BYTEA_P(p)` analog — encode a varlena pointer as a
/// Datum.
#[inline]
pub fn encode_bytea_p(p: *mut u8) -> Datum {
    p as Datum
}

// ─── Jenkins/lookup3 hash (Postgres' hash_bytes_uint32 family) ─────────────
//
// Verbatim port of `src/common/hashfn.c::hash_bytes_uint32` and
// `hash_bytes_uint32_extended`, which together back every fixed-width
// integer-type hash function in Postgres (hashint2/4/8, hashoid,
// hashchar, hashenum, and their *extended counterparts).
//
// The constant `0x9e3779b9 + 4 + 3923095` (= the `a=b=c` init in
// hashfn.c) is encoded literally so anyone tracking the Postgres source
// can match the line.
//
// rot(x, k) on a single u32 is `pg_rotate_left32` which is just
// `(x << k) | (x >> (32 - k))`. Rust's `u32::rotate_left` is the
// canonical wrap-safe form.

#[inline(always)]
fn rot(x: u32, k: u32) -> u32 {
    x.rotate_left(k)
}

#[inline(always)]
fn mix(a: &mut u32, b: &mut u32, c: &mut u32) {
    *a = a.wrapping_sub(*c); *a ^= rot(*c,  4); *c = c.wrapping_add(*b);
    *b = b.wrapping_sub(*a); *b ^= rot(*a,  6); *a = a.wrapping_add(*c);
    *c = c.wrapping_sub(*b); *c ^= rot(*b,  8); *b = b.wrapping_add(*a);
    *a = a.wrapping_sub(*c); *a ^= rot(*c, 16); *c = c.wrapping_add(*b);
    *b = b.wrapping_sub(*a); *b ^= rot(*a, 19); *a = a.wrapping_add(*c);
    *c = c.wrapping_sub(*b); *c ^= rot(*b,  4); *b = b.wrapping_add(*a);
}

#[inline(always)]
fn final_mix(a: &mut u32, b: &mut u32, c: &mut u32) {
    *c ^= *b; *c = c.wrapping_sub(rot(*b, 14));
    *a ^= *c; *a = a.wrapping_sub(rot(*c, 11));
    *b ^= *a; *b = b.wrapping_sub(rot(*a, 25));
    *c ^= *b; *c = c.wrapping_sub(rot(*b, 16));
    *a ^= *c; *a = a.wrapping_sub(rot(*c,  4));
    *b ^= *a; *b = b.wrapping_sub(rot(*a, 14));
    *c ^= *b; *c = c.wrapping_sub(rot(*b, 24));
}

/// Mirror of `hash_bytes_uint32`. Returns the same u32 Postgres returns
/// (via `UInt32GetDatum` packaging on the C side; we return the bare u32
/// and the V1 wrapper packages it as a Datum via `encode_u32`).
#[inline]
pub fn hash_bytes_uint32(k: u32) -> u32 {
    let init: u32 = 0x9e3779b9_u32
        .wrapping_add(4)
        .wrapping_add(3923095);
    let mut a = init;
    let mut b = init;
    let mut c = init;
    a = a.wrapping_add(k);
    final_mix(&mut a, &mut b, &mut c);
    c
}

/// Mirror of `hash_bytes_uint32_extended`.
#[inline]
pub fn hash_bytes_uint32_extended(k: u32, seed: u64) -> u64 {
    let init: u32 = 0x9e3779b9_u32
        .wrapping_add(4)
        .wrapping_add(3923095);
    let mut a = init;
    let mut b = init;
    let mut c = init;
    if seed != 0 {
        a = a.wrapping_add((seed >> 32) as u32);
        b = b.wrapping_add(seed as u32);
        mix(&mut a, &mut b, &mut c);
    }
    a = a.wrapping_add(k);
    final_mix(&mut a, &mut b, &mut c);
    ((b as u64) << 32) | (c as u64)
}

/// Mirror of `hash_bytes` for 16-byte fixed-size data (UUID).
/// Specialized implementation that processes 16 bytes (one mix loop + tail).
///
/// # Safety
/// `k` must point to at least 16 bytes of readable memory.
#[inline]
pub unsafe fn hash_bytes_16(k: *const u8) -> u32 {
    let init: u32 = 0x9e3779b9_u32
        .wrapping_add(16)
        .wrapping_add(3923095);
    let mut a = init;
    let mut b = init;
    let mut c = init;

    // Process first 12 bytes (one full iteration of the main loop)
    // Read as little-endian u32s (aligned reads from the input bytes):
    let k0 = u32::from_le_bytes([*k, *k.add(1), *k.add(2), *k.add(3)]);
    let k4 = u32::from_le_bytes([*k.add(4), *k.add(5), *k.add(6), *k.add(7)]);
    let k8 = u32::from_le_bytes([*k.add(8), *k.add(9), *k.add(10), *k.add(11)]);

    a = a.wrapping_add(k0);
    b = b.wrapping_add(k4);
    c = c.wrapping_add(k8);
    mix(&mut a, &mut b, &mut c);

    // Handle the last 4 bytes
    let k12 = u32::from_le_bytes([*k.add(12), *k.add(13), *k.add(14), *k.add(15)]);
    a = a.wrapping_add(k12);

    final_mix(&mut a, &mut b, &mut c);
    c
}

/// Mirror of `hash_bytes_extended` for 16-byte fixed-size data (UUID).
/// Returns a u64 combining b and c from the final state.
///
/// # Safety
/// `k` must point to at least 16 bytes of readable memory.
#[inline]
pub unsafe fn hash_bytes_16_extended(k: *const u8, seed: u64) -> u64 {
    let init: u32 = 0x9e3779b9_u32
        .wrapping_add(16)
        .wrapping_add(3923095);
    let mut a = init;
    let mut b = init;
    let mut c = init;

    // Apply seed if non-zero
    if seed != 0 {
        a = a.wrapping_add((seed >> 32) as u32);
        b = b.wrapping_add(seed as u32);
        mix(&mut a, &mut b, &mut c);
    }

    // Process first 12 bytes
    let k0 = u32::from_le_bytes([*k, *k.add(1), *k.add(2), *k.add(3)]);
    let k4 = u32::from_le_bytes([*k.add(4), *k.add(5), *k.add(6), *k.add(7)]);
    let k8 = u32::from_le_bytes([*k.add(8), *k.add(9), *k.add(10), *k.add(11)]);

    a = a.wrapping_add(k0);
    b = b.wrapping_add(k4);
    c = c.wrapping_add(k8);
    mix(&mut a, &mut b, &mut c);

    // Handle the last 4 bytes
    let k12 = u32::from_le_bytes([*k.add(12), *k.add(13), *k.add(14), *k.add(15)]);
    a = a.wrapping_add(k12);

    final_mix(&mut a, &mut b, &mut c);
    ((b as u64) << 32) | (c as u64)
}

/// Mirror of `hash_bytes` for 6-byte fixed-size data (macaddr).
/// Specialized implementation that processes 6 bytes.
///
/// # Safety
/// `k` must point to at least 6 bytes of readable memory.
#[inline]
pub unsafe fn hash_bytes_6(k: *const u8) -> u32 {
    let init: u32 = 0x9e3779b9_u32
        .wrapping_add(6)
        .wrapping_add(3923095);
    let mut a = init;
    let mut b = init;
    let mut c = init;

    // Process first 4 bytes
    let k0 = u32::from_le_bytes([*k, *k.add(1), *k.add(2), *k.add(3)]);
    a = a.wrapping_add(k0);

    // Handle the last 2 bytes
    let k4 = (*k.add(4) as u32) | ((*k.add(5) as u32) << 8);
    b = b.wrapping_add(k4);

    final_mix(&mut a, &mut b, &mut c);
    c
}

/// Mirror of `hash_bytes_extended` for 6-byte fixed-size data (macaddr).
/// Returns a u64 combining b and c from the final state.
///
/// # Safety
/// `k` must point to at least 6 bytes of readable memory.
#[inline]
pub unsafe fn hash_bytes_6_extended(k: *const u8, seed: u64) -> u64 {
    let init: u32 = 0x9e3779b9_u32
        .wrapping_add(6)
        .wrapping_add(3923095);
    let mut a = init;
    let mut b = init;
    let mut c = init;

    // Apply seed if non-zero
    if seed != 0 {
        a = a.wrapping_add((seed >> 32) as u32);
        b = b.wrapping_add(seed as u32);
        mix(&mut a, &mut b, &mut c);
    }

    // Process first 4 bytes
    let k0 = u32::from_le_bytes([*k, *k.add(1), *k.add(2), *k.add(3)]);
    a = a.wrapping_add(k0);

    // Handle the last 2 bytes
    let k4 = (*k.add(4) as u32) | ((*k.add(5) as u32) << 8);
    b = b.wrapping_add(k4);

    final_mix(&mut a, &mut b, &mut c);
    ((b as u64) << 32) | (c as u64)
}

#[inline]
pub fn decode_f32(d: Datum) -> f32 {
    f32::from_bits(d as u32)
}

#[inline]
pub fn decode_f64(d: Datum) -> f64 {
    f64::from_bits(d)
}

// (pgcmp_* helpers below this section handle NaN-aware float comparisons.)

#[inline]
pub fn decode_ptr<T>(d: Datum) -> *const T {
    d as usize as *const T
}

#[inline]
pub fn decode_mut_ptr<T>(d: Datum) -> *mut T {
    d as usize as *mut T
}

// ─── build_fcinfo! macro ──────────────────────────────────────────────────────

/// Construct a `FunctionCallInfoBaseData` on the caller's stack with
/// the given typed args. Each arg is encoded via the matching
/// `encode_*` helper; the macro infers from the type tag.
///
/// Usage:
/// ```ignore
/// let mut fcinfo = build_fcinfo!(args = [(i32, 42), (bool, true), (f64, 3.14)]);
/// ```
///
/// Expands to roughly:
/// ```ignore
/// let mut fcinfo = FunctionCallInfoBaseData::default();
/// fcinfo.nargs = 3;
/// fcinfo.args[0] = NullableDatum::val(encode_i32(42));
/// fcinfo.args[1] = NullableDatum::val(encode_bool(true));
/// fcinfo.args[2] = NullableDatum::val(encode_f64(3.14));
/// ```
///
/// For pointer args use the `ptr` / `mut_ptr` tags:
/// ```ignore
/// let entry: *mut MyStruct = ...;
/// let fcinfo = build_fcinfo!(args = [(mut_ptr, entry)]);
/// ```
///
/// For NULL args use the `null` tag (the value is ignored):
/// ```ignore
/// let fcinfo = build_fcinfo!(args = [(null, ()), (i32, 5)]);
/// ```
#[macro_export]
macro_rules! build_fcinfo {
    (args = [ $( ($tag:tt, $val:expr) ),* $(,)? ]) => {{
        let mut __fcinfo = $crate::FunctionCallInfoBaseData::default();
        let mut __i: usize = 0;
        $(
            __fcinfo.args[__i] = $crate::build_fcinfo!(@encode $tag, $val);
            __i += 1;
        )*
        __fcinfo.nargs = __i as i16;
        __fcinfo
    }};

    (@encode i8,      $v:expr) => { $crate::NullableDatum::val($crate::encode_i8($v))      };
    (@encode u8,      $v:expr) => { $crate::NullableDatum::val($crate::encode_u8($v))      };
    (@encode i16,     $v:expr) => { $crate::NullableDatum::val($crate::encode_i16($v))     };
    (@encode i32,     $v:expr) => { $crate::NullableDatum::val($crate::encode_i32($v))     };
    (@encode i64,     $v:expr) => { $crate::NullableDatum::val($crate::encode_i64($v))     };
    (@encode u32,     $v:expr) => { $crate::NullableDatum::val($crate::encode_u32($v))     };
    (@encode u64,     $v:expr) => { $crate::NullableDatum::val($crate::encode_u64($v))     };
    (@encode bool,    $v:expr) => { $crate::NullableDatum::val($crate::encode_bool($v))    };
    (@encode f32,     $v:expr) => { $crate::NullableDatum::val($crate::encode_f32($v))     };
    (@encode f64,     $v:expr) => { $crate::NullableDatum::val($crate::encode_f64($v))     };
    (@encode ptr,     $v:expr) => { $crate::NullableDatum::val($crate::encode_ptr($v))     };
    (@encode mut_ptr, $v:expr) => { $crate::NullableDatum::val($crate::encode_mut_ptr($v)) };
    (@encode null,    $v:expr) => {{ let _ = $v; $crate::NullableDatum::null() }};
}

// ─── ABI sanity check ─────────────────────────────────────────────────────────
//
// The C-side probe in `c_oracle/abi_probe.c` writes a few canonical
// offsets/sizes into a struct that this Rust crate reads at test time.
// If the layouts disagree we fail loudly.

extern "C" {
    fn pg_fcinfo_probe_sizeof_nullable_datum() -> usize;
    fn pg_fcinfo_probe_sizeof_fcinfo_base() -> usize;
    fn pg_fcinfo_probe_offsetof_nullable_value() -> usize;
    fn pg_fcinfo_probe_offsetof_nullable_isnull() -> usize;
    fn pg_fcinfo_probe_offsetof_fcinfo_args() -> usize;
    fn pg_fcinfo_probe_offsetof_fcinfo_nargs() -> usize;
    fn pg_fcinfo_probe_offsetof_fcinfo_isnull() -> usize;
}

/// Verify that the Rust mirrors here match Postgres' canonical layout.
/// Returns `Err(message)` if any size or offset disagrees. Run as a
/// startup check in any consumer that links the C-side oracle.
///
/// Note: the C-side `FunctionCallInfoBaseData` is a flexible-array
/// type (the `args[]` member has unspecified size at compile time).
/// The probe reports `sizeof` of the BASE struct (everything up to
/// and including the start of `args[]`), not the full sized variant.
/// We compare that against the offset of `args` in the Rust mirror,
/// which is the equivalent fixed-prefix size.
pub fn verify_abi() -> Result<(), String> {
    // FlexArray-base size = offset of args[] in the C struct.
    let c_base = unsafe { pg_fcinfo_probe_sizeof_fcinfo_base() };
    let r_args_off = std::mem::offset_of!(FunctionCallInfoBaseData, args);
    if c_base != r_args_off {
        return Err(format!(
            "FunctionCallInfoBaseData base size mismatch: C={c_base} Rust offset_of!(args)={r_args_off}"
        ));
    }
    let c_nd = unsafe { pg_fcinfo_probe_sizeof_nullable_datum() };
    let r_nd = std::mem::size_of::<NullableDatum>();
    if c_nd != r_nd {
        return Err(format!("sizeof(NullableDatum) mismatch: C={c_nd} Rust={r_nd}"));
    }
    let c_v = unsafe { pg_fcinfo_probe_offsetof_nullable_value() };
    let r_v = std::mem::offset_of!(NullableDatum, value);
    if c_v != r_v {
        return Err(format!("NullableDatum.value offset mismatch: C={c_v} Rust={r_v}"));
    }
    let c_inull = unsafe { pg_fcinfo_probe_offsetof_nullable_isnull() };
    let r_inull = std::mem::offset_of!(NullableDatum, isnull);
    if c_inull != r_inull {
        return Err(format!("NullableDatum.isnull offset mismatch: C={c_inull} Rust={r_inull}"));
    }
    let c_args = unsafe { pg_fcinfo_probe_offsetof_fcinfo_args() };
    if c_args != r_args_off {
        return Err(format!(
            "FunctionCallInfoBaseData.args offset mismatch: C={c_args} Rust={r_args_off}"
        ));
    }
    let c_n = unsafe { pg_fcinfo_probe_offsetof_fcinfo_nargs() };
    let r_n = std::mem::offset_of!(FunctionCallInfoBaseData, nargs);
    if c_n != r_n {
        return Err(format!("fcinfo.nargs offset mismatch: C={c_n} Rust={r_n}"));
    }
    let c_isn = unsafe { pg_fcinfo_probe_offsetof_fcinfo_isnull() };
    let r_isn = std::mem::offset_of!(FunctionCallInfoBaseData, isnull);
    if c_isn != r_isn {
        return Err(format!("fcinfo.isnull offset mismatch: C={c_isn} Rust={r_isn}"));
    }
    Ok(())
}

// ─── ereport mirror ─────────────────────────────────────────────────────────
//
// Postgres' V1 fmgr bodies signal errors via `ereport(ERROR, ...)`, which
// expands to a longjmp out of the surrounding setjmp set up by the
// transaction. Our diff-test harness needs to verify that BOTH the Rust
// translation AND the vendored real-Postgres C body either succeed with
// the same Datum, or both error. So we model Postgres' error path with
// a thread-local flag:
//
//   - The Rust V1 fmgr body sets the flag via `pg_ereport_*()` helpers
//     and returns 0 (the Datum value is meaningless after an error).
//   - The C oracle's vendored `ereport(...)` macro overrides expand to a
//     call into `pg_fcinfo_set_error()` followed by a longjmp out to the
//     surrounding `setjmp` in the `c_*` wrapper (see c_oracle_ereport.h
//     in the cmp / hash / arith crates). The longjmp is C-only (Rust
//     drops are not skipped); the Rust side never longjmps.
//   - The diff-test harness clears the flag, invokes one side, captures
//     `fmgr_take_error()`, repeats for the other, and asserts that the
//     error states match. On success-success, it also asserts Datum
//     equality.
//
// Why a TLS flag and not panic / Result-based: V1 fmgr's ABI is
// `fn(*mut FunctionCallInfoBaseData) -> Datum` — no in-band error
// channel. The flag preserves the ABI exactly while letting the
// caller observe whether the call errored.

use std::cell::Cell;

/// Kind of ereport-style error a V1 fmgr body raised. The exhaustive
/// SQLSTATE list is intentionally narrower than Postgres' — the diff-test
/// only needs to distinguish "what kind of error" classes that appear in
/// the lifted clusters. Extend as new clusters land.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum FmgrErrorKind {
    /// SQLSTATE 22003 — Postgres' `ERRCODE_NUMERIC_VALUE_OUT_OF_RANGE`.
    /// Raised by integer arithmetic ops on overflow ("integer out of
    /// range"), by date arithmetic on infinite-date edge cases, etc.
    NumericValueOutOfRange  = 1,
    /// SQLSTATE 22012 — `ERRCODE_DIVISION_BY_ZERO`. Raised by
    /// int{2,4,8}div on b == 0.
    DivisionByZero          = 2,
    /// SQLSTATE 22008 — `ERRCODE_DATETIME_VALUE_OUT_OF_RANGE`. Raised
    /// by interval / date / timestamp arithmetic on overflow or on
    /// "infinity − infinity" edge cases.
    DatetimeValueOutOfRange = 3,
}

impl FmgrErrorKind {
    fn from_code(code: u32) -> Self {
        match code {
            1 => Self::NumericValueOutOfRange,
            2 => Self::DivisionByZero,
            3 => Self::DatetimeValueOutOfRange,
            _ => Self::NumericValueOutOfRange, // future-proof
        }
    }
}

thread_local! {
    static FMGR_ERROR: Cell<Option<FmgrErrorKind>> = const { Cell::new(None) };
}

/// Clear the per-thread fmgr-error flag. Call before invoking a V1
/// fmgr function in the diff-test harness.
#[inline]
pub fn fmgr_clear_error() {
    FMGR_ERROR.with(|c| c.set(None));
}

/// Take (consume) the per-thread fmgr-error flag. Returns whatever
/// error a V1 fmgr body raised since the last `fmgr_clear_error()` (or
/// `None` if none).
#[inline]
pub fn fmgr_take_error() -> Option<FmgrErrorKind> {
    FMGR_ERROR.with(|c| c.take())
}

/// Peek at the per-thread fmgr-error flag without clearing it.
#[inline]
pub fn fmgr_peek_error() -> Option<FmgrErrorKind> {
    FMGR_ERROR.with(|c| c.get())
}

/// Equivalent of `ereport(ERROR, errcode(ERRCODE_NUMERIC_VALUE_OUT_OF_RANGE),
/// errmsg("integer out of range"))`. Sets the per-thread flag and
/// returns; the caller (the V1 fmgr body itself) must `return 0` after.
#[inline]
pub fn pg_ereport_numeric_value_out_of_range() {
    FMGR_ERROR.with(|c| c.set(Some(FmgrErrorKind::NumericValueOutOfRange)));
}

/// Equivalent of `ereport(ERROR, errcode(ERRCODE_DIVISION_BY_ZERO),
/// errmsg("division by zero"))`.
#[inline]
pub fn pg_ereport_division_by_zero() {
    FMGR_ERROR.with(|c| c.set(Some(FmgrErrorKind::DivisionByZero)));
}

/// Equivalent of `ereport(ERROR, errcode(ERRCODE_DATETIME_VALUE_OUT_OF_RANGE),
/// errmsg("interval out of range"))` (or similar — date/timestamp/
/// interval arithmetic all share this errcode).
#[inline]
pub fn pg_ereport_datetime_value_out_of_range() {
    FMGR_ERROR.with(|c| c.set(Some(FmgrErrorKind::DatetimeValueOutOfRange)));
}

/// C-callable entry point. The C oracle's `ereport(...)` macro
/// override expands into a call to this, followed by a `longjmp` back
/// to the wrapper's `setjmp`. `kind` is the discriminant of
/// `FmgrErrorKind`.
#[no_mangle]
pub extern "C" fn pg_fcinfo_set_error(kind: u32) {
    FMGR_ERROR.with(|c| c.set(Some(FmgrErrorKind::from_code(kind))));
}

// ─── Overflow-checked arithmetic primitives (Postgres' common/int.h) ───────
//
// One-for-one mirror of `pg_{add,sub,mul}_s{16,32,64}_overflow` from
// `src/include/common/int.h`. Returns true on overflow, false on
// success (with `*out` written). Implemented via Rust's `checked_*`,
// which lowers to LLVM's `*_with_overflow` intrinsics — same code that
// `__builtin_{add,sub,mul}_overflow` lowers to on the Postgres side.
//
// Signature mirrors the C side exactly (`a, b, &mut out`) so the
// emitted Rust V1 fmgr body retains the same statement structure as
// real Postgres' int.c — useful for the AST grounding gate later.

macro_rules! impl_overflow {
    ($add:ident, $sub:ident, $mul:ident, $ty:ty) => {
        #[inline] pub fn $add(a: $ty, b: $ty, out: &mut $ty) -> bool {
            match a.checked_add(b) { Some(v) => { *out = v; false } None => true }
        }
        #[inline] pub fn $sub(a: $ty, b: $ty, out: &mut $ty) -> bool {
            match a.checked_sub(b) { Some(v) => { *out = v; false } None => true }
        }
        #[inline] pub fn $mul(a: $ty, b: $ty, out: &mut $ty) -> bool {
            match a.checked_mul(b) { Some(v) => { *out = v; false } None => true }
        }
    };
}
impl_overflow!(pg_add_s16_overflow, pg_sub_s16_overflow, pg_mul_s16_overflow, i16);
impl_overflow!(pg_add_s32_overflow, pg_sub_s32_overflow, pg_mul_s32_overflow, i32);
impl_overflow!(pg_add_s64_overflow, pg_sub_s64_overflow, pg_mul_s64_overflow, i64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip_scalars() {
        for v in [i32::MIN, -1, 0, 1, 42, i32::MAX] {
            assert_eq!(decode_i32(encode_i32(v)), v);
        }
        for v in [i64::MIN, -1, 0, 1, 42, i64::MAX] {
            assert_eq!(decode_i64(encode_i64(v)), v);
        }
        for v in [i16::MIN, -1, 0, 1, 42, i16::MAX] {
            assert_eq!(decode_i16(encode_i16(v)), v);
        }
        assert!(decode_bool(encode_bool(true)));
        assert!(!decode_bool(encode_bool(false)));
        for v in [0.0_f32, 1.0, -1.0, f32::INFINITY, f32::NAN, f32::MIN, f32::MAX] {
            let r = decode_f32(encode_f32(v));
            if v.is_nan() { assert!(r.is_nan()); } else { assert_eq!(r, v); }
        }
        for v in [0.0_f64, 1.0, -1.0, f64::INFINITY, f64::NAN, f64::MIN, f64::MAX] {
            let r = decode_f64(encode_f64(v));
            if v.is_nan() { assert!(r.is_nan()); } else { assert_eq!(r, v); }
        }
    }

    #[test]
    fn encode_decode_ptr_roundtrip() {
        let v: u32 = 0xdead_beef;
        let p = &v as *const u32;
        let d = encode_ptr(p);
        let p2: *const u32 = decode_ptr(d);
        assert_eq!(p, p2);
        unsafe { assert_eq!(*p2, 0xdead_beef); }
    }

    #[test]
    fn build_fcinfo_macro_assembles_args() {
        let fcinfo = build_fcinfo!(args = [
            (i32, 42),
            (bool, true),
            (f64, 3.14),
        ]);
        assert_eq!(fcinfo.nargs, 3);
        assert_eq!(decode_i32(fcinfo.args[0].value), 42);
        assert_eq!(decode_bool(fcinfo.args[1].value), true);
        assert_eq!(decode_f64(fcinfo.args[2].value), 3.14);
        assert!(!fcinfo.args[0].isnull);
        assert!(!fcinfo.args[1].isnull);
        assert!(!fcinfo.args[2].isnull);
    }

    #[test]
    fn build_fcinfo_macro_handles_null() {
        let fcinfo = build_fcinfo!(args = [
            (null, ()),
            (i32, 7),
        ]);
        assert_eq!(fcinfo.nargs, 2);
        assert!(fcinfo.args[0].isnull);
        assert!(!fcinfo.args[1].isnull);
        assert_eq!(decode_i32(fcinfo.args[1].value), 7);
    }

    #[test]
    fn build_fcinfo_macro_empty_args() {
        let fcinfo = build_fcinfo!(args = []);
        assert_eq!(fcinfo.nargs, 0);
    }

    #[test]
    fn nullable_datum_helpers() {
        let n = NullableDatum::val(42);
        assert_eq!(n.value, 42);
        assert!(!n.isnull);
        let nn = NullableDatum::null();
        assert!(nn.isnull);
    }

    #[test]
    fn abi_layout_matches_postgres() {
        verify_abi().expect("ABI mismatch — check that the Rust mirrors match Postgres' fmgr.h");
    }

    #[test]
    fn fmgr_error_flag_round_trip() {
        fmgr_clear_error();
        assert_eq!(fmgr_peek_error(), None);
        pg_ereport_numeric_value_out_of_range();
        assert_eq!(fmgr_peek_error(), Some(FmgrErrorKind::NumericValueOutOfRange));
        assert_eq!(fmgr_take_error(), Some(FmgrErrorKind::NumericValueOutOfRange));
        assert_eq!(fmgr_peek_error(), None);

        pg_fcinfo_set_error(FmgrErrorKind::DivisionByZero as u32);
        assert_eq!(fmgr_take_error(), Some(FmgrErrorKind::DivisionByZero));
    }

    #[test]
    fn interval_layout_matches_postgres() {
        assert_eq!(std::mem::size_of::<Interval>(), SIZEOF_INTERVAL);
        assert_eq!(std::mem::size_of::<Interval>(), 16);
        assert_eq!(std::mem::align_of::<Interval>(), 8);
        // Field offsets must match Postgres' Interval struct.
        assert_eq!(std::mem::offset_of!(Interval, time),  0);
        assert_eq!(std::mem::offset_of!(Interval, day),   8);
        assert_eq!(std::mem::offset_of!(Interval, month), 12);
    }

    #[test]
    fn interval_sentinel_round_trip() {
        let mut a = Interval { time: 0, day: 0, month: 0 };
        assert!(!interval_is_nobegin(&a) && !interval_is_noend(&a));
        assert!(!interval_not_finite(&a));
        interval_nobegin(&mut a);
        assert!(interval_is_nobegin(&a) && !interval_is_noend(&a));
        assert!(interval_not_finite(&a));
        let mut b = Interval { time: 0, day: 0, month: 0 };
        interval_noend(&mut b);
        assert!(!interval_is_nobegin(&b) && interval_is_noend(&b));
    }

    #[test]
    fn interval_datum_codec_round_trip() {
        let mut iv = Interval { time: 123_456, day: 7, month: -2 };
        let d = encode_interval_p(&mut iv);
        let back = unsafe { &*decode_interval_p(d) };
        assert_eq!(back.time,  123_456);
        assert_eq!(back.day,   7);
        assert_eq!(back.month, -2);
    }

    #[test]
    fn varlena_4b_header_round_trip() {
        // Construct a varlena "Hello" (payload 5 bytes, total 9 bytes).
        let mut buf = [0u8; 16];
        unsafe {
            set_varsize_4b(buf.as_mut_ptr(), 9);
            let payload = vardata_4b_mut(buf.as_mut_ptr());
            payload.copy_from(b"Hello".as_ptr(), 5);
            assert_eq!(varsize_4b(buf.as_ptr()), 9);
            // Header bytes: 9 << 2 = 36 = 0x24, little-endian.
            assert_eq!(buf[0], 0x24);
            assert_eq!(buf[1], 0x00);
            assert_eq!(buf[2], 0x00);
            assert_eq!(buf[3], 0x00);
            // Payload bytes follow.
            assert_eq!(&buf[4..9], b"Hello");
        }
    }

    #[test]
    fn varsize_any_exhdr_handles_4b_header() {
        let mut buf = [0u8; 8];
        unsafe {
            set_varsize_4b(buf.as_mut_ptr(), 7);  // 3-byte payload
            assert_eq!(varsize_any_exhdr(buf.as_ptr()), 3);
            assert_eq!(vardata_any(buf.as_ptr()), vardata_4b(buf.as_ptr()));
        }
    }

    #[test]
    fn bytea_datum_codec_round_trip() {
        let mut buf = [0u8; 8];
        let d = encode_bytea_p(buf.as_mut_ptr());
        let back = unsafe { decode_bytea_p(d) };
        assert_eq!(back as usize, buf.as_ptr() as usize);
    }

    #[test]
    fn fmgr_error_datetime_kind_round_trips() {
        fmgr_clear_error();
        pg_ereport_datetime_value_out_of_range();
        assert_eq!(fmgr_take_error(), Some(FmgrErrorKind::DatetimeValueOutOfRange));
        // C-side path (the C oracle calls pg_fcinfo_set_error with the
        // discriminant value) — must round-trip the same way.
        pg_fcinfo_set_error(FmgrErrorKind::DatetimeValueOutOfRange as u32);
        assert_eq!(fmgr_take_error(), Some(FmgrErrorKind::DatetimeValueOutOfRange));
    }

    #[test]
    fn overflow_primitives_basic() {
        let mut out = 0i32;
        assert!(!pg_add_s32_overflow(1, 2, &mut out));
        assert_eq!(out, 3);
        assert!(pg_add_s32_overflow(i32::MAX, 1, &mut out));
        assert!(!pg_sub_s32_overflow(0, i32::MAX, &mut out));
        assert_eq!(out, -i32::MAX);
        assert!(pg_sub_s32_overflow(i32::MIN, 1, &mut out));
        assert!(pg_mul_s32_overflow(i32::MAX, 2, &mut out));
    }
}
