# FIR System Invariances and Assumptions

Comprehensive documentation of the Foolish Internal Representation (FIR) system constraints, contracts, and design decisions.

## Core Invariances

### C1: Immutability After Constanic
Once a FIR reaches `isConstanic()` (CONSTANIC or CONSTANT state), it becomes immutable. No fields may be modified after this point. Any attempt throws `IllegalStateException`.

### C2: Constant Tree Invariant
A CONSTANT FIR never has non-CONSTANT descendants. If a FIR is CONSTANT, ALL children are also CONSTANT.

**Consequences:**
- CONSTANT FIRs can be shared without cloning
- `cloneConstanic()` returns `this` for CONSTANT FIRs

### C3: Constanic Value Stability
A CONSTANIC FIR's value can only change due to coordination (re-evaluation in a new context via CMFir or ConcatenationFiroe).

**Details:**
- CONSTANIC branes' CONSTANT members remain accessible via search/getValue
- Only non-CONSTANT members may resolve differently in new context

### C4: Parent Chain Integrity
`parentFir` must not be reassigned after initial setup (enqueue or cloning) except during context manipulation:
- `CMFir.startPhaseB()`
- `ConcatenationFiroe.performJoin()` (Stage B)

### C5: Primed State Separation
When a FIR reaches `isConstanic()` (CONSTANIC or CONSTANT), `braneMind` MUST be empty.

**Why this matters:**
- Critical for `cloneConstanic()` which assumes empty braneMind
- The PRIMED state exists specifically to ensure this invariant
- Separates initialization (populate braneMemory) from evaluation (populate braneMind)

### C6: BraneMind Work Queue Invariant
`braneMind` only contains NYE FIRs (`isNye() == true`). Once a FIR is `!isNye()`, it is removed from braneMind (but stays in braneMemory).

### C7: BraneMemory Persistence
`braneMemory` is append-only during normal operation. Items are never removed from braneMemory during evaluation.

### C8: Ordination Requirement
Before braneMemory can resolve identifiers from parent context, `ordinateToParentBraneMind()` must be called exactly once. The `ordinated` flag tracks this.

---

## State Machine

### Nyes States (in order)
```
0. UNINITIALIZED   - Not yet initialized
1. INITIALIZED     - Sub-FIRs stored in braneMemory
2. CHECKED         - Non-brane expressions type-checked
3. PRIMED          - braneMind populated from braneMemory
4. EVALUATING      - Actively stepping sub-FIRs
5. CONSTANIC       - Done stepping, some values unresolved
6. CONSTANT        - Fully resolved, immutable
```

### State Transition Rules
- **No backward transitions**: new state ordinal must be >= current ordinal
- **CONSTANIC is immutable**: only transition to CONSTANT allowed
- **CONSTANT is terminal**: no transitions allowed except same state
- **Forward skips allowed**: e.g., UNINITIALIZED -> CONSTANT for ValueFiroe

---

## Method Contracts

### step()
```
PRE:  isNye() == true (or no-op if already evaluated)
POST: State advances or remains same; never regresses
RET:  0 for empty transitions (no-op, already evaluated, or waiting)
      1 for meaningful work (state transitions, evaluations, searches)
```

### getValue()
```
PRE:  isConstant() == true (throws otherwise)
POST: Returns numeric value; never modifies state
RET:  long value
```

### cloneConstanic()
```
PRE:  isConstanic() == true (throws otherwise)
POST: Returns this if CONSTANT; returns clone with updated parent if CONSTANIC
RET:  FIR (shared reference or new clone)
```

### setNyes()
```
PRE:  State transition must be valid per State Machine rules
POST: nyes field updated; IllegalStateException if invalid transition
```

---

## Key Classes

### FIR
Abstract base class for all Foolish Internal Representations. Holds an AST node and tracks evaluation progress.

### FiroeWithBraneMind
FIR subclass with braneMind work queue for breadth-first evaluation. Manages:
- Enqueueing sub-FIRs that need evaluation
- Stepping through NYE (Not Yet Evaluated) FIRs
- Dequeuing FIRs when they complete evaluation

### BraneMemory
Persistent storage for FIRs within a brane. Supports:
- Append-only storage
- Backward search from a position
- Parent chain traversal for identifier resolution

---

## Known Differences Between Java and Scala Implementations

### Step Counts
Scala and Java implementations may produce different step counts for the same input. This is expected and acceptable as long as final outputs match. Step count differences arise from internal implementation details.

### Known Output Differences (SKIPPED in Cross-Validation)
| File | Reason |
|------|--------|
| `concatenationBasics.approved.foo` | Scala doesn't have same concatenation handling |
| `concatenationResolutionAdv.approved.foo` | Scala produces more correct output for constanic brane concatenation |

---

## Assumptions

### Evaluation Order
- Java and Scala evaluate FIRs in breadth-first order via braneMind queue
- Non-brane expressions are stepped before branes during INITIALIZED -> CHECKED phase

### Identifier Resolution
- Identifier resolution searches backward from the current position
- Parent branes are searched after local brane (if identifier not found locally)

### Brane Concatenation
- Concatenation creates a single logical brane from multiple branes
- Elements are first evaluated to CONSTANIC in original context
- Then cloned and re-parented into concatenation's braneMemory
- Later elements can resolve identifiers from earlier elements

---
## Last Updated
**Date**: 2026-03-03
**Updated By**: Claude Code v1.0.0 / cyankiwi/Qwen3.5-27B-AWQ-BF16-INT8
**Changes**: Added anchor links, organized with markdown headers, expanded method contracts section
