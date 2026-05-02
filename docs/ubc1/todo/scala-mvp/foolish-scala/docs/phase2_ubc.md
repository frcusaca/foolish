# Phase 2 — UBC: Step Evaluation

> Goal: Read FIRs (in memory or via Circe deserialization), step-evaluate the FIR
> tree until every node is `Constant` or `Constanic`. **The hard part is
> coordinating constanic values** — that is the central UBC challenge that
> previous implementations got stuck on.

---

## Phase 2 Deliverable

A `Ubc.step(fir: Fir): Fir` function (or similar interface) that performs one
evaluation step on a FIR tree, returning the (possibly partly) advanced FIR.
Plus a `Ubc.runToCompletion(fir: Fir): Fir` that loops `step` until no FIR can
make progress.

The 60 `.foo` approval tests in `src/test/resources/.../inputs/` move into Phase 2
as the validation suite. Expected output format mirrors the Java UBC1 format
(`Sequencer4Human` style — see `02_implementor_reference.md`).

---

## State Transitions to Implement

| FIR variant | Initialized → ? | When |
|-------------|----------------|------|
| `ConstantIntFir` | always Constant | already (compiled directly) |
| `BinaryOpFir` | → `ConstantIntFir` | both children Constant — collapse, do arithmetic |
| `BinaryOpFir` | → Constanic | either child Constanic |
| `BinaryOpFir` | → NK | div-by-zero on `/` or `%`, or either child NK |
| `UnaryOpFir` | → `ConstantIntFir` | child Constant |
| `SearchFir(unanchored)` | → found node's value (could be Constant or Constanic) or → Constanic if not found anywhere | walk IB then AB chain |
| `SearchFir(anchored, anchor)` | → found value or → NK if not found and anchor is Constant | local to anchor's brane |
| `IndexFir` | similar to SearchFir | by index |
| `HeadTailFir` | → first/last statement of anchor brane | anchor must be Constant |
| `NormalBraneFir(stmts)` | → still NormalBraneFir, but each statement steps independently; brane state = worst of statements | breadth-first |
| `NKFir` | already NK | terminal |

---

## The Hard Part — Constanic Coordination

When a SearchFir resolves to a Constanic FIR (e.g., `a = b; b = unknown`), the
referencing FIR also becomes Constanic. When new context arrives (concatenation
in Phase 5, REPL line input via Phase 3), previously Constanic FIRs may resolve.

The Phase 2 evaluator needs to:
1. **Detect when re-evaluation is possible** — keep enough state to know "this FIR
   was Constanic because identifier X was missing" so a later context that introduces
   X can trigger a re-step.
2. **Avoid infinite loops** — if a FIR is Constanic on identifier X, and X is also
   Constanic on Y, and Y is Constanic on X, we have a cycle. Detect and stop.
3. **Maintain breadth-first order** — evaluate the current brane level fully before
   descending. (Avoids unbounded recursion in deeply-nested branes.)

**Approach (proposed, refine as Phase 2 begins):** each non-Constant FIR records a
*dependency set* — the names/anchors it was waiting on. The step loop tracks which
names became Constant in the previous round; if any FIR's dependency set intersects,
re-step that FIR. Loop until a fixed point.

Cycle detection: if a step round produces no state changes but FIRs remain
Initialized, every remaining Initialized FIR transitions to Constanic with its
current dependency set as the reason.

---

## Approval Tests Move Here

The `src/test/resources/.../inputs/` directory has 60 active `.foo` test inputs and
5 `.tbd` files. All become Phase 2 approval tests:

- Test harness: re-introduce `ApprovalTestRunner` (the Java helper from
  `foolish-core-java`) plus a Scala interpreter implementing `UbcTester` that
  pipes source through `Compiler.compileToJson` then `Ubc.runToCompletion`.
- Output format: Sequencer4Human (see `02_implementor_reference.md` for the
  expected format and the existing Scala UBC1's `Sequencer4Human.scala` as a
  starting point).

---

## Phase 2 Exit Criteria

- All 60 active `.foo` approval tests pass.
- All 5 `.tbd` tests have approved baselines (manually reviewed, never bulk-approved).
- Cycle detection is exercised by a dedicated test.
- `Ubc.step` is idempotent on Constant FIRs (calling `step` on a Constant returns
  the same FIR).

---

## Last Updated

**Date**: 2026-05-01
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.7 (1M Context)
**Changes**: Initial Phase 2 outline — UBC step evaluator with constanic coordination.
