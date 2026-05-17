---
foop: 20
title: Move SnapshotSuite to foolish-core and unify approval testing
author: opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4
status: Draft
type: Refactor
created: 2026-05-17
phase: phase-3
supersedes: []
---

# FOOP-02: Move SnapshotSuite to foolish-core and unify approval testing

## Abstract

Move `SnapshotSuite` from `foolish-ubcb-cli` into `foolish-core` and unify
approval testing so both the original UBC (`foolish-core::ubc`) and UBCb
(`foolish-ubcb`) use the same snapshot infrastructure. This eliminates
duplicated test harness code and ensures consistent output formatting
across implementations.

## Motivation

Currently `SnapshotSuite` lives in `foolish-ubcb-cli` and is tightly coupled
to `UbcbEngine::evaluate` which returns `EvaluationResult` / `StatementResult`
(types defined in `foolish-ubcb`). The original UBC in `foolish-core::ubc` has
its own approval tests in `lib.rs` that manually compile, evaluate, and format
output. This duplication means:

1. Format changes must be applied in two places.
2. New tests added to one harness don't automatically cover the other.
3. The snapshot suite cannot be reused for UBC tests.

After this refactor, `SnapshotSuite` is a generic approval test harness in
`foolish-core` that accepts any evaluator function. Both UBC and UBCb plug in
their respective evaluators.

## Specification

### Current State

**`foolish-ubcb-cli/src/snapshot_suite.rs`** (~334 lines):
- `SnapshotSuite` struct — discovers `.foo` files, runs evaluation, compares snapshots
- `SnapshotSuiteError`, `TestFailure` error types
- `format_result` / `fmt_fir_inline` — uses `SequenceableFir` + `HumanizingSequencer` (from core)
- `UbcbEngine`-specific: calls `UbcbEngine::evaluate`, formats `EvaluationResult`
- Test module: `approval_all`, `approval_all_states` — uses `insta`

**`foolish-core/src/lib.rs`** — approval tests:
- `run_foo()` — compiles, evaluates with `ubc::run_to_completion`, formats with `Sequencer`
- 100+ `#[test]` functions, each calling `insta::assert_snapshot!` directly
- No `SnapshotSuite`-style file discovery or parallel evaluation

**Key types:**
| Type | Location | Used by |
|------|----------|---------|
| `SequenceableFir`, `SequenceableStatement` | `foolish-core::fir` | Both |
| `HumanizingSequencer` | `foolish-core::sequencer` | Both |
| `EvaluationResult`, `StatementResult` | `foolish-ubcb::engine` | UBCb only |
| `UbcbEngine` | `foolish-ubcb::engine` | UBCb only |

### Target Architecture

```
foolish-core/
  src/
    snapshot_suite.rs   <-- NEW (moved from ubcb-cli, generalized)
    lib.rs              <-- exports SnapshotSuite, removes inline approval tests

foolish-ubcb-cli/
  src/
    lib.rs              <-- re-exports from core, provides UBCb evaluator adapter
    main.rs             <-- unchanged

foolish-core/
  snapshot_tests/       <-- NEW: UBC approval test inputs
    input/              <-- .foo files (shared or UBC-specific)
    approved/           <-- .snap files

foolish-ubcb-cli/
  snapshot_tests/       <-- EXISTING: UBCb approval test inputs (unchanged)
    input/
    approved/
```

### Changes

#### 1. Move and generalize `SnapshotSuite` to `foolish-core`

**File:** `foolish-core/src/snapshot_suite.rs` (new)

The `SnapshotSuite` struct moves largely unchanged, but the `evaluate` method
becomes generic over an evaluator function instead of being hardcoded to
`UbcbEngine`:

```rust
// Before (in ubcb-cli):
pub fn evaluate(&self, path: &Path, with_states: bool) -> Result<String, String> {
    let source = fs::read_to_string(path)?;
    let mut engine = UbcbEngine::new();
    let result = engine.evaluate(&source)?;
    Ok(format_result(&result, with_states))
}

// After (in foolish-core):
pub fn evaluate<F>(&self, path: &Path, with_states: bool, evaluator: &F) -> Result<String, String>
where
    F: Fn(&str) -> Result<Vec<StatementOutput>, String>,
{
    let source = fs::read_to_string(path)?;
    let stmts = evaluator(&source)?;
    Ok(format_statements(&stmts, with_states))
}
```

`StatementOutput` is a new lightweight type in `foolish-core`:
```rust
pub struct StatementOutput {
    pub name: Option<String>,
    pub fir: FirRef,
}
```

This replaces `StatementResult` (which lives in `foolish-ubcb` and includes
extra fields like `state`).

The formatting functions (`format_statements`, `fmt_fir_inline`) move to core.
They use `SequenceableFir::from(clone_steppable(fir))` + `HumanizingSequencer`
which are already in core.

Remove `SnapshotSuiteError` and `TestFailure` — these are internal to the
suite and don't need to be public API. Keep `SnapshotSuite` and its methods
public.

#### 2. Add `foolish-ubcb` as a dev-dependency to `foolish-core`

To allow UBCb snapshot tests to run from `foolish-core`, add:
```toml
# foolish-core/Cargo.toml
[dev-dependencies]
foolish-ubcb = { path = "../foolish-ubcb" }
rayon = "1"
num_cpus = "1"
```

`insta` is already a dev-dependency.

#### 3. Provide UBCb evaluator adapter in `foolish-ubcb-cli`

**File:** `foolish-ubcb-cli/src/lib.rs`

Provide an adapter function that wraps `UbcbEngine::evaluate` into the
`SnapshotSuite` signature:

```rust
pub fn ubcb_evaluator(source: &str) -> Result<Vec<foolish_core::StatementOutput>, String> {
    let mut engine = foolish_ubcb::UbcbEngine::new();
    let result = engine.evaluate(source)
        .map_err(|e| format!("UBCb evaluation failed: {}", e))?;
    Ok(result.statements.into_iter().map(|s| foolish_core::StatementOutput {
        name: s.name,
        fir: s.fir,
    }).collect())
}
```

Add UBCb approval tests in `foolish-ubcb-cli`:
```rust
#[cfg(test)]
mod approval_tests {
    use foolish_core::SnapshotSuite;
    use super::ubcb_evaluator;

    fn suite() -> SnapshotSuite { /* ... */ }

    #[test] fn approval_all() { /* suite + evaluator + insta */ }
    #[test] fn approval_all_states() { /* same with states=true */ }
}
```

Remove `snapshot_suite.rs` from `foolish-ubcb-cli` (deleted).

#### 4. Provide UBC evaluator adapter and tests in `foolish-core`

This is the **potentially challenging step** (see "Open Questions" below).

The original UBC evaluates a single `FirRef` via `ubc::run_to_completion`.
After evaluation, the brane's `statements` need to be extracted and formatted
as `StatementOutput` items.

```rust
pub fn ubc_evaluator(source: &str) -> Result<Vec<StatementOutput>, String> {
    let firs = Compiler::compile(source)
        .map_err(|e| format!("Compilation failed: {}", e))?;
    let mut fir_ref = fir_to_ref(firs[0].clone());
    ubc::run_to_completion(&mut fir_ref)
        .map_err(|e| format!("Evaluation failed: {}", e))?;
    let final_fir = clone_steppable(&fir_ref);

    // Extract statements from the brane
    match final_fir {
        Fir::NormalBrane(nb) => {
            Ok(nb.statements.iter().map(|stmt| StatementOutput {
                name: stmt.name.as_ref().map(|n| n.as_str().to_string()),
                fir: Rc::clone(&stmt.body) as FirRef,
            }).collect())
        }
        _ => Err("Expected brane".to_string()),
    }
}
```

**Risk:** The UBC may produce FIR structures where the `body` references don't
map cleanly to independent `FirRef` items suitable for `SequenceableFir`
conversion. If `SequenceableFir::from(Fir)` expects a fully cloned/deep-copied
tree, `clone_steppable` on each statement body may be needed.

UBC approval tests in `foolish-core`:
```rust
#[cfg(test)]
mod approval_tests {
    use super::ubc_evaluator;
    use crate::SnapshotSuite;

    fn suite() -> SnapshotSuite { /* ... */ }

    #[test] fn approval_all() { /* ... */ }
    #[test] fn approval_all_states() { /* ... */ }
}
```

Remove the current inline approval test module (~600 tests) from `lib.rs`.
These tests become file-driven via `SnapshotSuite`. Each existing test
corresponds to a `.foo` file in `snapshot_tests/input/`.

**Sub-question:** Should we keep the inline tests for speed (no file I/O,
targeted test names) and add `SnapshotSuite` tests alongside? This is
discussed in "Rejected Alternatives."

#### 5. Set up `snapshot_tests/` directory structure for UBC

Create `foolish-core/snapshot_tests/input/` and `foolish-core/snapshot_tests/approved/`.
Populate with `.foo` files that correspond to the existing inline approval
tests. Each file named `{test_name}.foo` contains the source, e.g.:

```
foolish-core/snapshot_tests/input/simple_addition.foo
{3 + 4;}
```

The approved snapshots can be generated by running the tests once with
`INSTA_UPDATE=always`.

### Dependency Flow (After)

```
foolish-core
  ├── foolish-parser
  ├── snapshot_suite.rs (new)
  ├── ubc_evaluator (new, in test module or public)
  └── dev-deps: insta, rayon, num_cpus, foolish-ubcb

foolish-ubcb
  └── foolish-core

foolish-ubcb-cli
  ├── foolish-core (for SnapshotSuite)
  ├── foolish-ubcb (for UbcbEngine + adapter)
  └── dev-deps: insta, rayon, num_cpus
```

## FIR Impact

None.

## UBC Step Impact

None.

## Test Plan

1. **Verify UBCb tests still pass** after moving `SnapshotSuite`:
   - `cargo test -p foolish-ubcb-cli --lib`
   - Confirm all snapshot tests pass with existing approved files.

2. **Verify UBC tests pass** with new `SnapshotSuite`-based tests:
   - `cargo test -p foolish-core -- approval`
   - Generate initial snapshots: `INSTA_UPDATE=always cargo test -p foolish-core -- approval`

3. **Full workspace check**:
   - `cargo check --workspace`
   - `cargo test --workspace`

## Rejected Alternatives

### A. Keep `SnapshotSuite` in `foolish-ubcb-cli`, create separate suite for UBC

This maintains the status quo of duplicated infrastructure. Any format
change requires updating both. Rejected because it defeats the purpose of
unification.

### B. Keep inline tests in `lib.rs`, don't convert to file-driven

Inline tests are fast and provide clear test names. However, they can't
leverage `SnapshotSuite`'s parallel evaluation or file discovery. A
hybrid approach (keep inline tests AND add `SnapshotSuite` tests) doubles
the test count. Rejected as the primary approach — inline tests are removed
in favor of file-driven tests for consistency.

### C. Make `SnapshotSuite` a standalone crate

Over-engineering for two consumers. A module in `foolish-core` suffices.

## Open Questions

1. **UBC FIR extraction:** Can `clone_steppable` on each `StatementFir.body`
   produce a valid `Fir` for `SequenceableFir::from()`? If the UBC evaluation
   leaves bodies in a state where the clone is incomplete or references are
   broken, this needs investigation. **If this proves difficult, ask the
   human before proceeding.**

2. **Inline vs. file-driven tests:** Should we keep some inline tests in
   `lib.rs` for rapid development (no file I/O, easy to add new cases),
   and use `SnapshotSuite` only for the full approval suite?

3. **Shared input files:** Should UBC and UBCb share the same `.foo` input
   files (cross-validation), or maintain separate directories? Sharing
   enables automatic cross-validation but may require different approved
   snapshots if implementations diverge.

4. **`_states` variant:** Should the `with_states` flag (which appends FIR
   state to output) become a configurable option on `SnapshotSuite`
   construction, or remain a per-call parameter?

## References

- Current snapshot suite: `foolish-ubcb-cli/src/snapshot_suite.rs`
- UBC approval tests: `foolish-core/src/lib.rs` (lines 262-1403)
- HumanizingSequencer: `foolish-core/src/sequencer.rs`
- `SequenceableFir`, `SequenceableStatement`: `foolish-core/src/fir.rs`
- `EvaluationResult`, `StatementResult`: `foolish-ubcb/src/engine.rs`
