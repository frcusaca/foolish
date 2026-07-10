---
foop: 42
title: Detachment — parameterized stay-foolish markers
author: Atlas hc.busy@gmail.com
status: Draft
type: Standards
created: 2026-07-09
phase: phase-2
supersedes: []
begun: [ ]
---

# FOOP-24: Detachment — parameterized stay-foolish markers

> Lean draft. Fuller notes in the Appendix and `NOTES-creation-lineage-and-search-family.md` §2.
> (Implementation order: #9 — the bridge from the search family to recursion. Renumbered
> 2026-07-09. Atlas: tightening detachment should help recursion definitions.)

## Abstract

Detachment is a **parameterized stay-foolish marker** — `[p1,p2,…]<<Expr>>` (SFF) or
`[p1,p2,…]<Expr>` (SF). The patterns (the **detachments**) are a per-candidate **search
prefilter**: every search inside `Expr` **auto-skips any candidate matching any pattern**, before
testing its own predicate. **SF and SFF are the two extremes of the spectrum:** `<E>` ≡ `[]<E>`
(empty — detach nothing, resolve normally), `<<E>>` ≡ `[*]<<E>>` (full — detach everything). The
general `[patterns]` is the middle: an **exclusion list** over otherwise-normal search.

## Motivation

Foolish has no way to say "resolve this expression, but *hide* certain candidates from its
searches." Detachment is that mechanism, and it unifies the two stay-foolish markers (whose
undetached defaults differ) under one parameter. Placed just before recursion because tightening
detachment (controlling which names a sub-computation can/can't see) is expected to help express
recursion cleanly.

## Specification

- **Syntax.** `[p1,p2,…]` immediately preceding an SF (`<…>`) or SFF (`<<…>>`) mark. The patterns
  are name patterns (same syntax as search patterns). Store them on the marker FIR as
  `detachments`.
- **Semantics.** For every search that `Expr` or its children perform, a candidate for which **any
  pattern matches** is **skipped** (not tested against the search's own predicate) — a
  per-candidate prefilter, not a boolean on the predicate.
- **The spectrum.** `<E>` ≡ `[]<E>` (empty detachment); `<<E>>` ≡ `[*]<<E>>` (full detachment).
  So bare SF detaches nothing (searches resolve normally) and bare SFF detaches everything (every
  search exhausts → ECONSTANIC per FOOP-43 → resolves on coordination).
- **Constanic-copy.** A parameterized marker is **stripped on constanic clone**, exactly like a
  bare SF/SFF (`constanic_clone_at`, `fir_kinds.rs:155-179`) — "coordination frees everything."
  Detachments do not survive coordination unless re-detached by another parameterized mark.

> **Strict detachment (`[[…]]`) was considered and BACKBURNERED** — see the Appendix
> (§Backburnered: strict detachment). This FOOP specifies only the permissive single-bracket
> `[…]` form.

## FIR Impact

No new FIR kind. Add `detachments: Vec<String>` to **both** `StayFoolishFir` (`fir_kinds.rs:2021`)
and `StayFullyFoolishFir` (`:2082`) — a parameterized marker is just SF/SFF with a non-empty
`detachments`.

## UBC Step Impact

- **Scope handoff.** Extend `Scope` (`fir_trait.rs:55`, today just `has_ancestral_sfm: bool`) to
  carry the **active detachment patterns** accumulated from enclosing parameterized marks
  (`active_detachments: Vec<String>`), pushed in `step_inner` (`fir_trait.rs:347`) when stepping
  under a parameterized marker.
- **Prefilter.** In the scan loop (`contextful_search_scan`, `fir_kinds.rs:1978`), before
  `predicate.matches`, skip a candidate if any active detachment pattern matches it (reuse
  `SearchFir::matches_pattern`). **Same locus as FOOP-93's `!`.**
- **Reason tag (FOOP-43 Component 3).** When a search under a detachment settles ECONSTANIC because
  a candidate was skipped by a detachment, set its `EconstanicReason::Detached`. (Adds the
  `Detached` variant to the enum FOOP-43 introduces.)
- **Keep existing SFF unchanged.** Naked `<<>>` keeps its current implementation; `[*]<<>>`
  forwards to it. The **new code path is only for specific (non-`*`) detachments** — an exclusion
  list over otherwise-normal search. Engage the prefilter iff `detachments` is non-empty and ≠
  `[*]`.

## Test Plan

- Unit: a `[a]<<…>>` marker sets active detachments; a search under it skips a candidate whose
  name matches `a`; `<E>` ≡ `[]<E>` (nothing skipped) and `<<E>>` ≡ `[*]<<E>>` (all skipped);
  detachment applies identically on SF and SFF for a given `[…]`; constanic-clone strips it.
- Approval: `[tmp.*]<<a?x>>` hides `tmp_k`; the two spectrum-extreme demonstrations.
- **Triple documentation** (see Plan).

## Rejected Alternatives

### A. A `[..]{..}` brane construct (the earlier reading)

Detachment as explicit AB/IB brane recoordination. **Rejected**: it is really a search prefilter
riding on the stay-foolish markers, which also cleanly explains the SF/SFF default asymmetry.

### B. Make it a new FIR

A dedicated `DetachmentFir`. **Rejected**: it is a field on the existing SF/SFF FIRs + a matcher
prefilter — no new stepping behavior.

## Open Questions

- Whether detachment patterns match on name only, or also value/characterization.
- Disposition of the old `[..]{..}` `DetachmentBrane` parse form (repurpose or deprecate).
- Must `[..]` be followed by a stay-foolish mark (error otherwise)?
- **How detachment helps recursion** — the specific recursion patterns detachment should enable
  (hiding a recursion variable's outer binding so the inner call rebinds cleanly?) — explore with
  the recursion FOOP (FOOP-34).
- (Strict detachment `[[…]]` is out of scope — backburnered; see the Appendix.)

## Plan (lean)

- [ ] Unit tests (spectrum extremes; skip behavior; clone-strip).
- [ ] Add `detachments` to `StayFoolishFir`/`StayFullyFoolishFir`.
- [ ] Parser: recognize `[patterns]` before an SF/SFF mark; attach to the marker node.
- [ ] Extend `Scope` with `active_detachments`; push in `step_inner`.
- [ ] Add the exclusion prefilter to the scan loop (new path only for specific detachments).
- [ ] **Doc TODO (mandated, three surfaces): write the SF≡`[]` / SFF≡`[*]` spectrum into
      README.md, code comments (SF/SFF FIRs + prefilter), and the snapshot tests themselves.**
- [ ] Approval cases; comprehensive `foop_24_comprehensive.foo`.
- [ ] Worktree lifecycle per `foop.md`.

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
