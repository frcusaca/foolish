# D0.0 — ProtoBrane Lifecycle

> AI agents: read [../../DOC_AGENTS.md](../../DOC_AGENTS.md) before editing this file.

This is the foundation document. All other d0 sections (d0_1 through d0_5) build on the ProtoBrane
lifecycle defined here. Read this first.

---

## The Central Mechanisms

All branes and proto-branes revolve around three fundamental mechanisms:

1. **Nyse (the lifecycle state)** — The state machine governing evaluation:
   ```
   PREMBRYONIC → EMBRYONIC → BRANING → constanic terminal state
   ```

2. **step()** — The computation performed each cycle. A brane's behavior in each state is what
   `step()` computes when `nyes` is in that state.

3. **handle(msg)** — The message handling system. During stepping, branes receive and respond to
   messages from parent, children, or peers.

> **The central question: For each lifecycle stage, what does step() compute and what messages flow?**

---

## Lifecycle States

| State | Meaning |
|-------|---------|
| **PREMBRYONIC** | Holding an AST; transitioning to structured representation |
| **EMBRYONIC** | Actively resolving searches through local work and message passing |
| **BRANING** | Child execution and message forwarding; monitoring dependencies |
| **CONSTANIC** | Reached a terminal state; value is stable (may change in new context) |
| **WOCONSTANIC** | Constanic with dependencies on other CONSTANIC branes |
| **CONSTANT** | Constanic with all CONSTANT descendants; immutable |
| **INDEPENDENT** | Detached from parent; no longer participates in parent's lifecycle |

---

## PREMBRYONIC Stage

### What Happens

A FIR in PREMBRYONIC holds a Foolish AST. The transition to EMBRYONIC is atomic — all actions
occur as a single step, not sequential substeps.

### Actions (All Atomic)

| # | Action | Purpose |
|---|--------|---------|
| 1 | **Count lines** | Establish statement array size |
| 2 | **Establish statement array** | Create indexed array from AST |
| 3 | **Build search cache** | Cache fully-characterized identifier names for regex matching |
| 4 | **Instantiate RHS FIRs** | Create child FIRs in PREMBRYONIC (do not step yet) |
| 5 | **Append child branes** | Add PREMBRYONIC branes to braneMind heap in writing order |
| 6 | **Establish LUID** | Locally unique identifier for message routing |

### What Does NOT Happen in PREMBRYONIC

- Searches are **not** found in PREMBRYONIC (that happens in EMBRYONIC)
- Searches are **not** resolved in PREMBRYONIC
- Searches are **not** dispatched to parents
- No message passing occurs

### Transition to EMBRYONIC

Upon completing the atomic PREMBRYONIC actions, the FIR transitions to EMBRYONIC:
```java
nyes = EMBRYONIC;
```

---

## EMBRYONIC Stage

### What Happens

The proto-brane actively resolves its searches. This is where identifier resolution occurs.

### step() in EMBRYONIC

Each call to `step()` when `nyes == EMBRYONIC`:

1. **Find searches in RHS** — Walk all RHS expressions and identify SearchFir instances
   - Stop at brane boundaries (searches inside nested branes belong to those branes)
   - Maintain writing order for precedence (first-to-write-first-to-find)
   - This is the first thing an Embryonic brane does — it discovers what searches it must resolve

2. **Handle inbound messages** — Process any `RespondToSearch` messages from parent
   - Apply results to unresolved searches
   - Forward results to child expressions that requested them

3. **Process wait-for queue** — Re-check expressions waiting on nigh dependencies
   - Call `resolve()` again (idempotent)
   - If target state changed (nigh → CONSTANIC), update accordingly
   - Remove finalized searches from wait-for queue

3. **Dispatch searches** — For up to `search_bandwidth` unresolved searches:
   - Attempt local resolution (for branes with boundary)
   - If not found locally, send `FulfillSearch` message to parent
   - Track outstanding search count

4. **Check transition** — If all searches resolved and wait-for queue empty:
   ```java
   nyes = BRANING;
   ```

### Message Handling: handle(FulfillSearch)

When the brane receives a `FulfillSearch` message from a child:

1. Attempt to resolve the search locally using cursor/query system
2. If found: send `RespondToSearch` back to the child
3. If not found: forward to parent (subject to parent's bandwidth)

### Message Handling: handle(RespondToSearch)

When the brane receives a `RespondToSearch` message from parent:

1. Match response to outstanding search using LUID
2. Apply the result to the requesting expression
3. If result is nigh: add to wait-for queue
4. If result is CONSTANIC/CONSTANT: may trigger constanic cloning

> **Note**: The `handle(RespondToSearch)` handler is separate from the general message handling
> because detachment branes alter this behavior — they may block certain responses or modify
> how results are applied based on their filter rules.

### Search Bandwidth

To prevent runaway evaluation, only a constant number of searches are dispatched per step:
```java
int search_bandwidth = 4;  // Constant, not configurable
```

Each step dispatches at most `search_bandwidth` new `FulfillSearch` messages.

### Writing Order Precedence

When multiple searches could find the same identifier, **first-to-write wins**. The order the
programmer wrote the code establishes precedence. Searches are maintained in writing order and
dispatched in that order.

---

## BRANING Stage

### What Happens

The proto-brane completes its child execution and monitors dependencies.

### step() in BRANING

Each call to `step()` when `nyes == BRANING`:

1. **Step child branes** — Pop from braneMind heap and step each child
   - Children may be in PREMBRYONIC, EMBRYONIC, BRANING, or constanic states
   - Heap ordering ensures correct scheduling by state and rank

2. **Continue handling search messages** — Same as EMBRYONIC:
   - Handle `FulfillSearch` from children (forward to parent if needed)
   - Handle `RespondToSearch` from parent (apply results to children)
   - This continues the search handling started in EMBRYONIC

3. **Monitor wait-for queue** — Check if dependencies have become constanic
   - Trigger constanic cloning if needed
   - Update expressions when dependencies settle

### Transition to Constanic Terminal State

When all work is complete:

- If all dependencies CONSTANT → transition to **CONSTANT**
- If dependencies CONSTANIC → transition to **WOCONSTANIC**
- If error → transition to **CONSTANT** with NK value

---

## Message Protocol

### Message Types

| Message | Direction | Purpose |
|---------|-----------|---------|
| `FulfillSearch` | Child → Parent | Request value for identifier |
| `RespondToSearch` | Parent → Child | Provide (or deny) identifier value |

### Message Structure

```java
class FulfillSearch {
    LUID requesterLUID;     // Who is asking
    String identifier;      // What they're looking for
    boolean anchored;       // Is this an anchored search?
    int depth;              // Search depth (for depth-limited searches)
}

class RespondToSearch {
    LUID requesterLUID;     // Who asked
    FIR result;             // The found value (or NK)
    boolean isNigh;         // Is the result still evaluating?
}
```

### Message Routing

Messages are routed using the LUID (Locally Unique Identifier):
- Each expression receives a unique LUID within its brane
- Parent uses LUID to route `RespondToSearch` to correct child
- Child uses LUID to correlate responses with outstanding searches

---

## Core Invariants

| ID | Invariant | Description |
|----|-----------|-------------|
| C1 | Immutability | No fields change once `isConstanic()` is true |
| C2 | CONSTANT tree | CONSTANT FIR never has non-CONSTANT descendants |
| C3 | Value stability | Value only changes via re-evaluation in new context |
| C4 | Parent integrity | `parentFir` not reassigned after setup (except constanic cloning) |

---

## The braneMind Heap

The `braneMind` is a min-heap that schedules evaluation. Key:

```
key = (state, cur_rank, intra_statement_order, fir)
```

- **state** — PREMBRYONIC < EMBRYONIC < BRANING < constanic
- **cur_rank** — Round-robin position (initially statement number)
- **intra_statement_order** — Position within statement (for expressions)
- **fir** — The FIR reference (tiebreaker)

Smallest key is on top. When top is not in current state, that state is complete for descendant FIRs.

---

## Next Steps

After reading this foundation:
- **d0_1_brane.md** — Branes with search boundaries
- **d0_2_system_operator.md** — System operators (🧠+, 🧠-, etc.)
- **d0_3_concatenation.md** — Concatenation semantics
- **d0_4_detachment.md** — Detachment filters
- **d0_5_brane_recoordination.md** — Recoordination after cloning

---

## Last Updated

**Date**: 2026-03-08
**Updated By**: Claude Code / cyankiwi/Qwen3.5-27B-AWQ-BF16-INT8
**Changes**: Moved "Find searches in RHS" from PREMBRYONIC to EMBRYONIC stage (first action).
Updated message handling to note separate RespondToSearch handler for detachment branes.
Clarified that BRANING continues search message handling from EMBRYONIC.
