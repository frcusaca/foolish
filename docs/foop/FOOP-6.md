---
foop: 6
title: Phase 2 evaluator is depth-first sequential; breadth-first deferred to Phase 4
author: hc <hc.busy@gmail.com>
status: Superseded
type: Standards
created: 2026-05-01
phase: phase-2
supersedes: []
---

# FOOP-6: Phase 2 evaluator is depth-first sequential; breadth-first deferred to Phase 4

> **Superseded 2026-07-14.** The depth-first, left-to-right sequential evaluation order is
> realized in UBCa's uniform two-phase stepping (FOOP-62). Early UBC-era text; terminology
> and mechanism details predate UBCa — cite FOOP-62 instead.

## Abstract

The Phase 2 UBC evaluator walks FIRs in **depth-first, left-to-right,
sequential** order. Each statement steps to a constanic terminal state
(`CONSTANT`, `INDEPENDENT`, `ECONSTANIC`, `WOCONSTANIC`, `NK`) before its
right siblings begin. There is no message passing, no wake-up queue, no
dependency tracking map.

Breadth-first evaluation — the order all previous Foolish designs (UBC1,
UBC2 design docs) targeted — is deferred to a dedicated Phase 4. Phase 4
exists specifically to introduce breadth-first stepping on the stable
Phase 2 foundation.

## Motivation

Phase 2's job is to make the 60+ approval tests pass with correct
semantics. The hardest part of UBC stepping — coordinating constanic
values, handling search short-circuiting, managing recoordination via
constanic cloning — is hard regardless of order. By making Phase 2
depth-first, we eliminate three sources of orthogonal complexity:

1. **No wake-up queue**: in breadth-first, when a search resolves to
   ECONSTANIC and a later step makes that ECONSTANIC become CONSTANT, we
   need to wake the dependent search. In depth-first, the search's
   dependencies are already at terminal states by the time the search is
   evaluated, so there's nothing to wake.

2. **No dependency map**: breadth-first needs a reverse-dependency
   structure ("which FIRs are waiting on which?"). Depth-first doesn't.

3. **No coordination of one Foolish-machine**: the breadth-first design
   needs an answer to "how does one machine schedule independent subtree
   progress without paralllelism?" — the H-uman coordination problem.
   Phase 2 punts.

These complexities are real and need solving — but separately, with the
core semantics already proven correct against the test suite.

## Specification

### Phase 2 step rule

A brane is evaluated by walking its statements left-to-right. For each
statement:

1. Step the statement's body to completion (recursively, depth-first).
2. The body reaches a constanic terminal state or NK.
3. Move on to the next statement.

"Step to completion" is implemented by repeatedly invoking `step()` on
the FIR until the state stops changing. The driver does not yield to
sibling subtrees mid-completion.

### Phase 4 will replace this

Phase 4 introduces a breadth-first driver that steps FIRs cooperatively.
The Phase 4 design has open questions noted in
[phase4_ubc_breadth_first.md](../scala-mvp/foolish-scala/docs/phase4_ubc_breadth_first.md).

Phase 4's exit criteria include "all Phase 2 approval tests pass with the
breadth-first driver." So depth-first is not merely a stepping stone — it
is the regression baseline.

## FIR Impact

None. Both depth-first and breadth-first work on the same FIR algebra.
The Nyes lifecycle is identical. Only the driver loop differs.

## UBC Step Impact

Establishes the depth-first contract for Phase 2:

- `Ubc.step(fir: Fir): Fir` advances a single FIR by one unit of work and
  returns. (Defined for completeness; in Phase 2, callers typically use
  `Ubc.runToCompletion`.)
- `Ubc.runToCompletion(fir: Fir): Fir` repeatedly calls `step` on `fir`
  and its descendants in depth-first left-to-right order until no FIR
  changes state.
- The driver MUST NOT introduce any wake-up queue, dependency map, or
  cross-FIR scheduling logic in Phase 2.

## Test Plan

The 60+ active `.foo` approval tests serve as the Phase 2 validation
suite. A test specifically asserts depth-first ordering by inspecting
intermediate states (e.g., "after stepping `{a = 1; b = 2; c = a + b}`
once, `c`'s body has not yet been visited because `a` is still being
stepped"). This guards against accidental breadth-first creep.

## Rejected Alternatives

### A. Skip Phase 2; do breadth-first directly

Tempting because breadth-first is the actual target. **Rejected**: too
many orthogonal challenges. The H-uman coordination problem alone is
substantial; combining it with the constanic semantics design is a
recipe for either a stuck implementation or one whose bugs are
ambiguous between layers. Phase 2 first lets us debug the semantics
against tests with predictable evaluation order.

### B. Keep depth-first forever; never write Phase 4

**Rejected**: depth-first cannot deliver the LOD browser's promise of
showing partial progress on multiple subtrees simultaneously (Phase 5).
Depth-first is also less stack-safe on deeply nested branes. The
breadth-first work is necessary; deferring it is not abandoning it.

### C. Keep both drivers permanently

Could be useful for debugging or for cases where depth-first is faster.
**Open question, not rejected outright**: Phase 4's exit criteria leave
this open. Decision deferred to Phase 4 implementation experience.

## Open Questions

- The five Phase 4 design questions about H-uman coordination of one
  Foolish-machine. Listed in `phase4_ubc_breadth_first.md` §"The Hard
  Part."
- Whether to keep both drivers permanently (see Rejected Alternatives C).

## References

- `scala-mvp/foolish-scala/docs/phase2_ubc.md`: detailed depth-first
  specification.
- `scala-mvp/foolish-scala/docs/phase4_ubc_breadth_first.md`: the future
  breadth-first phase.
- FOOP-7: constanic clone algorithm. Both depth-first and breadth-first
  call `constanicClone`; the algorithm is shared.
- d0_5 in the broader docs branch: the original UBC2 recoordination design.
