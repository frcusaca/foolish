# System Operators

> AI agents: read [../DOC_AGENTS.md](../DOC_AGENTS.md) before editing this file.

System operators are the denotations of built-in VM/hardware operations. The 🧠 prefix
(U+1F9E0) is the denotational marker for Foolish semantics — it signals that we are talking
about the *meaning* of an expression, not its surface syntax. A system operator like `🧠+` is
not something a programmer types; it is what the addition operator `+` means after the parser
has converted syntax to semantics.

## Purpose

System operators bridge the gap between high-level Foolish expressions and low-level machine
operations. When the parser encounters a binary or unary operator in Foolish source code, it
*desugars* the operator into a system operator proto-brane.

For example, the expression `1 + 2` is desugared to `{1}{2}{🧠+}`, which concatenates to form a
single proto-brane `{1, 2, 🧠+}` containing three statements.

## System Operator Table

| Operator | Foolish Syntax | System Symbol | Unicode | Operation | Arity |
|----------|---------------|---------------|---------|-----------|-------|
| Add | `a + b` | 🧠+ | U+1F9E0 U+002B | Addition | Binary |
| Subtract | `a - b` | 🧠- | U+1F9E0 U+002D | Subtraction | Binary |
| Multiply | `a * b` | 🧠* | U+1F9E0 U+002A | Multiplication | Binary |
| Divide | `a / b` | 🧠/ | U+1F9E0 U+002F | Division | Binary |
| Modulo | `a % b` | 🧠% | U+1F9E0 U+0025 | Modulo | Binary |
| Negate | `-a` | 🧠− | U+1F9E0 U+2212 | Unary negation | Unary |
| Not | `!a` | 🧠! | U+1F9E0 U+0021 | Logical NOT | Unary |
| And | `a && b` | 🧠∧ | U+1F9E0 U+2227 | Logical AND | Binary |
| Or | `a \|\| b` | 🧠∨ | U+1F9E0 U+2228 | Logical OR | Binary |
| Equal | `a == b` | 🧠= | U+1F9E0 U+003D | Equality test | Binary |
| Not Equal | `a != b` | 🧠≠ | U+1F9E0 U+2260 | Inequality test | Binary |
| Less Than | `a < b` | 🧠< | U+1F9E0 U+003C | Less than | Binary |
| Less Or Equal | `a <= b` | 🧠≤ | U+1F9E0 U+2264 | Less than or equal | Binary |
| Greater Than | `a > b` | 🧠> | U+1F9E0 U+003E | Greater than | Binary |
| Greater Or Equal | `a >= b` | 🧠≥ | U+1F9E0 U+2265 | Greater than or equal | Binary |

## How System Operators Work

### Desugaring

When the parser encounters an operator in Foolish source code, it desugars the expression:

Source:
```foolish
result = (3 + 4) * 2
```

Desugared (conceptual):
```
result = {3}{4}{🧠+}{2}{🧠*}
     → {3, 4, 🧠+, 2, 🧠*}
```

Note: The grouping `()` affects parse order but is not shown in the desugared form.

### Execution Model

Each system operator FIR is a proto-brane that:

1. Waits for operands to become constanic. The system operator monitors its parent brane's
   statement array for the preceding values.

2. Retrieves operands. When operands are available:
   - Binary operators call `parent.getValuesBeforeMe(me, 2)` to get the two preceding values
   - Unary operators call `parent.getValueBeforeMe(me)` to get the one preceding value

3. Computes the result — performs the operation (addition, negation, etc.)

4. Handles errors:
   - Arithmetic errors (division by zero, overflow) produce NK with a descriptive comment
   - Type errors (cannot add string + int) produce NK with a descriptive comment
   - Programming errors (NullPointerException) propagate as world-stopping exceptions

5. Transitions to CONSTANT. The system operator's `value()` method returns the computed scalar.

### Naive Implementation Example

The following is a conceptual implementation for the addition operator. The actual FIR would
integrate with the proto-brane lifecycle and handle constanic dependencies correctly.

```java
class SystemAddFir extends ProtoBrane {
    @Override
    public void step() {
        if (at_embryonic()) {
            // Wait for the two preceding values to become constanic
            FIR[] operands = parent.getValuesBeforeMe(this, 2);
            if (operands[0].hasConstanicity() && operands[1].hasConstanicity()) {
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

1. Domain errors (division by zero, type mismatches):
   - Produce NK (`???`) with a descriptive comment
   - Example: `＿result = ???;  !! Division by zero`

2. Programming errors (null references, assertion failures):
   - Propagate as world-stopping exceptions
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

The operands may be nigh (not-yet-evaluated), CONSTANIC, WOCONSTANIC, or CONSTANT.
The system operator must:
- Wait for operands to reach constanic states before computing
- Use the wait-for mechanism if operands are nigh
- Handle CONSTANIC operands (produce NK or wait for constanic cloning to resolve them)

## Adding New System Operators

To add a new system operator:

1. Define the symbol — choose a Unicode symbol and prefix with 🧠
2. Update the symbol table — add to [SYMBOL_TABLE.md](SYMBOL_TABLE.md)
3. Implement the FIR — create a new ProtoBrane subclass or add to a system operator factory
4. Add to parser — update the parser to desugar the Foolish syntax to the system symbol
5. Write tests — add approval tests for the operator with various operand states

## Future System Operators

The following system operators are planned but not yet implemented:

- String concatenation (`🧠++`)
- Bitwise operations (`🧠&`, `🧠|`, `🧠^`, `🧠~`, `🧠<<`, `🧠>>`)
- Array operations (`🧠[]`, `🧠[]=`)
- Brane operations (`🧠?`, `🧠$`, `🧠@`) — these may not need system operators if handled by
  search FIRs

---

## Last Updated

**Date**: 2026-03-12
**Updated By**: Claude Code / cyankiwi/Qwen3.5-27B-AWQ-BF16-INT8
**Changes**: Updated terminology: replaced `achievedConstanic()` with `hasConstanicity()` to align
with the new term "constanicity" (pronounced "con-stan-ISS-ity").
Previous (2026-02-26): Reduced emphatic markings throughout — removed bold from execution model
steps, error handling categories, operand access notes, future operator list, and label lines before
code blocks. Retained bold only on "not" in error handling rule. First occurrence of "desugars"
italicized as a definition.
Previous (2026-02-20): Initial creation by claude-sonnet-4-5-20250929.
