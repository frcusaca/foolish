---
foop: 34
title: Search settlement — miss settles by anchoring, SFF-marked searches are ECONSTANIC, and coordination removes search context
author: Atlas hc.busy@gmail.com
status: Superseded
type: Standards
created: 2026-07-09
phase: phase-2
supersedes: []
begun: [ ]
---

# FOOP-43: Search settlement — miss settles by anchoring, and coordination removes search context

> **Superseded 2026-07-14 — merged into [FOOP-93](FOOP-93.md)** (roadmap Track 2 opener).
> Components 1/2/3 are normatively restated there; this file remains as the full discussion.

> Lean draft. Fuller-spec notes are in the Appendix and in
> `docs/foop/NOTES-creation-lineage-and-search-family.md` §7.
>
> **Terminology: cite FOOP-84 Part 0, do not restate.** This FOOP's vocabulary — **anchored /
> unanchored** and **what a miss proves** (§0.2), **search context** (§0.3: home brane *in its own
> context* + statement number, carried by `FoolRefFir` at `ubc_children[1]` — the thing Component
> 2 strips on coordination), **contextless / contexted search** (§0.4), the **detachment family**
> (§0.5), and **NK vs ECONSTANIC** (§0.2) — is defined in FOOP-84 Part 0. FOOP-84 §1.5 restates
> this FOOP's Component 1 settlement rule and must stay in sync with it.

**Three components** (all about how a search's *context* settles and propagates):
1. **Miss settles by anchoring** — an **unanchored** miss → ECONSTANIC (may recoordinate); an
   **anchored** miss stays **NK** (provably not in that brane); an **SFF-marked search** →
   ECONSTANIC regardless of anchoring. Found-`???` and provable-impossibility stay NK.
2. **Coordination removes search context** — a coordinated (referenced) search is just its value;
   its positional context (the `FoolRefFir`) is stripped, so a continued `&`-search off it NKs.
3. **ECONSTANIC records *why*** — a reason tag (Miss | Detached | CharDemand | …) on the ECONSTANIC
   settlement, so downstream FOOPs read intent instead of re-deriving it. (Atlas: yes, add now.)

## Abstract

**Component 1.** A search that **exhausts its candidate stream with no match** (a *miss*) settles
according to **how it was anchored**:

- **Anchored miss → NK.** *Unchanged from today* (`fir_kinds.rs:1277`) and from FOOP-23 /
  AGENTS.md. An anchored search names the brane it searches; exhausting that brane without a
  match is a *proof* the name is not there. NK is exactly right, and keeping it preserves NK's
  meaning as "provably unknowable" rather than blurring it into "not found yet."
- **Unanchored miss → ECONSTANIC.** *Unchanged from today.* An unanchored search has no fixed
  brane to prove absence against — it may gain a value via recoordination.
- **SFF-marked search → ECONSTANIC**, regardless of anchoring. A search whose candidates were
  withheld by a stay-fully-foolish marker did not *fail to find* anything; it was *prevented from
  looking*. It must stay recoordinatable so the marker's whole purpose — defer resolution to the
  use site — still works. See Component 3 for the reason tag that distinguishes this from a
  genuine miss.

**NK otherwise survives for provable unknowability**: a *found* value that is `???` (NK), or a
provable-impossibility like an index out of a settled finite brane.

**What this FOOP actually changes.** Component 1 is now largely a *codification* of existing
behavior rather than a revision of it — the real change is the third bullet (SFF-marked searches
must not be swept into the anchored-miss→NK rule) plus the NK-propagation fix described in
Motivation. Components 2 and 3 are unchanged and carry the rest of this FOOP's substance.

## Motivation

The bug is visible in `{ a = b.c.d }` where `b` is undefined. Today the inner search for `b`
misses → settles NK → the `.c` deepen sees an NK anchor (`fir_kinds.rs:1252`) → forces NK → `a`
is NK.

Note **`b` is unanchored** — it heads the chain, so there is nothing to anchor it — and an
unanchored miss is ECONSTANIC under the rule above. So the defect here is *not* miss settlement.
It is **NK propagation through an anchor**: `.c` and `.d` are anchored searches that never miss;
they are *waiting* on an anchor that has not resolved. The correct result is

```
a = wconstanic("d", wconstanic("c", econstanic("b")))
```

i.e. `a` is **WOCONSTANIC**, waiting outward through the deepen chain on `b`'s ECONSTANIC search.
`b` is **not provably absent** — the brane is still being coordinated, and `b` could resolve
later. The fix is that a deepen whose anchor is *unresolved* must wait rather than force NK; it
does not require changing what an anchored miss settles to.

The discriminator that today's code misses is **found-but-NK vs not-found**:

- `{ b = ???, a = b.c.d }` → `a` **stays NK**. `b` *is found*; its value is `???` (NK). Deepening
  into a genuinely-unknowable value is unknowable → NK **propagates**. Correct, terminal.
- `{ a = b.c.d }` (no `b`) → `a` is **WOCONSTANIC**. `b` is *not found* — the search **missed**.

The current code conflates these: anchored miss settles NK, so a not-found `b` is
indistinguishable from a found-`???` `b` at the point `.c` deepens.

## Specification

### Component 1 — miss settles by anchoring; SFF-marked searches are ECONSTANIC

For any search over a candidate stream:

- **Anchored miss → NK.** *Unchanged.* The anchor names the brane; exhausting it without a match
  proves the name is absent from that brane. Terminal.
- **Unanchored miss → ECONSTANIC.** *Unchanged.* No fixed brane, so no proof of absence; "may
  gain a value via recoordination."
- **SFF-marked search → ECONSTANIC**, regardless of anchoring. **This is the change.** A search
  that came up empty because a stay-fully-foolish marker withheld its candidates has proved
  nothing — it never got to look. Settling it NK would make the marker destroy the search instead
  of deferring it, defeating SFF's purpose. It settles ECONSTANIC and carries
  `EconstanicReason::Detached` (Component 3) so the distinction is legible downstream.
- **Found a value that is `NK` → NK propagates.** Deepening/reading a genuinely-unknowable value
  is unknowable. Terminal.
- **Provable-impossibility → NK.** Cases where the answer is provably determined-absent on a
  *settled* structure — e.g. `#N` out of range on a settled finite brane, head/tail of a settled
  empty brane. (Enumerate and preserve these.)

**Separately — the NK-propagation fix.** A deepen-chain (`b.c.d`) whose anchor is **unresolved**
(ECONSTANIC/WOCONSTANIC) becomes **WOCONSTANIC**, waiting on that anchor — it must not force NK.
This is independent of miss settlement and is the actual defect Motivation describes.

**Relationship to FOOP-23 / AGENTS.md.** The "anchored miss → NK" rule those documents state is
**correct and stands** — no update needed there. What they do not yet cover is the SFF-marked
case, which this FOOP adds.

**Why not "all misses → ECONSTANIC."** An earlier draft of this FOOP made miss → ECONSTANIC
unconditional. Rejected: it costs NK its precise meaning. Under the rule above, NK continues to
mean "provably unknowable," which keeps anchored search a genuine assertion about a named brane
and lets real errors surface early. Programs that want deferred resolution have an explicit,
readable opt-out — wrap the search in `<<…>>` — rather than getting it implicitly from every
anchored miss in the program.

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
- **`SearchFir` miss branch** (`fir_kinds.rs:1277`): **unchanged** — anchored miss keeps
  `Nyes::Nk`, unanchored miss keeps `Nyes::Econstanic`.
- **SFF-marked searches**: a search whose candidates were withheld by a stay-fully-foolish marker
  must settle `Nyes::Econstanic` with `EconstanicReason::Detached`, *not* fall through to the
  anchored-miss NK branch. This requires the withheld-because-of-a-marker case to be
  **distinguishable** from a genuine exhausted-stream miss at the point of settlement — an empty
  candidate stream alone is not enough information. See FOOP-84 §2.4.1, which must be revised to
  match (it currently assumes exhaustion-implies-ECONSTANIC with no reason tag).
- **Deepen-chain NK check** (`fir_kinds.rs:1252`, `resolve_anchor` → NK): must fire only when the
  anchor's NK is a *found-`???`*, not when the anchor is merely **unresolved**
  (ECONSTANIC/WOCONSTANIC). An unresolved anchor makes the chain follow
  `deepest_econstanic_in_chain` (`fir_kinds.rs:86`) to WOCONSTANIC; a found-`???` anchor stays NK
  → chain NK. **This is the real Component-1 code change** (the miss branch itself is untouched).
- **Value-search miss paths** (`value_search_step`): audit that they follow the same
  anchored→NK / unanchored→ECONSTANIC split (FOOP-23 flagged this at FOOP-23.md:1051-1054).

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
- Unit: `{a=b.c.d}` (no `b`) → `a` WOCONSTANIC, structurally
  `wconstanic("d", wconstanic("c", econstanic("b")))`; `{b=???, a=b.c.d}` → `a` NK; bare `a?zzz`
  (anchored miss) → **still NK** (regression guard — the rule is unchanged); `?zzz` (unanchored
  miss) → ECONSTANIC; a search under `<<…>>` that finds nothing → ECONSTANIC with reason
  `Detached`, **even when anchored**; `#N` out-of-range on a *settled* brane → still NK; head/tail
  of settled empty brane → still NK.

**Component 2:**
- Unit: `{a=?x; b=a&=3}` → `b` NK (coordinated `a` has no position); a contexted `&` off an
  *in-place* (un-coordinated) search still resolves (regression guard the position isn't dropped
  too eagerly — only on coordination/clone).
- Verify the constanic-clone of a search drops `ubc_children[1]`.

**Both:**
- Approval: re-review **every** snapshot where a deepen-chain sits on an unresolved anchor, AND
  **every** snapshot where a coordinated search is followed by `&` (expect diffs — treat as
  *semantic* review per AGENTS.md). Snapshots that are NK purely by *anchored absence* should
  **not** change; a diff there is a regression, not an improvement. The snapshot review is where
  the exact Component-2 rule is finalized.
- Update AGENTS.md prose for the "coordination removes search context" rule only. AGENTS.md's
  §"NK vs ECONSTANIC miss outcomes" ("anchored miss → NK") is **correct as written** and must be
  left alone; add the SFF-marked case beside it.

## Rejected Alternatives

### A. Make *all* misses (including anchored) → ECONSTANIC

An earlier draft of this FOOP. **Rejected** (Atlas, 2026-07-28): it costs NK its precise meaning.
"Provably not in this named brane" is a genuine proof and deserves a terminal state; blurring it
into "not found yet" makes anchored search stop asserting anything and pushes real errors
downstream where they surface as confusing WOCONSTANIC chains instead of a clear NK at the point
of the mistake. Deferred resolution remains available, but **explicitly** — wrap the search in
`<<…>>` — which reads at the use site instead of being an implicit property of every anchored
miss in the program. The `{a=b.c.d}` case that motivated the blanket rule turns out not to need
it: `b` is *unanchored*, so it is already ECONSTANIC; that example was only ever about NK
propagation through an unresolved anchor.

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

**Component 1 — miss settles by anchoring; SFF-marked → ECONSTANIC:**
- [ ] Enumerate the NK-survivor cases (found-`???`, provable-impossibility); unit tests first.
- [ ] Regression-guard the *unchanged* settlements: anchored miss stays NK
      (`fir_kinds.rs:1277`), unanchored miss stays ECONSTANIC.
- [ ] Make an SFF-withheld search settle ECONSTANIC (reason `Detached`) rather than reaching the
      anchored-miss NK branch — requires distinguishing "withheld by a marker" from "stream
      genuinely exhausted" at the settlement site.
- [ ] Fix the deepen-chain NK check (`fir_kinds.rs:1252`) to distinguish found-`???` from an
      *unresolved* anchor. **This is the substantive code change in Component 1.**
- [ ] Audit `value_search_step` miss paths for the same anchored/unanchored split.

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
- [ ] Update FOOP-23 and AGENTS.md for the coordination/context rule **only** — leave their
      "anchored miss → NK" prose intact (it is correct); add the SFF-marked case beside it.
- [ ] Revise **FOOP-84 §1.5 and §2.4.1** to match this Component 1 (§1.5 currently claims
      anchored miss → ECONSTANIC; §2.4.1 currently derives full-detachment ECONSTANIC from plain
      stream exhaustion, which under this rule would settle NK for an anchored search).
- [ ] Regenerate snapshots; present to human for semantic review (never auto-accept).
- [ ] Worktree lifecycle per `foop.md` (create / verify / merge / cleanup).

## Appendix — notes toward the full spec

- This is the **keystone** of the search family (renumbered batch): FOOP-24/FOOP-24 (detachment
  reject-all/`[*]`/naked-`<<>>`), FOOP-04 (cascade "fail" signal), FOOP-63 (characterization-demand
  → WOCONSTANIC-wait) all depend on Component 1. Note the dependency is specifically on the
  **SFF-marked → ECONSTANIC** bullet, *not* on anchored misses recoordinating — detachment needs
  "withheld candidates leave the search deferrable," which is exactly that bullet.
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

**Date**: 2026-07-28 (2)
**Updated By**: Claude Code (Opus 5)
**Changes**: Added the "cite FOOP-84 Part 0" terminology banner. FOOP-84 Part 0 is now the single
definition site for anchoring/miss outcomes (§0.2), search context (§0.3 — the thing Component 2
strips on coordination, now formally defined as home brane *in its own context* + statement
number), the search families (§0.4), and the detachment family (§0.5). FOOP-84 §1.5 restates this
FOOP's Component 1 rule and must stay in sync.

**Date**: 2026-07-28
**Updated By**: Claude Code (Opus 5)
**Changes**: **Component 1 rewritten to the settled rule** (Atlas, this session): *anchored miss →
NK* (unchanged — an anchored search names its brane, so exhausting it genuinely proves absence,
and NK must keep meaning "provably unknowable"), *unanchored miss → ECONSTANIC* (unchanged), and
**SFF-marked searches → ECONSTANIC regardless of anchoring** — the actual new rule, because a
search whose candidates a marker withheld never got to look and must stay deferrable. The prior
draft's blanket "all misses → ECONSTANIC" is moved to Rejected Alternatives (A) with the reasoning
recorded; the old A ("keep anchored-miss → NK") is gone, being the position now adopted.
Corrected the `{a=b.c.d}` motivating example: `b` is **unanchored**, so it was never an
anchored-miss case at all — the real defect there is NK propagation through an *unresolved*
anchor, and the correct result is `wconstanic("d", wconstanic("c", econstanic("b")))`. That makes
the deepen-chain fix (`fir_kinds.rs:1252`) the substantive Component-1 code change, while the miss
branch (`:1277`) is now untouched. Updated UBC Step Impact, Test Plan (anchored miss → NK is now a
*regression guard*), and the plan checkboxes to match. Noted that FOOP-23/AGENTS.md "anchored miss
→ NK" prose is **correct and must be left alone**, reversing this FOOP's prior instruction to
rewrite it. Flagged that **FOOP-84 §1.5 and §2.4.1 must be revised** to match, and that
detachment's dependency on this FOOP is specifically the SFF-marked bullet, not anchored-miss
recoordination.

**Date**: 2026-07-09
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes**: Lean draft, now **two components**. (1) Anchored search miss settles ECONSTANIC (not
NK); found-`???` propagates NK; provable-impossibility keeps NK. (2) **Coordination removes search
context** (Atlas 2026-07-09): a coordinated/referenced search is just its value — the `FoolRefFir`
position (`ubc_children[1]`) is stripped on constanic clone, so a continued `&`-search off a
coordinated value NKs (`{a=?x; b=a&=3}` → NK). General rule: a `&`-continuation NKs when its anchor
isn't a contexted search carrying a live position. Requires a snapshot review to finalize the exact
rule. Foundational keystone for the search family and type system.
