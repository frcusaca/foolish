---
foop: 32
title: UBCb — Message-passing brane computer variant; SPA1 parity plan
author: hc <hc.busy@gmail.com>
status: Draft
type: Milestone
created: 2026-05-07
phase: meta
supersedes: []
---

# FOOP-(41): UBCb — Message-passing brane computer variant; SPA1 parity plan

## Abstract

Defines **UBCb** (Unicellular Brane Computer beta), a message-passing variant of the
Foolish VM. Unlike UBC (the reference implementation using depth-first function-call
stepping), UBCb evaluates branes via an asynchronous message-passing protocol.

UBCb is a **separate development track** from SPA1. It starts with a subset of UBC's
capabilities and grows toward full parity. At each parity checkpoint, UBCb must match
UBC's approval test results.

## Motivation

The UBC2 design documents (`docs/ubc1/how/ubc2_design.md`, `ubc2_message_protocol.md`)
describe a message-passing architecture for brane evaluation. This architecture offers
benefits that the reference UBC does not:

- **Natural concurrency**: independent subtrees can make progress simultaneously.
- **Decoupled components**: branes communicate via messages, not shared mutable state.
- **Breadth-first by design**: message scheduling naturally produces breadth-first
  evaluation order.
- **Better observability**: message channels provide natural hooks for debugging and
  visualization (feeding Phase 6's web brane browser).

However, implementing UBCb alongside UBC would complicate the initial SPA1 milestone.
UBCb is therefore a separate effort with its own development timeline.

## UBCb Architecture Overview

UBCb follows the UBC2 design specification:

- **Communication Medium**: branes communicate via typed messages through message channels
  (see `ubc2_message_protocol.md`).
- **LUIDs**: each brane and FIR has a Locally Unique Identifier for message routing.
- **FIR Lifecycle**: same Nyes states as UBC, but transitions are triggered by incoming messages, not
  by direct function calls.
- **Protobranes**: literal values and boundary-less expressions (per `d0_0_protobrane.md`).

### Key Differences from UBC

| Aspect | UBC (reference) | UBCb (message-passing) |
|--------|----------------|----------------------|
| Evaluation order | Depth-first within brane, breadth-first via FIFO braneMind | Breadth-first across entire proto-brane tree, stage-wise fairness |
| State mutation | In-place via `Rc<RefCell<Fir>>` | Message-driven state transitions |
| Constanic cloning | Synchronous, at search resolution (CMFir) | Asynchronous, triggered by recoordination messages |
| Search resolution | Immediate (depth-first guarantee, direct memory traversal) | Deferred (FulfillSearch/RespondToSearch message exchange) |
| Concurrency | None (single-threaded) | Natural (independent subtrees) |
| Architecture | Function-call stepping | Message-passing protocol |

### UBC Step Taxonomy (Research: 2026-05-08)

Analysis of UBC's evaluation loop reveals seven categories of work per step. Each category maps to a different UBCb requirement:

| Category | UBC Mechanism | UBCb Mechanism | Constant-Time? |
|----------|-------------|----------------|----------------|
| **1. BRANING** (create branches) | Convert AST → FIRs, populate braneMemory | Spawn proto-branes, assign LUIDs | Yes — O(statement count) |
| **2. SCHEDULING** | FIFO braneMind queue | Fair stage-wise work distribution | Yes — bounded by brane size |
| **3. SEARCHES** (introspective) | Internal BraneMemory lookup (parent chain walk) | FulfillSearch → RespondToSearch message exchange | **NO** — message round-trip, depends on tree depth |
| **4. BUILTIN COMPUTATION** | Step operands, compute | Wait for StateChange on operands, compute | Yes — once operands are constant |
| **5. BLOCKED** (CONSTANIC/WOCONSTANIC) | Empty braneMind, pause | Wait-for queue + StateChange registration | N/A — no work, just waiting |
| **6. CLONE + RECOORD** (CMFir) | Clone FIR, rewire parent chain | Clone proto-brane, rewire message channels | **NO** — recursive clone, O(subtree size) |
| **7. CONCATENATION** | Merge at parse time | ConcatenationBrane with search isolation | **NO** — merge and re-evaluate merged brane |

## UBCb Development Phases

UBCb development is organized into parity checkpoints, each targeting a subset of
UBC's capabilities:

### UBCb-Checkpoint-0: Parser and FIR only

| Goal | Share UBC's parser, compiler, and FIR algebra |
|------|----------------------------------------------|
| Scope | Phase 1 (compiler) — no evaluation |
| Delivered | Same FIR JSON contract as UBC |
| Governing FOOPs | FOOP-2, FOOP-4, FOOP-5, FOOP-9, FOOP-12 |

UBCb shares the parser and compiler infrastructure with UBC. The FIR algebra is
identical (same serde schema). UBCb adds message-passing fields (LUID, message
queue) to the FIR types.

### UBCb-Checkpoint-1: Basic evaluation

| Goal | Evaluate simple branes (literals, identification, arithmetic) |
|------|---------------------------------------------------------------|
| Scope | Phase 2 subset — no search, no constanic cloning |
| Delivered | UBCb can evaluate `{x = 42; y = x + 8}` |
| New FOOPs | UBCb message protocol for brane stepping |

At this checkpoint, UBCb can handle branes with no unresolved searches. The message
protocol handles:
- Literal value propagation
- Identification resolution within a single brane
- Arithmetic reduction (when all operands are constant)

### UBCb-Checkpoint-2: Search and constanic coordination

| Goal | Handle searches, short-circuiting, and constanic cloning |
|------|----------------------------------------------------------|
| Scope | Full Phase 2 — all UBC features including SF/SFF, seeks |
| Delivered | UBCb passes all 60+ Phase 2 approval tests |
| New FOOPs | Message protocol for search resolution, wake-up messages |

This is the hardest checkpoint: UBCb must replicate UBC's constanic coordination
using asynchronous messages instead of depth-first guarantees. Wake-up messages
are essential: when a search resolves to ECONSTANIC, a later message makes that
ECONSTANIC become CONSTANT, and the dependent search must be woken up.

### UBCb-Checkpoint-3: Concatenation

| Goal | Implement concatenation with message-driven recoordination |
|------|-----------------------------------------------------------|
| Scope | Phase 3 — concatenation operator |
| Delivered | UBCb passes all Phase 3 approval tests |
| New FOOPs | Message protocol for concatenation merge |

### UBCb-Checkpoint-4: Full SPA1 parity

| Goal | UBCb matches UBC on all approval tests |
|------|----------------------------------------|
| Scope | All Phases 1–4 |
| Delivered | UBCb passes the complete SPA1 test suite |

At this checkpoint, UBCb is feature-complete with UBC (though architecturally
different). Both implementations produce identical output on all approval tests.

## Governing FOOPs for UBCb

| FOOP | Description | Checkpoint |
|------|-------------|------------|
| FOOP-2 | Remove if-then-else | CP-0 |
| FOOP-4 | Bare identifiers → SearchFirs | CP-0 |
| FOOP-5 | FIR contract | CP-0 |
| FOOP-9 | OperatorFir | CP-0 |
| FOOP-12 | Alarms | CP-0 |
| FOOP-6 | Depth-first evaluator (UBC only; UBCb is breadth-first by design) | — |
| FOOP-7 | Constanic Clone contract | CP-2 |
| FOOP-8 | FIR mutability | CP-0 |
| FOOP-10 | Anchored search through constanic anchors | CP-2 |
| FOOP-11 | Search stops at NK | CP-2 |
| FOOP-3 | Concatenation algorithm | CP-3 |
| **New FOOPs** (to be written) | UBCb message protocol specifications | CP-1 through CP-4 |

Note: FOOP-6 (depth-first) governs UBC specifically. UBCb does NOT follow depth-first
ordering — it is breadth-first by design. UBCb's evaluation order is governed by its
message-passing protocol, not by FOOP-6.

## New FOOPs Needed for UBCb

The following FOOPs need to be written for UBCb:

1. **UBCb Message Protocol** — Define the message types, channels, and scheduling
   algorithm. Covers brane stepping, search resolution, and wake-up messages.
2. **UBCb Constanic Coordination** — Define how constanic cloning and recoordination
   work asynchronously. Covers the wake-up queue and dependency tracking.
3. **UBCb Concatenation Protocol** — Define how concatenation merge works via
   message-passing (vs UBC's synchronous merge).

These FOOPs are deferred until UBCb-Checkpoint-0 is complete and the development
team is ready to begin UBCb implementation.

## Test Plan

UBCb validates against UBC's approval test suite at each checkpoint:

- **CP-0**: FIR roundtrip tests pass (shared with UBC).
- **CP-1**: UBCb produces identical output to UBC on literal-only branes.
- **CP-2**: UBCb passes all 60+ Phase 2 approval tests.
- **CP-3**: UBCb passes all Phase 3 concatenation tests.
- **CP-4**: UBCb passes the complete SPA1 test suite.

At each checkpoint, the cross-validation module compares UBCb's output against UBC's
approved baselines byte-for-byte.

## Research Log

### 2026-05-08: UBC Step Taxonomy and NYES Stage Analysis

> **WARNING: BDFL reviewed this analysis and decided to alter the stage-wise fairness model in FOOP-31.** The findings below are retained as background research documenting the problem space. **They are EXPECTED TO CHANGE before FOOP-(41) implementation begins.** Consult FOOP-31 for the current authoritative design direction.

Detailed analysis of UBC's evaluation loop across `ubc_engineering.md`, `ubc2_design.md`, `ubc2_message_protocol.md`, and the Rust implementation (`foolish-core/src/ubc.rs`, `fir.rs`) produced the seven-category step taxonomy above and the following detailed NYES stage analysis.

#### NYES State Machine Overview

The Rust implementation defines these states (`fir.rs`):
```
PREMBRYONIC → EMBRYONIC → BRANING → ECONSTANIC / WOCONSTANIC / CONSTANT / INDEPENDENT / NK
```

The main stepping loop is `run_to_completion_with_scope()` (`ubc.rs:133`): it calls `step_with_scope()` repeatedly until state doesn't change or a terminal state is reached, with a safety limit of 100,000 iterations. For branes, `re_step_brane_bodies()` (`ubc.rs:216`) resets all searches to Embryonic, builds a scope chain, steps each statement body to completion via `step_boxed()`, then recomputes the brane state.

#### NYES Stage-by-Stage Breakdown (Background Research)

**PREMBRYONIC → EMBRYONIC (NormalBraneFir)**

| Aspect | Detail |
|--------|--------|
| **UBC behavior** | Single atomic state flag flip in `NormalBraneFir::step_one()` (`fir.rs:1068-1069`). No actual work. AST→FIR conversion happens in the compiler before the brane enters the UBC. |
| **What work is deferred** | In UBC2 design, PREMBRYONIC does: count lines, establish statement array, build search cache, instantiate RHS FIRs, find all searches, append child branes. In the Rust impl, most of this is done by the compiler; the UBC just flips the state flag. |
| **Synchronization** | None. No other FIRs are involved. |
| **Constant-time** | **Yes** — O(1) state flip. The deferred UBC2 work is O(statements) but bounded by source line count. |
| **UBCb mapping** | "Spawn proto-brane" step: assign LUID, create statement array, build search cache. Covered by CP-0. |

**EMBRYONIC → BRANING (NormalBraneFir)**

| Aspect | Detail |
|--------|--------|
| **UBC behavior** | Another state flag flip (`fir.rs:1070-1072`). The Rust impl collapses what UBC2 calls "EMBRYONIC search resolution" into the BRANING phase — `re_step_brane_bodies()` does search resolution and child stepping together. |
| **What work is deferred** | In UBC2 design, EMBRYONIC resolves all searches via message exchange before transitioning to BRANING. The Rust impl does this lazily inside BRANING. |
| **Synchronization** | None. |
| **Constant-time** | **Yes** — O(1) state flip. |
| **UBCb mapping** | Boundary between "search resolution" and "child stepping." UBCb must separate these — search resolution happens via `FulfillSearch`/`RespondToSearch` message exchange, child stepping happens after all searches are resolved. |

**BRANING (NormalBraneFir, repeated steps)**

| Aspect | Detail |
|--------|--------|
| **UBC behavior** | `re_step_brane_bodies()` (`ubc.rs:216-255`) is the workhorse. It: (1) clones all statements and resets searches to Embryonic, (2) builds scope chain by pushing named statements, (3) steps each statement body to completion via `step_boxed()` → `run_to_completion_with_scope()`, (4) recomputes brane state. |
| **Stepping strategy** | **Depth-first.** Each statement is stepped to completion (line 237-249: sequential loop) before the next statement starts. This is the key difference from UBCb's intended stage-wise fairness. |
| **Synchronization** | Depth-first nesting. `run_to_completion_with_scope()` recursively calls `step_boxed()` which calls `run_to_completion_with_scope()` again. This creates deep call stacks. |
| **Constant-time** | **NO.** Multiple unbounded operations. |
| **UBCb mapping** | Must be decomposed into EMBRYONIC micro-steps (resolve searches, up to bandwidth per step) and BRANING micro-steps (step children, up to bandwidth per step). |

**BRANING Sub-operations — Constant-Time Analysis:**

| Sub-operation | UBC Location | Cost | Bounded? | UBCb Solution |
|--------------|-------------|------|----------|---------------|
| Reset all searches | `reset_searches()` (`ubc.rs:259`) | O(statements) | Yes — bounded by source line count | Same |
| Build scope chain | `re_step_brane_bodies()` line 230-233 | O(statements) | Yes — bounded by source line count | Same |
| Step one statement to completion | `step_boxed()` → `run_to_completion_with_scope()` (`ubc.rs:133-157`) | **Unbounded** — tight loop up to 100,000 iterations, each iteration recurses | **NO** | Step-count limit (MESSAGE_BANDWIDTH) per step |
| Search scope chain walk | `Scope.search()` (`ubc.rs:95-101`) | O(scope depth) | Yes — bounded by source line count | Replace with message exchange |
| Constanic clone | `constanic_clone()` (`ubc.rs:458-493`) | O(subtree size) — recursive clone | Yes — bounded by subtree | Same, but async |
| Brane body re-step | `re_step_brane_bodies()` entire function | O(statements × depth) | **NO** — product of unbounded depth and statements | Decompose into per-stage micro-steps |
| Operator computation | `compute_operator()` (`ubc.rs:535-584`) | O(operands) | Yes — small constant | Same |

**ECONSTANIC (terminal)**

| Aspect | Detail |
|--------|--------|
| **UBC behavior** | No more stepping. `step_one()` returns `NoOp`. Reached when: (a) `SearchFir.step_unanchored()` finds nothing in scope (`fir.rs:763`), (b) `SearchFir.step_anchored()` anchor is missing (`fir.rs:802`), (c) `IndexFir.step_unanchored()` has no current brane (`fir.rs:892`). |
| **Synchronization** | None — this is a blocking state. The FIR is done. |
| **Constant-time** | **N/A** — terminal state. |
| **UBCb mapping** | Proto-brane enters wait-for queue. Waits for re-coordination via `StateChange` messages. When cloned into new context, starts at EMBRYONIC (per constanic cloning rules). |

**WOCONSTANIC (terminal, revivable)**

| Aspect | Detail |
|--------|--------|
| **UBC behavior** | No more stepping in this cycle. Reached when: (a) `SearchFir` finds a constanic target (`fir.rs:755-758`), (b) `OperatorFir` all operands are constanic but not constant (`fir.rs:677-678`), (c) `StayFoolishFir` inner expression is constanic (`fir.rs:952-956`), (d) `NormalBraneFir` any statement is ECONSTANIC/WOCONSTANIC (`ubc.rs:526-527`). |
| **How it revives** | Parent's next BRANING step retries via `re_step_brane_bodies()` which resets all searches to Embryonic and re-steps everything. Also: `run_to_completion_with_scope()` breaks on WOCONSTANIC but returns to its caller, which may retry from a higher level. |
| **Synchronization** | **THIS IS THE MAIN SYNCHRONIZATION POINT IN UBC.** WOCONSTANIC means "waiting on dependencies." The parent's next BRANING cycle provides the synchronization barrier. |
| **Constant-time** | **N/A** — terminal but waiting for external event. |
| **UBCb mapping** | `StateChange` message triggers WOCONSTANIC → CONSTANT transition. UBCb maintains a listener list per FIR. When a dependency transitions, it sends `StateChange` to all registered listeners. |

**CONSTANT / INDEPENDENT / NK (terminal)**

| Aspect | Detail |
|--------|--------|
| **UBC behavior** | No more stepping. `step_one()` returns `NoOp`. |
| **Synchronization** | None. |
| **Constant-time** | **N/A** — terminal state. |
| **UBCb mapping** | CONSTANT/INDEPENDENT: shared by reference (never cloned, per `constanic_clone()` line 465). NK: irrecoverable, shared by reference. |

#### Synchronization Points Summary (Background Research)

When does one FIR wait for another, and how does the wait end?

| Wait Condition | From State | To State | UBC Mechanism | UBCb Mechanism |
|---------------|-----------|----------|---------------|----------------|
| Dependency is constanic | Any → WOCONSTANIC | WOCONSTANIC → CONSTANT | Parent retries in next BRANING step (sync via `re_step_brane_bodies()`) | `StateChange` message from dependency |
| Search not found | Search → ECONSTANIC | ECONSTANIC → re-eval | Constanic clone + recoordination (triggered when brane is referenced in new context) | `StateChange` + clone on `FulfillSearch` response |
| Search found constanic target | Search → WOCONSTANIC | WOCONSTANIC → CONSTANT | Short-circuit deref chain on next step (`SearchFir::short_circuit_self()`, `fir.rs:810-830`) | `StateChange` → re-call `resolve()` |
| Operator operands not constant | Operator → WOCONSTANIC | WOCONSTANIC → CONSTANT | Each operand steps to completion, operator checks states | Each operand sends `StateChange` when constant |

**Synchronization Pattern:** In UBC, synchronization is implicit — the parent's `re_step_brane_bodies()` provides a natural barrier where all children are re-stepped. In UBCb, synchronization must be explicit — `StateChange` messages carry the barrier information.

#### Stage-wise Fairness Model (Background Research — Subject to Change per FOOP-31)

The original analysis proposed this scheduling discipline:

```
For each NYES stage in order:
  1. All proto-branes at PREMBRYONIC → transition to EMBRYONIC
  2. All proto-branes at EMBRYONIC → process searches (up to bandwidth/brane/step)
     - Repeat until ALL proto-branes exit EMBRYONIC
  3. All proto-branes at BRANING → step children (up to bandwidth/brane/step)
     - Repeat until ALL proto-branes exit BRANING
  4. All proto-branes at constanic → check for StateChange-triggered transitions
```

**Key insight:** UBC completes one brane's entire subtree before moving to siblings (depth-first across branes). UBCb processes all branes at the same stage simultaneously (breadth-first across branes). This is the core fairness property.

> **BDFL NOTE:** This model was reviewed and a change was decided in FOOP-31. The final scheduling discipline will be defined by FOOP-31 before any UBCb implementation begins. Retain this analysis as background — it documents the problem space.

#### Key Findings (Background Research)

1. **UBC is already breadth-first within a brane** (FIFO braneMind in the Java/Scala impl), but depth-first across branes (recurses into child branes fully before siblings proceed). UBCb extends breadth-first fairness across the entire proto-brane tree.

2. **Not all UBC steps are constant-time.** Four operations are unbounded in the current Rust implementation: recursive search-to-completion (`run_to_completion_with_scope()`), constanic clone (recursive `constanic_clone()`), scope chain walk (`Scope.search()`), and brane body re-stepping (`re_step_brane_bodies()`). UBCb bounds these via MESSAGE_BANDWIDTH per step.

3. **The Rust impl collapses EMBRYONIC and BRANING.** `re_step_brane_bodies()` does search resolution and child stepping in one depth-first loop. UBCb must separate these phases to achieve stage-wise fairness.

4. **WOCONSTANIC is the main synchronization point.** In UBC, the parent's retry cycle provides the barrier. In UBCb, `StateChange` messages provide the barrier. This is the mechanism that needs the most careful design in UBCb.

## Rejected Alternatives

### A. UBCb shares the evaluator with UBC (just adds message-passing as an option)

**Rejected**: the message-passing architecture is fundamentally different from
depth-first stepping. Sharing code would create a hybrid that is neither clean
depth-first nor clean message-passing. Separate implementations are clearer and
easier to maintain.

### B. UBCb replaces UBC entirely (no reference implementation)

**Rejected**: UBC's depth-first design is simpler and more understandable. It serves
as the canonical reference for language semantics. UBCb is an optimization, not a
replacement.

### C. UBCb starts with Phase 5 (breadth-first) features directly

**Rejected**: UBCb must first match UBC's depth-first capabilities before adding
breadth-first enhancements. Skipping the parity checkpoints would make it impossible
to verify correctness.

## Open Questions

- What message types does UBCb need beyond those described in `ubc2_message_protocol.md`?
  (To be answered by the UBCb Message Protocol FOOP.)
- How does UBCb handle the wake-up queue? What is the scheduling policy?
  (To be answered by the UBCb Constanic Coordination FOOP.)
- Should UBCb and UBC share the CLI binary, or have separate binaries?
  Implementation detail — likely shared binary with a flag to select the VM.

## References

- [ubc2_design.md](ubc1/how/ubc2_design.md) — UBC2 design specification
- [ubc2_message_protocol.md](ubc1/how/ubc2_message_protocol.md) — Message protocol specification
- [d0_0_protobrane.md](ubc1/how/d0_0_protobrane.md) — Proto-brane specification
- [d0_1_brane.md](ubc1/how/d0_1_brane.md) — Brane specification
- [d0_6_communication_medium.md](ubc1/how/d0_6_communication_medium.md) — Communication medium
- [phase5_ubc_breadth_first.md](todo/rust-mvp/phase5_ubc_breadth_first.md) — Phase 5 breadth-first design
- [01_phases_overview.md](todo/rust-mvp/01_phases_overview.md) — Phase roster
