/-!
# Pg.Ir.Emit.DateArithCommon — shared family table for the date arithmetic
emit modules (`DateArith` for Rust, `DateArithC` for the AST-grounding C).

v1 scope: 3 functions:
- `date_pli` (date + int32 → date)
- `date_mii` (date - int32 → date)
- `date_mi` (date - date → int32)

All functions decode DateADT (int32 encoding) and handle sentinels:
- DATEVAL_NOBEGIN (INT_MIN = -2147483648)
- DATEVAL_NOEND (INT_MAX = 2147483647)

Date arithmetic differs from int arithmetic in several ways:
1. Two of three functions return DateADT; one returns int32.
2. Both pli and mii check `IS_VALID_DATE(result)` after arithmetic.
3. Finite checks on infinity sentinels happen first; overflowing on infinite
   inputs is prevented by early returns.
4. The overflow detection for pli/mii uses comparison logic (checking sign
   reversal on wraparound), NOT the overflow primitives.

The Postgres sources follow this shape for pli/mii:

```
DateADT dateVal = PG_GETARG_DATEADT(0);
int32 days = PG_GETARG_INT32(1);
DateADT result;

if (DATE_NOT_FINITE(dateVal))
    PG_RETURN_DATEADT(dateVal);

result = dateVal ± days;

if ((days >= 0 ? (result <|> dateVal) : (result >|< dateVal)) ||
    !IS_VALID_DATE(result))
    ereport(ERROR, ...);

PG_RETURN_DATEADT(result);
```

And date_mi (date - date → int32):

```
DateADT dateVal1 = PG_GETARG_DATEADT(0);
DateADT dateVal2 = PG_GETARG_DATEADT(1);

if (DATE_NOT_FINITE(dateVal1) || DATE_NOT_FINITE(dateVal2))
    ereport(ERROR, ...);

PG_RETURN_INT32((int32) (dateVal1 - dateVal2));
```
-/

namespace Pg.Ir.Emit.DateArith

/-- The three date-arithmetic ops.
- `pli`: date + int → date
- `mii`: date - int → date
- `mi`: date - date → int
-/
inductive DateOp where
  | pli
  | mii
  | mi

/-- Postgres function-name suffix. -/
def DateOp.suffix : DateOp → String
  | .pli => "pli"
  | .mii => "mii"
  | .mi  => "mi"

/-- C overflow primitive shape — note that date arithmetic does NOT use
pg_add/pg_sub_overflow; instead it uses wraparound-detection logic. -/
def DateOp.rustFnBody : DateOp → String
  | .pli => "date_pli_body"
  | .mii => "date_mii_body"
  | .mi  => "date_mi_body"

/-- Lean spec: one family, three ops. -/
def ops : List DateOp := [.pli, .mii, .mi]

/-- For date_pli and date_mii: result is DateADT (int32).
For date_mi: result is int32 (same encoding as DateADT, but semantically
an elapsed day count). -/
def DateOp.rustResultTy : DateOp → String
  | .pli => "i32"
  | .mii => "i32"
  | .mi  => "i32"

def DateOp.rustEncoder : DateOp → String
  | .pli => "encode_i32"
  | .mii => "encode_i32"
  | .mi  => "encode_i32"

/-- C macro for extracting the result. -/
def DateOp.pgReturn : DateOp → String
  | .pli => "PG_RETURN_DATEADT"
  | .mii => "PG_RETURN_DATEADT"
  | .mi  => "PG_RETURN_INT32"

end Pg.Ir.Emit.DateArith
