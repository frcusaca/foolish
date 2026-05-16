---
foop: 18
title: FOOP-81 Implementation Plan — SequenceableFir, HumanizingSequencer, SnapshotSuite
status: Draft
created: 2026-05-15
updated: 2026-05-15
---

# FOOP-81 Implementation Plan

## Worktree

```
FULL_WORKTREE_PATH=${HOME}/tmp/foolish-worktrees/____-foop-81
```
(Generate random 4-digit prefix when plan execution begins.)

## Current State (Updated)

| Artifact | Status |
|---|---|
| `foolish-core/src/fir.rs` | `Fir` enum, `Steppable` trait, `Nyes`, FIR structs (~1409 lines) |
| `foolish-core/src/sequencer.rs` | `Sequencer` struct (simple `format()` method, 29 lines) |
| `foolish-core/src/lib.rs` | Exports `Fir`, `Sequencer`, `Steppable`, etc. |
| `foolish-ubcb/src/fir.rs` | `UbcbFir` struct wrapping `FirRef` + `Luid` + inbox |
| `foolish-ubcb-cli/src/lib.rs` | `SnapshotSuite` + formatting helpers (347 lines, inline formatting) |
| `foolish-ubcb-cli/src/main.rs` | CLI (retained formatting helpers for CLI use) |
| `foolish-ubcb-cli/Cargo.toml` | Has `[lib]` target, `insta`, `rayon`, `num_cpus` deps |
| `foolish-ubcb-cli/src/snapshots/` | 2 snapshots (`ubcb_test_1.snap`, `ubcb_test_1_states.snap`) |
| Workspace tests | `cargo test --workspace` passes, clippy clean |

## Design Decisions (Locked)

| Decision | Choice | Rationale |
|---|---|---|
| **SequenceableFir** | Enum wrapping `Fir` variants | Enables exhaustive pattern matching; avoids runtime `get_hs_type()` dispatch |
| **HumanizingSequencer** | In `foolish-core/src/sequencer.rs` | Common FIR formatting for both UBC and UBCb; `foolish-core` has no `foolish-ubcb` dep |
| **SnapshotSuite module** | Dedicated `snapshot_suite.rs` | Complexity warrants own module (same weight as `sequencer.rs`) |
| **Existing `Sequencer`** | Preserved | Backward compatibility — existing code uses it |
| **Parallel eval vs insta** | Parallel UBCb eval + sequential insta assert | Insta is not thread-safe; eval is the expensive part |
| **Result aggregation** | Collect all results, assert sequentially | Fail-At-End pattern |
| **Value resolution** | `get_hs_value()` chains through Search targets until constanic or loop | Resolves search chains for display |
| **Constant types** | Int only for now (String, Float deferred) | Minimum viable scope |

---

## Phase A: Define SequenceableFir enum in foolish-core

- [ ] Add `SequenceableFir` enum to `foolish-core/src/fir.rs` after `Fir` enum:
  ```rust
  pub enum SequenceableFir {
      ConstantInt { value: i64, state: Nyes },
      Nk { reason: String, state: Nyes, alarm: Option<Alarm> },
      Operator { op: String, operands: Vec<SequenceableFir>, state: Nyes },
      Search { pattern: String, direction: SearchDirection, anchored: bool,
                anchor: Option<Box<SequenceableFir>>,
                target: Option<Box<SequenceableFir>>, state: Nyes },
      Index { offset: i32, anchored: bool,
               anchor: Option<Box<SequenceableFir>>, state: Nyes },
      HeadTail { is_head: bool, anchored: bool,
                 anchor: Option<Box<SequenceableFir>>, state: Nyes },
      StayFoolish { expr: Box<SequenceableFir>, state: Nyes },
      StayFullyFoolish { expr: Box<SequenceableFir>, state: Nyes },
      Concatenation { elements: Vec<SequenceableFir>,
                      merged: Option<Box<SequenceableFir>>, state: Nyes },
      NormalBrane { characterizations: Vec<String>,
                    statements: Vec<SequenceableStatement>, state: Nyes },
  }
  ```
- [ ] Add `SequenceableStatement` struct:
  ```rust
  pub struct SequenceableStatement {
      pub name: Option<String>,
      pub body: SequenceableFir,
  }
  ```
- [ ] Implement `From<Fir> for SequenceableFir`:
  - Recursively convert each `Fir` variant to corresponding `SequenceableFir` variant
  - For `FirRef` children (operands, anchors, targets, elements, statements), dereference and recurse
  - Must handle the `Rc<RefCell<dyn Steppable>>` indirection
- [ ] Implement accessor methods on `SequenceableFir`:
  - `get_hs_type(&self) -> &'static str` — variant name
  - `hs_get_nyes(&self) -> Nyes` — state
  - `get_hs_children(&self) -> Vec<&SequenceableFir>` — child references
  - `get_hs_parent(&self) -> Option<&SequenceableFir>` — always `None`
  - `get_hs_value(&self) -> Result<SequenceableFir, SequenceableError>` — resolve search chain
- [ ] Implement `get_hs_value()` logic:
  - If FIR is constanic → return clone of self
  - If FIR is `Search` with a `target` → recurse on target
  - If chain loops (visited set exceeds threshold) → return error
  - For `ConstantInt` variant: `get_hs_int_value(&self) -> i64`
- [ ] Define `SequenceableError`:
  ```rust
  #[derive(Debug, thiserror::Error)]
  pub enum SequenceableError {
      #[error("Loop detected in search chain (depth: {depth})")]
      LoopDetected { depth: usize },
  }
  ```
- [ ] Verify `cargo check -p foolish-core` — compiles

---

## Phase B: Add HumanizingSequencer to foolish-core

- [ ] Extend `foolish-core/src/sequencer.rs`:
  - Keep existing `Sequencer` struct unchanged
  - Add `HumanizingSequencer` struct after existing code
- [ ] Define `HumanizingSequencer`:
  ```rust
  pub struct HumanizingSequencer {
      fir: SequenceableFir,
  }

  impl HumanizingSequencer {
      pub fn new(fir: SequenceableFir) -> Self { Self { fir } }
      pub fn format_for_snap_test(&self, indent: usize) -> String;
      pub fn format_for_repl(&self, indent: usize) -> String;
      pub fn fir(&self) -> &SequenceableFir { &self.fir }
  }
  ```
- [ ] Implement `format_for_snap_test(indent)`:
  - Match on `SequenceableFir` enum (exhaustive)
  - Single-statement branes: one line
  - Multi-statement branes: continuation lines start after `indent` spaces
  - For `ConstantInt`: `"Int({value})"` with state tag if not constanic
  - For `Nk`: `"NK({reason})"` with alarm info if present
  - For `NormalBrane`: `"{stmt1; stmt2; ...}"` with name = value format
  - For `Search`: `"Search(pattern='...', direction=..., anchor='...')" ` with resolved target
  - For `Operator`: Use resolved value if constanic, otherwise `"Operator(op='...', operands=[...])"`
  - For other variants: Use appropriate display format
  - State tag: Append `"[{nyes_state}]"` for non-constanic FIRs
  - **Recursive indent**: When formatting child FIRs (nested branes, Concatenation elements, Search targets), create a new `HumanizingSequencer` with `indent + 2`
- [ ] **States mode**: When `with_states` is true, always show NYES tags
- [ ] **Non-states mode**: Strip NYES tags from output
- [ ] Verify `cargo check -p foolish-core` — compiles

---

## Phase C: Export new types from foolish-core

- [ ] Update `foolish-core/src/lib.rs`:
  ```rust
  pub use fir::{..., SequenceableFir, SequenceableStatement, SequenceableError};
  pub use sequencer::{Sequencer, HumanizingSequencer};
  ```
- [ ] Verify `cargo check -p foolish-core` — compiles

---

## Phase D: Unit tests for HumanizingSequencer in foolish-core

- [ ] Add unit tests in `foolish-core/src/lib.rs` (new test module):
  - Test `SequenceableFir::from(Fir)` conversion for each variant
  - Test `get_hs_type()` returns correct variant name
  - Test `hs_get_nyes()` returns correct state
  - Test `get_hs_children()` returns correct children
  - Test `get_hs_value()` resolves search chains
  - Test `get_hs_value()` detects loops
  - Test `HumanizingSequencer::format_for_snap_test(indent)` for:
    - Empty brane → `"{}"`
    - Single constant → `"Int(42)"`
    - Brane with named statement → `"{x = Int(42);}"`
    - Multi-statement brane with continuation lines (indent > 0)
    - Search FIR → `"Search(pattern='...', ...)"`
    - NK FIR → `"NK(reason)"`
    - Operator FIR (constanic and nye)
    - Concatenation FIR (nested children with recursive indent)
    - Index, HeadTail, StayFoolish, StayFullyFoolish variants
  - Test states mode includes NYES tags
  - Test non-states mode strips NYES tags
  - Test `format_for_repl(indent)` produces expected output
- [ ] **Comprehensive coverage gate** — verify every `SequenceableFir` variant and every `HumanizingSequencer` method has at least one dedicated test. Do NOT proceed to Phase E until this passes; repairing both sequencer and snapshot tests simultaneously is too complex.
- [ ] Verify `cargo test -p foolish-core` — all tests pass

---

## Phase E: Implement SequenceableFir for UbcbFir in foolish-ubcb

- [ ] In `foolish-ubcb/src/fir.rs`, implement conversion for `UbcbFir`:
  ```rust
  impl UbcbFir {
      pub fn to_sequenceable(&self) -> SequenceableFir {
          // Convert inner FirRef to SequenceableFir
          let fir = clone_steppable(&self.fir);
          SequenceableFir::from(fir)
      }
  }
  ```
- [ ] Verify `cargo check -p foolish-ubcb` — compiles

---

## Phase F: Extract SnapshotSuite to dedicated module

- [ ] Create `foolish-ubcb-cli/src/snapshot_suite.rs`
- [ ] Move `SnapshotSuite`, `SnapshotSuiteError`, `TestFailure` from `lib.rs` to `snapshot_suite.rs`
- [ ] Move formatting helpers (`format_result`, `fmt_stmt`, `fmt_fir_inline`, `strip_nyes_tag`, `fmt_anchor`) to `snapshot_suite.rs`
- [ ] Refactor `SnapshotSuite` struct:
  - Replace `input_dir: PathBuf` with `base_dir: PathBuf`, `input_pattern: String`, `golden_pattern: String`
  - `(*)` in patterns is the capture group for test case names
- [ ] Refactor `new()` to accept `(base_dir, input_pattern, golden_pattern)` and validate pairing (missing snapshots, missing inputs)
- [ ] Refactor formatting to use `HumanizingSequencer`:
  - `evaluate()` converts `EvaluationResult` FIRs to `SequenceableFir`
  - Uses `HumanizingSequencer::format_for_snap_test(indent)` for output
  - Preserves states/non-states mode behavior
- [ ] Keep `lib.rs` minimal — re-export from `snapshot_suite.rs`:
  ```rust
  pub mod snapshot_suite;
  pub use snapshot_suite::{SnapshotSuite, SnapshotSuiteError, TestFailure};
  ```
- [ ] Move test module (`#[cfg(test)] mod approval_tests`) to `snapshot_suite.rs`
- [ ] Verify `cargo check -p foolish-ubcb-cli` — compiles
- [ ] Verify `cargo test -p foolish-ubcb-cli --lib` — tests pass

---

## Phase G: Refactor main.rs to use HumanizingSequencer

- [ ] In `foolish-ubcb-cli/src/main.rs`:
  - Import `HumanizingSequencer` from `foolish_core`
  - Replace inline formatting with `HumanizingSequencer` where applicable
  - Keep CLI-specific formatting (step output, REPL output) — these may need different formats
- [ ] Remove duplicate formatting code from `main.rs` (if any overlaps with `snapshot_suite.rs`)
- [ ] Verify `cargo run -p foolish-ubcb-cli -- run <test_file>` — CLI works
- [ ] Verify `cargo run -p foolish-ubcb-cli -- repl` — REPL works

---

## Phase H: Workspace verification

- [ ] Run `cargo check --workspace` — no compilation errors
- [ ] Run `cargo clippy --workspace` — no new warnings
- [ ] Run `cargo test --workspace` — all tests pass
- [ ] Verify snapshot files unchanged:
  - `foolish-ubcb-cli/src/snapshots/foolish_ubcb_cli__approval_tests__ubcb_test_1.snap`
  - `foolish-ubcb-cli/src/snapshots/foolish_ubcb_cli__approval_tests__ubcb_test_1_states.snap`
- [ ] Verify `foolish-core` approval tests still pass

---

## Phase I: Documentation

- [ ] Update `AGENTS.md` — add references to `HumanizingSequencer` and `SequenceableFir`
- [ ] Update `README.md` snapshot test section if needed
- [ ] Update both files' "Last Updated" sections
- [ ] Update `FOOP-81.md` — set status to `Implementing` (already done)
- [ ] Mark completed checkboxes in this plan with timestamps
- [ ] **Pre-finalize doc verification** — confirm `README.md` and `AGENTS.md` snapshot test commands are accurate:
  - Run commands listed in README match actual test targets (`-p foolish-ubcb-cli --lib`, etc.)
  - Verify `cargo insta review` / `cargo insta accept` workflow documented correctly
  - Verify `INSTA_UPDATE=always` env-var mentioned

---

## Phase J: Cleanup and merge

- [ ] Verify all work is complete and committed
- [ ] Merge to `foolish-rust` branch
  - [ ] If merge conflicts arise:
    - [ ] Repair conflicts
    - [ ] Re-run `cargo test --workspace`
    - [ ] Re-commit
- [ ] Cleanup worktree (if used)
  - [ ] Check that this plan has all but Cleanup checkboxes completed
  - [ ] Remove the worktree directory
  - [ ] This is the last checkbox to be checked in this plan
