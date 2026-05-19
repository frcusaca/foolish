---
foop: 18
title: FOOP-81 Implementation Plan — SequenceableFir, HumanizingSequencer, SnapshotSuite
status: Superseded
superseded_by: [FOOP-02]
created: 2026-05-15
updated: 2026-05-15
---

# FOOP-81 Implementation Plan

- [x] Canceled. Superseded by FOOP-02 which uses FirQueryable trait instead of SequenceableFir,
      single HumanizingSequencer via trait dispatch, and moves SnapshotSuite to foolish-core.
      (2026-05-19 12:00)

## Worktree

```
STARTING_PATH=/home/hcbusy/foolish-rust
STARTING_BRANCH=foolish-rust
WORKTREE_BRANCH_NAME=snapshot_test_suite-foop-81
WORKTREE_FULL_FS_PATH=${HOME}/tmp/foolish-worktrees/snapshot_test_suite-foop-81
```

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

- [-] Add `SequenceableFir` enum to `foolish-core/src/fir.rs` after `Fir` enum (superseded — FOOP-02 removes SequenceableFir)
- [-] Add `SequenceableStatement` struct (superseded)
- [-] Implement `From<Fir> for SequenceableFir` (superseded)
- [-] Implement accessor methods on `SequenceableFir` (superseded)
- [-] Implement `get_hs_value()` logic (superseded)
- [-] Define `SequenceableError` (superseded)
- [-] Verify `cargo check -p foolish-core` — compiles (superseded)

---

## Phase B: Add HumanizingSequencer to foolish-core

- [-] Extend `foolish-core/src/sequencer.rs` (superseded — FOOP-02 rewrites HumanizingSequencer via FirQueryable)
- [-] Define `HumanizingSequencer` (superseded)
- [-] Implement `format_for_snap_test(indent)` (superseded)
- [-] States mode / Non-states mode (superseded)
- [-] Verify `cargo check -p foolish-core` (superseded)

---

## Phase C: Export new types from foolish-core

- [-] Update `foolish-core/src/lib.rs` (superseded)
- [-] Verify `cargo check -p foolish-core` (superseded)

---

## Phase D: Unit tests for HumanizingSequencer in foolish-core

- [-] Add unit tests (superseded — FOOP-02 has new sequencer tests)
- [-] Comprehensive coverage gate — 35/35 tests (superseded)
- [-] Verify `cargo test -p foolish-core` (superseded)

---

## Phase E: Implement SequenceableFir for UbcbFir in foolish-ubcb

- [-] Implement `to_sequenceable()` for `UbcbFir` (superseded — FOOP-02 uses `to_fir()` via FirQueryable)
- [-] Verify `cargo check -p foolish-ubcb` (superseded)

---

## Phase F: Extract SnapshotSuite to dedicated module

- [-] Create `foolish-ubcb-cli/src/snapshot_suite.rs` (superseded — FOOP-02 moves to foolish-core)
- [-] Move `SnapshotSuite`, formatting helpers (superseded)
- [-] Refactor `SnapshotSuite` struct (superseded)
- [-] Refactor formatting to use `HumanizingSequencer` (superseded)
- [-] Keep `lib.rs` minimal (superseded)
- [-] Move test module (superseded)
- [-] Verify `cargo check` and `cargo test` (superseded)

---

## Phase G: Refactor main.rs to use HumanizingSequencer

- [-] No changes needed (superseded)
- [-] Remove duplicate code (superseded)
- [-] Verify CLI and REPL (superseded)

---

## Phase H: Workspace verification

- [-] Run `cargo check --workspace` (superseded)
- [-] Run `cargo clippy --workspace` (superseded)
- [-] Run `cargo test --workspace` (superseded)
- [-] Verify snapshots unchanged (superseded)
- [-] Verify `foolish-core` approval tests (superseded)

---

## Phase I: Documentation

- [-] Update `AGENTS.md` (superseded)
- [-] Update `README.md` (superseded)
- [-] Update "Last Updated" sections (superseded)
- [-] Update `FOOP-81.md` status (superseded)
- [-] Mark completed checkboxes (superseded)
- [-] Pre-finalize doc verification (superseded)

---

## Phase J: Cleanup and merge

- [-] Verify all work complete and committed (superseded — FOOP-81 merged 2026-05-15)
- [-] Merge to `foolish-rust` branch (superseded — fast-forward merge completed)
- [-] Cleanup worktree (superseded)
