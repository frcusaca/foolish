---
foop: 20
title: Consolidate FIR formatting into HumanizingSequencer; move SnapshotSuite to core
author: opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4
status: Implementing
type: Refactor
created: 2026-05-17
phase: phase-3
supersedes: []
---

# FOOP-02: Consolidate FIR formatting; unify approval testing

## Abstract

Three-step refactor (in order):

**Step 1** - Introduce `FirQueryable` trait so `HumanizingSequencer` formats `Fir` directly without cloning to `SequenceableFir`. Remove all `format()` implementations from `Steppable`. All FIR-to-String formatting lives in one place.

**Step 2** - Move `SnapshotSuite` from `foolish-ubcb-cli` to `foolish-core`, generalized over an evaluator function instead of hardcoded to `UbcbEngine`.

**Step 3** - Adapt original UBC evaluated FIRs into `SnapshotSuite` via `FirQueryable`. Both UBC and UBCb use the same snapshot harness.

## Motivation

### Current state - two formatting paths, ~800 lines combined

**Path A - `Sequencer` (used by `foolish-cli`, inline UBC tests):**
- `Sequencer::format(fir: &Fir)` delegates to `Steppable::format(&self, buf: &mut String, depth: usize)`
- 10 separate `impl Steppable` blocks across `fir.rs` (~658-1342), each 15-40 lines
- Writes into `&mut String` buffer, recursive with depth tracking
- Used by: `foolish-cli` (REPL, run, step), `lib.rs` inline tests

**Path B - `HumanizingSequencer` (used by `SnapshotSuite`, sequencer unit tests):**
- Single match on `SequenceableFir` in `sequencer.rs` lines 55-134 (~100 lines)
- Returns owned `String`, recursive with indent parameter
- Requires `SequenceableFir::from(Fir)` clone first (deep clone via `clone_steppable`)
- Used by: `foolish-ubcb-cli/snapshot_suite.rs`, `lib.rs` sequencer tests

**SnapshotSuite problem:**
- Lives in `foolish-ubcb-cli`, coupled to `UbcbEngine` + `EvaluationResult` + `StatementResult` (types in `foolish-ubcb`)
- Original UBC has 130+ inline `#[test]` in `lib.rs` with manual compile/evaluate/format
- Adding a test to one harness does not cover the other

### Target state

- ONE formatting path: `HumanizingSequencer` operates on `FirQueryable` trait
- `Sequencer` becomes a thin wrapper that adapts `&Fir` to `&dyn FirQueryable`
- `SnapshotSuite` in `foolish-core` accepts any evaluator closure
- `SequenceableFir` retained for hand-constructed test FIRs (avoids parser overhead)
- `Steppable::format()` removed entirely

## Specification

### Step 1: Trait-based sequencing

**Scope:** `foolish-core/src/fir.rs`, `foolish-core/src/sequencer.rs`

#### 1A. New trait: `FirQueryable`

Define in `fir.rs`. Structured accessors for every FIR variant. Both `Fir` and `SequenceableFir` implement it. This is the ONLY interface `HumanizingSequencer` knows about.

Trait sketch (not exhaustive - implementors fill in each method):

```
pub trait FirQueryable: std::fmt::Debug {
    // Identity
    fn hs_variant(&self) -> &'static str;
    fn hs_state(&self) -> Nyes;

    // Each variant returns Option<...> - Some only for matching variant, None otherwise.
    // This avoids exhaustive match in the trait, new variants just add new methods.

    fn hs_constant_int(&self) -> Option<i64>;

    fn hs_nk(&self) -> Option<(&str, &Option<Alarm>)>;
    // returns (reason, &alarm)

    fn hs_operator(&self) -> Option<(&str, Vec<Box<dyn FirQueryable>>)>;
    // returns (op, operands)

    fn hs_search(&self) -> Option<(
        &str,              // pattern
        SearchDirection,
        bool,              // anchored
        Option<Box<dyn FirQueryable>>,  // anchor FIR
        Option<Box<dyn FirQueryable>>,  // target FIR
    )>;

    fn hs_index(&self) -> Option<(i32, bool, Option<Box<dyn FirQueryable>>)
    // returns (offset, anchored, anchor FIR)

    fn hs_head_tail(&self) -> Option<(bool, bool, Option<Box<dyn FirQueryable>>)
    // returns (is_head, anchored, anchor FIR)

    fn hs_stay_foolish(&self) -> Option<Box<dyn FirQueryable>>;
    fn hs_stay_fully_foolish(&self) -> Option<Box<dyn FirQueryable>>;

    fn hs_concatenation(&self) -> Option<(Vec<Box<dyn FirQueryable>>, Option<Box<dyn FirQueryable>>)
    // returns (elements, merged)

    fn hs_brane(&self) -> Option<(Vec<&str>, Vec<SequenceableStatement>)
    // returns (characterizations, statements)
    // Note: statements use SequenceableStatement (name: Option<String>, body: SequenceableFir)
    // because statement bodies need to be boxed recursively
}
```

**Key design decision:** The trait returns `Box<dyn FirQueryable>` for child FIRs. For `Fir`, this means we need to wrap `Rc<RefCell<Fir>>` children in a lightweight wrapper that implements `FirQueryable`. For `SequenceableFir`, the children are already owned so boxing is cheap.

#### 1B. Implement `FirQueryable` for `Fir`

Each method returns `Some(...)` for the matching variant and `None` otherwise. Child FIRs (`Rc<RefCell<Fir>>`) are wrapped in a thin adapter:

```
pub struct FirChildRef {
    inner: Rc<RefCell<Fir>>,
}
impl FirQueryable for FirChildRef { ... }
```

This avoids cloning. The adapter simply borrows through the Rc.

**Where:** New section in `fir.rs`, after the `SequenceableFir::from(Fir)` impl.

**Methods to implement:** ~12 accessor methods, each is a small match on the `Fir` enum.

#### 1C. Implement `FirQueryable` for `SequenceableFir`

Each method returns `Some(...)` for the matching variant. Child `SequenceableFir`s are boxed directly.

**Where:** New section in `fir.rs`.

**Methods to implement:** Same ~12 accessor methods.

#### 1D. Rewrite `HumanizingSequencer`

Replace `HumanizingSequencer { fir: SequenceableFir }` with:

```
pub struct HumanizingSequencer<'a> {
    fir: &'a dyn FirQueryable,
}
```

OR keep it generic over owned `SequenceableFir` AND add a `HumanizingSequencerRef` for borrowed `&dyn FirQueryable`. The trait methods produce the same data, so the `format_fir` match can be shared.

**Preferred approach:** Keep `HumanizingSequencer` as-is for `SequenceableFir` (preserves existing API). Add a new method or wrapper struct `HumanizingSequencerRef` that works with `&dyn FirQueryable`. Both use the same internal `format_fir` logic - the trait just provides the data, the formatting logic is identical.

Actually, simplest: make `format_fir` a free function that takes `&dyn FirQueryable`. Both `HumanizingSequencer` and `HumanizingSequencerRef` call it.

```
fn hs_format_fir(fir: &dyn FirQueryable, indent: usize) -> String {
    // dispatch via trait methods:
    if let Some(val) = fir.hs_constant_int() { ... }
    else if let Some((reason, alarm)) = fir.hs_nk() { ... }
    ...
}
```

This replaces the current `match fir { SequenceableFir::... }` pattern.

#### 1E. Remove `Steppable::format()`

After Step 1D, remove `format(&self, buf: &mut String, depth: usize)` from the `Steppable` trait and all 10 implementations. Update `Sequencer::format` to adapt `&Fir` to `&dyn FirQueryable` and call `HumanizingSequencer`.

**Files affected:**
- `fir.rs` - remove `format` from trait + 10 impls (~350 lines removed)
- `sequencer.rs` - `Sequencer::format` now delegates to `HumanizingSequencer`
- `foolish-cli/src/main.rs` - may need `FirQueryable` import

#### 1F. Update callers of `Sequencer::format`

- `foolish-cli/src/main.rs` - uses `Sequencer::format(&final_fir)` - works via adapter
- `lib.rs` inline tests - uses `Sequencer::format` - works via adapter
- No behavior change - just a different internal path

### Step 2: Move SnapshotSuite to foolish-core

**Scope:** `foolish-core/src/snapshot_suite.rs` (new), `foolish-ubcb-cli/src/snapshot_suite.rs` (deleted), `foolish-ubcb-cli/src/lib.rs` (updated), `foolish-core/Cargo.toml`, `foolish-ubcb-cli/Cargo.toml`

#### 2A. Move `SnapshotSuite` struct

Move largely unchanged from `foolish-ubcb-cli/src/snapshot_suite.rs` to `foolish-core/src/snapshot_suite.rs`. The `SnapshotSuite` struct (discovery, input/approved directories) is engine-agnostic.

#### 2B. Generalize evaluation

Replace `UbcbEngine`-specific `evaluate` with a closure parameter:

Before:
```
pub fn evaluate(&self, path: &Path, with_states: bool) -> Result<String, String> {
    let source = fs::read_to_string(path)?;
    let mut engine = UbcbEngine::new();
    let result = engine.evaluate(&source)?;
    Ok(format_result(&result, with_states))
}
```

After:
```
pub fn evaluate<F>(&self, path: &Path, with_states: bool, evaluator: &F) -> Result<String, String>
where F: Fn(&str) -> Result<Vec<StatementOutput>, String>
{
    let source = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    let stmts = evaluator(&source)
        .map_err(|e| format!("Evaluation failed: {}", e))?;
    Ok(format_statements(&stmts, with_states))
}
```

`StatementOutput` is a new lightweight type in `foolish-core`:
```
pub struct StatementOutput {
    pub name: Option<String>,
    pub fir: FirRef,  // Rc<RefCell<Fir>>
}
```

#### 2C. Move formatting helpers

Move `format_result`, `fmt_stmt`, `fmt_fir_inline` to core. After Step 1, `fmt_fir_inline` uses `HumanizingSequencer` via `FirQueryable`:

```
fn fmt_fir_inline(fir: &FirRef, indent: usize, states: bool) -> String {
    let wrapper = FirChildRef { inner: Rc::clone(fir) };
    let output = hs_format_fir(&wrapper, indent);
    if states {
        format!("{} [{}]", output, fir.borrow().state())
    } else {
        output
    }
}
```

#### 2D. Add dev-dependencies to foolish-core

```
# foolish-core/Cargo.toml [dev-dependencies]
foolish-ubcb = { path = "../foolish-ubcb" }
rayon = "1"
num_cpus = "1"
```

`insta` is already present.

#### 2E. Provide UBCb adapter in foolish-ubcb-cli

`foolish-ubcb-cli/src/lib.rs`:
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

UBCb approval tests call `suite().evaluate_all(...)` with `&ubcb_evaluator`.

Remove `snapshot_suite.rs` from `foolish-ubcb-cli`.

#### 2F. Update foolish-ubcb-cli Cargo.toml

Remove `rayon` from `[dependencies]` (no longer needed). Keep `rayon` and `num_cpus` in `[dev-dependencies]` for tests.

### Step 3: Sequence UBC FIRs for SnapshotSuite

**Scope:** `foolish-core/src/lib.rs` (new adapter + tests), `foolish-core/snapshot_tests/` (new directory)

#### 3A. UBC evaluator adapter

The original UBC evaluates a single `FirRef` via `ubc::run_to_completion`. After evaluation, extract statements from the brane:

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

**Risk / Open Question:** The `Rc::clone(&stmt.body)` may not produce a fully independent `FirRef` suitable for `FirQueryable` formatting if the UBC evaluation leaves internal references (Rc loops, shared RefCell state). If `clone_steppable` on each statement body is needed instead, that's a one-line fix. **If this proves difficult, I will ask before proceeding.**

#### 3B. UBC approval tests

Add to `lib.rs` (or new module):
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

Two options:
1. **Convert existing inline tests to `.foo` files** - each test source becomes a file. High fidelity but labor intensive (~130 files).
2. **Start with a representative subset** - pick the most important tests (arithmetic, search, concatenation, scope, alarms) as seed files, grow incrementally.

**Recommendation:** Option 2. Start with ~20 representative files. The inline tests can coexist until migration is complete.

#### 3D. Remove inline approval tests (eventually)

The 130+ inline `#[test]` functions in `lib.rs` approval_tests module are replaced by `SnapshotSuite` tests. This is a final cleanup step - do it after `SnapshotSuite` coverage is verified equivalent.

## Dependency Flow (After)

```
foolish-core
  ├── foolish-parser
  ├── FirQueryable trait (fir.rs)
  ├── HumanizingSequencer (sequencer.rs) - single formatting path
  ├── SnapshotSuite (snapshot_suite.rs) - generic harness
  ├── ubc_evaluator (lib.rs, test module)
  └── dev-deps: insta, rayon, num_cpus, foolish-ubcb

foolish-ubcb
  └── foolish-core

foolish-ubcb-cli
  ├── foolish-core (for SnapshotSuite)
  ├── foolish-ubcb (for UbcbEngine + adapter)
  └── dev-deps: insta, rayon, num_cpus
```

## FIR Impact

None. `Fir` struct unchanged. `Steppable::format()` removed but `Steppable` trait remains for `step()` semantics.

## UBC Step Impact

None. Evaluation logic unchanged. Only formatting path changes.

## Test Plan

**Phase-gated execution - each step verified before proceeding:**

1. **After Step 1:** `cargo test -p foolish-core --workspace` - all tests pass. `foolish-cli` still produces identical output.
2. **After Step 2:** `cargo test -p foolish-ubcb-cli --lib` - UBCb tests still pass with existing approved files.
3. **After Step 3:** `cargo test -p foolish-core -- approval` - UBC tests pass (snapshots generated with `INSTA_UPDATE=always` first).
4. **Full workspace:** `cargo check --workspace && cargo test --workspace`

## Rejected Alternatives

### A. Keep both formatting paths

Maintains duplication. Any future change requires updating both. Rejected.

### B. Keep SnapshotSuite in ubcb-cli, create parallel suite for UBC

Two harnesses, two format functions, two sets of test infrastructure. Defeats the purpose. Rejected.

### C. Keep inline tests AND add SnapshotSuite tests

Doubles test count (~260 tests). Rejected as primary approach - migrate to file-driven.

### D. Make SnapshotSuite a standalone crate

Over-engineering for two consumers. A module in `foolish-core` suffices.

## Design Decisions

1. **UBC FIR extraction (Step 3A):** `Rc::clone(&stmt.body)` is sufficient. `FirChildRef` wraps `Rc<RefCell<Fir>>` and implements `FirQueryable` by borrowing — zero cloning, zero allocation. `clone_steppable` is only needed for independent mutable copies, not read-only formatting.

2. **Remove SequenceableFir entirely. Introduce FIR builders for both UBC and UBCb.**
   No separate FIR class for testing. Both UBC and UBCb get their own builder
   structs — one per FIR variant — that construct and return `Fir` values directly.
   Example: `ConstantIntFirBuilder::new(42).state(Nyes::Constant).build()`.
   Builders have their own unit tests. Snapshot tests parse `.foo` files through
   the normal compiler pipeline. The existing `*Fir` structs
   (`ConstantIntFir`, `NkFir`, `OperatorFir`, etc.) serve as the basis for
   builder APIs. UBCb gets analogous builders for `UbcbFir`.

3. **Separate input files:** UBC and UBCb maintain separate `snapshot_tests/input/` directories. They are at too different stages of development to share inputs.

4. **Inline test migration:** Copy ALL ~256 inline approval tests to `.foo` files. Leave everything failing initially — the full test corpus enters the snapshot harness, not a subset.

## Plan / Checkboxes

### Step 1: Trait-based sequencing

- [ ] Define `FirQueryable` trait in `fir.rs`
- [ ] Define `FirChildRef` wrapper for `Rc<RefCell<Fir>>` children
- [ ] Implement `FirQueryable` for `Fir`
- [ ] Rewrite `HumanizingSequencer` / `hs_format_fir` to use trait
- [ ] Update `Sequencer::format` to delegate to `HumanizingSequencer`
- [ ] Remove `format()` from `Steppable` trait and all impls
- [ ] Update callers (`foolish-cli`, inline tests)
- [ ] Verify: `cargo test --workspace` passes

### Step 1.5: FIR Builders (UBC and UBCb)

- [ ] Define `*FirBuilder` structs in `foolish-core/src/fir.rs` (one per FIR variant: `ConstantIntFirBuilder`, `NkFirBuilder`, `OperatorFirBuilder`, `SearchFirBuilder`, `IndexFirBuilder`, `HeadTailFirBuilder`, `StayFoolishFirBuilder`, `StayFullyFoolishFirBuilder`, `ConcatenationFirBuilder`, `NormalBraneFirBuilder`)
- [ ] Each builder: fluent API with `.field(value).state(Nyes).build() -> Fir`
- [ ] Unit tests for each builder (construct, verify fields, wrap in `FirRef`, format via `FirQueryable`)
- [ ] Define `UbcbFirBuilder` structs in `foolish-ubcb/src/fir.rs` (analogous, returns `UbcbFir`)
- [ ] Unit tests for UBCb builders
- [ ] Remove `SequenceableFir`, `SequenceableStatement`, `SequenceableError` from `fir.rs`
- [ ] Remove `SequenceableFir` usage from `foolish-ubcb/src/fir.rs`
- [ ] Update `HumanizingSequencer` — remove owned `SequenceableFir` variant, keep only `HumanizingSequencerRef` for `&dyn FirQueryable`
- [ ] Migrate hand-constructed sequencer tests in `lib.rs` to use builders + parse-based tests
- [ ] Verify: `cargo test --workspace` passes

### Step 2: Move SnapshotSuite to core

- [ ] Create `foolish-core/src/snapshot_suite.rs` (moved + generalized)
- [ ] Define `StatementOutput` struct in `foolish-core`
- [ ] Move formatting helpers (`format_statements`, `fmt_fir_inline`)
- [ ] Add dev-dependencies to `foolish-core/Cargo.toml`
- [ ] Provide `ubcb_evaluator` adapter in `foolish-ubcb-cli`
- [ ] Remove `snapshot_suite.rs` from `foolish-ubcb-cli`
- [ ] Verify: `cargo test -p foolish-ubcb-cli --lib` passes
- [ ] Verify: `cargo test --workspace` passes

### Step 3: Sequence UBC FIRs

- [ ] Implement `ubc_evaluator` adapter
- [ ] Create `foolish-core/snapshot_tests/input/` directory
- [ ] Create ~20 representative `.foo` input files
- [ ] Add UBC approval tests module
- [ ] Generate initial snapshots (`INSTA_UPDATE=always`)
- [ ] Verify: `cargo test -p foolish-core -- approval` passes
- [ ] (Later) Remove inline approval tests from `lib.rs`

## References

- Current `Steppable::format`: `foolish-core/src/fir.rs` lines ~658-1342 (10 impls)
- Current `HumanizingSequencer`: `foolish-core/src/sequencer.rs` lines 32-155
- Current `SnapshotSuite`: `foolish-ubcb-cli/src/snapshot_suite.rs` (334 lines)
- Current inline UBC tests: `foolish-core/src/lib.rs` lines 262-1403 (~130 tests)
- `SequenceableFir`: `foolish-core/src/fir.rs` lines 403-540
- `EvaluationResult`, `StatementResult`: `foolish-ubcb/src/engine.rs` lines 13-25

## Last Updated

**Date**: 2026-05-17
**Updated By**: opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Initial plan created. Three-step refactor: trait-based sequencing, move SnapshotSuite to core, adapt UBC FIRs for snapshot testing.
