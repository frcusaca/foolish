# ProtoBrane Derived FIRs — Design Document

> AI agents: read [../DOC_AGENTS.md](../DOC_AGENTS.md) before editing this file.

This document is a design proposal for task D0 from
[human_todo_index.md](../todo/human_todo_index.md). It specifies the contract for each of the
four FIR roles that derive from ProtoBrane — how each role leverages the shared lifecycle while
contributing its own specialized behavior.

---

## What We Want to Build — The Requirements to Satisfy

This document establishes what we need to build. The four proto-brane derivatives exhibit distinct
behaviors that serve different purposes in Foolish. This document specifies **what each must
satisfy** and **how step() computes** for each type.

### The Core Mechanism: step(), nyes, and handle(msg)

All branes and proto-branes revolve around three fundamental mechanisms:

1. **step()** — The computation that happens each cycle. Every brane type implements its behavior
   through what it computes in step().

2. **nyes (the lifecycle)** — The state machine that governs evaluation:
   ```
   PREMBRYONIC → EMBRYONIC → BRANING → constanic
   ```
   Each brane type traverses these states, but the *work done in step() for each state* differs.

3. **handle(msg)** — The message handling system. During step(), branes may receive and respond
   to messages from parent, children, or peers.

### The Framework

| Level | Description |
|-------|-------------|
| **Feature** | Described in terms of Foolish code and observable behavior |
| **How** | Specified through step(), nyes state, and handle(msg) |

### The Central Question

> **For each feature, what does step() compute, how does nyes progress, and what messages flow?**

We describe features in terms of Foolish code. We specify implementation through step(), nyes,
and handle(msg).

### Requirements to Satisfy

For each brane type, we must specify:

1. **What step() computes** in each nyes state (PREMBRYONIC, EMBRYONIC, BRANING)
2. **What handle(msg)** does for each message type (FulfillSearch, RespondToSearch, etc.)
3. **How nyes transitions** occur based on computation results
4. **How behaviors differ** between brane types in their step() and handle(msg) implementations

### The Four "Organs" of Foolish

Foolish has four primary structures — like organs in a living system — each serving a vital
function. This document clarifies how each organ works:

| Organ | Syntax | Purpose |
|-------|--------|---------|
| **Brane** | `{...}` | The cell: encapsulates values, manages evaluation, organizes scope |
| **Expression** | `1`, `🧠+`, `x` | The molecule: computes values, participates in evaluation |
| **Concatenation** | `{...}{...}` | The fusion: merges branes, extends context, resolves dependencies |
| **Detachment** | `[a]{...}` | The filter: controls visibility, enables free variables |

---

## Table of Contents

- [What We Don't Know](#what-we-dont-know-the-problems-to-be-resolved)
- [The Four Organs of Foolish](#the-four-organs-of-foolish)
- [Section 1: Features as Organs](#section-1-features-as-organs)
  - [Encapsulation and Organization](#encapsulation-and-organization)
  - [Value Computation](#value-computation)
  - [Context Extension](#context-extension)
  - [Visibility Control](#visibility-control)
- [Section 2: ProtoBrane — The Shared Contract](#section-2-protobrane--the-shared-contract)
- [Section 3: Role A — Normal Brane `{...}`](#section-3-role-a--normal-brane-)
- [Section 4: Role B — System Operator Brane](#section-4-role-b--system-operator-brane)
- [Section 5: Role C — ConcatenationBrane](#section-5-role-c--concatenationbrane)
- [Section 6: Role D — Detachment Brane `[...]`](#section-6-role-d--detachment-brane-)
- [Section 7: How Behaviors Map to Messages](#section-7-how-behaviors-map-to-messages)
- [Section 8: Lessons Learned from UBC1](#section-8-lessons-learned-from-ubc1)
- [Section 9: Design Proposal](#section-9-design-proposal)

---

## Section 1: Features as Organs

Each brane type exists because Foolish needs a specific capability. This section describes those
capabilities in terms of what they accomplish, not how they're implemented.

### Encapsulation and Organization

**The Problem**: How do we group values together while controlling what sees what?

**The Organ**: **Brane** `{...}`

The Brane is Foolish's primary organizational structure. It serves two fundamental purposes:

1. **Encapsulation**: A brane creates a boundary. Values inside cannot be directly accessed from
   outside without explicit search. Similarly, values outside are not automatically visible inside.

2. **Organization**: Within the boundary, a brane manages the evaluation of its members. It
   coordinates their resolution, handles their dependencies, and presents them as a coherent
   whole.

**Example**: In `{a=1; b={c=1}; d=c}`, the assignment to `d` cannot see `c` because `c` is
encapsulated inside brane `b`. The brane boundary blocks unspecific resolution.

**Questions to Answer**:
- How does a Brane block searches from entering its children?
- How does a Brane forward search requests from its descendants upward?
- How does a Brane differ from an Expression in handling search messages?

### Value Computation

**The Problem**: How do we compute scalar values (numbers, booleans) as opposed to brane values?

**The Organ**: **Expression** (ProtoBrane without boundary)

An Expression is a proto-brane that computes a single scalar value. Unlike a Brane, it does not
create a scope boundary. It participates in evaluation but does not organize or contain other
values in a namespace.

**Key Distinctions from Brane**:

| Aspect | Brane | Expression |
|--------|-------|------------|
| Scope boundary | Yes — blocks search into children | No — search passes through |
| value() returns | The brane itself (first-class) | A scalar (number, boolean) |
| Member resolution | Helps members resolve | Does not contain members to resolve |
| Search into it | Blocked (must use search operators) | Not applicable (no interior namespace) |

**Example**: System operators like `🧠+` are expressions. `1 + 2` desugars to a concatenation
containing `🧠1`, `🧠2`, and `🧠+`. The `🧠+` expression:
- Does NOT permit search into it (it has no namespace)
- DOES help its context evaluate (it computes a value for its parent)

**Questions to Answer**:
- How does an Expression signal it has no boundary?
- How does an Expression access its operands (via parent indexing)?
- How does message handling differ between Expression and Brane?

### Context Extension

**The Problem**: How do we combine partial contexts to form complete ones?

**The Organ**: **Concatenation** `{...}{...}`

Concatenation is the mechanism by which Foolish resolves dependencies across brane boundaries.
When a brane references something it doesn't have locally, concatenation with another brane can
provide it.

**The Three Stages**:

1. **Isolation**: Children evaluate independently, without access to parent context. Unresolved
   identifiers become CONSTANIC (waiting for more context).

2. **Merge**: Statements from all children are combined into a single contiguous array.

3. **Re-evaluation**: The merged brane evaluates in the combined context. Previously-CONSTANIC
   identifiers may now resolve.

**Why CONSTANIC, not NK?**: An unresolved identifier becomes CONSTANIC (not NK/unknown) because
concatenation might provide the missing binding. NK means "provably unfindable"; CONSTANIC means
"may become findable with more context."

**Questions to Answer**:
- How does Concatenation block FulfillSearch during isolation?
- How does Concatenation signal the transition from isolation to merge?
- How does re-parenting work at the message level?

### Visibility Control

**The Problem**: How do we temporarily hide identifiers from the search path?

**The Organ**: **Detachment** `[a]{...}`

Detachment creates a filter in the search path. When an identifier is "detached," searches for it
are blocked from escaping the wrapped brane. This creates free variables — identifiers that are
locally unbound but may be bound later through concatenation.

**Key Behaviors**:

- `[a]{...}` blocks searches for `a` from escaping (M-brane: block mode)
- `[+a]{...}` allows searches for `a` to pass (P-brane: permit mode, used to re-attach)
- The filter is temporary — once the wrapped brane reaches CONSTANIC, the filter becomes inactive

**Example**: In `{x=1; [x]{x;}}`, the inner `x` cannot see the outer `x=1` because the detachment
blocks it. The inner `x` becomes CONSTANIC (free variable).

**Questions to Answer**:
- How does Detachment intercept FulfillSearch messages?
- How does Detachment decide which identifiers to block?
- How does Detachment signal it's no longer filtering after CONSTANIC?

---

## Section 2: ProtoBrane — The Shared Contract

[Continues with existing Section 1 content, renumbered...]

All ProtoBranes traverse:

```
PREMBRYONIC → EMBRYONIC → BRANING → constanic terminal state
```

Where constanic terminal states are: CONSTANIC, WOCONSTANIC, CONSTANT, or INDEPENDENT.

### PREMBRYONIC (Atomic Transition)

All actions occur as a single step:

1. **Count lines** — establish statement array size
2. **Establish statement array** — indexed from 0
3. **Build search cache** — array of fully-characterized identifier strings
4. **Instantiate RHS FIRs** — in PREMBRYONIC, not stepped
5. **Find searches in RHS** — stopping at brane boundaries; maintain writing order
6. **Append child branes** — to braneMind, in writing order
7. **Establish LUID** — locally unique identifier for message routing

→ Transition to EMBRYONIC.

### EMBRYONIC (Search Resolution)

- Resolve searches through local search, dereferencing, and message passing
- Dispatch FulfillSearch to parent for searches that can't be resolved locally
- Receive RespondToSearch from parent
- Search bandwidth limits work per step
- Wait-for queue for dependencies not yet constanic

→ Transition to BRANING when all SearchFirs have reached terminal states.

### BRANING (Child Execution)

- Step child branes from braneMind heap
- Continue communication (forward messages up/down)
- Monitor wait-for queue; trigger constanic cloning when dependencies settle

→ Transition to constanic terminal state when all work is complete.

### Core Invariants

| ID | Invariant | Description |
|----|-----------|-------------|
| C1 | Immutability after constanic | No fields may change once `isConstanic()` is true |
| C2 | CONSTANT tree invariant | CONSTANT FIR never has non-CONSTANT descendants |
| C3 | Constanic value stability | Value only changes via re-evaluation in new context |
| C4 | Parent chain integrity | parentFir not reassigned after setup (except constanicCloning) |

---

## Section 3: Role A — Normal Brane `{...}`

### What It Is

The curly-brace brane: the fundamental Foolish structure that holds and evaluates a sequence of
statements. This is the "full brane" — it has its own namespace, its own search scope, and its
own identity as a first-class value.

### How It Serves Foolish

**Encapsulation**: The Brane creates a boundary that:
- Blocks unspecific search from parent into children
- Requires explicit search operators to access interior values
- Prevents accidental name collisions across boundaries

**Organization**: The Brane manages its members by:
- Maintaining a statement array with writing-order semantics
- Coordinating evaluation of all members
- Forwarding unresolved searches upward through the parent chain
- Presenting itself as a coherent value to its parent

### Search Behavior

When a search originates inside a Normal Brane:

1. First search local statements (backward from cursor, writing-order precedence)
2. If not found locally, escalate to parent via FulfillSearch message
3. If parent doesn't have it, continue up the chain

This is the **only** role where local search happens before parent escalation.

### value() Semantics

A Normal Brane is a first-class value. When another expression evaluates `b = {...}`, the RHS
evaluates to the brane object itself, not to a scalar extracted from it. To get a scalar out of
a brane, you use search operators (`b?name`, `b^`, `b$`, `b#0`).

### Example: Scope Resolution

```foolish
{
    a = 10;
    b = 20;
    outer'{
        c = 30;
        a + c;          !! a found in parent, c found locally → 40
        inner'{
            a = 40;     !! shadows outer a
            a + b + c;  !! a=40 (local), b=20 (grandparent), c=30 (parent) → 90
        };
        a + b;          !! a=10 (parent, not shadowed here), b=20 → 30
    };
}
```

**What this demonstrates**:
- `outer'` defines a search boundary — `c` is found locally before searching parent
- `inner'` shadows `a` — the local `a=40` is found before the grandparent's `a=10`
- `b` is found in grandparent because neither `outer'` nor `inner'` define it locally

### Related Approval Tests

- [`emptyBraneIsApproved.foo`](../../test-resources/org/foolish/fvm/inputs/emptyBraneIsApproved.foo)
- [`nestedBranesIsApproved.foo`](../../test-resources/org/foolish/fvm/inputs/nestedBranesIsApproved.foo)
- [`complexIdentifierScopeIsApproved.foo`](../../test-resources/org/foolish/fvm/inputs/complexIdentifierScopeIsApproved.foo)
- [`identifierShadowingIsApproved.foo`](../../test-resources/org/foolish/fvm/inputs/identifierShadowingIsApproved.foo)

### Contract Summary: Normal Brane

| Aspect | Specification |
|--------|---------------|
| Boundary | Yes — searches check local first, then escalate |
| value() | Returns the brane itself (first-class value) |
| Search escalation | Local → parent via FulfillSearch |
| Depth tracking | Yes; NK on depth exceeded |
| Lifecycle | Full: PREMBRYONIC → EMBRYONIC → BRANING → constanic |

---

## Section 4: Role B — System Operator Brane

### What It Is

A proto-brane produced by desugaring infix/prefix operators. `1 + 2` desugars to the
concatenation `{🧠1, 🧠2, 🧠+}` where `🧠+` is the system operator. The operator waits for its
operands to become CONSTANT, then computes the result.

### How It Serves Foolish

**Value Computation**: System operators are the computational engine of Foolish. They:
- Take scalar operands from their context
- Perform arithmetic, logical, or comparison operations
- Return scalar results (not branes)

**Key Distinction**: Unlike Branes, Expressions do NOT create scope boundaries. They are
transparent to search — a search passing through an Expression continues as if it weren't there.

### How It Differs from Brane

| Aspect | Brane | Expression (System Operator) |
|--------|-------|------------------------------|
| Boundary | Blocks search into children | No boundary; search passes through |
| value() | Returns self (brane) | Returns scalar (computation result) |
| Members | Contains named statements | No namespace; no members |
| Search handling | Local search, then escalate | No local search; immediate escalation |
| Operand access | N/A | Via parent statement array indexing |

### Operand Access

System operators access their operands by indexing into the parent brane's statement array:

```java
// Binary operator (🧠+, 🧠-, 🧠*, 🧠/)
a, b = parent.getValuesBeforeMe(me, 2);
myValue = a + b;

// Unary operator (🧠−, 🧠!)
a = parent.getValueBeforeMe(me);
myValue = -a;
```

### Wait-for Semantics

If an operand is not yet CONSTANT (e.g., it's CONSTANIC because an identifier hasn't resolved),
the system operator enters its own wait-for state. It does not compute until all operands are
available.

### Terminal States

| Condition | Terminal State |
|-----------|----------------|
| All operands CONSTANT, computation succeeds | CONSTANT |
| One or more operands CONSTANIC | WOCONSTANIC |
| Arithmetic error (division by zero) | CONSTANT with NK value |

### Example: Simple Arithmetic

```foolish
{3 + 4;}
```

Desugars to `{🧠3, 🧠4, 🧠+}`. After evaluation:
```
{
＿7;
}
```

### Related Approval Tests

- [`simpleAdditionIsApproved.foo`](../../test-resources/org/foolish/fvm/inputs/simpleAdditionIsApproved.foo)
- [`chainedArithmeticIsApproved.foo`](../../test-resources/org/foolish/fvm/inputs/chainedArithmeticIsApproved.foo)
- [`zeroDivisionIsApproved.foo`](../../test-resources/org/foolish/fvm/inputs/zeroDivisionIsApproved.foo)

### Contract Summary: System Operator

| Aspect | Specification |
|--------|---------------|
| Boundary | No — transparent to search |
| value() | Returns scalar result |
| Operand access | Parent statement array indexing |
| Lifecycle | Standard; computation in BRANING |

---

## Section 5: Role C — ConcatenationBrane

### What It Is

ConcatenationBrane combines multiple branes into a single evaluation context. This is the
mechanism by which:
- Operators work (`1 + 2` → concatenated operands and operator)
- Branes gain new members (`{a=1}{b=2}` → `{a=1; b=2}`)
- Dependencies resolve across contexts (`{x=?}{x=1}` → `x` resolves)

### How It Serves Foolish

**Context Extension**: Concatenation is Foolish's mechanism for dependency resolution. When a
brane references something it doesn't have, concatenation with another brane can provide it.

### The Three Stages

```
Stage A (Isolation)     Stage B (Merge)          Stage C (Evaluation)
─────────────────────   ──────────────────────   ─────────────────────
Step children to        Clone & re-parent all    Normal brane evaluation
constanic in isolation  into merged brane        on merged result

Search isolation ON     Search isolation OFF     Parent search enabled
FulfillSearch blocked   Merged brane created     Merged brane steps
```

### Stage A: Isolation

1. Instantiate FIRs from source elements
2. Step all children until each reaches constanic in isolation
3. **FulfillSearch from children is NOT forwarded to parent**
4. Children reach CONSTANIC if they have unresolved identifiers (expected — concatenation will provide)

### Stage B: Merge

1. For each child brane (in writing order), constanic-copy its statements
2. Flatten all statements into a single contiguous array
3. Update parent references on cloned statements
4. Build merged brane with combined statement array and search cache

### Stage C: Evaluation

The merged brane runs the standard Normal Brane lifecycle. Previously-CONSTANIC expressions
re-resolve in the new combined context.

### Why CONSTANIC, Not NK?

This is critical: unresolved identifiers become CONSTANIC (may gain value) not NK (provably
unknown). Concatenation can provide the missing context.

### Example: Concatenation Resolving CONSTANIC

```foolish
{
    f = {a = x;};         !! x not found → a is CONSTANIC
    g = {x = 42;};
    h = g f;              !! Concatenation resolves a to 42
}
```

**Walk-through**:
1. `f` evaluates: `a = x` searches for `x`, doesn't find it → `a` is CONSTANIC
2. `g` evaluates: `x = 42` is fully resolved → CONSTANT
3. `h = g f` concatenates:
   - Stage A: Both children stepped in isolation
   - Stage B: Merge → `[x=42, a=x]`, clone `a=x` with new parent
   - Stage C: Re-evaluate `a = x`. Now `x` is found (it's `42`). `a` becomes `42`.

### Related Approval Tests

- [`concatenationBasics.foo`](../../test-resources/org/foolish/fvm/inputs/concatenationBasics.foo)
- [`concatenationSearch.foo`](../../test-resources/org/foolish/fvm/inputs/concatenationSearch.foo)

### Contract Summary: ConcatenationBrane

| Aspect | Specification |
|--------|---------------|
| Boundary | No during isolation; delegates after merge |
| value() | Returns the merged brane |
| Search isolation | ON during Stages A/B; OFF after merge |
| Three stages | A (isolate), B (merge), C (evaluate) |

---

## Section 6: Role D — Detachment Brane `[...]`

### What It Is

Detachment wraps a brane and filters search messages. `[a]{...}` blocks searches for `a` from
escaping the wrapped brane, creating a free variable.

### How It Serves Foolish

**Visibility Control**: Detachment is Foolish's mechanism for:
- Free variables (temporarily unbound identifiers)
- Parameterization (branes with placeholders)
- Late binding (resolution through concatenation)

### Filter Behavior

When a search for a detached identifier would normally escalate to the parent, the detachment
intercepts it and responds with "not found" — even if the parent *does* have a binding.

**Filter Syntax**:
- `[a]` or `[-a]` — Block searches for `a` (M-brane, default)
- `[+a]` — Unblock searches for `a` (P-brane, re-attachment)
- `[a,b,c]` — Block multiple identifiers
- `[~pattern]` — Block by regex pattern
- `[#N]` — Block by position index

### Filter is Temporary

Once the wrapped brane reaches CONSTANIC, the filter becomes inactive. Subsequent references
do not re-apply the detachment.

### Example

```foolish
{
    x = 1;
    y = [x]{x;};  !! x is detached → inner x is CONSTANIC (free)
}
```

The inner `x` becomes CONSTANIC because the detachment blocks it from seeing `x=1`.

### Related Approval Tests

- [`detachmentBasic.foo`](../../test-resources/org/foolish/fvm/inputs/detachmentBasic.foo)
- [`detachmentWithConcatenation.foo`](../../test-resources/org/foolish/fvm/inputs/detachmentWithConcatenation.foo)

### Contract Summary: Detachment

| Aspect | Specification |
|--------|---------------|
| Boundary | Varies (typically no boundary itself) |
| Filter | Intercepts FulfillSearch for specified identifiers |
| Temporary | Filter inactive after CONSTANIC |
| Syntax | `[id]` block, `[+id]` permit |

---

## Section 7: How Behaviors Map to Messages

[This section will detail the message protocol for each brane type's distinctive behaviors]

### Message Types

| Message | Direction | Purpose |
|---------|-----------|---------|
| FulfillSearch | Child → Parent | Request value for identifier |
| RespondToSearch | Parent → Child | Provide (or deny) identifier value |
| Step | Parent → Child | Advance child's evaluation |
| ValueReady | Child → Parent | Notify constanic state reached |

### Brane-Specific Message Handling

| Brane Type | FulfillSearch Handling |
|------------|----------------------|
| Normal Brane | Search local first; escalate if not found |
| Expression | No local search; immediate escalation |
| Concatenation | Block during isolation; forward after merge |
| Detachment | Filter based on identifier; block or forward |

---

## Section 8: Lessons Learned from UBC1

[Existing content on UBC1 lessons]

## Section 9: Design Proposal

[Existing content on design proposal]

---

## Last Updated

**Date**: 2026-03-07
**Updated By**: Claude Code / cyankiwi/Qwen3.5-27B-AWQ-BF16-INT8
**Changes**: Restructured document to lead with "What We Don't Know" — the problems to be resolved. Reframed the four brane types as "organs" of Foolish, each serving a vital function (Encapsulation, Computation, Context Extension, Visibility Control). Described behaviors in natural language rather than trait terms. Added explicit "Questions to Answer" for each feature to guide implementation.
