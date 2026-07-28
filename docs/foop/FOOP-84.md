---
foop: 48
title: Search Engine Refactor — the authoritative search specification, and detachment on it
author: Atlas hc.busy@gmail.com
status: Draft
type: Standards
created: 2026-07-28
phase: phase-2
supersedes: []
begun: [ ]
---

# FOOP-84: Search Engine Refactor — the authoritative search specification, and detachment on it

> This FOOP **replaces FOOP-23 and FOOP-24 as the authoritative, precise description of how
> search works** in the UBCa reference implementation, and is written to be **self-contained
> enough that FOOP-93, FOOP-04, FOOP-14, and FOOP-85 can cite it instead of re-deriving shared
> background** — Part 1 carries the full operator reference table, `FoolRefFir` shape, the
> name+value atomicity rule, and the cursor-source×predicate/two-collaborator framing, not just
> pointers to FOOP-23. Where this document and FOOP-23/FOOP-24 disagree, this document wins;
> FOOP-23/FOOP-24 are retained for their historical design discussion and implementation-detail
> material (grammar productions, approval-test-input catalogs, Rejected Alternatives, bug-fix
> appendices) but should be read as superseded on anything restated here. This FOOP performs the
> core refactor (Navigator unification, per-candidate boundary-crossing evaluation) while
> **preserving all existing einmo/snapshot test behavior exactly** — no observable evaluation
> change ships in this FOOP except where explicitly called out (the `contexted && !anchored`
> dead-path fix, which is a documentation/pinning change, not a behavior change). Unimplemented
> features (`||`/`&&` matcher booleans, `|` cascade, coordination detachment) are specified here
> **for refactoring purposes only** — as consumers of the facilities this FOOP builds — and are
> implemented in their own FOOPs (FOOP-93, FOOP-04, FOOP-85 — see "Related FOOPs" below).

## Abstract

Two search implementations exist side-by-side in `foolish-ubca/src/fir_kinds.rs` today: the
older name-only `Fir`-trait methods (`_ib_search`/`_ab_search`) and the newer one-engine model
(`CandidateNavigator` × `SearchPredicate` × `contextful_search_scan`, `mod contextful_search`).
The newer model does not yet own the ancestral-brane (AB) walk — `ab_search_with_engine`
independently re-implements it as a hand-rolled loop, duplicating `BraneFir::_ab_search`'s
recursive walk. Neither implementation represents "the sequence of candidates visible to a
search, crossing brane boundaries, in order" as a single object.

This FOOP:

1. **Restates search semantics precisely and completely** (Specification, below) — the
   authoritative reference, superseding FOOP-23's description.
2. **Introduces `AncestralNavigator`**, a `CandidateNavigator` implementation that owns the AB
   walk, replacing `ab_search_with_engine`'s loop and `BraneFir::_ab_search`'s recursion with one
   traversal. `contextful_search_scan` and `SearchPredicate` are **untouched** by this change.
3. **Introduces the per-candidate boundary-crossing evaluation** that stay-foolish markers use to
   affect a crossing search — the mechanism that (in a later FOOP) coordination detachment,
   privacy detachment, and required-searches are all built from. This FOOP does **not** implement
   any marker behavior yet (no `[patterns]` parsing, no `Detachment` struct) — it builds and
   tests the **evaluation shape** (`CopyMode`, per-candidate marker-stack scan, innermost-first)
   against the **existing**, unparameterized SF/SFF markers only, replacing today's blanket
   `Scope.has_ancestral_sfm` boolean with an exact, per-candidate equivalent. This is the
   behavior-preserving core: today's SF/SFF semantics must be reproduced exactly by the new
   mechanism before anything new is layered on.
4. **Documents a confirmed dead path**: `contexted && !anchored` search silently degrades to
   plain unanchored search today (`self.contexted` is never consulted unless `self.anchored` is
   also true — `fir_kinds.rs:1192,1311,1511`). This FOOP states this as **intentional and
   permanent** — see "Contexted search cannot follow backward/unanchored search," below — and
   adds a test pinning the (currently silent, henceforth explicit) fallback. No runtime behavior
   changes.

## Motivation

**Why now, and why as its own FOOP.** FOOP-24 (Detachment) discovered, while trying to design
coordination detachment, that the search engine's shape does not have a place to hang a
per-candidate, marker-boundary-crossing decision. The design that emerged from working through
concrete nested-marker examples needs:

- Per-candidate (not per-search, not per-marker) evaluation.
- A stack of active markers walked **innermost to outermost**, where each level either declines
  (the candidate passes through untouched) or fires (the search-visible outcome for that
  candidate is decided **at that level**, and no further/outer level is consulted).
- A **two-channel outcome per candidate**, not a single flag: a candidate is either filtered out
  before it ever reaches the scan loop (`Detach` — SFF-style, contributes to search exhaustion;
  see §2.3 for why this is a pre-yield filter, not a value the scan loop sees) or it is yielded
  with a copy-mode tag attached (`Normal` — ordinary recoordinating copy — or `SfCopy` — SF-style,
  found normally but its constanic constituents must be cloned without recoordination).

None of this exists today. Today's SF/SFF markers work through a single global boolean
(`Scope.has_ancestral_sfm`, set in `step_inner`, consumed in `SearchFir::handle_found`) that
answers "is this search anywhere under any SF marker" — coarse, un-parameterized, and (per the
FOOP-24 design work) not extensible to per-candidate, per-pattern, multi-level decisions.
Bolting detachment onto the *existing* two search implementations was assessed and rejected (see
Rejected Alternatives) because it either re-inflates the six-parallel-search-paths problem the
FOOP-24 audit already documented, or threads a load-bearing ordered stack through `Scope` — a
structure built by `step_inner` for the *task-tree descent*, which is not guaranteed to coincide
with the *lexical AB-chain climb* a search performs (they coincide today only because searches
are normally stepped from within their lexical position; recoordination and future recursion
work will make them diverge).

**Why the refactor must land before detachment, `!`, `&&`/`||`, `|`, and find-all.** All five of
those pending search FOOPs extend or consume the one-engine model. If detachment's boundary
concern is bolted on ad hoc, each of the other four has to be individually checked for
interaction with it. If instead detachment's mechanism is a property of the **Navigator**
(which crosses boundaries) and the other four are properties of the **Predicate** (which
matches one candidate) or the **scan mode** (which decides how many results to collect), they
become orthogonal by construction — no pairwise interaction to design or test. This FOOP buys
that decomposition once, up front.

## Specification

### Part 1 — Search semantics, restated (supersedes FOOP-23)

This section is the complete, authoritative description of how Foolish search works. It restates
(does not merely reference) FOOP-23's content because FOOP-23's description is no longer
sufficient on its own — the boundary-crossing / detachment interaction it did not anticipate is
now part of the model.

#### 1.1 Three groups of search operators

Unchanged from FOOP-23 — restated here for completeness.

1. **Contextless Anchored Searches** ("contextless searches," or plainly "searches"):
   `.` `?` `~` `#` `^` `$` `~=` `?=`. Each demands its anchor resolve *through* to a whole brane
   and searches that brane. Does not read context (does not start from a statement position).

2. **Contexted Anchored Searches** ("`&`-searches," or "contexted searches"):
   `&?` `&~` `&#` `&^` `&$` `&~=` `&?=`. Anchors on a **statement's position** — the original
   statement a preceding search found — and searches forward/backward from there within that
   statement's **home brane**. Contexted searches stack (`a~step_1 &#1`).

3. **Value searches** — triggered by `=`, matching a statement's *value*. May be combined with a
   name pattern (atomic conjunctive form, e.g. `~name=value`) or bare (`&=`-search shorthand for
   a contexted value search).

**The `.` vs `&` chaining rule (unchanged):** `.` always deepens (searches *inside* the resolved
brane); `&` navigates from a position (searches *near* a found statement, within its home brane).

#### 1.1a Full operator reference table

The complete surface syntax, consolidated from FOOP-23 Parts A/C so downstream FOOPs (FOOP-93,
FOOP-04, FOOP-14, FOOP-85) can cite one table instead of splitting attention across two
documents. FOOP-23 remains the source of the approval-test-input catalog and grammar productions
for this table; this is the table itself, kept authoritative here.

| # | Syntax | Group | Anchoring | Direction | Matches |
|---|--------|-------|-----------|-----------|---------|
| 1 | `a.name` | Contextless | anchored | — (deepen) | name, inside the brane `a` resolves to |
| 2 | `a?name` | Contextless | anchored | backward | name |
| 3 | `?name` | Contextless | unanchored | backward | name (retrospective IB/AB) |
| 4 | `a~name` | Contextless | anchored | forward | name |
| 5 | `a#N` | Contextless | anchored | both (offset) | positional index |
| 6 | `a^` | Contextless | anchored | — | head (first statement) |
| 7 | `a$` | Contextless | anchored | — | tail (last statement) |
| 8 | `a~=value` | Contextless value | anchored | forward | value equals pattern |
| 9 | `a?=value` | Contextless value | anchored | backward | value equals pattern |
| 10 | `?=value` | Contextless value | unanchored | backward | value equals pattern |
| 11 | `a~name=value` | Contextless, atomic name+value | anchored | forward | name AND value, same candidate (§1.1c) |
| 12 | `a?name=value` | Contextless, atomic name+value | anchored | backward | name AND value, same candidate |
| 13 | `?name=value` | Contextless, atomic name+value | unanchored | backward | name AND value, same candidate |
| 14 | `X&?name` | Contexted | anchored (§1.2) | backward | name, from `X`'s found position, within its home brane |
| 15 | `X&~name` | Contexted | anchored | forward | name, from `X`'s found position |
| 16 | `X&#N` | Contexted | anchored | both (offset) | positional index from `X`'s found position |
| 17 | `X&^` / `X&$` | Contexted | anchored | — | head/tail of `X`'s home brane |
| 18 | `X&~=value` / `X&?=value` | Contexted value | anchored | forward/backward | value equals pattern, from `X`'s position |

There is **no `.=`** (Part A rejects it — `.` already means deepen, and `a.=10` reads as a
compound assignment). There is **no `&.`** (§1.1 chaining rule — `.` already deepens; a
"contexted deepen from a position" is not a distinct operation). There is **no unanchored
forward form** in either family (Foolish cannot look forward in its own brane without an anchor).

**Value pattern grammar** (forms 8–13, 18): the pattern (`value_pattern` in `arith_expr`
precedence) may be any expression, evaluated to constanic before comparison — `a~=1+2` searches
for `3`; a search result may itself be the pattern, parenthesized (`a~=(b.k)`). In Part A's
original MVP scope the pattern was restricted to independent-integer equality only; that
restriction is a property of `default_equal`'s implemented cases (FOOP-33), not of the search
grammar, and is unchanged by this FOOP.

#### 1.1b `FoolRefFir` and the two-child invariant, restated with its shape

Expanding §1.4 (below) forward for reference density: `FoolRefFir` is the FIR kind that carries
a found statement's *position* forward through a chain. Its defining shape:

```rust
/// An immutable reference to another FIR — the "fool's reference".
/// Wraps a STRONG (non-weak) FirRef to the original statement a search
/// found. Read-only: no method mutates the referent; born CONSTANT; takes
/// no steps; holds no children of its own.
pub struct FoolRefFir {
    pub(crate) core: ProtoBrane,   // no foolish_children, no ubc_children
    referent: FirRef,              // strong Rc — the ORIGINAL statement
}
```

Strong (not weak) so the original statement stays reachable through the search result even if
its home brane is later restructured. Invisible to values — `FirRefExt::value`, result-chain
walking, and the Humanizing Sequencer all read `ubc_children[0]` only; `FoolRefFir` never
appears in HFS output. See §1.4 for the two-child placement this enables.

#### 1.1c Name+value is an atomic conjunctive operator, not a chained search (restated, unchanged)

Forms 11–13 (`~name=value` and friends) are **not** sugar for "find by name, then filter by
value" — that chain is observably wrong. Given `b = {setting = 11; setting = 10;}`, the naive
chain `b&~setting &~=10` forward-finds the *first* `setting` (11), collapses to that one
position, and only then scans forward from there for value 10 — it does not mean "the `setting`
whose value is 10." The atomic operator `b~setting=10` instead tests **both the name gate and
the value gate on each candidate together, in one scan**, so it correctly reports the *second*
`setting` regardless of ordering. This is why forms 11–13 are single operators in the grammar,
not decomposable into two searches. Positional anchoring still composes normally *after* the
atomic form (`b~setting=10&#-1`).

#### 1.1d One engine: cursor-source × predicate, and the two collaborators

Every operator in the table above (§1.1a) is one engine, parameterized by two independent
properties — this is the "one-engine model" §2.1 builds on directly:

- **Cursor-source** — where the Navigator starts. *Contextless*: the anchor resolved to a brane,
  cursor at front/rear. *Contexted*: the incoming result's statement position, in its home
  brane. This is the *only* step that differs between the two families (§1.1a groups 1–13 vs.
  14–18); everything downstream is shared.
- **Predicate** — what counts as a hit on one candidate: `Name`, `Value`, `NameValue`, `Index`,
  `Head`, `Tail` (today's `SearchPredicate` variants — extended by FOOP-93 with negation and
  boolean composition, and by FOOP-85 with a detachment-adjacent gate; see §2.1).

The engine's two collaborators (`fir_kinds.rs`, `mod contextful_search`):

- **Candidate Navigator** (`CandidateNavigator` trait) — traverses the FIR tree and yields
  candidate statements in the mandated order. Its correctness contract, **load-bearing** and
  reused verbatim by §2.2's `AncestralNavigator`: **correctly ordered** (the one order the
  configured search semantics mandate) and **complete** (yields every reachable candidate,
  exactly once, then stops — nothing skipped, nothing repeated, nothing beyond bound). It knows
  nothing about what is being matched.
- **Statement Matcher** (`SearchPredicate::matches`) — given one candidate, approves or rejects
  it. Receives the *full* statement FIR (name, body/value, line number, parent/home-brane, NYES)
  — not a projection — because different predicates need different facets, and because handing
  the full candidate forward is what lets predicates compose (FOOP-93's `And`/`Or`/`negate`
  trees). It knows nothing about traversal order.

Two degeneracies fall out of the single engine and must hold for any Navigator (including
`AncestralNavigator`, §2.2) implementing it:

- **Contexted on a bare brane ≡ contextless.** `{…}&?c` has no incoming statement position; its
  cursor degenerates to the brane's rear, identical to `{…}?c`.
- **Contextless on a contexted result reads the value, not the position.** `X.y` where `X` is a
  contexted search: `.y` ignores the carried position, takes `X`'s value (a brane), and deepens.

#### 1.2 Contexted search cannot follow backward/unanchored search — a deliberate design choice

**Restated as an explicit rule, not merely observed as an implementation gap.** A contexted
(`&`-prefixed) search resumes from a **position** — specifically, the `FoolRefFir` carried as
`ubc_children[1]` of a prior search's result (the "two-child invariant," §1.4). Only an
**anchored** search result carries this position-bearing `FoolRefFir`; an unanchored search has
no fixed position to hand off (that is definitionally what "unanchored" means — the search may
still recoordinate, so it cannot commit to a position).

**Therefore: `&`-continuation off an unanchored search is not merely unimplemented — it is
structurally impossible under the two-child invariant**, and this FOOP makes that permanent by
policy: **a `contexted` search requires `anchored`.** Concretely, in the current implementation
every runtime call site that performs the contexted-resume already gates on
`self.contexted && self.anchored` (`fir_kinds.rs:1192`, `:1311`, `:1511` — verified, no exception
exists). When `contexted` is set without `anchored`, the search silently falls through to
ordinary unanchored behavior, ignoring the `&`. **This FOOP keeps that fallback behavior exactly
as-is** (no observable change) but makes it an explicit, named, tested case instead of an
unremarked-upon gap:

- **Rule:** `contexted ⟹ anchored` is a well-formedness expectation of the search family, not an
  enforced grammar restriction. `?some_name&#1` (backward, unanchored, contexted-continued)
  parses and compiles without error (the grammar has no way to reject it — `&` attaches
  generically to whatever postfix chain precedes it, per `parser.rs:670-771`), but evaluates
  identically to `?some_name` — the `&#1` is inert.
  - **Motivation for not raising a parse/compile error:** rejecting it would require the parser
    or compiler to track anchored/unanchored-ness through arbitrary postfix chains and is not
    worth the complexity for a case that already degrades safely (no crash, no silent wrong
    answer — just "the search behaves as if `&#1` were absent"). Revisit only if a real program
    is found to rely on the silent fallback in a confusing way.
- **Consequence for detachment (and everything else in this family):** no detachment mechanism —
  value-conditioned pattern or otherwise — can use `&`-continuation to gain additional search
  context for an unanchored candidate. A detachment pattern's value gate is a **one-shot check
  against the candidate's currently-settled (or not-yet-settled) value**, evaluated in place by
  the boundary-crossing walk (§2), with no continuation-search escape hatch in either direction:
  contexted search cannot leave its home brane (§1.3), and it cannot originate from an unanchored
  search (this section). Both facts are independently confirmed against the current
  implementation (`fir_kinds.rs:954-1009`, and `:1192/:1311/:1511` respectively).

#### 1.3 Contexted search is clipped to its home brane (restated, unchanged)

`contexted_search_from_anchor` resolves the anchor's `FoolRefFir` referent, finds that referent's
**home brane** (`_get_my_brane`), and scans only within `[0, p-1]` or `[p+1, len-1]` of that one
brane's children (`fir_kinds.rs:967-985`). It never walks to a parent or ancestral brane. This is
unchanged from FOOP-23/AGENTS.md and is restated here because §2's boundary-crossing mechanism
must not be confused with contexted search: they are different traversals — the AB walk crosses
brane boundaries outward through the lexical parent chain; contexted search stays inside one
brane and only moves among siblings.

#### 1.4 The FoolRefFir two-child invariant (restated, unchanged)

A resolved (anchored) search has exactly two `ubc_children`: `[0]` — the constanic clone of the
found statement's body (the value); `[1]` — a `FoolRefFir` wrapping a strong reference to the
original found statement (parent chain, line number, home brane intact). This is what makes
"providing context" universal and is the mechanism §1.2 depends on.

#### 1.5 NK vs ECONSTANIC miss outcomes (restated, unchanged — see FOOP-43 for the full
settlement rule)

Anchored miss → ECONSTANIC (may recoordinate) per FOOP-43, superseding the older
anchored-miss→NK rule. Found-`???` → NK propagates (terminal). This FOOP does not change FOOP-43;
it is restated here because the boundary-crossing walk (§2) must know which outcome an
exhausted, fully-Detached candidate stream produces (ECONSTANIC, via ordinary exhaustion — see
§2.4).

### Part 2 — The unified Navigator and per-candidate boundary evaluation

#### 2.1 The one-engine model, unchanged at its core

The engine described in §1.1d (cursor-source × predicate; `CandidateNavigator` ×
`SearchPredicate`) is **not modified by this FOOP**. `SearchPredicate` (Name/Value/NameValue/
Index/Head/Tail) and `contextful_search_scan` (the scan loop: iterate candidates from a
`CandidateNavigator`, apply the predicate, return `Found`/`NkStop`/`Miss`) are untouched. This is
deliberate: FOOP-93's predicate-tree extensions (`!`, `&&`/`||`), FOOP-14's collect-mode scan,
and FOOP-43's reason tags all land inside these two collaborators exactly as their own specs
already describe, with zero interaction with anything in this FOOP. What this FOOP adds is a new
*implementation* of the Navigator side of the cursor-source (§2.2) — it must satisfy §1.1d's
Navigator correctness contract (correctly ordered, complete) exactly as `BraneNavigator` does
today.

#### 2.2 `AncestralNavigator` — a `CandidateNavigator` that owns the AB walk

A new implementation of `CandidateNavigator` (`fir_kinds.rs`, alongside `BraneNavigator`) that
replaces:

- `ab_search_with_engine`'s hand-rolled `loop` (`fir_kinds.rs:1085-1119`), and
- `BraneFir::_ab_search`'s recursive walk (`fir_kinds.rs:826-842`)

with one traversal. It climbs the lexical parent chain (`_get_my_statement` /
`_get_my_brane`, the same primitives both existing implementations already use) and yields
candidates in the same visibility order the current loop produces — **no change in candidate
order or completeness**; this is the behavior-preserving half of the refactor.

**Boundary crossing is where markers are seen.** As `AncestralNavigator` steps from a child
brane to its parent, it inspects the FIR whose boundary it just crossed. If that FIR is a
`StayFoolish` or `StayFullyFoolish` marker, the Navigator becomes aware of it — this is the hook
point for §2.3. Because the Navigator crosses markers **innermost-first** (it starts at the
searching FIR's own position and climbs outward), any per-candidate marker-stack evaluation it
performs is innermost-first *by construction* — no separate bookkeeping, no `Scope` field, no
"reverse the search" trick. The Navigator sees the actual FIR chain directly by borrowing
`FirRef`s as it walks; it does not need `Scope` to have pre-computed anything for it.

`Scope.active_detachments` (the field FOOP-24's draft proposed) is **not added**. This FOOP
narrows `Scope`'s role: `has_ancestral_sfm` is superseded by the mechanism below (§2.3–2.5) and
should be considered for removal once nothing else legitimately depends on the coarse boolean —
confirming that is an Open Question (below), not asserted here.

#### 2.3 Per-candidate `CopyMode` evaluation — and why `Detach` is not a third value in the same channel

`CandidateNavigator::next_candidate(&mut self) -> Option<(FirRef, usize)>` (`fir_kinds.rs:1941`)
is the entire contract the scan loop consumes — a candidate is either yielded (`Some`) or
iteration is over (`None`). There is **no side-channel through which a yielded candidate can
carry "but treat me as absent."** This has a direct consequence for how `Detach` must be
implemented, and it is easy to mis-state (an earlier draft of this section did): `Detach` is
**not a third color returned alongside `Normal`/`SfCopy` for a candidate the scan loop
receives.** It is a **pre-yield filter internal to `AncestralNavigator`** — a Detached candidate
is one `next_candidate()` skips *inside its own iteration loop*, moving on to try the next
raw candidate, exactly as if that boundary-crossing FIR were simply not there. The scan loop
(`contextful_search_scan`, `SearchPredicate::matches`) never observes a Detached candidate at
all; it has no representation for one and needs none.

So the per-candidate evaluation the Navigator performs internally, before deciding whether to
even return a given raw candidate from `next_candidate()`, is:

```rust
/// Internal to AncestralNavigator's next_candidate() loop — never a
/// scan-loop-visible type. A candidate resolving to Detach is consumed and
/// discarded by the Navigator's own iteration; it is never returned.
enum BoundaryEffect {
    /// No marker on the walked path decided anything for this candidate.
    Pass,
    /// A marker's rule matched and its effect is "found, but copy constanic
    /// constituents as-is, without recoordination" (SF-style). The candidate
    /// IS yielded by next_candidate(), carrying this tag.
    SfCopy,
    /// A marker's rule matched and its effect is "this candidate does not
    /// exist for this search" (SFF-style). next_candidate() does NOT return
    /// this candidate — it continues its internal loop to the next raw
    /// candidate instead, silently, with no trace visible to the scan loop.
    Detach,
}
```

Only **`Pass`/`SfCopy`** ever need to travel *out* of `next_candidate()` attached to a yielded
item — this is the actual `CopyMode` the scan-outcome plumbing (§2.5) carries forward to
`SearchFir::handle_found`:

```rust
/// What SearchFir::handle_found actually receives, attached to a Found
/// candidate. Only two variants — Detach never reaches this type, by
/// construction (see above).
pub(crate) enum CopyMode {
    Normal,
    SfCopy,
}
```

**Resolution algorithm — innermost-first, first-match-wins, per raw candidate, run inside
`next_candidate()` before deciding whether to return it:**

```
# Called by AncestralNavigator::next_candidate() for each raw candidate it
# considers, in its own internal loop, BEFORE returning (or skipping) it.
fn resolve_boundary_effect(candidate, marker_stack /* innermost..outermost */) -> BoundaryEffect:
    for marker in marker_stack:                 # innermost first
        if marker.rule_applies_to(candidate):    # see §2.4 for what "applies" means today
            return marker.effect()                # StayFoolish -> SfCopy, StayFullyFoolish -> Detach
        # else: this marker declines; the candidate is TRANSPARENT to it — keep walking outward
    return BoundaryEffect::Pass                    # no marker on the path decided anything

# Inside next_candidate()'s loop:
#   loop {
#       let candidate = <next raw candidate from the walk, or return None>;
#       match resolve_boundary_effect(candidate, marker_stack) {
#           BoundaryEffect::Detach => continue,              // skip; try the next raw candidate
#           BoundaryEffect::Pass    => return Some((candidate, CopyMode::Normal, position)),
#           BoundaryEffect::SfCopy  => return Some((candidate, CopyMode::SfCopy, position)),
#       }
#   }
```

(The exact Rust return shape — whether `CopyMode` rides in the tuple `next_candidate()` returns,
or a parallel accessor, or a wrapper around `FirRef` — is an implementation decision for the
plan; the point fixed here is the *channel split*, not the Rust syntax.)

**This is emphatically not "first marker on the stack wins, full stop."** A candidate can pass
through an inner marker whose rule does not apply to it and still be caught by an outer marker.
Concretely, for the nested example already on record in FOOP-24 (kept here verbatim as the
worked example this algorithm must reproduce once patterns exist — see §2.6):

```foolish
{
    a = 10;
    result = [a]<{
        inner = [b]<<{
            x = a;    !! innermost marker [b] does not match candidate 'a' -> transparent;
                      !! outer marker [a] matches -> SfCopy; 'a' IS yielded, tagged SfCopy
            y = b;    !! innermost marker [b] matches candidate 'b' -> Detach; 'b' is
                      !! never yielded by next_candidate() at all; the outer [a] marker
                      !! is never even consulted for this candidate
        }>>;
    }>;
}
```

`x = a`: the walk for `a` first consults the innermost marker (`[b]<<...>>`, SFF) — its rule
(pattern `b`) does not match candidate `a` — transparent, continue outward. The next marker
(`[a]<...>`, SF) — its rule (pattern `a`) matches — resolution stops there and `resolve_boundary_
effect` returns `SfCopy`; `next_candidate()` yields `a` tagged `CopyMode::SfCopy`. `y = b`: the
innermost marker's rule (pattern `b`) matches immediately, returning `Detach` — `next_candidate()`
**does not yield `b` at all**; its internal loop moves on to try the next raw candidate on the
walk, exactly as if `b` were absent from the brane. The outer `[a]` marker is never consulted for
`b` — not because it "lost," but because the Navigator never asks it (the candidate was already
filtered out one level in).

**Note on this FOOP's actual scope:** `marker.rule_applies_to(candidate)` for parameterized
`[patterns]` markers (i.e. the `Detachment` struct, `decide_to_detach`, `RegexSet` matching) is
**not implemented in this FOOP** — it is coordination detachment's own concern (FOOP-85, see
"Related FOOPs"). What **this** FOOP implements and tests is the resolution *algorithm*
(`resolve_boundary_effect`) and the `CopyMode` plumbing for its `Pass`/`SfCopy` outcomes through
to the clone call sites (§2.5), exercised against **today's unparameterized SF/SFF markers
only**, where `rule_applies_to` is trivially "always applies" (an unparameterized marker has no
pattern to test against — it always fires for every candidate that reaches it, exactly like
today's `has_ancestral_sfm`). This is what makes the refactor behavior-preserving and
independently testable before any new syntax exists.

#### 2.4 Naked/unparameterized SF and SFF stay their own thing

Per the existing FOOP-24 design (kept unchanged in intent, restated in this mechanism's terms):
**naked `<E>` and `<<E>>` are not represented as a degenerate case of a general pattern-matching
`Detachment` — they remain their own, separate marker behavior**, with their own `FirKind`
variants (`StayFoolish`, `StayFullyFoolish`) as today. In the `resolve_boundary_effect`
algorithm, an unparameterized `StayFoolish` marker's `rule_applies_to` is unconditionally true
(it always fires) and its `effect()` is `SfCopy`; an unparameterized `StayFullyFoolish` marker's
is unconditionally true and `Detach`. For SFF this means: **every raw candidate under a naked
`<<E>>` is silently skipped by `next_candidate()`, never once yielded to the scan loop** —
reproducing exactly what today's naked-SFF code path already does (it does not call the engine
at all for this reason; see §2.4.1). For SF, every candidate under a naked `<E>` is yielded
tagged `SfCopy`, reproducing what `has_ancestral_sfm` does today for a blanket boolean. This is
expressed in the same per-candidate machinery that a later, parameterized detachment marker will
extend. **When coordination detachment (FOOP-85) adds `[patterns]<...>`/`[patterns]<<...>>`,
that is additional, separate marker configuration — not a rewrite of the naked-marker path.**
This directly preserves FOOP-24's "keep existing SFF unchanged; new code is only for specific
detachments" intent, but grounds it in the shared per-candidate algorithm instead of a two-tier
implementation split.

##### 2.4.1 A fully-Detached search still settles ECONSTANIC via ordinary exhaustion

A search entirely under a naked `<<E>>` (or, once FOOP-85 lands, a `[*]<<E>>`) has every raw
candidate on its walked path resolve to `Detach`. Mechanically, `next_candidate()` simply never
returns anything — its internal loop discards every raw candidate it considers and eventually
runs out, returning `None`. From `contextful_search_scan`'s point of view this is
indistinguishable from a brane that is genuinely empty of matches: the `while let Some(...)`
loop never executes its body even once, and the function falls through to `ScanOutcome::Miss`
(`fir_kinds.rs:2040`) exactly as it would for any other exhausted, no-match search. This
**miss** then settles ECONSTANIC per FOOP-43 (§1.5), exactly as today. No special-case
"reject-all" signal, no distinguished "everything was Detached" outcome, is needed anywhere in
the scan loop — full detachment is not a different *kind* of miss, it is the same miss the
engine already handles, arrived at by a Navigator that happened to filter every candidate before
yielding any of them.

#### 2.5 `CopyMode` replaces `Scope.has_ancestral_sfm` at the clone call sites

Today, `SearchFir::handle_found` (`fir_kinds.rs:935-940`) passes `scope.has_ancestral_sfm` — a
single boolean, constant for the whole search regardless of which candidate was found — into
`clone_stmt_result` → `constanic_clone_at`'s `descendent_of_sfm_and_foolishly_ignorant` parameter
(`fir_kinds.rs:159-198` and its recursive call sites throughout the same function).

**This FOOP changes `handle_found` (and the scan-outcome plumbing feeding it) to carry the
`CopyMode` resolved for the specific candidate that was found**, and to pass
`copy_mode == CopyMode::SfCopy` as the `descendent_of_sfm_and_foolishly_ignorant` argument,
instead of the blanket `scope.has_ancestral_sfm`.

**This is a real, intentional behavior refinement, not a no-op plumbing change**, and must be
called out precisely: today, *any* candidate found while lexically under *any* SF ancestor is
unconditionally foolishly-ignorant-copied. Under this FOOP's mechanism, restricted to
unparameterized markers only (§2.4), the two coincide exactly — an unparameterized marker always
fires, so every candidate under it still gets `SfCopy`, reproducing `has_ancestral_sfm`'s
blanket behavior bit-for-bit. **The observable divergence only becomes possible once
parameterized detachment patterns exist (FOOP-85)** — a candidate that an `[a]<...>` marker's
pattern does *not* match should NOT be foolishly-ignorant-copied even though it is lexically
under an SF marker, because with a real pattern the marker is no longer unconditional. This FOOP
must therefore ship with a test asserting **bit-for-bit equivalence with today's
`has_ancestral_sfm` behavior for every existing SF/SFF snapshot/unit test**, since no
parameterized pattern exists yet to produce the divergence — the divergence is future capability
this FOOP unlocks, not something it exercises.

#### 2.6 Nested-marker resolution is answered by this mechanism (resolves FOOP-24's open question)

FOOP-24's "Nested markers" section (status: UNDECIDED) asked whether resolving nested SF/SFF
requires "reversing the search: match on everything first, then check the stack of detachments
from innermost outward." **Answer: no reversal is needed.** §2.3's algorithm walks the marker
stack innermost-to-outward *as part of yielding each candidate* — there is no separate "match
everything first" phase. The worked example in §2.3 is the resolution FOOP-24 asked for. This
FOOP formally closes that open question; FOOP-85 (coordination detachment) should cite this
section rather than re-deriving it.

### Part 3 — Detachment terminology (renames, for FOOP-85 to build on)

This FOOP does not implement detachment, but — at the user's direction — establishes the
corrected vocabulary now, so FOOP-85 and any interim discussion use consistent names. The
existing FOOP-24 content is **renamed**, not respecified, as follows:

- **"Detachment" (unqualified, FOOP-24's primary `[patterns]<...>`/`[patterns]<<...>>` feature)
  is renamed to "Coordination detachment."** It governs how the detachment target is
  **coordinated** — i.e. it is a statement about the *resolution/copy behavior* (`SfCopy` vs.
  `Detach`) of candidates during a search that crosses the marker's boundary, not about whether
  they can be *discovered* at all. This is the feature FOOP-85 implements, built directly on
  Part 2 of this FOOP.
- **"Privacy detachment"** (FOOP-24's already-named deferred feature: detachment that prevents
  not just resolution but *discovery* — invisible even to anchored searches) is **unchanged in
  name and description**. It remains a distinct, deferred future feature, layered conceptually
  on top of coordination detachment (a privacy-detached candidate would need to be invisible to
  the Navigator entirely, not merely resolved differently by it) — not respecified here.
- **"Exclusive detachment"** (FOOP-24's deferred completeness-assertion idea: "these are the ONLY
  names that may become constanic in this scope") is **removed as a standalone future feature**
  and **reframed as one realization mechanism of a new, more general future feature: "Required
  Searches."** Required Searches is the idea that a brane's validity can demand that certain
  searches succeed (not stay ECONSTANIC) after coordination — "the entire brane is invalid unless
  these searches are found." Exclusive/coordination-detachment patterns are the most likely
  *mechanism* for expressing which searches are required (a detached pattern list already names
  a set of candidates the marker cares about; requiring them to resolve is a natural reading),
  but Required Searches is stated as the general feature and exclusive detachment as one instance
  of it, not the other way around. See the worked example immediately below.
- **"Strict detachment `[[…]]`"** remains backburnered exactly as FOOP-24's Appendix describes
  (the regex-intersection/undecidability argument is unaffected by this FOOP and is not
  reopened here).

**Worked example motivating Required Searches** (added at the user's direction): consider
`[a=10]<...>` where the actual candidate named `a` is currently `???` (NK, unresolved). Per §1.5
and the settlement rule, `??? == 10` does not force the detachment's value-gate to an
undecidable/forceful outcome — it is simply treated as **not matching** at that gate (the pattern
`a=10` does not catch a candidate whose value cannot be shown equal to `10`), and the candidate
falls through to whatever the next-outer marker or normal search behavior says (§2.3). This is a
deliberate, explicit **divergence from FOOP-33's `default_equal`**, which defines
`(NK, concrete) ⇒ Unknowable`, not `NotEqual` — detachment's value-gate comparison does not reuse
`default_equal`'s three-valued result directly; it collapses `Unknowable` to "gate does not fire"
specifically at this prefilter, while `default_equal` elsewhere (ordinary `=` comparisons,
`Value`/`NameValue` search predicates) keeps its real three-valued semantics unchanged. **This
divergence is exactly the motivating case for Required Searches**: under permissive coordination
detachment, an unresolvable `a` simply isn't caught by `[a=10]` and the search proceeds normally
(WOCONSTANIC-waits, may recoordinate later). Under a **future** Required Searches assertion that
declared `a=10` as required, the same situation — `a` unresolvable in the search path — would
instead be an **error condition**, because the assertion says the brane is invalid unless that
search is known to succeed, and an unresolvable value can never be shown to satisfy it. This
worked example should be preserved verbatim (or referenced) when Required Searches gets its own
FOOP.

## FIR Impact

**No new FIR kind added by this FOOP.** `StayFoolish`/`StayFullyFoolish` are unchanged in shape
(this FOOP does not add the `detachments` field — that is FOOP-85's work). `AncestralNavigator`
is a new Rust type implementing the existing `CandidateNavigator` trait — a traversal helper, not
a FIR.

**Changed:** the scan-outcome plumbing between the Navigator/scan loop and `SearchFir` must carry
a `CopyMode` per found candidate (today it carries none — the caller separately reads
`scope.has_ancestral_sfm`). This is a signature change to whatever internal type represents
`ScanOutcome::Found` (or an additional field alongside it) — exact shape is an implementation
decision for the plan, not fixed here, but it must reach `SearchFir::handle_found` and replace
the `scope.has_ancestral_sfm` argument at the `clone_stmt_result` call site
(`fir_kinds.rs:937`).

## UBC Step Impact

- `ab_search_with_engine` (`fir_kinds.rs:1085-1119`) is **removed**, replaced by constructing an
  `AncestralNavigator` and calling the existing `contextful_search_scan`/
  `contextful_search_scan_no_body_check`.
- `BraneFir::_ab_search` (`fir_kinds.rs:826-842`) is **removed** (or reduced to a thin
  compatibility shim if anything outside the engine still calls it directly — verify during
  planning) in favor of the same unified `AncestralNavigator` path.
- `SearchFir::handle_found` (`fir_kinds.rs:935-940`) changes its clone-mode source from
  `scope.has_ancestral_sfm` to the per-candidate `CopyMode` resolved during the scan (§2.5).
- `contexted_search_from_anchor`, `contextful_search_scan`, `SearchPredicate` — **unchanged**.
- `Scope` (`fir_trait.rs:55`) — **no new fields added** by this FOOP. `has_ancestral_sfm`'s
  continued necessity (or removal) is an Open Question, not resolved here.
- The `contexted && !anchored` dead path (§1.2) — **no behavior change**; a test is added to pin
  the existing silent-fallback behavior as intentional.

## Test Plan

- **Unit — `AncestralNavigator` order/completeness:** for a range of nested-brane fixtures
  (reuse or extend existing `ib_search`/`ab_search` unit-test fixtures), assert
  `AncestralNavigator` yields the same candidates, in the same order, as the current
  `ab_search_with_engine` loop and `BraneFir::_ab_search` recursion did — a direct
  before/after equivalence check, not just "some plausible order."
- **Unit — boundary-effect resolution against unparameterized markers:** a search under a single
  naked `<E>` (SF) yields every candidate on the crossed boundary tagged `CopyMode::SfCopy`;
  under naked `<<E>>` (SFF) `next_candidate()` yields **nothing** for every candidate on the
  crossed boundary (assert emptiness/exhaustion, not a tagged value — `Detach` never reaches the
  scan loop, §2.3); a search with no SF/SFF ancestor on its path yields `CopyMode::Normal`
  throughout. Nested naked markers (naked-in-naked combinations already expressible today)
  resolve consistently with innermost-first (trivial today since unparameterized markers are
  unconditional, but the test should exist so FOOP-85 has a baseline to extend).
- **Unit — bit-for-bit equivalence with `has_ancestral_sfm`:** run every existing SF/SFF-touching
  unit test through both the old code path (before this FOOP's changes) and the new `CopyMode`
  path, asserting identical `constanic_clone_at` output. This is the core regression guard for
  the refactor's "behavior-preserving" claim.
- **Unit — `contexted && !anchored` fallback:** `?name&#1` (unanchored, contexted-suffixed)
  parses without error and evaluates identically to `?name` alone; assert this explicitly rather
  than leaving it as an unremarked gap.
- **Approval — all existing snapshot/einmo tests pass unchanged.** No `.snap` files should
  require regeneration; if any diff appears, treat it as a semantic regression to investigate,
  not a formatting update to accept (per AGENTS.md's snapshot discipline).
- **`cargo clippy -D warnings` clean.**
- Comprehensive snapshot test `foop_84_comprehensive.foo` per `foop.md`'s mandate: exercise
  nested naked SF/SFF combinations, contexted search chained after both anchored and (per §1.2)
  unanchored searches, and AB walks crossing multiple brane levels with and without SF/SFF
  ancestors — demonstrating the refactor is transparent to ordinary programs.

## Rejected Alternatives

### A. Extend `_ib_search`/`_ab_search` with a matcher, override on SF/SFF markers (FOOP-24's
original Phase A plan)

Give `StayFoolishFir`/`StayFullyFoolishFir` their own `_ab_search` override that intercepts
parental search and returns `Found { sf_marked: true }` or `NotFound` directly. **Rejected**:
this inverts the dependency — the marker starts making search-outcome decisions, and every
future predicate feature (`!`, `&&`/`||`, find-all, characterization gates) would need to be
re-plumbed through an override that knows nothing about them, reproducing the six-parallel-
search-paths problem FOOP-24's own "Current Search Implementations" audit documents. Keeping the
marker passive (it owns configuration; the Navigator does the acting) avoids this.

### B. Thread the marker stack through `Scope` (FOOP-24's `active_detachments` field)

**Rejected**: `Scope` is built by `step_inner` from the *task-tree descent*, not the *lexical
AB-chain climb* a search performs by walking `_get_my_statement`/`_get_my_brane`. The two
traversals coincide today only because searches are normally stepped from within their lexical
position; recoordination (and later recursion, FOOP-34) can step a search from a task-queue
position whose descent path differs from its lexical ancestry, at which point an
ordering-sensitive stack (nested-marker resolution is order-dependent, §2.3) carried in `Scope`
can desynchronize from the actual boundaries the search logically crosses. Concentrating the
walk-and-decide logic in one object (`AncestralNavigator`) that performs both the traversal and
the boundary observation in the same pass removes the possibility of that desync entirely.

### C. Do nothing — build coordination detachment directly on today's two search
implementations, one-off

**Rejected**: this was the situation this FOOP responds to. It would require re-deriving
per-candidate, per-marker-level resolution ad hoc inside whichever of the two implementations
detachment is bolted onto, without the order/completeness testing discipline a dedicated
Navigator type gets for free, and without unblocking the duplication cleanup
(`ab_search_with_engine` vs. `BraneFir::_ab_search`) that already independently needed doing.

## Open Questions

- Does anything outside the search engine still legitimately need `Scope.has_ancestral_sfm` as a
  coarse "am I anywhere under any SF" signal, once `CopyMode` is resolved per-candidate at the
  clone call sites? If not, remove the field in this FOOP or a fast-follow; if so, document the
  remaining consumer.
- Exact Rust shape for threading `CopyMode` out of the scan loop to `SearchFir::handle_found` —
  a field on `ScanOutcome::Found`, a parallel return value, or a different mechanism. Left to the
  plan/implementation, not fixed by this spec.
- Whether `BraneFir::_ab_search` can be fully removed or must remain as a thin compatibility
  shim for a caller outside the engine — verify during planning by searching all call sites.
- Confirm no other code path (beyond the three already-audited call sites) branches on
  `self.contexted` without also checking `self.anchored`, so the §1.2 policy statement is
  complete.

## References

- Supersedes (on search semantics and detachment mechanism — see header note): FOOP-23 (search
  family spec — Part 1 of this FOOP absorbs and is now authoritative over FOOP-23's operator
  tables, `FoolRefFir` shape, name+value atomicity rule, and one-engine/cursor-source×predicate
  framing), FOOP-24 (detachment — "Current Search Implementations," "SF/SFF delegation
  discussion," "Implementation Plan → Phase A," "Nested markers" sections in particular).
  **FOOP-23 remains authoritative** for: the full grammar productions (lexer tokens, parser
  productions), the approval-test-input catalog (`value_search_forward_and_backward.foo`,
  `contexted_index.foo`, `mixed_chain_walk.foo`, and others under FOOP-23 Part C.4), its
  Rejected Alternatives (the `.=`/colon-notation/weak-reference/no-`&`/`&.` rejections), and its
  post-implementation bug-fix Appendix — none of that historical/implementation-detail material
  is restated here.
- Depends on: FOOP-43 (miss→ECONSTANIC; this FOOP relies on but does not modify it).
- Enables / to be built on this FOOP: FOOP-85 (Coordination detachment — reserved next number;
  not yet created) implements `Detachment`/`decide_to_detach`/`[patterns]` parsing on top of
  Part 2's `resolve_boundary_effect`/`CopyMode` mechanism, and cites Part 1 (§1.1a–d) for base
  search vocabulary instead of restating it. FOOP-93 (`!`, `&&`/`||`), FOOP-04 (`|` cascade),
  FOOP-14 (find-all) each extend the `SearchPredicate`/`CandidateNavigator` collaborators
  described in §1.1d/§2.1 and should land after this FOOP to build on the de-duplicated
  `AncestralNavigator` — see each FOOP's own text for what it now cites here rather than
  restates.
- Code: `foolish-ubca/src/fir_kinds.rs` — `mod contextful_search` (`:1680-2062`),
  `ab_search_with_engine` (`:1085-1119`), `BraneFir::_ab_search` (`:826-842`),
  `SearchFir::handle_found`/`clone_stmt_result` (`:935-940`, `:913-933`), `constanic_clone_at`
  (`:159-198` and recursive call sites), `contexted_search_from_anchor` (`:954-1009`).
  `foolish-ubca/src/fir_trait.rs` — `Scope` (`:55`), `step_inner` (`:375`).
  `foolish-parser/src/parser.rs` — `parse_postfix_expr` (`:578-777`, the `&` continuation
  branch at `:670`).
- Related planning doc: `docs/foop/NOTES-creation-lineage-and-search-family.md` (§2 Detachment,
  §9 boolean-combinator search, "Engineering guidance" section).

## Last Updated

**Date**: 2026-07-28 (2)
**Updated By**: Claude Code (Sonnet 5)
**Changes**: Two corrections to the initial draft, both from Atlas review. (1) **`Detach` is not
a third scan-visible value** — `CandidateNavigator::next_candidate()`'s contract
(`Option<(FirRef, usize)>`) has no side-channel for "yielded but treat as absent"; a Detached
candidate is filtered *inside* `next_candidate()`'s own loop and never returned, so the
scan-loop-visible `CopyMode` type only ever carries `Normal`/`SfCopy` (renamed the internal,
non-scan-visible three-way decision to `BoundaryEffect`; §2.3/§2.4 rewritten accordingly,
including the worked nested-marker example and the "fully-Detached settles ECONSTANIC via
ordinary exhaustion" note split into its own §2.4.1). (2) **Part 1 expanded into a genuinely
self-contained, comprehensive user-facing search reference** rather than a thin restatement: added
§1.1a (full operator table, all 18 forms), §1.1b (`FoolRefFir` shape), §1.1c (name+value
atomicity rule, restated with the worked counter-example), §1.1d (cursor-source×predicate,
the two collaborators, their correctness contracts, the two degeneracies) — all absorbed from
FOOP-23 at reference-table density so FOOP-93/FOOP-04/FOOP-14/FOOP-85 can cite this document
alone for shared search vocabulary instead of splitting attention across FOOP-23 and FOOP-84.
FOOP-23 remains authoritative only for grammar productions, the approval-test-input catalog,
Rejected Alternatives, and its bug-fix Appendix — none of that duplicated here. §2.1 and the
References section updated to point at the new Part 1 subsections precisely.

**Date**: 2026-07-28
**Updated By**: Claude Code (Sonnet 5)
**Changes**: Initial draft. Written after a multi-round design review with Atlas that corrected
an earlier same-session proposal on three points: (1) per-candidate `CopyMode` (not a binary
filter) must ride the same boundary-crossing decision that determines search visibility; (2)
nested-marker resolution is per-candidate, innermost-to-outward, first-*matching*-level wins —
not "first stack entry wins" as originally mis-stated; (3) a detachment value-gate comparing
against an NK candidate value is treated as non-match (not forceful-NK), which surfaced the need
for a documented, deliberate divergence from FOOP-33's `default_equal` and motivated pulling
"Required Searches" out as its own future-feature generalization of exclusive detachment. Also
confirmed directly against code: `ab_search_with_engine`/`BraneFir::_ab_search` duplication;
`contexted` always requires `anchored` at every runtime call site (`fir_kinds.rs:1192/1311/1511`,
no exception); contexted search cannot leave its home brane (`:954-1009`) — the combination
closes off any "value search gets extra context via continuation search" design direction.
Renamed detachment terminology per Atlas's direction: primary feature → "Coordination
detachment"; "Privacy detachment" unchanged; "Exclusive detachment" reframed as one mechanism
under a new future "Required Searches" feature, with a worked NK-value-gate example preserved
as its motivating case.
