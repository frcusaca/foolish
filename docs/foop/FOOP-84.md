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
> search works** in the UBCa reference implementation, and is written so that **a reader can
> finish it with complete working vocabulary and never need to open another document for a
> search-related question.** **Part 0 is the single definition site** for every term the search
> family uses — search context, the two search families, anchoring and what a miss proves, the
> detachment family (coordination vs. privacy vs. required searches vs. strict), marker scope, and
> engine vocabulary. Part 1 carries the full operator reference table, `FoolRefFir` shape, the
> name+value atomicity rule, and the cursor-source×predicate/two-collaborator framing, not just
> pointers to FOOP-23.
>
> **Downstream FOOPs cite, they do not restate.** On first use of any Part 0 term, write the term
> plus a pointer — "search context (FOOP-84 §0.3)" — then use the bare term. Do not redefine these
> terms elsewhere; change them here and the citations stay valid. This document is **deliberately
> redundant** with FOOP-23/FOOP-24/FOOP-43/AGENTS.md: consolidating their scattered definitions is
> the point, and the redundancy is intentional, not an oversight to be tidied away later. Where this document and FOOP-23/FOOP-24 disagree, this document wins;
> FOOP-23/FOOP-24 are retained for their historical design discussion and implementation-detail
> material (grammar productions, approval-test-input catalogs, Rejected Alternatives, bug-fix
> appendices) but should be read as superseded on anything restated here. This FOOP performs the
> core refactor in **two halves with different risk profiles, and they must not be conflated**:
> Part 1 (restatement) and §2.2 (Navigator unification) are **strictly behavior-preserving** — no
> snapshot may change. §2.3–§2.5 (per-candidate boundary evaluation) are a **deliberate semantic
> change** to how SF/SFF markers act, with expected SF/SFF snapshot churn enumerated in the Test
> Plan. Land them as separate commits. Unimplemented
> features (`||`/`&&` matcher booleans, `|` cascade, coordination detachment) are specified here
> **for refactoring purposes only** — as consumers of the facilities this FOOP builds — and are
> implemented in their own FOOPs (FOOP-93, FOOP-04, FOOP-24 — see "Related FOOPs" below).

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
   traversal. `SearchPredicate` is **untouched** by this change; whether
   `contextful_search_scan`/`CandidateNavigator` change signature depends on the §2.3.1 TBD
   (which collaborator resolves `CopyMode`).
3. **Introduces the per-candidate boundary-crossing evaluation** that stay-foolish markers use to
   affect a crossing search — the mechanism that (in a later FOOP) coordination detachment,
   privacy detachment, and required-searches are all built from. Its scope is deliberately
   narrow (§2.2.0): a marker affects **only** a backward/ancestral search **originating inside
   it**, and **only** where that search's AB climb **crosses the marker's own boundary outward**.
   Contexted (`&`) searches and searches that resolve without reaching the boundary are never
   affected. This FOOP does **not** implement any marker behavior yet (no `[patterns]` parsing, no
   `Detachment` struct) — it builds and tests the **evaluation shape** (`CopyMode`, per-candidate
   marker-stack scan, innermost-first) against the **existing**, unparameterized SF/SFF markers
   only, replacing today's `Scope.has_ancestral_sfm` boolean. **This is a semantic change, not a
   no-op**: the boolean is indexed on the *searcher* while `CopyMode` is indexed on the *boundary
   crossing*, and they diverge in cases reachable today (§2.5).
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

### Part 0 — Terminology (authoritative; cite this section, do not restate it)

**This Part is the single definition site for every term the search family uses.** It is
deliberately redundant with FOOP-23, FOOP-24, FOOP-43, and AGENTS.md — those documents' scattered
definitions are consolidated here so that a reader can finish Part 0 with complete working
vocabulary and never need to open another document for a search-related question.

**Convention for downstream FOOPs.** On **first use** of any term defined here, a later FOOP
writes the term followed by a pointer — e.g. "search context (FOOP-84 §0.3)" — and then uses the
bare term thereafter. Downstream FOOPs **must not redefine** these terms; if a definition needs to
change, change it here and the citations stay valid.

#### 0.1 Foundational terms

**Brane.** A containment structure holding an ordered sequence of statements. A brane is
*brane-like* (`is_brane_like`) if searches may descend into it.

**Statement.** A named or unnamed member of a brane, at a definite **statement number** (its
0-based position among the brane's `foolish_children`). A statement holds a **body** — the
expression it evaluates to.

**Home brane of a FIR** (synonym: **brane of a FIR**). The first brane reached by walking the
FIR's `.parent` chain; equivalently, the brane in which the FIR's statement has a correct
statement number. Accessor: `_get_my_brane` (`fir_trait.rs:216`). Use "home brane" when a second
brane is also under discussion; "brane of" otherwise.

**Candidate.** A statement that a search *considers* — offered to the predicate for a match
decision. Being a candidate implies nothing about matching; it means only that the traversal
reached it and did not filter it out.

**Constanic.** A FIR in any terminal NYES state: ECONSTANIC, WOCONSTANIC, CONSTANT, INDEPENDENT,
or NK. **Pre-constanic** (*nigh*): PREMBRYONIC, EMBRYONIC, BRANING — more stepping is appropriate.

**Coordination.** Referencing a resolved thing by name, producing a **constanic clone** that is
recoordinated into the referencing context. "Coordination frees everything": a coordinated value
is *just its value*, shorn of evaluation scaffolding — its markers are stripped
(`constanic_clone_at`, `fir_kinds.rs:159-198`) and its search context is stripped (§0.3, FOOP-43
Component 2).

#### 0.2 Anchoring, and what a miss proves

**Anchored search.** A search that names the brane it searches (`a?name`, `a.name`, `a#N`). It
resolves its anchor *through* to a whole brane, then searches that brane.

**Unanchored search.** A search with no anchor (`?name`, bare `name`), searching retrospectively
through IB then AB context.

**Miss.** A search whose candidate stream is **exhausted with no match**. What a miss *proves*
depends on anchoring, and this is why the two settle differently (FOOP-43 Component 1, restated
at §1.5):

- **Anchored miss → NK.** The anchor named the brane; exhausting it is a *proof* of absence.
  Terminal.
- **Unanchored miss → ECONSTANIC.** No fixed brane, so no proof; may gain a value via
  recoordination.
- **SFF-marked search → ECONSTANIC** regardless of anchoring (§0.5), carrying
  `EconstanicReason::Detached`.

**NK (`???`, "no-no").** Provably unknowable. Terminal. Reserved for genuine proofs: an anchored
miss, a *found* value that is itself NK, or a provable impossibility (`#N` out of range on a
settled finite brane). **NK is not "not found yet"** — preserving that distinction is why anchored
misses stay NK.

**ECONSTANIC** ("Exactly CONSTANt IN Context"). Searched, nothing found, *but the question may be
answerable later* — a recoordination into a richer context may give it a value.

#### 0.3 Search context — the formal definition

**Search context** (of a search result) is the pair:

> **(home brane, statement number)** — the brane in which the matched statement lives, *in its own
> context*, together with that statement's 0-based position within it.

Two requirements make this precise, and both are load-bearing:

1. **The brane must be the real, in-context brane** — the live structure with a correct `.parent`
   chain, reachable by walking the matched statement's ancestry. It is **not** a detached
   constanic clone, not a copy, and not a reconstruction. A clone has been severed from the
   ancestry that gives statement numbers and further searches their meaning, so a clone cannot
   serve as search context.
2. **The statement number must be that statement's position in that brane** — the index at which
   the brane actually holds it, such that `brane.stmt_at(n)` returns the matched statement.

**What carries it.** Search context is carried by a **`FoolRefFir`** — an immutable FIR wrapping a
**strong** `Rc` to the *original matched statement* (not a clone), with its parent chain, line
number, and home brane intact. Strong, so the statement stays reachable through the result even if
its home brane is later restructured. Born CONSTANT, takes no steps, holds no children of its own,
and is **invisible to values** (`FirRefExt::value`, result-chain walking, and the Humanizing
Sequencer all read `ubc_children[0]` only — a `FoolRefFir` never appears in HFS output). Shape:

```rust
pub struct FoolRefFir {
    pub(crate) core: ProtoBrane,   // no foolish_children, no ubc_children
    referent: FirRef,              // strong Rc — the ORIGINAL statement
}
```

**Where it lives — the two-child invariant.** A resolved search has exactly **two**
`ubc_children`:

| slot | content |
|------|---------|
| `[0]` | the constanic clone of the matched statement's **body** — the search's *value* |
| `[1]` | a **`FoolRefFir`** holding the matched statement — the search's *search context* |

Built by `push_search_result_pair` (`fir_kinds.rs:1667`). This split is what makes "providing
context" universal: **every** search result carries a position a following contexted search can
resume from, while values remain unpolluted by it.

**Providing vs. reading context.** A search **provides** context (every search result does, via
`[1]`). A search **reads** context only if it is contexted (§0.4). Contextless searches provide
but do not read.

**Coordination removes search context.** Referencing a search result by name coordinates it: the
clone keeps `[0]` and drops `[1]` (FOOP-43 Component 2). A coordinated value is *positionless*, so
a contexted search off it has nothing to resume from. This is why `{a = ?x; b = a&=3}` makes `b`
NK.

#### 0.4 The two search families

**Contextless anchored search** (shorthand: **contextless search**, or plainly **search**):
`.` `?` `~` `#` `^` `$` `~=` `?=`. Resolves its anchor *through* to a whole brane and searches that
brane. **Does not read context** — it does not start from a statement position. Full table at
§1.1a.

**Contexted anchored search** (shorthand: **`&`-search**, or **contexted search**):
`&?` `&~` `&#` `&^` `&$` `&~=` `&?=`. **Reads the search context** of the preceding result and
searches forward/backward from that statement number, **within that statement's home brane**.
Contexted searches stack (`a~step_1 &#1`).

**The chaining rule.** `.` always **deepens** (searches *inside* the resolved brane); `&` always
**navigates** from a position (searches *near* a matched statement, within its home brane). This
resolves the `a.brane_field.x` ambiguity: contextless deepens, contexted navigates neighbors.

**Clipping.** A contexted search is **clipped to its home brane** — it scans `[0, p-1]` or
`[p+1, len-1]` of that one brane and **never** walks to a parent or ancestral brane
(`contexted_search_from_anchor`, `fir_kinds.rs:954-1009`).

**`contexted ⟹ anchored`.** A contexted search resumes from a `FoolRefFir`, and only an anchored
search carries one — so contexted-off-unanchored is structurally impossible, not merely
unimplemented. See §1.2.

#### 0.5 Stay-foolish markers and the detachment family

**Stay-foolish marker.** `<E>` (SF) or `<<E>>` (SFF) — a wrapper that changes how searches
*inside* `E` interact with the world *outside* `E`.

- **SF `<E>`** — *stay foolish*. Candidates are **found**, but their constanic constituents are
  copied **as-is, without recoordination** (`CopyMode::SfCopy`). The result keeps its expression
  form (`10+s`) rather than its fully-evaluated value (`11`).
- **SFF `<<E>>`** — *stay fully foolish*. Candidates are **withheld from the search entirely**
  (`Detach`); a search that finds nothing under this settles ECONSTANIC (§0.2), remaining
  resolvable at each later use site.

**Detachment.** The general mechanism of which SF/SFF are the unparameterized cases: a marker
*parameterized* with patterns, `[p1,p2,…]<E>` or `[p1,p2,…]<<E>>`, affecting **only** the
candidates its patterns select. Bare markers are full detachment (`<E>` ≡ `[*]<E>`, `<<E>>` ≡
`[*]<<E>>`); `[]` detaches nothing.

The family — **use these names exactly**:

| Term | Meaning | Status |
|------|---------|--------|
| **Coordination detachment** | Governs how a candidate is **coordinated** — the resolution/copy behavior (`SfCopy` vs `Detach`) of candidates during a boundary-crossing search. The candidate remains **discoverable**; only its resolution changes. | **FOOP-24** — the live spec |
| **Privacy detachment** | Goes further: prevents **discovery**, not merely resolution. A privacy-detached candidate is invisible to searches *entirely*, including anchored ones — information hiding. Would require the candidate to be invisible to the traversal itself, not merely resolved differently by it. | Deferred, future FOOP |
| **Required Searches** | A brane's validity may *demand* that certain searches succeed (not stay ECONSTANIC) after coordination — "the entire brane is invalid unless these searches are found." **Exclusive detachment** (FOOP-24's "these are the ONLY names that may become constanic") is one realization mechanism of it, not a separate feature. | Deferred, future FOOP |
| **Strict detachment `[[…]]`** | A completeness assertion forbidding unexplained non-resolution. Backburnered — the semantics are undecidable in scope (regex-intersection argument, FOOP-24 Appendix). | Backburnered |

**The distinction that matters most:** coordination detachment changes *how a candidate resolves*;
privacy detachment changes *whether it can be seen at all*. Everything specified in this FOOP and
in FOOP-24 is **coordination** detachment. Privacy detachment is layered conceptually on top and
is not specified anywhere yet.

#### 0.6 Marker scope — three conditions (the most commonly mis-stated rule)

A marker affects a search **only** when all three hold. Stated here in Part 0 because nearly every
mis-reading of the detachment family comes from assuming markers are ambient scope:

1. **Descendant-only** — the search must originate lexically *inside* the marker's `E`.
2. **Outward-crossing only** — the marker is consulted only where the search's ancestral climb
   *leaves* the marker, inside → outside.
3. **Backward/ancestral only** — only the outward AB climb crosses boundaries. **Contexted (`&`)
   searches are never affected** (clipped to home brane, §0.4); intra-brane scans that never climb
   are never affected.

**A marker cannot reach sideways or downward, and cannot affect a search that resolves locally.**
In `[a]<{ x = a; y = local }>` with `local` defined inside the marker's own brane, the search for
`local` never crosses the boundary and is never tested against `[a]` at all. Engine statement and
worked examples: §2.2.0.

#### 0.7 Engine vocabulary

**Candidate Navigator** (`CandidateNavigator`) — traverses the FIR tree and yields candidates in
the mandated order. Correctness contract: **correctly ordered** (the one order the configured
semantics mandate) and **complete** (every reachable candidate, exactly once, then stops). Knows
nothing about what is being matched.

**Statement Matcher** (`SearchPredicate::matches`) — given one candidate, approves or rejects it.
Receives the *full* statement FIR (name, body/value, statement number, parent/home brane, NYES),
not a projection. Knows nothing about traversal order.

**Cursor-source** — where the Navigator starts: *Contextless* (anchor resolved to a brane, cursor
at front/rear) or *Contexted* (the incoming result's search context). The **only** step that
differs between the two families.

**`CopyMode`** — `Normal` (ordinary recoordinating copy) or `SfCopy` (copy constanic constituents
without recoordination). Resolved **per candidate**, carried by the search result (§2.3.1).

**`BoundaryEffect`** — the internal three-way decision at a boundary crossing: `Pass` (no marker
decided anything), `SfCopy`, or `Detach` (candidate withheld — filtered *before* yield, never seen
by the scan loop). Not the same type as `CopyMode`; see §2.3.

**Home brane vs. boundary crossing — do not conflate.** The AB walk *crosses* brane boundaries
outward through the lexical parent chain; a contexted search *stays inside* one brane and moves
among siblings. Markers act on the former only.

---

### Part 1 — Search semantics, restated (supersedes FOOP-23)

This section is the complete, authoritative description of how Foolish search works. It restates
(does not merely reference) FOOP-23's content because FOOP-23's description is no longer
sufficient on its own — the boundary-crossing / detachment interaction it did not anticipate is
now part of the model.

#### 1.1 Three groups of search operators

The two families and the `.` vs `&` chaining rule are **defined in §0.4**. Summarized for
orientation, with the third group added:

1. **Contextless anchored searches** — `.` `?` `~` `#` `^` `$` `~=` `?=` (§0.4).
2. **Contexted anchored searches** (`&`-searches) — `&?` `&~` `&#` `&^` `&$` `&~=` `&?=` (§0.4).
3. **Value searches** — triggered by `=`, matching a statement's *value* rather than its name. May
   be combined with a name pattern into an **atomic conjunctive operator** (`~name=value`, §1.1c)
   or written bare (`&=`-search shorthand for a contexted value search).

#### 1.1a Full operator reference table

The complete surface syntax, consolidated from FOOP-23 Parts A/C so downstream FOOPs (FOOP-93,
FOOP-04, FOOP-14, FOOP-24) can cite one table instead of splitting attention across two
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

#### 1.1b `FoolRefFir` and the two-child invariant

**Defined in §0.3** (search context, its `FoolRefFir` carrier, the two-child invariant, and the
coordination-strips-context rule). Not restated here — §0.3 is the definition site, and §1.4 below
notes the invariant's role in the chaining rules.

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
properties — **cursor-source × predicate**, both defined in §0.7 along with the two collaborators
(Candidate Navigator, Statement Matcher) and their correctness contracts. This is the "one-engine
model" §2.1 builds on directly.

Two points worth emphasizing beyond the §0.7 definitions:

- **Cursor-source is the *only* step that differs between the two families** (§1.1a groups 1–13
  vs. 14–18); everything downstream is shared. That is what makes it one engine rather than two.
- **The Matcher receives the full statement FIR**, not a projection — different predicates need
  different facets, and handing the whole candidate forward is what lets predicates compose
  (FOOP-93's `And`/`Or`/`negate` trees). The `SearchPredicate` variants are extended by FOOP-93
  with negation and boolean composition; FOOP-24's detachment acts in the *Navigator*, not here
  (§2.3).

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

#### 1.4 The FoolRefFir two-child invariant

**Defined in §0.3.** In brief: `[0]` is the value (constanic clone of the matched statement's
body), `[1]` is the `FoolRefFir` carrying search context. Universal "providing context" and the
`contexted ⟹ anchored` rule of §1.2 both rest on this.

#### 1.5 NK vs ECONSTANIC miss outcomes (restated, unchanged — see FOOP-43 for the full
settlement rule)

Settlement depends on **how the search was anchored**, and on whether an SF/SFF marker withheld
its candidates:

- **Anchored miss → NK.** An anchored search names the brane it searches; exhausting that brane
  with no match *proves* the name is absent from it. Terminal. This is the long-standing rule in
  FOOP-23 and AGENTS.md, and it **stands** — NK keeps its precise meaning, "provably unknowable,"
  rather than blurring into "not found yet."
- **Unanchored miss → ECONSTANIC.** No fixed brane, so no proof of absence; may recoordinate.
- **SFF-marked search → ECONSTANIC, regardless of anchoring**, carrying
  `EconstanicReason::Detached`. A search whose candidates a marker withheld did not fail to find
  anything — it was prevented from looking, so it must stay deferrable.

Found-`???` → NK propagates (terminal). This FOOP does not change FOOP-43; it is restated here
because the boundary-crossing walk (§2) must know what an exhausted candidate stream produces —
and critically, **plain exhaustion is not sufficient** to reach ECONSTANIC for an anchored search.
See §2.4.1, which depends on this distinction.

> **Dependency note.** FOOP-43 is still `status: Draft`, `begun: [ ]`. This FOOP treats its
> Component 1 and Component 3 (the `EconstanicReason` tag) as settled background, so **FOOP-43
> must land before this FOOP**, per the index's implementation order. An earlier revision of this
> section asserted "anchored miss → ECONSTANIC, superseding the older anchored-miss→NK rule" —
> that was wrong and is corrected above.

### Part 2 — The unified Navigator and per-candidate boundary evaluation

#### 2.1 The one-engine model, unchanged at its core

The engine described in §1.1d (cursor-source × predicate; `CandidateNavigator` ×
`SearchPredicate`) is **not modified in its shape** by this FOOP. `SearchPredicate`
(Name/Value/NameValue/Index/Head/Tail) is **untouched**, and the scan loop's *logic* — iterate
candidates from a `CandidateNavigator`, apply the predicate, return `Found`/`NkStop`/`Miss` — is
unchanged.

**Caveat (§2.3.1 TBD):** whether `contextful_search_scan` and the `CandidateNavigator` trait keep
their exact current *signatures* depends on which collaborator ends up resolving `CopyMode`. A
search result must carry its copy mode (§2.3.1), and if the Navigator supplies it, both the trait
return type and the scan loop's destructuring change even though neither's logic does. Do not read
"untouched" as "no signature change" until that TBD is settled in the plan.

This separation is deliberate: FOOP-93's predicate-tree extensions (`!`, `&&`/`||`), FOOP-14's
collect-mode scan, and FOOP-43's reason tags all land inside these two collaborators exactly as
their own specs already describe, with zero interaction with anything in this FOOP. What this FOOP adds is a new
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

##### 2.2.0 Scope rule — what a marker can and cannot affect (READ THIS FIRST)

**A stay-foolish marker affects exactly one thing: a backward search, originating inside the
marker, at the moment it crosses the marker's own boundary outward.** Nothing else. Stated as
three conditions that must *all* hold before a marker is even consulted for a candidate:

1. **Descendant-only.** The searching FIR must be lexically *inside* the marker's expression. A
   marker never affects a search that originates outside it, and never affects a search in a
   sibling or unrelated brane. A marker is not ambient context — it is a property of a boundary
   the search walks through.
2. **Outward-crossing only.** The marker applies only where the search's AB walk *leaves* the
   marker's boundary, climbing from inside to outside. A search that finds its answer without
   ever reaching the marker's boundary is entirely unaffected, even though it is lexically under
   the marker.
3. **Backward/ancestral searches only.** Only the outward AB climb crosses boundaries at all.
   **Contextless intra-brane scans and contexted (`&`) searches are never affected**, because
   contexted search is clipped to its home brane (§1.3) and never leaves it. There is no forward
   unanchored form to consider (§1.1a). In engine terms: markers live in `AncestralNavigator`
   only — `BraneNavigator` and `contexted_search_from_anchor` are untouched by this mechanism.

The practical consequence, worth internalizing because it makes the whole feature much smaller
than it first appears: **a marker cannot reach sideways or downward, and it cannot affect a
search that resolves locally.** In `[a]<{ x = a; y = local }>` where `local` is defined inside the
marker's own brane, the search for `local` never crosses the marker boundary and so is never
tested against `[a]` at all — regardless of whether the pattern would have matched its name.

This scope rule is what keeps the mechanism orthogonal to FOOP-93 (predicates), FOOP-04 (cascade),
and FOOP-14 (collect-mode): those act on candidates or scan modes, this acts on one boundary
crossing in one Navigator.

##### 2.2.1 Boundary crossing is where markers are seen

As `AncestralNavigator` steps from a child brane to its parent, it inspects the FIR whose boundary
it just crossed. If that FIR is a `StayFoolish` or `StayFullyFoolish` marker, the Navigator becomes
aware of it — this is the hook point for §2.3, subject to the scope rule in §2.2.0 above. Because the Navigator crosses markers **innermost-first** (it starts at the
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

**Scope reminder (§2.2.0):** the `marker_stack` below contains **only** markers whose boundary
this particular search actually crosses on its outward AB climb — i.e. markers the searching FIR
is lexically inside, encountered as the walk leaves them. It is not "every marker in the program,"
nor "every marker enclosing the candidate," nor "every marker enclosing the search." A search that
resolves without reaching any marker boundary has an **empty** `marker_stack` and always yields
`Pass`. Contexted (`&`) searches never reach this code at all (§1.3).

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

##### 2.3.1 What a candidate carries, and where `CopyMode` is resolved (one fixed, one TBD)

**Fixed — a search result carries its copy mode.** Whenever a search produces a result, that
result must carry the `CopyMode` that was resolved for the found candidate. This is a requirement
on the *result*, not a suggestion about plumbing: `SearchFir::handle_found` needs it to choose the
clone behavior (§2.5), and it must be the mode resolved for **that specific candidate**, not a
property re-derived from the searcher's lexical position afterward. A result with no copy mode is
ill-formed. (Where no marker boundary was crossed, the mode is `Normal` — §2.2.0.)

**Fixed — a candidate is more than a `(FirRef, usize)` pair.** A search result already carries
*position* so that a following contexted (`&`) search can continue from it: the `FoolRefFir` at
`ubc_children[1]`, holding the original statement with its home brane and line number intact
(§1.4), which `contexted_search_from_anchor` reads (`fir_kinds.rs:954-1009`). So the information
travelling with a candidate is, at minimum:

- the **candidate FIR** itself;
- its **home brane** — the context a continuation search needs;
- its **statement number** within that brane;
- its **`CopyMode`**.

Today's `next_candidate() -> Option<(FirRef, usize)>` (`fir_kinds.rs:1941`) carries the first and
third; the brane is re-derived downstream via `_get_my_brane`, which is exactly the reconstruction
an `AncestralNavigator` makes unreliable, since it yields candidates from *several* branes as it
climbs. Carrying the brane explicitly with the candidate is therefore preferred to re-deriving it.

**TBD — which collaborator resolves `CopyMode`.** Whether the Navigator (`BraneNavigator`/
`AncestralNavigator`) attaches the mode as it yields, or the contextful-search layer resolves it
around the scan, is **deliberately left open** by this spec. Both can satisfy the fixed
requirements above; they trade differently:

- *Navigator-resolved* keeps the boundary walk and the boundary decision in one object (the
  argument behind Rejected Alternative B), but puts `CopyMode` in a trait every navigator
  implements — including `BraneNavigator`, which per §2.2.0 can only ever produce `Normal`.
- *Search-layer-resolved* keeps `CandidateNavigator` narrow and lets the marker concern live
  beside the settlement logic that consumes it, at the cost of splitting the walk from the
  decision.

The plan decides this, and the decision determines whether `contextful_search_scan` and
`CandidateNavigator` change signature — so the "scan loop and predicate are untouched" claim in
§2.1/UBC Step Impact holds only under some resolutions of this TBD, and must be re-checked once
it is settled. What **is** fixed here is the *channel split* (`Detach` filtered pre-yield;
`Pass`/`SfCopy` travelling with the result) and the carried-context requirement above — not the
Rust syntax or the owning collaborator.

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
**not implemented in this FOOP** — it is coordination detachment's own concern (FOOP-24, see
"Related FOOPs"). What **this** FOOP implements and tests is the resolution *algorithm*
(`resolve_boundary_effect`) and the `CopyMode` plumbing for its `Pass`/`SfCopy` outcomes through
to the clone call sites (§2.5), exercised against **today's unparameterized SF/SFF markers
only**, where `rule_applies_to` is trivially "always applies" (an unparameterized marker has no
pattern to test against — it always fires for every candidate **that reaches its boundary
crossing**, per §2.2.0). Note this is *not* the same set of candidates as today's
`has_ancestral_sfm` covers (§2.5) — the mechanism is independently testable before any new syntax
exists, but it is not behavior-preserving.

#### 2.4 Naked/unparameterized SF and SFF stay their own thing

Per the existing FOOP-24 design (kept unchanged in intent, restated in this mechanism's terms):
**naked `<E>` and `<<E>>` are not represented as a degenerate case of a general pattern-matching
`Detachment` — they remain their own, separate marker behavior**, with their own `FirKind`
variants (`StayFoolish`, `StayFullyFoolish`) as today. In the `resolve_boundary_effect`
algorithm, an unparameterized `StayFoolish` marker's `rule_applies_to` is unconditionally true
(it always fires) and its `effect()` is `SfCopy`; an unparameterized `StayFullyFoolish` marker's
is unconditionally true and `Detach`.

**"Unconditionally true" is scoped by §2.2.0, and the distinction matters.** It means *"fires for
every candidate that reaches this marker's boundary crossing"* — **not** *"fires for every search
lexically under the marker."* A search inside a naked `<<E>>` that resolves entirely within its own
brane never crosses the marker boundary, so the marker is never consulted and the search resolves
normally. Only candidates the search must climb *past* the marker to reach are Detached. Reading
"unconditional" as "blanket" is the single easiest way to mis-implement this section.

For SF, every candidate reached by climbing past a naked `<E>` is yielded tagged `SfCopy`. Note
this is **narrower than today's `scope.has_ancestral_sfm`**, which is a property of the *searcher*
(set by `step_inner` for any search lexically under an SF wrapper, `fir_trait.rs:387-388`) rather
than of the boundary crossing — see §2.5 for what that difference costs.

Both are expressed in the same per-candidate machinery that a later, parameterized detachment
marker will extend. **When coordination detachment (FOOP-24) adds
`[patterns]<...>`/`[patterns]<<...>>`, that is additional, separate marker configuration — not a
rewrite of the naked-marker path.** This preserves FOOP-24's "new code is only for specific
detachments" intent, grounding it in the shared per-candidate algorithm instead of a two-tier
implementation split.

##### 2.4.1 A fully-Detached search settles ECONSTANIC — and needs a reason tag to do so

A search entirely under a naked `<<E>>` (or, once FOOP-24 lands, a `[*]<<E>>`) has every raw
candidate on its walked path resolve to `Detach`. Mechanically, `next_candidate()` never returns
anything — its internal loop discards every raw candidate it considers and eventually runs out,
returning `None`. From `contextful_search_scan`'s point of view the `while let Some(...)` loop
never executes its body, and the function falls through to `ScanOutcome::Miss`
(`fir_kinds.rs:2040`).

**But `Miss` alone is not enough to settle correctly**, and an earlier revision of this section
got this wrong. Under FOOP-43's settlement rule (§1.5), a **`Miss` on an anchored search settles
NK**, not ECONSTANIC. If a fully-Detached anchored search produced a bare `Miss`, it would settle
NK — terminal, unrecoordinatable — which is precisely the opposite of what a stay-fully-foolish
marker means. The marker exists to *defer* resolution to the use site; settling NK would make it
*destroy* the search instead.

**Therefore the Detached case must be distinguishable from a genuine miss at the settlement
site.** Exhaustion-with-every-candidate-filtered and exhaustion-with-no-candidate-matching are
different events that happen to produce the same `ScanOutcome` today, and the settlement logic
needs to tell them apart. Concretely, one of:

- the Navigator records that it filtered at least one candidate (or that a marker was in force at
  all), and the scan outcome carries that fact — e.g. a `ScanOutcome::DetachedMiss` variant, or a
  flag alongside `Miss`; or
- the search's settlement consults the marker context directly, independent of the scan outcome.

Either way, a Detached exhaustion settles **ECONSTANIC with `EconstanicReason::Detached`**
(FOOP-43 Component 3), while a genuine exhaustion keeps today's anchored→NK / unanchored→ECONSTANIC
split. The exact mechanism is an implementation decision for the plan; what is fixed here is that
**a distinguished signal is required** — the prior claim that "no special-case reject-all signal
is needed anywhere in the scan loop" is withdrawn.

This makes FOOP-43 Component 3 (`EconstanicReason`) a **hard prerequisite** of this FOOP's Part 2,
not merely an adjacent nicety.

#### 2.5 `CopyMode` replaces `Scope.has_ancestral_sfm` at the clone call sites

Today, `SearchFir::handle_found` (`fir_kinds.rs:935-940`) passes `scope.has_ancestral_sfm` — a
single boolean, constant for the whole search regardless of which candidate was found — into
`clone_stmt_result` → `constanic_clone_at`'s `descendent_of_sfm_and_foolishly_ignorant` parameter
(`fir_kinds.rs:159-198` and its recursive call sites throughout the same function).

**This FOOP changes `handle_found` (and the scan-outcome plumbing feeding it) to carry the
`CopyMode` resolved for the specific candidate that was found**, and to pass
`copy_mode == CopyMode::SfCopy` as the `descendent_of_sfm_and_foolishly_ignorant` argument,
instead of the blanket `scope.has_ancestral_sfm`.

**This is a real, intentional behavior change, and it diverges from today even with no
parameterized patterns in play.** An earlier revision of this section claimed the two "coincide
exactly" for unparameterized markers, with divergence deferred to FOOP-85. That was wrong. The two
are **indexed differently**:

- `scope.has_ancestral_sfm` is a property of **the searcher** — `step_inner` sets it for any
  search lexically under an SF wrapper (`fir_trait.rs:387-388`) and propagates it down the task
  tree. It answers *"is this search anywhere under an SF marker?"*
- `CopyMode` is a property of **the boundary crossing that reached the found candidate** — per
  §2.2.0, it fires only where the search's AB climb actually leaves the marker.

These disagree in a case reachable **today**, with naked markers only: a search inside `<E>` that
finds a candidate **without crossing the marker boundary** (the candidate is in the marker's own
brane, or nearer). Today it is foolishly-ignorant-copied because the searcher is under SF. Under
§2.2.0 it is `Normal`, because no boundary was crossed. The new rule is the more precise one —
that is the point of the refinement — but it **is** a change.

A second divergence, also reachable today: `has_ancestral_sfm` is set **only** for
`FirKind::StayFoolish`, never for `StayFullyFoolish`. So SFF's behavior under §2.3–§2.4 is not
a re-expression of any existing boolean at all.

**Consequences for the Test Plan.** The previously-specified "bit-for-bit equivalence with today's
`has_ancestral_sfm` for every existing SF/SFF test" assertion **cannot hold and is withdrawn**.
What ships instead: an enumeration of exactly which SF/SFF cases change and why, with the affected
approved snapshots regenerated and presented for human semantic review per AGENTS.md — never
auto-accepted. See the Test Plan for the specific files expected to move.

#### 2.6 Nested-marker resolution is answered by this mechanism (resolves FOOP-24's open question)

FOOP-24's "Nested markers" section (status: UNDECIDED) asked whether resolving nested SF/SFF
requires "reversing the search: match on everything first, then check the stack of detachments
from innermost outward." **Answer: no reversal is needed.** §2.3's algorithm walks the marker
stack innermost-to-outward *as part of yielding each candidate* — there is no separate "match
everything first" phase. The worked example in §2.3 is the resolution FOOP-24 asked for. This
FOOP formally closes that open question; FOOP-24 (coordination detachment) should cite this
section rather than re-deriving it.

### Part 3 — Detachment terminology (renames, for FOOP-24 to adopt)

This FOOP does not implement detachment — **FOOP-24 is the coordination detachment FOOP** and
remains the live specification for the feature. What this Part establishes is the corrected
*vocabulary*, so FOOP-24 and any interim discussion use consistent names. FOOP-24's content is
**renamed, not respecified**, as follows:

**The definitions live in §0.5** (the detachment-family table). This Part records the *renames*
against FOOP-24's original vocabulary, so a reader of the older document can map its terms
forward:

| FOOP-24's original term | Now called | Change |
|---|---|---|
| "Detachment" (unqualified — the `[patterns]<...>`/`[patterns]<<...>>` feature) | **Coordination detachment** (§0.5) | Renamed only. Same feature, same semantics, still specified in FOOP-24 — now built on Part 2's mechanism rather than FOOP-24's superseded Phase A plan. |
| "Privacy detachment" | **Privacy detachment** (§0.5) | Unchanged in name and description. Still deferred. |
| "Exclusive detachment" | one mechanism under **Required Searches** (§0.5) | Reframed. Removed as a standalone future feature; the general feature is Required Searches, and exclusive detachment is one way to express which searches are required — not the other way around. |
| "Strict detachment `[[…]]`" | **Strict detachment** (§0.5) | Unchanged; remains backburnered on FOOP-24's regex-intersection/undecidability argument, which this FOOP does not reopen. |

The naming rationale for the first row is worth stating once: the feature governs how a candidate
is **coordinated** (its resolution/copy behavior) and *not* whether it can be **discovered**. That
is precisely what separates it from privacy detachment, and it is why the qualifier is needed —
"detachment" unqualified had been doing both jobs.

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
(this FOOP does not add the `detachments` field — that is FOOP-24's work). `AncestralNavigator`
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
- `contexted_search_from_anchor` and `SearchPredicate` — **unchanged**.
  `contextful_search_scan` — logic unchanged; **signature depends on the §2.3.1 TBD** (if the
  Navigator supplies `CopyMode`, both the `CandidateNavigator` trait return type and this
  function's destructuring change, along with `BraneNavigator` and the navigator unit tests at
  `fir_kinds.rs:4663-4719`).
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
  unconditional, but the test should exist so FOOP-24 has a baseline to extend).
- **Unit — enumerate the SF/SFF divergences (replaces the withdrawn bit-for-bit test):** per
  §2.5, `CopyMode` and `has_ancestral_sfm` are indexed differently and disagree today. Write
  explicit tests for each divergence rather than asserting equivalence: (a) a search under `<E>`
  finding a candidate **without crossing** the marker boundary → `Normal` under the new rule
  (was foolishly-ignorant-copied); (b) a search under `<E>` finding a candidate **by crossing**
  the boundary → `SfCopy` (agrees with today); (c) SFF cases, which `has_ancestral_sfm` never
  covered at all. Each test states the old and new outcome side by side.
- **Unit — scope rule (§2.2.0):** a search lexically under a marker that resolves **within its own
  brane** is unaffected by the marker; a contexted (`&`) search is never affected (§1.3); a search
  originating **outside** a marker is never affected by it. These pin the three scope conditions
  and are the guard against the "markers are ambient context" mis-implementation.
- **Unit — `contexted && !anchored` fallback:** `?name&#1` (unanchored, contexted-suffixed)
  parses without error and evaluates identically to `?name` alone; assert this explicitly rather
  than leaving it as an unremarked gap.
- **Approval — split by part, because the two halves differ.**
  - **Part 1 + §2.2 (Navigator unification): no snapshot may change.** These are genuinely
    behavior-preserving. Any diff here is a semantic regression to investigate, not a formatting
    update to accept.
  - **§2.3–§2.5 (per-candidate boundary evaluation): SF/SFF snapshots are expected to change.**
    Per §2.5 the new rule is deliberately more precise than `has_ancestral_sfm`, and per §2.4
    naked SFF now routes through the marker path. Candidate files to review:
    `sff_basic`, `sff_nested`, `sff_vs_sf_timing_difference`, `sff_resolves_on_each_use`,
    `sff_in_binary_op`, `sff_in_assignment_chain`, `sf_of_sff`, `sf_sff_nested_combined`,
    `complex_sff_with_nested_scope`, `complex_sff_in_nested_brane`. Each diff must be justified
    against §2.2.0's scope rule before being presented for human review. **Never auto-accept**
    (AGENTS.md).
  - Landing the two halves as **separate commits** (unification first, verified snapshot-clean;
    then the boundary mechanism) makes this split reviewable and is strongly recommended in the
    plan.
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
- **(§2.3.1 TBD — the load-bearing one.)** Which collaborator resolves `CopyMode`: the Navigator
  (attached as it yields) or the contextful-search layer (resolved around the scan)? Both satisfy
  the fixed requirement that a search result carries its copy mode; they differ in whether
  `CandidateNavigator`/`contextful_search_scan` change signature, and therefore in whether §2.1's
  "untouched" claim survives. Settle this in the plan **before** coding, since it determines the
  blast radius. Related: the exact Rust shape for carrying the mode through to
  `SearchFir::handle_found` — a field on `ScanOutcome::Found`, a richer candidate struct, or a
  parallel return value.
- Should the candidate/result type carry the **home brane** explicitly (§2.3.1) rather than
  re-deriving it via `_get_my_brane`? Preferred yes, because `AncestralNavigator` yields
  candidates from several branes as it climbs, but confirm against the `FoolRefFir` path that
  already stores the original statement.
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
- Enables / to be built on this FOOP: **FOOP-24 (Coordination detachment — the live spec for the
  feature)** implements `Detachment`/`decide_to_detach`/`[patterns]` parsing on top of Part 2's
  `resolve_boundary_effect`/`CopyMode` mechanism, and cites Part 1 (§1.1a–d) for base search
  vocabulary instead of restating it. (A separate "FOOP-85" was briefly reserved for this in the
  2026-07-28 draft and is **withdrawn** — it split a live feature from its own specification, and
  85 is not a valid next number under little-endian numbering; `gen_next` yields FOOP-94.) FOOP-93 (`!`, `&&`/`||`), FOOP-04 (`|` cascade),
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

**Date**: 2026-07-28 (5)
**Updated By**: Claude Code (Opus 5)
**Changes**: **Added Part 0 — Terminology**, the single definition site for the whole search
family, so a reader can finish this FOOP with complete working vocabulary and never open another
document for a search question. Sections: §0.1 foundational terms (brane, statement, home brane,
candidate, constanic, coordination); §0.2 anchoring and **what a miss proves** (anchored→NK,
unanchored→ECONSTANIC, SFF-marked→ECONSTANIC; why NK must keep meaning "provably unknowable");
**§0.3 formal definition of search context** — the pair **(home brane *in its own context*,
statement number)**, with both requirements made explicit (the brane must be the live in-context
structure with a correct `.parent` chain, **not a detached constanic clone**; the statement number
must be the position at which that brane actually holds the matched statement), its `FoolRefFir`
carrier and shape, the two-child invariant table, providing-vs-reading context, and the
coordination-strips-context rule; §0.4 the two search families, chaining rule, clipping,
`contexted ⟹ anchored`; **§0.5 the detachment family table** — coordination vs. privacy vs.
Required Searches vs. strict, with the discoverability-vs-resolution distinction stated as the one
that matters most; §0.6 marker scope (the three conditions); §0.7 engine vocabulary. Established
the **citation convention**: downstream FOOPs cite "term (FOOP-84 §0.x)" on first use and never
redefine. De-duplicated §1.1, §1.1b, §1.1d, §1.4 and Part 3 into pointers at Part 0, keeping only
what each adds beyond the definition (Part 3 is now a rename-mapping table plus the Required
Searches worked example). Header note rewritten to say the redundancy with
FOOP-23/24/43/AGENTS.md is deliberate.

**Date**: 2026-07-28 (4)
**Updated By**: Claude Code (Opus 5)
**Changes**: (1) **FOOP-85 withdrawn — coordination detachment is FOOP-24.** Part 3 renamed
FOOP-24's feature; it did not fork a new one, so reserving a separate number split a live feature
from its own specification. 85 was also not a valid next number (little-endian `gen_next` yields
FOOP-94). Every live FOOP-85 reference in this document now points at FOOP-24; Part 3 retitled
"…for FOOP-24 to adopt" and the References entry rewritten. (2) **New §2.3.1 — what a candidate
carries, and where `CopyMode` is resolved.** Fixed: a search result **must** carry the `CopyMode`
resolved for its specific found candidate (ill-formed otherwise), and a candidate carries more
than `(FirRef, usize)` — the FIR, its **home brane**, its statement number, and the copy mode,
since results already carry position for `&`-continuation (§1.4) and `AncestralNavigator` yields
candidates from several branes as it climbs, making downstream `_get_my_brane` re-derivation
unreliable. **TBD, left deliberately open:** whether the Navigator or the contextful-search layer
resolves `CopyMode`; both satisfy the fixed requirements and trade differently (one object owning
walk-and-decide, vs. keeping `CandidateNavigator` narrow when `BraneNavigator` can only ever
produce `Normal`). Consequently §2.1, Abstract item 2, and UBC Step Impact no longer assert
`contextful_search_scan`/`CandidateNavigator` are signature-stable — their *logic* is unchanged,
but signature stability holds only under some resolutions of the TBD, now flagged at each site and
promoted to the first Open Question as the decision to settle before coding.

**Date**: 2026-07-28 (3)
**Updated By**: Claude Code (Opus 5)
**Changes**: Three corrections from Atlas review, all narrowing or correcting claims that would
have mis-guided implementation.

(1) **§1.5 rewritten — anchored miss stays NK.** The prior text claimed "anchored miss →
ECONSTANIC, superseding the older anchored-miss→NK rule." That is not the settled rule. Per
FOOP-43 (revised this session): anchored miss → **NK** (an anchored search names its brane, so
exhausting it proves absence, and NK must keep meaning "provably unknowable"); unanchored miss →
ECONSTANIC; **SFF-marked search → ECONSTANIC regardless of anchoring**. Added an explicit
dependency note that FOOP-43 is still Draft and must land first.

(2) **§2.4.1 rewritten — full detachment needs a reason tag, not bare exhaustion.** The prior text
argued a fully-Detached search settles ECONSTANIC "via ordinary exhaustion," with "no special-case
reject-all signal needed anywhere." Under the corrected §1.5 that is wrong: a bare `Miss` on an
*anchored* search settles NK, so a fully-Detached anchored search would be destroyed rather than
deferred — the opposite of what SFF means. Detached exhaustion must be **distinguishable** from
genuine exhaustion at the settlement site and settle ECONSTANIC with `EconstanicReason::Detached`.
FOOP-43 Component 3 is therefore a **hard prerequisite**, not an adjacent nicety.

(3) **New §2.2.0 "Scope rule" — the marker mechanism is much narrower than the draft implied.**
At the user's direction, stated up front and repeated at each point of ambiguity: a stay-foolish
marker affects **only** a backward/ancestral search **originating inside** it, and **only** where
that search's AB climb **crosses the marker's own boundary outward**. Contexted (`&`) searches are
never affected (clipped to home brane, §1.3); searches resolving without reaching the boundary are
never affected. Markers live in `AncestralNavigator` only — `BraneNavigator` and
`contexted_search_from_anchor` are untouched. Reinforced in §2.3 (the `marker_stack` contains only
crossed markers; an unreached marker means an empty stack and `Pass`) and §2.4 ("unconditionally
true" means *per crossing*, not *per search lexically under the marker* — the easiest way to
mis-implement this).

Consequences of (3): **§2.5's bit-for-bit equivalence claim withdrawn.** `has_ancestral_sfm` is
indexed on the *searcher* (`fir_trait.rs:387-388`, and set for `StayFoolish` only — never SFF)
while `CopyMode` is indexed on the *boundary crossing*; they diverge in cases reachable today with
naked markers, not only once `[patterns]` exist. Test Plan updated accordingly: the equivalence
test is replaced by an enumeration of each divergence, plus new scope-rule tests; the approval
section is split so Part 1 + §2.2 must be snapshot-clean while §2.3–§2.5 have expected SF/SFF
churn (ten candidate files named), with a recommendation to land the two halves as separate
commits. Header note and Abstract item 3 updated to stop claiming blanket behavior preservation.

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
