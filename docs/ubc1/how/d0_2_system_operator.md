# D0.2 — System Operators

> AI agents: read [../../DOC_AGENTS.md](../../DOC_AGENTS.md) before editing this file.

This document describes System Operator branes — the computational engine of Foolish. Read
[D0.0 ProtoBrane Lifecycle](d0_0_protobrane.md) first.

Concatenating: `cat d0_0_protobrane.md d0_2_system_operator.md` produces a coherent document.

---

## What Is a System Operator?

A System Operator is a proto-brane that computes scalar values (numbers, booleans) rather than
brane values. System operators are produced by desugaring infix and prefix operators.

### Desugaring Examples

| Source | Desugared |
|--------|-----------|
| `1 + 2` | `{🧠1, 🧠2, 🧠+}` |
| `a - b` | `{a, b, 🧠-}` |
| `-x` | `{x, 🧠−}` |
| `!flag` | `{flag, 🧠!}` |

The `🧠` prefix (U+1F9E0) marks system operators. It signals the *meaning* of an expression,
not its surface syntax.

### System Operator Table

| Symbol | Arity | Operation | Terminal State Conditions |
|--------|-------|-----------|---------------------------|
| 🧠+ | Binary | Addition | CONSTANT on success, NK on overflow |
| 🧠- | Binary | Subtraction | CONSTANT on success, NK on overflow |
| 🧠* | Binary | Multiplication | CONSTANT on success, NK on overflow |
| 🧠/ | Binary | Division | CONSTANT on success, NK on div-by-zero |
| 🧠% | Binary | Modulo | CONSTANT on success, NK on div-by-zero |
| 🧠− | Unary | Negation | CONSTANT on success, NK on overflow |
| 🧠! | Unary | Logical NOT | CONSTANT (boolean result) |
| 🧠∧ | Binary | Logical AND | CONSTANT (boolean result) |
| 🧠∨ | Binary | Logical OR | CONSTANT (boolean result) |
| 🧠= | Binary | Equality | CONSTANT (boolean result) |
| 🧠≠ | Binary | Inequality | CONSTANT (boolean result) |
| 🧠< | Binary | Less than | CONSTANT (boolean result) |
| 🧠≤ | Binary | Less or equal | CONSTANT (boolean result) |
| 🧠> | Binary | Greater than | CONSTANT (boolean result) |
| 🧠≥ | Binary | Greater or equal | CONSTANT (boolean result) |

---

## How System Operators Work

### Key Distinction: No Boundary

Unlike Normal Branes, System Operators do **not** create a search boundary:
- They have no interior namespace
- Searches pass through them transparently
- They compute values, they do not organize scope

### Operand Access Protocol

System operators access operands by indexing into their parent brane's statement array:

```java
// Binary operator (🧠+, 🧠-, 🧠*, 🧠/)
FIR[] operands = parent.getValuesBeforeMe(this, 2);
FIR left = operands[0];   // Value immediately before
FIR right = operands[1];  // Value two positions before

// Unary operator (🧠−, 🧠!)
FIR operand = parent.getValueBeforeMe(this);
```

### Operand States

Operands may be in any state:
- **CONSTANT** — Ready to use
- **CONSTANIC** — Not ready; operator enters wait-for
- **WOCONSTANIC** — Not ready; operator enters wait-for
- **NK** — Error; propagate as NK with description

---

## Lifecycle Traversal

System operators follow the standard ProtoBrane lifecycle with modifications to `step()`.

### PREMBRYONIC

Same as ProtoBrane (see D0.0):
- Count lines (typically 1: just the operator)
- Establish statement array
- Build search cache (none for simple operators)
- Instantiate RHS FIRs (operands)
- Append child branes (none for simple operators)
- Establish LUID

→ Transition to EMBRYONIC

### EMBRYONIC

System operators have no searches of their own. In EMBRYONIC:
- Handle any inbound messages (rare for operators)
- Check if operands are ready
- If operands ready, transition to BRANING

```java
step() {
    if (nyes == EMBRYONIC) {
        // No searches to resolve for simple operators
        // Check operand readiness
        if (operandsAreConstant()) {
            nyes = BRANING;
        }
    }
}
```

### BRANING

This is where computation happens:

```java
step() {
    if (nyes == BRANING) {
        // Step children first (operands must be CONSTANT)
        stepChildren();

        // Once children are constant, compute
        if (allChildrenConstant()) {
            try {
                myValue = compute(operands);
                transitionTo(CONSTANT);
            } catch (ArithmeticException e) {
                myValue = NK;
                myComment = e.getMessage();
                transitionTo(CONSTANT);
            }
        }
    }
}
```

### Computation Logic

```java
int compute(FIR[] operands) {
    int a = operands[0].value();
    int b = operands[1].value();

    switch (operator) {
        case 🧠+: return a + b;
        case 🧠-: return a - b;
        case 🧠*: return a * b;
        case 🧠/:
            if (b == 0) throw new ArithmeticException("Division by zero");
            return a / b;
        case 🧠%:
            if (b == 0) throw new ArithmeticException("Modulo by zero");
            return a % b;
        // ... other operators
    }
}
```

### Error Handling

System operators distinguish:

| Error Type | Result | Example |
|------------|--------|---------|
| Domain error (div by zero) | NK with comment | `1 / 0 → ???` |
| Type error (string + int) | NK with comment | `"hi" + 1 → ???` |
| Programming error (NPE) | Exception | World-stopping |

Do **not** catch `Exception` broadly. Only catch domain errors like `ArithmeticException`.

---

## Step() Behavior by State

| State | What step() Does |
|-------|------------------|
| PREMBRYONIC | Set up statement array, operands |
| EMBRYONIC | Check operand readiness |
| BRANING | Step children, then compute |
| CONSTANT | No-op (immutable) |

---

## Message Flow

System operators have minimal message flow since they don't have searches to resolve:

```
┌─────────────────────────────────────────────────────────────┐
│ Parent Brane                                                 │
│                                                              │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│   │   Operand 1 │  │   Operand 2 │  │   🧠+       │        │
│   │   (CONSTANT)│  │   (CONSTANT)│  │   (BRANING) │        │
│   └─────────────┘  └─────────────┘  └─────────────┘        │
│                            │                                 │
│                            ▼                                 │
│                   🧠+ reads operands from                    │
│                   parent statement array                     │
│                            │                                 │
│                            ▼                                 │
│                   Computes: 1 + 2 = 3                        │
│                   Transitions to CONSTANT                    │
└─────────────────────────────────────────────────────────────┘
```

---

## Example: Simple Addition

### Input

```foolish
{3 + 4;}
```

### Desugared

```
{🧠3, 🧠4, 🧠+}
```

### Lifecycle Timeline

| Step | Action |
|------|--------|
| 1 | PREMBRYONIC: All three FIRs create statement arrays |
| 2 | EMBRYONIC: Literals (🧠3, 🧠4) are already CONSTANT |
| 3 | BRANING: 🧠+ sees operands ready, computes 3 + 4 = 7 |
| 4 | CONSTANT: Final result `{7;}` |

### Output

```
{
＿7;
}
```

---

## Example: Chained Arithmetic

### Input

```foolish
{2 + 3 * 4;}
```

### Desugared (with precedence)

```
{2, 3, 🧠+, 4, 🧠*}
```

Wait — that's wrong! The parser handles precedence. Correct desugaring:

```
{2, 3, 4, 🧠+, 🧠*}  -- No, still wrong
```

Actually the parser builds an AST reflecting precedence:

```
{2, 3, 🧠+, 4, 🧠*}  with structure: ((2 + 3) * 4)
```

### Evaluation

1. 🧠+ waits for 2 and 3 → computes 5
2. 🧠* waits for result of 🧠+ (5) and 4 → computes 20

### Output

```
{
＿20;
}
```

---

## Example: Division by Zero

### Input

```foolish
{10 / 0;}
```

### Evaluation

1. PREMBRYONIC: Set up statement array `[10, 0, 🧠/]`
2. EMBRYONIC: Operands (literals) are CONSTANT
3. BRANING: 🧠/ attempts division, catches exception
4. CONSTANT with NK: `{???; !! Division by zero}`

### Output

```
{
＿???;
}
```

---

## Example: Operator with Identifier

### Input

```foolish
{
    x = 5;
    result = x + 3;
}
```

### Lifecycle

| Step | Action |
|------|--------|
| 1 | PREMBRYONIC: Set up both statements |
| 2 | EMBRYONIC: `x` resolves to 5 (local search) |
| 3 | EMBRYONIC: `x + 3` — SearchFir for `x` resolves to 5 |
| 4 | BRANING: 🧠+ computes 5 + 3 = 8 |
| 5 | CONSTANT: `{x = 5; result = 8;}` |

### Output

```
{
＿x = 5;
＿result = 8;
}
```

---

## Comparison: System Operator vs Normal Brane

| Aspect | System Operator | Normal Brane |
|--------|-----------------|--------------|
| Boundary | No | Yes |
| value() returns | Scalar | Brane itself |
| Searches | None | Resolves identifier searches |
| Operand access | Parent indexing | Local namespace |
| step() in BRANING | Computes | Steps children |

---

## Related Approval Tests

### simpleAdditionIsApproved.foo

```foolish
!!INPUT!!
{
  a = 1 + 2;
  b = 3 + 4;
}

!!!
FINAL RESULT:
{
＿a = 3;
＿b = 7;
}
```

**Why it works**:
1. `a = 1 + 2` desugars to assignment with RHS `[🧠1, 🧠2, 🧠+]`
2. Literals are CONSTANT immediately
3. 🧠+ in BRANING reads operands from parent array, computes 3
4. Same for `b = 3 + 4` → 7

### zeroDivisionIsApproved.foo

```foolish
!!INPUT!!
{
  result = 10 / 0;
}

!!!
FINAL RESULT:
{
＿result = ???;
}
```

**Why it works**:
1. 🧠/ attempts division in BRANING
2. Catches `ArithmeticException`
3. Returns NK with comment "Division by zero"

### complexArithmeticIsApproved.foo

```foolish
!!INPUT!!
{
  result = (2 + 3) * (4 - 1);
}

!!!
FINAL RESULT:
{
＿result = 15;
}
```

**Why it works**:
1. Parser handles parentheses, builds nested AST
2. `(2 + 3)` evaluates to 5 first
3. `(4 - 1)` evaluates to 3
4. Outer 🧠* computes 5 * 3 = 15

---

## Edge Cases

### Unresolved Identifier in Operator

```foolish
{
    result = x + 1;  -- x is not defined
}
```

**Behavior**:
1. EMBRYONIC: SearchFir for `x` dispatched to parent
2. Parent doesn't have `x` → search fails
3. SearchFir becomes NK (not found)
4. 🧠+ receives NK as operand → result is NK

**Result**: `{result = ???;}`

### CONSTANIC Operand

```foolish
{
    x = y;        -- y not found, x is CONSTANIC
    result = x + 1;
}
```

**Behavior**:
1. EMBRYONIC: `x = y` → SearchFir for `y` fails, `x` is CONSTANIC
2. 🧠+ in BRANING sees operand `x` is CONSTANIC, not CONSTANT
3. 🧠+ waits (cannot compute with CONSTANIC operand)
4. Eventually: result is WOCONSTANIC or NK depending on context

---

## Next Steps

After reading this:
- **d0_1_brane.md** — Normal branes with search boundaries
- **d0_3_concatenation.md** — Concatenation semantics
- **d0_4_detachment.md** — Detachment filters

---

## Last Updated

**Date**: 2026-03-08
**Updated By**: Claude Code / cyankiwi/Qwen3.5-27B-AWQ-BF16-INT8
**Changes**: Initial creation describing system operators as ProtoBrane derivatives. Documented lifecycle (PREMBRYONIC → EMBRYONIC → BRANING → CONSTANT), operand access protocol, error handling, and included approval test examples with step-by-step explanations.
