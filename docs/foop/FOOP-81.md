---
foop: 18
title: Enhanced SnapshotSuite with HumanizingSequencer and SequenceableFir
author: Sisyphus <agent>
status: Implementing
type: Standards
created: 2026-05-15
phase: meta
supersedes: []
enhances: [FOOP-71]
---

# FOOP-81: Enhanced SnapshotSuite for UBCb with HumanizingSequencer

## Abstract

This FOOP introduces three interconnected components to modernize the Foolish
Rust test infrastructure:

1. **`SequenceableFir`** — An enum in `foolish-core` that wraps `Fir` variants
   for exhaustive pattern matching, providing a common printable profile that
   both UBC and UBCb implementations can convert their FIRs into.

2. **`HumanizingSequencer`** — A struct in `foolish-core` that takes a
   `SequenceableFir` and produces various human-readable outputs (snapshot test
   format, REPL format, etc.). This replaces the ad-hoc formatting scattered
   across `main.rs` and `lib.rs`.

3. **`SnapshotSuite`** — A dedicated module (same weight as `sequencer.rs`)
   that automatically discovers `.foo` files, validates pairing against
   approved snapshots, evaluates files through the UBCb engine in parallel
   via Rayon, and uses `HumanizingSequencer` for formatted output.

This enhances FOOP-71 (the insta crate integration remains in effect).

## Motivation

The previous FOOP-71 approach hard-coded individual test functions for each
`.foo` file and scattered FIR formatting logic across multiple files.
Adding a new test required editing `lib.rs`. Formatting logic was duplicated
between the CLI and the test suite.

The new design addresses three problems:
- **Test discovery** — Write a `.foo` file and it's automatically tested.
- **Formatting encapsulation** — `HumanizingSequencer` centralizes FIR-to-text
  conversion in `foolish-core`, usable by both UBC and UBCb.
- **Parallel execution** — UBCb evaluation is CPU-bound; parallelizing across
  threads reduces total test time as the suite grows.

## Specification

### Part 1: SequenceableFir (foolish-core)

`SequenceableFir` is an enum in `foolish-core/src/fir.rs` that wraps the `Fir`
variants to enable exhaustive pattern matching. This avoids the need to match
on runtime type strings (`get_hs_type()`).

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

pub struct SequenceableStatement {
    pub name: Option<String>,
    pub body: SequenceableFir,
}
```

**Conversion**: `impl From<Fir> for SequenceableFir` recursively converts a
`Fir` enum into a `SequenceableFir` enum. This is a one-time conversion that
detaches from `Rc<RefCell<>>` references.

**Value Resolution** (`get_hs_value`): For any `SequenceableFir`, repeatedly
follow the `target` field of `Search` variants until a constanic value is
reached or a loop is detected (error). This allows the sequencer to display
the resolved value of a search chain.

**Type Accessor** (`get_hs_type`): Return the variant identifier as a string
(e.g., `"ConstantInt"`, `"Search"`, `"NormalBrane"`).

**NYES Accessor** (`hs_get_nyes`): Return the `Nyes` state of the FIR.

**Children Accessor** (`get_hs_children`): Return a vector of child
`SequenceableFir` instances (operands, elements, statements, etc.).

**Parent Reference** (`get_hs_parent`): Returns `None` (sequenceable FIRs
are detached copies without parent links).

**Constant Value Accessors** (for `ConstantInt` variant only):
- `get_hs_int_value()` → `i64`
- Future variants: `get_hs_str_value()`, `get_hs_float_value()` (deferred)

### Part 2: HumanizingSequencer (foolish-core)

`HumanizingSequencer` is a struct in `foolish-core/src/sequencer.rs` that
takes a `SequenceableFir` and produces formatted output for different contexts.

```rust
pub struct HumanizingSequencer {
    fir: SequenceableFir,
}

impl HumanizingSequencer {
    pub fn new(fir: SequenceableFir) -> Self { Self { fir } }

    /// Format for snapshot test approval (deterministic, state-aware).
    /// The `indent` parameter specifies the column offset (in spaces) for
    /// continuation lines. Default is 0. Recursive calls into child FIRs
    /// pass an increased indent value.
    pub fn format_for_snap_test(&self, indent: usize) -> String;

    /// Format for REPL display (human-friendly, may omit internal state).
    /// The `indent` parameter controls continuation line indentation.
    pub fn format_for_repl(&self, indent: usize) -> String;

    /// Get the underlying FIR (for inspection).
    pub fn fir(&self) -> &SequenceableFir { &self.fir }
}
```

`format_for_snap_test(indent)` produces a multi-line representation suitable
for insta snapshots. Single-statement branes are one line. Branes with
multiple statements use continuation lines — each starting after `indent`
spaces. It includes NYES state tags for non-constanic FIRs.

When a `HumanizingSequencer` recursively formats a child FIR (e.g. a nested
brane inside a `Concatenation` or `Search` target), it creates a new
`HumanizingSequencer` for that child with an increased indent value (e.g.
`indent + 2`).

`format_for_repl(indent)` produces a cleaner display for interactive use.

**The existing `Sequencer` struct remains as-is** for backward compatibility.
`HumanizingSequencer` is the new, more capable sequencer.

### Part 3: SnapshotSuite (dedicated module)

`SnapshotSuite` is extracted from `foolish-ubcb-cli/src/lib.rs` into its own
module: `foolish-ubcb-cli/src/snapshot_suite.rs`. It is as important and
complex as `sequencer.rs` and deserves the same treatment.

```rust
pub struct SnapshotSuite {
    input_pattern: String,      // e.g. "inputs/(*).foo"
    golden_pattern: String,     // e.g. "golden/(*).foo.snap"
    base_dir: PathBuf,          // common ancestor of both directories
}
```

The `(*)` marker in each pattern denotes the capture group that yields the
test case name. The two patterns must use the same capture — i.e. for every
matched input file, the captured name must resolve to exactly one golden file,
and vice versa.

Example configuration:

```
base_dir = "/.../foolish/test-resources/fancy_feature_snapshot/"
input_pattern  = "inputs/(*).foo"
golden_pattern = "golden/(*).foo.snap"
```

With this configuration:
- `inputs/literals.foo` captures test name `literals`
- `golden/literals.foo.snap` captures test name `literals`
- The suite pairs them by the captured name.

### 3.1 Initialization & Validation

`SnapshotSuite::new(base_dir, input_pattern, golden_pattern)` performs:

1. **Resolve patterns** — expand the `base_dir` + each pattern into a
   `regex::Regex` where `(*)` is replaced with a named capture group `(?P<name>.+)`.
2. **Discover inputs** — walk the input directory, match files against the
   input regex, extract the captured test name.
3. **Discover goldens** — walk the golden directory, match files against the
   golden regex, extract the captured test name.
4. **Cross-reference** — compute:
   - **Paired** — test names present in both sets.
   - **Missing snapshots** — inputs with no matching golden file.
   - **Missing inputs** — golden files with no matching input.
5. **Report errors** — missing inputs are test errors (orphaned goldens).
   Missing snapshots are queued as test errors but the input is still
   evaluated so that a `.snap.new` file can be generated.
6. Return `Ok(Self)` with the paired set, the missing-snapshot set, and the
   missing-input error list.

### 3.2 Methods

| Method | Purpose |
|---|---|
| `discover(&self) -> Vec<(String, PathBuf)>` | Return sorted list of (test_name, input_path) pairs |
| `evaluate(&self, path: &Path, with_states: bool) -> Result<String, String>` | Evaluate a `.foo` file via UBCb, return formatted output using `HumanizingSequencer` |
| `evaluate_all(&self, threads: usize, with_states: bool) -> Vec<(String, Result<String, String>)>` | Parallel evaluation with Rayon |
| `get_missing_snapshots(&self) -> Vec<String>` | Test names that lack a golden file |
| `get_missing_inputs(&self) -> Vec<String>` | Test names whose golden file exists but input is gone |

### 3.3 Parallel Execution

Each `.foo` file evaluation is independent (each uses a fresh `UbcbEngine`
instance). The suite uses Rayon's `par_iter` to distribute evaluations across
a configurable thread pool:

```rust
pub fn evaluate_all(&self, threads: usize, with_states: bool) -> Vec<...> {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads.max(1))
        .build()
        .expect("Failed to create Rayon thread pool");
    pool.install(|| { /* par_iter over discovered files */ })
}
```

**Default:** `threads = num_cpus::get()` for maximum parallelism.
**Override:** Pass `1` for sequential execution (useful for debugging).

**Result aggregation:** All failures are collected and reported together
(Fail-At-End), not short-circuiting on the first failure.

### 3.4 Golden File Naming

The golden file path is derived from the test name by substituting it into
the golden pattern:

- Test name `literals` → `golden/literals.foo.snap` (normal)
- Test name `literals` → `golden/literals.foo.snap_states` (states variant)
- New/updated snapshots produce `.foo.snap.new` alongside the golden.
- `cargo insta review` / `cargo insta accept` manage approval lifecycle.

### 3.5 Test Module Usage

```rust
#[cfg(test)]
mod approval_tests {
    use super::*;

    fn suite() -> SnapshotSuite {
        SnapshotSuite::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("approval_test_resources"),
            "inputs/(*).foo",
            "golden/(*).foo.snap",
        ).expect("SnapshotSuite initialization failed")
    }

    #[test]
    fn approval_all()          { /* evaluate and assert snapshots */ }

    #[test]
    fn approval_all_states()   { /* evaluate with states and assert snapshots */ }
}
```

### 3.6 Behavior Matrix

| Situation | Result |
|---|---|
| Input has golden, matches | Pass |
| Input has golden, mismatches | Fail (diff reported; `.snap.new` written) |
| Input has no golden | Evaluated; `.snap.new` generated; queued as test error |
| Golden has no input | Test error (orphaned golden reported) |
| `.foo.disabled` file | Ignored |

### 4. Dependencies

- `rayon` — parallel iteration (new dev-dependency)
- `num_cpus` — default thread count (new dev-dependency)
- `insta` — snapshot comparison (existing)

## FIR Impact

`SequenceableFir` adds a new enum to `foolish-core/src/fir.rs` (~100 lines)
with `From<Fir>` conversion. This is additive — no existing FIR behavior
changes.

`HumanizingSequencer` adds a new struct to `foolish-core/src/sequencer.rs`.
The existing `Sequencer` is preserved for backward compatibility.

## UBC Step Impact

None. Evaluation semantics are unchanged.

## Deferred work

- `SequenceableFir` variants for String, Float constants (after Int)
- `HumanizingSequencer::format_for_repl()` (snap_test format is priority)
- Cross-validation snapshot testing between UBC and UBCb (separate FOOP)
- UBC snapshot inspection — review 194 existing `foolish-core` snapshots (separate FOOP)

## Rejected Alternatives

### A. SequenceableFir as a trait

A trait (`trait SequenceableFir { fn get_hs_type()... }`) would require
matching on `get_hs_type()` strings at runtime, defeating exhaustive
pattern matching. The enum approach gives compile-time exhaustiveness.

### B. Single test function that iterates sequentially

This would be simpler but wouldn't parallelize. The suite will grow and
parallelism is worth the small added complexity of Rayon.

### C. Build script to generate test functions

A `build.rs` approach would give independent test functions but adds build
time complexity and makes the test discovery implicit rather than explicit.

### D. Keep formatting in lib.rs / main.rs

The formatting logic is complex and shared between CLI and tests.
Centralizing in `HumanizingSequencer` eliminates duplication and makes
the formatting testable with unit tests.

## References

- [FOOP-71](FOOP-71.md) — Previous (reverted) snapshot testing approach
- [insta.rs](https://insta.rs/) — Official insta documentation
- [Rayon](https://docs.rs/rayon/) — Data parallelism library
