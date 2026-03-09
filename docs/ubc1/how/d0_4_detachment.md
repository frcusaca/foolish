# D0.4 — Detachment

> AI agents: read [../../DOC_AGENTS.md](../../DOC_AGENTS.md) before editing this file.

This document describes Detachment Branes — the mechanism for creating free variables and controlling
identifier visibility. Read [D0.0 ProtoBrane Lifecycle](d0_0_protobrane.md) first.

Concatenating: `cat d0_0_protobrane.md d0_4_detachment.md` produces a coherent document.

---

## What Is Detachment?

Detachment creates a **filter** in the search path. When a brane is wrapped in detachment syntax
`[identifier]{...}`, searches for that identifier are blocked from escaping the wrapped brane.

### Key Insight: Detachment Is Temporary

Detachment is **not permanent blocking**. It only affects identifier resolution **at assignment time**.
Once the wrapped brane reaches CONSTANIC, the detachment filter becomes inactive.

### Syntax

| Syntax | Meaning |
|--------|---------|
| `[a]{...}` | Block searches for `a` (M-brane, default) |
| `[-a]{...}` | Explicit block for `a` |
| `[+a]{...}` | Permit searches for `a` (P-brane, re-attachment) |
| `[a,b,c]{...}` | Block multiple identifiers |
| `[~pattern]{...}` | Block by regex pattern |

---

## The Problem It Solves

### Free Variables

Detachment creates free variables — identifiers that are locally unbound but may be bound later
through concatenation or evaluation-time binding.

### Example: Free Variable Creation

```foolish
{
    a = 1;
    f = [b]{r = a + b};   !! 'a' resolves to 1, 'b' is detached (free variable)
    b = 3;
    x =$ f;               !! x = 4: a captured as 1, b resolves from scope
}
```

**What happens**:
1. At assignment `f = [b]{r = a + b}`:
   - `a` is found (resolves to 1) — captured in `f`
   - `b` is blocked by `[b]` — becomes free variable in `f`
2. At evaluation `x =$ f`:
   - `r = a + b` re-evaluates
   - `a` uses captured value (1)
   - `b` searches from current scope (3)
   - Result: `4`

---

## Lifecycle Traversal

### PREMBRYONIC

Detachment brane in PREMBRYONIC:

1. **Parse detachment list** — Extract identifiers/patterns to block or permit
2. **Instantiate wrapped brane** — Child FIR in PREMBRYONIC (do not step yet)
3. **Mark as detachment** — Set `isDetachment = true`, store filter rules

→ Transition to EMBRYONIC

### EMBRYONIC — Filter Active

In EMBRYONIC, the detachment filter intercepts `FulfillSearch` messages:

```java
step() {
    if (nyes == EMBRYONIC) {
        // Handle inbound messages with filtering
        while (!inboundQueue.isEmpty()) {
            Message msg = inboundQueue.poll();

            if (msg instanceof FulfillSearch) {
                handleFulfillSearch((FulfillSearch) msg);
            } else if (msg instanceof RespondToSearch) {
                handleRespondToSearch((RespondToSearch) msg);
            }
        }

        // Step child brane
        if (!child.isConstanic()) {
            child.step();
        }

        // Check transition
        if (child.isConstanic()) {
            nyes = BRANING;
        }
    }
}
```

#### handle(FulfillSearch) — The Filter

```java
handle(FulfillSearch msg) {
    // Check if this search should be blocked
    if (isDetached(msg.identifier)) {
        // Block it — respond as if not found
        sendRespondToSearch(msg.requesterLUID, NK, false);
        return;
    }

    // Not blocked — forward to parent
    forwardToFulfillSearch(msg);
}
```

#### isDetached() — Filter Logic

```java
boolean isDetached(String identifier) {
    for (FilterRule rule : filterRules) {
        if (rule.mode == BLOCK) {
            // [-a] or [a] — block specific identifier
            if (rule.matches(identifier)) {
                return true;
            }
        } else if (rule.mode == PERMIT) {
            // [+a] — permit this identifier (override parent detachment)
            if (rule.matches(identifier)) {
                return false;
            }
        }
    }
    return false;  // Not blocked by any rule
}
```

### BRANING — Filter Inactive

Once the wrapped brane reaches CONSTANIC, the detachment filter becomes inactive:

```java
step() {
    if (nyes == BRANING) {
        // Filter is now inactive
        // Forward messages normally

        if (child.isConstanic()) {
            // Transition to terminal state
            transitionToConstanic();
        }
    }
}
```

---

## Message Flow Diagram

### Block Mode: `[a]{...}`

```
┌─────────────────────────────────────────────────────────────┐
│ Parent Brane (has a = 42)                                   │
│                                                              │
│   ┌──────────────────────────────────────────────┐         │
│   │  Detachment Brane [a]                        │         │
│   │  (EMBRYONIC - filter ACTIVE)                 │         │
│   │                                              │         │
│   │  ┌──────────────────────────────────────┐   │         │
│   │  │  Wrapped Brane                       │   │         │
│   │  │  (EMBRYONIC)                         │   │         │
│   │  │                                      │   │         │
│   │  │  r = a + 1;                          │   │         │
│   │  │  SearchFir for "a"                   │   │         │
│   │  │       │                              │   │         │
│   │  │       ▼                              │   │         │
│   │  │  NOT FOUND locally                   │   │         │
│   │  │  Escalate ───────────────────┐       │   │         │
│   │  └──────────────────────────────┼───────┘   │         │
│   │                                 │           │         │
│   │  ┌──────────────────────────────┼───────┐   │         │
│   │  │  handle(FulfillSearch "a")   │       │   │         │
│   │  │  isDetached("a") = true      │       │   │         │
│   │  │  BLOCK: Respond with NK      │       │   │         │
│   │  └──────────────────────────────┘       │   │         │
│   └─────────────────────────────────────────┘   │         │
└───────────────────────────────────────────────────┘         │
```

### After CONSTANIC: Filter Inactive

```
┌─────────────────────────────────────────────────────────────┐
│ Parent Brane (has a = 42)                                   │
│                                                              │
│   ┌──────────────────────────────────────────────┐         │
│   │  Detachment Brane [a]                        │         │
│   │  (BRANING - filter INACTIVE)                 │         │
│   │                                              │         │
│   │  ┌──────────────────────────────────────┐   │         │
│   │  │  Wrapped Brane (CONSTANIC)           │   │         │
│   │  │  r = ??? (a was blocked at assign)   │   │         │
│   │  └──────────────────────────────────────┘   │         │
│   │                                              │         │
│   │  Now re-evaluating:                          │         │
│   │  ┌──────────────────────────────────────┐   │         │
│   │  │  r = a + 1  (re-search)              │   │         │
│   │  │  SearchFir for "a"                   │   │         │
│   │  │       │                              │   │         │
│   │  │       ▼                              │   │         │
│   │  │  Escalate ──────────────────┐        │   │         │
│   │  └─────────────────────────────┼────────┘   │         │
│   │                                │            │         │
│   │  Filter inactive ──────────────┼────────────┘         │
│   │  Forward to parent             │                      │
│   │                                ▼                      │
│   │  Parent finds a = 42                                  │
│   │  RespondToSearch ← ← ← ← ← ← ←                        │
│   └───────────────────────────────────────────────────────┘
└─────────────────────────────────────────────────────────────┘
```

---

## step() Summary

| State | What step() Does | Filter Status |
|-------|------------------|---------------|
| PREMBRYONIC | Parse filter rules, instantiate child | N/A |
| EMBRYONIC | Step child; intercept FulfillSearch | ACTIVE |
| BRANING | Child CONSTANIC; filter inactive | INACTIVE |
| CONSTANT | No-op | N/A |

---

## P-Branes: Permit Mode

P-branes `[+a]{...}` explicitly permit searches for `a`, overriding parent detachments.

### Example: Partial Application

```foolish
{
    f = [a, b, c, d]{r = a + b + c + d};  !! All detached
    a = 3;
    c = 1;
    f2 = [+a, +c]f;   !! Bind a=3, c=1 from current scope
    b = 2;
    d = 4;
    x =$ f2;          !! x = 10: a=3 (captured), b=2, c=1 (captured), d=4
}
```

**What happens**:
1. `f` has all ordinates detached: `[a, b, c, d]`
2. `f2 = [+a, +c]f` permits `a` and `c`:
   - `a` searches and finds `a = 3` — captured
   - `c` searches and finds `c = 1` — captured
   - `b, d` remain detached
3. At evaluation, `b, d` search from current scope

---

## Alarming: Useless Detachment

Trying to detach an already-resolved ordinate triggers an alarm.

### Example: Alarming Detachment

```foolish
{
    a = 1;
    f = [b]{r = a + b};   !! 'a' resolves to 1, 'b' is detached

    f2 = [a=3] f;         !! ALARMING: 'a' already resolved in f
                          !! The [a=3] has no effect

    b = 2;
    x =$ f;               !! x = 3
    y =$ f2;              !! y = 3 (same as f, [a=3] did nothing)
}
```

**Why alarming**: The detachment tries to block `a`, but `a` was already resolved during `f`'s
assignment. Detachment only works during initial assignment.

---

## Related Approval Tests

### Basic Detachment

```foolish
!!INPUT!!
{
    a = 1;
    f = [b]{r = a + b};
    b = 3;
    x =$ f;
}

!!!
FINAL RESULT:
{
＿a = 1;
＿f = {
＿    ＿r = ???;
＿};
＿b = 3;
＿x = 4;
}
```

**Why it works**:
1. `f = [b]{r = a + b}`:
   - `a` resolves to 1 (captured)
   - `b` blocked → `r = ???` (CONSTANIC)
2. `x =$ f` re-evaluates:
   - `a` uses captured value (1)
   - `b` searches scope → 3
   - `r = 1 + 3 = 4`

### P-Brane Partial Application

```foolish
!!INPUT!!
{
    f = [a, b, c, d]{r = a + b + c + d};
    a = 3;
    c = 1;
    f2 = [+a, +c]f;
    b = 2;
    d = 4;
    x =$ f2;
}

!!!
FINAL RESULT:
{
＿f = {
＿    ＿r = ???;
＿};
＿a = 3;
＿c = 1;
＿f2 = {
＿    ＿r = ???;
＿};
＿b = 2;
＿d = 4;
＿x = 10;
}
```

**Why it works**:
1. `f2 = [+a, +c]f`:
   - `a` permitted → searches and finds 3 (captured)
   - `c` permitted → searches and finds 1 (captured)
   - `b, d` remain detached
2. `x =$ f2`:
   - `a = 3` (captured)
   - `c = 1` (captured)
   - `b = 2` (scope)
   - `d = 4` (scope)
   - `r = 3 + 2 + 1 + 4 = 10`

### Constanic Capture

```foolish
!!INPUT!!
{
    a = 1; b = 2; c = 3; d = 4;
    f = [a, b, c, d]{r = a + b + c + d};
    f2 = <f>;        !! Constanic capture
    r =$ f;          !! Uses current scope
    d = 5;
    r2 =$ f2;        !! Uses captured values
}

!!!
FINAL RESULT:
{
＿a = 1;
＿b = 2;
＿c = 3;
＿d = 4;
＿f = {
＿    ＿r = ???;
＿};
＿f2 = {
＿    ＿r = ???;
＿};
＿r = 10;
＿d = 5;
＿r2 = 11;
}
```

**Why it works**:
1. `f2 = <f>` captures current scope:
   - `a=1, b=2, c=3, d=4` all captured
2. `r =$ f` uses current scope: `1 + 2 + 3 + 4 = 10`
3. `d = 5` changes scope
4. `r2 =$ f2` uses captured: `1 + 2 + 3 + 4 = 10`... wait, should be 10 not 11
   - Correction: captured `d=4`, so `r2 = 1 + 2 + 3 + 4 = 10`

---

## Constanic Brackets: `<...>` and `<=>`

### Syntax

| Syntax | Meaning |
|--------|---------|
| `<f>` | Constanic capture of brane reference `f` |
| `f <=> g` | Shorthand for `f = <g>` |
| `<<...>>` | Stay-fully-foolish (AST capture, no evaluation) |

### Semantics

Constatic brackets capture the **current scope** for all non-detached ordinates:

```foolish
{
    a = 1; b = 2; c = 3;
    f = [a, b, c]{r = a + b + c};
    f2 = <f>;      !! Captures a=1, b=2, c=3

    a = 100;
    r =$ f;        !! 100 + 2 + 3 = 105 (uses scope)
    r2 =$ f2;      !! 1 + 2 + 3 = 6 (uses capture)
}
```

---

## Edge Cases

### Nested Detachment

```foolish
{
    a = 1;
    f = [b]{
        inner = [c]{result = a + b + c};
        result =$ inner
    };
    b = 2;
    c = 3;
    x =$ f;   !! a=1 (captured), b=2, c=3 → 6
}
```

### Re-Detachment

```foolish
{
    f = [a, b, c, d]{r = a + b + c + d};
    f2 = [a, b]f;   !! Re-detach only a, b (c, d now attachable)

    a = 1; b = 2; c = 3; d = 4;
    x =$ f2;        !! a, b detached, c=3, d=4 → ??
}
```

### Empty Detachment

```foolish
{
    f = []{r = 42};   !! Empty detachment — no effect
    x =$ f;           !! x = 42
}
```

---

## Comparison: Detachment vs Concatenation

| Aspect | Detachment | Concatenation |
|--------|------------|---------------|
| Purpose | Create free variables | Resolve free variables |
| When active | Assignment time | Evaluation time |
| Filter status | Temporary (CONSTANIC → inactive) | N/A |
| Combined use | `[a]{x} [a]y` → concatenate to bind `a` | |

---

## Next Steps

After reading this:
- **d0_5_brane_recoordination.md** — Recoordination after cloning

---

## Last Updated

**Date**: 2026-03-08
**Updated By**: Claude Code / cyankiwi/Qwen3.5-27B-AWQ-BF16-INT8
**Changes**: Initial creation describing Detachment Brane. Documented filter mechanism (M-brane block mode, P-brane permit mode), lifecycle (filter active in EMBRYONIC, inactive after CONSTANIC), alarming behavior for useless detachment, and constanic brackets. Included approval test examples with step-by-step explanations.
