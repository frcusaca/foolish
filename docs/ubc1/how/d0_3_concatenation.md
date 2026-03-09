# D0.3 — Concatenation

> AI agents: read [../../DOC_AGENTS.md](../../DOC_AGENTS.md) before editing this file.

This document describes ConcatenationBrane — the mechanism for combining branes into a single
evaluation context. Read [D0.0 ProtoBrane Lifecycle](d0_0_protobrane.md) first.

Concatenating: `cat d0_0_protobrane.md d0_3_concatenation.md` produces a coherent document.

---

## What Is Concatenation?

Concatenation combines two or more branes into a single evaluation context. The syntax places
branes adjacent to each other:

```foolish
{a=1,b=2}{c=3,d=4}
```

### The Problem It Solves

When a brane references something it doesn't have locally, concatenation with another brane can
provide it. This is Foolish's mechanism for:
- Dependency resolution across contexts
- Building larger contexts from smaller ones
- Late binding of free variables

---

## The Three Stages

Concatenation operates in three distinct stages:

```
Stage A (Isolation)     Stage B (Merge)          Stage C (Evaluation)
─────────────────────   ──────────────────────   ─────────────────────
Step children to        Clone & re-parent all    Normal brane evaluation
constanic in isolation  into merged brane        on merged result

Search isolation ON     Merged brane created     Parent search enabled
FulfillSearch blocked   Statements flattened     Re-resolve CONSTANIC
```

### Stage Overview

| Stage | Purpose | Search Behavior |
|-------|---------|-----------------|
| **A: Isolation** | Children evaluate independently | Searches blocked from escaping |
| **B: Merge** | Combine statements into single array | N/A (structural transformation) |
| **C: Evaluation** | Re-evaluate in combined context | Normal search resolution |

---

## Lifecycle Traversal

### PREMBRYONIC

ConcatenationBrane in PREMBRYONIC:

1. **Count lines** — Sum of all children's statement counts
2. **Establish structure** — Record child FIRs in writing order
3. **Instantiate child FIRs** — Each child in PREMBRYONIC (do not step yet)
4. **Establish LUID** — For the concatenation itself

→ Transition to EMBRYONIC

### EMBRYONIC — Stage A: Isolation

In EMBRYONIC, ConcatenationBrane steps its children to constanic **in isolation**:

```java
step() {
    if (nyes == EMBRYONIC) {
        // Stage A: Isolation

        // Step children (they progress through their own lifecycle)
        for (FIR child : children) {
            if (!child.isConstanic()) {
                child.step();
            }
        }

        // Check if all children are constanic
        if (allChildrenConstanic()) {
            // Transition to Stage B (Merge)
            nyes = BRANING;
        }
    }
}
```

#### Search Isolation

During Stage A, **FulfillSearch messages from children are blocked**:

```java
handle(FulfillSearch msg) {
    if (stage == STAGE_A_ISOLATION) {
        // Do NOT forward to parent
        // Child will become CONSTANIC (free variable)
        return;
    }
    // In other stages, forward normally
    forwardToFulfillSearch(msg);
}
```

This is critical: if a child searches for something it doesn't have, the search does NOT
escalate. The child becomes CONSTANIC (waiting for more context), not NK.

### BRANING — Stage B: Merge

When all children reach constanic, merge begins:

```java
step() {
    if (nyes == BRANING && stage == STAGE_B_MERGE) {
        // Stage B: Merge

        // 1. For each child, constanic-copy its statements
        List<FIR> allStatements = new ArrayList<>();
        for (FIR child : children) {
            for (FIR stmt : child.getStatements()) {
                FIR clone = stmt.cloneConstanic();
                clone.setParent(mergedBrane);  // Re-parent!
                allStatements.add(clone);
            }
        }

        // 2. Flatten into single contiguous array
        mergedBrane.statements = allStatements.toArray();

        // 3. Build search cache on merged brane
        mergedBrane.buildSearchCache();

        // 4. Transition to Stage C (Evaluation)
        stage = STAGE_C_EVALUATION;
    }

    if (stage == STAGE_C_EVALUATION) {
        // Stage C: Normal brane evaluation
        stepMergedBrane();
    }
}
```

#### Why Clone?

We clone statements during merge because:
1. **Parent reference change** — Statements need to point to merged brane, not original child
2. **Constanic guarantee** — Only constanic statements can be safely cloned and relocated
3. **Re-evaluation** — Cloned CONSTANIC expressions will re-resolve in new context

### Stage C: Evaluation

The merged brane now evaluates normally:

```java
stepMergedBrane() {
    // Normal ProtoBrane BRANING behavior:
    // 1. Step child branes (if any nested branes in merged result)
    // 2. Handle messages (FulfillSearch, RespondToSearch)
    // 3. Monitor wait-for queue

    // Previously-CONSTANIC expressions now have chance to resolve
    // because their identifiers may exist in the combined context
}
```

---

## Message Flow During Concatenation

### Stage A: Isolation (Searches Blocked)

```
┌─────────────────────────────────────────────────────────────┐
│ Parent Brane                                                 │
│                                                              │
│   ┌──────────────────────┐  ┌──────────────────────┐       │
│   │  ConcatenationBrane  │  │                      │       │
│   │  (EMBRYONIC)         │  │                      │       │
│   │                      │  │                      │       │
│   │  ┌──────────────┐    │  │  ┌──────────────┐   │       │
│   │  │ Child Brane 1│    │  │  │ Child Brane 2│   │       │
│   │  │ (EMBRYONIC)  │    │  │  │ (EMBRYONIC)  │   │       │
│   │  │              │    │  │  │              │   │       │
│   │  │  x = y;      │    │  │  │  z = 42;     │   │       │
│   │  │  SearchFir   │    │  │  │  (CONSTANT)  │   │       │
│   │  │  for "y"     │    │  │  └──────────────┘   │       │
│   │  │      │       │    │  │                     │       │
│   │  │      ▼       │    │  │                     │       │
│   │  │  NOT FOUND   │    │  │                     │       │
│   │  │  Escalate?───┼────┼──┤ NO — Blocked!       │       │
│   │  │              │    │  │                     │       │
│   │  │  becomes     │    │  │                     │       │
│   │  │  CONSTANIC   │    │  │                     │       │
│   │  └──────────────┘    │  │                     │       │
│   └──────────────────────┘  └──────────────────────┘       │
└─────────────────────────────────────────────────────────────┘
```

### Stage B: Merge (Structural)

```
┌─────────────────────────────────────────────────────────────┐
│ ConcatenationBrane (BRANING - Stage B)                      │
│                                                              │
│   Child 1 Statements:       Child 2 Statements:             │
│   [x = y (CONSTANIC)]   +   [z = 42 (CONSTANT)]             │
│                                                              │
│            │                    │                            │
│            ▼                    ▼                            │
│   Clone: x = y          Clone: z = 42                       │
│   Reparent to merged    Reparent to merged                  │
│                                                              │
│            └──────────┬──────────┘                          │
│                       ▼                                      │
│   Merged Brane Statements:                                  │
│   [x = y, z = 42]                                           │
│                                                              │
│   Now x's search can find z=42! (if x searches for z)       │
└─────────────────────────────────────────────────────────────┘
```

### Stage C: Re-evaluation (Searches Work)

```
┌─────────────────────────────────────────────────────────────┐
│ Merged Brane (BRANING - Stage C)                            │
│                                                              │
│   Statements: [x = y, z = 42]                               │
│                                                              │
│   Re-evaluate x = y:                                        │
│   ┌──────────────────────────────────────────────────┐     │
│   │  SearchFir for "y" → NOT FOUND                   │     │
│   │  Escalate to parent (no longer blocked!)         │     │
│   └──────────────────────────────────────────────────┘     │
│                                                              │
│   If parent has y, x resolves. If not, x stays CONSTANIC.   │
└─────────────────────────────────────────────────────────────┘
```

---

## step() Summary by Stage

| Stage | nyse | What step() Does |
|-------|------|------------------|
| A | EMBRYONIC | Step children to constanic; block FulfillSearch |
| B | BRANING | Clone and re-parent statements; create merged brane |
| C | BRANING | Evaluate merged brane normally |

---

## Why CONSTANIC, Not NK?

This is critical to concatenation's design:

| State | Meaning | Concatenation Implication |
|-------|---------|---------------------------|
| **CONSTANIC** | May gain value in new context | Concatenation can provide binding |
| **NK** | Provably unknown/unfindable | Nothing can fix this |

When a child's search fails during isolation, it becomes **CONSTANIC**, not NK. This signals
"waiting for more context" rather than "definitely not found."

---

## Example: Basic Concatenation

### Input

```foolish
!!INPUT!!
{
    c = {a=1,b=2,c=3}{e=4,f=5,g=6};
}

!!!
FINAL RESULT:
{
＿c = {
＿    ＿{
＿＿＿  ＿a = 1;
＿＿＿  ＿b = 2;
＿＿＿  ＿c = 3;
＿    ＿};
＿    ＿{
＿＿＿  ＿e = 4;
＿＿＿  ＿f = 5;
＿＿＿  ＿g = 6;
＿    ＿};
＿};
}
```

### Lifecycle Trace

| Step | Action |
|------|--------|
| 1 | PREMBRYONIC: ConcatenationBrane creates structure with 2 children |
| 2 | EMBRYONIC (Stage A): Step child 1 `{a=1,b=2,c=3}` to CONSTANIC |
| 3 | EMBRYONIC (Stage A): Step child 2 `{e=4,f=5,g=6}` to CONSTANIC |
| 4 | All children constanic → Transition to BRANING |
| 5 | BRANING (Stage B): Clone child 1's statements |
| 6 | BRANING (Stage B): Clone child 2's statements |
| 7 | BRANING (Stage B): Merge into `[a=1, b=2, c=3, e=4, f=5, g=6]` |
| 8 | BRANING (Stage C): Merged brane evaluates (already done) |
| 9 | CONSTANT: Concatenation complete |

**Note**: In this example, both children had all identifiers resolved locally. The concatenation
just combines them without needing re-resolution.

---

## Example: Concatenation Resolving CONSTANIC

Consider this case where concatenation actually resolves dependencies:

### Conceptual Input

```foolish
{
    f = {a = x;};      -- x not found → a is CONSTANIC
    g = {x = 42;};
    h = g f;           -- Concatenation: x should resolve to 42
}
```

### Lifecycle Trace

| Step | Action |
|------|--------|
| 1 | PREMBRYONIC: Create `f`, `g`, and concatenation `h = g f` |
| 2 | EMBRYONIC: `f` evaluates, `a = x` searches for `x`, not found → CONSTANIC |
| 3 | EMBRYONIC: `g` evaluates, `x = 42` is CONSTANT |
| 4 | EMBRYONIC (Stage A): Concatenation `g f` steps children in isolation |
| 5 | Stage B: Merge → `[x=42, a=x]`, clone `a=x` with new parent |
| 6 | Stage C: Re-evaluate `a = x` |
| 7 | Search for `x` now finds `x = 42` (it's in the merged context!) |
| 8 | `a` becomes 42 |
| 9 | CONSTANT: `h = {x=42, a=42}` |

---

## Related Approval Tests

### concatenationBasics.approved.foo

See the test output above. Key observations:

1. **test_1**: Inline concatenation `{a=1,b=2,c=3}{e=4,f=5,g=6}`
   - Both children fully resolved in isolation
   - Merge produces combined brane with both sets of statements
   - Result: `{a=1, b=2, c=3, e=4, f=5, g=6}` (shown as two branes concatenated)

2. **test_2**: Reference concatenation `b1 b2`
   - `b1` and `b2` are named branes
   - Search resolves `b1` and `b2` to their values
   - Concatenation merges them
   - Same result as test_1

3. **test_3**: Mixed concatenation
   - `c1 = {literal} b2` — literal brane concatenated with reference
   - `c2 = b1 {literal}` — reference concatenated with literal brane
   - All produce the same merged result

---

## Step() Behavior by State and Stage

| State | Stage | What Happens |
|-------|-------|--------------|
| PREMBRYONIC | — | Set up structure, instantiate children |
| EMBRYONIC | A: Isolation | Step children; block FulfillSearch |
| BRANING | B: Merge | Clone, re-parent, flatten |
| BRANING | C: Evaluation | Normal brane evaluation |
| CONSTANT | — | No-op (immutable) |

---

## Edge Cases

### Empty Concatenation

```foolish
{}{}
```

**Result**: Empty brane `{}` — Both children empty, merge produces empty

### Concatenation with Unresolved Identifiers

```foolish
{
    x = {a = y;};  -- y not found, a is CONSTANIC
    z = {y = 1;};
    w = x z;        -- Should resolve a to 1
}
```

**Behavior**:
1. `x` evaluates: `a = y` → CONSTANIC (search blocked in isolation)
2. `z` evaluates: `y = 1` → CONSTANT
3. Concatenation `x z`:
   - Stage A: Both children constanic
   - Stage B: Merge → `[y=1, a=y]`
   - Stage C: Re-evaluate `a = y`, finds `y=1`
4. Result: `w = {y=1, a=1}`

### Nested Concatenation

```foolish
{
    a = {x=1}{y=2};
    b = a{z=3};
}
```

**Behavior**:
1. `a` concatenates first: `{x=1, y=2}`
2. `b` concatenates `a` (already merged) with `{z=3}`
3. Result: `b = {x=1, y=2, z=3}`

---

## Comparison: Concatenation vs Normal Brane

| Aspect | ConcatenationBrane | Normal Brane |
|--------|-------------------|--------------|
| Boundary | No (temporary isolation) | Yes (permanent) |
| value() | Returns merged brane | Returns self |
| Search handling | Block → Forward → Normal | Always local then forward |
| Stages | 3 (Isolation, Merge, Evaluation) | 1 (standard lifecycle) |

---

## Next Steps

After reading this:
- **d0_4_detachment.md** — Detachment filters
- **d0_5_brane_recoordination.md** — Recoordination after cloning

---

## Last Updated

**Date**: 2026-03-08
**Updated By**: Claude Code / cyankiwi/Qwen3.5-27B-AWQ-BF16-INT8
**Changes**: Initial creation describing ConcatenationBrane. Documented three stages (Isolation, Merge, Evaluation), search isolation behavior, clone-and-reparent mechanism, and included approval test examples with lifecycle traces.
