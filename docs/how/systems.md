# System Operators

System operators are built-in operations that the Foolish VM/hardware implements directly. They
are prefixed with the **🧠** symbol (U+1F9E0, "brain") to distinguish them from user-defined
operations.

## Purpose

System operators bridge the gap between high-level Foolish expressions and low-level machine
operations. When the parser encounters a binary or unary operator in Foolish source code, it
**desugars** the operator into a system operator proto-brane.

For example, the expression `1 + 2` is desugared to `{1}{2}{🧠+}`, which concatenates to form a
single proto-brane `{1, 2, 🧠+}` containing three statements.

## System Operator Table

| Operator | Foolish Syntax | System Symbol | Unicode | Operation | Arity |
|----------|---------------|---------------|---------|-----------|-------|
| **Add** | `a + b` | 🧠+ | U+1F9E0 U+002B | Addition | Binary |
| **Subtract** | `a - b` | 🧠- | U+1F9E0 U+002D | Subtraction | Binary |
| **Multiply** | `a * b` | 🧠* | U+1F9E0 U+002A | Multiplication | Binary |
| **Divide** | `a / b` | 🧠/ | U+1F9E0 U+002F | Division | Binary |
| **Modulo** | `a % b` | 🧠% | U+1F9E0 U+0025 | Modulo | Binary |
| **Negate** | `-a` | 🧠− | U+1F9E0 U+2212 | Unary negation | Unary |
| **Not** | `!a` | 🧠! | U+1F9E0 U+0021 | Logical NOT | Unary |
| **And** | `a && b` | 🧠∧ | U+1F9E0 U+2227 | Logical AND | Binary |
| **Or** | `a \|\| b` | 🧠∨ | U+1F9E0 U+2228 | Logical OR | Binary |
| **Equal** | `a == b` | 🧠= | U+1F9E0 U+003D | Equality test | Binary |
| **Not Equal** | `a != b` | 🧠≠ | U+1F9E0 U+2260 | Inequality test | Binary |
| **Less Than** | `a < b` | 🧠< | U+1F9E0 U+003C | Less than | Binary |
| **Less Or Equal** | `a <= b` | 🧠≤ | U+1F9E0 U+2264 | Less than or equal | Binary |
| **Greater Than** | `a > b` | 🧠> | U+1F9E0 U+003E | Greater than | Binary |
| **Greater Or Equal** | `a >= b` | 🧠≥ | U+1F9E0 U+2265 | Greater than or equal | Binary |

## How System Operators Work

### Desugaring

When the parser encounters an operator in Foolish source code, it desugars the expression:

**Source:**
```foolish
result = (3 + 4) * 2
```

**Desugared (conceptual):**
```
result = {3}{4}{🧠+}{2}{🧠*}
     → {3, 4, 🧠+, 2, 🧠*}
```

Note: The grouping `()` affects parse order but is not shown in the desugared form.

### Execution Model

Each system operator FIR is a **proto-brane** that:

1. **Waits for operands to become constanic.** The system operator monitors its parent brane's
   statement array for the preceding values.

2. **Retrieves operands.** When operands are available:
   - **Binary operators:** Call `parent.getValuesBeforeMe(me, 2)` to get the two preceding values
   - **Unary operators:** Call `parent.getValueBeforeMe(me)` to get the one preceding value

3. **Computes result.** Perform the operation (addition, negation, etc.)

4. **Handles errors:**
   - **Arithmetic errors** (division by zero, overflow): Produce NK with descriptive comment
   - **Type errors** (cannot add string + int): Produce NK with descriptive comment
   - **Programming errors** (NullPointerException): Propagate as world-stopping exception

5. **Transitions to CONSTANT.** The system operator's `value()` method returns the computed scalar.

### Naive Implementation Example

The following is a **conceptual** implementation for the addition operator. The actual FIR would
integrate with the proto-brane lifecycle and handle constanic dependencies correctly.

```java
class SystemAddFir extends ProtoBrane {
    @Override
    public void step() {
        if (at_embryonic()) {
            // Wait for the two preceding values to become constanic
            FIR[] operands = parent.getValuesBeforeMe(this, 2);
            if (operands[0].is_constanic() && operands[1].is_constanic()) {
                try {
                    int a = operands[0].value();
                    int b = operands[1].value();
                    myValue = a + b;
                    transition(CONSTANT);
                } catch (ArithmeticException e) {
                    transition(NK);  // e.g., overflow
                    myComment = "Arithmetic error: " + e.getMessage();
                }
            }
        }
    }

    @Override
    public int value() {
        if (at_constant()) return myValue;
        throw new IllegalStateException("Cannot get value of non-constant FIR");
    }
}
```

### Error Handling

System operators must distinguish:

1. **Domain errors** (division by zero, type mismatches):
   - Produce **NK** (`???`) with a descriptive comment
   - Example: `＿result = ???;  !! Division by zero`

2. **Programming errors** (null references, assertion failures):
   - Propagate as **world-stopping exceptions**
   - Halt evaluation immediately and surface to user

Do **not** catch `Exception` broadly — only catch specific domain exceptions like
`ArithmeticException`.

## Operand Access Protocol

System operators access their operands through the parent brane's statement array. The protocol is:

```java
// For binary operators:
FIR[] operands = parent.getValuesBeforeMe(this, 2);
FIR left = operands[0];   // Value immediately before this operator
FIR right = operands[1];  // Value two positions before this operator

// For unary operators:
FIR operand = parent.getValueBeforeMe(this);
```

**Important:** The operands may be NYE (not-yet-evaluated), CONSTANIC, WOCONSTANIC, or CONSTANT.
The system operator must:
- Wait for operands to reach constanic states before computing
- Use the wait-for mechanism if operands are NYE
- Handle CONSTANIC operands (produce NK or wait for constanic cloning to resolve them)

## Adding New System Operators

To add a new system operator:

1. **Define the symbol:** Choose a Unicode symbol and prefix with 🧠
2. **Update the symbol table:** Add to [SYMBOL_TABLE.md](SYMBOL_TABLE.md)
3. **Implement the FIR:** Create a new ProtoBrane subclass or add to a system operator factory
4. **Add to parser:** Update the parser to desugar the Foolish syntax to the system symbol
5. **Write tests:** Add approval tests for the operator with various operand states

## Future System Operators

The following system operators are planned but not yet implemented:

- **String concatenation** (`🧠++`)
- **Bitwise operations** (`🧠&`, `🧠|`, `🧠^`, `🧠~`, `🧠<<`, `🧠>>`)
- **Array operations** (`🧠[]`, `🧠[]=`)
- **Brane operations** (`🧠?`, `🧠$`, `🧠@`) — these may not need system operators if handled by
  search FIRs

---

## Last Updated

**Date**: 2026-02-20
**Updated By**: Claude Code v1.0.0 / claude-sonnet-4-5-20250929
**Changes**: Initial creation. Comprehensive documentation of system operators with 🧠 prefix,
desugaring examples, execution model, error handling protocol, and operand access protocol.
