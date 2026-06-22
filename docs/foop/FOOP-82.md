---
foop: 28
title: UBCa Code Review — Findings and Recommended Changes
author: Sisyphus <agent>
status: Draft
type: Review
created: 2026-06-23
phase: phase-2
supersedes: []
---

# FOOP-82: UBCa Code Review — Findings and Recommended Changes

FOOP numbering is little-endian; the full rules live in `foop.md` at the
repository root — **read it before creating or editing a FOOP.** This FOOP
records the outcome of a focused code review of the `foolish-ubca` crate (the
FOOP-62 worktree) and proposes concrete changes. It is a review document, not
an implementation plan; the changes themselves will be sequenced in a later
`.plan.md` (or folded into FOOP-62's plan).

## Abstract

A line-by-line review of `foolish/foolish-ubca/src/` (8 files, ~5,800 lines)
found five classes of issue: (1) **correctness bugs** in the HeadTail/Index
settle path, the operator's handling of non-integer operands, and several
`unreachable!`/silent-failure sites; (2) **architectural divergences** from
FOOP-62 (no `bon` builders, an interior-mutability pivot not reflected in the
spec, a 400-line output bridge that reintroduces the `clone_into_fir` pattern
the spec's Motivation rejects); (3) **convention violations** against
`rust_instructions.md` (`#[allow]` vs `#[expect]`, `unsafe_code = "warn"`,
`eprintln!` alarms, missing `#[non_exhaustive]`, a placeholder `FirKind::Unknown`
variant); (4) **performance issues** (regex compiled per statement match, O(N)
statement-index scans, Vec cloning on every `ubc_children()` read); (5) **test
gaps** (a fake `BraneFir` in `fir_trait::tests`, a non-asserting
`*_nyes_transitions` case, committed debug scaffolding). None of the
correctness bugs are catastrophic — the crate's 119 unit tests pass and most
failures are masked by constant-valued test trees — but several produce
silently wrong output on non-constant inputs. The recommendations are
prioritized; the top five are small, surgical fixes that should land before
merge.

## Motivation

UBCa (FOOP-62) is a new sibling crate that reimplements the Foolish VM on a
two-store `ProtoBrane` tree with a queue-driven NYES state machine. It is
intended to be **its own source of truth** (Atlas ruling, 2026-06-19) —
validated byte-for-byte against its own approved snapshots, not against UBC.
That makes internal correctness and conformance to the project's Rust standards
(`rust_instructions.md`) the primary quality gates, since there is no external
oracle to catch divergences.

This review was scoped to UBCa only (`foolish-ubca/src/`), performed without
running tests, by reading every source file in full and tracing the algorithms
(borrow discipline, search/clone/settle logic, indexing, concatenation merge)
line by line. The findings below are the actionable output. They are grouped,
cross-referenced where related, and prioritized. Several are directly
traceable to incomplete items in `FOOP-62.plan.md` (Phase −1, Notes); this FOOP
makes those gaps explicit and adds the ones the plan does not yet record.

### The world today

- UBCa compiles, has 119 passing unit tests, and good NYES-progression coverage
  per FIR kind.
- The spec (FOOP-62 §6) mandates `bon` builders; the code has none (hand-written
  free functions, `pub(crate)` fields).
- The spec (§3.1/§3.2) describes `fir_op_step(&mut self)` with transient
  borrows; the code uses `Cell`/`RefCell` interior mutability and
  `fir_op_step(&self)` — a defensible but **undocumented** pivot.
- The spec (§Phase 3a) mandates a thin `FirQueryable` adapter for the
  sequencer; the code has a 400-line recursive `proto_to_core_fir` bridge that
  rebuilds a `foolish_core::Fir` tree for rendering.
- HeadTailFir was missed by the plan #23 fix that gave SearchFir/IndexFir a
  `settle_from_ubc_result` drain pattern — it still settles prematurely.

### The world after

- HeadTail/Index/Search share one settle-and-cap pattern; no premature
  settling, consistent `search_nyes_from_found` capping.
- Silent failure modes (MAX_STEPS, non-integer operands, `unreachable!`) are
  converted to errors or alarms.
- `constanic_clone_at` is a generic helper or `bon` `updater` chain, not 200
  lines of per-kind copy-paste.
- `ubc_children()` returns a borrowed slice view (zero-alloc); regexes are
  cached; statement-position bounds come from `line_number`, not `ptr_eq`
  scans.
- The spec and code agree on the mutability model (whichever direction is
  chosen); the output path is a documented adapter or the bridge is explicitly
  scoped as interim.

## Summary of findings

Findings are labeled **C** (correctness), **A** (architecture/spec-divergence),
**V** (convention violation), **P** (performance), **T** (test quality).
Relationships are noted inline ("related: …").

| ID | Severity | One-liner |
|----|----------|-----------|
| C1 | 🔴 P0 | HeadTailFir does not drain its cloned result — settles prematurely |
| C2 | 🔴 P0 | Index/HeadTail skip `search_nyes_from_found` capping |
| C3 | 🟠 P1 | SF-wrapped unanchored search re-checks only the immediate brane |
| C4 | 🟠 P1 | Concatenation shares statement FIRs by `Rc::clone` (not constanic-clone) |
| C5 | 🔴 P0 | Operator silently eternal-Woconstanic on non-integer operands |
| C6 | 🔴 P0 | `MAX_STEPS` exhaustion returns `Ok(())` — partial tree rendered as final |
| C7 | 🔴 P0 | `unreachable!` in `constanic_clone_at` SF/SFF arm (reachable via empty child) |
| C8 | 🔴 P0 | `unreachable!` in compiler on parser output |
| C9 | 🟠 P1 | `get_value` and SF re-search have no depth guard (stack-overflow risk) |
| A1 | 🔴 P0 | `bon` builders not used — spec §6 unimplemented |
| A2 | 🟠 P1 | Interior-mutability pivot not reflected in spec §3 |
| A3 | 🟠 P1 | `proto_to_core_fir` 400-line bridge reintroduces `clone_into_fir` pattern |
| A4 | 🟡 P2 | Bridge recomputes brane NYES from converted children (layering violation) |
| V1 | 🟡 P2 | `#[allow(dead_code)]` instead of `#[expect(...)]` |
| V2 | 🟡 P2 | `unsafe_code = "warn"`; no clippy correctness gate |
| V3 | 🟡 P2 | `eprintln!` for alarms/logging |
| V4 | 🟡 P2 | `as i64` cast in compiler without `TryFrom` |
| V5 | 🟡 P2 | `FirKind::Unknown` placeholder variant in production enum |
| V6 | 🟡 P2 | Missing `#[non_exhaustive]` on public enums/structs |
| V7 | 🟡 P2 | `UbcError` is single-variant stringly-typed |
| P1 | 🟠 P1 | `Regex::new` compiled per statement match |
| P2 | 🟠 P1 | `find_stmt_index_in_brane` is O(N) per search |
| P3 | 🟠 P1 | `ubc_children()`/`all_children()` clone the Vec every read |
| P4 | 🟡 P2 | Concatenation walks children 3× |
| T1 | 🟡 P2 | `search_anchored_found_nyes_transitions` asserts no terminal state |
| T2 | 🟡 P2 | `sff_struct_probe` — committed debug scaffolding (no assertions, unformatted) |
| T3 | 🟡 P2 | Duplicate test `BraneFir`/`LeafFir` in `fir_trait::tests` test a fake, not the real kind |
| T4 | 🟡 P2 | `make_root_brane` leaves children self-rooted (acknowledged broken) |

## Recommendations

Listed in priority order. "Related" markers show which findings move together.

### R1 — Finish the plan #23 settle pattern for HeadTail (fixes C1, C2)

HeadTailFir was missed when plan #23 gave SearchFir and IndexFir the
`push_search_result → stay Braning → settle_from_ubc_result` drain pattern.
Give HeadTail the same shape, and route Index's *and* HeadTail's settle through
`search_nyes_from_found()` so a found ECONSTANIC/WOCONSTANIC body caps to
WOCONSTANIC and a found CONSTANT/INDEPENDENT caps to CONSTANT (never
INDEPENDENT — a search is context-dependent). This makes all three
search-classified kinds (Search/Index/HeadTail) share one settle-and-cap rule.
**Related**: C1, C2. Small, surgical.

### R2 — Convert silent-failure modes to errors/alarms (fixes C5, C6, C7, C8)

Four sites currently swallow failure or panic:

- **C5**: operator with non-integer operands → eternal Woconstanic. Decide the
  language rule (FOOP-62 does not define brane arithmetic) and emit NK with a
  reason, or return a typed `UbcError` variant. Do not silently stall.
- **C6**: `MAX_STEPS` (10,000) exhaustion returns `Ok(())`. Return
  `UbcError::DidNotSettle` (or emit an `Alarm`) so a stuck evaluation is visible,
  not rendered as final output.
- **C7**: `unreachable!("SF/SFF resolved to source at fn top")` in
  `constanic_clone_at` is reachable when an SF/SFF has no children (the top guard
  falls through). Replace with a graceful NkFir fallback.
- **C8**: `unreachable!` in `build_fir` for `Assignment`/unknown arms. The
  compiler is the parser→FIR boundary; a parser bug or new `Astn` variant should
  return `anyhow::Error`, not panic.

**Related**: C5, C6, C7, C8 are independent bugs but the same fix shape (replace
silent/panic with typed failure). Small each.

### R3 — Resolve the `bon` question (fixes A1, and unblocks D1 below)

Either implement spec §6 (add `bon` dependency, privatize fields,
`#[non_exhaustive]`, builder-only construction enforced by visibility) **or**
amend FOOP-62 §6 to drop the `bon` requirement with Atlas's sign-off. The
current state — `bon` in `[workspace.dependencies]`, human approval recorded in
the plan, but no `bon` in `foolish-ubca/Cargo.toml` and hand-written builders —
is the largest spec gap and leaves "create a Fir without a builder" fully
representable. This is the decision that unblocks D1 (constanic-clone helper)
and V5/V6 (encapsulation hygiene). **Related**: A1, D1, V5, V6.

### R4 — Cache regexes and borrow `ubc_children` slices (fixes P1, P3)

Two changes with large eval-speedup for real (non-tiny) programs:

- **P1**: `matches_pattern` compiles `Regex::new(pattern)` for every
  statement-match check. Cache the compiled regex on `SearchFir` via
  `OnceLock<Option<Regex>>`, and short-circuit the common `^identifier$` case
  with `extract_simple_name(pattern) == stmt_name` before touching regex.
- **P3**: `ubc_children() -> Vec<FirRef>` clones the whole Vec on every read
  (hot path: `as_i64`, `get_value`, the bridge). Return
  `Ref<'_, [FirRef]>` via `Ref::map(self.ubc_children.borrow(), |v| v.as_slice())`
  — zero-alloc, borrow visible in the type.

**Related**: P1 and P3 are independent but both are "hot-path allocation"
fixes. P3 also makes the RefCell-borrow discipline visible (see A2/R6).

### R5 — Collapse `constanic_clone_at` repetition (fixes D1; depends on R3)

~200 lines of per-kind clone code where every arm is structurally identical
(`new_cyclic` → map foolish_children → loop ubc_children → `clone_nyes` →
build struct). Adding a FIR kind means copy-pasting another ~20-line arm. If
R3 adopts `bon`, the `updater().parent(..).nyes(..).build()` chain makes each
kind's clone ~3 lines. If R3 drops `bon`, extract a generic helper + a
`clone_leaf_data` trait method. Either way, the per-kind arm shrinks to its
leaf-data extraction. **Related**: A1/R3, D1.

### R6 — Reconcile spec §3 with the interior-mutability pivot (fixes A2, A4)

The spec describes `fir_op_step(&mut self)` + transient `borrow_mut` discipline;
the code uses `Cell`/`RefCell` + `fir_op_step(&self)`. The implementation is
defensible (shared borrows coexist, sidestepping aliasing panics), but the spec
has not been updated. Two actions:

- Update FOOP-62 §3.1/§3.2 to describe the interior-mutability model actually in
  use, including the **safety invariant** that `ubc_children()` returns a clone
  precisely so no `borrow_mut()` overlaps a `borrow()` (R4's `Ref<'_, [FirRef]>`
  makes this invariant compiler-checked).
- **Stop recomputing brane NYES in the bridge** (`evaluator.rs:236-249`). If
  UBCa's NYES is wrong, fix the evaluator; if right, trust it. The recompute is
  a layering violation that masks evaluator bugs and contradicts "UBCa is its
  own source of truth."

**Related**: A2, A4, P3/R4.

### R7 — Convention cleanup (fixes V1, V2, V3, V4, V5, V6, V7)

Small, mechanical, mostly independent:

- V1: `#[allow(dead_code)]` → `#[expect(dead_code, reason = "…")]` on
  `ProtoBrane::push_task`.
- V2: `unsafe_code = "deny"` in `[lints.rust]`; add
  `[lints.clippy] correctness = "deny"`.
- V3: `eprintln!` alarms → `tracing::warn!` or the core `Alarm` types
  (`fir_kinds.rs:163`, `ubca_snapshot_tester.rs:31,37`). (Plan Notes already
  flag this.)
- V4: `n as i64` in the compiler → `i64::try_from(n)` or remove if the parser
  type is already `i64`.
- V5: remove `FirKind::Unknown` from the production enum; confine stubs to
  `#[cfg(test)]`.
- V6: `#[non_exhaustive]` on `FirKind`, `StepReport`, `UbcError`, `Scope`.
- V7: either make `UbcError` `anyhow`-opaque, or add domain variants
  (`DivisionByZero`, `UnknownOperator`, `UnanchoredOffset`, `DidNotSettle`,
  `TypeMismatch`) so callers can branch. The single `Eval(String)` variant is
  stringly-typed.

### R8 — Test fidelity and hygiene (fixes T1, T2, T3, T4)

- T1: `search_anchored_found_nyes_transitions` must call `assert_progression`
  with the expected terminal (like every other `*_nyes_transitions` test), not
  just `is_constanic()`.
- T2: delete `sff_struct_probe` (minified, unformatted, asserts nothing) or
  convert it to a real test. Unblocks `cargo fmt --check`.
- T3: delete the test-only `BraneFir`/`LeafFir` in `fir_trait::tests`; rewrite
  those tests on the real kinds (as `fir_kinds::tests` already does). The fake
  `BraneFir` has different `fir_op_step` logic and cannot catch regressions in
  the real kind.
- T4: `make_root_brane` acknowledges it leaves children self-rooted ("harmless
  for these tests"). Rebuild children inside the root's `new_cyclic` closure
  (as the real compiler does) so `is_root()`/parent traversal is correct in
  tests that exercise `ib_search`/`ab_search`.

### R9 — Lower-priority improvements (C3, C4, C9, P2, P4, A3)

- **C3**: SF re-evaluation only re-checks the immediate brane. Run the full
  IB→AB pipeline against the consuming context, or document the limitation and
  add a regression test.
- **C4**: concatenation shares statements by `Rc::clone` rather than
  constanic-cloning them (spec §2 says "constanically clones"). Masked because
  concat results are terminal, but it's an aliasing hazard.
- **C9**: `get_value` and `search_brane_children`'s SF re-search recurse
  without a depth guard. Add a bound or iterate.
- **P2**: `find_stmt_index_in_brane` is O(N) per search via `ptr_eq` scan. The
  `line_number` field already exists on `StatementFir`; use it as the bound
  (this is the spec's Phase 3a "upward navigation trio" — not implemented).
- **P4**: concatenation walks children 3× (`any_nk`, `any_woconstanic`,
  merge). Single pass with cached resolutions.
- **A3**: the 400-line `proto_to_core_fir` bridge. Either implement the spec's
  `FirQueryable` adapter on ProtoBrane (Phase 3a) or document the bridge as an
  explicit interim measure and decompose it per-kind (replace the
  `preserve_search: bool` flag with an enum).

## Detailed findings

### Correctness (C1–C9)

#### C1 — HeadTailFir does not drain its cloned result

**Location**: `fir_kinds.rs:1256-1344` (`HeadTailFir::fir_op_step`).

`SearchFir` and `IndexFir` were fixed (plan #23) to push the NICC-cloned
result, stay `Braning`, and settle *next step* via `settle_from_ubc_result()`.
`HeadTailFir` was not. In both the unanchored Prembrionic arm (1277–1285) and
the anchored Braning arm (1303–1306):

```rust
self.core.push_search_result(constanic_clone_at(&body, &self_weak, 0, scope.has_ancestral_sfm));
self.core.set_nyes(nyes);   // settles in the SAME step — clone not drained
```

`push_ubc_child` enqueues the clone as a task, but `set_nyes(nyes)` settles
`HeadTailFir` immediately. The driver sees it as constanic and pops it from its
parent's queue **without draining its own task queue** — the EMBRYONIC
(NICC-reset) clone never steps. The sequencer then renders a stale EMBRYONIC
result under a "settled" HeadTail. Masked for constant bodies (CONSTANT clones
stay CONSTANT). This is the same class of bug plan #23 fixed for SearchFir.

#### C2 — Index/HeadTail skip `search_nyes_from_found` capping

**Location**: `fir_kinds.rs:1157-1165` (`IndexFir::settle_from_ubc_result`);
`HeadTailFir` (no `settle_from_ubc_result` at all).

`SearchFir::settle_from_ubc_result` caps via `search_nyes_from_found()` —
ECONSTANIC/WOCONSTANIC/NK found → WOCONSTANIC; CONSTANT/INDEPENDENT → CONSTANT
(never INDEPENDENT). `IndexFir::settle_from_ubc_result` copies the clone's nyes
verbatim:

```rust
fn settle_from_ubc_result(&self) {
    let nyes = self.core.ubc_children().first()
        .map(|r| r.borrow().core().get_nyes())
        .unwrap_or(Nyes::Nk);
    self.core.set_nyes(nyes);   // ← NO capping
}
```

FOOP-62 §Terminology classifies Index and HeadTail as **searches** (anchor +
result). The Atlas ruling (plan #15) says "a search is never INDEPENDENT" and
caps found states. Index/HeadTail violate this: a found ECONSTANIC body yields
ECONSTANIC (should be WOCONSTANIC); a found INDEPENDENT yields INDEPENDENT
(should be CONSTANT). **Related to C1**: the fix is one shared
`settle_from_ubc_result` + `search_nyes_from_found` path for all three kinds.

#### C3 — SF-wrapped unanchored search re-checks only the immediate brane

**Location**: `fir_kinds.rs:829-836` (`search_brane_children`, SF branch).

When a statement's body is an SF wrapping an unanchored search, the code
re-runs `search_brane_children(brane, pattern, before, forward)` — on the
**same brane** with the **same `before` bound**. It does not escalate to
ancestral branes. If the SF's inner pattern resolves only in an *ancestral*
brane, this returns `None` and the SF value is lost. The fix is to run the full
IB→AB pipeline against the consuming context, not a single
`search_brane_children` call.

#### C4 — Concatenation shares statement FIRs by `Rc::clone`

**Location**: `fir_kinds.rs:1496-1510`.

```rust
for stmt in borrowed.core().foolish_children() {
    merged_stmts.push(Rc::clone(stmt));   // shared, not cloned
}
```

The result brane **shares** statement nodes with the input branes rather than
constanic-cloning them (spec §2: "constanically clones elements into the result
brane"). Shared mutable `Rc<RefCell<…>>` across independent branes is an
aliasing hazard: a future re-step of the result mutates the originals. Masked
because concat results are currently terminal.

#### C5 — Operator silently eternal-Woconstanic on non-integer operands

**Location**: `fir_kinds.rs:537-539`.

```rust
if values.len() != children.len() {
    self.core.set_nyes(Nyes::Woconstanic);
    return Ok(());
}
```

If an operand isn't an `i64` (a brane, or a search resolving to a brane),
`as_i64()` returns `None`, `values` is short, and the operator goes Woconstanic
— and stays there on every subsequent Braning call. No NK, no error, no alarm.
FOOP-62 does not define brane arithmetic; the operator simply never resolves.
**Related to C6**: both are silent-failure modes that produce wrong output
without any alarm.

#### C6 — `MAX_STEPS` exhaustion returns `Ok(())`

**Location**: `evaluator.rs:50-58`.

```rust
for _ in 0..MAX_STEPS {
    match crate::step_fir_ref(fir_ref, scope)? {
        StepReport::Progress(nyes) if nyes.is_constanic() => return Ok(()),
        StepReport::NoProgress => return Ok(()),
        _ => {}
    }
}
Ok(())   // ← 10,000 steps, not settled — silent success
```

A non-settling tree (e.g. an operator stuck per C5) burns 10,000 steps, then
returns `Ok(())` with a partially-evaluated tree. `proto_to_core_fir` renders
it as if final — no alarm, no error. The spec (§4) says `NoProgress` should
drive clean termination and `max_steps` is a "belt-and-suspenders guard".

#### C7 — `unreachable!` in `constanic_clone_at` SF/SFF arm

**Location**: `fir_kinds.rs:159-164, 289-291`.

The top guard strips SF/SFF by recursing on the inner child — but only if
`foolish_children().first()` is `Some`. If an SF/SFF node has **no children**
(malformed tree, future variant), the guard falls through to the `eprintln!`
alarm and continues into the `match` with `kind == StayFoolish`, hitting
`unreachable!("SF/SFF resolved to source at fn top")` → **panic in a production
evaluation path**. The empty-child case is reachable from sibling code, so this
is not a proven-impossible state. **Related to C8**: both are `unreachable!` in
interpreter paths.

#### C8 — `unreachable!` in the compiler on parser output

**Location**: `compiler.rs:229-232`.

```rust
Astn::Assignment { .. } => unreachable!("standalone Assignment should be wrapped in Brane by parser"),
_ => unreachable!("validate_astn should have rejected this"),
```

The compiler is the parser→FIR boundary. A parser bug or a new `Astn` variant
reaches here as a panic. `rust_instructions.md`: "A syntax error in user code
is a diagnostic, not a Rust panic." Return `anyhow::Error` (the function
already returns `FirRef`; make `build_fir` return `Result` or push the
validation into `validate_astn`).

#### C9 — `get_value` and SF re-search have no depth guard

**Location**: `fir_trait.rs:192-206` (`get_value`); `fir_kinds.rs:829-836`
(`search_brane_children` SF recursion).

`get_value` recurses through `ubc_children` with no depth limit (unlike
`step_fir_ref_inner`'s `MAX_DEPTH`). `search_brane_children`'s SF re-search
recurses on nested SF wrappers. A pathological chain can stack-overflow. Add
a depth bound or iterate.

### Architecture / spec-divergence (A1–A4)

#### A1 — `bon` builders not used (spec §6 unimplemented)

**Location**: `foolish-ubca/Cargo.toml` (no `bon` dep); `fir_kinds.rs:1539-1631`
(hand-written builders); `fir_kinds.rs:362-447` (`pub(crate)` fields).

Spec §6/§6a makes `bon`-generated builders with compile-time-required
`parent`, `#[non_exhaustive]`, private fields, and builder-only construction a
**hard invariant** ("enforced by the language, not just docs"). `bon = "3"` is
in `[workspace.dependencies]` and human approval is recorded in the plan, but
`foolish-ubca/Cargo.toml` does not depend on it. Fields are `pub(crate)`, not
private. No `#[non_exhaustive]`. A struct literal from anywhere in the crate
compiles. This is the largest spec gap and the root of D1 (clone-code
repetition) and V5/V6 (encapsulation).

#### A2 — Interior-mutability pivot not reflected in spec §3

**Location**: `proto_brane.rs:24-52` (`Cell`/`RefCell` fields);
`fir_trait.rs:107` (`fir_op_step(&self)`); spec §3.1/§3.2 (`&mut self` +
transient borrows).

The implementation uses `nyes: Cell<Nyes>`, `ubc_children: RefCell<Vec>`,
`tasks: RefCell<VecDeque>`, and `fir_op_step(&self)`. `step_fir_ref_inner`
calls `this.borrow().fir_op_step()` (shared borrow). The spec describes
`fir_op_step(&mut self)` and the elaborate "read neighbors into locals, drop
borrow, then write" discipline. The implementation is defensible (shared
borrows coexist), but the **safety invariant** — `ubc_children()` returns a
clone precisely so no `borrow_mut()` overlaps a `borrow()` — is undocumented.
The spec has not been updated. **Related to P3**: returning `Ref<'_, [FirRef]>`
makes the invariant compiler-checked.

#### A3 — `proto_to_core_fir` 400-line bridge reintroduces `clone_into_fir`

**Location**: `evaluator.rs:142-540` (`proto_to_core_fir_inner` +
`proto_to_core_fir_sff_body` + `anchor_to_core_fir`).

The spec's **Motivation** rejects "pervasive `clone_into_fir()` calls used
purely to pattern-match"; §Phase 3a mandates a "thin `FirQueryable` adapter
(~100 lines)". Instead, the output path is a 400-line recursive translator that
rebuilds a `foolish_core::Fir` tree for rendering. The `Search` arm alone
(256–343) is ~90 lines with 5 nested special cases. The `preserve_search: bool`
flag threads through 300 lines as a flag argument. This is the main complexity
center and the largest maintainability risk.

#### A4 — Bridge recomputes brane NYES from converted children

**Location**: `evaluator.rs:236-249`.

```rust
let mut effective_state = state;
if state == Nyes::Constant || state == Nyes::Independent {
    for (_, body) in &stmt_tuples {
        let body_state = body.hs_state();
        if body_state == Nyes::Econstanic || body_state == Nyes::Woconstanic {
            effective_state = Nyes::Woconstanic;
            break;
        }
        // …
    }
}
```

The rendering layer overrides UBCa's computed brane NYES from converted
children's `hs_state()`. If UBCa says a brane is CONSTANT but a child body is
ECONSTANIC, the bridge says WOCONSTANIC. This is a layering violation: the
renderer re-evaluates. It masks evaluator bugs and contradicts "UBCa is its own
source of truth." **Related to A3**: both are bridge-design issues.

### Convention violations (V1–V7)

#### V1 — `#[allow(dead_code)]` instead of `#[expect(...)]`

**Location**: `proto_brane.rs:131`. `rust_instructions.md` Don'ts: "Don't use
`#[allow(lint)]`. → `#[expect(lint)]` with a reason."

#### V2 — `unsafe_code = "warn"`; no clippy correctness gate

**Location**: `foolish-ubca/Cargo.toml:18`; `foolish/Cargo.toml:23-24`.
`rust_instructions.md` treats this crate as security-critical. `warn` allows
`unsafe` with a warning. No `[lints.clippy]` at all (no `correctness = "deny"`).

#### V3 — `eprintln!` for alarms/logging

**Location**: `fir_kinds.rs:163`; `ubca_snapshot_tester.rs:31,37`. Plan Notes
already flag this ("Use `tracing` for alarms instead of `eprintln!`").

#### V4 — `as i64` cast in the compiler

**Location**: `compiler.rs:108` (`value: n as i64`). Lossy cast smell; use
`TryFrom` or remove if the parser type is already `i64`.

#### V5 — `FirKind::Unknown` placeholder variant

**Location**: `fir_trait.rs:44`; `fir_kinds.rs:348-353` (silently converts to
NkFir). "Make illegal states unrepresentable." Confine stubs to `#[cfg(test)]`.

#### V6 — Missing `#[non_exhaustive]`

**Location**: `FirKind`, `StepReport`, `UbcError`, `Scope` — all public, all
may gain variants.

#### V7 — `UbcError` is single-variant stringly-typed

**Location**: `fir_trait.rs:85-89` — only `Eval(String)`. Callers can't branch.
Either `anyhow` (opaque) or domain variants (`DivisionByZero`,
`UnknownOperator`, `UnanchoredOffset`, `DidNotSettle`, `TypeMismatch`).
**Related to C5/C6**: those fixes want typed errors to branch on.

### Performance (P1–P4)

#### P1 — `Regex::new` compiled per statement match

**Location**: `fir_kinds.rs:760-768` (`matches_pattern`).

`search_brane_children` calls this for each statement in each brane searched.
For N statements × M searches, that's N×M regex compilations. Cache on
`SearchFir` via `OnceLock<Option<Regex>>`; short-circuit `^identifier$` via
`extract_simple_name`.

#### P2 — `find_stmt_index_in_brane` is O(N) per search

**Location**: `fir_kinds.rs:750-758`. `Rc::ptr_eq` linear scan per search. For
nested searches, O(N²). `StatementFir.line_number` already exists; use it as
the bound (this is the spec's Phase 3a "upward navigation trio" — not
implemented).

#### P3 — `ubc_children()`/`all_children()` clone the Vec every read

**Location**: `proto_brane.rs:78,83`. Returns `Vec<FirRef>` (cloned), not
`&[FirRef]`. Hot path: `as_i64` (`fir_trait.rs:116`), `get_value` (197), the
bridge (dozens of calls per render). Return `Ref<'_, [FirRef]>` via
`Ref::map`. **Related to A2**: makes the borrow invariant compiler-checked.

#### P4 — Concatenation walks children 3×

**Location**: `fir_kinds.rs:1486-1510`. `any_nk` (with `get_value` recursion),
`any_woconstanic` (same), then the merge loop — three passes. Single pass with
cached `(FirRef, Nyes)` resolutions.

### Test quality (T1–T4)

#### T1 — `search_anchored_found_nyes_transitions` asserts no terminal

**Location**: `fir_kinds.rs:3168-3179`. Only `is_constanic()` + starts
Prembrionic. Every other `*_nyes_transitions` test pins the exact terminal via
`assert_progression`. A CONSTANT→WOCONSTANIC regression would pass.

#### T2 — `sff_struct_probe` — committed debug scaffolding

**Location**: `fir_kinds.rs:3448-3462`. Minified, unformatted `walk` + a `probe`
test that only `eprintln!`s. Fails `cargo fmt --check`. Delete or convert.

#### T3 — Duplicate test `BraneFir`/`LeafFir` in `fir_trait::tests`

**Location**: `fir_trait.rs:267-339`. Defines its OWN `BraneFir`/`LeafFir`
with different `fir_op_step` logic than the real kinds. So
`step_brane_drains_children_then_classifies` tests a **fake**. Cannot catch
regressions in the real `BraneFir`.

#### T4 — `make_root_brane` leaves children self-rooted

**Location**: `fir_trait.rs:370-376`. Acknowledged broken: "the incorrect
parent on children is harmless for these tests." `is_root()` returns true for
those children; any test exercising parent traversal through them sees wrong
behavior. Rebuild children inside the root's `new_cyclic` closure.

## FIR Impact

No new FIR variants are proposed. The fixes touch existing kinds'
`fir_op_step` and settle logic (C1, C2, C5), the clone path (C7, D1), and the
output bridge (A3, A4). The `FirKind` enum loses `Unknown` (V5) and gains
`#[non_exhaustive]` (V6).

## UBC Step Impact

- **C1/C2 (R1)**: HeadTailFir gains `settle_from_ubc_result`; all three
  search-classified kinds settle through `search_nyes_from_found`. A found
  ECONSTANIC/WOCONSTANIC body now yields WOCONSTANIC (was ECONSTANIC for Index,
  varied for HeadTail); a found INDEPENDENT yields CONSTANT (was INDEPENDENT).
  This may shift snapshot output for programs that index/head/tail into a brane
  containing unresolved searches — present for human review, do not auto-accept.
- **C5 (R2)**: operators on non-integer operands now settle NK (was: eternal
  Woconstanic). Snapshot-visible for programs that exercise this.
- **C6 (R2)**: max-steps exhaustion now errors (was: silent partial render).
  No snapshot shift for well-formed programs that settle within 10,000 steps.
- **A4 (R6)**: brane NYES is trusted from the evaluator (was: recomputed in the
  bridge). May shift snapshots if the evaluator's NYES was actually wrong and
  the bridge was masking it — exactly the class of bug this exposes.

## Test Plan

- **C1/C2**: extend `headtail_nyes_transitions` / `index_nyes_transitions` to
  cover a brane containing an ECONSTANIC search result; assert the terminal is
  WOCONSTANIC (not ECONSTANIC). Add a `*_settle_from_drained` regression test
  mirroring the plan #23 SearchFir test.
- **C5**: `operator_on_brane_operand_is_nk` (or returns error, per the decided
  semantics).
- **C6**: `max_steps_exhaustion_returns_error` (or emits alarm).
- **C7**: `constanic_clone_of_childless_sf_does_not_panic`.
- **C8**: `compiler_rejects_unsupported_astn` (returns error, not panic).
- **T1**: fix `search_anchored_found_nyes_transitions` to call
  `assert_progression` with the expected terminal.
- **T2**: delete `sff_struct_probe`.
- **T3**: rewrite `fir_trait::tests` on the real kinds; delete the stubs.
- **P1**: add a stress test (large brane, many statements) to catch the
  regex-compilation cliff.
- Snapshot shifts from C1/C2/C5/A4 are the intended review set — present
  `.snap.new` to the human; **AI must not auto-accept** (AGENTS.md).

## Rejected Alternatives

### A. Do nothing — UBCa is "its own source of truth," tests pass

The 119 unit tests pass and most failures are masked by constant-valued test
trees. But C1/C2 produce silently wrong output on non-constant inputs (exactly
the inputs the snapshot corpus exercises), C5/C6 are silent-failure modes with
no alarm, and A1 (no `bon`) leaves the spec's hard invariant unenforced.
"Tests pass" is not "correct" when the tests don't cover the failing cases
(T1, T3). Doing nothing ships known latent bugs.

### B. Defer everything to a post-merge cleanup

The P0 items (C1, C2, C5, C6, C7, C8) are small, surgical fixes that are
cheaper to land before merge than after (no snapshot re-review churn, no
separate FOOP). A1 (the `bon` decision) blocks D1 and V5/V6 — deferring it
means the clone code and encapsulation stay as-is until a second refactor. The
P1 items (P1 regex, P3 borrow) are the eval-speedup that makes the snapshot
suite pleasant to run. Deferring all is strictly more work.

## Open Questions

- **A1 (bon)**: adopt `bon` (implement spec §6) or amend FOOP-62 §6 to drop the
  requirement? Needs Atlas sign-off either way. This decision unblocks R3/R5.
- **C5 (operator semantics)**: non-integer operand → NK with reason, or typed
  `UbcError::TypeMismatch`? FOOP-62 does not define brane arithmetic.
- **V7 (UbcError)**: stay single-variant + `anyhow`, or add domain variants?
  Related to C5/C6 (those fixes want typed errors).
- **A3 (bridge)**: implement the spec's `FirQueryable` adapter now (Phase 3a),
  or document the bridge as interim and decompose it per-kind? The adapter is
  larger but retires the largest complexity center.
- **C3 (SF re-eval)**: run full IB→AB, or document the immediate-brane-only
  limitation and add a regression test? Depends on whether any snapshot
  exercises SF-wrapped ancestral-only searches.

## References

- Prior FOOPs: FOOP-62 (UBCa Two-Store ProtoBrane — the spec under review),
  FOOP-72 (FNS / snapshot organization).
- Standards: `rust_instructions.md` (Rust conventions), `AGENTS.md` (project
  rules, snapshot/auto-accept policy).
- Code locations:
  - `foolish/foolish-ubca/src/fir_kinds.rs` (C1–C9, P1, P2, P4, D1, T1)
  - `foolish/foolish-ubca/src/fir_trait.rs` (V5, V6, C9, T3, T4)
  - `foolish/foolish-ubca/src/evaluator.rs` (A3, A4, C6)
  - `foolish/foolish-ubca/src/proto_brane.rs` (V1, P3, A2)
  - `foolish/foolish-ubca/src/compiler.rs` (C8, V4)
  - `foolish/foolish-ubca/Cargo.toml` (A1, V2)
- Plan cross-refs: `FOOP-62.plan.md` Phase −1 (clone model), Notes/discoveries
  (bon `build_from`, `tracing` alarms), task #23 (settle-from-drained — missed
  HeadTail).

## Last Updated

**Date**: 2026-06-23
**Updated By**: Sisyphus / z-ai/glm-5.2
**Changes**: Created FOOP-82 from a focused code review of `foolish-ubca`.
Recorded 28 findings (9 correctness, 4 architecture, 7 convention, 4
performance, 4 test quality) with 9 prioritized recommendations, cross-
references, FIR/UBC step impact, and open questions for Atlas.
