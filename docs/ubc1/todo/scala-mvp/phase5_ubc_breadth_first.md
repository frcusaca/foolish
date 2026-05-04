# Phase 5 — UBC: Breadth-First Evaluation

> Goal: Re-implement UBC stepping in **breadth-first order** so deeply-nested
> branes don't blow the JVM stack and so unrelated subtrees can progress
> independently. All previous Foolish designs (UBC1, UBC2 design docs)
> assumed breadth-first as the target evaluation order; Phase 2 used
> depth-first as a deliberate simplification (FOOP-6).

> **This phase has open design questions** noted at the bottom. The user (HC
> = the H-uman) has design context that needs to be elicited before
> implementation can begin in earnest.

---

## Why Breadth-First

1. **Stack safety**: depth-first recursion on a 1000-deep brane structure
   blows the JVM stack on the first naive implementation. Phase 2 mitigates
   this with explicit `stepToCompletion` loops, but the underlying call
   structure is still depth-first.

2. **Independent subtrees**: in `{a = {long_subtree_1...}, b = {long_subtree_2...},
   c = a + b}`, depth-first evaluates `a`'s entire subtree before touching
   `b`'s. Breadth-first interleaves them. For UI responsiveness in Phase 6
   (web brane browser), partial progress on multiple subtrees is preferable
   to "all-or-nothing on the leftmost subtree."

3. **Future parallelism**: breadth-first is a precondition for any future
   parallel evaluation. (Not in scope for this phase, but the breadth-first
   step engine should not foreclose it.)

4. **Alignment with UBC2 design intent**: UBC2's message-passing model
   (`FulfillSearch`/`RespondToSearch`) implicitly assumed breadth-first
   stepping. We didn't adopt the message-passing model in Phase 2 (FOOP-6)
   but bringing it back as part of breadth-first remains an option.

---

## What Phase 5 Must Preserve

- **Identical observable output** for all Phase 2 approval tests. The 60+
  `.foo` files must produce the same final states.
- The Nyes lifecycle (PREMBRYONIC → EMBRYONIC → BRANING → ECONSTANIC | WOCONSTANIC | CONSTANT | INDEPENDENT, plus NK).
- The `constanicClone` recoordination algorithm (FOOP-7).
- Search short-circuiting through WOCONSTANIC chains.
- Writing-order semantics (a name is not visible to its own RHS; backward
  search only finds statements written earlier).

---

## What Phase 5 Changes

- **The driver loop**: instead of `stepToCompletion(child)` recursing eagerly
  into one child until done, the driver maintains a queue of FIRs ready to
  step and processes them round-robin.
- **The `step()` contract**: instead of returning the FIR fully advanced as
  far as possible, `step()` performs one unit of work and returns. The
  driver decides what to step next based on dependencies.
- **Dependency tracking**: when a search resolves to a WOCONSTANIC, the
  search registers a dependency on the underlying ECONSTANIC; when that
  ECONSTANIC's clone is recoordinated and resolves, the dependents wake up.
  (In Phase 2, this was unnecessary because depth-first guaranteed
  dependencies were resolved before dependents.)

---

## The Hard Part — Coordinating One Foolish-Machine

This is the open design question. It is the central reason Phase 5 is its
own phase rather than a refactor of Phase 2.

> **TODO — design discussion needed with HC**:
>
> The previous Foolish designs proposed a "BraneMind" or "MUM" or similar
> single-machine coordination mechanism. The H-uman (HC) has the design
> context for this and will be consulted before this section is written.
>
> Specifically, we need to answer:
>
> 1. What data structure represents the "queue of FIRs ready to step"?
>    Per-brane? Global? A priority queue?
> 2. How does the driver decide which FIR to step next when multiple are
>    ready? Writing order? Dependency depth? Round-robin?
> 3. How is dependency tracking represented? Reverse-dependency map keyed
>    by ECONSTANIC FIR? Per-FIR list of "things waiting on me"?
> 4. How does the driver detect "no progress possible" and terminate?
>    A pass with zero state transitions?
> 5. How does breadth-first interact with constanicClone? In Phase 2, the
>    cloning happens synchronously in the same `step()` that resolves the
>    search. Does breadth-first allow the search to register its target
>    and let the clone be a separate scheduled step?

---

## Implementation Plan (placeholder — fill in after design discussion)

1. **Step 1**: Design discussion with HC; produce a written breadth-first
   evaluation specification that supersedes this TODO section.
2. **Step 2**: Implement the new driver alongside the Phase 2 driver,
   selectable by configuration. Keep both passing the approval tests.
3. **Step 3**: Add tests specific to breadth-first observable behavior
   (e.g., partial progress on a brane with mixed-depth subtrees).
4. **Step 4**: Decide whether to retire the Phase 2 depth-first driver or
   keep both. Likely retire, since Phase 6 (web browser) consumes the
   breadth-first driver's output structure.

---

## Phase 5 Exit Criteria

- All Phase 2 approval tests pass with the breadth-first driver.
- A new test exercises "partial progress on multiple subtrees" — verifying
  that breadth-first does not stall on one deep subtree.
- A documented answer to each of the 5 TODO design questions above.

---

## Last Updated

**Date**: 2026-05-01
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.7 (1M Context)
**Changes**: Renumbered Phase 4 → Phase 5 (after promoting Concatenation to
Phase 3 and shifting CLI to Phase 4). Phase 6 (web browser) is the immediate
consumer of breadth-first output. Still placeholder with TODO for design
discussion with HC about single-Foolish-machine coordination.
