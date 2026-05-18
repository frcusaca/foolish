---
foop: 20
title: Consolidate FIR formatting into HumanizingSequencer; move SnapshotSuite to core
author: opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4
status: Draft
type: Refactor
created: 2026-05-17
phase: phase-3
supersedes: []
---

# FOOP-02: Consolidate FIR formatting; unify approval testing

## Abstract

Three-step refactor (executed in order) to eliminate duplicated FIR-to-String
formatting code and unify approval testing across both VM implementations:

1. Introduce a `FirQueryable` trait. Move ALL FIR-to-String formatting into
   `HumanizingSequencer` operating against the trait. Remove every `format()`
   implementation from `Steppable`.
2. Move `SnapshotSuite` from `foolish-ubcb-cli` into `foolish-core`, generalized
   over an evaluator function.
3. Adapt the original UBC's evaluated FIRs into `SnapshotSuite` via `FirQueryable`.

After this, there is one formatting path, one snapshot harness, and both UBC and
UBCb produce identical approval test output through the same infrastructure.

## Motivation

### Two formatting paths, ~800 lines combined

**Path A - `Sequencer` (used by `foolish-cli`, inline UBC tests):**
- `Sequencer::format(fir: &Fir)` delegates to `Steppable::format(&self, buf, depth)`
- 10 separate `impl Steppable` blocks across `fir.rs` (lines ~658-1342)
- Each variant writes 15-40 lines into a `&mut String` buffer
- Used by: `foolish-cli` (REPL, run, step), `lib.rs` inline tests

**Path B - `HumanizingSequencer` (used by `SnapshotSuite`, sequencer unit tests):**
- Single match on `SequenceableFir` in `sequencer.rs` (lines 55-134, ~100 lines)
- Returns owned `String`, recursive with indent parameter
- Requires `SequenceableFir::from(Fir)` deep clone first
- Used by: `foolish-ubcb-cli/snapshot_suite.rs`, `lib.rs` sequencer tests

Any formatting change (new variant, output style adjustment, state display)
must be applied to both paths. The APIs are incompatible (buffer vs owned string).

### SnapshotSuite coupled to UBCb

`SnapshotSuite` lives in `foolish-ubcb-cli`, hardcoded to `UbcbEngine::evaluate`
which returns `EvaluationResult` (a type defined in `foolish-ubcb`). The original
UBC has 130+ inline `#[test]` functions in `lib.rs` that manually compile, evaluate,
and format. Adding a test to one harness doesn't cover the other.

## Specification

### Core Invariant: Single Formatting Path

**`HumanizingSequencer` (via `FirQueryable` trait) is THE ONLY path for converting
any `Fir` into human-readable strings — period.**

After this FOOP:
- Every `Fir` → `String` conversion goes through `HumanizingSequencerRef::new(&dyn FirQueryable)`.
- `SnapshotSuite` never formats FIR directly — it delegates to `HumanizingSequencer`.
- `foolish-cli` (REPL, run, step) formats via `Sequencer::format` which delegates to `HumanizingSequencer`.
- No `Steppable::format()`, no `Display` on FIR, no ad-hoc formatting anywhere.

### HumanizingSequencer Formatting Rules

`HumanizingSequencer` produces properly indented, multi-line output:

1. **Single-statement branes** are rendered inline: `Brane{a = Int(1)}`
2. **Multi-statement branes** are rendered with each statement on its own indented line:
   ```
   Brane{
     a = Int(1);
     b = Int(2);
   }
   ```
3. **Indentation propagates recursively** — nested branes inherit the parent's indent level plus the standard increment (2 spaces).
4. **This formatting behavior is permanent** — it exists in `foolish-core` and is preserved across all refactors, including the move of `SnapshotSuite` to core.

### Step 1: Trait-based sequencing (all FIR-to-String in HumanizingSequencer)

**Scope:** `foolish-core/src/fir.rs`, `foolish-core/src/sequencer.rs`

#### 1A. Introduce `FirQueryable` trait

A trait with structured accessors for every FIR variant. Both `Fir` and
`SequenceableFir` implement it. `HumanizingSequencer` knows ONLY this trait --
never `Fir` directly, never `SequenceableFir` directly.

```
pub trait FirQueryable: std::fmt::Debug {
    // Identity
    fn hs_variant(&self) -> &'static str;
    fn hs_state(&self) -> Nyes;

    // Each accessor returns Option<...>. Some only for the matching variant.
    // New variants just add new accessor methods -- no exhaustive match required.

    fn hs_constant_int(&self) -> Option<i64>;

    fn hs_nk(&self) -> Option<(&str, &Option<Alarm>)>;

    fn hs_operator(&self) -> Option<(&str, Vec<Box<dyn FirQueryable>>)>;

    fn hs_search(&self) -> Option<(
        &str,              // pattern
        SearchDirection,
        bool,              // anchored
        Option<Box<dyn FirQueryable>>,  // anchor
        Option<Box<dyn FirQueryable>>,  // target
    )>;

    fn hs_index(&self) -> Option<(i32, bool, Option<Box<dyn FirQueryable>>)
    // (offset, anchored, anchor)

    fn hs_head_tail(&self) -> Option<(bool, bool, Option<Box<dyn FirQueryable>>)
    // (is_head, anchored, anchor)

    fn hs_stay_foolish(&self) -> Option<Box<dyn FirQueryable>>;
    fn hs_stay_fully_foolish(&self) -> Option<Box<dyn FirQueryable>>;

    fn hs_concatenation(&self) -> Option<(
        Vec<Box<dyn FirQueryable>>,     // elements
        Option<Box<dyn FirQueryable>>,  // merged
    )>;

    fn hs_brane(&self) -> Option<(
        Vec<String>,                     // characterizations
        Vec<SequenceableStatement>,      // statements (name, body)
    )>;
}
```

**Design note:** Child FIRs are returned as `Box<dyn FirQueryable>`. This avoids
exposing `Rc<RefCell<Fir>>` or `SequenceableFir` through the trait boundary.
For `Fir`, a thin `FirChildRef` wrapper adapts `Rc<RefCell<Fir>>` to the trait
without cloning.

#### 1B. Implement `FirQueryable` for `Fir`

New struct in `fir.rs`:

```
pub struct FirChildRef { inner: Rc<RefCell<Fir>> }
impl FirQueryable for FirChildRef { ... }
```

Then `impl FirQueryable for Fir` with each accessor returning `Some(...)` for
the matching variant, `None` otherwise. Child `Rc<RefCell<Fir>>` fields are
wrapped in `FirChildRef`.

**Where:** New section in `fir.rs`, after `SequenceableFir::from(Fir)`.

#### 1C. Implement `FirQueryable` for `SequenceableFir`

Same accessors. Child `SequenceableFir`s are boxed directly (cheap, already owned).

**Where:** New section in `fir.rs`.

#### 1D. Rewrite `HumanizingSequencer` to use the trait

Current: `fn format_fir(fir: &SequenceableFir, indent: usize) -> String` with
a `match fir { SequenceableFir::... }` on 10 variants.

New: `fn hs_format_fir(fir: &dyn FirQueryable, indent: usize) -> String` that
dispatches via trait accessor methods.

`HumanizingSequencer` (owned `SequenceableFir`) is removed entirely. Only
`HumanizingSequencerRef` (`&dyn FirQueryable`) remains. This is THE formatting
path — `SnapshotSuite` uses it exclusively. No direct FIR formatting anywhere.

```
fn hs_format_fir(fir: &dyn FirQueryable, indent: usize) -> String {
    if let Some(val) = fir.hs_constant_int() {
        format!("Int({})", val)
    } else if let Some((reason, alarm)) = fir.hs_nk() {
        // format NK
    } else if let Some((op, operands)) = fir.hs_operator() {
        // format Operator, recurse on operands
    } else if let Some(...) = fir.hs_search() {
        // format Search, recurse on anchor/target
    } ...
}
```

The formatting logic is identical -- only the dispatch mechanism changes from
pattern matching on an enum to querying trait accessors.

Only `HumanizingSequencerRef` for `&dyn FirQueryable`:

```
pub struct HumanizingSequencerRef<'a> { fir: &'a dyn FirQueryable }
impl<'a> HumanizingSequencerRef<'a> {
    pub fn new(fir: &'a dyn FirQueryable) -> Self { ... }
    pub fn format_with_indent(&self, indent: usize) -> String { ... }
}
```

#### 1E. Remove `Steppable::format()` and update `Sequencer`

Remove `format(&self, buf: &mut String, depth: usize)` from the `Steppable`
trait definition and all 10 implementations (~350 lines removed from `fir.rs`).

`Sequencer::format(fir: &Fir)` now adapts `&Fir` to `&dyn FirQueryable` and
delegates to `HumanizingSequencerRef`:

```
pub fn format(fir: &Fir) -> String {
    HumanizingSequencerRef::new(fir).format_for_snap_test()
}
```

#### 1F. Update callers

- `foolish-cli/src/main.rs` -- uses `Sequencer::format(&final_fir)` -- no change
- `lib.rs` inline tests -- uses `Sequencer::format` -- no change
- `foolish-ubcb-cli/snapshot_suite.rs` -- uses `HumanizingSequencer` -- no change
  (Step 2 will migrate this to `HumanizingSequencerRef`)

### Step 1.5: FIR Builders (UBC and UBCb)

**Scope:** `foolish-core/src/fir.rs`, `foolish-ubcb/src/fir.rs`

#### 1.5A. UBC FIR Builders

Remove `SequenceableFir`, `SequenceableStatement`, `SequenceableError` entirely.

Introduce one builder struct per FIR variant. Each builder uses a fluent API:

```
pub struct ConstantIntFirBuilder {
    value: i64,
    state: Nyes,
}
impl ConstantIntFirBuilder {
    pub fn new(value: i64) -> Self { Self { value, state: Nyes::Prembrionic } }
    pub fn state(mut self, state: Nyes) -> Self { self.state = state; self }
    pub fn build(self) -> Fir {
        Fir::ConstantInt(Box::new(ConstantIntFir { value: self.value, state: self.state }))
    }
}
```

Full set: `ConstantIntFirBuilder`, `NkFirBuilder`, `OperatorFirBuilder`,
`SearchFirBuilder`, `IndexFirBuilder`, `HeadTailFirBuilder`,
`StayFoolishFirBuilder`, `StayFullyFoolishFirBuilder`, `ConcatenationFirBuilder`,
`NormalBraneFirBuilder`.

#### 1.5B. Builder Unit Tests

Each builder gets unit tests:
- Construct with minimal fields, verify `build()` produces correct `Fir`.
- Set all optional fields, verify correctness.
- Wrap result in `FirRef`, format via `FirQueryable`, verify output.

#### 1.5C. UBCb FIR Builders

Analogous builders in `foolish-ubcb/src/fir.rs` that construct `UbcbFir`
instead of `Fir`. Same fluent pattern. Same test structure.

#### 1.5D. Migrate Sequencer Tests

Hand-constructed sequencer tests in `lib.rs` (~140 lines) are rewritten. Two
options for constructing test FIRs — use whichever is clearer for the test:

**Option A — Builder (preferred for unit tests):** Direct, no parser overhead.
```
let fir = OperatorFirBuilder::new("+")
    .add_operand(ConstantIntFirBuilder::new(1).build())
    .add_operand(ConstantIntFirBuilder::new(2).build())
    .state(Nyes::Constant)
    .build();
```

**Option B — Parse from Foolish source:** When the test is about
compiler/evaluator behavior, parse through the normal pipeline:
```
let firs = Compiler::compile("{a = 1 + 2;}").unwrap();
let fir = firs[0].clone();
```

Use builders for focused unit tests (single FIR, specific state, edge cases).
Use parse/compile for integration tests (full pipeline, scope resolution, etc.).
No separate FIR class needed for either approach.

#### 1.5E. Remove `SequenceableFir` from `HumanizingSequencer`

`HumanizingSequencer` (owned `SequenceableFir`) is removed. Only
`HumanizingSequencerRef` (`&dyn FirQueryable`) remains.

### Step 2: Move SnapshotSuite to foolish-core

**Scope:** `foolish-core/src/snapshot_suite.rs` (new), `foolish-ubcb-cli/src/snapshot_suite.rs` (deleted),
`foolish-ubcb-cli/src/lib.rs` (updated), `foolish-core/Cargo.toml`, `foolish-ubcb-cli/Cargo.toml`

#### 2A. Move and generalize `SnapshotSuite`

The `SnapshotSuite` struct (discovery, input/approved directory management) is
engine-agnostic. Move to `foolish-core/src/snapshot_suite.rs`.

#### 2B. SnapshotSuite must NOT format FIR directly

`SnapshotSuite` is an orchestrator — it discovers files, runs evaluators, and
delegates ALL formatting to `HumanizingSequencerRef` via `FirQueryable`.
It must never contain its own FIR-to-String logic.

The `fmt_fir_inline`, `fmt_stmt`, and `format_result` helpers in the current
`snapshot_suite.rs` are moved to `foolish-core` and rewritten to use
`HumanizingSequencerRef`. No FIR formatting escapes this single path.

#### 2C. Replace hardcoded `UbcbEngine` with evaluator function

Current `evaluate` is hardcoded:

```
pub fn evaluate(&self, path: &Path, with_states: bool) -> Result<String, String> {
    let source = fs::read_to_string(path)?;
    let mut engine = UbcbEngine::new();
    let result = engine.evaluate(&source)?;
    Ok(format_result(&result, with_states))
}
```

New `evaluate` accepts any evaluator:

```
pub fn evaluate<F>(&self, path: &Path, with_states: bool, evaluator: &F) -> Result<String, String>
where F: Fn(&str) -> Result<Vec<StatementOutput>, String>
```

`StatementOutput` is a new lightweight type in `foolish-core`:

```
pub struct StatementOutput {
    pub name: Option<String>,
    pub fir: FirRef,  // Rc<RefCell<Fir>>
}
```

#### 2D. Add dev-dependencies to foolish-core

```
[dev-dependencies]
foolish-ubcb = { path = "../foolish-ubcb" }
rayon = "1"
num_cpus = "1"
```

#### 2E. UBCb adapter in foolish-ubcb-cli

`foolish-ubcb-cli/src/lib.rs` provides:

```
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

Remove `snapshot_suite.rs` from `foolish-ubcb-cli`.

### Step 3: Sequence UBC FIRs for SnapshotSuite

**Scope:** `foolish-core/src/lib.rs` (adapter + tests), `foolish-core/snapshot_tests/` (new directory)

#### 3A. UBC evaluator adapter

The original UBC evaluates via `ubc::run_to_completion`. After evaluation,
extract statements from the brane as `StatementOutput`:

```
pub fn ubc_evaluator(source: &str) -> Result<Vec<StatementOutput>, String> {
    let firs = Compiler::compile(source)
        .map_err(|e| format!("Compilation failed: {}", e))?;
    let mut fir_ref = fir_to_ref(firs[0].clone());
    ubc::run_to_completion(&mut fir_ref)
        .map_err(|e| format!("Evaluation failed: {}", e))?;
    let final_fir = clone_steppable(&fir_ref);

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

**Risk:** `Rc::clone(&stmt.body)` may not produce a fully independent `FirRef`
if the UBC evaluation leaves internal Rc references. If `clone_steppable` on
each body is needed instead, that is a one-line fix. **If this proves difficult,
ask the human before proceeding.**

#### 3B. UBC approval tests

```
#[cfg(test)]
mod ubc_approval_tests {
    use super::ubc_evaluator;
    use crate::SnapshotSuite;

    fn suite() -> SnapshotSuite { ... }
    #[test] fn approval_all() { ... }
    #[test] fn approval_all_states() { ... }
}
```

#### 3C. Populate snapshot_tests/input

Create `foolish-core/snapshot_tests/input/` and `foolish-core/snapshot_tests/approved/`.
Start with ~20 representative `.foo` files (arithmetic, search, concatenation,
scope, alarms). Grow incrementally.

#### 3D. Remove inline tests (eventually)

After `SnapshotSuite` coverage is verified equivalent, remove the 130+ inline
`#[test]` functions from `lib.rs`.

## FIR Impact

None. `Fir` struct unchanged. `Steppable` trait remains for `step()` semantics;
only `format()` is removed.

## UBC Step Impact

None. Evaluation logic unchanged. Only the formatting and test infrastructure changes.

## Test Plan

Phase-gated -- each step verified before proceeding:

1. **After Step 1:** `cargo test --workspace` -- all tests pass. `foolish-cli` produces identical output.
2. **After Step 2:** `cargo test -p foolish-ubcb-cli --lib` -- UBCb tests pass with existing approved files.
3. **After Step 3:** `INSTA_UPDATE=always cargo test -p foolish-core -- approval` -- generate snapshots.
4. **Full workspace:** `cargo check --workspace && cargo test --workspace`

## Rejected Alternatives

### A. Keep both formatting paths

Maintains duplication. Any future variant or style change requires updating both. Rejected.

### B. Keep SnapshotSuite in ubcb-cli, create parallel suite for UBC

Two harnesses, two format functions, two test infrastructures. Defeats the purpose. Rejected.

### C. Keep inline tests AND add SnapshotSuite tests

Doubles test count (~260). Rejected as primary approach -- migrate to file-driven.

### D. Make SnapshotSuite a standalone crate

Over-engineering for two consumers. A module in `foolish-core` suffices.

## Design Decisions

1. **UBC FIR extraction (Step 3A):** `Rc::clone(&stmt.body)` is sufficient.
   `FirChildRef` wraps `Rc<RefCell<Fir>>` and implements `FirQueryable` by
   borrowing — zero cloning, zero allocation. `clone_steppable` is only needed
   for independent mutable copies, not read-only formatting.

2. **Remove SequenceableFir entirely. Introduce FIR builders for both UBC and UBCb.**
   No separate FIR class for testing. Both UBC and UBCb get their own builder
   structs — one per FIR variant — that construct and return `Fir` values directly.
   Example: `ConstantIntFirBuilder::new(42).state(Nyes::Constant).build()`.
   Builders have their own unit tests. Snapshot tests parse `.foo` files through
   the normal compiler pipeline. The existing `*Fir` structs
   (`ConstantIntFir`, `NkFir`, `OperatorFir`, etc.) serve as the basis for
   builder APIs. UBCb gets analogous builders for `UbcbFir`.

3. **Separate input files:** UBC and UBCb maintain separate `snapshot_tests/input/`
   directories. They are at too different stages of development to share inputs.

4. **Inline test migration:** Copy ALL ~256 inline approval tests to `.foo` files.
   Leave everything failing initially — the point is to have the full test corpus
   in the snapshot harness, not to have passing tests immediately.

## References

- `Steppable::format` implementations: `foolish-core/src/fir.rs` lines ~658-1342 (10 impls)
- `HumanizingSequencer` / `HumanizingSequencerRef`: `foolish-core/src/sequencer.rs`
- `SnapshotSuite`: `foolish-ubcb-cli/src/snapshot_suite.rs` (334 lines)
- Inline UBC tests: `foolish-core/src/lib.rs` lines 262-1403 (~130 tests)
- Sequencer unit tests (hand-constructed): `foolish-core/src/lib.rs` lines 1406-1795 (~140 lines)
- `SequenceableFir` (to be removed): `foolish-core/src/fir.rs` lines 403-920
- `*Fir` structs (builder basis): `foolish-core/src/fir.rs` lines 209-310
- `UbcbFir`: `foolish-ubcb/src/fir.rs`
- `EvaluationResult`, `StatementResult`: `foolish-ubcb/src/engine.rs` lines 13-25
