---
foop: 17
title: Snapshot testing with cargo-insta for UBCb — approval testing infrastructure
author: Sisyphus <agent>
status: Deprecated
type: Standards
created: 2026-05-15
phase: meta
supersedes: []
---

# FOOP-71: Snapshot testing with cargo-insta for UBCb

> **Status: Deprecated** (2026-07-03 18:23)
>
> Canceled as it stands. This feature should be later respecified and reimplemented.

## Abstract

Adopt **cargo-insta** (the `insta` crate) as the canonical snapshot/approval
testing framework for the UBCb implementation. This replaces the hand-rolled
`ApprovalSuite` in `foolish-ubcb-cli` with insta-based snapshot tests and
establishes clear documentation so that snapshot tests become the primary
validation mechanism for UBCb development.

The initial implementation uses a single comprehensive test file
(`ubcb_test_1.foo`) evaluated through a refactored REPL component that treats
each line as a separate ROOT brane. Cross-validation between UBC and UBCb,
snapshot testing for UBC itself, and migration of existing `foolish-core`
snapshots are deferred to separate FOOPs.

Snapshot testing — also called "approval testing," "gold master testing,"
"characterization testing," "reference testing," or "baseline testing" — compares
the actual output of a program against a previously approved reference. It is
the natural fit for a language implementation: given a `.foo` source file, the
output of the VM is the observable behavior, and the approved output is the
behavioral contract.

## Motivation

### Current state

The workspace currently has multiple approval test systems:

1. **foolish-core** — Uses `insta::assert_snapshot!` correctly. 194 `.snap` files
   in `foolish-core/src/snapshots/`. These snapshots should be inspected after
   this FOOP's feature completion and migrated as needed via a separate FOOP.

2. **foolish-ubcb-cli** — Has a hand-rolled `ApprovalSuite` (~400 lines) that reads
   `.foo` from `approval_test_input/`, compares against `.foo.approved` golden
   masters in `approval_test_output/`, and writes `.foo.received` on mismatch.
   It also has 12 existing insta snapshots for inline tests. The two systems
   coexist awkwardly.

3. **foolish-ubcb** — Declares `insta` as a dev-dependency but uses it for nothing.

4. **Java/Scala JVM implementations** — Have their own approval test infrastructure
   outside the Rust workspace. These are NOT touched by this FOOP.

### Why insta?

1. **Human-readable inputs and outputs** — `.foo` source files and their
   Sequencer-formatted output are readable by humans. Insta snapshots preserve
   this readability.
2. **Deterministically hooked to the codebase** — Each snapshot test runs the
   compiler and VM directly. No indirection through shell scripts or external tools.
3. **Built-in tools for editing expected outputs** — `cargo insta review` provides
   an interactive TUI for accepting/rejecting snapshot changes. This is critical
   when UBCb behavior evolves during development and needs manual approval.
4. **Standard workflow** — `cargo test` detects pending snapshots.
   `cargo insta review` approves them. `cargo insta accept` bulk-accepts.
   `cargo insta reject` bulk-rejects.

### What snapshot testing covers

Snapshot tests in this project serve multiple testing roles simultaneously:

| Testing Role | Snapshot Role |
|---|---|
| **Unit test** | Verify a single FIR evaluates correctly |
| **Integration test** | Verify compiler + VM pipeline produces correct output |
| **Regression test** | A bug was detected, the snapshot captured the broken state, was fixed, and the snapshot now preserves the correct behavior |
| **Characterization test** | Document current behavior of a feature even if the "correct" answer is debated |

When a user says "write a regression test for that bug in snapshot," they mean:
replicate the bug detection as a snapshot test, fix the bug so the snapshot
reflects correct behavior, and keep the snapshot to prevent future regression.

## Specification

### 1. Scope boundaries

**In scope:**
- `foolish-ubcb-cli` — Replace `ApprovalSuite` with insta snapshot tests
- `foolish-ubcb-cli/approval_test_input/ubcb_test_1.foo` — Sole active test input
- Snapshot documentation in `AGENTS.md` and `README.md`

**Out of scope (deferred to future FOOPs):**
- Cross-validation between UBC and UBCb (separate FOOP to be specified)
- Snapshot testing for UBC (`foolish-core`) — existing 194 snapshots to be inspected and FOOP'd after feature completion
- Java/Scala JVM approval tests — not touched
- Re-enabling the 24 `.foo.disabled` files in `approval_test_input/`

### 2. Workspace insta configuration

The workspace `Cargo.toml` already declares:

```toml
insta = { version = "1", features = ["yaml"] }
```

`foolish-ubcb-cli` adds `insta = { workspace = true }` as a dev-dependency.

### 3. Snapshot directory convention

Snapshots for `foolish-ubcb-cli` live at:

```
foolish-ubcb-cli/src/snapshots/
```

Files are named (insta default):

```
foolish_ubcb_cli__approval_tests__<test_name>.snap
```

### 4. Evaluation model: REPL-style ROOT brane evaluation

The test infrastructure uses a refactored component from the current REPL that
evaluates each line as a separate ROOT brane. This means:

- Each line of `ubcb_test_1.foo` is compiled and evaluated independently
- The REPL component provides the parse-and-evaluate cycle per line
- Each line's output (parsed FIR + evaluation result) is captured

**Why this model?** The current REPL evaluates each line as a ROOT brane.
Later, when the REPL is upgraded to a multi-turn REPL (where each line is the
next line of an accumulating ROOT brane), this test infrastructure will be
upgradable to that setting by changing the evaluation mode, not the test files.

### 5. Snapshot content format

Every snapshot contains the input source and the evaluation output:

```yaml
---
source: foolish-ubcb-cli/src/lib.rs
expression: output
---
INPUT: {3 + 4;}
[0] PARSED:
Brane [EMBRYONIC]
  Operator(+) [EMBRYONIC]
    Int(3) [INDEPENDENT]
    Int(4) [INDEPENDENT]
RESULT:
Brane [CONSTANT]
  Int(7) [CONSTANT]
```

The `Sequencer::format_with_header()` function produces the output format.

### 6. States output format

The states-flagged snapshot variant annotates each FIR with its NYES state
**only when the state is not CONSTANT or INDEPENDENT**. This keeps the output
concise while still documenting non-terminal states (EMBRYONIC, BRANING,
CONSTANIC, NYES, etc.) that are diagnostically useful.

For example, an FIR that is CONSTANT or INDEPENDENT appears without a state
tag. An FIR that is EMBRYONIC appears as `Search(...) [EMBRYONIC]`.

This convention is standardized for all tests.

### 7. Migration: foolish-ubcb-cli ApprovalSuite → insta

The current `ApprovalSuite` in `foolish-ubcb-cli/src/main.rs` (lines ~249-556)
and the old `mod approval_tests` (lines ~562-665) are replaced with an
insta-based module in `foolish-ubcb-cli/src/lib.rs`.

**Why move to `lib.rs`?** Currently `foolish-ubcb-cli` is a `bin` crate only.
Having a `lib.rs` with the test modules is the standard Rust pattern for
testable crates.

The migration steps:

1. Add `src/lib.rs` to `foolish-ubcb-cli` with `mod approval_tests`.
2. The `approval_tests` module contains tests for `ubcb_test_1.foo` (normal and states modes).
3. The old `ApprovalSuite` struct, `approval_test_input/` (except `ubcb_test_1.foo`),
   `approval_test_output/`, and `approval_test_output_states/` are removed.
4. The 12 existing insta snapshots in `foolish-ubcb-cli/src/snapshots/` are
   retained as-is (they test inline expressions, not file-based approval).

### 8. Featured comprehensive test: `ubcb_test_1.foo`

A single, comprehensive test file `ubcb_test_1.foo` exercises the majority
of UBCb's CP-1 capabilities. It is located at:

```
foolish-ubcb-cli/approval_test_input/ubcb_test_1.foo
```

**What it tests:**

| Section | Feature | Notes |
|---|---|---|
| `start=1` | Literal + identification | Baseline |
| `test_identifier` | Nested brane with name resolution | `a=1; b=a` inside child brane |
| `test_ancestral_identifier_with_shadow` | Ancestral search + shadowing | Multi-level scope, nested branes (`n1`, `n2`), arithmetic with shadowed names |
| `test_preserving_constanics` | Constanic search preservation | `a=b` before `b=1` — search remains constanic |
| `test_preserving_constanics_over_shadows` | Constanic across shadow boundaries | Inner brane shadows `a` and `b`; constanic searches should not resolve across shadow boundaries |

### 9. First-failure verification gate

**Important:** When the snapshot test system is first run, `ubcb_test_1.foo`
is expected to produce a **failure** on the first run (a new snapshot is
pending, which insta treats as a test failure). This first failure is itself
a verification that the test system works — it proves that:

1. The test can compile and execute
2. The UBCb engine can evaluate the input
3. The Sequencer can format the output
4. Insta can detect the mismatch between the new output and the absent snapshot

**The agent must stop at this first failure** and present the output to the
user for review before proceeding. Do not auto-accept. The workflow is:

1. Run the test — insta detects a pending snapshot (test "fails"):
   ```bash
   cargo test -p foolish-ubcb-cli -- ubcb_test_1
   ```
2. **STOP.** Present the pending snapshot to the user.
3. The user inspects:
   - Are the FIR trees correctly parsed?
   - Are constanic searches preserved or resolved as expected?
   - Are arithmetic results correct given shadowing rules?
   - Are states annotated correctly (only non-CONSTANT/non-INDEPENDENT)?
4. If correct → `cargo insta accept` → verify tests pass
5. If incorrect → diagnose UBCb engine bug → fix → re-run → re-review

### 10. Command reference

The following commands must be documented in both `AGENTS.md` and `README.md`:

| Command | Purpose |
|---|---|
| `cargo test -p foolish-ubcb-cli -- ubcb_test_1` | Run the featured comprehensive test |
| `cargo test -p foolish-ubcb-cli -- approval` | Run all UBCb approval/snapshot tests |
| `cargo test --workspace -- approval` | Run all approval tests across workspace |
| `INSTA_UPDATE=always cargo test -p foolish-ubcb-cli -- approval` | Force-update all UBCb snapshots |
| `cargo insta review` | Interactive TUI: accept/reject pending snapshots |
| `cargo insta accept` | Accept all pending snapshots |
| `cargo insta reject` | Reject all pending snapshots |
| `cargo insta test --review -p foolish-ubcb-cli` | Run UBCb tests then immediately review |

## FIR Impact

None. This FOOP changes test infrastructure only.

## UBC Step Impact

None. This FOOP changes test infrastructure only.

## Test Plan

1. **Infrastructure build**: Add `src/lib.rs` to `foolish-ubcb-cli` with
   `mod approval_tests` containing `ubcb_test_1` and `ubcb_test_1_states`.
   Evaluation uses the refactored REPL component (line-as-ROOT-brane mode).
2. **First-failure gate**: Run the test, expect insta to report a pending
   snapshot (test failure). Verify this proves the pipeline works.
3. **Human review gate**: Present the pending snapshot to the user. Do NOT
   accept until the user confirms correctness.
4. **Accept or fix**: If correct, `cargo insta accept`. If incorrect, diagnose
   and fix the UBCb engine, then repeat.
5. **Remove old ApprovalSuite**: Strip the hand-rolled `ApprovalSuite` from
   `src/main.rs`, remove old output directories.
6. **Documentation**: Update `AGENTS.md` and `README.md` with snapshot
   testing conventions and command reference.

## Deferred work (separate FOOPs)

The following items are explicitly deferred and should each be their own FOOP:

1. **Cross-validation snapshot testing** — Specify a new FOOP for snapshot-based
   cross-validation between UBC (`foolish-core`) and UBCb (`foolish-ubcb`).
   This FOOP should be created and specified as a follow-up task during or
   after the completion of this FOOP.

2. **UBC snapshot testing** — `foolish-core` has 194 existing insta snapshots.
   After this FOOP is feature-complete, inspect these snapshots, assess whether
   they need migration, renaming, or restructuring, and FOOP the changes.

3. **Re-enabling remaining test inputs** — The 24 `.foo.disabled` files in
   `approval_test_input/` can be re-enabled once the snapshot infrastructure
   is proven stable.

## Rejected Alternatives

### A. Keep the hand-rolled ApprovalSuite

The current `ApprovalSuite` works but requires ~400 lines of maintenance code,
has no interactive review tool, and produces `.foo.received` files that clutter
the working directory. Insta provides all this functionality in a single,
well-maintained crate.

### B. Use `assert_cmd` for CLI-level snapshot testing

`assert_cmd` is designed for testing CLI binaries via subprocess invocation.
It would test the `foolish-ubcb-cli` binary's stdout, which is less precise
than testing the evaluation engine directly. Direct testing catches more issues
and runs faster.

### C. Centralize all snapshots in one shared crate

A single `foolish-tests` crate could house all snapshots. This would eliminate
duplication but create a circular dependency problem — `foolish-tests` would
need to depend on every crate it tests, and the snapshot locations would be
far from the source code under test. Per-crate snapshots keep tests close to
the code they test.

### D. Include cross-validation in this FOOP

Cross-validation between UBC and UBCb adds significant complexity: wiring the
UBC evaluation pipeline through `foolish-ubcb`, handling known divergences at
CP-1, and maintaining dual-output snapshots. It deserves its own FOOP with
proper scoping. Keeping this FOOP narrow (UBCb approval only) ensures it ships
faster and provides a foundation that the cross-validation FOOP can build on.

## Open Questions

- What is the exact API of the refactored REPL component for line-as-ROOT-brane
  evaluation? (To be determined during implementation.)
- Should the 12 existing insta snapshots in `foolish-ubcb-cli/src/snapshots/`
  be migrated to the new module structure, or left as-is?
- Should `foolish-parser` get snapshot tests for AST output, or are the existing
  unit tests (asserting on parsed structures directly) sufficient?

## References

- [FOOP-31](FOOP-31.md) — SPA1 milestone, defines approval test role
- [FOOP-41](FOOP-41.md) — UBCb parity plan, defines checkpoint cross-validation
- [phase2_ubc.md](../ubc1/todo/rust-mvp/phase2_ubc.md) — Phase 2 spec, approval test harness description
- [phase1_compiler.md](../ubc1/todo/rust-mvp/phase1_compiler.md) — Phase 1 spec, Sequencer output format
- [ubc2_design.md](../ubc1/how/ubc2_design.md) — UBC2 design, FIR lifecycle stages
- [insta.rs](https://insta.rs/) — Official insta documentation
- [foolish-core approval tests](../../foolish/foolish-core/src/lib.rs) — Current insta usage (L261-1403)
- [foolish-ubcb-cli ApprovalSuite](../../foolish/foolish-ubcb-cli/src/main.rs) — Current hand-rolled system (L249-556)

---

## Last Updated

**Date**: 2026-07-04
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Status -> Deprecated. Canceled as it stands per user request; feature should be later respecified and reimplemented. Added Deprecation Notice section.
