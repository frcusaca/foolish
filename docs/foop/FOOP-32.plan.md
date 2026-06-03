---
foop: 23
title: FOOP-32 Implementation Plan — Repair rudimentary FVM and Sequencer bugs
status: Draft
created: 2026-06-01
---

# FOOP-32 Implementation Plan

## Worktree

```
WORKTREE_BRANCH_NAME=bugfix-rudimentary-foop-32
WORKTREE_FULL_FS_PATH=${HOME}/tmp/foolish-worktrees/bugfix-rudimentary-foop-32
```

## Scope

Fix 6 bugs: 3 FVM evaluation (scope resolution, precedence, boundary clamping) and
3 Sequencer formatting (braces, search_result indentation).

**Files to modify:**
- `foolish/foolish-core/src/sequencer.rs` — Bugs E, F (formatting)
- `foolish/foolish-core/src/ubc.rs` — Bugs A, C (scope resolution)
- `foolish/foolish-core/src/search.rs` — Bugs A, B, C (search/precedence)
- `foolish/foolish-core/src/fir.rs` — Bug D (Index bounds checking)
- `foolish/foolish-core/src/sequencer_tests.rs` — new unit tests for E, F
- `foolish/foolish-core/src/unit_tests.rs` — new unit tests for D

---

## Phase A: Sequencer formatting bugs (E, F) — independent, lowest risk

Bugs E and F are pure formatting changes in `sequencer.rs`. No evaluation logic affected.

- [x] Create worktree at ${WORKTREE_FULL_FS_PATH} with branch `foop/bugfix-rudimentary-foop-32`
      (2026-06-01 15:00)

- [x] **Bug E:** Add `{` and `}` delimiters to anonymous brane HSSnap output
      (2026-06-01 15:10)
  - In `sequencer.rs`, locate `format_fir` for `NormalBraneFir`
  - After emitting `Brane`, append `{` on the same line
  - After emitting all children, emit `}` at the parent indent level
  - For named branes (`name = Brane{...}`), apply the same rule
  - Add unit test in `sequencer_tests.rs`: `test_format_brane_with_braces`
  - Verify: `cargo test -p foolish-core -- sequencer_tests` passes

- [x] **Bug F:** Fix search_result formatting — indent as child, not sibling
      (2026-06-01 15:10)

- [x] Regenerate affected `.snap.new` files:
      (2026-06-01 15:30)
  - Run `cargo insta test -p foolish-core --lib`
  - Check that `anchored_seek_*.foo.snap.new` show `{}` delimiters
  - Check that `chained_undeclared.foo.snap.new` has search_result indented as child
  - **STOP — Human review gate:** Present updated `.snap.new` files for Bugs E, F

- [x] On human approval, promote approved `.snap.new` → `.snap` (strip LGTM lines)

## Phase B: Negative seek bounds clamping (Bug D) — isolated fix

Bug D is a focused bounds-checking fix in the Index FIR evaluation.

- [x] **Bug D:** Fix negative seek out-of-bounds to return NK instead of clamping
      (2026-06-01 15:15)
  - In `search.rs`, `index_in_brane()` — replaced `.max(0)` with early `return None` when `len + offset < 0`
  - Add unit test in `unit_tests.rs`: `test_negative_seek_oob_returns_nk`
  - Verify: `cargo test -p foolish-core -- unit_tests` passes

- [x] Regenerate affected `.snap.new` file:
      (2026-06-01 15:30)
  - Run `cargo insta test -p foolish-core --lib`
  - Check that `anchored_seek_negative_boundary.foo.snap.new` shows NK for `oob`
  - **STOP — Human review gate:** Present updated `.snap.new` for Bug D

- [x] On human approval, promote approved `.snap.new` → `.snap` (strip LGTM lines)

## Phase B: Negative seek bounds clamping (Bug D) — isolated fix

Bug D is a focused bounds-checking fix in the Index FIR evaluation.

- [ ] **Bug D:** Fix negative seek out-of-bounds to return NK instead of clamping
      (2026-06-01 XX:XX)
  - In `fir.rs` or `ubc.rs`, locate the Index FIR evaluation for negative offsets
  - Compare with positive offset handling (which correctly produces NK)
  - Apply symmetric bounds check: if `abs(offset) > brane.len()`, return NK
  - Add unit test in `unit_tests.rs`: `test_negative_seek_oob_returns_nk`
  - Verify: `cargo test -p foolish-core -- unit_tests` passes

- [ ] Regenerate affected `.snap.new` file:
      (2026-06-01 XX:XX)
  - Run `cargo insta test -p foolish-core --lib`
  - Check that `anchored_seek_negative_boundary.foo.snap.new` shows NK for `oob`
  - **STOP — Human review gate:** Present updated `.snap.new` for Bug D

- [ ] On human approval, promote approved `.snap.new` → `.snap` (strip LGTM lines)

## Phase C: Scope resolution bugs (A, C) — interrelated, requires investigation

Bugs A and C are opposite failure modes of brane boundary crossing in search.
Investigate the AB/IB recoordination logic before fixing.

- [x] **Investigation:** Understand current search boundary behavior
      (2026-06-01 15:20)
  - Read `foolish-core/src/search.rs` — how does search traverse brane boundaries?
  - Read `foolish-core/src/ubc.rs` — how does AB/IB recoordination work?
  - Trace Bug A: `{nested = {inner = {val = x}}; x = 42;}` — why does `x` resolve?
  - Trace Bug C: `{sum = a + b; nested = {inner = sum / 2};}` — why does `sum` fail?
  - Identify the single rule that governs when search crosses brane boundaries
  - **STOP — Report findings to human before proceeding with fix**

- [x] **Bug A & C:** Fix brane boundary crossing in search
      (2026-06-01 15:45)
  - Apply progressive scoping in `re_step_brane_bodies` (ubc.rs)
  - Key distinction: `x` in Bug A is defined AFTER the nested brane (should NOT resolve)
    vs `sum` in Bug C is defined BEFORE the nested brane (SHOULD resolve)
  - The fix: progressive scoping — only names from statements evaluated before current index are visible
  - Add unit tests for both cases
  - Verify: `cargo test -p foolish-core -- unit_tests` passes

- [x] Regenerate affected `.snap.new` files:
      (2026-06-01 15:50)
  - Run `cargo insta test -p foolish-core --lib`
  - Check `complex_forward_refs_in_nested_branes.foo.snap.new` — `val` should be Search, not Int(42)
  - Check `complex_full_program_with_all_features.foo.snap.new` — `sum` should resolve to Int(30)
  - **STOP — Human review gate:** Present updated `.snap.new` files for Bugs A, C

- [x] On human approval, promote approved `.snap.new` → `.snap` (strip LGTM lines)

## Phase D: Operator precedence bug (Bug B) — requires isolation

Bug B may be a parser precedence issue or an evaluation order issue. Isolate first.

- [x] **Isolation:** Test `b1 (target.c)` to determine if fix is parser or evaluator
      (2026-06-01 15:25)
  - Created a test input and ran through evaluator
  - Root cause: `search_in_brane()` iterated forward (first match wins) instead of backward (last match wins)
  - `target` had two `c` entries (`c=2` and `c={a=1,b=2,c=3}`); forward search found `Int(2)` instead of brane

- [x] **Bug B:** Fix search vs concatenation precedence
      (2026-06-01 15:35)
  - Fixed `search_in_brane()` in search.rs to accept `SearchDirection` parameter
  - Backward direction now iterates in reverse (last match wins)
  - Updated caller in fir.rs to pass `self.direction`
  - Add unit tests
  - Verify: `cargo test -p foolish-core` passes

- [x] Regenerate affected `.snap.new` file:
      (2026-06-01 15:30)
  - Run `cargo insta test -p foolish-core --lib`
  - Check `complex_search_and_concatenation.foo.snap.new` — `result` should be `{x=10; a=1; b=2; c=3}`
  - **STOP — Human review gate:** Present updated `.snap.new` for Bug B

- [x] On human approval, promote approved `.snap.new` → `.snap` (strip LGTM lines)

## Phase E: Workspace verification

- [x] Run `cargo test -p foolish-core --lib` — all tests pass
      (2026-06-01 15:50) — 91/91 passed
- [x] Run `cargo check --workspace` — no compilation errors
- [ ] Run `cargo clippy -p foolish-core` — no new warnings
- [ ] Verify all 7 bug files are now `.snap` (promoted from `.snap.new`)
- [ ] Verify no `@Agent` comments remain in any `.snap` file
- [ ] Run `cargo test --workspace` — full workspace passes

## Phase F: Documentation updates

- [ ] Update `AGENTS.md` Last Updated section
- [ ] Update FOOP-32.md status from Draft → Implementing → complete
- [ ] Update FOOP-32.bugs.md status to "All bugs fixed"

## Phase G: Cleanup and merge

- [ ] Verify all work is complete in ${WORKTREE_FULL_FS_PATH} and committed to `foop/bugfix-rudimentary-foop-32`
- [ ] Merge `foop/bugfix-rudimentary-foop-32` to alpha
  - [ ] If merge conflicts arise:
    - [ ] Repair conflicts
    - [ ] Re-run `cargo test --workspace`
    - [ ] Re-commit
- [ ] Cleanup ${WORKTREE_FULL_FS_PATH}
  - [ ] Check that this plan has all but Cleanup checkboxes completed
  - [ ] Remove "${WORKTREE_FULL_FS_PATH}"
  - [ ] This is the last checkbox to be checked in this plan
