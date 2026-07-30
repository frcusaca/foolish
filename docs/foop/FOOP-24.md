---
foop: 42
title: Coordination detachment — parameterized stay-foolish markers
author: Atlas hc.busy@gmail.com
status: Draft
type: Standards
created: 2026-07-09
phase: phase-2
supersedes: []
begun: [ ]
---

# FOOP-24: Coordination detachment — parameterized stay-foolish markers

> **THIS IS THE COORDINATION DETACHMENT FOOP — the live specification, not a historical
> document.** Renamed 2026-07-28: what this file has always called "Detachment" is now called
> **"Coordination detachment"** (it governs how a detachment target is *coordinated* — the
> resolution/copy behavior of candidates crossing the marker's boundary — as distinct from
> "Privacy detachment," which would prevent *discovery*, and from "Required Searches"; see
> FOOP-84 §Part 3 for the full terminology). The feature, syntax, semantics, cross-tabulation
> table, pattern types, and test plan below are all current and authoritative.
>
> **What FOOP-84 supersedes here is mechanism only, not the feature.** FOOP-84 (Search Engine
> Refactor) is the authoritative *search-engine* specification and replaces two things in this
> document:
> 1. **"Implementation Plan → Phase A"** — the `_ab_search`-override design is superseded by
>    FOOP-84 Part 2's `AncestralNavigator` / `resolve_boundary_effect` / `CopyMode` mechanism.
>    Build against that instead.
> 2. **The "Prefilter locus" bullet** — detachment acts in the **Navigator** (pre-yield), not in
>    the scan loop before `predicate.matches`. Struck in place below.
>
> Also updated: **"Nested markers"** is no longer UNDECIDED — resolved by FOOP-84 §2.3/§2.6
> (per-candidate, innermost-to-outward, first-*matching*-level wins; no search reversal needed).
> **"Exclusive detachment"** is reframed as one mechanism under FOOP-84's new "Required Searches"
> future feature. **"Privacy detachment"** is unchanged. The **Scope** section added below
> (descendant-only, outward-boundary-crossing-only, backward/ancestral-only) is load-bearing and
> narrows the feature considerably — read it first.
>
> **Prerequisites:** FOOP-84 (the mechanism this builds on — must land first) and FOOP-43
> (specifically its SFF-marked→ECONSTANIC rule and `EconstanicReason::Detached`, so a
> fully-detached search defers rather than settling NK).
>
> *(An earlier revision of the 2026-07-28 commit reserved a separate "FOOP-85" for the
> implementation of this feature. That was a mistake on two counts — it split a live feature from
> its own specification, and 85 is not a valid next number under little-endian numbering
> (`gen_next` yields FOOP-94). FOOP-85 is withdrawn; coordination detachment is specified and
> tracked here, in FOOP-24.)*
>
> Lean draft. Fuller notes in the Appendix and `NOTES-creation-lineage-and-search-family.md` §2.
> (Implementation order: #9 — the bridge from the search family to recursion. Renumbered
> 2026-07-09. Atlas: tightening detachment should help recursion definitions.)
>
> **Terminology: cite FOOP-84 Part 0, do not restate.** Every search-family term this FOOP uses —
> **search context** (§0.3: home brane *in its own context* + statement number of the matched
> statement, carried by `FoolRefFir` at `ubc_children[1]`), **contextless/contexted search**
> (§0.4), **anchored/unanchored** and what a miss proves (§0.2), the **detachment family**
> (§0.5: coordination vs. privacy vs. Required Searches vs. strict — this FOOP specifies
> **coordination** detachment), **marker scope** (§0.6), and **engine vocabulary** (§0.7) — is
> defined there. On first use write the term plus its pointer, e.g. "search context (FOOP-84
> §0.3)"; use the bare term thereafter.

## Abstract

Detachment is a **parameterizable stay-foolish marker** — `[p1,p2,…]<<Expr>>` (SFF) or
`[p1,p2,…]<Expr>` (SF). SF and SFF are parameterizable markers; when not parameterized they
affect all searches inside `Expr`. When parameterized (with patterns between `[]`), they affect
only the searches specified in the detachment configuration.

- **SF `<…>`** (equivalent to `[*]<…>`): The expression is evaluated but stays "foolish" —
  not fully resolved. Detached candidates are found but their constanic constituents are
  copied constanic (not recoordinated).
- **SFF `<<…>>`** (equivalent to `[*]<<…>>`): Search candidates are filtered by detachment.
  When searches find nothing under such filtration, they become ECONSTANIC. The optimized
  implementation of `[*]<<>>` is to simply initiate all searches as ECONSTANIC (already
  implemented and should be kept).

The general `[patterns]` form selects *which* candidates are detached. Without patterns
(`[]<E>` or `[]<<E>>`), no candidates are detached and the markers behave as their
undetached defaults.

## Motivation

Foolish has no way to say "resolve this expression, but *hide* certain candidates from its
searches." Detachment is that mechanism, and it unifies the two stay-foolish markers (whose
undetached defaults differ) under one parameter. Placed just before recursion because tightening
detachment (controlling which names a sub-computation can/can't see) is expected to help express
recursion cleanly.

## Specification

### Syntax

`[p1,p2,…]` immediately preceding an SF (`<…>`) or SFF (`<<…>>`) mark. The patterns
are name patterns (same syntax as search patterns). Store them on the marker FIR as
`detachments`. Whitespace between `[patterns]` and the marker is permitted.

### Semantics

#### Scope — what coordination detachment can and cannot affect (read this before the rest)

**Coordination detachment affects only descendant searches of the SF/SFF marker, and only as
those searches cross the marker's own boundary.** Three conditions must all hold before a
detachment pattern is even tested against a candidate:

1. **Descendant-only.** The search must originate lexically *inside* the marker's `Expr`. A marker
   never affects searches outside it, in sibling branes, or in unrelated code. Detachment is not
   ambient scope — it is a property of a boundary a search walks through.
2. **Outward boundary crossing only.** The pattern is consulted only where the search's ancestral
   (AB) climb *leaves* the marker, going from inside to outside. A search that finds its answer
   **without reaching the marker boundary is entirely unaffected**, even though it sits lexically
   under the marker.
3. **Backward/ancestral searches only.** Only the outward AB climb crosses boundaries.
   **Contexted (`&`) searches are never affected**, because a contexted search is clipped to its
   home brane and never leaves it (FOOP-84 §1.3). Intra-brane scans that never climb are likewise
   unaffected.

So in `[a]<{ x = a; y = local }>` where `local` is defined inside the marker's own brane, the
search for `local` never crosses the marker boundary and is never tested against `[a]` at all.
Only `a`, which must be reached by climbing *past* the marker, is subject to detachment.

The practical upshot: **a detachment cannot reach sideways or downward, and cannot affect a search
that resolves locally.** The feature is considerably smaller than "these names are hidden inside
this expression" suggests. See FOOP-84 §0.6/§2.2.4 for the engine-level statement of the same rule
(markers live in `AncestralNavigator` only; `BraneNavigator` and `contexted_search_from_anchor`
are untouched).

#### Marker behavior

SF and SFF are parameterizable markers. When not parameterized (bare `<E>` or `<<E>>`), they
affect all *boundary-crossing* searches inside `Expr` (per the scope rule above — not literally
all searches). When parameterized (with `[patterns]`), they affect only the searches specified in
the detachment configuration. For every boundary-crossing search that `Expr` or its children
perform, a candidate matching any detachment pattern is handled according to the marker type:

- **SF**: candidate is found but constanic constituents are copied constanic (not recoordinated)
- **SFF**: search candidates are filtered by detachment; when searches find nothing under such
  filtration, they become ECONSTANIC

**Bare markers are full detachment.** `<E>` ≡ `[*]<E>` (detach everything, SF behavior);
`<<E>>` ≡ `[*]<<E>>` (detach everything, SFF behavior). The `[]` form detaches nothing.

**Constanic-copy.** A parameterized marker is **stripped on constanic clone**, exactly like a
bare SF/SFF (`constanic_clone_at`, `fir_kinds.rs:155-179`) — "coordination frees everything."
Detachments do not survive coordination unless re-detached by another parameterized mark.

### Detachment pattern types

**Regular detachment patterns** support name and/or value conditions (same syntax as search
patterns, FOOP-23):
- `A` — name-only: detach candidates named `A`
- `C=10` — name+value: detach a candidate named `C` **whose value is 10**
- `=5` — value-only: detach **any** candidate whose value is 5 (no name constraint)
- `r.*=0` — pattern+value: detach candidates whose name matches `r.*` **and** whose value is 0

**Continuation anchored detachment patterns** (`&~`, `&?`, `&#`) anchor on the detachment
target (the brane) and search for candidates by name or index. **Value conditions are NOT
permitted** on continuation anchored patterns because the detachment target's statements have
not been evaluated yet — their values do not exist at detachment time.

| Pattern type | Syntax | Value conditions | Example |
|--------------|--------|------------------|---------|
| Name (exact) | `A` | Allowed | `[a]`, `[a=10]` |
| Name (pattern) | `2*r.*` | Allowed | `[a*]`, `[r.*=0]` |
| Value only | `=5` | Allowed | `[=10]`, `[=0]` |
| Index | `#N` | Not allowed | `[#1]`, `[#-2]` |
| Contexted forward | `&~pattern` | Not allowed | `[&~r.*]`, `[&~f.*]` |
| Contexted backward | `&?pattern` | Not allowed | `[&?r.*]`, `[&?f.*]` |
| Contexted index | `&#N` | Not allowed | `[&#1]`, `[&#-2]` |

### Cross-tabulation: marker × detachment × behavior

Context: `{s=1; a=10+s; …}` where `s=1` (constant), so `a=11`.

| Marker | Detachment | Example | Result |
|--------|------------|---------|--------|
| None   | None       | `{s=1;a=10+s; r={x=a}}` | `r = {x = 11}`  |
| SF | `[]` (none) | `{s=1;a=10+s; r=<{x=a}>}` | `r = {x = 10+s}` equivalent to `[*]<…>` |
| SFF | `[]` (none) | `{s=1;a=10+s; r=<<{x=a}>>}` | `r = {x = a}` equivalent to `[*]<<…>>` |
| SF | `[a]` (exact) | `{s=1;a=10+s; r=[a]<{x=a}>}` | `r = {x = 10+s}` (`a` found, but constanic constituents are copied constanic (not recoordinated)) |
| SFF | `[a]` (exact) | `{s=1;a=10+s; r=[a]<<{x=a}>>}` | `r = {x = a}` (`a` is never searched) |
| SF | `[b]` (no match) | `{s=1;a=10+s; r=[b]<{x=a}>}` | `r = {x = 11}` |
| SFF | `[b]` (no match) | `{s=1;a=10+s; r=[b]<<{x=a}>>}` | `r = {x = 11}` |
| SF | `[a*]` (pattern) | `{s=1;alpha=10+s; r=[a*]<{x=alpha}>}` | `r = {x = 10+s}` |
| SFF | `[a*]` (pattern) | `{s=1;alpha=10+s; r=[a*]<<{x=alpha}>>}` | `r = {x = a}` |
| SF | `[a=11]` (name+value) | `{s=1;a=10+s; r=[a=11]<{x=a}>}` | `r = {x = 10+s}` |
| SFF | `[a=11]` (name+value) | `{s=1;a=10+s; r=[a=11]<<{x=a}>>}` | `r = {x = a}` |
| SF | `[=11]` (value only) | `{s=1;a=10+s; r=[=11]<{x=a}>}` | `r = {x = 10+s}` |
| SFF | `[=11]` (value only) | `{s=1;a=10+s; r=[=11]<<{x=a}>>}` | `r = {x = a}` |
| SF | `[&~f.*]` (ctx fwd pattern) | `{s=1;alpha=10+s; result=[&~f.*]<{fin=alpha}>}` | `result = {fin=10+s}` SF applied to `fin=alpha` |
| SFF | `[&~f.*]` (ctx fwd pattern) | `{s=1;alpha=10+s; result=[&~f.*]<<{fin=alpha}>>}` | `result = {fin=alpha}` SFF applied to `fin=alpha` |
| SF | `[&?f.*]` (ctx bwd pattern) | `{s=1;alpha=10+s; result=[&?f.*]<{fin=alpha}>}` | `result = {fin=10+s}` SF applied to `fin=alpha` |
| SFF | `[&?f.*]` (ctx bwd pattern) | `{s=1;alpha=10+s; result=[&?f.*]<<{fin=alpha}>>}` | `result = {fin=alpha}` SFF applied to `fin=alpha` |
| SF | `[&#1]` (ctx index) | `{s=1;a=10+s; result=[&#1]<{z=10,result=a,y=10}>}` | `result = {z=10, result=s+1, y=10}` SF affected stmnt 2 of brane |
| SFF | `[&#1]` (ctx index) | `{s=1;a=10+s; result=[&#1]<<{z=10,result=a,y=10}>>}` | `result = {z=10, result=a, y=10}` SFF affected stmnt 2 of brane |

### Key distinction

The marker type determines what happens to detached candidates:
- **SF**: Expression is evaluated but stays "foolish" — detached candidates are found but their
  constanic constituents are copied constanic (not recoordinated). Result is the expression form
  (e.g., `10+s`), not the fully evaluated value (e.g., `11`).
- **SFF**: Search candidates are filtered by detachment; when searches find nothing under such
  filtration, they become ECONSTANIC. The optimized implementation of `[*]<<>>` is to simply
  initiate all searches as ECONSTANIC. Result is the identifier form (e.g., `a`), not the value.

### Pattern matching

Detachment patterns use the same regex matching as search patterns (FOOP-23).
`[a*]` matches any candidate whose name starts with `a`. If the pattern matches, the candidate is
detached (SF → evaluated but foolish, SFF → filtered from searches; ECONSTANIC if nothing found).

### Nested markers

When SF/SFF markers are nested, the closest matching marker to the search determines the
detachment behavior. For example:

```foolish
{
    a = 10;
    result = [a]<{
        inner = [b]<<{
            x = a;    !! Which marker governs the search for 'a'?
            y = b;    !! Which marker governs the search for 'b'?
        }>>;
    }>;
}
```

The prevailing thought is that the closest matching marker to the search should win and
decide the behavior of a matching search. This may require reversing the search: match
search on everything first, then check the stack of detachments from innermost outward.

**Status: UNDECIDED.** The exact semantics of nested detachment are not yet specified.
This needs to be flushed out with concrete examples before implementation. The table above
shows single-level behavior; nested behavior will be added once the semantics are resolved.

### Out of scope

- **Alerting on constanics.** This FOOP does not specify any mechanism for alerting when
  detachment patterns match candidates that are already constanic. That is a separate concern.
- **Search result type updates.** The search result type may need to carry SF/SFF marking
  information, but that is a design/implementation detail deferred to the implementation phase.

## Related and Future Features

The following features are related to detachment but are deferred to future FOOPs. They are
documented here to capture the design space and motivate the current specification.

### Exclusive detachment

A previous version of detachment brane wanted to make detachments like function declarations
in other languages — if there's a variable not found, it is a compiler error. We defer that
feature for a later time.

Exclusive detachment is mainly about specifying, exclusively, what can be constanic, which is
not the same as preventing some variables from being searched for. The idea: a detachment
pattern `[a, b, c]` would mean "these are the ONLY variables that can be constanic in this
scope" — any other variable that becomes constanic would be an error. This is a completeness
assertion, similar to the backburnered strict detachment `[[…]]`.

**Why deferred:** The semantics are complex (similar to strict detachment) and the use case
is not yet clear. The current permissive detachment is sufficient for the recursion use case.

### Privacy detachment

A related feature where detachment reverses and anchored searches cannot find some things
that are blocked from search. This is information hiding — the detachment prevents not just
resolution but also discovery.

In the current specification, detachment only affects the *resolution* of a candidate (SF
copies constanic, SFF filters out). But the candidate is still *discoverable* — a search
can find it, even if it can't resolve it. Privacy detachment would go further: the candidate
would be invisible to searches entirely, even to anchored searches.

**Why deferred:** This is a significant change to the search model. The current specification
is already complex enough; privacy detachment can be added later as a modifier on top of the
existing detachment semantics.

### Strict detachment `[[…]]`

See the Appendix (§Backburnered: strict detachment) for the full discussion. Strict
detachment would be a completeness assertion: not only do the patterns skip candidates,
but the mark would additionally forbid any unexplained non-resolution. This was backburnered
because the semantics are unresolvable in scope (see the Appendix for the detailed argument).

## FIR Impact

No new FIR kind. The matching logic is **fully encapsulated in a `Detachment` struct** (a plain
struct, *not* a FIR — a detachment never steps). **All detachment data and functions live on it**
— no free-floating functions or bare data structures.

- **`Detachment` owns:** the parsed entry list (each entry: name pattern + optional value), and a
  **lazily-populated compiled cache** (the `regex::RegexSet` over the entries' name patterns +
  the per-index value conditions). `regex` is already a dependency (`fir_kinds.rs:5`).
- **`StayFoolishFir`/`StayFullyFoolishFir`** (`fir_kinds.rs:2021`/`:2082`) each **hold an
  `Option<Detachment>`** (or an empty one for the bare marker). The marker **initiates and caches**
  the `Detachment` when a search first passes through it.
- **The `Detachment` exposes essentially one method:** `decide_to_detach(statement)`, run on each
  search candidate. Everything else (RegexSet build, value checks) is internal.

## UBC Step Impact

- **`Detachment::decide_to_detach(statement)` — the single entry point.** On its **first call** the
  `Detachment` lazily builds its `RegexSet` from the entries' name patterns (index *i* ↔ entry *i*;
  **value-only entries contribute a `.*` pattern**) and caches it; later calls reuse it. Per call:
  1. Require the candidate `statement` be **constanic** — `decide_to_detach` (on a non-FIR struct)
     **cannot step**, so it **raises/panics if the statement is non-constanic** (the "candidates
     are all constanic" invariant, in force here as in the matcher).
  2. `RegexSet::matches(candidate_name)` → triggered entry indices, **in one scan**.
  3. For each triggered index, apply that entry's optional value condition to the candidate's value.
  4. Return a **three-way result** (via the FIR-or-NK sealed search-result type — see below):
     **Detach** (some triggered entry's full name∧value condition holds), **Keep** (none holds), or
     **NK** (a required value comparison was **undecidable because the candidate value is NK** —
     detachments are *forceful* filters, so an NK comparison forces the search result to NK, not a
     silent keep).

  Return type: the **optional/sealed search-result trait (a resolved FIR or an NK)** — the same one
  searches use — so "NK due to an undecidable equality" surfaces as a genuine search NK.
  Example `[A, B, C=10, =5]` → RegexSet `["A", "B", "C", ".*"]`; a candidate `C=10` triggers index 2
  and index 3 → Detach; `x=5` triggers index 3 → Detach; `y=???` under `=5` → **NK** (can't compare
  `??? == 5`).
- **Scope handoff.** Extend `Scope` (`fir_trait.rs:55`, today just `has_ancestral_sfm: bool`) to
  carry the **active `Detachment`(s)** accumulated from enclosing parameterized marks, pushed in
  `step_inner` (`fir_trait.rs:347`) when stepping under a parameterized marker.
- ~~**Prefilter locus.** In the scan loop (`contextful_search_scan`, `fir_kinds.rs:1978`),
  **before** `predicate.matches`, call `decide_to_detach` on the candidate; skip on **Detach**,
  NK-the-search on **NK**, proceed to the matcher on **Keep**. **Same locus as FOOP-93's `!`**,
  applied first (a filter — order-idempotent for Detach/Keep; the NK outcome is likewise
  position-independent).~~
  **Superseded by FOOP-84 §2.3.** The locus is **not** the scan loop and **not** shared with
  FOOP-93. Detachment acts inside the **Navigator** (`AncestralNavigator::next_candidate`),
  filtering candidates *before they are ever yielded* to the scan loop — so the predicate never
  sees a detached candidate and there is no ordering interaction with FOOP-93's `!`/`&&`/`||`.
  The two are orthogonal (Navigator vs. Predicate) and need no relative sequencing. Only the
  ancestral walk is involved, per the Scope section above.
- **Reason tag (FOOP-43 Component 3).** When a search settles ECONSTANIC because a candidate was
  **Detached**, set `EconstanicReason::Detached`.
- **Keep existing SFF unchanged.** Naked `<<>>` keeps its current implementation; `[*]<<>>`
  forwards to it. The **new `Detachment` path is only for specific (non-`*`) detachments**. Engage
  it iff the marker has a non-empty, non-`[*]` `Detachment`.

## Test Plan

- Unit — `Detachment` in isolation: `decide_to_detach` returns **Detach** for a name-match
  (`A`), for a name+value match (`C=10` on a `C`-valued-10 candidate) and not otherwise, for a
  value-only match (`=5`); **Keep** when nothing triggers; **NK** when a triggered value condition
  can't be decided (candidate value is `???`); **panics** on a non-constanic candidate. Lazy
  `RegexSet` built once (value-only entry → `.*`).
- Unit — integration: a `[a]<<…>>` marker skips a candidate named `a`; `<E>` ≡ `[]<E>` (nothing
  skipped) and `<<E>>` ≡ `[*]<<E>>` (all skipped); detachment applies identically on SF and SFF;
  constanic-clone strips it.
- Approval: `[tmp.*]<<a?x>>` hides `tmp_k`; a `[=5]` value-only detachment; the two spectrum-extreme
  demonstrations.
- **Triple documentation** (see Plan).

## Rejected Alternatives

### A. A `[..]{..}` brane construct (the earlier reading)

Detachment as explicit AB/IB brane recoordination. **Rejected**: it is really a search prefilter
riding on the stay-foolish markers, which also cleanly explains the SF/SFF default asymmetry.

### B. Make detachment a FIR (`FirKind::Detachment`)

A first-class stepping FIR. **Rejected**: a detachment never steps or settles on its own — it is a
per-candidate *filter*. It is encapsulated as a plain **`Detachment` struct** (owning the entries +
lazy `RegexSet` + `decide_to_detach`), held by the SF/SFF markers. No `FirKind` variant, no NYES
story, but still fully encapsulated (all data + fns on the struct — no free-floating helpers).

## Open Questions

- **RESOLVED:** detachment entries match **name and/or value** (same forms as searches:
  `A`, `C=10`, `=5`). (Characterization: since chars are part of the name/pattern per FOOP-63, a
  `b'x` detachment naturally gates on characterization too.)
- Disposition of the old `[..]{..}` `DetachmentBrane` parse form (repurpose or deprecate).
- Must `[..]` be followed by a stay-foolish mark (error otherwise)?
- **How detachment helps recursion** — the specific recursion patterns detachment should enable
  (hiding a recursion variable's outer binding so the inner call rebinds cleanly?) — explore with
  the recursion FOOP (FOOP-34).
- (Strict detachment `[[…]]` is out of scope — backburnered; see the Appendix.)

## Current Search Implementations

The Foolish UBCa has multiple search implementations scattered across the codebase.
Understanding these is essential for the refactoring that detachment requires.

### The `contextful_search` module (the engine)

The canonical search engine lives in `fir_kinds.rs` (lines 1680–2062). It provides:

- **`CandidateNavigator` trait** — yields `(FirRef, usize)` candidates from a brane
- **`BraneNavigator`** — iterates `foolish_children()` forward or backward, optionally bounded by range
- **`SearchPredicate`** — matches candidates by name (`Name`), value (`Value`), name+value (`NameValue`), index (`Index`), head (`Head`), or tail (`Tail`)
- **`contextful_search_scan`** — the core scan loop: iterate candidates, apply predicate, return `ScanOutcome::Found`/`NkStop`/`Miss`
- **`contextful_search_scan_no_body_check`** — variant that skips the body-NYES gate (for name-only searches where body settling is the caller's responsibility)

The engine is clean and well-factored. The problem is that it's not the only search path.

### FIR-attached search methods

Multiple search methods are attached to FIR kinds and the `Fir` trait:

| Location | Method | Signature | Purpose |
|----------|--------|-----------|---------|
| `Fir` trait | `_ib_search` | `(&self, self_ref: &FirRef, name: &str) -> Option<(FirRef, Nyes)>` | Immediate brane search (name-only) |
| `Fir` trait | `ib_search` | `(&self, scope: &Scope, name: &str) -> Option<(FirRef, Nyes)>` | Scope-cached IB search |
| `Fir` trait | `_ab_search` | `(&self, self_ref: &FirRef, name: &str) -> Option<(FirRef, Nyes)>` | Ancestral brane search (name-only) |
| `Fir` trait | `ab_search` | `(&self, scope: &Scope, name: &str) -> Option<(FirRef, Nyes)>` | Scope-cached AB search |
| `Fir` trait | `_search_brane` | `(&self, expr: &str, start: usize, end: usize) -> Option<(usize, FirRef, Nyes)>` | Low-level range scan |
| `StatementFir` | `_ib_search` | override | Searches backward from statement's line number |
| `BraneFir` | `_ab_search` | override | Walks parent branes recursively |
| `BraneFir` | `_search_brane` | override | Linear scan of `foolish_children()` |
| `SearchFir` | `ib_search_with_engine` | `(&self, scope: &Scope) -> Option<(FirRef, Nyes)>` | Engine-based IB search |
| `SearchFir` | `ab_search_with_engine` | `(&self, scope: &Scope) -> Option<(FirRef, Nyes)>` | Engine-based AB search (walks parents) |
| `SearchFir` | `contexted_search_from_anchor` | `(&self, scope: &Scope) -> Option<(FirRef, Nyes)>` | Contexted search from anchor position |

### Limitations of current implementations

**String pattern only.** `_ib_search` and `_ab_search` take `name: &str` — they can only match by name. They cannot match by value, by name+value, or by index. The engine (`SearchPredicate`) supports all these, but the FIR-attached methods don't.

**No matcher abstraction.** The FIR-attached methods hardcode name matching. They don't accept a `SearchPredicate` or any matcher interface. This means:
- Value search (`?=v`, `~=v`) must go through `SearchFir::value_search_step` (a separate path)
- Index search (`#N`) must go through `IndexFir::fir_op_step` (another separate path)
- Contexted search (`&?`, `&~`, `&#`) must go through `SearchFir::contexted_search_from_anchor` (yet another path)

**Duplicated AB walk.** `BraneFir::_ab_search` and `SearchFir::ab_search_with_engine` both implement the "walk up parent branes" logic independently. They should be one function.

**SearchFir's state machine is complex.** `SearchFir::fir_op_step` has multiple code paths depending on `anchored`, `contexted`, `is_value_search`, and NYES state. Each path calls different search methods. The logic is correct but hard to follow.

### The SF/SFF delegation discussion

For detachment to work, SF/SFF marker FIRs need to control how parental search behaves when it reaches their scope boundary. The proposed design:

**Upgrade `_ib_search` and `_ab_search` to accept matchers** (not just name strings). This allows:
- Regular FIRs: pass through to normal parental search
- SF/SFF marker FIRs: override `_ab_search` to apply detachment behavior

**SF marker's `_ab_search` override:**
When a child's search reaches the SF scope, if the candidate matches the detachment pattern, return it with `sf_marked: true` (the caller will copy it constanic, not recoordinated). Otherwise, pass through to normal parental search.

**SFF marker's `_ab_search` override:**
When a child's search reaches the SFF scope, if the candidate matches the detachment pattern, return `NotFound` (the candidate is filtered out). Otherwise, pass through to normal parental search.

This design encapsulates detachment behavior in the marker itself. The search system doesn't need to know about detachment patterns — it just calls `_ab_search` on the parent chain, and the marker decides how to handle it.

**Search result type update:**
The search result type needs to carry SF marking information:
```rust
enum SearchResult {
    NotFound,
    Found {
        brane: FirRef,
        line: usize,
        statement: FirRef,
        sf_marked: bool,  // true if found through an SF marker's detachment
    }
}
```

When `sf_marked` is true, the caller knows to copy the result constanic (not recoordinated).

## Implementation Plan

The implementation has two phases: refactoring (Phase A) and feature addition (Phase B).

### Phase A — Refactor search to facilitate detachment

**Goal.** Refactor existing search behavior to maintain current behavior (passing all tests) while facilitating Phase B. The key change: upgrade `_ib_search` and `_ab_search` to accept matchers, and implement custom `_ab_search` on SF/SFF marker FIRs.

**What changes:**

1. **Upgrade `_ib_search` and `_ab_search` signatures** to accept `SearchPredicate` (or a trait object matcher) instead of `name: &str`. This allows value search, index search, and name+value search to use the same parental search path.

2. **Implement custom `_ab_search` on `StayFoolishFir` and `StayFullyFoolishFir`** that intercepts parental search at the marker's scope boundary. When the marker has detachment patterns:
   - SF: if candidate matches detachment pattern, return `Found { sf_marked: true }`
   - SFF: if candidate matches detachment pattern, return `NotFound`
   - Otherwise: pass through to normal parental search

3. **Consolidate AB walk logic** — `BraneFir::_ab_search` and `SearchFir::ab_search_with_engine` should become one function (the upgraded `_ab_search` on the Fir trait).

4. **Update search result type** to carry `sf_marked: bool` (or a richer enum).

5. **Remove redundant search methods** — `ib_search_with_engine`, `ab_search_with_engine`, `contexted_search_from_anchor` become thin wrappers or are removed entirely.

**Validation.** All existing snapshot/einmo tests must pass unchanged. The refactoring preserves behavior; it only changes the call structure.

- [ ] Define `SearchResult` enum with `sf_marked` field
- [ ] Upgrade `_ib_search` and `_ab_search` to accept `SearchPredicate`
- [ ] Implement `StayFoolishFir::_ab_search` override (detachment-aware)
- [ ] Implement `StayFullyFoolishFir::_ab_search` override (detachment-aware)
- [ ] Consolidate AB walk logic into one function
- [ ] Update `SearchFir::fir_op_step` to use upgraded `_ab_search`
- [ ] Remove `ib_search_with_engine`, `ab_search_with_engine`, `contexted_search_from_anchor`
- [ ] All snapshot/einmo tests pass
- [ ] `cargo clippy -D warnings` clean

### Phase B — Add detachment behavior

**Goal.** Add the detachment feature as specified in this FOOP. The SF/SFF marker FIRs already have custom `_ab_search` from Phase A; Phase B adds the `Detachment` struct and the parser.

**What changes:**

1. **`Detachment` struct** — owns parsed entries + lazy `RegexSet` + `decide_to_detach`
2. **Parser** — recognize `[patterns]` before SF/SFF mark; build `DetachmentEntry` list
3. **`StayFoolishFir`/`StayFullyFoolishFir`** hold `Option<Detachment>`
4. **`_ab_search` override** uses `Detachment::decide_to_detach` to filter candidates
5. **Constanic-copy** strips detachment (already specified)

**Validation.** New approval tests for detachment behavior. Existing tests still pass.

- [ ] `Detachment` struct + `DetachmentEntry` + `decide_to_detach`
- [ ] Unit tests for `decide_to_detach`
- [ ] Parser: recognize `[patterns]` before SF/SFF mark
- [ ] `StayFoolishFir`/`StayFullyFoolishFir` hold `Option<Detachment>`
- [ ] `_ab_search` override uses `decide_to_detach`
- [ ] `EconstanicReason::Detached` for the Detach→ECONSTANIC path
- [ ] Spectrum validation: `<E>` ≡ `[*]<E>`, `<<E>>` ≡ `[*]<<E>>`, `[a]<E>` skips only `a`
- [ ] Constanic-clone strips detachment
- [ ] Approval cases; comprehensive `foop_24_comprehensive.foo`
- [ ] **Doc TODO: SF≡`[]` / SFF≡`[*]` spectrum in README.md, code comments, snapshots**
- [ ] Worktree lifecycle per `foop.md`

## Project note — memoize `decide_to_detach` (later)

`Detachment::decide_to_detach(statement)` is called once per candidate per search. Beyond the lazy
`RegexSet` compilation (already specified), **memoize the decision itself** — e.g. cache the
Detach/Keep/NK result keyed by candidate identity — so repeated searches under the same marker over
the same brane don't re-run the RegexSet + value checks. Deferred; note it so a future pass picks
it up. (Requires care around the constanic invariant and candidate identity.)

## Appendix — notes toward the full spec

- **Depends on FOOP-43** — reject-all (`[*]`/naked-`<<>>`) needs miss → ECONSTANIC (not NK), so the
  detached searches "wait" rather than "die."
- Shares the prefilter locus with **FOOP-93** (`!`); the search predicate work lands first,
  detachment reuses the seam. Composes with FOOP-14 (`a![p..]~~q`).
- **Placed before recursion (FOOP-34)** because controlling name visibility is expected to help
  express recursion — coordinate the two.
- SF "does something extra hard to describe via search matchers" — the empty-detachment default is
  *why* SF isn't purely a prefilter; the marker's other behavior is untouched.

## Appendix — Backburnered: strict detachment `[[…]]`

**Status: BACKBURNERED (2026-07-10) — the scope is too complex for now.** The idea and the reason
it is hard are recorded here for a future revisit; it is **not** part of this FOOP's implementation.

**The idea.** A strict (double-bracket) detachment `[[p1,…]]<E>` / `[[…]]<<E>>` would be a
*completeness assertion*: not only do the patterns skip candidates (as in permissive `[…]`), but
the mark would additionally forbid any *unexplained* non-resolution — an ECONSTANIC search that the
detachments did **not** cause should error (NK). Motivation: "if we are bothering to write down what
must not resolve, we should account for *all* non-resolution; anything else unresolved is a bug."
The four forms would be `[p,]<E>`, `[[p,]]<E>`, `[p,]<<E>>`, `[[p,]]<<E>>`.

**Why it is hard — the case we could not resolve.** Whether a currently-ECONSTANIC search is
"caused by a detachment" is **not** answerable from the present state. Concretely:

- A search inside a strict mark may, *right now*, **find nothing AND skip no candidate** (no
  detachment fired for it) — yet in a **future coordination context** it could match a candidate
  that a detachment pattern would then skip. In that future the search *would* be detached, so
  morally its ECONSTANIC should be considered "accounted for" even now (it may become detached).
- We have **no way to detect this at the current moment.** Deciding "could this search ever match a
  name that a detachment pattern also matches, over names that do not exist yet" is the question of
  whether the **intersection of the search's pattern language and the detachment patterns' language
  is non-empty** — i.e. **regular-expression intersection/containment**, which is decidable for
  plain regular languages but expensive, and undecidable/ill-posed once patterns can grow or names
  are drawn from an open, coordination-dependent universe. So a naive runtime "did it skip a
  candidate" signal is **insufficient**: it sees only present candidates, and would prematurely NK a
  search that a later coordination + detachment would have legitimately excused.

**Consequence.** A correct strict rule has to choose between (a) **conservative** — never NK a
search that *might* be future-detached, which (given undecidability) collapses toward permissive
except where non-coverage is *provable*; or (b) **aggressive** — NK any unresolved-now search,
which is decidable but violates Foolish's "wait, may recoordinate" spirit by foreclosing future
resolutions. Neither is clearly right; the tension is real, not a detail.

**If revisited — research directions (for a future FOOP/plan):**
- The theory: **intersection/emptiness of regular languages** (the search pattern vs the detachment
  patterns) is the formal core; catalogue where it is cheap (DFA product), expensive, or ill-posed
  (open/growing name universe).
- **Coverage heuristics**: sound-but-incomplete tests for "this search can never be detached"
  (e.g. disjoint literal prefixes/alphabets, syntactic non-overlap) that let strict NK *only*
  provably-uncoverable searches, staying conservative otherwise.
- Whether a weaker but well-defined variant is worth it: e.g. `[[]]` (empty strict) = "no search may
  remain ECONSTANIC/WOCONSTANIC" as a pure everything-must-resolve assertion, sidestepping the
  coverage question entirely (there are no patterns to reason about).

Mechanism sketch (for whoever revisits): a `strict: bool` on the SF/SFF marker + an
ECONSTANIC-is-NK flag propagated down to `SearchFir`s and through `constanic_clone_at` (alongside
the existing SFM clone flag), an extra settle loop converting leftover ECONSTANIC/WOCONSTANIC → NK,
and a final `is_constantew()` sanity check (no non-`constantew` `constanic`s remain). This mechanism
is straightforward; the *semantics* (which searches to NK) is the unresolved part above.

## References

- Prior: FOOP-43 (miss semantics), FOOP-93 (shared prefilter locus), FOOP-23 (one-engine model),
  ECOSYSTEM.md (stay-foolish semantics).
- Code: `fir_kinds.rs:2021`/`:2082` (SF/SFF FIRs), `:155-179` (`constanic_clone_at` strip),
  `:1978` (scan loop); `fir_trait.rs:55`/`:347` (Scope handoff).
- Notes: `NOTES-creation-lineage-and-search-family.md` §2 + Engineering guidance; memory
  `foop-detachment-as-parameterized-sfmarker`.

## Last Updated

**Date**: 2026-07-28 (5)
**Updated By**: Claude Code (Opus 5)
**Changes**: Added the "cite FOOP-84 Part 0, do not restate" terminology banner — FOOP-84 Part 0
is now the single definition site for search context (§0.3), the two search families (§0.4),
anchoring/miss outcomes (§0.2), the detachment family (§0.5), marker scope (§0.6), and engine
vocabulary (§0.7). First use of any such term in this FOOP carries a §-pointer; no redefinition
here. This FOOP specifies **coordination** detachment specifically (§0.5).

**Date**: 2026-07-28 (3)
**Updated By**: Claude Code (Opus 5)
**Changes**: Added a **Scope** section at the head of Semantics stating plainly, at the user's
direction, that **coordination detachment affects only descendant searches of the SF/SFF marker,
as they cross the marker's own boundary** — three conditions (descendant-only, outward
boundary-crossing only, backward/ancestral only), the `[a]<{x=a; y=local}>` worked example showing
`local` is never tested against the pattern, and the upshot that a detachment cannot reach
sideways or downward or affect a locally-resolving search. Qualified the following paragraph's
"affect all searches inside `Expr`" to "all *boundary-crossing* searches." Struck the
**Prefilter locus** bullet in the Implementation Plan (it placed `decide_to_detach` in the scan
loop before `predicate.matches`, sharing a locus with FOOP-93) and replaced it with the FOOP-84
§2.3 position: detachment acts in the Navigator, pre-yield, so FOOP-93 and FOOP-85 are orthogonal
and need no relative sequencing. Body otherwise unchanged.

**Date**: 2026-07-28 (4)
**Updated By**: Claude Code (Opus 5)
**Changes**: **This file is restored as the live coordination detachment FOOP; FOOP-85 is
withdrawn.** Retitled to "Coordination detachment — parameterized stay-foolish markers"
(frontmatter and heading). Replaced the PARTIALLY SUPERSEDED banner — which had demoted this file
to a historical document and pointed implementation at a separate FOOP-85 — with an accurate one:
FOOP-84 supersedes **mechanism only** (the Phase A `_ab_search`-override design and the scan-loop
prefilter locus), while this file's feature, syntax, semantics, cross-tabulation table, pattern
types, and test plan all remain **current and authoritative**. Rationale: FOOP-84 Part 3 *renamed*
this file's feature, it did not fork a new one, so reserving a second number split a live feature
from its own specification; and 85 was not a valid next number under little-endian numbering
(`gen_next` yields FOOP-94). Banner now also records prerequisites (FOOP-84, and FOOP-43's
SFF-marked→ECONSTANIC rule + `EconstanicReason::Detached`) and points at the Scope section as
load-bearing. The 2026-07-28 (2) entry below is retained as an accurate record of what that commit
did, not as current guidance.

**Date**: 2026-07-28 (2)
**Updated By**: Claude Code (Sonnet 5)
**Changes**: *(Superseded by the 2026-07-28 (4) entry above — the FOOP-85 reservation described
here was withdrawn.)* Added a PARTIALLY SUPERSEDED banner pointing to **FOOP-84** (Search Engine Refactor
— the new authoritative search specification) and **FOOP-85** (Coordination detachment, the
implementation FOOP for this file's feature, built on FOOP-84's `AncestralNavigator`/`CopyMode`
mechanism instead of this file's `_ab_search`-override Phase A plan). The "Nested markers"
UNDECIDED question is resolved in FOOP-84 §2.3/§2.6 (per-candidate, innermost-to-outward,
first-*matching*-level-wins — not "reverse the search"). Terminology rename recorded in FOOP-84
§Part 3: this file's unqualified "Detachment" → "Coordination detachment"; "Exclusive detachment"
→ reframed as one mechanism under a new "Required Searches" future feature; "Privacy detachment"
unchanged. This file's body is otherwise untouched and retained for historical design discussion.

**Date**: 2026-07-28
**Updated By**: Sisyphus / xiaomi/mimo-v2.5-pro
**Changes**: Added "Nested markers" section (status: UNDECIDED — closest matching marker wins,
needs flushing out with concrete examples). Expanded "Related and Future Features" to `##`
header with three subsections: Exclusive detachment (completeness assertion, deferred), Privacy
detachment (information hiding, deferred), Strict detachment `[[…]]` (backburnered, see Appendix).

**Date**: 2026-07-11
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes (Atlas — encapsulation + RegexSet design)**: Detachment matching is encapsulated in a
plain **`Detachment` struct** (NOT a FIR — it never steps), held as `Option<Detachment>` on the
SF/SFF markers; **all data + fns on the struct, no free-floating helpers**. It exposes essentially
one method **`decide_to_detach(statement)`** returning **Detach / Keep / NK** (via the FIR-or-NK
sealed search-result type). Implementation: a **lazily-built, cached `regex::RegexSet`** over the
entries' name patterns (built on first search through the marker, not at parse time), value-only
entries contribute `.*`; `RegexSet::matches` does all names in one scan, only triggered entries pay
a value check. **Detachments are FORCEFUL filters:** an undecidable value comparison (candidate
value NK) → the search result is **NK**. Candidates must be **constanic** — `decide_to_detach`
can't step, so it **panics on a non-constanic candidate**. Entries carry **name and/or value**
(`A`, `C=10`, `=5`) — resolved the name-only-vs-value open question. Added a **project note to
memoize `decide_to_detach`** later. Rejected-Alt B updated (FIR rejected; `Detachment` struct is
the encapsulation).

**Date**: 2026-07-10
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes**: **Strict detachment `[[…]]` BACKBURNERED** and removed from the active spec (moved to
an Appendix). Reason: the semantics are unresolvable in scope — a strict mark needs to know whether
an ECONSTANIC search is "caused by a detachment," but a search that finds nothing and skips nothing
*now* could become detached under *future coordination*; detecting that is regular-expression
intersection/emptiness over an open name universe (undecidable/ill-posed), so no runtime signal
suffices and premature NK would violate Foolish's "may recoordinate" spirit. The Appendix records
the idea, the expanded hard-case explanation, the theory (regex intersection), coverage-heuristic
research directions, and the (straightforward) mechanism sketch for a future revisit. This FOOP now
specifies only the permissive single-bracket `[…]` form.

**Date**: 2026-07-09
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes**: Renumbered from FOOP-63 to FOOP-24 (impl-order reorg — detachment now its own FOOP
just before recursion). Parameterized SF/SFF marker (exclusion-list prefilter); SF≡`[]` / SFF≡`[*]`
spectrum; triple-documentation mandate; depends on FOOP-43. (Strict detachment was explored this day
and backburnered the next — see the 2026-07-10 entry.)
