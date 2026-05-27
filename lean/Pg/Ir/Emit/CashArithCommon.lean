/-!
# Pg.Ir.Emit.CashArith — shared family table for the cash arithmetic
emit modules (`CashArith` for Rust, `CashArithC` for the AST-grounding C).

Cash is typedef'd as `int64` in Postgres. v1 scope: 6 functions =
{pl, mi, mul_int4, div_int4, larger, smaller}.

The Postgres sources follow two distinct shapes:

1. **Overflow-checked arithmetic** (cash_pl, cash_mi, cash_mul_int4):
```
T1 arg1 = PG_GETARG_T1(0);
T2 arg2 = PG_GETARG_T2(1);
Cash result;
if (unlikely(pg_<op>_s64_overflow(<cast>arg1, <cast>arg2, &result)))
    ereport(ERROR,
            (errcode(ERRCODE_NUMERIC_VALUE_OUT_OF_RANGE),
             errmsg("money out of range")));
PG_RETURN_CASH(result);
```

2. **Comparison-only** (cashlarger, cashsmaller):
```
Cash c1 = PG_GETARG_CASH(0);
Cash c2 = PG_GETARG_CASH(1);
Cash result;
result = (c1 > c2) ? c1 : c2;
PG_RETURN_CASH(result);
```

3. **Division with zero-check** (cash_div_int4):
```
Cash c = PG_GETARG_CASH(0);
int32 i = PG_GETARG_INT32(1);
if (unlikely(i == 0))
    ereport(ERROR,
            (errcode(ERRCODE_DIVISION_BY_ZERO),
             errmsg("division by zero")));
PG_RETURN_CASH(c / (int64) i);
```
-/

namespace Pg.Ir.Emit.CashArith

/-- The six cash arithmetic operations. -/
inductive CashOp where
  | pl              -- cash + cash
  | mi              -- cash - cash
  | mul_int4        -- cash × int4
  | div_int4        -- cash ÷ int4
  | larger          -- max(cash, cash)
  | smaller         -- min(cash, cash)
  deriving DecidableEq, Repr

/-- Postgres suffix on the function name. -/
def CashOp.suffix : CashOp → String
  | .pl        => "pl"
  | .mi        => "mi"
  | .mul_int4  => "mul_int4"
  | .div_int4  => "div_int4"
  | .larger    => "larger"
  | .smaller   => "smaller"

/-- Lowercase op name used to pick the `pg_<add|sub|mul>_s64_overflow` primitive
(only for pl, mi, mul_int4; others don't use overflow). -/
def CashOp.primitive : CashOp → String
  | .pl       => "add"
  | .mi       => "sub"
  | .mul_int4 => "mul"
  | _         => ""

/-- Body shape classifier. -/
inductive BodyShape where
  | binaryAddSub    -- cash_pl, cash_mi: overflow-checked add/sub
  | binaryMulInt    -- cash_mul_int4: overflow-checked multiply by int4
  | binaryDivInt    -- cash_div_int4: division-by-zero check, simple divide
  | binaryCompare   -- cashlarger, cashsmaller: ternary comparison, no overflow
  deriving DecidableEq, Repr

def CashOp.bodyShape : CashOp → BodyShape
  | .pl       => .binaryAddSub
  | .mi       => .binaryAddSub
  | .mul_int4 => .binaryMulInt
  | .div_int4 => .binaryDivInt
  | .larger   => .binaryCompare
  | .smaller  => .binaryCompare

structure CashFamily where
  /-- Postgres function name (e.g., "cash_pl", "cashlarger"). -/
  fnName      : String
  /-- The operation. -/
  op          : CashOp
  /-- The `pg_fcinfo::decode_*` helper name for arg[0] on the Rust side:
  always "decode_i64" for cash. -/
  decoder1    : String := "decode_i64"
  /-- The `pg_fcinfo::decode_*` helper name for arg[1]:
  "decode_i64" for cash, "decode_i32" for int4. -/
  decoder2    : String
  /-- The Rust type the body computes in (always i64 for cash result). -/
  rustResultTy : String := "i64"
  /-- The `pg_fcinfo::encode_*` helper name for the return value:
  always "encode_i64" for cash. -/
  rustEncoder  : String := "encode_i64"
  /-- Non-empty iff arg1 needs an explicit `as i64` widening cast
  (true for cash_mul_int4 where arg2 is i32). -/
  rustCast1   : String := ""
  /-- Non-empty iff arg2 needs an explicit `as i64` widening cast
  (true for cash_mul_int4 where arg2 is i32; false for cash ops where both
  are already i64). -/
  rustCast2   : String := ""
  /-- The `PG_GETARG_*` macro name for arg[0]: always PG_GETARG_CASH. -/
  pgGetArg1   : String := "PG_GETARG_CASH"
  /-- The `PG_GETARG_*` macro name for arg[1]:
  PG_GETARG_CASH for cash, PG_GETARG_INT32 for int4. -/
  pgGetArg2   : String
  /-- The `PG_RETURN_*` macro name: always PG_RETURN_CASH. -/
  pgReturn    : String := "PG_RETURN_CASH"
  /-- C type name for arg1 (always "Cash" for cash). -/
  cWidth1     : String := "Cash"
  /-- C type name for arg2 ("Cash" for cash, "int32" for int4). -/
  cWidth2     : String
  /-- C result type (always "Cash" / int64). -/
  cResultTy   : String := "Cash"
  /-- C-side cast string (with trailing space) applied to arg1 before
  the overflow primitive — typically empty for cash. -/
  cCast1      : String := ""
  /-- C-side cast string applied to arg2 (e.g., "(int64) " for int4). -/
  cCast2      : String := ""
  /-- Name of the static inline helper function (empty if no helper). -/
  helperName  : String := ""
  /-- The C signature of the helper's second parameter type (empty if no helper). -/
  helperArg2Type : String := ""

/-- The 6 operations. -/
def families : List CashFamily := [
  { fnName := "cash_pl", op := .pl,
    decoder2 := "decode_i64",
    pgGetArg2 := "PG_GETARG_CASH",
    cWidth2 := "Cash",
    helperName := "cash_pl_cash",
    helperArg2Type := "Cash" },
  { fnName := "cash_mi", op := .mi,
    decoder2 := "decode_i64",
    pgGetArg2 := "PG_GETARG_CASH",
    cWidth2 := "Cash",
    helperName := "cash_mi_cash",
    helperArg2Type := "Cash" },
  { fnName := "cash_mul_int4", op := .mul_int4,
    decoder2 := "decode_i32",
    rustCast2 := "yes",
    pgGetArg2 := "PG_GETARG_INT32",
    cWidth2 := "int32",
    cCast2 := "(int64) ",
    helperName := "cash_mul_int64",
    helperArg2Type := "int64" },
  { fnName := "cash_div_int4", op := .div_int4,
    decoder2 := "decode_i32",
    pgGetArg2 := "PG_GETARG_INT32",
    cWidth2 := "int32",
    helperName := "cash_div_int64",
    helperArg2Type := "int64" },
  { fnName := "cashlarger", op := .larger,
    decoder2 := "decode_i64",
    pgGetArg2 := "PG_GETARG_CASH",
    cWidth2 := "Cash" },
  { fnName := "cashsmaller", op := .smaller,
    decoder2 := "decode_i64",
    pgGetArg2 := "PG_GETARG_CASH",
    cWidth2 := "Cash" },
]

end Pg.Ir.Emit.CashArith
