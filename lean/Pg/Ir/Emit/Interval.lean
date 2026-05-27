import Pg.Ir.Emit.IntervalCommon

/-!
# Pg.Ir.Emit.Interval — Rust emit for the interval-arithmetic cluster.

Family-specific emit patterns:
  - `unaryNegate`: emits `<fn>_internal` helper + fmgr stub that delegates to it.
  - `binaryPl`, `binaryMi`: NO internal helper; dispatch goes directly in fmgr stub,
    which calls the `finite_*` delegates for the finite case.

This mirrors Postgres' own factoring: interval_um has a separate _internal
helper, but interval_pl/mi only have finite_* delegates.
-/

namespace Pg.Ir.Emit.Interval.Rust

open Pg.Ir.Emit.Interval (IntervalBody IntervalFamily families)

/-- Render the `_internal` helper body for unaryNegate. The helper
returns `true` on overflow/error, `false` on success. -/
def renderUnaryNegateBody : String :=
  "    if interval_is_nobegin(interval) {\n" ++
  "        interval_noend(result);\n" ++
  "        false\n" ++
  "    } else if interval_is_noend(interval) {\n" ++
  "        interval_nobegin(result);\n" ++
  "        false\n" ++
  "    } else {\n" ++
  "        // Negate each field, guarding against overflow.\n" ++
  "        let o1 = pg_sub_s64_overflow(0, interval.time,  &mut result.time);\n" ++
  "        let o2 = pg_sub_s32_overflow(0, interval.day,   &mut result.day);\n" ++
  "        let o3 = pg_sub_s32_overflow(0, interval.month, &mut result.month);\n" ++
  "        o1 || o2 || o3 || interval_not_finite(result)\n" ++
  "    }\n"

/-- Render the `<fn>_internal` Rust function for unaryNegate. -/
def renderUnaryNegateInternal (fam : IntervalFamily) : String :=
  "unsafe fn " ++ fam.fnName ++ "_internal(interval: &Interval, result: &mut Interval) -> bool {\n" ++
  renderUnaryNegateBody ++
  "}\n"

/-- Check if the body has a separate internal helper (only unaryNegate does). -/
def hasInternalHelper (body : IntervalBody) : Bool :=
  match body with
  | .unaryNegate => true
  | .binaryPl | .binaryMi => false

/-- Render the `<fn>_internal` Rust function if needed. -/
def renderInternal (fam : IntervalFamily) : String :=
  match fam.body with
  | .unaryNegate => renderUnaryNegateInternal fam
  | _ => "" -- binaryPl and binaryMi don't have separate internal helpers

/-- Render the fmgr stub for unaryNegate: delegates to _internal. -/
def renderUnaryNegateFmgrStub (fam : IntervalFamily) : String :=
  "#[no_mangle]\n" ++
  "pub unsafe extern \"C\" fn " ++ fam.fnName ++
    "(fcinfo: *mut FunctionCallInfoBaseData) -> u64 {\n" ++
  "    let f = &mut *fcinfo;\n" ++
  "    let interval = &*decode_interval_p(f.args[0].value);\n" ++
  "    let result_ptr = palloc(SIZEOF_INTERVAL) as *mut Interval;\n" ++
  "    if " ++ fam.fnName ++ "_internal(interval, &mut *result_ptr) {\n" ++
  "        pg_ereport_datetime_value_out_of_range();\n" ++
  "        return 0;\n" ++
  "    }\n" ++
  "    encode_interval_p(result_ptr)\n" ++
  "}\n"

/-- Render the fmgr stub for binaryPl: inline dispatch with finite_interval_pl delegate. -/
def renderBinaryPlFmgrStub (fam : IntervalFamily) : String :=
  "#[no_mangle]\n" ++
  "pub unsafe extern \"C\" fn " ++ fam.fnName ++
    "(fcinfo: *mut FunctionCallInfoBaseData) -> u64 {\n" ++
  "    let f = &mut *fcinfo;\n" ++
  "    let span1 = &*decode_interval_p(f.args[0].value);\n" ++
  "    let span2 = &*decode_interval_p(f.args[1].value);\n" ++
  "    let result_ptr = palloc(SIZEOF_INTERVAL) as *mut Interval;\n" ++
  "    if interval_is_nobegin(span1) {\n" ++
  "        if interval_is_noend(span2) {\n" ++
  "            pg_ereport_datetime_value_out_of_range();\n" ++
  "            return 0;\n" ++
  "        }\n" ++
  "        interval_nobegin(&mut *result_ptr);\n" ++
  "    } else if interval_is_noend(span1) {\n" ++
  "        if interval_is_nobegin(span2) {\n" ++
  "            pg_ereport_datetime_value_out_of_range();\n" ++
  "            return 0;\n" ++
  "        }\n" ++
  "        interval_noend(&mut *result_ptr);\n" ++
  "    } else if interval_not_finite(span2) {\n" ++
  "        (*result_ptr).time = span2.time;\n" ++
  "        (*result_ptr).day = span2.day;\n" ++
  "        (*result_ptr).month = span2.month;\n" ++
  "    } else {\n" ++
  "        if finite_interval_pl(span1, span2, &mut *result_ptr) {\n" ++
  "            pg_ereport_datetime_value_out_of_range();\n" ++
  "            return 0;\n" ++
  "        }\n" ++
  "    }\n" ++
  "    encode_interval_p(result_ptr)\n" ++
  "}\n"

/-- Render the fmgr stub for binaryMi: inline dispatch with finite_interval_mi delegate. -/
def renderBinaryMiFmgrStub (fam : IntervalFamily) : String :=
  "#[no_mangle]\n" ++
  "pub unsafe extern \"C\" fn " ++ fam.fnName ++
    "(fcinfo: *mut FunctionCallInfoBaseData) -> u64 {\n" ++
  "    let f = &mut *fcinfo;\n" ++
  "    let span1 = &*decode_interval_p(f.args[0].value);\n" ++
  "    let span2 = &*decode_interval_p(f.args[1].value);\n" ++
  "    let result_ptr = palloc(SIZEOF_INTERVAL) as *mut Interval;\n" ++
  "    if interval_is_nobegin(span1) {\n" ++
  "        if interval_is_nobegin(span2) {\n" ++
  "            pg_ereport_datetime_value_out_of_range();\n" ++
  "            return 0;\n" ++
  "        }\n" ++
  "        interval_nobegin(&mut *result_ptr);\n" ++
  "    } else if interval_is_noend(span1) {\n" ++
  "        if interval_is_noend(span2) {\n" ++
  "            pg_ereport_datetime_value_out_of_range();\n" ++
  "            return 0;\n" ++
  "        }\n" ++
  "        interval_noend(&mut *result_ptr);\n" ++
  "    } else if interval_is_nobegin(span2) {\n" ++
  "        interval_noend(&mut *result_ptr);\n" ++
  "    } else if interval_is_noend(span2) {\n" ++
  "        interval_nobegin(&mut *result_ptr);\n" ++
  "    } else {\n" ++
  "        if finite_interval_mi(span1, span2, &mut *result_ptr) {\n" ++
  "            pg_ereport_datetime_value_out_of_range();\n" ++
  "            return 0;\n" ++
  "        }\n" ++
  "    }\n" ++
  "    encode_interval_p(result_ptr)\n" ++
  "}\n"

/-- Render the fmgr stub for the given family. -/
def renderFmgrStub (fam : IntervalFamily) : String :=
  match fam.body with
  | .unaryNegate => renderUnaryNegateFmgrStub fam
  | .binaryPl => renderBinaryPlFmgrStub fam
  | .binaryMi => renderBinaryMiFmgrStub fam

def rustSource : String :=
  let header :=
    "// Generated by `rules_postgres/lean/Pg/Ir/Emit/Interval.lean`.\n" ++
    "// DO NOT EDIT BY HAND. Regenerate via\n" ++
    "// `rules_postgres/tools/regen/regen-interval.sh`.\n" ++
    "//\n" ++
    "// First palloc-using Pg.Ir cluster. Each family emits a pair:\n" ++
    "//   1. A `<fn>_internal` Rust fn (the actual algorithm; returns\n" ++
    "//      `true` on overflow, writes its &mut out-param).\n" ++
    "//   2. The V1 fmgr stub `<fn>` (palloc + delegate + return).\n" ++
    "//\n" ++
    "// On overflow / infinity-edge errors, both sides set the pg_fcinfo\n" ++
    "// TLS flag (FmgrErrorKind::DatetimeValueOutOfRange) and the fmgr\n" ++
    "// stub returns Datum 0. The diff-test harness compares the\n" ++
    "// (out_datum, err_state) tuple and (on success) the palloc'd\n" ++
    "// Interval's field contents.\n" ++
    "\n" ++
    "#![allow(clippy::missing_safety_doc)]\n" ++
    "\n" ++
    "use pg_fcinfo::{\n" ++
    "    decode_interval_p, encode_interval_p,\n" ++
    "    interval_is_nobegin, interval_is_noend, interval_nobegin, interval_noend,\n" ++
    "    interval_not_finite,\n" ++
    "    pg_ereport_datetime_value_out_of_range,\n" ++
    "    pg_add_s32_overflow, pg_add_s64_overflow,\n" ++
    "    pg_sub_s32_overflow, pg_sub_s64_overflow,\n" ++
    "    FunctionCallInfoBaseData, Interval, SIZEOF_INTERVAL,\n" ++
    "};\n" ++
    "use pg_palloc::palloc;\n" ++
    "\n" ++
    "// Internal helper for interval_pl: finite case.\n" ++
    "// Both span1 and span2 are guaranteed not to be infinity sentinels.\n" ++
    "// Returns true on overflow, false on success (writes result in-place).\n" ++
    "unsafe fn finite_interval_pl(span1: &Interval, span2: &Interval, result: &mut Interval) -> bool {\n" ++
    "    if pg_add_s32_overflow(span1.month, span2.month, &mut result.month) ||\n" ++
    "       pg_add_s32_overflow(span1.day,   span2.day,   &mut result.day) ||\n" ++
    "       pg_add_s64_overflow(span1.time,  span2.time,  &mut result.time) ||\n" ++
    "       interval_not_finite(result)\n" ++
    "    {\n" ++
    "        true\n" ++
    "    } else {\n" ++
    "        false\n" ++
    "    }\n" ++
    "}\n" ++
    "\n" ++
    "// Internal helper for interval_mi: finite case.\n" ++
    "// Both span1 and span2 are guaranteed not to be infinity sentinels.\n" ++
    "// Returns true on overflow, false on success (writes result in-place).\n" ++
    "unsafe fn finite_interval_mi(span1: &Interval, span2: &Interval, result: &mut Interval) -> bool {\n" ++
    "    if pg_sub_s32_overflow(span1.month, span2.month, &mut result.month) ||\n" ++
    "       pg_sub_s32_overflow(span1.day,   span2.day,   &mut result.day) ||\n" ++
    "       pg_sub_s64_overflow(span1.time,  span2.time,  &mut result.time) ||\n" ++
    "       interval_not_finite(result)\n" ++
    "    {\n" ++
    "        true\n" ++
    "    } else {\n" ++
    "        false\n" ++
    "    }\n" ++
    "}\n"
  let body :=
    Id.run do
      let mut s := ""
      for fam in families do
        s := s ++ "\n" ++ renderInternal fam ++ "\n" ++ renderFmgrStub fam
      return s
  header ++ body

end Pg.Ir.Emit.Interval.Rust

def main : IO Unit := IO.print Pg.Ir.Emit.Interval.Rust.rustSource
