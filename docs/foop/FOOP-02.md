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
dispatches via trait accessor methods:

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

Keep `HumanizingSequencer` struct for `SequenceableFir` (preserves existing API
for hand-constructed test FIRs). Add `HumanizingSequencerRef` for `&dyn FirQueryable`:

```
pub struct HumanizingSequencerRef<'a> { fir: &'a dyn FirQueryable }
impl<'a> HumanizingSequencerRef<'a> {
    pub fn new(fir: &'a dyn FirQueryable) -> Self { ... }
    pub fn format_with_indent(&self, indent: usize) -> String { ... }
}
```

Both call `hs_format_fir`.

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

### Step 2: Move SnapshotSuite to foolish-core

**Scope:** `foolish-core/src/snapshot_suite.rs` (new), `foolish-ubcb-cli/src/snapshot_suite.rs` (deleted),
`foolish-ubcb-cli/src/lib.rs` (updated), `foolish-core/Cargo.toml`, `foolish-ubcb-cli/Cargo.toml`

#### 2A. Move and generalize `SnapshotSuite`

The `SnapshotSuite` struct (discovery, input/approved directory management) is
engine-agnostic. Move to `foolish-core/src/snapshot_suite.rs`.

#### 2B. Replace hardcoded `UbcbEngine` with evaluator function

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

#### 2C. Move formatting helpers to core

`format_result`, `fmt_stmt`, `fmt_fir_inline` move to `foolish-core`.
After Step 1, `fmt_fir_inline` uses `HumanizingSequencerRef`:

```
fn fmt_fir_inline(fir: &FirRef, indent: usize, states: bool) -> String {
    let wrapper = FirChildRef { inner: Rc::clone(fir) };
    let output = HumanizingSequencerRef::new(&wrapper).format_with_indent(indent);
    if states {
        format!("{} [{}]", output, fir.borrow().state())
    } else {
        output
    }
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

## Open Questions

1. **UBC FIR extraction (Step 3A):** Does `Rc::clone(&stmt.body)` produce a valid
   `FirRef` for `FirQueryable` formatting, or is `clone_steppable` needed?
   **If cloning breaks references, ask human before proceeding.**

2. **SequenceableFir retention:** Keep `SequenceableFir` and `SequenceableStatement`
   for hand-constructed tests in the sequencer_tests module? Yes -- they avoid
   parser/compiler overhead.

3. **Shared vs separate input files:** Should UBC and UBCb share `.foo` inputs
   (cross-validation) or maintain separate directories? Separate initially, merge
   later once outputs match.

4. **Inline test migration:** How many `.foo` files initially? Recommendation:
   ~20 representative files for seed, grow incrementally.

## References

- `Steppable::format` implementations: `foolish-core/src/fir.rs` lines ~658-1342 (10 impls)
- `HumanizingSequencer`: `foolish-core/src/sequencer.rs` lines 32-155
- `SnapshotSuite`: `foolish-ubcb-cli/src/snapshot_suite.rs` (334 lines)
- Inline UBC tests: `foolish-core/src/lib.rs` lines 262-1403 (~130 tests)
- `SequenceableFir`: `foolish-core/src/fir.rs` lines 403-540
- `EvaluationResult`, `StatementResult`: `foolish-ubcb/src/engine.rs` lines 13-25
