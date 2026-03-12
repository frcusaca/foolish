# D0.1 — Brane with Boundary

> AI agents: read [../../DOC_AGENTS.md](../../DOC_AGENTS.md) before editing this file.

This document describes Normal Branes — the curly-brace structures that define search boundaries.
Read [D0.0 ProtoBrane Lifecycle](d0_0_protobrane.md) first.

For how branes communicate via the communication adapter, see [D0.6 Communication Medium](d0_6_communication_medium.md).

Concatenating: `cat d0_0_protobrane.md d0_1_brane.md` produces a coherent document.

---

## What Is a Brane?

A Brane is a ProtoBrane that defines a search boundary. The syntax is `{...}`.

### Key Properties

1. **Search boundary** — Searches inside check local namespace first, then escalate
2. **First-class value** — `value()` returns the brane itself, not a scalar
3. **Scope organization** — Contains named statements with writing-order precedence

### Syntax

```foolish
{
    a = 1;
    b = 2;
    c = a + b;
}
```

---

## The Boundary Behavior

### What "Search Boundary" Means

When a search originates inside a brane:

1. **Search local first** — Look through the brane's named statements
2. **Writing-order precedence** — First match in writing order wins
3. **Escalate if not found** — Send `FulfillSearch` to parent
4. **Stop at boundary** — Parent cannot search *into* children without explicit operators

### Example: Boundary Blocks Search

```foolish
{
    x = 1;
    y = {
        z = 2;
    };
    w = x;     -- Finds x=1 (local)
    v = y.z;   -- Must use explicit search to get inside y
    -- u = z;  -- ERROR: z not found (blocked by y's boundary)
}
```

---

## Lifecycle with Boundary

### PREMBRYONIC

Same as ProtoBrane (see D0.0), plus:

- **Mark as boundary** — Set `hasBoundary = true`
- **Build local search cache** — Index all named statements for fast lookup

### EMBRYONIC — Search Resolution with Boundary

This is where the boundary behavior matters most.

The brane uses its *communication adapter* to send and receive messages. The adapter abstracts
the underlying communication medium — see [D0.6 Communication Medium](d0_6_communication_medium.md).

```java
step() {
    if (nyes == EMBRYONIC) {
        // 1. Handle inbound messages (via communication adapter)
        while (adapter.canReceive() && adapter.hasIncoming()) {
            handle(adapter.receive());
        }

        // 2. Process wait-for queue
        recheckWaitForQueue();

        // 3. Resolve local searches (up to compute bandwidth)
        for (SearchFir search : unresolvedSearches) {
            if (computeBandwidthExhausted()) break;

            FIR result = searchLocally(search);

            if (result != null) {
                // Found locally — bind result
                search.bind(result);
            } else {
                // Not found locally — escalate to parent (via adapter)
                if (adapter.canSend()) {
                    adapter.send(new FulfillSearch(search));
                }
            }
        }

        // 4. Check transition
        if (allSearchesResolved() && waitForQueueEmpty()) {
            nyes = BRANING;
        }
    }
}
```

The adapter's `canReceive()` and `canSend()` methods enforce communication bandwidth limits.
Pending messages queue until the next step.

### Local Search Algorithm

```java
FIR searchLocally(SearchFir search) {
    // Start from cursor position, go backward (writing order)
    for (int i = cursor - 1; i >= 0; i--) {
        FIR candidate = statements[i];

        // Check if candidate matches the search
        if (matches(search.identifier, candidate)) {
            return candidate;
        }

        // Stop at brane boundary — don't search into children
        if (candidate.isBrane() && search.isAnchored()) {
            break;
        }
    }
    return null;  // Not found locally
}
```

### handle(FulfillSearch) — Being a Parent

When another brane searches and reaches you:

```java
handle(FulfillSearch msg) {
    // 1. Try to resolve locally
    FIR result = searchLocally(msg);

    if (result != null) {
        // Found — send back
        sendRespondToSearch(msg.requesterLUID, result, result.isNigh());
    } else {
        // Not found — forward to my parent (if any)
        if (parent != null) {
            forwardToFulfillSearch(msg);
        } else {
            // Top of chain — not found
            sendRespondToSearch(msg.requesterLUID, NK, false);
        }
    }
}
```

### handle(RespondToSearch) — Receiving Results

```java
handle(RespondToSearch msg) {
    // 1. Find the requesting search by LUID
    SearchFir search = findSearchByLUID(msg.requesterLUID);

    // 2. Apply result
    if (msg.result.isNigh()) {
        // Result still evaluating — wait for it
        search.waitFor(msg.result);
    } else if (msg.result.isConstanic()) {
        // Result is constanic — may need cloning
        search.bind(msg.result);
        if (msg.result.isConstanicButNotConstant()) {
            triggerConstanicCloning(search, msg.result);
        }
    } else {
        // NK or other — bind as-is
        search.bind(msg.result);
    }
}
```

### BRANING

Same as ProtoBrane (see D0.0):
- Step child branes
- Continue message handling
- Monitor wait-for queue

---

## Message Flow Diagram

### Search That Finds Locally

```
┌─────────────────────────────────────────────────────────────┐
│ Parent Brane                                                 │
│                                                              │
│   ┌─────────────┐  ┌──────────────────────────────────┐    │
│   │   x = 1     │  │   y = {                          │    │
│   │   (CONSTANT)│  │       z = x + 1;                 │    │
│   └─────────────┘  │                                  │    │
│                    │  ┌────────────────────────────┐  │    │
│                    │  │   SearchFir for "x"        │  │    │
│                    │  └────────────────────────────┘  │    │
│                    │           │                      │    │
│                    │           ▼                      │    │
│                    │   Local search finds x=1         │    │
│                    │   Bind result (no escalation)    │    │
│                    └──────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

### Search That Escalates

```
┌─────────────────────────────────────────────────────────────┐
│ Grandparent Brane                                            │
│   ┌──────────────────────────────────────────────────────┐  │
│   │   Parent Brane                                        │  │
│   │   ┌──────────────────────────────────────────────┐   │  │
│   │   │   Child Brane                                 │   │  │
│   │   │   ┌──────────────────────────────────────┐   │   │  │
│   │   │   │   SearchFir for "x"                  │   │   │  │
│   │   │   │   Local search: NOT FOUND            │   │   │  │
│   │   │   └────────────────┬─────────────────────┘   │   │  │
│   │   │                    │ FulfillSearch           │   │  │
│   │   │                    ▼                         │   │  │
│   │   │   Local search: NOT FOUND                    │   │  │
│   │   │   Forward FulfillSearch                      │   │  │
│   │   └────────────────────┬─────────────────────────┘   │  │
│   │                        │ FulfillSearch               │  │
│   │                        ▼                             │  │
│   │   Local search: FOUND x = 42                         │  │
│   │   RespondToSearch ← ← ← ← ← ← ← ← ← ← ← ← ← ←       │  │
│   └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## Shadowing

When the same identifier appears multiple times, writing order determines which is found.

### Example: Shadowing

```foolish
{
    a = 10;
    {
        a = 20;      -- Shadows outer a
        result = a;  -- Finds a=20 (local), not a=10 (outer)
    };
}
```

### Why It Works

1. Inner brane's local search starts from cursor, goes backward
2. Finds `a = 20` before reaching the boundary
3. Never escalates to parent — local found wins

---

## Anchored vs Unanchored Search

### Anchored Search (`?name`)

Searches only within the current brane. Never escalates.

```java
FIR resolveAnchoredSearch(String name) {
    FIR result = searchLocally(name);
    return result != null ? result : NK;  // No escalation
}
```

### Unanchored Search (`name` or `'name`)

Searches locally first, then escalates if not found.

```java
void resolveUnanchoredSearch(String name) {
    FIR result = searchLocally(name);

    if (result != null) {
        bind(name, result);
    } else {
        sendToFulfillSearch(name);  // Escalate
    }
}
```

---

## Example: Basic Brane

### Input

```foolish
!!INPUT!!
{
    a = 1;
    b = 2;
    c = a + b;
}

!!!
FINAL RESULT:
{
＿a = 1;
＿b = 2;
＿c = 3;
}
```

### Lifecycle Trace

| Step | Action |
|------|--------|
| 1 | PREMBRYONIC: Create statement array `[a=1, b=2, c=a+b]` |
| 2 | PREMBRYONIC: Build search cache `["a", "b", "c"]` |
| 3 | PREMBRYONIC: Find SearchFir for `a` in RHS of `c` |
| 4 | PREMBRYONIC: Find SearchFir for `b` in RHS of `c` |
| 5 | EMBRYONIC: Resolve `a` — found locally at index 0 |
| 6 | EMBRYONIC: Resolve `b` — found locally at index 1 |
| 7 | BRANING: 🧠+ computes 1 + 2 = 3 |
| 8 | CONSTANT: All statements finalized |

---

## Example: Nested Branes with Search

### Input

```foolish
!!INPUT!!
{
    x = 10;
    {
        y = 20;
        result = x + y;
    };
}

!!!
FINAL RESULT:
{
＿x = 10;
＿{
＿＿y = 20;
＿＿result = 30;
＿};
}
```

### Lifecycle Trace

| Step | Action |
|------|--------|
| 1 | PREMBRYONIC: Outer brane creates `[x=10, inner={...}]` |
| 2 | PREMBRYONIC: Inner brane creates `[y=20, result=x+y]` |
| 3 | EMBRYONIC: Inner resolves `y` — found locally |
| 4 | EMBRYONIC: Inner resolves `x` — NOT found locally |
| 5 | EMBRYONIC: Inner sends `FulfillSearch("x")` to outer |
| 6 | EMBRYONIC: Outer receives, finds `x=10` locally |
| 7 | EMBRYONIC: Outer sends `RespondToSearch(x=10)` to inner |
| 8 | EMBRYONIC: Inner receives, binds `x` to 10 |
| 9 | BRANING: Inner 🧠+ computes 10 + 20 = 30 |
| 10 | CONSTANT: Both branes finalized |

---

## Example: Search Not Found

### Input

```foolish
!!INPUT!!
{
    result = missing;
}

!!!
FINAL RESULT:
{
＿result = ???;
}
```

### Lifecycle Trace

| Step | Action |
|------|--------|
| 1 | PREMBRYONIC: Create `[result=missing]` |
| 2 | EMBRYONIC: Resolve `missing` — NOT found locally |
| 3 | EMBRYONIC: Send `FulfillSearch("missing")` to parent |
| 4 | EMBRYONIC: Parent (root) doesn't have it |
| 5 | EMBRYONIC: Parent sends `RespondToSearch(NK)` |
| 6 | EMBRYONIC: Bind `missing` to NK |
| 7 | CONSTANT: Result is `???` |

---

## Related Approval Tests

### identifierShadowingIsApproved.foo

```foolish
!!INPUT!!
{
    x = 1;
    {
        x = 2;
        result = x;
    };
}

!!!
FINAL RESULT:
{
＿x = 1;
＿{
＿＿x = 2;
＿＿result = 2;
＿};
}
```

**Why it works**:
1. Inner brane's local search for `x` finds `x = 2` first
2. Never escalates to outer — shadowing complete
3. `result = 2` (not 1)

### nestedScopeIdentifierIsApproved.foo

```foolish
!!INPUT!!
{
    a = 1;
    {
        b = 2;
        result = a + b;
    };
}

!!!
FINAL RESULT:
{
＿a = 1;
＿{
＿＿b = 2;
＿＿result = 3;
＿};
}
```

**Why it works**:
1. Inner `b` found locally
2. Inner `a` NOT found locally → escalates to parent
3. Parent finds `a = 1`, returns via `RespondToSearch`
4. Inner binds `a` to 1, computes `1 + 2 = 3`

### complexIdentifierScopeIsApproved.foo

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
1. Deepest inner resolves `c` locally → 3
2. Deepest inner resolves `b` — not local, escalates to middle
3. Middle finds `b = 2`, returns to inner
4. Deepest inner resolves `a` — not local, escalates to middle, then to outer
5. Outer finds `a = 1`, returns through chain to inner
6. Inner computes `1 + 2 + 3 = 6`

---

## Edge Cases

### Empty Brane

```foolish
{}
```

**Result**: `{}` — Empty brane, no statements, immediately CONSTANIC

### Self-Reference

```foolish
{
    x = x;
}
```

**Result**: `{x = ???;}` — Search for `x` finds nothing (nothing defined before it), becomes NK

### Forward Reference

```foolish
{
    x = y;
    y = 42;
}
```

**Result**: `{x = ???; y = 42;}` — Writing order matters! `x` searches before `y` is defined

---

## Next Steps

After reading this:
- **d0_2_system_operator.md** — System operators
- **d0_3_concatenation.md** — Concatenation semantics
- **d0_4_detachment.md** — Detachment filters
- **d0_6_communication_medium.md** — How branes exchange messages

---

## Last Updated

**Date**: 2026-03-12
**Updated By**: Claude Code / cyankiwi/Qwen3.5-27B-AWQ-BF16-INT8
**Changes**: Added reference to D0.6 Communication Medium. Updated EMBRYONIC lifecycle code to show communication adapter usage (adapter.canReceive(), adapter.send()). Separated compute bandwidth from communication bandwidth in step() method.
Previous (2026-03-08): Initial creation describing Normal Brane with search boundary. Documented local search algorithm, message handling (FulfillSearch, RespondToSearch), shadowing behavior, anchored vs unanchored search. Included approval test examples with step-by-step lifecycle traces.
