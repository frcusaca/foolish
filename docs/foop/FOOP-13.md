---
foop: 31
title: MAX_BRANE_SIZE — auto-sizing via a non-merging ConcatBrane equivalent to the merged brane
author: Atlas <hc.busy@gmail.com>
status: Draft
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

Branes are the unit of containment, cloning, and recoordination. A single very large brane is a
single very large granule: it clones as one block and (in future phases) ships across evaluator
boundaries as one block. `MAX_BRANE_SIZE` bounds the granule: no `BraneFir` statement store ever
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
`ConcatenationFir` (the ConcatBrane). The chunks inside a ConcatBrane — **segments** — are
hidden storage, NOT a new FIR kind, and are **never exposed through the parent chain**. This is
what keeps the change local: name resolution walks parents to "the brane" and asks it questions;
if the answers obey the Equivalence Law, nothing upstream needs a forwarding protocol.

#### Hidden storage tree (two-store placement)

The hidden storage is a **k-ary tree of bags** held in the ConcatBrane's `ubc_children`
(storage is a compute-time result — exactly what `ubc_children` is for, per FOOP-62's two-store
design). `foolish_children` remain the original, parse-time element FIRs, untouched. A bag is
either:

- a **segment** — a plain settled `BraneFir` holding ≤ `k` cloned statements (leaf), or
- an **inner bag** — a settled `ConcatenationFir` used as storage, holding ≤ `k` child bags
  (internal node; this is how a nested-ConcatBrane element is adopted, preserving its shape).

Bags never step, never resolve names, and never appear in the parent chain. No new FIR kind:
both bag shapes reuse the two existing kinds as inert containers. Deliberate deviations from
ordinary children:

- **Statement parents bypass the whole tree**: every statement cloned into any bag has
  `.parent = the top (public) ConcatBrane`, no matter how deep its bag sits. Bags' own parents
  also point at the top ConcatBrane. Since no statement's parent points at a bag, bags never
  appear in `get_my_brane` / `get_my_statement` walks — only the top ConcatBrane is visible.
- **Global line numbers**: each cloned statement's `line_number` is rewritten to its global
  index across the entire tree, in order. This single rule makes `StatementFir::_ib_search`
  (`brane._search_brane(name, line_number − 1, 0)`) work UNCHANGED for cross-segment IB —
  `{a=10}{b=a}` resolves with zero changes to StatementFir.
- Offsets are not stored; they are prefix sums over bag statement-counts, computed on access
  (tree fan-out is ≤ `k` and depth is logarithmic). Zero-length bags are skipped in the
  arithmetic (`concatenation_of_empty_branes` case).

#### Populate step (replaces the merge)

`ConcatenationFir::fir_op_step` becomes:

- `PREMBRIONIC`/`EMBRYONIC`: empty element list → settle exactly as today (empty constant
  brane); otherwise push every element as a task and go `BRANING` (unchanged shape).
- `BRANING`, all elements constanic, storage not yet built: for each element in order, take its
  value and **constanic-clone** it into a bag (NYES transformed per the constanic-clone rules of
  FOOP-7/FOOP-62): a brane value becomes a segment; a ConcatBrane value is **adopted as an inner
  bag** — its own storage tree is deep-cloned beneath, preserving its shape. (It is NOT spliced
  flat: flattening would rebuild an unbounded array at the top node, exactly what the bound
  forbids.) Assign global line numbers across the finished tree and rewire every cloned
  statement's parent to the top ConcatBrane. Push every non-constanic cloned statement as a
  task — recoordination against the new context is what resolves cross-element references — and
  remain `BRANING`.
- `BRANING`, storage built, tasks drained: settle by the existing rule — any NK → `NK`, any
  ECONSTANIC/WOCONSTANIC → `WOCONSTANIC`, else `CONSTANT`.

#### Capability dispatch instead of kind matching

How two brane types "generically, dynamically" share operations: through `dyn Fir` trait methods
— the mechanism `_search_brane` already uses (default `None` on `Fir`, overridden by branes).
New/changed `Fir` methods:

```rust
/// Number of statements this FIR presents as a brane. `None` = not brane-like.
fn stmt_count(&self) -> Option<usize>;          // Brane: foolish_children.len(); ConcatBrane: Σ over tree
/// The statement at a global index, per the Equivalence Law.
fn stmt_at(&self, idx: usize) -> Option<FirRef>; // Brane: foolish_children[idx]; ConcatBrane: tree descent
/// True when this constanic FIR IS its own value (ubc_children are not a result chain).
fn is_own_value(&self) -> bool;                  // default false; ConcatBrane: true
```

- `ConcatenationFir::_search_brane(expr, start, end)`: map the global, direction-aware range
  (the same reversed-range convention `BraneFir::_search_brane` uses) onto per-bag local ranges
  via prefix sums, recursing down inner bags; delegate the leaf scan; translate the hit index
  back to global.
- `ConcatenationFir::_ab_search`: identical logic to `BraneFir::_ab_search` (enclosing
  statement's IB, then parent brane) — shared via a free function or default method, not
  duplicated.
- `Fir::as_i64`'s default reads `ubc_children[0]` — ConcatBrane overrides to `None` (its
  ubc_children are segments, not a result chain).

**`FirRefExt::value`** gains one check: if the FIR `is_own_value()`, stop — a settled ConcatBrane
resolves to itself (like a BraneFir, which today only works because it happens to have no
ubc_children). Without this, `value()` would "resolve" a ConcatBrane to its first segment.

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

#### Constanic-cloning a ConcatBrane

When a settled ConcatBrane is itself the found result (e.g. `x = c` where `c` names a
concatenation), `constanic_clone_at` must: deep-clone the storage tree (statements included),
rewire the cloned statements' parents to the NEW ConcatBrane clone, preserve global line
numbers and tree shape, and transform NYES per the standard clone rules — so
previously-ECONSTANIC searches inside may recoordinate against the destination context. The
existing clone machinery walks `foolish_children` generically; the ConcatBrane arm must be
taught about the storage tree explicitly.

#### Sequencing

`proto_to_core_fir` renders a settled ConcatBrane as ONE flat brane — every statement of the
storage tree in global order — per the Equivalence Law. Where evaluation semantics are
unchanged, sequenced output is byte-identical to today's merged-brane rendering.

## FIR Impact

No new FIR variant and no new NYES state. `FirKind` is unchanged. The `Fir` trait gains
`stmt_count`, `stmt_at`, and `is_own_value` (defaults preserve current behavior for every
existing kind); `ConcatenationFir` gains segment storage semantics for its `ubc_children`.
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

Unit tests in `foolish-ubca`. No new `.foo` approval inputs are required, but existing
concatenation snapshots will regenerate (see UBC Step Impact) and require human review.

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

Structure, value, clone:
- `concat_statement_parents_point_at_top_concat` — parents bypass the whole storage tree; bags
  never surface via `get_my_brane`.
- `concat_is_own_value` — `value()` of a settled ConcatBrane is itself, not its first bag;
  `as_i64` is `None`.
- `concat_constanic_clone_rewires_and_recoordinates` — cloning a settled ConcatBrane as a search
  result deep-clones the storage tree, rewires parents to the clone, preserves numbering and
  shape.
- `nested_concat_elements_are_adopted_not_spliced` — an element value that is a ConcatBrane
  becomes an inner bag preserving its subtree; the top node's direct bag count does not grow
  beyond its element count; search/indexing/IB still see the flat global order.
- `concatenation_nyes_transitions` — extended for the populate-then-drain progression.

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

Keeps `value()` untouched but breaks FOOP-62's two-store uniformity: generic traversal, Debug,
and clone code that walks `ubc_children` would silently miss the segments. The `is_own_value`
hook is a smaller, explicit deviation.

### E. Split evaluated branes at step time (rebalancing merged results)

Touches every step rule and can oscillate against concatenation. Out of scope: this FOOP bounds
construction-from-AST and makes concatenation bound-preserving; it never rebalances.

### F. Splice nested-ConcatBrane elements flat into the parent's storage

Flattening an adopted ConcatBrane's segments into the parent's bag list rebuilds an unbounded
array at the top node — with `n` statements and chunk size `k`, the top ConcatBrane would hold
`⌈n/k⌉` bags, which exceeds `k` for `n > k²`. Adoption as an inner bag keeps every node's
fan-out ≤ `k` and is what the Phase B iterative grouping emits anyway.

## Open Questions

- Should characterized branes eventually split, with characterizations carried by the
  ConcatBrane? Deferred until characterizations have merge semantics.
- Should the root brane's statement list be bounded too (implicit wrapper)? Deferred; changes
  the root convention.
- Should a cloned ConcatBrane rebalance its segments to the CURRENT `MAX_BRANE_SIZE` if the
  configuration differs from construction time? Current answer: no — segments are preserved
  as-is; revisit with distribution work.

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
