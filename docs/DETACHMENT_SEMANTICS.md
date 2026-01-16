# Detachment Semantics in Foolish

## Overview

Detachment in Foolish creates **free variables** (unresolved ordinates) within a brane. These ordinates must be provided later through explicit binding, partial application, or constantic capture.

**CRITICAL**: Detachment is NOT permanent blocking. It creates branes with unresolved ordinates that can be bound in various ways.

## Core Concept: Detached Ordinates

When you write:
```foolish
f = [a, b] {r = a + b}
```

During the assignment:
- The brane body `{r = a + b}` is evaluated
- Variables `a` and `b` are **not resolved** from the parent scope
- They become **detached ordinates** (free variables) of `f`
- Any other variables (not in the detachment list) resolve normally

## Situation 1: Detachment During Parsing/Assignment

```foolish
{
    a = 1;
    f = [b]{r = a + b};   // 'a' resolves to 1 (captured), 'b' is detached
    b = 3;
    x2 =$ f;              // x2 = 4 (a is 1 from capture, b resolves to 3)

    f2 = [a=3] f          // ALARMING! f already has 'a' resolved
                          // Trying to detach 'a' again has no effect
}
```

**Key Points:**
- When `f` is assigned, `a` resolves immediately to `1`
- `b` is detached and remains a free variable
- Later assignment `b = 3` doesn't affect `f`'s captured `a`
- Trying to re-detach an already-resolved variable should trigger a warning/alarm

## Situation 2: Re-Escape Resolution

You can apply detachment to an already-evaluated brane:

```foolish
{
    f  = [a,b,c,d] {r = a+b+c+d};
    f2 = [a,b,c,d] f;              // Re-detaches (behaves same as f)
    r  =$ [a=1,b=2,c=3,d=4] f      // Evaluates to 10
    r2 =$ [a=1,b=2,c=3,d=5] f2     // Evaluates to 11
}
```

## Detachment Variants

### 1. Explicit Values: `[a=1, b=2]`

Provides explicit values for detached ordinates:

```foolish
f = [a,b,c,d]{r = a+b+c+d};
f2 = [a=1, c=2]f              // Binds a=1, c=2; b,d remain detached
r =$ [b=3, d=4] f2            // r = 10
```

### 2. P-Brane (Partial Application): `[+a, b]`

Binds detached ordinates from the **current scope**:

```foolish
{
    f = [a,b,c,d]{r = a+b+c+d};
    a = 3;
    c = 1;
    f2 = [+a, c]f              // Binds a=3, c=1 from current scope
                               // f2 still has detached ordinates b, d
}
```

The `+` prefix means "bind from current scope value".

### 3. Constantic Brackets: `<f>`

Captures **all detached ordinates** from current scope at assignment time:

```foolish
{
    a=1; b=2; c=3; d=4;
    f  = [a,b,c,d] {r = a+b+c+d};
    f2 = <f>;                     // Captures a=1, b=2, c=3, d=4

    r  =$ f      // Evaluates to 10 (uses current scope)
    d=5;
    r2 =$ f2     // Evaluates to 11 (f2 captured d=5)
    d=6;
    r3 =$ f2     // Still 11 (f2's capture doesn't change)
}
```

### 4. Constantic Assignment: `f2 <=> f`

Shorthand for `f2 = <f>;`:

```foolish
{
    a=1; b=2; c=3; d=4;
    f  = [a,b,c,d] {r = a+b+c+d};
    f3 <=> f;                     // Equivalent to f3 = <f>;

    d=6;
    r3 =$ f3     // Evaluates to 12 (captured a=1,b=2,c=3,d=6 at assignment)
}
```

## Important Notes

1. **Constantic brackets only affect brane references**, not freshly parsed branes:
   ```foolish
   f1 = <[a,b]{r = a+b}>;     // The <> has no effect - brane is fresh
   f2 = [a,b]{r = a+b};       // Equivalent to f1
   ```

2. **Alarming behavior**: Attempting to detach already-resolved variables should trigger a warning

3. **Nested detachment**: Can create complex partial application chains

4. **Evaluation time binding**: `=$ [a=1] f` provides values at evaluation, doesn't modify `f`

## Grammar Additions

### Constantic Brackets
```
ConstanticExpr ::= '<' Expr '>'
```

### Constantic Assignment
```
ConstanticAssignment ::= Identifier '<=>' Expr ';'
```

## Implementation Requirements

1. **Track detached ordinates** in brane representation
2. **Distinguish resolved vs free variables** in brane memory
3. **Implement P-brane binding** from current scope
4. **Implement constantic capture** of all detached ordinates
5. **Detect and alarm** on useless detachment attempts
6. **Support evaluation-time binding** via `=$ [a=1] f`

## Test Coverage Required

- Basic detachment during assignment
- Re-detachment of brane references
- P-brane partial application
- Constantic capture with `<>` and `<=>`
- Nested detachment scenarios
- Alarming on useless detachment
- Mixed binding strategies
- Evaluation-time ordinate provision

## Last Updated

**Date**: 2026-01-16
**Updated By**: Claude Code v1.0.0 / claude-sonnet-4-5-20250929
**Changes**: Initial documentation of correct detachment semantics based on user specification. Detachment creates free variables, not permanent blocking.
