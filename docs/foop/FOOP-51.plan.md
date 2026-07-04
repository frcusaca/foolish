# FOOP-51: AB list, search_result, short-circuit accumulation — Implementation Plan

This FOOP defines AB, the determinism invariant on `search_result`,
short-circuit accumulation, and AB compression. It is consumed by
FOOP-7 (constanic clone) and FOOP-61 (UBCb state machine).

The implementation lives in `foolish-core-ubcb` (UBCb's parallel
crate) — see [FOOP-61.plan.md](FOOP-61.plan.md) for the overall
sequencing. UBC's existing implementation in `foolish-core` does NOT
adopt FOOP-51; the AB model is UBCb-specific.

## Dependency FOOPs

- [x] Canceled. This feature should be later respecified and reimplemented.
      (2026-07-03 18:23)
- [-] FOOP-51 (this FOOP, design) → Brewing
- [-] Coordinates with FOOP-7 (clone) and FOOP-61 (state machine);
      all three move forward together.

## Phase 1 — `Ab` type and operations

- [-] Implement `Ab` (immutable list of `(Rc<NormalBraneFir>, usize)`):
  - [-] `Ab::empty()`.
  - [-] `Ab::append(self, entry: (Rc<NormalBraneFir>, usize)) -> Ab`.
  - [-] `Ab::iter_front_to_back() -> impl Iterator<...>`.
  - [-] `Ab::compress() -> Ab` per FOOP-51's line-aware dedup rule.
- [-] Per FOOP-51 implementation note: simple immutable Vec with
      O(|ab|) copy on append. Document inefficiency for future
      optimization.

## Phase 2 — Name resolution algorithm

- [-] Implement `resolve_search(starting_fir, pattern,
      originating_line)` per FOOP-51 spec:
  - [-] Phase 1: own statements before `cur_line`.
  - [-] Phase 2: AB front-to-back; each entry's statements before
        recorded line.
  - [-] Phase 3: ascend to live parent; repeat.
- [-] Implement `brane.search_local(pattern, from_line)` and
      `brane.search_ancestral(pattern, from_line)` primitives.

## Phase 3 — search_result field and determinism invariant

- [-] Rename `target` → `search_result` on `SearchFir`. (UBCb
      starts with the new name; UBC keeps `target`.)
- [-] Document the determinism invariant in code: search_result is
      set at most once per search; never re-resolved by the
      algorithm.
- [-] Implement mandatory unit tests for the invariant — see
      [FOOP-61.plan.md](FOOP-61.plan.md) Phase 6.

## Phase 4 — Short-circuit accumulation

- [-] Implement chain collapse for `S₁.search_result → S₂ → ... →
      T`:
  - [-] Walk the chain; collect AB extensions per FOOP-51 §
        "Short-circuit accumulation".
  - [-] Build `final_ab = T.ab.append_all(accumulated)` and run
        `compress()`.
  - [-] Install `S₁.search_result = clone_with_ab(T, final_ab)`.
- [-] Test: chain of three searches collapses to a single
      reference; AB on collapsed target reflects the union of hops.

## Phase 5 — AB compression

- [-] Implement `Ab::compress()` per the line-aware dedup rule:
  - [-] Walk left to right.
  - [-] Drop entry `(b, n)` if an earlier entry `(b, m)` exists
        with `m ≥ n`.
  - [-] HashMap-keyed by brane LUID/identity; track max-line-seen
        per LUID.
- [-] Apply compression after every constanic_clone (handled by
      FOOP-7 — but no-op there since constanic_clone doesn't extend
      AB).
- [-] Apply compression after every preconstanic_clone (FOOP-7
      reserved feature).
- [-] Apply compression after every short-circuit accumulation.
- [-] Optional: apply compression on serialization for smaller
      output.
- [-] Test: pre/post compression results in identical search
      behavior; uncompressed and compressed AB give the same name
      resolution outcomes.

## Phase 6 — CONSTANT → INDEPENDENT detach step

- [-] Implement `step_finalize`: clear AB, clear parent, advance
      state to INDEPENDENT.
- [-] Adjust the driver loop to allow this extra step (terminate on
      `is_fully_terminal()` = INDEPENDENT or NK, NOT on CONSTANT).

## Phase 7 — Builder / BuilderFrom

- [-] Add per-variant `Builder` types (FirBuilder for the umbrella).
- [-] Add `BuilderFrom::new(source)` that wraps an existing FIR
      and exposes `.with_ab(...)`, `.with_state(...)`,
      `.with_parent(...)`, `.build()`.
- [-] Test: every existing FIR construction site uses Builder /
      BuilderFrom.

## Worktree

- [-] Create worktree at `${HOME}/tmp/foolish-worktrees/3604-foop-51`
      with branch `foop/51-ab-search-result`
- [-] Verify all work is complete in
      `${HOME}/tmp/foolish-worktrees/3604-foop-51` and committed to
      `foop/51-ab-search-result`
- [-] Merge `foop/51-ab-search-result` to alpha

## Notes

This FOOP's implementation is interleaved with FOOP-61 and FOOP-7.
Treat the three plans as a single coordinated effort; each phase of
this plan corresponds to specific phases in FOOP-61.plan.md.

## Last Updated

**Date**: 2026-05-09
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.7 xHigh effort
**Changes**: Initial plan.
