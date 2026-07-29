---
foop: 48
title: FOOP-84 Implementation Plan — Search Engine Refactor (AncestralNavigator, multiplexed marker stream, CopyMode)
status: Draft
created: 2026-07-29
---

# FOOP-84 Implementation Plan

**Read `FOOP-84.md` first — this plan assumes the specification's context.** In particular Part 0
(terminology), §2.2.1–§2.2.6 (the traversal design), and §2.3–§2.5 (boundary evaluation and
`CopyMode`).

## Worktree

All values expanded (per `foop.md`):

```
WORKTREE_ORIGIN_BRANCH=jia
WORKTREE_ORIGIN_PATH=/yolo/src
WORKTREE_BRANCH_NAME=foop-84-search-engine-refactor
WORKTREE_FULL_FS_PATH=/home/agent/tmp/foolish-worktrees/foop-84-search-engine-refactor
```

## Prerequisites — HARD BLOCKERS

**FOOP-43 must land before this FOOP.** FOOP-84 §1.5 depends on its Component 1 settlement rule
(anchored miss → NK, unanchored → ECONSTANIC, SFF-marked → ECONSTANIC), and §2.4.1 depends on its
Component 3 `EconstanicReason::Detached` tag — without which a fully-detached *anchored* search
would settle NK and be destroyed rather than deferred.

- [ ] Confirm FOOP-43 is merged and its `EconstanicReason` enum exists in the tree
- [ ] Confirm `cargo test --workspace` is green on `jia` BEFORE starting (per AGENTS.md: never
      start Phase-or-larger work when any test is broken)

## Scope

Two halves with **different risk profiles**, landed as **separate commits** (FOOP-84 header note):

| Half | Sections | Snapshot expectation |
|------|----------|----------------------|
| **A — Navigator unification** | Part 1 restatement, §2.2, §2.2.6 | **Strictly behavior-preserving.** No `.snap` may change. Any diff is a regression to investigate. |
| **B — Boundary evaluation** | §2.2.1–§2.2.5, §2.3–§2.5 | **Deliberate semantic change.** SF/SFF snapshot churn expected and enumerated below. |

**Files to modify:**
- `foolish-ubca/src/fir_kinds.rs` — `mod contextful_search` (`CandidateNavigator`,
  `BraneNavigator`, `contextful_search_scan`, `contextful_search_scan_no_body_check`), new
  `AncestralNavigator`, `CopyMode`/`BoundaryEffect`, `SearchFir::handle_found`/`clone_stmt_result`,
  removal of `ab_search_with_engine` and `BraneFir::_ab_search`, ConcatBrane accessors
- `foolish-ubca/src/fir_trait.rs` — `Scope.has_ancestral_sfm` (removal or documented retention),
  `_ab_search` default trait method
- `foolish-ubca/snapshot_tests/input/` — `foop_84_comprehensive.foo`
- `AGENTS.md` — Searches section, once semantics settle

**Out of scope (deliberately NOT in this FOOP):** `[patterns]` parsing, the `Detachment` struct,
`decide_to_detach` (all FOOP-24); `!`/`&&`/`||` (FOOP-93); `|` cascade (FOOP-04); find-all
(FOOP-14); batching (§2.2.3 — proposal only, follow-on FOOP).

---

## Plan

- [ ] Begin work: commit `FOOP-84.md` and `FOOP-84.plan.md` to origin, check `begun: [x]` in
      `FOOP-84.md` frontmatter
- [ ] Create worktree at /home/agent/tmp/foolish-worktrees/foop-84-search-engine-refactor with
      branch `foop/foop-84-search-engine-refactor`

### Stage 0 — Read and confirm the design

- [ ] Read §2.2.1 of `FOOP-84.md` (why the walk must be link-by-link; markers invisible to
      `_get_my_brane`/`_get_my_statement`)
- [ ] Read §2.2.2 (the multiplexed stream and the self-ordering guarantee)
- [ ] Read §2.2.4 (marker scope — the three conditions) and §2.2.6 (ConcatBrane stipulation)
- [ ] Read §2.3–§2.5 (`BoundaryEffect` vs `CopyMode`, the resolution algorithm, clone call sites)
- [ ] Resolve the remaining Open Question: the exact Rust shape of the multiplexed stream item —
      an enum over candidate/marker, two accessors, or a richer candidate struct. **Decide before
      writing code**; it determines the blast radius.
  - [ ] sub-agent: consult primary agent or human on the chosen stream-item shape before
        proceeding past this point

### Stage 1 — ConcatBrane: standardize the brane interface (§2.2.6)

Prerequisite for Half A, because the unified walk must enter ConcatBrane with no special-casing.

- [ ] Unit test FIRST: a `BraneNavigator` over a `ConcatenationFir` yields the merged statement
      sequence in global order, identical to a `BraneFir` holding the same statements
- [ ] Unit test FIRST: **the population hazard** — enumerate a ConcatBrane whose helpers are NOT
      yet populated and assert it does not report empty. Today `stmt_count()` populates on demand
      (`fir_kinds.rs:2461-2467`) while `stmt_at`/`_search_brane` return `None` when unpopulated
      (`:2479`, `:2503`), so the trio is coherent only if `stmt_count()` is called first —
      an accident of `BraneNavigator::new`'s call order, not a contract.
- [ ] Fix the hazard: make `stmt_at` populate on demand (preferred), or otherwise guarantee
      population before any enumeration begins
- [ ] Audit every brane-like FIR for the complete trio (`is_brane_like` / `stmt_count` /
      `stmt_at`): `BraneFir`, `ConcatenationFir`, `ConcatHelper`
- [ ] Verify `concat_ab_search_reaches_outward` (`fir_kinds.rs:6265`) still passes
- [ ] `cargo test --workspace` green; `cargo clippy -D warnings` clean

### Stage 2 — Half A: `AncestralNavigator` (behavior-preserving)

- [ ] Unit test FIRST: capture the **current** candidate order and completeness from
      `ab_search_with_engine` (`fir_kinds.rs:1085-1119`) and `BraneFir::_ab_search` (`:826-841`)
      across nested-brane fixtures — the before/after equivalence baseline
- [ ] Note and pin the difference between the two old paths: `_ab_search` calls `_ib_search` on
      each ancestor *statement*; `ab_search_with_engine` scans the parent brane's `[0, idx-1]`
      range directly. Confirm they agree on all fixtures, or document where they do not.
- [ ] Implement `AncestralNavigator` as a `CandidateNavigator`, walking the raw `.parent` chain
      **link by link** (§2.2.1) and deriving brane/statement boundaries from that walk — NOT
      delegating to `_get_my_brane`/`_get_my_statement`
- [ ] Unit test: the link-by-link walk lands on every FIR the skipping accessors would reach, in
      the same order, plus the FIRs they skip (the refinement property that makes Half A
      behavior-preserving). Include a `ConcatBrane` fixture.
- [ ] Replace `ab_search_with_engine` with `AncestralNavigator` + existing
      `contextful_search_scan`/`contextful_search_scan_no_body_check`
- [ ] Remove `BraneFir::_ab_search` — or reduce to a thin shim if a caller outside the engine
      remains. **Verify all call sites first** (it is a default trait method, `fir_trait.rs:258`,
      with a `ConcatBrane` path).
- [ ] Consider removing `_search_brane` overrides (`fir_kinds.rs:844`, `:2262`, `:2502`) now that
      search routes through `stmt_count`/`stmt_at` + the shared engine. Verify all call sites;
      removing them is what actually delivers "ConcatBrane is just a brane" (§2.2.6).
- [ ] `cargo test --workspace` green; `cargo clippy -D warnings` clean
- [ ] **Snapshot gate: `cargo insta test -p foolish-ubca --lib` produces ZERO `.snap.new`.** Half A
      is behavior-preserving; any diff here is a regression to investigate, NOT a formatting update
      to accept.
- [ ] COMMIT Half A separately, with a message stating the snapshot-clean result

### Stage 3 — Half B: multiplexed stream and boundary evaluation

- [ ] Define `BoundaryEffect` (`Pass`/`SfCopy`/`Detach`, Navigator-internal) and `CopyMode`
      (`Normal`/`SfCopy`, scan-visible) per §2.3
- [ ] Extend `AncestralNavigator` to yield the **multiplexed stream**: marker items interleaved
      with candidate items, in traversal order (§2.2.2)
- [ ] Unit test: **the self-ordering guarantee** — the Navigator yields a marker item BEFORE any
      candidate that marker governs. This is the property the whole design rests on; test it
      directly on nested-marker fixtures.
- [ ] Unit test: a walk with no markers yields candidate items only; `BraneNavigator` never yields
      a marker item
- [ ] Implement filter-list accumulation in the contextful-search layer: append each marker item in
      traversal order (innermost-first by construction); per candidate, walk the list front-to-back
      (first-in-first-check) resolving `BoundaryEffect`
- [ ] Unit test: inner marker declines → candidate still tested against outer markers; `Detach`
      skips the candidate without the predicate ever seeing it
- [ ] Unit test: scope rule (§2.2.4) — a search under a marker that resolves **within its own
      brane** is unaffected; a contexted (`&`) search is never affected; a search originating
      **outside** a marker is never affected by it
- [ ] Thread `CopyMode` to `SearchFir::handle_found` (`fir_kinds.rs:935-940`) and pass it as
      `descendent_of_sfm_and_foolishly_ignorant` to `clone_stmt_result` → `constanic_clone_at`,
      replacing `scope.has_ancestral_sfm` (§2.5)
- [ ] Unit test: **enumerate the divergences from `has_ancestral_sfm`** (§2.5) — the boolean is
      indexed on the *searcher*, `CopyMode` on the *boundary crossing*:
      (a) search under `<E>` finding a candidate **without crossing** the boundary → `Normal` (was
      foolishly-ignorant-copied); (b) finding a candidate **by crossing** → `SfCopy` (agrees with
      today); (c) SFF cases, which `has_ancestral_sfm` never covered (it is set only for
      `StayFoolish`, `fir_trait.rs:387-388`). State old and new outcome side by side in each test.
- [ ] SFF settlement: a fully-Detached search settles ECONSTANIC with `EconstanicReason::Detached`,
      NOT via bare `Miss` (which would settle NK for an anchored search) — §2.4.1
- [ ] Decide `Scope.has_ancestral_sfm`: remove it, or document the remaining consumer (Open
      Question). `Scope.active_detachments` is NOT added (§2.2.5).
- [ ] Pin the `contexted && !anchored` fallback (§1.2): `?name&#1` parses and evaluates identically
      to `?name`; the `&#1` is inert
- [ ] `cargo test --workspace` green; `cargo clippy -D warnings` clean

### Stage 4 — Snapshot review (Half B churn is EXPECTED)

- [ ] `cargo clean -p foolish-ubca && cargo insta test -p foolish-ubca --lib`
- [ ] For EACH `.snap.new`, justify the diff against §2.2.4's scope rule and §2.5's divergence
      analysis BEFORE presenting it. An unexplained diff is a bug, not churn.
- [ ] Expected-churn candidates (§2.5, §2.4): `sff_basic`, `sff_nested`,
      `sff_vs_sf_timing_difference`, `sff_resolves_on_each_use`, `sff_in_binary_op`,
      `sff_in_assignment_chain`, `sf_of_sff`, `sf_sff_nested_combined`,
      `complex_sff_with_nested_scope`, `complex_sff_in_nested_brane`
- [ ] **Note the FOOP-13 interaction:** commit `770fa394` deliberately made the SFF wrapper settle
      WOCONSTANIC, not ECONSTANIC. Confirm this FOOP does not silently revert that; if it does,
      raise it with the human as a distinct decision.
- [ ] NEVER run `cargo insta accept` or `INSTA_UPDATE=always` (AGENTS.md)

### Stage 5 — Comprehensive test and documentation

- [ ] Write and verify `foolish-ubca/snapshot_tests/input/foop_84_comprehensive.foo` — exercise
      nested naked SF/SFF combinations, contexted search chained after both anchored and (per §1.2)
      unanchored searches, AB walks crossing multiple brane levels with and without SF/SFF
      ancestors, and a ConcatBrane in the ancestral path
- [ ] Update `AGENTS.md` §Searches: add the SFF-marked→ECONSTANIC case beside the existing
      "anchored miss → NK" (which is **correct and must be left alone**, per FOOP-43 as revised)
- [ ] Verify FOOP-24/93/04/14 banners still describe this FOOP accurately after implementation

### Stage 6 — Merge and cleanup

- [ ] Verify all work is complete in
      /home/agent/tmp/foolish-worktrees/foop-84-search-engine-refactor and committed to
      `foop/foop-84-search-engine-refactor`
- [ ] Merge `foop/foop-84-search-engine-refactor` to `jia`
  - [ ] Confirm `foop_84_comprehensive.foo` exists, passes, and is human-approved
  - [ ] Repair ALL tests in `jia` at /yolo/src
  - [ ] STOP! STOP!! STOP!!! ASK HUMAN to check this box before continuing. UNDER NO
        CIRCUMSTANCES will Agent continue past this point automatically!!
    - [ ] Present human with
          `cd /home/agent/tmp/foolish-worktrees/foop-84-search-engine-refactor` and ask them to
          review snapshots BEFORE checking the parent checkbox
- [ ] Cleanup /home/agent/tmp/foolish-worktrees/foop-84-search-engine-refactor
  - [ ] Check that `.plan.md` has all but Cleanup checkboxes completed
  - [ ] Remove /home/agent/tmp/foolish-worktrees/foop-84-search-engine-refactor
  - [ ] This is the last sub-task checkbox to be checked in this block

---

## Notes for the executing agent

- **The two halves must stay separable.** If Half A cannot be made snapshot-clean, STOP and raise
  it — that means the Navigator unification is not behavior-preserving after all, which is a
  specification-level finding, not an implementation detail to work around.
- **`_search_brane` removal is optional but valuable.** It is a second, redundant search path
  (name matching + range handling already done by `SearchPredicate` + the scan loop). Leaving it in
  place risks drift; removing it delivers §2.2.6's intent. If call sites make removal risky, say so
  and leave it, documenting why.
- **Batching (§2.2.3) is out of scope.** Do not implement it. Do keep the `AncestralNavigator`
  interface from foreclosing it.

## Last Updated

**Date**: 2026-07-29
**Updated By**: Claude Code (Opus 5)
**Changes**: Initial plan. Structured around FOOP-84's two-half split (Navigator unification,
snapshot-clean; boundary evaluation, expected SF/SFF churn) with a separate commit per half.
Stage 1 front-loads the ConcatBrane interface standardization (§2.2.6) including a regression test
for the helper-population hazard, since the unified walk must enter ConcatBrane with no
special-casing. FOOP-43 recorded as a hard blocker (both its Component 1 settlement rule and
Component 3's `EconstanicReason::Detached`). Stage 0 ends on the one remaining Open Question — the
Rust shape of the multiplexed stream item — with a consult-before-proceeding sub-task.
