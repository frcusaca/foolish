---
foop: 31
title: MAX_BRANE_SIZE — auto-sizing via a non-merging ConcatBrane equivalent to the merged brane
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

Each UBCa FVM gains a configuration value `MAX_BRANE_SIZE`. During brane construction from the
AST, the compiler automatically converts any brane whose statement count exceeds `MAX_BRANE_SIZE`
into a concatenation of smaller branes, each within the limit. For that to be possible at all,
`ConcatenationFir` must stop merging its elements into one big brane (the merged brane would
itself violate the limit). Instead it becomes a **ConcatBrane**: a brane-like container that
constanic-clones its elements' statements into hidden, bounded segments and answers every brane
operation — search, IB/AB resolution, indexing, sequencing, cloning — through offset arithmetic
over the segment series. The governing law:

> **The Equivalence Law.** A settled ConcatBrane is observationally identical to a single big
> brane containing every statement of its element branes, in the same order — for every brane
> operation — except that the big brane is never materialized.

The work is **two phases**: **Phase A — the ConcatBrane upgrade** (the Equivalence Law made
true for source-level concatenation; a semantic repair in its own right, since today's merge
shares statements by `Rc::clone` without recoordination or parent rewiring), then **Phase B —
the MAX_BRANE_SIZE limit** (configuration plus the auto-sizing rewrite). The default
configuration is unlimited, so Phase B changes nothing unless configured. Note the limit is
self-referential: chunking `n` statements yields `⌈n/k⌉` chunk branes, and that *element array*
may itself exceed `k` — so chunking iterates, grouping elements into nested concatenations until
every node fits: a k-ary concatenation tree. Phase A's design must therefore handle nested
ConcatBrane elements natively.

## Motivation

Branes are the unit of containment, cloning, and recoordination. Foolish branes are by design
finite in size. This foop implements that fact by setting a uniform finite size limit for branes
in the UBCa fvm implementation.`MAX_BRANE_SIZE` bounds the granule: no `BraneFir` statement store ever
exceeds `k` statements; a ConcatBrane is an unbounded *view* over bounded units.

The current `ConcatenationFir` defeats that bound by construction: its `Braning` arm concatenates
every element's statements into one merged `BraneFir` — exactly the oversized brane the
configuration forbids. It also has two latent defects the redesign fixes:

- The merged brane's statements are `Rc::clone`d, not constanic-cloned: their `.parent` links
  still point into the original element branes, and no recoordination happens — an element like
  `{b=a}` that settled ECONSTANIC before combination never gets the chance to resolve `a` against
  an earlier element (`{a=10}{b=a}`).
- Element identity is lost: the merged brane is a fresh unbounded statement list with no memory
  of the units it came from.

After this FOOP: UBCa has a per-FVM configuration surface, oversized branes are transparently
chunked at construction, concatenation preserves bounded units internally, cross-element
references resolve through genuine constanic-clone recoordination, and the observable result
obeys the Equivalence Law.

## Specification

### Phasing

The specification is implemented in two strictly ordered phases:

- **Phase A — ConcatBrane upgrade.** `ConcatenationFir` stops merging and satisfies the
  Equivalence Law over a hidden storage tree, including nested-ConcatBrane elements. No
  configuration involved; source-level concatenation semantics are repaired. Phase A stands
  alone and is reviewed (snapshot churn included) before Phase B begins.
- **Phase B — MAX_BRANE_SIZE.** The `UbcaConfig` surface and the iterative auto-sizing rewrite.
  Depends on Phase A: the rewrite emits exactly the nested-concatenation shapes Phase A must
  already evaluate correctly.

### Size

The **size of a brane** is its number of statements. Nested content does not count toward the
size of the outer brane; each brane is measured on its own statement list.

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

- `UbcaEvaluator` changes from a unit struct to `pub struct UbcaEvaluator { pub config: UbcaConfig }`
  with `Default`.
- The compiler gains `Compiler::compile_with(source: &str, config: &UbcaConfig)`;
  `Compiler::compile(source)` delegates with `UbcaConfig::default()`.
- `NonZeroUsize` makes the degenerate `MAX_BRANE_SIZE = 0` unrepresentable; "no limit" is spelled
  `None`, not `0`.

### The auto-sizing rewrite (AST→AST) — Phase B

Applied inside the compiler after `validate_astn` and before `build_fir`. Let
`k = max_brane_size`. The rewrite recurses structurally through every `Astn` variant; the only
rewriting arm is `Astn::Brane` with `n = statements.len()`:

1. Recurse into each statement first (nested branes are auto-sized independently).
2. If `n <= k`, or the brane is **exempt** (below), leave it as-is.
3. Otherwise split `statements` (order preserved) into `m = ⌈n/k⌉` consecutive chunks — every
   chunk holds exactly `k` statements except possibly the last — giving the element array
   `[Brane(chunk₁), …, Brane(chunkₘ)]`, each chunk brane with empty characterizations.
4. **Iterate until it fits**: while the element array is longer than `k`, group it (order
   preserved) into consecutive runs of ≤ `k` elements, wrapping each run in an
   `Astn::Concatenation`; the runs become the new element array. When the array is ≤ `k`
   elements, the result is a single `Astn::Concatenation` over it.

The result is a k-ary concatenation tree of depth `⌈log_k m⌉ + 1`: leaves are chunk branes of
≤ `k` statements, internal nodes are concatenations of ≤ `k` elements. The bound is uniform —
**no node in the constructed program holds more than `k` children**, whether those children are
statements or concatenation elements.

Exemptions (never split): the **root brane** (only a Brane may be root — FOOP-62 root
convention; its nested branes still auto-size) and **characterized branes** (a ConcatBrane
carries no characterizations; conservative until characterizations gain merge semantics).

### The ConcatBrane redesign — Phase A

#### Two public brane kinds — no SubBrane FIR kind

There are exactly two brane-like FIR kinds visible to the resolution machinery: `BraneFir` and
`ConcatenationFir` (the ConcatBrane). The chunks inside a ConcatBrane — **SubBranes** — are a
ROLE, not a new FIR kind: plain `BraneFir`s used as hidden storage, **never exposed through the
parent chain**. This is what keeps the change local: name resolution walks parents to "the
brane" and asks it questions; if the answers obey the Equivalence Law, nothing upstream needs a
forwarding protocol.

#### Hidden storage tree (two-store placement)

The hidden storage is a **k-ary tree** held in the ConcatBrane's `ubc_children` (storage is a
compute-time result — exactly what `ubc_children` is for, per FOOP-62's two-store design).
`foolish_children` remain the original, parse-time element FIRs, untouched. A storage node is
either:

- a **SubBrane** — a plain `BraneFir` holding ≤ `k` constanic-copied lines (leaf), or
- a **stack ConcatBrane** — a `ConcatenationFir` used as storage, holding ≤ `k` child storage
  nodes (internal node: "stacks of ConcatBranes of ConcatBranes until SubBranes").

**The tree shape is a pure function of the total line count `n` and `k`** — computed at
populate time, independent of element boundaries (a nested-ConcatBrane element contributes its
lines like any other brane; no element shape is inherited). Unlimited `k` degenerates to a
single SubBrane holding all `n` lines. Storage nodes never resolve names and never appear in
the parent chain. No new FIR kind: both node roles reuse the two existing kinds as inert
containers. Deliberate deviations from ordinary children:

- **Statement parents bypass the whole tree**: every line copied into any SubBrane has
  `.parent = the top (public) ConcatBrane`, no matter how deep its SubBrane sits. Storage
  nodes' own parents also point at the top ConcatBrane. Since no line's parent points at a
  storage node, the storage is invisible to `get_my_brane` / `get_my_statement` walks — only
  the top ConcatBrane exists as far as resolution is concerned.
- **Global line numbers**: each copied line's `line_number` is rewritten to its global index
  across the entire tree, in order. This single rule makes `StatementFir::_ib_search`
  (`brane._search_brane(name, line_number − 1, 0)`) work UNCHANGED for cross-SubBrane IB —
  the joined lines chain their IB searches exactly as if written in one brane.
- Offsets are not stored; they are prefix sums over SubBrane line-counts, computed on access
  (fan-out ≤ `k`, depth logarithmic). Empty constituents contribute zero lines
  (`concatenation_of_empty_branes` case).

#### The ConcatBrane protocol (replaces the merge)

**Fundamental contract: a ConcatBrane takes a list of BRANES and concatenates them.** The
contract is enforced at every phase; a violation raises an alarm and labels the ConcatBrane
`NK` — it no longer steps.

1. **Element typing — construction-time first pass.** Every direct element of
   `foolish_children` must be one of:
   1. a Foolish brane that was typed out (a literal), e.g. `{asdf=1}`;
   2. a search expression that could resolve to a brane — plain (`b`), anchored (`b.x`),
      regexp (`c~something`), or positional; or
   3. an **explicitly SF-marked expression** `<…>` whose inner expression is itself form
      (1) or (2).

   Any other FIR kind — and any **explicitly SFF-marked (`<<…>>`) element**, which is an
   error — raises an alarm and the ConcatBrane is constructed `NK`. (The parser may reject
   earlier where the grammar already knows.)
2. **Auto-wrapping at construction.** ConcatBrane construction normalizes every element to a
   wrapped form:
   - a bare **literal brane** is wrapped in **SFF**: reusing the existing `under_sff` build
     rule (FOOP-62 #17), every search inside it is BORN ECONSTANIC and never resolves
     standalone — the literal's innards are deferred to the join (resolve AFTER);
   - a bare **search** is wrapped in **SF**: it resolves BEFORE (it identifies the
     constituent), and the found brane's lines detach-and-recoordinate at copy time;
   - an **explicit SF around a search** is **idempotent** — a NOOP, identical to the auto-SF;
   - an **explicit SF around a literal brane** is an **override of the automated SFF wrapping**
     that would otherwise apply to that constituent — same protocol, different default. The
     override selects **local preparation**: the literal's internal searches resolve BEFORE the
     copy, from the
     ConcatBrane's own statement position — IB against the lines preceding the concat's
     statement in the enclosing brane, AB through the concat's ancestors — exactly as if the
     brane were written standalone at the concat's source location. Its settled lines then
     copy with standard recoordination. Needs special care in implementation: the element must
     NOT be built `under_sff`, and its pre-copy stepping must run against the concat's own
     context (the ordinary parent chain element → ConcatBrane → statement → enclosing brane
     already provides it).

   The full timing menu is therefore expressible per element, with no new syntax:

   | element as written | wrapping | timing of its searches |
   |---|---|---|
   | `{…}` bare literal | auto-SFF (default) | innards resolve AFTER, in position in the join |
   | bare search (`b`, `b.x`, `c~…`) | auto-SF | resolves BEFORE (identifies the constituent); lines recoordinate on copy |
   | `<search>` | idempotent NOOP | same as bare search |
   | `<{…}>` | SF overrides the auto-SFF | innards resolve BEFORE, locally, from the concat's own statement position, then copy |
   | **all other FIRs** (incl. explicit `<<…>>`) | — | error: alarm + `NK`; the ConcatBrane never steps |
3. **Step to constanic.** `PREMBRIONIC`/`EMBRYONIC`: empty element list settles as today (empty
   constant brane); otherwise every element is pushed as a task and stepped until constanic.
   SFF-wrapped literals settle without any internal resolution; SF-marked literals settle WITH
   full internal resolution in the concat's own context.
4. **Settle-time typing — second pass.** Each element's value must be a brane; anything else
   (a search that resolved to an integer, an NK element): alarm + `NK`.
5. **Count and arrange.** Count the total lines `n` across all constituent values, in order.
   Compute the storage arrangement as the pure function of `(n, k)`: lines chunk into SubBranes
   of ≤ `k`; while the node array exceeds `k`, stack ConcatBranes over runs of ≤ `k` nodes,
   until the top fits.
6. **Constanic-copy and coordinate.** Each line is constanic-copied to its SubBrane position
   and coordinated THERE: parent rewired (bypass to the top ConcatBrane), global line number
   assigned, NYES transformed per the clone rules — in particular the SFF-born ECONSTANIC
   searches revive **ECONSTANIC → EMBRYONIC** with correct parents, ready to search. Lines from
   SF'd elements — searched branes AND locally-prepared SF-marked literals alike — recoordinate
   per FOOP-7: constants keep their values, ECONSTANIC revives, NK stays NK. Non-constanic
   copies enter the task queue; remain `BRANING` until drained.
7. **Terminal rule** (unchanged): any NK → `NK`; any ECONSTANIC/WOCONSTANIC → `WOCONSTANIC`;
   else `CONSTANT`.

**Worked example.** `{cb = {a=1, b=2} {c = a + b};}` — both literals are auto-SFF'd, so
`c = a + b`'s searches are born ECONSTANIC and skip standalone resolution; the elements settle
immediately; `n = 3`; the lines copy into position; `c`'s searches revive EMBRYONIC and IB-find
`a` and `b` in the joined brane: `c = 3`. (Verified 2026-07-04 against the current evaluator:
today this program leaves `c` UNRESOLVED — the constituents settle standalone, `a`/`b` are not
findable from inside `{c=a+b}`, and nothing revives after the merge. This protocol is the
repair.) Likewise `{cb = {1,2,3} a {r=1,s=2,t=3} d e f g {asdf = #-1 + t + r};}`: `a d e f g`
resolve BEFORE (auto-SF'd element searches); `#-1`, `t`, `r` resolve AFTER, in position —
`#-1` addresses `g`'s contribution, `t`/`r` IB-find the earlier element — with no manual
wrappers and no wrong-context NK. Third example, exercising the full element menu:

```foolish
{a=1} b~child_brane ancestrally_defined_brane <sf_mark_around_search> <{locally_prepared_brane = field_searched_from_parent_of_concat}>
```

The first three elements take the defaults (literal auto-SFF'd; two searches auto-SF'd). The
fourth, an explicit SF around a search, is the idempotent NOOP. The fifth, an SF-marked literal,
is local preparation: `field_searched_from_parent_of_concat` resolves BEFORE the copy, from the
ConcatBrane's own statement position (here, ancestrally through the concat's parent) — the
opposite timing from the auto-SFF default, chosen per element by the Foolisher. An explicitly
SFF-marked element (`<<{…}>>`) is an error: alarm + `NK`.

#### Capability dispatch instead of kind matching

How two brane types "generically, dynamically" share operations: through `dyn Fir` trait methods
— the mechanism `_search_brane` already uses (default `None` on `Fir`, overridden by branes).
New/changed `Fir` methods:

```rust
/// Number of statements this FIR presents as a brane. `None` = not brane-like.
fn stmt_count(&self) -> Option<usize>;          // Brane: foolish_children.len(); ConcatBrane: Σ over tree
/// The statement at a global index, per the Equivalence Law.
fn stmt_at(&self, idx: usize) -> Option<FirRef>; // Brane: foolish_children[idx]; ConcatBrane: tree descent
/// The settled result this FIR resolves to, if any. Each kind interprets its own
/// ubc_children; the default preserves today's behavior for the result-style kinds.
/// CONTRACT: applies the constanic gate ITSELF — pre-constanic always answers None
/// (a mid-flight search may hold a pending entry in its store; that is not yet a
/// result). None therefore means "nothing to resolve into": either not settled yet,
/// or settled and I AM my own value. It can never return "self" — a pointee method
/// cannot mint its own FirRef; the CALLER holding the handle substitutes self.
fn settled_result(&self) -> Option<FirRef> {
    if !self.core().get_nyes().is_constanic() {
        return None;
    }
    self.core().ubc_children().into_iter().next()
}                                                // ConcatBrane: None — it IS its value
```

- `ConcatenationFir::_search_brane(expr, start, end)`: map the global, direction-aware range
  (the same reversed-range convention `BraneFir::_search_brane` uses) onto per-bag local ranges
  via prefix sums, recursing down stack ConcatBranes; delegate the leaf scan; translate the hit
  index back to global.
- `ConcatenationFir::_ab_search`: identical logic to `BraneFir::_ab_search` (enclosing
  statement's IB, then parent brane) — shared via a free function or default method, not
  duplicated.
- `Fir::as_i64` needs NO override: a brane never yields an integer, so the default chain
  (first ubc child's `as_i64`) reaches a bag — a brane — and returns `None` naturally.

**`FirRefExt::value`** is re-expressed over `settled_result()` and becomes trivial: `Some(r)` →
recurse into `r`; `None` → return the handle itself (the FIR is its value — settled or not; the
constanic gate already lives inside `settled_result()`). Today `value()` hard-codes
"constanic + non-empty `ubc_children` ⇒ resolve to `ubc_children[0]`" — a fact about the
result-style kinds (Search, Index, Operator) written as if it were a law of ProtoBrane. It is
not: `ubc_children` is the general compute-time store, stepped by the shared machinery, and each
kind interprets its own contents (see Doctrine correction below). The default `settled_result()`
preserves current behavior for every existing kind; ConcatBrane answers `None` because its
ubc_children are storage, not a result chain.

#### Doctrine correction: `ubc_children` is kind-interpreted

`ubc_children` is the compute-time child store. The stepping machinery steps what is pushed
there (`push_ubc_child` enqueues non-constanic entries as tasks); **what the settled contents
MEAN is decided by each FIR kind**. Search/Index/HeadTail hold at most one entry that is the
result (the SINGULAR-RESULT INVARIANT — correctly scoped to search FIRs on
`ProtoBrane::push_search_result`, and likewise in FOOP-62 §8); ConcatBrane holds its storage
tree; future kinds may hold something else. Documentation that states or hints a universal
"ubc_children[0] is the value" rule is wrong and is corrected by this FOOP:

- `FirRefExt::value` doc comment (`fir_trait.rs`): "walks the chain until it reaches a terminal
  value (one with no ubc_children, like IndepInt, Nk, or BraneFir)" — equates having
  ubc_children with being a result wrapper. Rewrite around `settled_result()`: a FIR is terminal
  when its kind reports no settled result, not when the store happens to be empty.
- `Fir::as_i64` doc comment (`fir_trait.rs`): "Default: look through ubc_children for resolved
  results" — rephrase to "delegates to `settled_result()`-style resolution; kinds that are not
  integer-valued yield None."
- `ProtoBrane::all_children` doc comment (`proto_brane.rs`): "ubc first (result=), then foolish"
  — the `result=` aside describes the sequencer's rendering of result-style kinds, not a
  property of the store; scope the comment accordingly.
- Sweep `foolish-ubca` (code comments and `docs/`) for further "ubc = result" phrasing during
  Phase A and correct in place. `proto_brane.rs`'s store description ("search results, etc.")
  and FOOP-62's singular-result scoping are already correct and need no change.

**Kind-match conversion.** Every site that hard-matches `FirKind::Brane` to mean "a brane-like
thing" switches to the capability (`stmt_count().is_some()`, or an `is_brane_like()` helper on
`FirKind`): `get_my_brane` (fir_trait.rs), `find_parent_brane` and the SearchFir anchored arm
(fir_kinds.rs), `step_inner`'s `current_brane` assignment (fir_trait.rs), and the
`proto_to_core_fir` bridge sites in evaluator.rs. Sites that genuinely mean the `BraneFir` kind
(e.g. FIR construction) stay as-is.

#### Indexing

`FirRefNavExt::index_into` and `find_stmt_index` currently read `foolish_children` directly —
on a ConcatBrane that is the *element list*, not the statement series, so `cat.#9` would index
garbage. Both are re-expressed over `stmt_count`/`stmt_at`, which makes them correct for both
brane kinds by construction:

- `index_into(offset)`: non-negative counts from the global front, negative from the global
  back; `#9` into a concatenation of two 5-statement branes lands on segment 2, local index 4 —
  the last statement. Out of range → `None` → NK, as today.
- `find_stmt_index` returns the global index (identity scan across segments).
- `index_into_brane_relative` (unanchored `^#-n`) and `HeadTailFir` route through the same
  accessors and need no further changes.

#### Parent links (normative summary)

| node                              | `.parent`             |
|-----------------------------------|-----------------------|
| element FIRs (`foolish_children`) | ConcatBrane (as today)|
| bags (segments and inner bags), any depth | the top ConcatBrane |
| statements inside any bag         | **the top ConcatBrane** (bypassing the whole tree) |
| bodies cloned OUT by search/index | the addressing FIR's context, via `constanic_clone_at` (as today) |

#### Constanic-cloning a ConcatBrane — `skip_foolish_children`

When a settled ConcatBrane is itself the found result (e.g. `x = c` where `c` names a
concatenation), the clone copies **just the `ubc_children`** — the settled storage tree — via a
new general clone option:

```rust
constanic_clone(…, skip_foolish_children = true)
```

The element FIRs in `foolish_children` are NOT cloned and the element searches NEVER re-run:
a settled ConcatBrane clones as a **value**, exactly like any other settled brane, not as a
formula re-executed in the destination context. (This resolves the auto-SF reuse question —
destination-context shadowing cannot change the constituents of an already-settled
concatenation. Re-execution semantics, if ever wanted, would be a Foolisher-visible SF on the
concat's own assignment, a separate concern.) The clone must still: deep-clone the storage tree
(lines included), rewire the cloned lines' parents to the NEW ConcatBrane clone, preserve
global line numbers and arrangement, and transform NYES per the standard clone rules — so
previously-ECONSTANIC lines may recoordinate against the destination context.

**The option is general, not ConcatBrane-private.** Every singular-result kind is in the same
position once constanic: a settled `SearchFir` (and `IndexFir`/`HeadTailFir`) holds its result
in `ubc_children` and no longer needs its original `foolish_children` (the anchor expression
it was built from). Cloning such a settled FIR with `skip_foolish_children = true` copies the
result without dragging the dead anchor subtree along — smaller clones, less work, same
observable behavior. Implemented in this FOOP: the option itself plus the ConcatBrane use are
Phase A scope; adopting it in the settled-search clone path is included as a Phase A cleanup
once the option exists.

#### Sequencing (REQUISITE): a ConcatBrane displays as a single brane

Sequencing is not cosmetic here — it is the observable half of the Equivalence Law, and the
snapshot strategy below depends on it. Normative rules:

- **A settled ConcatBrane sequences as ONE brane.** `proto_to_core_fir` walks the storage tree
  in global order and emits a single brane: one pair of enclosing braces, every line in
  sequence. No SubBrane boundaries, no stack-node structure, no element boundaries, and none of
  the auto-SF/SFF construction wrappers appear in the output. The rendering is byte-identical
  to the equivalent big brane's rendering.
- **Consequence — k-invariance.** Because the display flattens, the same program sequences
  byte-identically under ANY `MAX_BRANE_SIZE`, split or unsplit. This is what lets a snapshot
  approved in Phase A (unsplit) stand unchanged through Phase B (split): the snapshot IS the
  Equivalence Law's observable test, and any k-dependent output difference is a bug.
- **Pre-constanic and NK rendering.** A concat that has not settled renders its elements as
  written (as today); an NK ConcatBrane renders as NK with its alarm reason. Lines that settled
  ECONSTANIC/WOCONSTANIC display their states exactly as the same lines would in a plain brane.

#### The Integer brane (informative — a consumer of this design)

The ConcatBrane upgrade is the enabling mechanism for the **Integer brane**, documented here as
a design driver; its implementation is a follow-on FOOP.

The idea: numbers like `10` stop being lexical literals and become **identifiers**. The Integer
brane is a **custom-built brane** — not compiled from Foolish source — that, when asked, returns
what the identifier `10` means. It also carries the arithmetic operators, so that

```foolish
{r = {1,2} plus}
```

resolves `plus` the same way any name resolves, supplying Foolish with the very *idea* of
"integer" and "integer arithmetic" through name resolution rather than through built-in syntax.

Programs then run as:

```text
Concat( IntegerBrane, program_brane )
```

Prepending via concatenation is the whole delivery mechanism: the program's statements sit
AFTER the Integer brane's in global order, so ordinary backward IB search — with zero new
resolution machinery — falls through the program's own statements into the Integer brane. What
today would be a miss (ECONSTANIC on `10` or `plus`) becomes a hit in the prepended context.

The Integer brane cannot pre-store every integer; it invokes the **Creation Postulate**
(`docs/why/creation_postulate.md` — you can always create something new) to create the idea of
each integer on demand, at the moment it is first asked. This slots into the FOOP's
architecture at exactly the seams it defines:

- **The Integer brane is a THIRD KIND of brane** — distinct from the normal Brane and the
  ConcatBrane — **but concat-able exactly like them.** After this FOOP, "being a brane" means
  implementing the trait surface (`_search_brane`, `stmt_count`, `stmt_at`, `_ab_search`,
  `settled_result`) — not being the `BraneFir` struct. The Integer brane is a third implementor
  whose answers are COMPUTED (create-on-ask) instead of stored. Nothing in the resolution
  machinery can tell the difference; that is the point.
- **Concat-ability of a third kind constrains the populate step.** `Concat(IntegerBrane,
  program_brane)` only works if ConcatBrane accepts ANY brane-like element, not just the two
  kinds this FOOP ships. The populate step's adoption rule must therefore be capability-based,
  not kind-matched: a plain statement-list brane clones into a segment; anything else
  brane-like is adopted as a bag THROUGH ITS TRAIT SURFACE (queries delegate to it; a
  generative brane is never flattened into cloned statements — it may not be finitely
  enumerable). Phase A must leave this seam open even though only two kinds exist yet.
- **The kind-interpreted `ubc_children` doctrine gives created ideas a home.** Ideas
  materialized by the Creation Postulate are memoized into the Integer brane's own compute-time
  store, interpreted its own way — a third interpretation alongside "singular result" and
  "storage tree."
- **One known tension to resolve in the follow-on FOOP: the protocol counts and copies, a
  generative brane cannot be counted or copied.** Protocol stages 5 (count the total lines)
  and 6 (constanic-copy each line into its SubBrane position) both require finite enumeration
  of every constituent's statements. The Integer brane's conceptual statement list is
  unbounded — the Creation Postulate mints statements on demand — so
  `Concat(IntegerBrane, program_brane)` cannot pass stage 5 as specified. Three candidate
  resolutions, each with a distinct cost:
  1. *Materialized-only participation*: the Integer brane reports (and copies) only the ideas
     it has already memoized. Cheap and finite, but the copy FREEZES the set — an integer
     first asked for AFTER the join would not exist in the copied lines, defeating
     create-on-ask delivery. Probably wrong for the delivery use case.
  2. *Adoption by reference*: the generative element is never copied; it becomes a delegated
     storage node that the ConcatBrane's `_search_brane`/`stmt_at` consult LIVE through the
     trait surface. Create-on-ask keeps working inside the join. Positional addressing over it
     is then either excluded or materialized-only (an unbounded element makes global `#n`
     ill-defined past it).
  3. *Context, not content*: the generative brane does not occupy line positions at all; it
     participates only as a fall-through layer for otherwise-failed IB searches from the
     joined lines. Cleanest arithmetic (it contributes zero lines), but it bends the
     Equivalence Law — the element is "in scope" without being "in the brane."
  Deliberately not decided here. The Phase A obligation is narrower and concrete: stages 5
  and 6 must reach constituents ONLY through the trait surface (`stmt_count`, `stmt_at`, and
  the copy hook), never by direct `foolish_children` enumeration — so a future kind can
  override how (or whether) it is counted and copied without touching the protocol.

## FIR Impact

No new FIR variant and no new NYES state. `FirKind` is unchanged. The `Fir` trait gains
`stmt_count`, `stmt_at`, and `settled_result` (defaults preserve current behavior for every
existing kind); `ConcatenationFir` interprets its `ubc_children` as the storage tree, per the
kind-interpreted doctrine.
`concatenation_nyes_transitions` must be extended for the populate-then-drain progression; no
other `*_nyes_transitions` change (the terminal-state rule is unchanged).

## UBC Step Impact

`ConcatenationFir` only. Before: `BRANING` merges element statements (Rc-shared) into one new
`BraneFir` pushed as the result. After: `BRANING` constanic-clones element statements into
hidden bounded segments, re-steps the non-constanic clones (recoordination), then settles by the
same terminal rule. Two observable consequences:

1. **Step counts change** for every concatenation program (extra recoordination steps).
2. **Cross-element references may now resolve** where they previously stayed ECONSTANIC (the
   `{a=10}{b=a}` class), because cloning replaces Rc-sharing. This is a deliberate semantic
   repair aligned with FOOP-3's constanicClone clause.

Both mean `.snap.new` churn in the concatenation snapshots. Every such change goes through the
human review workflow — NEVER auto-accepted — and the semantic ones are the point of the FOOP.

**Relation to FOOP-3:** this FOOP *implements* FOOP-3's "concatenation produces constanicCloned
elements" clause properly, and *supersedes* its "further steps delegate to the merged brane"
clause — there is no merged brane; further steps run against the segment statements in place.

## Test Plan

**Snapshot tests are developed FIRST, then unit tests.** The UBCa snapshot suite runs with
`max_brane_size = 13` once the configuration exists (13 is small enough that modest inputs
force real splitting — 200 statements > 13² = 169 forces a three-level tree — and large enough
that existing small-brane snapshots are untouched). By the Sequencing requirement, snapshot
output is k-invariant, so the concat snapshots double as the Equivalence Law's observable
tests AND as Phase B's no-churn sentinels.

**The new `concat_brane_*` snapshot family REPLACES the existing `concatenation_*` snapshots.**
Retiring the old approved snaps is a HUMAN action (AI never moves or deletes `.snap` files);
the plan gates it accordingly.

### Snapshot tests (`foolish-ubca/snapshot_tests/input/`)

**The fenced `foolish` blocks below ARE the snapshot inputs** — complete, valid programs to be
copied VERBATIM into `foolish-ubca/snapshot_tests/input/` at plan step A1 (byte-for-byte; the
spec is the source of truth for these files). The "expected" notes state the intended
resolutions for BDFL review — the actual `.snap` bytes are generated by the harness and signed
by the human, never hand-written.

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

Expected: `named_lit = {a=1; b=2; c=3}` (the cross-element repair — `c`'s searches revive in
the join); `lit_lit = {x=1; y=2; z=3}` (both literals auto-SFF'd, all resolution in the join);
`with_empty = {p=1}` (empty constituents contribute zero lines); `twice = {a=1; b=2; a=1; b=2}`
(a named brane may appear more than once; each appearance copies independently).

#### `concat_brane_foolish_concatenations.foo`

Nested WRITTEN concatenations — braces-of-concatenations as elements, per the sketch
`{ { { {..}{...}{..} }{..}{ {..}{ {..}{..} } } } ...}`:

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

Expected: `flat` is one brane of five lines in source order. In `deep`, each braced element is
a BRANE whose single anonymous line settles to a joined brane — so `deep`'s lines are: the
anonymous `{f=6; g=7}`, the anonymous join of `{h=8}` with the anonymous `{i=9; j=10}`, and
`k=11`. The snapshot documents precisely this containment (nested concatenation does NOT
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

The final line `total = a1 + a100 + a200` is a deliberate cross-chunk probe: under k=13 its
three operands live in the 1st, 8th, and 16th SubBrane. Expected: 201 lines, `total = 301`,
rendered as ONE flat brane — byte-identical unsplit (Phase A) and split (Phase B, 16 SubBranes
under stacked ConcatBranes). Approved once in Phase A, this snap is the Phase B no-churn
sentinel. (@human: strike the `total` line if you want the pure `{a1…a200}` form originally
specified.)

#### `concat_brane_nested_shadowed_resolution.foo`

The exhaustive shadowing matrix — every resolution layer represented, with the SAME name
(`shadow`) bound differently in each, so each result pins WHERE its search resolved:

```foolish
{
	p = 100;
	shadow = 111;
	orig = {shadow=1, keep=shadow, late=zz};
	cb = {shadow=2, zz=9} orig {from_join=shadow, from_parent=p} <{prepared=shadow, prepared_p=p}>
	
   extended = <{grab_p = p, grab_shadow=shadow}>  {x=cb.shadow} {y=<grab_shadow>} {z=<<grab_shadow>>} <{grab_p_ao = p+1, grab_shadow_ao=shadow+1}> {g1=grab_p_ao, g2=<grab_p_ao>, g3=<<grab_p_ao>>}
}
```

Expected resolutions inside `cb` (join line order: `shadow=2; zz=9; shadow=1; keep=1; late=9;
from_join=1; from_parent=100; prepared=111; prepared_p=100`):

| line | layer pinned | resolves to | why |
|---|---|---|---|
| `keep` | (iii) ORIGINAL context | `1` | resolved to `orig`'s own `shadow=1` BEFORE the copy; the copied constant keeps its value — join-time `shadow=2` cannot rebind it |
| `late` | (i) PREFIX constituent | `9` | ECONSTANIC in `orig` (no `zz` there); revives on copy and IB-finds `zz=9` from the earlier constituent |
| `from_join` | (i) PREFIX, nearest match | `1` | auto-SFF'd literal resolves in the join; nearest preceding `shadow` is `orig`'s copied `shadow=1`, NOT the farther `shadow=2` |
| `from_parent` | (ii) PARENT context | `100` | no `p` anywhere in the join; AB search exits the ConcatBrane to the enclosing brane |
| `prepared` | (iv) `<{…}>` local preparation | `111` | SF override: resolves BEFORE the copy from `cb`'s own statement position — sees the OUTER `shadow=111`, never the constituents' |
| `prepared_p` | (iv) `<{…}>` local preparation | `100` | same timing; `p` from the enclosing brane |

The `prepared = 111` vs `from_join = 1` contrast is the heart of the test: the SAME search
pattern (`shadow`) with different per-element timing yields different, individually correct
answers.

### Unit tests

Unit tests in `foolish-ubca` follow the snapshots; existing concatenation-related unit
expectations regenerate (see UBC Step Impact) and semantic changes require human review.
- `concat_brane_split_long_brane_hierarchy` — companion to the snap: under k=13 the a1…a200
  storage tree is the pure function of (200, 13) — 16 SubBranes of ≤ 13 lines under stacked
  ConcatBranes, every node's fan-out ≤ 13, balanced per the arrangement rule, global indices
  0..199 in order.

Equivalence Law and search:
- `concat_equals_big_brane` — the same statements evaluated as `{s₁…sₙ}` and as
  `{s₁…s₅}{s₆…sₙ}` settle to identical sequenced output (the Law, end to end).
- `concat_search_brane_translates_global_indices` — forward and reverse `_search_brane` over a
  ConcatBrane find the correct statement with the correct global index, including hits in the
  first, middle, and last segment.
- `concat_ib_search_crosses_segments` — `{a=10}{b=a}` resolves `b` to `10` (populate-time
  recoordination + global line numbers).
- `concat_ab_search_reaches_outward` — a statement inside a ConcatBrane resolves a name defined
  in the enclosing brane.

Indexing:
- `concat_index_spans_segments` — `#9` into 5+5 finds the last statement; `#-1` finds the same;
  head/tail across the boundary; out-of-range → NK.
- `concat_find_stmt_index_is_global` — identity scan returns global indices.

Protocol (element typing, auto-wrapping, copy-and-coordinate):
- `concat_element_typing_rejects_non_brane` — a non-brane, non-search direct element
  (e.g. `1 {a=2}`) raises an alarm and constructs the ConcatBrane `NK`; it never steps. Same
  for the settle-time pass: an element search resolving to a non-brane → alarm + `NK`.
- `concat_construction_auto_wraps` — literal brane elements are SFF-wrapped (internal searches
  BORN ECONSTANIC; no standalone resolution); search elements are SF-wrapped.
- `concat_cross_element_reference_resolves` — `{cb = {a=1, b=2} {c = a + b};}` settles with
  `c = 3` (the worked example; pins the semantic repair — today `c` stays unresolved).
- `concat_sff_born_searches_revive_embryonic` — the copy transforms a literal's SFF-born
  ECONSTANIC searches to EMBRYONIC in position, with correct parents.
- `concat_sf_on_search_is_noop` — `<search>` behaves identically to the bare search element
  (idempotent with the auto-SF).
- `concat_sf_marked_literal_prepares_locally` — `<{x = name_from_concat_context}>` resolves its
  innards BEFORE the copy, from the ConcatBrane's own statement position (IB against preceding
  lines, AB through the concat's ancestors); the settled lines then copy with standard
  recoordination.
- `concat_explicit_sff_element_is_error` — `<<{…}>>` as a direct element: alarm + `NK`; the
  ConcatBrane never steps.

Structure, value, clone:
- `concat_statement_parents_point_at_top_concat` — parents bypass the whole storage tree;
  storage nodes never surface via `get_my_brane`.
- `concat_value_is_itself` — `settled_result()` of a ConcatBrane is `None`, so `value()` of a
  settled ConcatBrane is itself, not its first SubBrane; `as_i64` is `None` via the unmodified
  default (branes are not integers — no override exists).
- `concat_constanic_clone_rewires_and_recoordinates` — cloning a settled ConcatBrane as a search
  result deep-clones the storage tree with `skip_foolish_children = true`, rewires parents to
  the clone, preserves numbering and arrangement; the clone carries NO element FIRs and the
  element searches do NOT re-run (value semantics: destination-context shadowing cannot change
  the constituents).
- `settled_search_clone_skips_foolish_children` — cloning a settled `SearchFir` with the new
  option copies the result from `ubc_children` without the anchor subtree; observable behavior
  identical to today's clone.
- `concat_arrangement_is_function_of_n_and_k` — the storage tree shape depends only on total
  line count and `k`: a nested-ConcatBrane element contributes its lines like any brane
  (no inherited shape); unlimited `k` yields a single SubBrane; boundary cases n=k² and n=k²+1.
- `concatenation_nyes_transitions` — extended for the typed, auto-wrapped,
  settle-count-arrange-copy-drain progression.

Auto-sizing (compiler, Phase B):
- `unlimited_config_is_identity`, `brane_at_or_under_max_is_not_split`,
  `oversized_brane_splits_into_chunked_concatenation` (5 statements, k=2 → chunks 2,2,1),
  `root_brane_is_never_split`, `characterized_brane_is_never_split`,
  `split_brane_settles_to_same_result_as_unsplit` (includes a cross-chunk reference).
- `iterative_grouping_bounds_every_node` — n=30, k=3: 10 chunk branes exceed k, so grouping
  iterates; assert the resulting concatenation tree has NO node (statements or elements) with
  more than 3 children, leaves preserve statement order, and the program settles identically to
  the unlimited compile. Also the exact-boundary case n=k² (no third level) and n=k²+1 (third
  level appears).

## Rejected Alternatives

### A. Do nothing

Keeps the merge, which structurally violates any brane-size bound and keeps the Rc-sharing
defects (wrong parents, no recoordination).

### B. AST-level chunking only, keeping the merging ConcatenationFir (this FOOP's first draft)

Self-defeating: the compiler splits an oversized brane into chunks, and the concatenation's own
step immediately reassembles the oversized brane the configuration forbids.

### C. Three FIR kinds — a public SubBrane in the parent chain

SubBrane tracks its offset and forwards IB queries to the previous sibling or the container.
Rejected: statements' parents would point at their chunk, so every resolution path
(`get_my_brane`, `_ib_search`, AB traversal) needs forwarding logic, a new FIR kind needs a NYES
row and transitions tests, and ~14 kind-match sites need three-way decisions. The hidden-segment
design achieves the same bound with ONE new implementation point (`ConcatBrane::_search_brane`)
and no new kind. (Considered and discarded during design discussion, 2026-07-03.)

### D. Segments in a dedicated struct field instead of `ubc_children`

Breaks FOOP-62's two-store uniformity: generic traversal, Debug, and clone code that walks
`ubc_children` would silently miss the segments — and it concedes the false premise that
`ubc_children` must mean "result chain." The store is kind-interpreted by design (see Doctrine
correction); the ConcatBrane using it for storage is intended usage, and `settled_result()`
makes each kind's interpretation explicit in the trait.

### E. Split evaluated BraneFirs at step time (spontaneous re-splitting of settled branes)

Touches every step rule and can oscillate against concatenation. Out of scope: the only
arrangement computation happens inside the ConcatBrane protocol's count-and-arrange step; a
settled `BraneFir` is never spontaneously re-split.

### F. Splice constituent lines into one flat unbounded array

Copying all `n` lines into a single flat store rebuilds exactly the oversized brane the bound
forbids (`n > k`). The count-and-arrange step re-chunks instead: SubBranes of ≤ `k` under
stacked ConcatBranes of fan-out ≤ `k`.

### G. Shape-preserving adoption of nested-ConcatBrane elements (this FOOP's second revision)

Adopting a nested ConcatBrane's storage tree as-is (an "inner bag" preserving its shape) keeps
the bound but makes the storage shape depend on element history. Superseded by count-and-arrange
(Atlas, 2026-07-04): after the constituents settle, element boundaries carry no semantics — the
lines are being constanic-copied anyway, so the arrangement might as well be the pure function
of `(n, k)`, which is simpler to verify and gives every equal-sized ConcatBrane an identical
storage shape.

## Open Questions

- Should characterized branes eventually split, with characterizations carried by the
  ConcatBrane? Deferred until characterizations have merge semantics.
- Should the root brane's statement list be bounded too (implicit wrapper)? Deferred; changes
  the root convention.
- Should a cloned ConcatBrane recompute its arrangement for the CURRENT `MAX_BRANE_SIZE` if the
  configuration differs from construction time? Current answer: no — the clone preserves the
  arrangement; revisit with distribution work. (Less pressing now that the arrangement is a
  pure function of `(n, k)` — clones under the same config are identical anyway.)
- **Auto-SF/SFF wrappers must not leak mechanism.** Two hygiene sub-questions to answer with
  tests: (a) the step driver sets `with_ancestral_sfm(true)` when stepping under a StayFoolish
  — the auto-SF wrappers will raise that flag for everything the concat steps, and
  `constanic_clone_at` takes the flag as a parameter, so the auto-wrapper must not accidentally
  change the NYES transform of copied lines; (b) the sequencer must render elements as the
  Foolisher wrote them — the auto-wrappers are construction bookkeeping and must be invisible
  in `hfssnap` output.

## Future Work (TODO)

- [ ] TODO: think hard about implementing **multi-search** on top of this brane concatenation.
      Multi-search is like the current search except that instead of returning the FIRST match
      it returns ALL possible matches, and the result is represented by a brane. The
      materialized version of multi-search will most likely resort to brane concatenation —
      there may be many matches, so the result brane is naturally assembled as a ConcatBrane of
      bounded units rather than one unbounded brane. Machinery this FOOP provides toward it:
      the Equivalence Law gives the flat all-matches view over chunked storage; MAX_BRANE_SIZE
      is respected by construction during accumulation; and per the kind-interpreted
      `ubc_children` doctrine, a multi-search FIR is simply a new kind whose settled store means
      "a brane of matches" (the SINGULAR-RESULT INVARIANT stays scoped to the existing search
      FIRs and is not violated — multi-search's `settled_result()` reports its result brane).
      Deliberately NOT designed here; needs its own FOOP (search-order semantics, IB/AB scope of
      "all", NYES progression, sequencer rendering).
- [ ] TODO: the **Integer brane** (see §"The Integer brane" under the ConcatBrane redesign):
      a THIRD kind of brane — different from normal Brane and ConcatBrane, yet concat-able
      exactly like them — supplying integers-as-identifiers and arithmetic operators, delivered
      to programs as `Concat(IntegerBrane, program_brane)`, creating ideas on demand via the
      Creation Postulate. Needs its own FOOP; this FOOP must keep the populate step's adoption
      rule capability-based (any brane-like element concats) and avoid baking in the assumption
      that every element's statement count is cheap, finite, and stable.

## References

- Prior FOOPs: FOOP-3 (partially superseded — see UBC Step Impact), FOOP-7 (constanic clone
  recoordination), FOOP-62 (two-store ProtoBrane, root convention, ownership of NYES).
- Code locations: `foolish-ubca/src/fir_kinds.rs` (`ConcatenationFir`, `BraneFir::_search_brane`,
  `StatementFir::_ib_search`, `FirRefNavExt::{index_into, find_stmt_index}`, `find_parent_brane`),
  `foolish-ubca/src/fir_trait.rs` (`Fir` defaults, `get_my_brane`, `FirRefExt::value`,
  `step_inner`), `foolish-ubca/src/proto_brane.rs` (constanic clone),
  `foolish-ubca/src/evaluator.rs` (`UbcaEvaluator`, `proto_to_core_fir`),
  `foolish-ubca/src/compiler.rs` (`Compiler::compile`, `compile_standalone`).
- Snapshots exercising concatenation: `concatenation_three_way`, `concatenation_references`,
  `concatenation_inline_branes`, `concatenation_of_empty_branes`,
  `concatenation_with_unresolved_search` under `foolish-ubca/snapshot_tests/approved/`.

## Last Updated

**Date**: 2026-07-04
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Ninth revision — the fenced foolish blocks in §Test Plan are now declared to BE
the snapshot input files (copied verbatim at A1; the spec is the source of truth), and
`concat_brane_split_long_brane` is written out literally in full (all 200 assignments plus the
`total` probe) instead of abbreviated with an ellipsis.

**Date**: 2026-07-04
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Eighth revision — the four `concat_brane_*` snapshot inputs are now written IN
FULL in the Test Plan (Foolish source plus expected-resolution notes) for BDFL reading:
test_basic (incl. empty constituents and a twice-used named brane), foolish_concatenations
(flat vs braced nesting — braces are branes, only juxtaposition joins),
split_long_brane (with a `total = a1 + a100 + a200` cross-chunk probe, flagged @human for
possible strike), and nested_shadowed_resolution (full expected-resolution table; the
`prepared=111` vs `from_join=1` contrast pins per-element timing).

**Date**: 2026-07-04
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Seventh revision per Atlas: (1) Sequencing promoted to a REQUISITE section — a
settled ConcatBrane displays as ONE brane (flat, wrapper-free, boundary-free), with the
k-invariance consequence stated: snapshots approved unsplit must survive splitting unchanged.
(2) Test Plan restructured snapshot-first: suite runs at max_brane_size=13; new `concat_brane_*`
snap family (test_basic, foolish_concatenations, split_long_brane — the a1…a200 Phase B
sentinel, 200 > 13² forcing the third tree level — and nested_shadowed_resolution, the
exhaustive shadowing matrix) REPLACES the `concatenation_*` family, retirement human-gated;
unit tests follow, incl. `concat_brane_split_long_brane_hierarchy` for the hidden tree.

**Date**: 2026-07-04
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Sixth revision per Atlas: (1) resolved the auto-SF reuse Open Question — a settled
ConcatBrane clones as a VALUE via the new general `constanic_clone(…, skip_foolish_children =
true)` option: only `ubc_children` (the storage tree) is cloned, element FIRs are not, element
searches never re-run; question removed from Open Questions per template. (2) The option is
general: settled singular-result kinds (SearchFir/IndexFir/HeadTailFir) can clone without their
dead anchor subtree — adopting it there is Phase A cleanup scope. (3) The element timing menu is
now a table in the protocol, with the error row generalized to "all other FIRs" (including
explicit `<<…>>`). Tests and plan updated accordingly.

**Date**: 2026-07-04
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Fifth revision per Atlas: (1) element typing extended with the third accepted form
— an explicitly SF-marked expression; SF on a search is idempotent (NOOP), SF on a literal brane
is an OVERRIDE of the automated SFF wrapping selecting local preparation (innards resolve BEFORE
copy, from the concat's own statement context); explicitly SFF-marked elements are an error
(alarm + NK). Third worked example added exercising the full element menu. (2) Expanded the two
terse notes: the auto-SF reuse question (formula vs. value readings of a cloned ConcatBrane) and
wrapper hygiene (has_ancestral_sfm leak, sequencer invisibility) in Open Questions; the Integer
brane count-and-copy conflict with three candidate resolutions and the narrowed Phase A
obligation (stages 5/6 through the trait surface only). Four protocol tests added.

**Date**: 2026-07-04
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Fourth revision — the ConcatBrane protocol per Atlas: (1) fundamental contract
(list of BRANES; violations alarm + NK) enforced at construction and settle time; (2)
construction auto-wraps SFF around literal elements (searches born ECONSTANIC, deferred to the
join — resolves the resolve-BEFORE/AFTER timing question structurally) and SF around search
elements; (3) count-and-arrange replaces shape-preserving adoption: storage tree is a pure
function of (n, k) — SubBranes ≤ k under stacked ConcatBranes (new Rejected Alternative G);
(4) copy-and-coordinate transforms SFF-born ECONSTANIC → EMBRYONIC in position. Terminology:
segments/bags → SubBranes/stack ConcatBranes. Worked example `{cb={a=1,b=2}{c=a+b}}` added with
verified current behavior (c unresolved today). Protocol tests added; Open Question on exact
auto-SF semantics added.

**Date**: 2026-07-03
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Status Draft → Brewing — design converged after the full discussion; submitted for
BDFL review.

**Date**: 2026-07-03
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Pinned the `settled_result()` contract after Atlas's question: the constanic gate
lives INSIDE the method (pre-constanic always answers None); None means "nothing to resolve
into" (not settled yet, or settled and I AM my value); it can never return self — the caller
holding the handle substitutes self. `value()` becomes trivial over it.

**Date**: 2026-07-03
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Noted the Integer brane is a THIRD kind of brane, concat-able exactly like normal
Brane and ConcatBrane; made the populate step's adoption rule normatively capability-based (any
brane-like value concats; segment/inner-bag are this FOOP's two instances), so a future brane
kind slots in without touching the step. Plan A3 updated to match.

**Date**: 2026-07-03
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Added informative subsection "The Integer brane" under the ConcatBrane redesign:
integers as identifiers answered by a custom-built brane (create-on-ask via the Creation
Postulate, `docs/why/creation_postulate.md`), arithmetic operators resolved by name
(`{r = {1,2} plus}`), delivered as `Concat(IntegerBrane, program_brane)`; noted the design
constraint it imposes on this FOOP (no assumption that element statement counts are cheap,
finite, stable) and added a matching Future Work TODO.

**Date**: 2026-07-03
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Added "Future Work (TODO)" section: think hard about multi-search (all matches
instead of first, result represented as a brane, materialized via brane concatenation) as a
follow-on FOOP building on the ConcatBrane machinery.

**Date**: 2026-07-03
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Third revision (Atlas's correction): removed the spurious `as_i64` override (branes
are never integers; the default chain yields None untouched); replaced the `is_own_value()` bool
hook with `settled_result() -> Option<FirRef>` so `value()` asks each kind for its result; added
the "Doctrine correction: ubc_children is kind-interpreted" subsection enumerating the doc
comments in fir_trait.rs/proto_brane.rs that overstate a universal "ubc_children[0] is the
value" rule, to be corrected in Phase A. Rejected Alternative D updated accordingly.

**Date**: 2026-07-03
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Second revision: added explicit two-phase structure (Phase A ConcatBrane upgrade,
Phase B MAX_BRANE_SIZE), the iterative chunking rule (element arrays > k group recursively into
a k-ary concatenation tree), and the hidden storage generalized from a flat segment list to a
k-ary tree of bags with nested-ConcatBrane elements adopted (not spliced — new Rejected
Alternative F). Parent bypass now targets the top ConcatBrane from any depth.

**Date**: 2026-07-03
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Major revision after design discussion: the merging ConcatenationFir is replaced by
a non-merging ConcatBrane with hidden bounded segments (no SubBrane FIR kind), governed by the
Equivalence Law (ConcatBrane ≡ big brane of all statements in order, never materialized). Added
capability dispatch (`stmt_count`/`stmt_at`/`is_own_value`), global line-number rule, parent-link
table, constanic-clone requirements, indexing over prefix sums, FOOP-3 supersession note, and
expanded test plan. First-draft AST-only design moved to Rejected Alternative B.
