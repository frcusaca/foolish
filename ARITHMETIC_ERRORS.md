# Arithmetic Errors and NK Values

## Not-Known (NK) Values

The UBC (Unicelular Brane Computer) represents invalid computation results as **NK (Not-Known)** values, displayed as `???`.

### Current Implementation (Integer Arithmetic)

The following operations result in NK values:

1. **Division by zero**: `10 / 0` → `???`
2. **Modulo by zero**: `10 % 0` → `???`
3. **Propagation**: Any operation with an NK operand produces NK
   - `??? + 5` → `???`
   - `10 * ???` → `???`
   - `??? / ???` → `???`

### Future: Floating-Point Support

When floating-point arithmetic is added, the following should also result in NK:

1. **NaN (Not a Number)** operations:
   - `0.0 / 0.0` → `???`
   - `sqrt(-1)` → `???`  (if square root is added)
   - `log(-1)` → `???`  (if logarithm is added)

2. **Infinity** handling (design decision needed):
   - Should `1.0 / 0.0` (positive infinity) be NK or a special value?
   - Should `-1.0 / 0.0` (negative infinity) be NK or a special value?

## Implementation Details

### NKFiroe Class

Located at: `src/main/java/org/foolish/fvm/ubc/NKFiroe.java`

Key properties:
- `isAbstract()` returns `true` (NK is abstract/unknown)
- `getValue()` throws `IllegalStateException` (no concrete value exists)
- `toString()` returns `"???"`

### BinaryFiroe Error Handling

Located at: `src/main/java/org/foolish/fvm/ubc/BinaryFiroe.java`

Error detection:
1. Checks if operands are abstract (NK) before evaluation
2. Checks for division/modulo by zero before performing operation
3. Returns `NKFiroe` instead of throwing exceptions

### Sequencer Display

The `Sequencer4Human` class handles NK display:
- Checks `isAbstract()` flag on FIR results
- Displays `???` for NK values
- Works with assignments: `x = 10 / 0` displays as `x = ???`

## Design Philosophy

**Errors as Values**: Rather than throwing exceptions that halt computation, arithmetic errors produce NK values that propagate through the computation. This allows:

1. Partial results to be computed even when some operations fail
2. More graceful error handling in interactive environments
3. Debugging by seeing where NK values originate

## Examples

```
{
    a = 10 / 0;      // a = ???
    b = 5 + 3;       // b = 8
    c = a * 2;       // c = ??? (propagated from a)
    d = b - 1;       // d = 7 (unaffected by NK values)
}
```

Result:
```
{
  ＿a = ???;
  ＿b = 8;
  ＿c = ???;
  ＿d = 7;
}
```
