# UBC Engineering Reference

> AI agents: read [../DOC_AGENTS.md](../DOC_AGENTS.md) before editing this file.

The Unicellular Brane Computer (UBC) is the reference implementation of the Foolish Virtual Machine.
This document consolidates the engineering details of the current UBC implementation (UBC1) as built
in the `foolish-core-java` and `foolish-core-scala` modules.

For the next-generation design, see [UBC2 Design Specification](ubc2_design.md).

---

## Table of Contents

- [Architecture Overview](#architecture-overview)
- [FIR Class Hierarchy](#fir-class-hierarchy)
- [Nyes State Machine](#nyes-state-machine)
- [BraneMind and BraneMemory](#branemind-and-branemory)
- [Brane Evaluation Cycle](#brane-evaluation-cycle)
- [Search and Query System](#search-and-query-system)
- [Identifier Resolution](#identifier-resolution)
- [Context Manipulation: CMFir](#context-manipulation-cmfir)
- [Depth Limiting](#depth-limiting)
- [Constraints and Invariants](#constraints-and-invariants)
- [State Checking Methods](#state-checking-methods)
- [Parallel Implementations](#parallel-implementations)

---

## Architecture Overview

The UBC implements breadth-first evaluation of Foolish programs. A Foolish `.foo` file is parsed
into an AST, which is then translated into Foolish Internal Representations (FIRs). Each FIR
progresses through a state machine until it reaches a terminal state: CONSTANIC or CONSTANT.

The core components are:

| Component | Class | Purpose |
|-----------|-------|---------|
| Brane evaluator | `BraneFiroe` | Evaluates a brane: loads AST statements, manages lifecycle |
| Base with queues | `FiroeWithBraneMind` | Provides braneMind (work queue) and braneMemory (storage) |
| Memory | `BraneMemory` | Append-only statement storage with hierarchical parent-chain search |
| Query | `Query` | Sealed interface: `StrictlyMatchingQuery` and `RegexpQuery` |
| Identifier | `IdentifierFiroe` | Variable resolution via memory search |
| Search | `AbstractSearchFiroe` | Base for brane search operations (`RegexpSearchFiroe`, `OneShotSearchFiroe`) |
| Context manip | `CMFir` | Two-phase "Stay Foolish" re-evaluation in new context |
| State enum | `Nyes` | State machine: UNINITIALIZED → ... → CONSTANT |

Source locations:
- Java: `foolish-core-java/src/main/java/org/foolish/fvm/ubc/`
- Scala: `foolish-core-scala/src/main/scala/org/foolish/fvm/scubc/`

---

## FIR Class Hierarchy

FIR (Foolish Internal Representation) objects represent expressions during evaluation. All FIRs
extend the abstract `FIR` base class. The key types are:

| FIR Type | Description |
|----------|-------------|
| `ValueFiroe` | Constants (integers, strings) |
| `NKFiroe` | "Not Known" values (`???`), optionally with comment explaining why |
| `BraneFiroe` | Evaluates brane expressions `{...}` |
| `AssignmentFiroe` | Variable bindings `name = expr` |
| `IdentifierFiroe` | Variable references with optional characterizations |
| `BinaryFiroe` | Binary operators (`+`, `-`, `*`, `/`, `%`, etc.) |
| `UnaryFiroe` | Unary operators (`-x`) |
| `IfFiroe` | Conditional expressions `if ... then ... else ...` |
| `OneShotSearchFiroe` | Head (`^`) and tail (`$`) search |
| `RegexpSearchFiroe` | Pattern-based search (`.`, `?`, `/`) |
| `SearchUpFiroe` | `↑` operator for upward scope traversal |
| `CMFir` | Context manipulation wrapper for re-evaluation |
| `DetachmentBraneFiroe` | Liberation/detachment branes `[...]` |

Each FIR holds:
- A reference to its originating AST node (immutable, shared)
- A `Nyes` state tracking evaluation progress
- A `parentFir` reference to its containing FIR
- A statement number within its containing brane

---

## Nyes State Machine

The Nyes enum tracks each FIR's evaluation progress:

```
UNINITIALIZED → INITIALIZED → CHECKED → PRIMED → EVALUATING → CONSTANIC → CONSTANT
```

| State | Meaning |
|-------|---------|
| `UNINITIALIZED` | Raw AST, no initialization performed |
| `INITIALIZED` | FIR structure created from AST, caches ready |
| `CHECKED` | Type/reference checking complete (currently pass-through, reserved for future) |
| `PRIMED` | Non-constant items enqueued into braneMind |
| `EVALUATING` | Active stepping: dequeue from braneMind, step, re-enqueue if still nigh |
| `CONSTANIC` | Constant In Context — paused due to missing information. Terminal for this context |
| `CONSTANT` | Fully evaluated, immutable. Terminal |

The state machine was simplified from an earlier design that had intermediate states
REFERENCES_IDENTIFIED, ALLOCATED, and RESOLVED between INITIALIZED and EVALUATING. These were
condensed into the single CHECKED state.

The generic term *constanicity* (lowercase, pronounced "con-stan-ISS-ity") refers to the condition
of being constant in context. An FIR has constanicity when it is in any terminal state
(CONSTANIC, WOCONSTANIC, CONSTANT, or INDEPENDENT) — no longer stepping. The predicate
`hasConstanicity` means `at_constanic || at_woconstanic || at_constant || at_independent`.

---

## BraneMind and BraneMemory

### BraneMind (Work Queue)

`braneMind` is a `LinkedList<FIR>` that acts as a FIFO work queue. It drives breadth-first
evaluation:

1. `prime()`: Enqueue all non-CONSTANT items from braneMemory
2. Each step: dequeue front item, call `step()`, re-enqueue at back if still nigh
3. When queue empties: check if all items are CONSTANT (→ CONSTANT) or any are CONSTANIC (→ CONSTANIC)

Invariant C6: only nigh FIRs may be in the braneMind.
Invariant C5: at CONSTANIC state, braneMind must be empty.

### BraneMemory (Persistent Storage)

`BraneMemory` is an append-only list of FIRs representing the statements of a brane. It provides:

- Indexed access: `get(int index)` — 0-based statement position
- Backward search: `get(Query, fromLine)` — searches backward from `fromLine`, then up parent chain
- Local backward search: `getLocal(Query, fromLine)` — backward within this brane only
- Local forward search: `getLocalForward(Query, fromLine)` — forward within this brane only
- Parent chain: hierarchical link to parent brane's memory for scope resolution

Invariant C7: BraneMemory is append-only; items are never removed during evaluation.

### Storage Operations

When a FIR is stored in braneMemory:
1. Appended to memory list
2. `parentFir` set to the containing brane
3. Index recorded in `indexLookup` map
4. If the FIR is a `FiroeWithBraneMind`, it is ordinated to the parent brane's memory

---

## Brane Evaluation Cycle

`BraneFiroe.step()` drives the brane through its lifecycle:

1. **UNINITIALIZED → INITIALIZED**: `initialize()` converts AST.Brane statements into FIR objects
   stored in braneMemory. Each `AST.Expr` becomes a FIR via `createFiroeFromExpr()`.

2. **INITIALIZED → CHECKED**: Step all non-brane expressions until they reach CHECKED state.

3. **CHECKED → PRIMED**: `prime()` enqueues non-constant items from braneMemory into braneMind.

4. **PRIMED → EVALUATING**: Transition state.

5. **EVALUATING**: Main loop:
   - If braneMind is empty: transition to CONSTANT (all done) or CONSTANIC (some items stuck)
   - Otherwise: dequeue front FIR, step it, re-enqueue if still nigh

6. **CONSTANIC or CONSTANT**: Terminal. `step()` returns 0.

Because the braneMind is a FIFO queue and branes are finite, each step terminates in finite time,
and the brane tree is evaluated breadth-first.

---

## Search and Query System

### Query Types

The `Query` sealed interface has two implementations:

- `StrictlyMatchingQuery`: exact identifier match. Checks if a brane line is an
  `AssignmentFiroe` whose LHS matches the query identifier (including characterization chain).
- `RegexpQuery`: pattern-based match. Auto-anchors patterns with `^...$` for whole-identifier
  matching.

### Search Operators

| Operator | Type | Direction | Scope |
|----------|------|-----------|-------|
| `B.x` | Backward anchored | End → Start | Within B only |
| `B?pattern` | Backward anchored | End → Start | Within B only |
| `B/pattern` | Forward anchored | Start → End | Within B only |
| `B^` | Head | First element | Within B only |
| `B$` | Tail | Last element | Within B only |
| `B#N` | Index | Nth element | Within B only |
| `x` (identifier) | Level-skipping | Backward + parents | Full scope chain |
| `?pattern` | Unanchored | Backward + parents | Full scope chain |

### NK vs CONSTANIC Search Results

- Anchored searches return `???` (NK) on failure — the brane is fully known and the name is
  simply not there.
- Level-skipping searches produce CONSTANIC on failure — the identifier might be found when the
  brane is recoordinated into a new context.

### Search Implementation

`AbstractSearchFiroe` drives search evaluation:
1. Step anchor expressions until ready
2. Unwrap anchor (dereference IdentifierFiroe, AssignmentFiroe, etc.)
3. Call `executeSearch(BraneFiroe target)` — implemented by subclasses
4. Handle result: NK → CONSTANT with `???`; found → follow result state

---

## Identifier Resolution

`IdentifierFiroe` resolves variable references:

1. **INITIALIZED**: Search for identifier in braneMemory hierarchy via `memoryGet(query, fromLine)`.
   - Not found → CONSTANIC (might resolve in future context)
   - Found → proceed to CHECKED with reference to found FIR

2. **CHECKED**: Monitor the resolved FIR's state:
   - Target at CONSTANIC → this identifier also CONSTANIC
   - Target at CONSTANT → this identifier CONSTANT
   - Target still nigh → EVALUATING (wait for it)

3. **EVALUATING**: Continue monitoring until target settles.

Characterization support: identifiers may be qualified as `type'name`, forming a chain used for
disambiguation during search.

---

## Context Manipulation: CMFir

CMFir implements "Stay Foolish" behavior — re-evaluating code defined in one scope within a
different scope (dynamic scoping). It has two phases:

### Phase A: Step Original

Step the original FIR (`o`) until it reaches a terminal state:
- If `o` reaches CONSTANIC: enter Phase B (needs re-evaluation in new context)
- If `o` reaches CONSTANT: done (fully resolved in original context, no re-evaluation needed)

### Phase B: Clone and Re-evaluate

1. `stayFoolishClone(o)` → `o2` with state reset to INITIALIZED
2. Link `o2`'s braneMemory parent to CMFir's containing brane
3. Step `o2` in the new context
4. CMFir mirrors `o2`'s state

### State Delegation

CMFir delegates state queries to the active phase's FIR:
- Before Phase B: state comes from `o`
- During Phase B: state comes from `o2`

Use `atConstanic()` (exact match), not `hasConstanicity()` (≥ CONSTANIC), when deciding whether
to start Phase B. The latter includes CONSTANT, which would incorrectly trigger re-evaluation of
already-resolved expressions.

---

## Depth Limiting

`BraneFiroe` tracks nesting depth to prevent infinite recursion:

- Root brane: depth 0
- Each nested `BraneFiroe` increments depth by 1
- Maximum depth: 96,485
- At limit: brane terminates immediately with `???` value and raises a MILD alarm

Depth is calculated by walking the `parentFir` chain and counting `BraneFiroe` ancestors.

---

## Constraints and Invariants

| ID | Constraint | Description |
|----|-----------|-------------|
| C5 | PRIMED STATE SEPARATION | At CONSTANIC, braneMind MUST be empty |
| C6 | BRANEMIND WORK QUEUE | Only nigh FIRs in queue |
| C7 | BRANEMEMORY PERSISTENCE | Append-only, never removes items |
| C8 | ORDINATION REQUIREMENT | Parent link must be established before identifier resolution |

---

## State Checking Methods

| Method | Returns True When | Use Case |
|--------|------------------|----------|
| `atConstant()` | `nyes == CONSTANT` | Exact check: fully evaluated |
| `atConstanic()` | `nyes == CONSTANIC` | Exact check: paused, awaiting context |
| `isConstant()` | `nyes >= CONSTANT` | At least CONSTANT (just CONSTANT today) |
| `hasConstanicity()` | `nyes >= CONSTANIC` | Has constanicity (CONSTANIC, WOCONSTANIC, CONSTANT, or INDEPENDENT) |
| `isNigh()` | `nyes < CONSTANIC` | Still evaluating (no constanicity yet) |

Use `at*` methods for exact state checks. Use `hasConstanicity()` for "are we done?" checks.

---

## Parallel Implementations

The UBC is implemented in both Java and Scala:

| Aspect | Java (`ubc`) | Scala (`scubc`) |
|--------|-------------|-----------------|
| Base package | `org.foolish.fvm.ubc` | `org.foolish.fvm.scubc` |
| State enum | `Nyes.java` | `FiroeState.scala` |
| Brane eval | `BraneFiroe.java` | `BraneFiroe.scala` |
| Tests | `UbcApprovalTest.java` | `ScUbcApprovalTest.scala` |

Both implementations share:
- The same AST (Java records in `foolish-parser-java`)
- The same ANTLR grammar (`Foolish.g4`)
- The same test input files (`.foo` programs)

Both implementations must produce byte-identical approval test outputs. Cross-validation tests in
`foolish-crossvalidation` enforce this.

---

## Last Updated

**Date**: 2026-03-12
**Updated By**: Claude Code / cyankiwi/Qwen3.5-27B-AWQ-BF16-INT8
**Changes**: Updated terminology: replaced "achievedConstanic" with "hasConstanicity" to align with
the new term "constanicity" (pronounced "con-stan-ISS-ity"). Updated State Checking Methods table.
Previous (2026-02-26): Reduced emphatic markings throughout — removed bold from invariant labels,
definition list items, running-prose callouts, and search result descriptions. Prose now relies on
sentence structure and capitalized state names rather than typographic emphasis.
Previous (2026-02-16): Initial creation from ECOSYSTEM.md, UBC_FEATURES.md, search-semantics.md,
004-nyes-state-simplification.md, and 008-cmfir-nyes-state-review.md.
