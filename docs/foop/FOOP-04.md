---
foop: 40
title: Cascading connector for search — the | fallback operator
author: Atlas hc.busy@gmail.com
status: Draft
type: Standards
created: 2026-07-09
phase: phase-2
supersedes: []
begun: [ ]
---

# FOOP-04: Cascading connector for search — the `|` fallback operator

> **Roadmap note (2026-07-14, Track 2 #2):** depends on FOOP-93's settlement batch — the
> cascade's whole premise is that a miss is detectable and non-fatal (43-C1 miss→ECONSTANIC
> now lives in FOOP-93). Define cascade-over-ECONSTANIC precisely: `a?x | b?x` falls through
> on Miss-reason ECONSTANIC (read the 43-C3 `EconstanicReason`), and does NOT fall through
> on found-NK. Execute after FOOP-93, before FOOP-14.

> Lean draft. Fuller notes in the Appendix and `NOTES-creation-lineage-and-search-family.md` §9.
> (Implementation order: #7 — Search FOOP B of three. Renumbered 2026-07-09.)

## Abstract

The **cascading connector** `|` runs the next search **only if the previous one fails**.
`(=10 | =4)` returns 4 **only if 10 was not found**: run search-1; if it *misses* (→ ECONSTANIC
per FOOP-43), fall back to search-2 **from the same anchor** — every branch of a cascade is
contexted off the one position that preceded the whole `(...)` block. Unlike the matcher boolean
operators `&&`/`||` (FOOP-93, per-candidate), `|` is **control-flow between whole searches**, so
it needs its own stateful **`CascadingSearchFir`**.

Vocabulary (per FOOP-93 / to be documented): `|` is the *cascading connector for search*; `&` is
the *continuation connector for search* (existing contexted `&`); `&&`/`||` are *matcher boolean
operators* (FOOP-93).

## Motivation

"Try this, else that" is a routine query need that neither a single predicate nor `&&`/`||`
expresses — those decide per candidate; `|` sequences *whole searches* by success/failure. It is
also the key enabler for defaulting patterns (search a specific name, else fall back to a general
one).

## Specification

- **All branches share one anchor context.** `CascadingSearchFir` is not a chain of independent
  searches — every branch is a *contexted* search (`&`-form) that starts from the **same anchor**:
  wherever the expression preceding the cascade left off. `blahblah(=4 | =10 | =1)` searches for
  `4` from `blahblah`'s position; if that's ECONSTANIC (miss, FOOP-43), `=10` searches from
  **`blahblah`'s position again** (not from where `=4`'s failed attempt would have left a cursor —
  it never established one); if that also misses, `=1` searches from `blahblah`'s position.
  (Atlas, 2026-07-11 — supersedes the earlier "resume from nearest earlier established position"
  reading below, which conflated the cascade with a moving cursor. There is no moving cursor: the
  anchor is fixed to whatever preceded the `(...)` block, for every branch.)
- **Semantics.** `A | B` — run `A` (contexted, from the shared anchor); if `A` finds a match, that
  is the result; if `A` **misses** (settles ECONSTANIC per FOOP-43), run `B` **from the same
  anchor**; and so on down the chain.
- **Precedence: `&` (continuation) binds tighter than `|` (cascade).** The continuation connector
  `&` glues a search onto the position immediately to its left; `|` only ever separates whole
  `&`-chains, and re-applies `&` to the start of every branch after the first. Concretely:
  `a&=b&=c|=d` parses as `a&=b(&=c|&=d)` — `c` and `d` are **both** contexted off `a&=b`'s
  position (not off each other), because the cascade inserts a fresh `&` in front of each
  fallback branch. This is what "all branches share one anchor" means syntactically: the anchor
  every branch is contexted against is whatever sits to the left of the outermost `|` in the
  chain, and every branch after the first gets its `&` supplied by the cascade itself.

## FIR Impact

A **new `CascadingSearchFir`** (per the engineering guidance — it has its own stepping: run
branches in order against one fixed anchor). Owes a `*_nyes_transitions` test.

## UBC Step Impact

- **`CascadingSearchFir::fir_op_step`:** all branches are contexted (`&`-form) off the **same**
  incoming anchor position (the `FoolRefFir` immediately preceding the `(...)` block — FOOP-23's
  `[1]` child). Run branch[0] from that anchor; if found → result. If it settles a **miss**
  (ECONSTANIC, FOOP-43) → run branch[1] **from the same anchor** (not from anything branch[0] did);
  repeat down the chain. No anchor threading, no "current fallback position" — one fixed anchor,
  reused by every branch via the existing contexted-search resume path.
- **Precedence.** `&` binds tighter than `|`. The cascade parses each fallback branch as if a
  fresh `&` were prepended to it: `a&=b&=c|=d` → `a&=b(&=c|&=d)`, so `c` and `d` are both
  contexted off `a&=b`'s position, not off each other.
- **Parser:** a new `|` token (single pipe — no `Pipe` token today); `|` parses inline in the
  search chain and emits the cascading FIR (lightweight, not a new precedence layer). Typically
  inside a leading `(...)` combine-block (shared with FOOP-93).

## Test Plan

- Unit: the cascade runs branch-2 only on branch-1 miss, and both branches search from the
  *identical* anchor (worked examples for `A|B|C` under each failure pattern, including the
  `a&=b&=c|=d` precedence example); `*_nyes_transitions` for `CascadingSearchFir`.
- Approval: `(=10 | =4)` fallback (10 present→10; 10 absent, 4 present→4; neither→miss); a
  name-then-general-fallback (`(?specific | ~general.*)`); compose with `!`/`&&`/`||` (FOOP-93).

## Rejected Alternatives

### A. `|` as another matcher combinator (like `||`)

Treat cascade as a per-candidate predicate. **Rejected**: `|` is control-flow between *whole
searches*, run in sequence against a shared anchor — it needs a wrapper FIR, not a predicate.
(`||` is the per-candidate one — FOOP-93.)

### C. Anchor-propagation (each branch resumes from the nearest prior branch that "established a
position")

An earlier draft of this FOOP had `A|B|C` thread a moving fallback anchor — B resumes from A's
position, C from B's if B established one, else from A's. **Rejected (Atlas, 2026-07-11):**
wrong model. There is no per-branch position to establish; every branch is contexted off the
*same* fixed anchor that preceded the whole cascade (confirmed by the `a&=b&=c|=d` →
`a&=b(&=c|&=d)` precedence example — `c` and `d` are siblings off `a&=b`, not a chain of
resumptions off each other). This also dissolves the "what counts as established" open question,
since nothing needs to be established.

### B. Bare `~a.*|~*.b` (no parens)

`|` inline between chains without a combine-block. **Rejected**: `|` collides with regex
alternation and lacks clear precedence. The parenthesized combine-block (FOOP-93) avoids both.

## Open Questions

- **RESOLVED (Atlas, 2026-07-11):** the anchor is **not** propagated/threaded between branches —
  every branch of a cascade shares the *one* fixed anchor that preceded the `(...)` block. "What
  counts as an established position" is now moot; there is no per-branch position to establish.
- **RESOLVED (Atlas, 2026-07-11):** precedence is `&` (continuation) binds tighter than `|`
  (cascade); `a&=b&=c|=d` parses as `a&=b(&=c|&=d)` — this also answers "does `|` compose with
  `&&`/`||`" for the `&`/`|` pair specifically (mixed precedence with the *matcher* `&&`/`||`
  inside one `(...)` block remains open, see below).
- Does `|` compose with `&&`/`||` (the FOOP-93 *matcher* boolean operators) inside one `(...)`
  block — i.e. can a single branch be a `&&`/`||` combination, not just a bare search?
- Observable difference between `||` (per-candidate, first candidate matching either) and cascade
  `|` (whole search-1 first, all from the same anchor) — document it.

## Plan (lean)

- [ ] Add the `|` token; parse `|` inline (typically in the leading `(...)` block), prepending a
      fresh `&` to each branch after the first (the `a&=b&=c|=d` → `a&=b(&=c|&=d)` precedence
      rule).
- [ ] `CascadingSearchFir`: run branches in order against one fixed incoming anchor (no threading);
      reuse the contexted-resume path. `*_nyes_transitions` test. **Needs FOOP-43** (miss = fail
      signal).
- [ ] Worked-example unit tests for `A|B|C` under each failure pattern, plus the `a&=b&=c|=d`
      precedence case (the correctness core).
- [ ] Approval cases; comprehensive `foop_04_comprehensive.foo`.
- [ ] Worktree lifecycle per `foop.md`.

## Appendix — notes toward the full spec

- **Search FOOP B of three** (A = predicates FOOP-93, C = all-results FOOP-14). Separated from A
  because `|` is heavy and different-in-kind (stateful wrapper with a shared fixed anchor) vs the
  pure predicate work.
- **Depends on FOOP-43** — the cascade "fail" signal is a miss → ECONSTANIC. Composes with FOOP-93
  (`!` / `&&`/`||`) and FOOP-14 (`~~`/`??`).
- The fixed-shared-anchor semantics and the `&`-binds-tighter-than-`|` precedence rule are the
  correctness core; write the `A|B|C` and `a&=b&=c|=d` worked examples as unit tests before
  implementing.

## References

- Prior: FOOP-23 (`FoolRefFir` position carrier; contexted resume), FOOP-43 (miss = fail signal),
  FOOP-93 (matcher `&&`/`||`, the `(...)` block, vocabulary).
- Code: `FoolRefFir` / `push_search_result_pair`; contexted-search resume path; `parser.rs:573`
  (postfix), `:797` (regex-group).
- Notes: `NOTES-creation-lineage-and-search-family.md` §9 + Engineering guidance.

## Last Updated

**Date**: 2026-07-11
**Updated By**: Claude Code 2.1.119 (Claude Code); Sonnet 5
**Changes**: Replaced the anchor-propagation model with the correct one (Atlas): every branch of
a cascade is contexted off the *same fixed anchor* that preceded the `(...)` block — no threading,
no per-branch "established position." Added the precedence rule `&` binds tighter than `|`
(`a&=b&=c|=d` parses as `a&=b(&=c|&=d)`). This resolves the former top open question outright;
added Rejected-Alt C documenting the discarded anchor-propagation model.

**Date**: 2026-07-09
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes**: New FOOP-04 = the cascading connector `|` (split out from old FOOP-24's beefy search
as Search FOOP B). `CascadingSearchFir` + anchor-propagation (resume from nearest earlier branch
that established a position, via `FoolRefFir`); needs FOOP-43.
