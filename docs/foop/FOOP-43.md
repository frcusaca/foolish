---
foop: 34
title: Search settlement — miss becomes ECONSTANIC, and coordination removes search context
author: Atlas hc.busy@gmail.com
status: Superseded
type: Standards
created: 2026-07-09
phase: phase-2
supersedes: []
begun: [ ]
---

# FOOP-43: Search settlement — miss → ECONSTANIC, and coordination removes search context

> **Superseded 2026-07-14 — merged into [FOOP-93](FOOP-93.md)** (roadmap Track 2 opener).
> Components 1/2/3 are normatively restated there; this file remains as the full discussion.

> Lean draft. Fuller-spec notes are in the Appendix and in
> `docs/foop/NOTES-creation-lineage-and-search-family.md` §7.

**Three components** (all about how a search's *context* settles and propagates):
1. **Miss → ECONSTANIC** (not NK) — a not-found search may recoordinate; only found-`???` and
   provable-impossibility stay NK.
2. **Coordination removes search context** — a coordinated (referenced) search is just its value;
   its positional context (the `FoolRefFir`) is stripped, so a continued `&`-search off it NKs.
3. **ECONSTANIC records *why*** — a reason tag (Miss | Detached | CharDemand | …) on the ECONSTANIC
   settlement, so downstream FOOPs read intent instead of re-deriving it. (Atlas: yes, add now.)

## Abstract

**Component 1.** A search that **exhausts its candidate stream with no match** (a *miss*) currently
settles **NK**
when anchored (`fir_kinds.rs:1277`) — the rule FOOP-23 and AGENTS.md document as "anchored miss →
NK, provably not in that brane." This FOOP revises that: a **miss settles ECONSTANIC** (may gain a
value via recoordination), regardless of anchoring. **NK survives only for provable
unknowability** — a *found* value that is `???` (NK), or a provable-impossibility like an index
out of a settled finite brane. This is foundational: the search family (FOOP-53/63/24) and the
type system (FOOP-44) all rely on "not-found means *wait*, not *die*."

## Motivation

The bug is visible in `{ a = b.c.d }` where `b` is undefined. Today the inner search for `b`
misses → settles NK → the `.c` deepen sees an NK anchor (`fir_kinds.rs:1252`) → forces NK → `a`
is NK. But `b` is **not provably absent** — the brane is still being coordinated; `b` could
resolve later. `a` should be **WOCONSTANIC** (waiting on `b`'s ECONSTANIC search), not NK.

The discriminator that today's code misses is **found-but-NK vs not-found**:

- `{ b = ???, a = b.c.d }` → `a` **stays NK**. `b` *is found*; its value is `???` (NK). Deepening
  into a genuinely-unknowable value is unknowable → NK **propagates**. Correct, terminal.
- `{ a = b.c.d }` (no `b`) → `a` is **WOCONSTANIC**. `b` is *not found* — the search **missed**.

The current code conflates these: anchored miss settles NK, so a not-found `b` is
indistinguishable from a found-`???` `b` at the point `.c` deepens.

## Specification

### Component 1 — miss → ECONSTANIC

For any search over a candidate stream:

- **Miss (stream exhausted, no candidate matched) → ECONSTANIC.** Unchanged for unanchored
  searches; **changed for anchored searches** (were NK). The result "may gain a value via
  recoordination."
- **Found a value that is `NK` → NK propagates.** Deepening/reading a genuinely-unknowable value
  is unknowable. Terminal.
- **Provable-impossibility → NK.** Cases where the answer is provably determined-absent on a
  *settled* structure — e.g. `#N` out of range on a settled finite brane, head/tail of a settled
  empty brane. (Enumerate and preserve these; they are the *only* remaining NK-by-structure
  cases.)

A deepen-chain (`b.c.d`) whose anchor **missed** becomes **WOCONSTANIC** (waiting on the
anchor's ECONSTANIC search), not NK.

This revises FOOP-23 §"Miss" and AGENTS.md §"NK vs ECONSTANIC miss outcomes": "anchored miss →
NK" becomes "anchored miss → ECONSTANIC." Those documents must be updated.

### Component 2 — coordination removes search context (the `&`-anchor rule)

A search result carries **two** things (FOOP-23): its **value** (`ubc_children[0]`) and its
**position** (`ubc_children[1]`, a `FoolRefFir` — the found statement's place, which a contexted
`&`-search reads). **Coordination — referencing a search result by name (a constanic clone) —
keeps only the value and strips the position.** A coordinated search is *just its value*; it is
positionless.

Consequently:

- **`{ a = ?x; b = a&=3 }` → `b` is NK.** `a` is a search (it found a position). But `b` references
  `a`, coordinating it → `a`'s position is gone → `a&=3` (a contexted `&`-search) has no position
  to continue from → **NK**.
- **General rule: a continued search (`&`-prefix) NKs when its anchor is not a *contexted search*
  carrying a live position.** The `&` continuation connector requires an anchor that still holds a
  `FoolRefFir` (an in-place, un-coordinated search result). A coordinated value, or any
  non-positional anchor, makes the `&`-search NK.

**Current behavior is the bug:** the constanic-clone `Search` arm (`fir_kinds.rs:246-253`) copies
**all** `ubc_children` including `[1]`, so coordination currently *preserves* the position. The
fix is to clone **only `[0]`** (the value) for a coordinated search, dropping `[1]`.

**Empirical requirement (Atlas):** this changes what `&`-off-coordinated-values does, which will
flip snapshots. **Review the existing snapshots** to (a) find where coordinated-then-`&` occurs,
(b) confirm the NK outcome is right in each, and (c) settle the exact rule (which anchors count as
"carrying a live position") against the real corpus **before finalizing.** This is part of the
FOOP.

### Component 3 — ECONSTANIC records *why* it settled (reason tag)

After this batch, a search settles ECONSTANIC for **distinct reasons** that mean different things:
- **Miss** — genuinely found no matching candidate (Component 1).
- **Detached** — a candidate was hidden by a detachment pattern (FOOP-24 `[…]`/`[*]`).
- **CharDemand** — the candidate existed but had the wrong characterization (FOOP-63).
- (extensible — future reasons slot in.)

**Add a reason tag to the ECONSTANIC settlement** — a small enum recorded when a FIR settles
ECONSTANIC (parallel to the existing NK reason string, e.g. `NkFir.reason`). Downstream FOOPs read
the reason instead of re-deriving intent from context. This is cheap to thread through now and a
wide retrofit later, so it lands with this FOOP even though its *consumers* are FOOP-24/63.

- **Design the enum here** (`EconstanicReason { Miss, Detached, CharDemand, … }` or similar) so the
  producers (miss branch; detachment prefilter; char gate) and consumers agree on one vocabulary.
- **WOCONSTANIC** dependents may want to carry/propagate the underlying reason too (a WOCONSTANIC is
  waiting on an ECONSTANIC) — decide whether the reason bubbles up the chain.
- This does **not** change *which* searches settle ECONSTANIC (Components 1–2 do that); it only
  annotates *why*.

## FIR Impact

No new FIR kind. Three behavioral changes: (1) the **settlement value** of a missed `SearchFir`
and the chain-propagation read of an NK anchor; (2) the **constanic-clone of a `Search`** drops
`ubc_children[1]` (the `FoolRefFir` position); (3) an **`EconstanicReason` tag** recorded on the
ECONSTANIC settlement (a small enum; where it is stored — on the ProtoBrane alongside NYES, or a
per-kind field — is an impl choice).

## UBC Step Impact

**Component 1:**
- **`SearchFir` miss branch** (`fir_kinds.rs:1277`): anchored miss `Nyes::Nk` → `Nyes::Econstanic`.
- **Deepen-chain NK check** (`fir_kinds.rs:1252`, `resolve_anchor` → NK): must fire only when the
  anchor's NK is a *found-`???`*, not a *miss*. Once miss ≠ NK, a not-found anchor is ECONSTANIC →
  the chain follows `deepest_econstanic_in_chain` (`fir_kinds.rs:86`) to WOCONSTANIC, while a
  found-`???` anchor stays NK → chain NK. The two cases separate automatically.
- **Value-search miss paths** (`value_search_step`): audit for the same miss→ECONSTANIC (FOOP-23
  already flagged this at FOOP-23.md:1051-1054).

**Component 2:**
- **Constanic-clone `Search` arm** (`fir_kinds.rs:246-253`): currently clones **all**
  `ubc_children` including `[1]` (the `FoolRefFir`). Change to clone **only `[0]`** (the value) —
  coordination drops the position.
- **Contexted `&`-search step** (`fir_kinds.rs:895-902`, reads anchor `ubc_children[1]`): when the
  anchor has no `[1]` (a coordinated value, or a non-positional anchor), settle **NK** instead of
  returning `None`/looping. (Decide NK vs the current `None` outcome against the snapshot review.)

**Component 3:**
- **Define `EconstanicReason`** and set it wherever a search settles ECONSTANIC — the miss branch
  (`Miss`); later, the detachment prefilter (`Detached`, FOOP-24) and the char gate (`CharDemand`,
  FOOP-63) set their reasons. For *this* FOOP, only `Miss` is produced; the enum + plumbing land
  now so FOOP-24/63 can add their variants without a retrofit.

## Test Plan

**Component 1:**
- Unit: `{a=b.c.d}` (no `b`) → `a` WOCONSTANIC; `{b=???, a=b.c.d}` → `a` NK; bare `a?zzz`
  (anchored miss) → ECONSTANIC; `#N` out-of-range on a *settled* brane → still NK; head/tail of
  settled empty brane → still NK.

**Component 2:**
- Unit: `{a=?x; b=a&=3}` → `b` NK (coordinated `a` has no position); a contexted `&` off an
  *in-place* (un-coordinated) search still resolves (regression guard the position isn't dropped
  too eagerly — only on coordination/clone).
- Verify the constanic-clone of a search drops `ubc_children[1]`.

**Both:**
- Approval: re-review **every** snapshot currently NK-by-absence AND **every** snapshot where a
  coordinated search is followed by `&` (expect diffs — treat as *semantic* review per AGENTS.md).
  The snapshot review is where the exact Component-2 rule is finalized.
- Update FOOP-23 and AGENTS.md prose (both the miss rule and the "coordination removes search
  context" rule).

## Rejected Alternatives

### A. Keep anchored-miss → NK (do nothing)

The status quo. **Rejected**: it prematurely commits to NK on an absent name while context is
incomplete, breaking `{a=b.c.d}` and blocking FOOP-24/63/04 (which need "miss = wait").

### B. Everything (even found-`???`) becomes ECONSTANIC

Drop NK-by-absence entirely. **Rejected**: a *found* `???` is genuinely unknowable and must stay
NK (per the discriminator). Only *misses* become ECONSTANIC.

## Open Questions

- **(Component 1)** Exact enumeration of the "provable-impossibility" cases that keep NK (`#N`
  out-of-range; empty-brane head/tail; others?).
- **(Component 2)** The exact rule for which anchors carry a "live position" for `&` — settled
  against the snapshot review. Does `&`-off-no-position settle NK, or ECONSTANIC (could a position
  reappear on recoordination)? (Lean: NK — a coordinated value is positionless by construction.)
- **(Component 3)** The `EconstanicReason` enum's exact variants and where it is stored; whether a
  WOCONSTANIC bubbles up its dependency's reason. (Promoted from an open question to Component 3 —
  Atlas: add the reason tag now. Note: this reason tag is *also* what FOOP-24's backburnered strict
  detachment would have needed to tell "detached-away" from "genuinely-absent" — though the
  *future-coordination* case there stays undecidable regardless.)
- Snapshot-churn scope for all three components — how many approved snapshots flip.

## Plan (lean)

**Component 1 — miss → ECONSTANIC:**
- [ ] Enumerate the NK-survivor cases (found-`???`, provable-impossibility); unit tests first.
- [ ] Change the anchored-miss settlement (`fir_kinds.rs:1277`) to ECONSTANIC.
- [ ] Fix the deepen-chain NK check (`fir_kinds.rs:1252`) to distinguish found-`???` from miss.
- [ ] Audit `value_search_step` miss paths.

**Component 2 — coordination removes search context:**
- [ ] Unit test `{a=?x; b=a&=3}` → NK; `&` off an in-place search still resolves.
- [ ] Constanic-clone `Search` arm (`fir_kinds.rs:246-253`): clone only `[0]`, drop `[1]`.
- [ ] Contexted `&`-search step (`fir_kinds.rs:895`): no anchor position → NK.
- [ ] **Snapshot review** to find coordinated-then-`&` cases and finalize the exact rule.

**Component 3 — ECONSTANIC reason tag:**
- [ ] Define `EconstanicReason` (variants: `Miss`, plus room for `Detached`/`CharDemand`); decide
      storage (ProtoBrane alongside NYES vs per-kind field).
- [ ] Set `Miss` at the miss branch(es). (FOOP-24/63 add `Detached`/`CharDemand` in their FOOPs.)
- [ ] Decide whether WOCONSTANIC propagates its dependency's reason.

**All:**
- [ ] Update FOOP-23 (§Miss and the coordination/context rule) and AGENTS.md.
- [ ] Regenerate snapshots; present to human for semantic review (never auto-accept).
- [ ] Worktree lifecycle per `foop.md` (create / verify / merge / cleanup).

## Appendix — notes toward the full spec

- This is the **keystone** of the search family (renumbered batch): FOOP-24 (detachment
  reject-all/`[*]`/naked-`<<>>`), FOOP-04 (cascade "fail" signal), FOOP-63 (characterization-demand
  → WOCONSTANIC-wait) all depend on Component 1's miss → ECONSTANIC.
- **Component 2 is conceptually deep:** "coordination frees everything" (the SF/SFF strip rule,
  FOOP-24) and "coordination removes search context" (this) are the same principle — a coordinated
  thing is just its value, shorn of its evaluation scaffolding (marker, position). Consider stating
  them together.
- Relates to the FOOP-51 residual state-machine issues (NYES cleanup) — coordinate if live.
- Component 1's fix is nearly one line at the settlement site, but the *chain-propagation*
  correctness (an NK anchor from a miss vs a found-`???`) is subtle — wire the discriminator
  explicitly; consider a helper that tags *why* a FIR is NK.
- Comprehensive test `foop_43_comprehensive.foo`: chained deepens over undefined vs `???` names,
  anchored/unanchored misses, index-out-of-range, AND `{a=?x; b=a&=3}` coordination-context NK, in
  one program.

## References

- Prior: FOOP-23 (§Miss, the rule being revised; the `FoolRefFir` two-child result), FOOP-24
  (detachment — "coordination frees everything", the sibling principle), the FOOP-51 residual
  issues.
- Code: `fir_kinds.rs:1277` (miss settlement), `:1252` (deepen NK check), `:86`
  (`deepest_econstanic_in_chain`), `:246-253` (constanic-clone Search — the `[1]` copy),
  `:895-902` (contexted `&` reads anchor `[1]`); AGENTS.md §"NK vs ECONSTANIC miss outcomes".
- Notes: `docs/foop/NOTES-creation-lineage-and-search-family.md` §7 + Engineering guidance.

## Last Updated

**Date**: 2026-07-09
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes**: Lean draft, now **two components**. (1) Anchored search miss settles ECONSTANIC (not
NK); found-`???` propagates NK; provable-impossibility keeps NK. (2) **Coordination removes search
context** (Atlas 2026-07-09): a coordinated/referenced search is just its value — the `FoolRefFir`
position (`ubc_children[1]`) is stripped on constanic clone, so a continued `&`-search off a
coordinated value NKs (`{a=?x; b=a&=3}` → NK). General rule: a `&`-continuation NKs when its anchor
isn't a contexted search carrying a live position. Requires a snapshot review to finalize the exact
rule. Foundational keystone for the search family and type system.
