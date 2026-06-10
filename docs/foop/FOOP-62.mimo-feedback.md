# FOOP-62 Feedback: UBCa — Two-Store ProtoBrane Tree and Uniform Two-Phase Stepping

**Reviewer**: Sisyphus (mimo-v2.5-pro)
**Date**: 2026-06-10
**Status**: Initial review — read-only, no code changes

---

## Executive Summary

The two-store ProtoBrane design is the right abstraction. The `bon` builders are the right enforcement mechanism. The UBC-as-oracle strategy is correct for a representation change this large. But three things need explicit resolution before implementation begins:

1. How kind-specific leaf data structurally attaches to ProtoBrane.
2. How the task-list stepping model reproduces UBC's batched step counts.
3. How the compiler wires parents at construction time.

Without these, the implementation will hit walls at the first snapshot comparison.

---

## What's Strong

### Two-store split

The `foolish_children` / `ubc_children` separation makes structural what AGENTS.md already states semantically: `foolish_children` is the fixed Foolish meaning; `ubc_children` is the evolving evaluation record. This eliminates:

- The ad-hoc `set_brane_statement_at` mutation path (`fir.rs:740`).
- The pervasive `clone_into_fir()` calls used purely to pattern-match `dyn Steppable` (e.g. `has_unresolved_forward_refs` clones 4× in one function — `ubc.rs:191–225`).
- The `ChildrenItr` enum with its 6 arms including the `unsafe` pointer-projection arm (`fir.rs:567`).

The invariant that `foolish_children` has no public mutator is enforceable by the type system. This is correct Rust.

### NYES-driven task list

The `VecDeque<FirRef>` task list is a clean abstraction. "One action per `step()` call" (pop a constanic front, or step an unfinished front) eliminates the ad-hoc stepping logic in `re_step_brane_bodies` (`ubc.rs:262–309`) and the `run_to_completion_with_scope` fixpoint loop (`ubc.rs:159–183`). The `debug_assert` at queue emptiness (§3.2) is a good terminal invariant.

### UBC-as-oracle

Building UBCa as a new crate that clones the UBC public interface and tests, then guts the internals, is the correct strategy for a representation change. The finalized `.snap` corpus is the byte-exact correctness reference. Any behavioral divergence is immediately visible.

### `bon` builders

Making `parent` a required builder input (not a defaulted field) enforces "a node without a parent" unrepresentable at compile time. The `#[builder(on(_, overwritable))]` + hand-written `updater(self)` pattern for constanic clone is well-designed. The privacy + `#[non_exhaustive]` enforcement of "only the builder creates Firs" is correct.

### Parent link design

The `Weak<RefCell<Fir>>` parent with self-referential root via `Rc::new_cyclic` is the standard Rust pattern for parent-linked graphs with trait objects. Making parent immutable-after-construction and expressing re-parenting as `clone_with_parent` is clean.

---

## What Will Not Work As Written

### 1. Step counts will not match UBC

**The biggest risk.** The current UBC uses `step_boxed` (`ubc.rs:250–254`) — clone a statement body, run it to completion via `run_to_completion_with_scope` (an inner loop with `max_steps`), then move to the next statement. This is **batched**: each statement body runs to constanic before the next starts.

The task-list model in §3.2 steps *one* child *one* transition per `step()` call, **interleaving** across children.

These produce different step counts. The spec says "the per-input step count recorded in snapshots must match UBC" (§3.3) — but note: the current snapshot format does NOT include step counts. Snapshots contain only the formatted FIR output (`[i] RESULT:` blocks + signature footer — see `snapshot_suite.rs:136–171`). Step counts are tracked by `FirSequencer.steps` but only appear in `format_with_header` (debug/REPL output), not in snapshots.

So the step count concern is about **spec compliance** (the spec says they must match), not about snapshot byte-exactness. The actual acceptance criterion is byte-exact **sequencer output** (§8). If the stepping model produces the same final state, the snapshots will match — even if step counts differ.

However, the stepping model could affect the final state if children share mutable state via `Rc<RefCell<..>>`. In the current UBC, `step_boxed` clones each statement body before stepping (`ubc.rs:251`), so each is independent. In the task-list model, tasks are `Rc<RefCell<Fir>>` — if two tasks share the same underlying Fir via Rc, stepping one affects the other. The spec should clarify whether tasks are fresh clones or shared references.

**The fix** is either:
- (a) The outer loop replicates UBC's batching (step each child to completion before moving on), which means the task list is not truly interleaved — it's a sequential queue processed to completion per item. This preserves step counts but loses the elegance of the interleaved model.
- (b) The step counts are accepted as different (they're not in snapshots anyway) and the focus is on byte-exact sequencer output. This is the pragmatic path.
- (c) Each task is a fresh clone (like `step_boxed`), ensuring independence. This preserves correctness but adds cloning overhead.

This must be resolved before implementation. The task-list model is clean, but it is not step-count-compatible with UBC's batched model — and if tasks share state, it may not be output-compatible either.

### 2. Kind-specific leaf data has no structural home

§2 shows a table mapping each kind to its leaf data (Operator → `op name`, Search → `pattern/direction/anchored`, etc.). But §1's `ProtoBrane` struct has **no payload field** — only `foolish_children`, `ubc_children`, `nyes`, `tasks`, `parent`.

The implied design is that each kind is a struct *containing* a ProtoBrane:

```rust
struct OperatorFir {
    protobrane: ProtoBrane,
    op: String,
}

struct SearchFir {
    protobrane: ProtoBrane,
    pattern: String,
    direction: SearchDirection,
    anchored: bool,
}
```

And `FirRef = Rc<RefCell<dyn Fir>>` where each kind implements `Fir` by delegating `step` to `protobrane_step(&mut self.protobrane, scope)` and implementing `fir_op_step` with kind-specific logic.

But this is never stated. The spec says "every node is a ProtoBrane" (§2) and "`Fir` trait" (§3.2) without specifying whether `Fir` **IS** ProtoBrane or whether kinds **contain** a ProtoBrane. This is a fundamental architectural ambiguity that affects every line of code.

**Recommendation**: Explicitly state the composition model. The "kinds contain a ProtoBrane" model is the correct one — it allows kind-specific leaf data while sharing topology and stepping via the ProtoBrane. The `Fir` trait should be:

```rust
pub trait Fir: std::fmt::Debug {
    fn protobrane(&self) -> &ProtoBrane;
    fn protobrane_mut(&mut self) -> &mut ProtoBrane;
    fn fir_op_step(&mut self, scope: &Scope) -> Result<(), UbcError>;
    fn kind(&self) -> FirKind;
    // ... narrow leaf-data accessors per kind ...
}
```

And the default `step` implementation delegates to `protobrane_step(self.protobrane_mut(), scope)`.

### 3. Sequencer re-expression is severely underestimated

The spec says (§8): "retiring the trait is a *goal*, byte-exact output is the *constraint*." But the sequencer (`sequencer.rs`, 641 lines) has ~200 lines of variant-specific rendering logic in `render_fir` (line 279 onward). Each variant has different formatting rules:

| Kind | Format | Notes |
|------|--------|-------|
| Operator | `Op+(operands..., STATE)` | Parenthesized, state after all operands |
| Search | `?(pattern='...', UNANCHORED, STATE, result=...)` | Specific field order, `result=` from `ubc_children` |
| Brane | `{statements...; STATE}` | Semicolons, characterization prefix |
| SF | `<STATE\n body\n>` | State in opener, marker wrapping |
| SFF | `<<STATE\n body\n>>` | State in opener, marker wrapping |
| Concatenation | `⨃{elements...}` | Unicode prefix, merged result handling |
| Index | `#(offset=N, UNANCHORED, STATE, result=...)` | Specific field order |
| HeadTail | `^(STATE, result=...)` / `$(STATE, result=...)` | Direction-dependent prefix |

The `proto_brane_formatter_with_result` function (line 180) handles the `result=` labeling from `ubc_children` — this maps cleanly to the two-store model. But the variant-specific opener/closer/separator logic must be reimplemented per kind.

Re-expressing this over `kind()` + `foolish_children_itr` + `ubc_children_itr` + leaf accessors is not a simple port. A thin `FirQueryable` shim over ProtoBrane is the safer path, as the spec itself acknowledges ("If the `FirQueryable` retirement proves to make byte-exact output materially harder than keeping a thin query shim, the fallback is to keep a minimal query adapter rather than risk the output" — §8).

**Recommendation**: Keep a thin `FirQueryable` adapter as the first-pass approach. Retire it only after the snapshot corpus is green. The adapter reads from ProtoBrane's two stores + leaf accessors and returns the same tuples the sequencer expects. This is ~100 lines of glue code vs. ~400 lines of sequencer rewrite.

### 4. `ParentPtr` → `Weak<RefCell<Fir>>` with immutable-after-construction breaks the compiler

The current compiler (`compiler.rs`, 298 lines) creates FIR nodes with `parent: ParentPtr::new()` (empty) and relies on a later pass to wire parents. FOOP-62 §5 makes parent immutable-after-construction, requiring the builder to take it up front.

But the compiler builds the AST **bottom-up** — it doesn't know the parent until the containing brane is constructed. For example, in `compile_astn` (`compiler.rs:28`), an `Astn::Identifier` becomes a `SearchFir` with `parent: ParentPtr::new()`. The parent (the containing brane) doesn't exist yet — it's being built from the outer `Astn::Brane` match arm.

The compiler would need to be restructured to pass parent context **downward** during compilation:

```rust
fn compile_astn(ast: Astn, parent: Weak<RefCell<Fir>>) -> anyhow::Result<Fir> {
    match ast {
        Astn::Identifier { id, .. } => Ok(Fir::Search(Box::new(SearchFir {
            pattern: format!("^{}$", id),
            parent,  // now provided by caller
            ...
        }))),
        Astn::Brane { characterizations, statements } => {
            // Build the brane first, get its Weak self-ref
            let brane_weak = ...; // Rc::new_cyclic
            let mut stmt_firs = Vec::new();
            for stmt in statements {
                stmt_firs.push(Self::compile_stmt(stmt, brane_weak.clone())?);
            }
            ...
        }
    }
}
```

This is a significant refactor of `compiler.rs`. It's doable but not trivial — the compiler must construct the brane's `Rc` via `Rc::new_cyclic` to get the self-`Weak`, then pass that `Weak` to each statement's compilation. The order of operations changes from "build children, then build parent" to "build parent shell, then build children with parent reference, then fill in parent."

**Recommendation**: Plan this as a dedicated sub-task. The compiler refactor is the highest-risk part of the implementation because it touches every code path that creates FIR nodes.

### 5. The `Evaluator` trait uses `FirRef = Rc<RefCell<dyn Steppable>>`

The snapshot suite (`snapshot_suite.rs:9`) defines:

```rust
pub trait Evaluator {
    fn evaluate(&self, source: &str) -> Result<Vec<FirRef>, String>;
}
```

where `FirRef` is `Rc<RefCell<dyn Steppable>>`. UBCa's FIR type would be `Rc<RefCell<dyn Fir>>` (the new trait). These are incompatible types.

The suite must be genericized:

```rust
pub trait Evaluator<F> {
    fn evaluate(&self, source: &str) -> Result<Vec<F>, String>;
}
```

Or UBCa must implement `Steppable` (which defeats the purpose). Or the suite accepts a trait object that can format itself (e.g. `Box<dyn FirQueryable>`).

**Recommendation**: Genericize the `Evaluator` trait over the FIR reference type, or have UBCa implement a thin `Steppable` adapter that delegates to its own `Fir` trait. The adapter is ~50 lines.

---

## What Needs Clarification

### `EvalContext` (SF/SFF markers)

Added in FOOP-52, lives on `Scope` as `eval_context: EvalContext` (`ubc.rs:46`). Not mentioned in FOOP-62. The `EvalContext` enum (`fir.rs:196–213`) controls whether searches go to ECONSTANIC (in SFF context) or proceed normally. It must survive into UBCa.

Since `EvalContext` is on `Scope` (not on the FIR), it should carry over unchanged. But the spec should mention it explicitly to avoid confusion during implementation.

### `foolish_children` stepping model

Current code replaces statement bodies during stepping (`set_brane_statement_at` at `ubc.rs:292–294`). ProtoBrane model mutates in place via `RefCell::borrow_mut().step()`. These produce different intermediate states:

- **Current**: A new `StatementFir` is created with the stepped body, replacing the old one at the same index.
- **ProtoBrane**: The existing `StatementFir`'s body is stepped in place via `RefCell`.

The step counts and intermediate states may differ. The current model creates fresh `Rc` wrappers at each replacement; the ProtoBrane model reuses the same `Rc`. This affects any code that compares `Rc::ptr_eq` or tracks identity.

### Scope threading with interleaved stepping

The spec mentions `Scope` continues to thread `current_brane` / `current_stmt_idx` (§5). But in the task-list model, all children share the parent's scope. In the current UBC, each statement body gets its own scope clone with `current_brane` and `current_stmt_idx` set to the statement's position (`ubc.rs:278`).

If the task-list model interleaves stepping across children, the scope's `current_stmt_idx` must be correct for each child being stepped. This means the scope must be per-child, not per-parent. The spec should clarify whether each task carries its own scope or whether the parent provides it.

### `has_unresolved_forward_refs`

This function (`ubc.rs:187–237`) walks the FIR tree checking for ECONSTANIC descendants. It uses `clone_into_fir()` to pattern-match. In the ProtoBrane model, it would walk `foolish_children` + `ubc_children`. The replacement should be specified — it's used by `run_to_completion_with_scope` to decide whether to break the stepping loop (`ubc.rs:179`).

### Serialization

The `Fir` enum derives `Serialize`/`Deserialize`. The ProtoBrane model changes the struct layout. The snapshot format must remain byte-identical. The current `ParentPtr` serializes as `none` (parents are never serialized). The ProtoBrane model's `Weak<RefCell<Fir>>` parent should serialize the same way. But the spec should confirm this explicitly.

---

## Open Questions from the Spec

### Finite-time-to-constanic guarantee

§3 relies on the language guaranteeing every Fir reaches a constanic NYES in finite time. The spec says "locate where this guarantee is documented" — this should be resolved before implementation. If it's only implicit, the FOOP should state it explicitly. The current `max_steps` guard (`ubc.rs:160`) is belt-and-suspenders; the semantic guarantee must be proven separately.

### NYES transitions

§3.3 says "the first pass mirrors UBC's transitions" and "the exact per-kind progression is deferred to 'match UBC'." This is fine for the first pass, but the transition table should be documented somewhere (even as a reference to the UBC code) so implementers have a concrete target.

### `bon` version

The spec says `bon = "3"` is approved. The latest at writing is `3.9.1`. The MSRV must be checked — `bon` 3.x requires Rust 1.65+. The workspace is on edition 2024 (Rust 1.85+), so this is fine. But pin a specific `3.x` version in `Cargo.toml` to avoid surprise breakage.

---

## Implementation Order Recommendation

1. **Resolve the three blocking ambiguities** (leaf data attachment, step count model, compiler parent wiring). These must be decided before any code is written.

2. **Create `foolish-ubca` crate skeleton** with `Cargo.toml`, `lib.rs`, and the `ProtoBrane` struct. Copy the finalized `.snap` corpus from `foolish-core`.

3. **Implement `ProtoBrane` + `Fir` trait** with the composition model (kinds contain ProtoBrane). Implement `protobrane_step` as the default `step`.

4. **Implement the compiler** with parent-wiring-at-construction. This is the highest-risk refactor.

5. **Implement kind-specific `fir_op_step`** for each of the ~10 kinds. Start with ConstantInt and Nk (leaves), then Operator, then Search, then Brane.

6. **Implement the sequencer adapter** (thin `FirQueryable` shim over ProtoBrane). Verify byte-exact output against the snapshot corpus.

7. **Run the oracle cross-check** — every `.foo` through both UBC and UBCa, assert identical output and step counts.

8. **Iterate** until the snapshot corpus is green.

---

## Summary

| Concern | Severity | Status |
|---------|----------|--------|
| Step count mismatch (batched vs. interleaved) | **Blocking** | Needs design decision before implementation |
| Kind-specific leaf data attachment | **Blocking** | Needs explicit struct specification |
| Compiler parent wiring | **High** | Significant refactor, plan as sub-task |
| Sequencer re-expression | **Medium** | Thin adapter is the safe first pass |
| `Evaluator` trait type mismatch | **Medium** | Genericize or adapter |
| `EvalContext` not mentioned | **Low** | Carries over from Scope, just needs documentation |
| `Rc::new_cyclic` for root | **Low** | Standard pattern, just needs care |
| `bon` dependency | **Low** | Approved, just pin version |

The design is sound. The execution risks are real but manageable. Resolve the three blocking concerns, then implement.
