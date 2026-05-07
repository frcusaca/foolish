# Phase 5 — UBC: Breadth-First Evaluation

> Goal: Re-implement UBC stepping in **breadth-first order** so deeply-nested
> branes don't blow the call stack and so unrelated subtrees can progress
> independently.

> **This phase has open design questions** noted at the bottom.

---

## Why Breadth-First

1. **Stack safety**: depth-first recursion on a 1000-deep brane structure
   can overflow the call stack. Phase 2 mitigates this with explicit loops,
   but the underlying call structure is still depth-first.

2. **Independent subtrees**: breadth-first interleaves evaluations across
   sibling subtrees. For UI responsiveness in Phase 6, partial progress on
   multiple subtrees is preferable.

3. **Future parallelism**: breadth-first is a precondition for any future
   parallel evaluation.

---

## What Phase 5 Must Preserve

- **Identical observable output** for all Phase 2 approval tests.
- The Nyes lifecycle.
- The `constanic_clone` recoordination algorithm (FOOP=7).
- Search short-circuiting through WOCONSTANIC chains.
- Writing-order semantics.

---

## What Phase 5 Changes

- **The driver loop**: instead of `step_to_completion(child)` recursing eagerly
  into one child, the driver maintains a queue of FIRs ready to step.
- **The `step()` contract**: one unit of work per call. The driver decides
  what to step next based on dependencies.
- **Dependency tracking**: when a search resolves to WOCONSTANIC, it registers
  a dependency; when the underlying ECONSTANIC resolves, dependents wake up.

---

## The Hard Part — Coordinating One Foolish-Machine

> **TODO — design discussion needed with HC**:
>
> 1. What data structure represents the "queue of FIRs ready to step"?
>    Per-brane? Global? A priority queue?
> 2. How does the driver decide which FIR to step next? Writing order?
>    Dependency depth? Round-robin?
> 3. How is dependency tracking represented? Reverse-dependency map?
>    Per-FIR list of "things waiting on me"?
> 4. How does the driver detect "no progress possible" and terminate?
>    A pass with zero state transitions?
> 5. How does breadth-first interact with constanic_clone?

---

## Rust-Specific Considerations

For the breadth-first queue, Rust offers:
- `VecDeque<FirRef>` — simple FIFO queue
- `BinaryHeap` — priority queue (needs `Ord` implementation)
- `crossbeam::Queue` — if multi-threaded evaluation is desired later

For dependency tracking, a `HashMap<FirRef, Vec<FirRef>>` maps each
ECONSTANIC FIR to its dependents.

## Lessons from SF/SFF Implementation

The Phase 2 SF/SFF implementation reveals several patterns that affect Phase 5:

1. **SFF is naturally breadth-first friendly**: SFF blocks all expansion, so the
   queue can skip it entirely. No queuing needed for SFF content.

2. **SF needs its own stepping loop**: `step_except_brane_searches()` has its own
   loop. In breadth-first, this logic needs to be integrated into the main queue
   loop, likely as a per-FIR flag.

3. **Scope carries evaluation context**: `block_brane_searches` is a Scope field.
   Breadth-first will need per-thread context rather than a shared Scope.

4. **`resolve_to_value()` unifies wrapper stripping**: breadth-first can use
   `resolve_to_value()` for all operand resolution, eliminating the need for
   `strip_sf_wrapper()` as a separate function.

5. **RefCell borrow pattern**: the `Variant` enum pattern for releasing borrows
   before mutation will be needed for any read-then-mutate step function.
   Consider `Fir::into_variant()` as a general utility.

See [phase2_sf_sff_seek_insights.md](phase2_sf_sff_seek_insights.md) for details.

---

## Phase 5 Exit Criteria

- All Phase 2 approval tests pass with the breadth-first driver.
- A new test exercises partial progress on multiple subtrees.
- Documented answers to the 5 TODO design questions.

---

## Last Updated

**Date**: 2026-05-06
**Updated By**: Claude Code; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Added "Lessons from SF/SFF Implementation" section. Documents 5 key
patterns from Phase 2 that affect Phase 5 design: SFF queuing, SF stepping,
Scope context, resolve_to_value unification, and RefCell borrow pattern.

**Date**: 2026-05-05
**Updated By**: Claude Code; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Initial creation — Rust Phase 5 breadth-first plan.
