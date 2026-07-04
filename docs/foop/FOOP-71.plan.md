---
foop: 17
title: FOOP-71 Implementation Plan — cargo-insta snapshot testing for UBCb
status: Draft
created: 2026-05-15
---

# FOOP-71 Implementation Plan

## Worktree

FULL_WORKTREE_PATH=${HOME}/tmp/foolish-worktrees/____-foop-71
(Generate random 4-digit prefix when plan execution begins.)

## Scope Note

Only `ubcb_test_1.foo` is active. All other `.foo` files in
`approval_test_input/` are renamed to `.foo.disabled`. The snapshot
infrastructure is developed with this single comprehensive test file.
Re-enabling the remaining files is deferred to a future FOOP.

**Java/Scala JVM implementations are NOT touched by this plan.**

---

## Phase A: foolish-ubcb-cli — Add lib.rs with snapshot test for ubcb_test_1

Build the minimal insta-based snapshot infrastructure around `ubcb_test_1.foo`,
using a refactored component from the current REPL that evaluates each line as
a separate ROOT brane.

- [x] Canceled. This feature should be later respecified and reimplemented.
      (2026-07-03 18:23)
- [-] Create worktree at ${HOME}/tmp/foolish-worktrees/12063-foop-71 with branch `foop/insta-snapshots-71`
- [-] Refactor REPL evaluation logic from `foolish-ubcb-cli` (or shared module)
  - Extract the parse-and-evaluate-per-line cycle into a reusable function
  - Each line is treated as a ROOT brane (current REPL behavior)
  - The function returns per-line output: (input_source, parsed_fir, evaluation_result)
  - **Note:** This design is forward-compatible — when the REPL is later upgraded
    to multi-turn mode (each line appended to an accumulating ROOT brane), this
    function can be reconfigured without changing the test files
- [-] Add `src/lib.rs` to `foolish-ubcb-cli` (currently bin-only)
  - Expose the refactored REPL evaluation function
  - Define `evaluate_and_format(source: &str, with_states: bool) -> String`
  - States formatting convention: only display NYES state tags when the state
    is NOT CONSTANT or NOT INDEPENDENT
- [-] In `src/lib.rs`, create `mod approval_tests` with two tests:
  - `fn ubcb_test_1()` — reads `approval_test_input/ubcb_test_1.foo`, evaluates line-by-line via REPL component, snapshots output
  - `fn ubcb_test_1_states()` — same input, with states flag, snapshots state-annotated output
- [-] Run `cargo test -p foolish-ubcb-cli -- ubcb_test_1` — expect **failure**
  (new snapshot pending — this first failure verifies the pipeline works)
- [-] **STOP — First-failure verification:**
  - Confirm that the test compiled, executed, and produced output
  - Confirm that insta detected the pending snapshot (test failure)
  - This proves: compilation ✓, evaluation ✓, formatting ✓, snapshot detection ✓
- [-] **STOP — Human review gate:**
  - Present the pending snapshot output to the user for inspection
  - User checks: FIR trees correct? Constanic searches preserved/resolved as expected?
    Arithmetic results correct under shadowing? States annotated only for
    non-CONSTANT/non-INDEPENDENT FIRs?
  - If user confirms correct → proceed to accept
  - If user finds errors → diagnose UBCb engine bug, fix, re-run, re-review
- [-] After user approval, run `cargo insta accept` to finalize the snapshot(s)
- [-] Verify `cargo test -p foolish-ubcb-cli -- ubcb_test_1` passes (both normal and states)
- [-] Commit: add lib.rs, approval_tests module, snapshots; `.foo.disabled` files already in place

## Phase B: Remove old ApprovalSuite

Clean up the hand-rolled approval testing infrastructure.

- [-] Remove `ApprovalSuite` struct and all related code from `src/main.rs` (lines ~249-556)
- [-] Remove old `mod approval_tests` from `src/main.rs` (lines ~562-665)
- [-] Remove `approval_test_output/` directory (golden masters, replaced by insta)
- [-] Remove `approval_test_output_states/` directory
- [-] Retain `approval_test_input/` directory (contains `ubcb_test_1.foo` + `.foo.disabled` files)
- [-] Retain existing 12 insta snapshots in `src/snapshots/` (inline expression tests)
- [-] Verify `cargo test -p foolish-ubcb-cli` still passes after removal
- [-] Verify `cargo check --workspace` — no compilation errors
- [-] Commit: remove old ApprovalSuite infrastructure

## Phase C: Documentation updates — AGENTS.md and README.md

Document the snapshot testing conventions and commands.

- [-] Add "Snapshot Testing" section to `AGENTS.md`:
  - What it is: approval testing, gold master, characterization test, reference test, baseline test
  - Why Foolish uses it:
    1. Human-readable inputs (`.foo`) and outputs (Sequencer-formatted FIR trees)
    2. Deterministically hooked to the codebase under test
    3. Built-in tools (`cargo insta review`) for editing expected outputs
    4. Unit tests, integration tests, and regression tests all appear in snapshot suites — this is normal
  - Regression test workflow: replicate bug detection in snapshot → fix → keep snapshot to prevent regression
  - Command reference:
    ```
    cargo test -p foolish-ubcb-cli -- ubcb_test_1       # featured comprehensive test
    cargo test -p foolish-ubcb-cli -- approval           # all UBCb snapshot tests
    cargo test --workspace -- approval                   # all snapshot tests across workspace
    INSTA_UPDATE=always cargo test -p foolish-ubcb-cli -- approval  # force-update
    cargo insta review                                   # interactive accept/reject TUI
    cargo insta accept                                   # bulk accept pending snapshots
    cargo insta reject                                   # bulk reject pending snapshots
    cargo insta test --review -p foolish-ubcb-cli        # run + review in one step
    ```
  - States format convention: only non-CONSTANT/non-INDEPENDENT FIRs display state tags
  - First-failure verification: first run of a new snapshot test is expected to fail
    (pending snapshot) — this proves the pipeline works before human review
- [-] Add condensed "Snapshot Testing" section to `README.md` with same command table
- [-] Update both files' "Last Updated" sections
- [-] Commit documentation

## Phase D: Workspace verification

- [-] Run `cargo test --workspace` — all tests pass
- [-] Run `cargo check --workspace` — no compilation errors
- [-] Run `cargo clippy --workspace` — no new warnings from changes
- [-] Verify snapshot files exist:
  - `foolish-ubcb-cli/src/snapshots/foolish_ubcb_cli__approval_tests__ubcb_test_1.snap`
  - `foolish-ubcb-cli/src/snapshots/foolish_ubcb_cli__approval_tests__ubcb_test_1_states.snap`
- [-] Verify existing snapshots untouched:
  - `foolish-core/src/snapshots/` (~194 files, unchanged)
  - `foolish-ubcb-cli/src/snapshots/` (12 existing inline snapshots, unchanged)
- [-] Verify old infrastructure removed:
  - No `ApprovalSuite` in `foolish-ubcb-cli/src/main.rs`
  - No `approval_test_output/` directory
  - No `approval_test_output_states/` directory

## Phase E: Specify deferred FOOPs

Before completing this plan, identify the follow-up FOOPs needed.

- [-] Ask the user to specify a new FOOP for:
  - **Cross-validation snapshot testing** between UBC and UBCb
  - **UBC snapshot inspection** — review the 194 existing `foolish-core` snapshots,
    assess migration needs, and FOOP changes as needed
- [-] Record the new FOOP numbers in this plan's notes
- [-] Update FOOP-71.md "Deferred work" section with the new FOOP references

## Phase F: Cleanup and merge

- [-] Verify all work is complete in ${HOME}/tmp/foolish-worktrees/12063-foop-71 and committed to `foop/insta-snapshots-71`
- [-] Merge `foop/insta-snapshots-71` to alpha
  - [-] If merge conflicts arise:
    - [-] Repair conflicts
    - [-] Re-run `cargo test --workspace`
    - [-] Re-commit
- [-] Cleanup ${HOME}/tmp/foolish-worktrees/12063-foop-71
  - [-] Check that this plan has all but Cleanup checkboxes completed
  - [-] Remove "${HOME}/tmp/foolish-worktrees/12063-foop-71"
  - [-] This is the last checkbox to be checked in this plan
