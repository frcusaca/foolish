---
foop: 31
title: "MAX_BRANE_SIZE — auto-sizing via a non-merging ConcatBrane equivalent to the merged brane"
author: Atlas <hc.busy@gmail.com>
status: Brewing
type: Standards
created: 2026-07-03
phase: phase-2
supersedes: []
begun: [ ]
---

# FOOP-13: MAX_BRANE_SIZE — auto-sizing via a non-merging ConcatBrane

## Abstract

Each UBCa FVM gains a configuration value `MAX_BRANE_SIZE`. During brane
construction from the AST, the compiler automatically converts any brane whose
statement count exceeds `MAX_BRANE_SIZE` into a concatenation of smaller branes,
each within the limit. For that to be possible at all, `ConcatenationFir` must
stop merging its elements into one big brane (the merged brane would itself
violate the limit). Instead it becomes a **ConcatBrane**: a brane-like container
that constanic-clones its elements' statements into hidden, bounded
**_ConcatHelpers** and answers every brane operation — search, IB/AB resolution,
indexing, sequencing, cloning — through offset arithmetic over the
_ConcatHelper series. The governing law:

> **The Equivalence Law.** A settled ConcatBrane is observationally identical to
> a single big brane containing every statement of its element branes, in the
> same order — for every brane operation — except that the big brane is never
> materialized.

The work is **two phases**: **Phase A — the ConcatBrane upgrade** (the
Equivalence Law made true for source-level concatenation; a semantic repair in
its own right, since today's merge shares statements by `Rc::clone` without
recoordination or parent rewiring), then **Phase B — the MAX_BRANE_SIZE limit**
(configuration plus the auto-sizing rewrite). The default configuration is
unlimited, so Phase B changes nothing unless configured.

## Motivation

Branes are the unit of containment, cloning, and recoordination. Foolish branes
are by design finite in size. This FOOP implements that fact by setting a
uniform finite size limit for branes in the UBCa FVM implementation.
`MAX_BRANE_SIZE` bounds the granule: no `BraneFir` statement store ever exceeds
`k` statements; a ConcatBrane is an unbounded *view* over bounded units.

The current `ConcatenationFir` defeats that bound by construction: its `Braning`
arm concatenates every element's statements into one merged `BraneFir` — exactly
the oversized brane the configuration forbids. It also has two latent defects
the redesign fixes:

- The merged brane's statements are `Rc::clone`d, not constanic-cloned: their
  `.parent` links still point into the original element branes, and no
  recoordination happens — an element like `{b=a}` that settled ECONSTANIC
  before combination never gets the chance to resolve `a` against an earlier
  element (`{a=10}{b=a}`).
- Element identity is lost: the merged brane is a fresh unbounded statement list
  with no memory of the units it came from.

After this FOOP: UBCa has a per-FVM configuration surface, oversized branes are
transparently chunked at construction, concatenation preserves bounded units
internally, cross-element references resolve through genuine constanic-clone
recoordination, and the observable result obeys the Equivalence Law.

## Definitions

- **Brane** — a containment structure holding an ordered list of statements.
  Today: `BraneFir`. After this FOOP: `BraneFir` and `ConcatenationFir` (the
  ConcatBrane) are the two brane-like FIR kinds visible to resolution.
- **ConcatBrane** — the redesigned `ConcatenationFir`: a brane-like container
  whose statements live in hidden _ConcatHelpers, never materialized as one
  flat list. It IS its own value (like a BraneFir).
- **_ConcatHelper** — a new `FirKind` introduced by this FOOP. A **carrier**:
  a `ProtoBrane`-holding FIR that tracks a segment of lines (≤ `k`) on behalf
  of its ConcatBrane. It is **neither a brane nor a statement**. It implements
  no brane trait methods (inherits all defaults → transparent to resolution).
  Its sole behavior is stepping its segment's lines through the unaltered driver.
- **Element** — a direct child of a ConcatBrane's `foolish_children`; the
  brane-like values being concatenated.
- **Line** — a statement copied into a _ConcatHelper during the populate step.
- **k** — `MAX_BRANE_SIZE`. The bound on statements per BraneFir / _ConcatHelper.
- **Constanic** (adjective) — any terminal NYES state: ECONSTANIC, WOCONSTANIC,
  CONSTANT, INDEPENDENT, or NK. Pre-constanic (nigh) = needs more stepping.
- **Equivalence Law** — see above. The observable-contract guarantee.
- **Capability dispatch** — recognizing "brane-like" FIRs by capability
  (`stmt_count().is_some()`) rather than by `FirKind::Brane` kind-match. See
  §"Labeling and discipline" and Appendix C.

## Specification

### Phasing

The specification is implemented in two strictly ordered phases:

- **Phase A — ConcatBrane upgrade.** `ConcatenationFir` stops merging and
  satisfies the Equivalence Law over hidden _ConcatHelper storage. No
  configuration involved; source-level concatenation semantics are repaired.
  Phase A stands alone and is reviewed (snapshot churn included) before Phase B
  begins. Phase A also includes a behavior-neutral **labeling and discipline**
  refactor (§"Labeling and discipline") that makes the brane-finding machinery
  coherent before the ConcatBrane depends on it.
- **Phase B — MAX_BRANE_SIZE.** The `UbcaConfig` surface and the iterative
  auto-sizing rewrite. Depends on Phase A: the rewrite emits exactly the nested
  concatenation shapes Phase A must already evaluate correctly.

### Size

The **size of a brane** is its number of statements. Nested content does not
count toward the size of the outer brane; each brane is measured on its own
statement list.

### Configuration

```rust
/// Per-FVM configuration for UBCa (FOOP-13).
#[derive(Debug, Clone, Default)]
pub struct UbcaConfig {
    /// Maximum number of statements a constructed brane may hold.
    /// `None` (the default) means unlimited — no auto-sizing occurs.
    pub max_brane_size: Option<std::num::NonZeroUsize>,
}
```

- `UbcaEvaluator` changes from a unit struct to
  `pub struct UbcaEvaluator { pub config: UbcaConfig }` with `Default`.
- The compiler gains `Compiler::compile_with(source: &str, config: &UbcaConfig)`;
  `Compiler::compile(source)` delegates with `UbcaConfig::default()`.
- `NonZeroUsize` makes the degenerate `MAX_BRANE_SIZE = 0` unrepresentable; "no
  limit" is spelled `None`, not `0`.

### The auto-sizing rewrite (AST→AST) — Phase B

Applied inside the compiler after `validate_astn` and before `build_fir`. Let
`k = max_brane_size`. The rewrite recurses structurally through every `Astn`
variant; the only rewriting arm is `Astn::Brane` with `n = statements.len()`:

1. Recurse into each statement first (nested branes are auto-sized independently).
2. If `n <= k`, or the brane is **exempt** (below), leave it as-is.
3. Otherwise split `statements` (order preserved) into `m = ⌈n/k⌉` consecutive
   chunks — every chunk holds exactly `k` statements except possibly the last —
   giving the element array `[Brane(chunk₁), …, Brane(chunkₘ)]`, each chunk
   brane with empty characterizations.
4. **Iterate until it fits**: while the element array is longer than `k`, group
   it (order preserved) into consecutive runs of ≤ `k` elements, wrapping each
   run in an `Astn::Concatenation`; the runs become the new element array. When
   the array is ≤ `k` elements, the result is a single `Astn::Concatenation`
   over it.

The result is a k-ary concatenation tree of depth `⌈log_k m⌉ + 1`: leaves are
chunk branes of ≤ `k` statements, internal nodes are concatenations of ≤ `k`
elements. The bound is uniform — **no node in the constructed program holds more
than `k` children**, whether those children are statements or concatenation
elements.

Exemptions (never split): the **root brane** (only a Brane may be root — FOOP-62
root convention; its nested branes still auto-size) and **characterized branes**
(a ConcatBrane carries no characterizations; conservative until characterizations
gain merge semantics).

`Astn` is defined in `foolish-parser/src/ast.rs` (cross-crate); the rewrite
reuses existing `Brane`/`Concatenation` variants — no parser change expected.

### The ConcatBrane redesign — Phase A

#### _ConcatHelper — a carrier, not a brane

A new `FirKind::ConcatHelper`. Its struct:

```rust
pub struct ConcatHelper {
    pub(crate) core: ProtoBrane,
}
impl Fir for ConcatHelper {
    fn core(&self) -> &ProtoBrane { &self.core }
    fn kind(&self) -> FirKind { FirKind::ConcatHelper }
    fn fir_op_step(&self, _scope: &Scope) -> Result<(), UbcError> {
        // BraneFir-shaped: push children → Braning → _decide_nyes_due_to_children
        // (see §"The ConcatBrane protocol" below)
    }
}
```

_ConcatHelper is **neither a brane nor a statement**. Consequences:

- **Not brane-like**: does NOT implement `stmt_count`/`stmt_at` (defaults return
  `None`). `is_brane_like()` → false. `get_my_brane` walks through it.
  `_search_brane` stays at default (`None`). `settled_result()` stays at default
  (`None`). No `_ab_search` override.
- **Not a statement**: `get_my_statement` walks through it (default only stops
  at `FirKind::Statement`).
- **A FIR**: `impl Fir` — the driver dispatches to it via `dyn Fir`. It composes
  a `ProtoBrane` (same as every FIR kind), which gives it `foolish_children`,
  `parent`, NYES, and the task-queue primitives for free.
- **Transparent to both brane-finding mechanisms**: parent-walk (`get_my_brane`)
  skips it (not brane-like); scope-cached (`scope.current_brane`, set by
  `step_inner` only for `FirKind::Brane`) is never set to it. Both reach the
  ConcatBrane. This eliminates the IB/AB-vs-indexing asymmetry (Appendix C).

Its `fir_op_step` mirrors `BraneFir`'s shape (push `foolish_children` as tasks →
set `Braning` → `_decide_nyes_due_to_children` on drain). The unaltered driver
drains its segment's lines through it without any special-casing. A
`concat_helper_nyes_transitions` unit test is required (per AGENTS.md).

#### Storage: flat Vec of _ConcatHelpers

The ConcatBrane's `ubc_children` holds a **flat `Vec<FirRef>` of _ConcatHelpers**.
There are no "stack ConcatBrane" internal nodes — the "k-ary tree" is the
**logical access pattern** via prefix sums, not a physical tree of
`ConcatenationFir`s. This avoids the stack-node `fir_op_step` misfire (a
`ConcatenationFir` in storage mode would hit the empty-`foolish_children` branch
and corrupt the tree).

- Offsets are prefix sums over _ConcatHelper line-counts, computed on access.
- The tree shape is a pure function of the total line count `n` and `k`: lines
  chunk into _ConcatHelpers of ≤ `k`; the flat Vec holds all of them. Unlimited
  `k` degenerates to a single _ConcatHelper holding all `n` lines.
- The ConcatBrane reads _ConcatHelper children via `core().foolish_children()`
  directly (it owns them), NOT via the trait capability surface.

#### The ConcatBrane protocol (replaces the merge)

**Fundamental contract: a ConcatBrane takes a list of BRANES and concatenates
them.** The contract is enforced at every phase; a violation raises an alarm
and labels the ConcatBrane `NK` — it no longer steps.

**Three-phase progression** (discriminated by `ubc_children` emptiness — no
phase field, no new NYES state):

| `fir_op_step` call | NYES at entry | Queue | `ubc_children` | Action |
|---|---|---|---|---|
| 1 | Embryonic | empty | empty | push elements as tasks → set `Braning` |
| *(driver drains elements)* | | | | |
| 2 | Braning | empty (all elements constanic) | empty | **populate**: count lines, arrange into _ConcatHelpers, constanic-copy, push _ConcatHelpers → stay `Braning` |
| *(driver drains _ConcatHelper revived searches)* | | | | |
| 3 | Braning | empty (_ConcatHelpers drained) | non-empty | **settle**: terminal rule |

The discriminator within `Braning` is `ubc_children` emptiness: call 2 (empty →
populate) vs call 3 (non-empty → settle). `fir_op_step` is only called when the
task queue is empty, so call 2's elements are guaranteed constanic. Corner case:
if `n=0` at call 2, settle as an empty constant brane immediately (don't push
_ConcatHelpers).

**Protocol steps:**

1. **Element typing — construction-time first pass.** Every direct element of
   `foolish_children` must be one of:
   1. a Foolish brane that was typed out (a literal), e.g. `{asdf=1}`;
   2. a search expression that could resolve to a brane — plain (`b`), anchored
      (`b.x`), regexp (`c~something`), or positional; or
   3. an **explicitly SF-marked expression** `<…>` whose inner expression is
      itself form (1) or (2).

   Any other FIR kind — and any **explicitly SFF-marked (`<<…>>`) element**,
   which is an error — raises an alarm and the ConcatBrane is constructed `NK`.
2. **Auto-wrapping at construction.** ConcatBrane construction normalizes every
   element to a wrapped form:
   - a bare **literal brane** is wrapped in **SFF**: reusing the existing
     `under_sff` build rule (FOOP-62.plan.md task #17), every search inside it
     is BORN ECONSTANIC and never resolves standalone — the literal's innards
     are deferred to the join (resolve AFTER);
   - a bare **search** is wrapped in **SF**: it resolves BEFORE (it identifies
     the constituent), and the found brane's lines detach-and-recoordinate at
     copy time;
   - an **explicit SF around a search** is **idempotent** — a NOOP, identical to
     the auto-SF;
   - an **explicit SF around a literal brane** is an **override of the automated
     SFF wrapping** — the literal's internal searches resolve BEFORE the copy,
     from the ConcatBrane's own statement position. Built WITHOUT `under_sff`.

   | element as written | wrapping | timing of its searches |
   |---|---|---|
   | `{…}` bare literal | auto-SFF (default) | innards resolve AFTER, in position in the join |
   | bare search (`b`, `b.x`, `c~…`) | auto-SF | resolves BEFORE; lines recoordinate on copy |
   | `<search>` | idempotent NOOP | same as bare search |
   | `<{…}>` | SF overrides the auto-SFF | innards resolve BEFORE, locally, then copy |
   | **all other FIRs** (incl. explicit `<<…>>`) | — | error: alarm + `NK` |

3. **Step to constanic (call 1 → drain).** Push elements as tasks; stay
   `Embryonic`. The driver drains them to constanic. SFF-wrapped literals settle
   without any internal resolution; SF-marked literals settle WITH full internal
   resolution in the concat's own context.
4. **Settle-time typing — second pass (call 2).** Each element's value must be a
   brane; anything else (a search that resolved to an integer, an NK element):
   alarm + `NK`.
5. **Count and arrange (call 2).** Count the total lines `n` across all
   constituent values, in order. Compute the storage as the pure function of
   `(n, k)`: lines chunk into _ConcatHelpers of ≤ `k`. The flat Vec of
   _ConcatHelpers is the arrangement. Unlimited `k` → single _ConcatHelper.
6. **Constanic-copy and coordinate (call 2).** Each line is constanic-copied to
   its _ConcatHelper position. Each line's:
   - **parent** = its _ConcatHelper (uniform parent chain — no bypass magic).
   - **line_number** = its global index across the whole joined brane (this
     single rule makes `StatementFir::_ib_search`
     `brane._search_brane(name, line_number − 1, 0)` work for cross-_ConcatHelper
     IB once the ConcatBrane's `_search_brane` maps global ranges to per-_ConcatHelper
     local ranges).
   - **NYES** transformed per `transform_for_clone(sfm=false)`: SFF-born
     ECONSTANIC searches revive **ECONSTANIC → EMBRYONIC**, ready to search.
     Lines from SF'd elements recoordinate per FOOP-7: constants keep their
     values, ECONSTANIC revives, NK stays NK.
   - Push each _ConcatHelper via `push_ubc_child` (auto-enqueues non-constanic
     revived searches). Set `Braning`.
7. **Settle (call 3).** Terminal rule (unchanged): any NK → `NK`; any
   ECONSTANIC/WOCONSTANIC → `WOCONSTANIC`; else `CONSTANT`. (NK wins over
   WOCONSTANIC, matching the current inline rule at `fir_kinds.rs:1512-1518`.)

**Worked example.** `{cb = {a=1, b=2} {c = a + b};}` — both literals are
auto-SFF'd, so `c = a + b`'s searches are born ECONSTANIC and skip standalone
resolution; the elements settle immediately; `n = 3`; the lines copy into
position; `c`'s searches revive EMBRYONIC and IB-find `a` and `b` in the joined
brane: `c = 3`. (Verified 2026-07-04 against the current evaluator: today this
program leaves `c` UNRESOLVED — the constituents settle standalone, `a`/`b` are
not findable from inside `{c=a+b}`, and nothing revives after the merge. This
protocol is the repair.)

#### Capability dispatch instead of kind matching

How two brane types "generically, dynamically" share operations: through `dyn Fir`
trait methods — the mechanism `_search_brane` already uses. New/changed `Fir`
methods:

```rust
/// Number of statements this FIR presents as a brane. `None` = not brane-like.
fn stmt_count(&self) -> Option<usize>;          // Brane: foolish_children.len(); ConcatBrane: Σ over _ConcatHelpers
/// The statement at a global index, per the Equivalence Law.
fn stmt_at(&self, idx: usize) -> Option<FirRef>; // Brane: foolish_children[idx]; ConcatBrane: _ConcatHelper descent
/// The settled result this FIR resolves to, if any. Each kind interprets its own
/// ubc_children; the default preserves today's behavior for the result-style kinds.
/// CONTRACT: applies the constanic gate ITSELF — pre-constanic always answers None.
/// None means "nothing to resolve into": either not settled yet, or settled and
/// I AM my own value. It can never return "self" — the caller holding the handle
/// substitutes self.
fn settled_result(&self) -> Option<FirRef> {
    if !self.core().get_nyes().is_constanic() { return None; }
    self.core().ubc_children().into_iter().next()
}                                                // ConcatBrane: None — it IS its value
```

- `ConcatenationFir::_search_brane(expr, start, end)`: map the global,
  direction-aware range onto per-_ConcatHelper local ranges via prefix sums,
  read each `_ConcatHelper.core().foolish_children()` directly and scan (same
  pattern as `BraneFir::_search_brane`). Translate the hit index back to global.
- `ConcatenationFir::_ab_search`: identical logic to `BraneFir::_ab_search`
  (enclosing statement's IB, then parent brane) — shared via a free function or
  default method, not duplicated.
- `Fir::as_i64` needs NO override: a brane never yields an integer, so the
  default chain reaches a _ConcatHelper — a brane — and returns `None` naturally.

**`FirRefExt::value`** is re-expressed over `settled_result()` and becomes
trivial: `Some(r)` → recurse into `r`; `None` → return the handle itself. Today
`value()` hard-codes "constanic + non-empty `ubc_children` ⇒ resolve to
`ubc_children[0]`" — a fact about the result-style kinds written as if it were a
law of ProtoBrane. It is not: `ubc_children` is the general compute-time store,
and each kind interprets its own contents. The default `settled_result()`
preserves current behavior for every existing kind; ConcatBrane answers `None`
because its `ubc_children` is storage, not a result chain. **This rewrite is
mandatory** — without it, `value()` on a settled ConcatBrane returns its first
_ConcatHelper, breaking search-result resolution, `constanic_clone_at`'s
`child.value()` calls, and `proto_to_core_fir`.

**Kind-match conversion.** Every site that hard-matches `FirKind::Brane` to mean
"a brane-like thing" switches to the capability (`stmt_count().is_some()`, or an
`is_brane_like()` helper on `FirKind`): `get_my_brane` (fir_trait.rs),
`find_parent_brane` (fir_kinds.rs — unified, see §"Labeling and discipline"),
the SearchFir anchored arm (fir_kinds.rs), `step_inner`'s `current_brane`
assignment (fir_trait.rs), and the `proto_to_core_fir` bridge sites in
evaluator.rs. Sites that genuinely mean the `BraneFir` kind (e.g. FIR
construction) stay as-is. _ConcatHelper is NOT brane-like (does not implement
`stmt_count`) — it is transparent to all of these.

**FOOP-23 interaction — `BraneNavigator` and contexted search.** FOOP-23
introduced the `ContextfulSearch` engine (private `contextful_search` module in
`fir_kinds.rs`), including `BraneNavigator` — the candidate iterator used by
contexted searches (`&?`, `&~`, `&#`, `&^`, `&$`, `&~=`, `&?=`).
`BraneNavigator::new(brane, forward)` reads `brane.core().foolish_children()`
directly — for a ConcatBrane that is the *element list*, not the joined
statements. Contexted searches would scan elements instead of statements.
`BraneNavigator` must be re-expressed over `stmt_count`/`stmt_at`: the navigator
holds a `Vec<FirRef>` of statements built by iterating `0..stmt_count()` and
collecting `stmt_at(i)`. Similarly, `contexted_search_from_anchor`
(`fir_kinds.rs`) reads `h_brane.borrow().core().foolish_children().len()` for
`brane_len` — this must become `stmt_count()`. Without these, contexted
searches from a ConcatBrane position are broken.

**FOOP-23 interaction — two-child invariant.** FOOP-23's `push_search_result_pair`
pushes `[clone_of_body, FoolRefFir]` to `ubc_children`. When a search resolves
to a statement inside a ConcatBrane, the FoolRefFir wraps the ORIGINAL statement
(inside its _ConcatHelper's `foolish_children`). A following contexted search
reads this FoolRefFir, calls `get_my_brane` (→ ConcatBrane, walking through the
_ConcatHelper), then `find_stmt_index` (→ needs `stmt_at`-based scan, not
`foolish_children`). This chain works IF `find_stmt_index` and `BraneNavigator`
are re-expressed over `stmt_count`/`stmt_at` (above). The two-child invariant
itself needs no change — the FoolRefFir at `[1]` is invisible to `value()` and
`settled_result()` (both read `[0]` only).

#### Indexing

`FirRefNavExt::index_into` and `find_stmt_index` currently read `foolish_children`
directly — on a ConcatBrane that is the *element list*, not the statement series.
Both are re-expressed over `stmt_count`/`stmt_at`:

- `index_into(offset)`: non-negative counts from the global front, negative from
  the global back; `#9` into a concatenation of two 5-statement branes lands on
  _ConcatHelper 2, local index 4 — the last statement. Out of range → `None` → NK.
- `find_stmt_index` returns the global index (identity scan across _ConcatHelpers).
- `index_into_brane_relative` (unanchored `^#-n`) and `HeadTailFir` route through
  the same accessors.
- **`BraneNavigator`** (FOOP-23's `contextful_search` module): the candidate
  iterator used by contexted searches reads `foolish_children()` directly —
  re-expressed over `stmt_count`/`stmt_at` (see §"FOOP-23 interaction" above).
  Also: `contexted_search_from_anchor`'s `brane_len` computation and the anchored
  search arm's `len` reads in SearchFir/IndexFir/HeadTailFir — all switch from
  `foolish_children().len()` to `stmt_count()`.

#### Parent links (normative summary)

| node | `.parent` |
|---|---|
| element FIRs (`foolish_children`) | ConcatBrane (as today) |
| _ConcatHelpers (`ubc_children`) | ConcatBrane |
| statements inside any _ConcatHelper | their _ConcatHelper (uniform chain — no bypass) |
| bodies cloned OUT by search/index | the addressing FIR's context, via `constanic_clone_at` (as today) |

The chain from a line to the enclosing brane is: `line → _ConcatHelper →
ConcatBrane → enclosing brane`. `get_my_brane` walks through _ConcatHelper (not
brane-like) and stops at ConcatBrane (brane-like). No parent-bypass rewiring is
needed — the chain is uniform, and transparency does the work.

#### Constanic-cloning a ConcatBrane — `skip_foolish_children`

When a settled ConcatBrane is itself the found result (e.g. `x = c` where `c`
names a concatenation), the clone copies **just the `ubc_children`** — the
settled _ConcatHelper storage — via a new general clone option:

```rust
constanic_clone(…, skip_foolish_children = true)
```

The element FIRs in `foolish_children` are NOT cloned and the element searches
NEVER re-run: a settled ConcatBrane clones as a **value**, exactly like any other
settled brane. The clone must: deep-clone the _ConcatHelper storage, rewire the
cloned lines' parents to the NEW _ConcatHelpers (and _ConcatHelpers' parents to
the new ConcatBrane clone), preserve global line numbers and arrangement, and
transform NYES per the standard clone rules.

The option is general: every singular-result kind (SearchFir, IndexFir,
HeadTailFir) is in the same position once constanic — cloning with
`skip_foolish_children = true` copies the result without dragging the dead
anchor subtree. Phase A scope: the option itself + the ConcatBrane use + adoption
in the settled-search clone path.

#### Sequencing (REQUISITE): a ConcatBrane displays as a single brane

- **A settled ConcatBrane sequences as ONE brane.** `proto_to_core_fir` walks the
  _ConcatHelper storage in global order and emits a single brane: one pair of
  enclosing braces, every line in sequence. No _ConcatHelper boundaries, no
  element boundaries, no auto-SF/SFF wrappers appear in the output. Byte-identical
  to the equivalent big brane's rendering.
- **k-invariance.** Because the display flattens, the same program sequences
  byte-identically under ANY `MAX_BRANE_SIZE`. This is what lets a snapshot
  approved in Phase A (unsplit) stand unchanged through Phase B (split).
- **Pre-constanic and NK rendering.** A concat that has not settled renders its
  elements as written; an NK ConcatBrane renders as NK with its alarm reason.

#### Labeling and discipline (Phase A, behavior-neutral)

The codebase has three mechanisms for "find my home brane" (Appendix C) that are
inconsistent and duplicated. Phase A unifies and documents them:

1. **Rename** `Fir::get_my_brane` → `Fir::_get_my_brane` and
   `Fir::get_my_statement` → `Fir::_get_my_statement` (underscore prefix =
   iterative parent-walk, matching the existing `_ib_search`/`_ab_search`/
   `_search_brane` convention).

2. **Document the call chains** in doc comments:
   - `_get_my_brane`: "Iterative parent-walk. Climbs `.parent()` until a
     brane-like kind is found (capability: `stmt_count().is_some()`). Returns the
     brane that owns `self`, or `None` at the root."
   - `_ib_search`: document the full chain — `StatementFir::_ib_search` →
     `_get_my_brane(self_ref)` (parent-walk) → `brane._search_brane(name,
     line_number-1, 0)`. Note the scope-cached twin `ib_search(scope, name)`.
   - `_ab_search`: document the chain — `_get_my_brane(self_ref)` →
     `brane._ab_search(brane, name)` → recurses up. Note the scope-cached twin
     `ab_search(scope, name)`.

3. **Unify `find_parent_brane`** (free fn, `fir_kinds.rs:1075`) into a thin
   wrapper over `_get_my_brane`:
   ```rust
   fn find_parent_brane(start: &ProtoBrane) -> Option<FirRef> {
       start.parent().and_then(|p| p.borrow()._get_my_brane(&p))
   }
   ```
   Delete the duplicated walk logic. `find_enclosing_stmt_and_brane` similarly
   delegates. One implementation of the brane-walk; the indexing and search
   subsystems share it.

4. **Doctrine correction**: rewrite the `FirRefExt::value` and `Fir::as_i64` doc
   comments in `fir_trait.rs` and scope the `(result=)` aside on
   `ProtoBrane::all_children` in `proto_brane.rs`; sweep `foolish-ubca` code
   comments and `docs/` for any other "ubc_children = result" phrasing.

This refactor is behavior-neutral: `cargo insta test` produces ZERO `.snap.new`.
It makes the brane-finding machinery coherent before the ConcatBrane depends on
capability dispatch.

#### The Integer brane (informative — a consumer of this design)

The ConcatBrane upgrade is the enabling mechanism for the **Integer brane**,
documented here as a design driver; its implementation is a follow-on FOOP.

The idea: numbers like `10` stop being lexical literals and become
**identifiers**. The Integer brane is a **custom-built brane** — not compiled
from Foolish source — that, when asked, returns what the identifier `10` means.
Programs run as `Concat(IntegerBrane, program_brane)`; ordinary backward IB
search falls through the program's statements into the Integer brane.

The Integer brane is a THIRD kind of brane, concat-able exactly like them. After
this FOOP, "being a brane" means implementing the trait surface
(`_search_brane`, `stmt_count`, `stmt_at`, `_ab_search`, `settled_result`) — not
being the `BraneFir` struct. The _ConcatHelper design keeps this seam open:
_ConcatHelper is a carrier, and a future generative brane could be another
carrier kind (or the populate step could adopt generative branes through their
trait surface without flattening). Deliberately not decided here; needs its own
FOOP.

## FIR Impact

- **New FIR variant**: `FirKind::ConcatHelper`. Struct: `ConcatHelper { core: ProtoBrane }`.
  Implements `core`, `fir_op_step`, `kind` (required); inherits all defaults.
  Required: `concat_helper_nyes_transitions` unit test.
- **No new NYES state.** `FirKind` gains `ConcatHelper`. `ConcatenationFir`
  interprets its `ubc_children` as _ConcatHelper storage. `concatenation_nyes_transitions`
  is extended for the populate-then-drain progression.
- **`Fir` trait gains** `stmt_count`, `stmt_at`, `settled_result` (defaults
  preserve current behavior; ConcatBrane overrides `stmt_count`/`stmt_at`/
  `settled_result`; _ConcatHelper inherits all defaults). Note: FOOP-23 added
  `as_search_is_value`, `as_search_contexted`, `set_contexted`,
  `as_fool_ref_referent` to the trait — these are unaffected by this FOOP.
- **Exhaustiveness matches** that MUST gain a `ConcatHelper` arm:
  `constanic_clone_at` (`fir_kinds.rs`) and `proto_to_core_fir_inner`
  (`evaluator.rs`). Note: FOOP-23 added `FirKind::FoolRef` — both matches already
  handle it; the new `ConcatHelper` arm is added alongside.
- **Renames**: `get_my_brane` → `_get_my_brane`, `get_my_statement` →
  `_get_my_statement`. `find_parent_brane` becomes a wrapper.
- **FOOP-23 interaction**: `BraneNavigator` (contextful_search module) and
  `contexted_search_from_anchor`'s `brane_len` re-expressed over `stmt_count`/
  `stmt_at`. Anchored search `len` reads in SearchFir/IndexFir/HeadTailFir
  switch to `stmt_count()`.

## UBC Step Impact

`ConcatenationFir` only. Before: `Braning` merges element statements (Rc-shared)
into one new `BraneFir` pushed as the result. After: `Embryonic` drains elements
→ populate _ConcatHelpers → `Braning` drains revived searches → settle by the
terminal rule. Two observable consequences:

1. **Step counts change** for every concatenation program (extra recoordination
   steps + the populate step).
2. **Cross-element references may now resolve** where they previously stayed
   ECONSTANIC (the `{a=10}{b=a}` class), because cloning replaces Rc-sharing.

Both mean `.snap.new` churn in the concatenation snapshots. Every such change
goes through the human review workflow — NEVER auto-accepted — and the semantic
ones are the point of the FOOP.

**Relation to FOOP-3:** this FOOP *implements* FOOP-3's "concatenation produces
constanicCloned elements" clause properly, and *supersedes* its "further steps
delegate to the merged brane" clause — there is no merged brane; further steps
run against the _ConcatHelper statements in place.

## Test Plan

**Snapshot tests are developed FIRST, then unit tests.** The UBCa snapshot suite
runs with `max_brane_size = 13` once the configuration exists (13 is small enough
that modest inputs force real splitting — 200 statements > 13² = 169 forces a
three-level tree — and large enough that existing small-brane snapshots are
untouched). By the Sequencing requirement, snapshot output is k-invariant.

**The new `concat_brane_*` snapshot family REPLACES the existing
`concatenation_*` snapshots.** Retiring the old approved snaps is a HUMAN action
(AI never moves or deletes `.snap` files); the plan gates it accordingly. There
are 14 existing `concatenation_*` snapshots; A4 lists all of them for the
reviewer's retire/keep/regenerate decision.

### Snapshot tests (`foolish-ubca/snapshot_tests/input/`)

**The fenced `foolish` blocks below ARE the snapshot inputs** — complete, valid
programs to be copied VERBATIM into `foolish-ubca/snapshot_tests/input/` at plan
step A1 (byte-for-byte; the spec is the source of truth for these files).

#### `concat_brane_test_basic.foo`

```foolish
{
	b1 = {a=1, b=2};
	named_lit = b1 {c = a + b};
	lit_lit   = {x=1, y=2} {z = x + y};
	with_empty = {} {p=1} {};
	twice = b1 b1;
	twice_twice = twice twice;
   mixed_resolve = {source = b1} {
     field1=source.a,     !! 1
     field2=source.b,     !! 2
     field3=huh,          !! not found
     field4=named_lit.e;  !! not found
     field4=with_empty.p; !! 1
   }
}
```

Expected: `named_lit = {a=1; b=2; c=3}` (the cross-element repair); `lit_lit =
{x=1; y=2; z=3}`; `with_empty = {p=1}` (empty constituents contribute zero
lines); `twice = {a=1; b=2; a=1; b=2}` (a named brane may appear more than once).

#### `concat_brane_foolish_concatenations.foo`

```foolish
{
	flat = {a=1} {b=2} {c=3} {d=4} {e=5};
	deep = { {f=6} {g=7} } { {h=8} { {i=9} {j=10} } } {k=11}
   front = {x=1,y=2} flat;
   back = flat {x=-1,y=-2};
   back = front {middle=0} back
   search=back.x;                    !! -1
   bk_search=back~x;                 !!  1
   fail_search=back?nada;            !! NK
}
```

Expected: `flat` is one brane of five lines in source order. In `deep`, each
braced element is a BRANE whose single anonymous line settles to a joined brane.
The snapshot documents precisely this containment (nested concatenation does NOT
flatten across brace levels — braces are branes; only sibling juxtaposition joins).

#### `concat_brane_split_long_brane.foo`

```foolish
{a1=1, a2=2, a3=3, a4=4, a5=5, a6=6, a7=7, a8=8, a9=9, a10=10,
 a11=11, a12=12, a13=13, a14=14, a15=15, a16=16, a17=17, a18=18, a19=19, a20=20,
 a21=21, a22=22, a23=23, a24=24, a25=25, a26=26, a27=27, a28=28, a29=29, a30=30,
 a31=31, a32=32, a33=33, a34=34, a35=35, a36=36, a37=37, a38=38, a39=39, a40=40,
 a41=41, a42=42, a43=43, a44=44, a45=45, a46=46, a47=47, a48=48, a49=49, a50=50,
 a51=51, a52=52, a53=53, a54=54, a55=55, a56=56, a57=57, a58=58, a59=59, a60=60,
 a61=61, a62=62, a63=63, a64=64, a65=65, a66=66, a67=67, a68=68, a69=69, a70=70,
 a71=71, a72=72, a73=73, a74=74, a75=75, a76=76, a77=77, a78=78, a79=79, a80=80,
 a81=81, a82=82, a83=83, a84=84, a85=85, a86=86, a87=87, a88=88, a89=89, a90=90,
 a91=91, a92=92, a93=93, a94=94, a95=95, a96=96, a97=97, a98=98, a99=99, a100=100,
 a101=101, a102=102, a103=103, a104=104, a105=105, a106=106, a107=107, a108=108, a109=109, a110=110,
 a111=111, a112=112, a113=113, a114=114, a115=115, a116=116, a117=117, a118=118, a119=119, a120=120,
 a121=121, a122=122, a123=123, a124=124, a125=125, a126=126, a127=127, a128=128, a129=129, a130=130,
 a131=131, a132=132, a133=133, a134=134, a135=135, a136=136, a137=137, a138=138, a139=139, a140=140,
 a141=141, a142=142, a143=143, a144=144, a145=145, a146=146, a147=147, a148=148, a149=149, a150=150,
 a151=151, a152=152, a153=153, a154=154, a155=155, a156=156, a157=157, a158=158, a159=159, a160=160,
 a161=161, a162=162, a163=163, a164=164, a165=165, a166=166, a167=167, a168=168, a169=169, a170=170,
 a171=171, a172=172, a173=173, a174=174, a175=175, a176=176, a177=177, a178=178, a179=179, a180=180,
 a181=181, a182=182, a183=183, a184=184, a185=185, a186=186, a187=187, a188=188, a189=189, a190=190,
 a191=191, a192=192, a193=193, a194=194, a195=195, a196=196, a197=197, a198=198, a199=199, a200=200,
 total = a1 + a100 + a200}
```

The final line `total = a1 + a100 + a200` is a deliberate cross-chunk probe: under
k=13 its three operands live in the 1st, 8th, and 16th _ConcatHelper. Expected:
201 lines, `total = 301`, rendered as ONE flat brane — byte-identical unsplit
(Phase A) and split (Phase B, 16 _ConcatHelpers). Approved once in Phase A, this
snap is the Phase B no-churn sentinel.

#### `concat_brane_nested_shadowed_resolution.foo`

```foolish
{
	p = 100;
	shadow = 111;
	orig = {shadow=1, keep=shadow, late=zz};
	cb = {shadow=2, zz=9} orig {from_join=shadow, from_parent=p} <{prepared=shadow, prepared_p=p}>
	
   extended = <{grab_p = p, grab_shadow=shadow}>  {x=cb.shadow} {y=<grab_shadow>} {z=<<grab_shadow>>} <{grab_p_ao = p+1, grab_shadow_ao=shadow+1}> {g1=grab_p_ao, g2=<grab_p_ao>, g3=<<grab_p_ao>>}
}
```

Expected resolutions inside `cb`:

| line | layer pinned | resolves to | why |
|---|---|---|---|
| `keep` | ORIGINAL context | `1` | resolved to `orig`'s own `shadow=1` BEFORE the copy; the copied constant keeps its value |
| `late` | PREFIX constituent | `9` | ECONSTANIC in `orig` (no `zz` there); revives on copy and IB-finds `zz=9` |
| `from_join` | PREFIX, nearest match | `1` | auto-SFF'd literal resolves in the join; nearest preceding `shadow` is `orig`'s copied `shadow=1` |
| `from_parent` | PARENT context | `100` | no `p` in the join; AB search exits the ConcatBrane to the enclosing brane |
| `prepared` | `<{…}>` local preparation | `111` | SF override: resolves BEFORE the copy from `cb`'s own statement position — sees the OUTER `shadow=111` |
| `prepared_p` | `<{…}>` local preparation | `100` | same timing; `p` from the enclosing brane |

The `prepared = 111` vs `from_join = 1` contrast is the heart of the test: the
SAME search pattern (`shadow`) with different per-element timing yields different,
individually correct answers.

### Unit tests

In `foolish-ubca` (tests module of `fir_kinds.rs`); existing concatenation-related
unit expectations regenerate. Required new tests:

- `concat_brane_split_long_brane_hierarchy` — under k=13 the a1…a200 storage is
  16 _ConcatHelpers of ≤ 13 lines, global indices 0..199 in order.
- `concat_equals_big_brane` — same statements as `{s₁…sₙ}` vs `{s₁…s₅}{s₆…sₙ}`
  settle to identical sequenced output (the Law, end to end).
- `concat_search_brane_translates_global_indices` — forward and reverse
  `_search_brane` over a ConcatBrane find the correct statement with the correct
  global index, including hits in the first, middle, and last _ConcatHelper.
- `concat_ib_search_crosses_segments` — `{a=10}{b=a}` resolves `b` to `10`.
- `concat_ab_search_reaches_outward` — a statement inside a ConcatBrane resolves
  a name defined in the enclosing brane.
- `concat_index_spans_segments` — `#9` into 5+5 finds the last statement; `#-1`
  the same; head/tail across a boundary; out-of-range → NK.
- `concat_find_stmt_index_is_global` — identity scan returns global indices.
- `concat_element_typing_rejects_non_brane` — non-brane/non-search direct element
  → alarm + NK at construction; element resolving to a non-brane at settle time →
  alarm + NK.
- `concat_construction_auto_wraps` — literal elements SFF-wrapped (searches BORN
  ECONSTANIC), search elements SF-wrapped.
- `concat_cross_element_reference_resolves` — `{cb = {a=1, b=2} {c = a + b};}` →
  `c = 3`.
- `concat_sff_born_searches_revive_embryonic` — copy transforms ECONSTANIC →
  EMBRYONIC in position with correct parents.
- `concat_sf_on_search_is_noop` — `<search>` ≡ bare search.
- `concat_sf_marked_literal_prepares_locally` — `<{x = name_from_concat_context}>`
  resolves BEFORE copy, from the concat's own statement position.
- `concat_explicit_sff_element_is_error` — `<<{…}>>` → alarm + NK.
- `concat_statement_parents_point_at_concat_helper` — line.parent =
  _ConcatHelper; _ConcatHelper.parent = ConcatBrane; `get_my_brane` from a line
  returns the ConcatBrane (walks through _ConcatHelper).
- `concat_value_is_itself` — `settled_result()` of a ConcatBrane is `None`, so
  `value()` of a settled ConcatBrane is itself.
- `concat_constanic_clone_rewires_and_recoordinates` — cloning a settled
  ConcatBrane deep-clones _ConcatHelper storage via
  `skip_foolish_children = true`, rewires parents to the clone, preserves
  numbering/arrangement; NO element FIRs cloned.
- `settled_search_clone_skips_foolish_children` — settled SearchFir clone with
  the option drops the anchor subtree.
- `concat_arrangement_is_function_of_n_and_k` — nested ConcatBrane element
  contributes its lines like any brane; unlimited k → single _ConcatHelper; n=k²
  and n=k²+1 boundaries.
- `concat_helper_nyes_transitions` — PREMBRIONIC start, monotone, constanic
  terminal (per AGENTS.md rule for new FIR kinds).
- `concatenation_nyes_transitions` — extended for the populate-then-drain
  progression.

Auto-sizing (compiler, Phase B):
- `unlimited_config_is_identity`, `brane_at_or_under_max_is_not_split`,
  `oversized_brane_splits_into_chunked_concatenation` (5 statements, k=2 → chunks
  2,2,1), `root_brane_is_never_split`, `characterized_brane_is_never_split`,
  `split_brane_settles_to_same_result_as_unsplit` (includes a cross-chunk
  reference).
- `iterative_grouping_bounds_every_node` — n=30, k=3: NO node holds more than 3
  children; order preserved; settles identically to unlimited. Boundary cases
  n=k² and n=k²+1.

## Rejected Alternatives

### A. Do nothing
Keeps the merge, which structurally violates any brane-size bound and keeps the
Rc-sharing defects.

### B. AST-level chunking only, keeping the merging ConcatenationFir
Self-defeating: the compiler splits an oversized brane into chunks, and the
concatenation's own step immediately reassembles the oversized brane.

### C. A public SubBrane FIR kind in the parent chain (with custom resolution)
Rejected for complexity: statements' parents would point at their chunk, so every
resolution path needs forwarding logic, and ~14 kind-match sites need three-way
decisions. **The _ConcatHelper design in this FOOP avoids this by making the
carrier transparent** (not brane-like, not a statement) — `get_my_brane` walks
through it to the ConcatBrane, which owns all resolution. No forwarding, no
three-way matches. (This is a deliberate departure from the earlier "SubBrane as
participant" sketch: transparency eliminates the IB/AB asymmetry that a
participating segment would create — see Appendix C.)

### D. Segments in a dedicated struct field instead of `ubc_children`
Breaks FOOP-62's two-store uniformity. The store is kind-interpreted by design;
the ConcatBrane using it for _ConcatHelper storage is intended usage.

### E. Split evaluated BraneFirs at step time
Touches every step rule and can oscillate against concatenation. Out of scope.

### F. Physical stack ConcatBrane internal nodes
A `ConcatenationFir` used as a storage-mode internal node would hit the
empty-`foolish_children` branch of `fir_op_step` and corrupt the tree. The
**flat Vec of _ConcatHelpers** avoids this entirely — there are no internal
ConcatBrane nodes; the "k-ary tree" is the logical access pattern via prefix sums.

### G. Shape-preserving adoption of nested-ConcatBrane elements
Superseded by count-and-arrange: after constituents settle, element boundaries
carry no semantics — the arrangement is the pure function of `(n, k)`.

## Open Questions

- Should characterized branes eventually split, with characterizations carried by
  the ConcatBrane? Deferred until characterizations have merge semantics.
- Should the root brane's statement list be bounded too? Deferred.
- **Auto-SF/SFF wrapper hygiene**: (a) the step driver sets
  `with_ancestral_sfm(true)` when stepping under a StayFoolish — the auto-SF
  wrappers must not accidentally change the NYES transform of copied lines;
  (b) the sequencer must render elements as the Foolisher wrote them — the
  auto-wrappers are construction bookkeeping and must be invisible in output.

## Future Work (TODO)

- **Multi-search**: returns ALL matches (not first), result represented as a
  brane, materialized via brane concatenation. Needs its own FOOP.
- **The Integer brane**: a THIRD kind of brane, concat-able, supplying
  integers-as-identifiers and arithmetic operators, delivered as
  `Concat(IntegerBrane, program_brane)`. Needs its own FOOP; this FOOP keeps the
  populate step's adoption rule capability-based.

## References

- Prior FOOPs: FOOP-3 (partially superseded), FOOP-7 (constanic clone
  recoordination), FOOP-62 (two-store ProtoBrane, root convention, NYES
  ownership; root convention and `under_sff` build rule are in FOOP-62.plan.md
  tasks #17–#18, semantics in FOOP-62.md §10.1), FOOP-23 (value search +
  contexted `&`-searches; `FoolRefFir`, `push_search_result_pair`, the
  two-child `ubc_children` invariant, and the `ContextfulSearch` engine with
  `BraneNavigator` — merged to `jia`; status Draft).
- Code locations: `foolish-ubca/src/fir_kinds.rs` (`ConcatenationFir`,
  `BraneFir::_search_brane`, `StatementFir::_ib_search`,
  `FirRefNavExt::{index_into, find_stmt_index}`, `find_parent_brane`,
  `push_search_result_pair`, `FoolRefFir`, `contextful_search` module with
  `BraneNavigator`), `foolish-ubca/src/fir_trait.rs` (`Fir` defaults,
  `get_my_brane`, `FirRefExt::value`, `step_inner`), `foolish-ubca/src/
  proto_brane.rs` (constanic clone, `all_children`), `foolish-ubca/src/
  evaluator.rs` (`UbcaEvaluator`, `proto_to_core_fir`), `foolish-ubca/src/
  compiler.rs` (`Compiler::compile`, `validate_astn`, `build_fir`),
  `foolish-parser/src/ast.rs` (`Astn` enum — cross-crate).
- Snapshots exercising concatenation: 14 `concatenation_*` files under
  `foolish-ubca/snapshot_tests/approved/` (all listed for retire/keep/regenerate
  in plan A4).

## Appendix A — Brane interface enumeration

The brane's query interface toward its children (beyond holding/stepping) is:
**IB search, AB search, indexing (anchored and unanchored), identity scan,
contexted search navigation.** The methods:

| Method | Signature | Who calls | BraneFir today |
|---|---|---|---|
| `_search_brane` | `(&self, expr, start, end) -> Option<(usize, FirRef, Nyes)>` | `StatementFir::_ib_search` (IB), anchored search arm | scans `foolish_children[start..=end]` |
| `_ab_search` | `(&self, self_ref, name) -> Option<(FirRef, Nyes)>` | `Fir::_ab_search` default, `BraneFir` override | walk to enclosing stmt → IB → parent brane |
| `index_into` | `(offset: i32) -> Option<(FirRef, Nyes)>` | IndexFir, HeadTailFir (anchored) | `foolish_children[idx]` → body |
| `find_stmt_index` | `(stmt) -> Option<usize>` | IndexFir, HeadTailFir (unanchored), `contexted_search_from_anchor` (FOOP-23) | identity scan over `foolish_children` |
| `index_into_brane_relative` | `(brane, stmt_idx, offset)` free fn | IndexFir, HeadTailFir (unanchored) | `foolish_children[stmt_idx + offset]` |
| `BraneNavigator` | (private, `contextful_search` module) | `contexted_search_from_anchor` (`&?`/`&~`/`&#`/`&^`/`&$`) | reads `foolish_children()` → `Vec<FirRef>` |

ConcatBrane overrides `_search_brane` (global→local over _ConcatHelpers),
`_ab_search` (shared with BraneFir), `stmt_count`/`stmt_at` (tree walk). All
indexing methods are re-expressed over `stmt_count`/`stmt_at` in A2.
`BraneNavigator` is re-expressed over `stmt_count`/`stmt_at` (builds its
candidate `Vec` from `stmt_at` instead of `foolish_children`). The `brane_len`
reads in `contexted_search_from_anchor` and the anchored search arms switch to
`stmt_count()`. _ConcatHelper implements NONE of these (transparent).

## Appendix B — Step driver semantics

The driver (`step_inner`, `fir_trait.rs:317`):

- `fir_op_step` is called ONLY when the node's own task queue is empty.
- The driver only drains `tasks` (the `VecDeque`); it never enumerates
  `ubc_children` directly.
- `push_ubc_child` enqueues non-constanic children as tasks (constanic children
  are stored but never stepped).
- A task is popped only when it becomes constanic.

This means: a _ConcatHelper pushed via `push_ubc_child` while non-constanic (its
revived searches need draining) IS enrolled and drained by the unaltered driver.
A _ConcatHelper pushed while constanic is stored but never stepped. Both are
correct for the protocol. The three-phase progression (Embryonic drain →
populate → Braning drain → settle) falls out naturally — the discriminator is
`ubc_children` emptiness (empty → populate; non-empty → settle).

## Appendix C — The three brane-finding mechanisms

The codebase has three mechanisms for "find my home brane":

1. **`.parent`** (raw `Weak` link, `proto_brane.rs:34`) — the ground-truth
   structural parent. Fixed at construction; walked by the methods below.
2. **`_get_my_brane`** (trait method, parent-walk) — climbs `.parent()` until a
   brane-like kind. Used by IB/AB search (`StatementFir::_ib_search`,
   `Fir::_ab_search`). Renamed from `get_my_brane` in A2.
3. **`Scope.current_brane`** (cached, set by `step_inner`) — set ONLY when
   `this_kind == FirKind::Brane`. Used by `ab_search(scope, …)` and
   `ib_search(scope, …)`.

The inconsistency: the `_`-prefixed variants parent-walk; the non-prefixed
variants read the scope cache. These can disagree if the scope cache is stale or
set by a different kind-match. **The _ConcatHelper design eliminates this
asymmetry**: _ConcatHelper is neither `FirKind::Brane` (so scope isn't set to
it) nor brane-like by capability (so `_get_my_brane` walks through it). Both
mechanisms reach the ConcatBrane. A2 unifies `find_parent_brane` into a wrapper
over `_get_my_brane` so there is one implementation of the walk.

## Last Updated

**Date**: 2026-07-06
**Updated By**: Sisyphus / z-ai/glm-5.2
**Changes**: Tenth revision — restructured exposition (definitions section,
tightened flow). _ConcatHelper design: new `FirKind::ConcatHelper` carrier
(neither brane nor statement, transparent to resolution); flat Vec storage
(replaces k-ary tree of stack ConcatBranes — avoids fir_op_step misfire);
uniform parent chain (no bypass); three-phase protocol (Embryonic drains
elements → populate → Braning drains _ConcatHelpers → settle, discriminated by
ubc_children emptiness). Added §"Labeling and discipline" (rename
get_my_brane→_get_my_brane, document iterative call chains, unify
find_parent_brane). Added Appendices A (brane interface), B (step driver), C
(three brane-finding mechanisms). Updated Rejected Alternatives (C and F
rewritten for _ConcatHelper transparency and flat storage).

**Date**: 2026-07-06
**Updated By**: Sisyphus / z-ai/glm-5.2
**Changes**: Updated in view of FOOP-23 merge to `jia`. Added §"FOOP-23
interaction" (BraneNavigator re-expression over stmt_count/stmt_at;
contexted_search_from_anchor brane_len fix; two-child invariant interaction
with ConcatBrane — FoolRefFir wraps original statement inside _ConcatHelper,
contexted search chain works IF find_stmt_index and BraneNavigator are
re-expressed). Updated FIR Impact (FirKind::FoolRef variant exists; new FOOP-23
trait methods noted). Updated References (added FOOP-23; added
push_search_result_pair, FoolRefFir, contextful_search module to code
locations). Updated Appendix A (added BraneNavigator to the brane interface
table; added contexted search to the caller list). Fixed _search_brane
description (removed parenthetical artifact).
