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

> Lean draft. Fuller notes in the Appendix and `NOTES-creation-lineage-and-search-family.md` §9.
> (Implementation order: #7 — Search FOOP B of three. Renumbered 2026-07-09.)

## Abstract

The **cascading connector** `|` runs the next search **only if the previous one fails**.
`(=10 | =4)` returns 4 **only if 10 was not found**: run search-1; if it *misses* (→ ECONSTANIC
per FOOP-43), fall back to search-2. Unlike the matcher boolean operators `&&`/`||` (FOOP-93,
per-candidate), `|` is **control-flow between whole searches** with **anchor-propagation**, so it
needs its own stateful **`CascadingSearchFir`**.

Vocabulary (per FOOP-93 / to be documented): `|` is the *cascading connector for search*; `&` is
the *continuation connector for search* (existing contexted `&`); `&&`/`||` are *matcher boolean
operators* (FOOP-93).

## Motivation

"Try this, else that" is a routine query need that neither a single predicate nor `&&`/`||`
expresses — those decide per candidate; `|` sequences *whole searches* by success/failure. It is
also the key enabler for defaulting patterns (search a specific name, else fall back to a general
one).

## Specification

- **Semantics.** `A | B` — run `A`; if `A` finds a match, that is the result; if `A` **misses**
  (settles ECONSTANIC per FOOP-43), run `B`; and so on down the chain.
- **Anchor-propagation (the subtle core).** `A | B | C` is **not** flat or-else. Each fallback
  branch **resumes from the nearest earlier branch that established a position.** The wrapper
  threads a running "current fallback anchor/position": A runs from the original anchor; A fails →
  B runs from A's; if B *establishes a position* it becomes current, so C resumes from B's; if B
  *also fails to establish one*, C falls back further to A's. ("C searches A's anchor if A and B
  both fail; C searches B's anchor if only B fails.")
- **Position reuse.** "The position a branch established" *is* its `FoolRefFir` (FOOP-23's `[1]`
  child — the same position-carrier `&`-searches read); "resume from B's position" is a contexted
  resume off B's `FoolRefFir`.

## FIR Impact

A **new `CascadingSearchFir`** (per the engineering guidance — it has its own stepping: run
branches in order, thread the fallback anchor). Owes a `*_nyes_transitions` test.

## UBC Step Impact

- **`CascadingSearchFir::fir_op_step`:** run branch[0] from the original anchor; if found → result
  (its position becomes the current fallback); if it settles a **miss** (ECONSTANIC, FOOP-43) →
  run the next branch **from the current fallback position** (nearest earlier established one).
  Reuse the `FoolRefFir` carrier and the contexted-search resume path.
- **Parser:** a new `|` token (single pipe — no `Pipe` token today); `|` parses inline in the
  search chain and emits the cascading FIR (lightweight, not a new precedence layer). Typically
  inside a leading `(...)` combine-block (shared with FOOP-93).

## Test Plan

- Unit: the cascade runs branch-2 only on branch-1 miss and resumes from the correct anchor
  (worked examples for `A|B|C` under each failure pattern); `*_nyes_transitions` for
  `CascadingSearchFir`.
- Approval: `(=10 | =4)` fallback (10 present→10; 10 absent, 4 present→4; neither→miss); a
  name-then-general-fallback (`(?specific | ~general.*)`); compose with `!`/`&&`/`||` (FOOP-93).

## Rejected Alternatives

### A. `|` as another matcher combinator (like `||`)

Treat cascade as a per-candidate predicate. **Rejected**: `|` is control-flow between *whole
searches* with anchor state — it needs a wrapper FIR, not a predicate. (`||` is the per-candidate
one — FOOP-93.)

### B. Bare `~a.*|~*.b` (no parens)

`|` inline between chains without a combine-block. **Rejected**: `|` collides with regex
alternation and lacks clear precedence. The parenthesized combine-block (FOOP-93) avoids both.

## Open Questions

- **What counts as a branch having "established a position"** (the anchor-propagation hinges on it
  — likely "resolved its anchor far enough to produce a `FoolRefFir`, even if the final predicate
  missed").
- Does `|` compose with `&&`/`||` inside one `(...)` (mixed precedence)?
- Observable difference between `||` (per-candidate, first candidate matching either) and cascade
  `|` (whole search-1 first) — document it.

## Plan (lean)

- [ ] Add the `|` token; parse `|` inline (typically in the leading `(...)` block).
- [ ] `CascadingSearchFir` with anchor-propagation (thread the fallback `FoolRefFir`); reuse the
      contexted-resume path. `*_nyes_transitions` test. **Needs FOOP-43** (miss = fail signal).
- [ ] Worked-example unit tests for `A|B|C` under each failure pattern (the correctness core).
- [ ] Approval cases; comprehensive `foop_04_comprehensive.foo`.
- [ ] Worktree lifecycle per `foop.md`.

## Appendix — notes toward the full spec

- **Search FOOP B of three** (A = predicates FOOP-93, C = all-results FOOP-14). Separated from A
  because `|` is heavy and different-in-kind (stateful wrapper + anchor-propagation) vs the pure
  predicate work.
- **Depends on FOOP-43** — the cascade "fail" signal is a miss → ECONSTANIC. Composes with FOOP-93
  (`!` / `&&`/`||`) and FOOP-14 (`~~`/`??`).
- The anchor-propagation is the subtle correctness core; write the `A|B|C` worked examples as unit
  tests before implementing.

## References

- Prior: FOOP-23 (`FoolRefFir` position carrier; contexted resume), FOOP-43 (miss = fail signal),
  FOOP-93 (matcher `&&`/`||`, the `(...)` block, vocabulary).
- Code: `FoolRefFir` / `push_search_result_pair`; contexted-search resume path; `parser.rs:573`
  (postfix), `:797` (regex-group).
- Notes: `NOTES-creation-lineage-and-search-family.md` §9 + Engineering guidance.

## Last Updated

**Date**: 2026-07-09
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes**: New FOOP-04 = the cascading connector `|` (split out from old FOOP-24's beefy search
as Search FOOP B). `CascadingSearchFir` + anchor-propagation (resume from nearest earlier branch
that established a position, via `FoolRefFir`); needs FOOP-43.
