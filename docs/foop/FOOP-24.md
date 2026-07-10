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

### Strict detachment — `[[p1,p2,…]]<E>` / `[[…]]<<E>>` (proposed extension, Atlas)

**The thought:** a (single-bracket) detachment writes down what must *not* resolve. But an
ECONSTANIC search that was **not** caused by a detachment is also unresolved — silently. If we are
already declaring non-resolution, should we declare *all* of it? Strict detachment says yes: it is
a **completeness assertion** — "these are *all* the names this expression may leave unresolved;
anything else unresolved is a bug."

**Semantics.** Under `[[p1,…]]`, **every ECONSTANIC search inside `E` must be *accounted for* by
some detachment pattern.** An ECONSTANIC search that matches **no** pattern → that search settles
**NK**, which propagates the brane to NK. (Single `[]` stays permissive: unaccounted ECONSTANIC is
fine — it waits for coordination.)

**"Accounted for" is a RUNTIME question — decidability of regex containment is irrelevant.**
(Corrected 2026-07-09 — an earlier draft wrongly worried about static pattern-subset
undecidability.) Detachment is applied per-candidate *at run time*: the scan already tests each
concrete candidate name against each detachment pattern (`SearchFir::matches_pattern` — a concrete
string vs a regex, trivially decidable). Strict mode reuses exactly that runtime signal.

**The rule (runtime, decidable):** a search that settles ECONSTANIC is **accounted for** iff,
during its scan, **it actually skipped at least one candidate because of a detachment pattern** —
i.e. the detachment is *why* it found nothing. If it found nothing with **no candidate skipped by
a detachment** (the name is genuinely absent), it is **unaccounted → the search settles NK**
(propagating the brane to NK). This directly captures "did the detachment cause this
non-resolution," and it's just a boolean the scan sets while it runs.

- Implementation: the scan loop (`fir_kinds.rs:1978`) already decides per candidate whether to
  skip (detachment prefilter) vs reject (predicate). Under a strict marker, track a
  `skipped_by_detachment: bool` for the scan; at the ECONSTANIC settle point, `Miss &&
  !skipped_by_detachment` → NK.
- No pattern analysis, no `as_search_pattern` comparison, no regex containment — the concrete
  runtime skips are the whole signal. Literal-name and regex search patterns are handled
  identically and correctly.

`[[` lexes as two `LBracket`s (no new token).

**Interaction with the spectrum.** `[[]]` (empty strict) means **no detachment pattern is present,
so no ECONSTANIC search can ever be accounted-for → every ECONSTANIC search NKs** — i.e. "**no
search may go ECONSTANIC; everything must resolve or the brane is NK.**" A strong assertion, useful
on its own. `[[*]]` detaches everything (every candidate skipped), so every search's ECONSTANIC
*is* accounted-for (it skipped candidates) → it behaves like `[*]` but asserts that *every* search
did in fact hit a detachment. Whether `[[*]]` differs observably from `[*]` is minor; the useful
range is the specific `[[p1,…]]` middle and the `[[]]` endpoint.

## FIR Impact

No new FIR kind. Add to **both** `StayFoolishFir` (`fir_kinds.rs:2021`) and `StayFullyFoolishFir`
(`:2082`): `detachments: Vec<String>` and a `strict: bool` (the `[[…]]` flag). A parameterized
marker is just SF/SFF with a non-empty `detachments`; strict is the double-bracket variant.

## UBC Step Impact

- **Scope handoff.** Extend `Scope` (`fir_trait.rs:55`, today just `has_ancestral_sfm: bool`) to
  carry the **active detachment patterns** (and the strict flag) accumulated from enclosing
  parameterized marks (`active_detachments: Vec<String>`, `strict_detachment: bool`), pushed in
  `step_inner` (`fir_trait.rs:347`) when stepping under a parameterized marker.
- **Prefilter (permissive `[]`).** In the scan loop (`contextful_search_scan`, `fir_kinds.rs:1978`),
  before `predicate.matches`, skip a candidate if any active detachment pattern matches it (reuse
  `SearchFir::matches_pattern`). **Same locus as FOOP-93's `!`.**
- **Accounting (strict `[[]]`).** The scan tracks a `skipped_by_detachment: bool` (set when the
  prefilter skips any candidate). At the ECONSTANIC settle sites (`fir_kinds.rs:1273` and the
  value-/contexted-search equivalents), when under a strict marker: `Miss && !skipped_by_detachment`
  → settle **NK** instead of ECONSTANIC. Pure runtime signal — no pattern analysis.
- **Keep existing SFF unchanged.** Naked `<<>>` keeps its current implementation; `[*]<<>>`
  forwards to it. The **new permissive code path is only for specific (non-`*`) detachments**;
  strict is an additional gate on ECONSTANIC settlement.

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
- **Strict detachment `[[]]`.** The "accounted for" rule is settled (runtime:
  `skipped_by_detachment` — see §Strict detachment); remaining Qs: is `[[]]` ("no search may go
  ECONSTANIC") worth shipping as its own assertion? Is strict in scope for the first cut or a
  follow-on? Does a WOCONSTANIC (not just ECONSTANIC) unaccounted search also NK under strict?

## Plan (lean)

- [ ] Unit tests (spectrum extremes; skip behavior; clone-strip).
- [ ] Add `detachments` + `strict` to `StayFoolishFir`/`StayFullyFoolishFir`.
- [ ] Parser: recognize `[patterns]` (and `[[patterns]]` strict) before an SF/SFF mark; attach to
      the marker node.
- [ ] Extend `Scope` with `active_detachments` + `strict_detachment`; push in `step_inner`.
- [ ] Add the exclusion prefilter to the scan loop (permissive path, specific detachments).
- [ ] **Strict:** gate ECONSTANIC settlement on the "accounted for" check (decide the rule first).
- [ ] **Doc TODO (mandated, three surfaces): write the SF≡`[]` / SFF≡`[*]` spectrum (and the
      permissive-vs-strict `[]`/`[[]]` distinction) into README.md, code comments, and the
      snapshot tests themselves.**
- [ ] Approval cases (incl. a `[[…]]` strict accounted/unaccounted pair); comprehensive
      `foop_24_comprehensive.foo`.
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

## References

- Prior: FOOP-43 (miss semantics), FOOP-93 (shared prefilter locus), FOOP-23 (one-engine model),
  ECOSYSTEM.md (stay-foolish semantics).
- Code: `fir_kinds.rs:2021`/`:2082` (SF/SFF FIRs), `:155-179` (`constanic_clone_at` strip),
  `:1978` (scan loop); `fir_trait.rs:55`/`:347` (Scope handoff).
- Notes: `NOTES-creation-lineage-and-search-family.md` §2 + Engineering guidance; memory
  `foop-detachment-as-parameterized-sfmarker`.

## Last Updated

**Date**: 2026-07-09
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes**: Renumbered from FOOP-63 to FOOP-24 (impl-order reorg — detachment now its own FOOP
just before recursion, since tightening it should help recursion definitions). Parameterized SF/SFF
marker (exclusion-list prefilter); SF≡`[]` / SFF≡`[*]` spectrum; triple-documentation mandate;
depends on FOOP-43.
**Added §Strict detachment `[[]]`** (Atlas coherence pass): a completeness assertion — under
`[[…]]`, every ECONSTANIC search must be *accounted for* by a detachment, else that search → NK
(brane NK). **"Accounted for" is a RUNTIME rule (corrected — regex-containment undecidability is
IRRELEVANT, Atlas):** a search's ECONSTANIC is accounted-for iff its scan actually **skipped a
candidate due to a detachment** (`skipped_by_detachment` bool); `Miss && !skipped_by_detachment`
→ NK. Pure runtime signal, no pattern analysis. Corollary: `[[]]` (empty strict, no patterns) =
"no ECONSTANIC search can be accounted-for → **no search may go ECONSTANIC**". Added `strict` flag
to the FIR/Scope/plan.
