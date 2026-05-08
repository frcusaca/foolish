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

# FOOP-32: UBCb — Message-passing brane computer variant; SPA1 parity plan

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
- **FIR Lifecycle**: same Nyes states as UBC (PREMBRYONIC → EMBRYONIC → BRANING →
  constanic terminal states), but transitions are triggered by incoming messages, not
  by direct function calls.
- **Protobranes**: literal values and boundary-less expressions (per `d0_0_protobrane.md`).

### Key Differences from UBC

| Aspect | UBC (reference) | UBCb (message-passing) |
|--------|----------------|----------------------|
| Evaluation order | Depth-first, sequential | Breadth-first, message-driven |
| State mutation | In-place via `Rc<RefCell<Fir>>` | Message-driven state transitions |
| Constanic cloning | Synchronous, at search resolution | Asynchronous, triggered by recoordination messages |
| Search resolution | Immediate (depth-first guarantee) | Deferred (may require wake-up messages) |
| Concurrency | None (single-threaded) | Natural (independent subtrees) |
| Architecture | Function-call stepping | Message-passing protocol |

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
