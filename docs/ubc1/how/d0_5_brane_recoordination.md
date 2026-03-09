# D0.5 — Brane Recoordination

> AI agents: read [../../DOC_AGENTS.md](../../DOC_AGENTS.md) before editing this file.

This document describes brane recoordination — what happens when a CONSTANIC brane is cloned into
a new parent context and must re-resolve its searches. Read [D0.0 ProtoBrane Lifecycle](d0_0_protobrane.md) first.

Concatenating: `cat d0_0_protobrane.md d0_5_brane_recoordination.md` produces a coherent document.

---

## What Is Recoordination?

Recoordination is the process by which a CONSTANIC brane gains new value when placed in a new
context. When a brane is cloned into a different parent, its CONSTANIC sub-expressions get a
fresh chance to resolve against the new context.

### The Problem It Solves

A brane may contain CONSTANIC expressions — searches that couldn't resolve in their original
context. When that brane is:
- Concatenated with another brane
- Assigned to a different parent
- Used in a new scope

...those CONSTANIC expressions may now find what they were searching for.

### Example: Constanic Cloning on Concatenation

```foolish
{
    f = {a = x;};     !! x not found → a is CONSTANIC
    g = {x = 42;};
    h = g f;          !! Concatenation: a should re-resolve to 42
}
```

**What happens**:
1. `f` evaluates independently: `a = x` searches for `x`, not found → `a` is CONSTANIC
2. `g` evaluates: `x = 42` is CONSTANT
3. `h = g f` concatenates:
   - Stage A: Children stepped in isolation
   - Stage B: Merge → clone `a=x` with new parent (the merged brane)
   - Stage C: Clone re-resolves `x`, finds `x = 42` in merged context
4. Result: `h = {x=42, a=42}`

---

## When Recoordination Occurs

Recoordination is triggered when:

1. **Concatenation** — Merged brane re-evaluates CONSTANIC children
2. **Parent change** — Constanic cloning updates parent reference
3. **Context extension** — New bindings become available

---

## The Constanic Cloning Process

### Trigger Condition

Constanic cloning occurs when:
- A brane reaches EMBRYONIC or BRANING
- It contains CONSTANIC or WOCONSTANIC sub-expressions
- Those sub-expressions have a new parent context

### Step 1: Clone Structure

```java
FIR cloneConstanic(FIR original, FIR newParent) {
    if (original.isConstant()) {
        return original;  // Reference, don't clone CONSTANT
    }

    if (original instanceof ProtoBrane) {
        ProtoBrane clone = new ProtoBrane();
        clone.statements = original.statements.clone();
        clone.setParent(newParent);  // New parent!
        clone.nyes = original.nyes;  // Start in same state
        return clone;
    }

    // For expressions
    Expression clone = original.clone();
    clone.setParent(newParent);
    return clone;
}
```

### Step 2: Clone Sub-Expressions

For each statement in the brane:

```java
void cloneStatements(ProtoBrane original, ProtoBrane clone) {
    for (int i = 0; i < original.statementCount; i++) {
        FIR stmt = original.getStatement(i);

        if (stmt.isConstant()) {
            // Reference to immutable
            clone.setStatement(i, stmt);
        } else if (stmt.isConstanic()) {
            // Clone and re-parent
            FIR clonedStmt = cloneConstanic(stmt, clone);
            clone.setStatement(i, clonedStmt);
        }
    }
}
```

### Step 3: Determine Clone Initial State

| Original State | Clone Initial State | Why |
|----------------|---------------------|-----|
| CONSTANIC | EMBRYONIC | Re-search in new context |
| WOCONSTANIC | BRANING | Wait for children, then re-dereference |
| CONSTANT | N/A | Reference original (no clone) |

---

## Recoordination Lifecycle

### CONSTANIC Clone → EMBRYONIC

A CONSTANIC brane cloned into a new context enters EMBRYONIC:

```java
step() {
    if (nyes == EMBRYONIC) {
        // 1. Handle inbound messages
        handleInboundMessages();

        // 2. Re-resolve searches
        for (SearchFir search : unresolvedSearches) {
            if (searchBandwidthExhausted()) break;

            FIR result = searchLocally(search);

            if (result != null) {
                // Found in new context!
                search.bind(result);
            } else {
                // Still not found — escalate
                sendToFulfillSearch(search);
            }
        }

        // 3. Check transition
        if (allSearchesResolved()) {
            nyes = BRANING;
        }
    }
}
```

### WOCONSTANIC Clone → BRANING

A WOCONSTANIC brane (has CONSTANIC children) enters BRANING:

```java
step() {
    if (nyes == BRANING) {
        // 1. Step children
        stepChildren();

        // 2. Handle messages
        handleInboundMessages();

        // 3. Check if children became constant
        if (allChildrenConstant()) {
            // Re-evaluate dereferencing
            reevaluateDereferencing();
            transitionTo(CONSTANT);
        } else if (allChildrenConstanic()) {
            transitionTo(WOCONSTANIC);
        }
    }
}
```

---

## Message Flow During Recoordination

### Before Cloning: CONSTANIC in Original Context

```
┌─────────────────────────────────────────────────────────────┐
│ Original Parent Brane                                        │
│                                                              │
│   ┌────────────────────────────────────────────────┐       │
│   │  Brane f (CONSTANIC)                           │       │
│   │                                                │       │
│   │  a = x;    ← SearchFir for "x"                 │       │
│   │          │                                     │       │
│   │          ▼                                     │       │
│   │  Local search: NOT FOUND                       │       │
│   │  Escalate to parent ──→ NOT FOUND              │       │
│   │  Result: a is CONSTANIC                        │       │
│   └────────────────────────────────────────────────┘       │
└─────────────────────────────────────────────────────────────┘
```

### After Cloning: Re-resolve in New Context

```
┌─────────────────────────────────────────────────────────────┐
│ Merged Brane (new parent)                                    │
│                                                              │
│   Statements: [x = 42, a = x]                               │
│                                                              │
│   ┌────────────────────────────────────────────────┐       │
│   │  Clone of 'a = x' (EMBRYONIC)                  │       │
│   │  Parent updated to merged brane                │       │
│   │                                                │       │
│   │  Re-resolve: SearchFir for "x"                 │       │
│   │          │                                     │       │
│   │          ▼                                     │       │
│   │  Local search finds x = 42!                    │       │
│   │  Bind a to 42                                  │       │
│   │  Transition to BRANING                         │       │
│   └────────────────────────────────────────────────┘       │
└─────────────────────────────────────────────────────────────┘
```

---

## step() Summary for Recoordination

| Clone Type | Initial State | What step() Does |
|------------|---------------|------------------|
| CONSTANIC | EMBRYONIC | Re-search in new context |
| WOCONSTANIC | BRANING | Wait for children, re-dereference |

---

## Dereferencing Loop During Recoordination

When a search result is WOCONSTANIC (still stepping), the dereferencing loop continues:

```java
FIR resolve() {
    FIR target = searchLocally();

    if (target == null) {
        return null;  // Not found
    }

    // Dereferencing loop: chase through WOCONSTANIC
    while (target.isWONSTANIC()) {
        // Wait for target to become constant
        // When target changes state, re-check
        target = followDereferenceChain(target);
    }

    // Now target is CONSTANIC or CONSTANT
    return target;
}
```

### Example: Chained Dereferencing

```foolish
{
    a = x;    !! x not found → a is CONSTANIC
    b = a;    !! b dereferences a → b is WOCONSTANIC (waits for a)
    x = 42;
}
```

**During concatenation recoordination**:
1. `a` re-resolves: finds `x = 42` → `a` becomes CONSTANT
2. `b`'s dereferencing loop detects `a` changed state
3. `b` follows chain: `a` is now 42 → `b` becomes CONSTANT

---

## Related Approval Tests

### Concatenation Resolves CONSTANIC

```foolish
!!INPUT!!
{
    test_2={
       b1={a=1,b=2,c=3};
       b2={e=4,f=5,g=6};
       c = b1 b2;
    };
}

!!!
FINAL RESULT:
{
＿testˍ2 = {
＿         ＿b1 = {
＿         ＿     ＿a = 1;
＿         ＿     ＿b = 2;
＿         ＿     ＿c = 3;
＿         ＿};
＿         ＿b2 = {
＿         ＿     ＿e = 4;
＿         ＿     ＿f = 5;
＿         ＿     ＿g = 6;
＿         ＿};
＿         ＿c = {
＿         ＿    ＿{
＿＿＿      ＿    ＿a = 1;
＿＿＿      ＿    ＿b = 2;
＿＿＿      ＿    ＿c = 3;
＿         ＿    ＿};
＿         ＿    ＿{
＿＿＿      ＿    ＿e = 4;
＿＿＿      ＿    ＿f = 5;
＿＿＿      ＿    ＿g = 6;
＿         ＿    ＿};
＿         ＿};
＿};
}
```

**Why it works**:
1. `b1` and `b2` evaluate independently (all identifiers resolved locally)
2. `c = b1 b2` concatenates:
   - Stage A: Both children constanic
   - Stage B: Merge → clone statements, re-parent to merged brane
   - Stage C: Merged brane evaluates (already done, no re-resolution needed)
3. Result: Concatenation produces combined brane

### Nested Scope with Parent Search

```foolish
!!INPUT!!
{
    a = 1;
    {
        b = 2;
        {
            c = 3;
            result = a + b + c;
        };
    };
}

!!!
FINAL RESULT:
{
＿a = 1;
＿{
＿＿b = 2;
＿＿{
＿＿＿c = 3;
＿＿＿result = 6;
＿＿};
＿};
}
```

**Why it works**:
1. Innermost resolves `c` locally → 3
2. Innermost resolves `b` — not local, escalates to middle
3. Middle finds `b = 2`, returns via `RespondToSearch`
4. Innermost resolves `a` — not local, escalates through middle to outer
5. Outer finds `a = 1`, returns through chain
6. Innermost computes `1 + 2 + 3 = 6`

---

## Edge Cases

### CONSTANT References Not Cloned

```java
// Optimization: CONSTANT values are immutable
if (stmt.isConstant()) {
    return stmt;  // Reference, not clone
}
```

**Why**: CONSTANT values are immutable and can be safely shared. No need to clone.

### Self-Reference After Cloning

```foolish
{
    f = {a = f;};  !! Self-reference → CONSTANIC
}
```

**Behavior**: Self-reference creates a cycle. The search for `f` finds `f` itself, which is
CONSTANIC. This is WOCONSTANIC (waiting on itself). Eventually resolves or becomes NK.

### Multiple Parent Changes

```foolish
{
    f = {a = x;};
    g = {x = 1;};
    h = {x = 2;};

    i = g f;   !! a resolves to 1
    j = h i;   !! What happens to a?
}
```

**Behavior**: Each concatenation triggers cloning. `a` re-resolves in each new context. The
most recent context's binding wins.

---

## Comparison: UBC1 vs UBC2 Recoordination

| Aspect | UBC1 (CMFir) | UBC2 (Constanic Clone) |
|--------|-------------|----------------------|
| Mechanism | Wrap in CMFir, two-phase eval | Clone and replace |
| When triggered | During evaluation | When object is constanic |
| Parent update | Unclear | Explicit re-parenting |
| WOCONSTANIC | Not distinguished | Enters BRANING |
| CONSTANT | Wrapped unnecessarily | Referenced directly |

---

## Next Steps

All d0 documents complete:
- **d0_0_protobrane.md** — Foundation (ProtoBrane lifecycle)
- **d0_1_brane.md** — Normal brane with boundary
- **d0_2_system_operator.md** — System operators
- **d0_3_concatenation.md** — Concatenation semantics
- **d0_4_detachment.md** — Detachment filters
- **d0_5_brane_recoordination.md** — Recoordination (this document)

---

## Last Updated

**Date**: 2026-03-08
**Updated By**: Claude Code / cyankiwi/Qwen3.5-27B-AWQ-BF16-INT8
**Changes**: Initial creation describing brane recoordination. Documented constanic cloning process, clone lifecycle (CONSTANIC → EMBRYONIC, WOCONSTANIC → BRANING), message flow during recoordination, dereferencing loop, and comparison with UBC1's CMFir approach. Included approval test examples with step-by-step explanations.
