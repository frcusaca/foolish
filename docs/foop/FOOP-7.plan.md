# FOOP-7: Constanic Clone — Implementation Plan

This FOOP, in its current revision (2026-05-09), defines two clone
operations: `constanic_clone` (constanic sources, no AB extension)
and `preconstanic_clone` (pre-constanic sources, AB extension with
compression). The implementation lives in `foolish-core-ubcb`
(UBCb's parallel crate) — see [FOOP-61.plan.md](FOOP-61.plan.md) for
the overall sequencing.

UBC's existing `constanic_clone` in `foolish-core` is the legacy
recursive-clone-with-state-reset algorithm; FOOP-7 (this revision)
applies to UBCb only.

## Dependency FOOPs

- [ ] FOOP-7 (this FOOP, design) → Brewing
- [ ] Depends on FOOP-51 (AB, search_result, compression). The
      Builder / BuilderFrom mechanism comes from FOOP-51.
- [ ] Coordinates with FOOP-61 (state machine — defines what
      "constanic" and "pre-constanic" mean and what state-reset
      means).

## Revision history of this plan

The original plan file described the legacy algorithm (recursive
descent with per-NYES-state dispatch, resetting ECONSTANIC →
EMBRYONIC and WOCONSTANIC → BRANING). That algorithm is **obsolete**
under the 2026-05-09 split-operation model. The current plan
implements the new model.

Legacy plan items (now obsolete and removed):
- Per-NYES dispatch in a single function.
- Reset ECONSTANIC → EMBRYONIC.
- Reset WOCONSTANIC → BRANING with recursively-cloned constanic
  children.
- Caller invariant requiring constanic terminal state before clone.

## Phase 1 — `constanic_clone` (no AB extension)

- [ ] Implement `constanic_clone(source: &FirRef) -> FirBuilder`:
  - [ ] Precondition check: `source.state.is_constanic()`. Panic on
        violation. (Per FOOP-7: pre-constanic sources are caller
        bugs; immediate panic, not a debug-only assert.)
  - [ ] CONSTANT/INDEPENDENT/NK: return `FirBuilder::wrapping(source)`.
        The caller's `.setParent(...).build()` chain is a no-op
        for these (or skips parent-setting).
  - [ ] WOCONSTANIC/NOTFOUNDIC: build a shallow copy with state
        reset to BRANING; AB unchanged.
  - [ ] Recurse into children via `recursively_clone_children`:
    - [ ] Children in CONSTANT/INDEPENDENT/NK: share by reference.
    - [ ] Children in WOCONSTANIC/NOTFOUNDIC: shallow copy with
          parent pointer rewritten to the cloned root, state
          reset to BRANING, recurse.
    - [ ] NYE children of constanic parent: panic (should not
          occur per FOOP-61's brane-state-computation invariants).
- [ ] Implement caller usage pattern: every search-result site
      uses `constanic_clone(&target).setParent(self_as_parent_ref).build()`.

## Phase 2 — `preconstanic_clone` (extends AB, preserves NYES)

- [ ] Implement `preconstanic_clone(source: &FirRef) -> FirBuilder`:
  - [ ] Precondition check: `source.state.is_preconstanic()`. Panic
        on violation. (`is_preconstanic` is the complement of
        `is_constanic`: PREMBRYONIC, EMBRYONIC, BRANING, WOBRANING.)
  - [ ] Compute `new_ab = source.ab.append((source.parent,
        source.line_in_parent)).compress()` (compression per
        FOOP-51).
  - [ ] Build shallow copy with `new_ab`; state preserved unchanged.
  - [ ] Recurse into pre-constanic children with parent pointer
        rewrites. (Children's AB handling: TBD per FOOP-7 open
        question — likely AB extends only on the cloned root.)
- [ ] Implement `Nyes::is_preconstanic()` — complement of
      `is_constanic()`.
- [ ] **Reserved feature.** `preconstanic_clone` is implemented and
      tested but NOT invoked by current FOOP set. Reserved for a
      future special case.

## Phase 3 — Builder / BuilderFrom integration

(Coordinated with FOOP-51 Phase 7.)

- [ ] `BuilderFrom::new(source)` wraps an existing FIR.
- [ ] `.with_ab(new_ab)`, `.with_state(state)`, `.with_parent(p)`,
      `.setParent(p)` (alias for `.with_parent(Some(p))`).
- [ ] `.build()` produces the final FirRef.
- [ ] Debug-mode check: `.build()` after `constanic_clone` requires
      `.setParent(...)` to have been called intermediate
      (catches the caller-pattern violation).

## Phase 4 — Tests

### Phase 4.1 — `constanic_clone` per-state behavior

- [ ] Test: panic on PREMBRYONIC source.
- [ ] Test: panic on EMBRYONIC source.
- [ ] Test: panic on BRANING source.
- [ ] Test: panic on WOBRANING source.
- [ ] Test: CONSTANT source returns wrapping builder; `.build()`
      yields the source by reference (or pointer-equal).
- [ ] Test: INDEPENDENT source returns wrapping builder.
- [ ] Test: NK source returns wrapping builder.
- [ ] Test: WOCONSTANIC source produces clone with state = BRANING,
      AB unchanged, parent set by caller's `.setParent(...)`.
- [ ] Test: NOTFOUNDIC source produces clone with state = BRANING,
      AB unchanged, parent set by caller's `.setParent(...)`.

### Phase 4.2 — Recursive descent

- [ ] Test: cloning a WOCONSTANIC NormalBrane recurses into
      children. WOCONSTANIC/NOTFOUNDIC children get rewritten
      parent pointers and state reset to BRANING.
      CONSTANT/INDEPENDENT/NK children shared by reference.
- [ ] Test: cloning a NOTFOUNDIC Operator recurses into operands
      (same rule).
- [ ] Test: cloning a CONSTANT NormalBrane: no recursion, source
      shared by reference.

### Phase 4.3 — NOTFOUNDIC rescue scenario

- [ ] Test: NOTFOUNDIC Search (search exhausted in current AB) is
      cloned via `constanic_clone(&s).setParent(new_host).build()`.
      The clone:
  - [ ] Has state = BRANING.
  - [ ] Has AB unchanged.
  - [ ] Has parent = new_host.
  - [ ] On next step, re-walks resolution and finds the name in
        the new parent's chain.
  - [ ] Reaches CONSTANT.
- [ ] Test: NOTFOUNDIC Search cloned into a host where the new
      parent's chain still doesn't define the name. Clone re-walks,
      re-exhausts, returns to NOTFOUNDIC.

### Phase 4.4 — `preconstanic_clone` per-state behavior

- [ ] Test: panic on WOCONSTANIC source.
- [ ] Test: panic on NOTFOUNDIC source.
- [ ] Test: panic on CONSTANT source.
- [ ] Test: panic on INDEPENDENT source.
- [ ] Test: panic on NK source.
- [ ] Test: PREMBRYONIC source produces clone with state =
      PREMBRYONIC, AB extended by `(source.parent,
      source.line_in_parent)`, then compressed.
- [ ] Test: EMBRYONIC, BRANING, WOBRANING — same pattern.

### Phase 4.5 — AB compression integration

- [ ] Test: `preconstanic_clone` of a FIR whose existing AB
      already contains `(p, n)` and we're appending `(p, m)` with
      `m ≤ n`: dedup drops the new entry (or the old, whichever
      is dominated).
- [ ] Test: short-circuit accumulation followed by compression (per
      FOOP-51 §"Short-circuit accumulation").

### Phase 4.6 — Approval test parity

- [ ] All `.foo` approval tests pass via the cross-validation
      harness from FOOP-61.plan.md Phase 2.

## Worktree

- [ ] Create worktree at `${HOME}/tmp/foolish-worktrees/6842-foop-7`
      with branch `foop/7-constanic-clone-split`
- [ ] Verify all work is complete in
      `${HOME}/tmp/foolish-worktrees/6842-foop-7` and committed to
      `foop/7-constanic-clone-split`
- [ ] Merge `foop/7-constanic-clone-split` to `jia`

## Notes

- **`preconstanic_clone` is implemented and tested for future use.**
  Per the current FOOP set, no operation invokes it. It is reserved
  for a future special case to be specified later. Tests confirm it
  works and panics correctly on bad inputs; integration with the
  main evaluation flow is deferred.
- **The legacy algorithm in `foolish-core` is unaffected.** This
  plan's implementation lives in `foolish-core-ubcb`. UBC's existing
  `constanic_clone` continues to work per its own (different)
  semantics.

## Last Updated

**Date**: 2026-05-09
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.7 xHigh effort
**Changes**: Rewrote plan to match FOOP-7's 2026-05-09 split into
`constanic_clone` (no AB extension) and `preconstanic_clone` (AB
extension with compression). Replaced the legacy per-NYES-dispatch
plan items. Added recursive descent tests and the
`.setParent(...).build()` caller pattern.

**Date**: 2026-05-01 (legacy)
**Updated By**: hc
**Changes**: Initial plan describing the legacy recursive-clone
algorithm. Superseded by the 2026-05-09 revision.
