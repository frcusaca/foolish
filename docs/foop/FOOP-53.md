---
foop: 35
title: Computed index — #${...}
author: Atlas hc.busy@gmail.com
status: Draft
type: Standards
created: 2026-07-09
phase: phase-2
supersedes: []
begun: [ ]
---

# FOOP-53: Computed index — `#${...}`

> **Roadmap note (2026-07-14, Track 2 #4, small):** after FOOP-93. Reuses the value-pattern
> machinery FOOP-23 shipped: the `#${…}` computed child steps to constanic exactly like a
> `?=`/`~=` pattern child, then feeds `SearchPredicate::Index`. Out-of-range on a settled
> brane stays NK (a preserved provable-impossibility under 43-C1).

> Lean draft. Fuller notes in the Appendix and `NOTES-creation-lineage-and-search-family.md` §8.
> (Implementation order: #2 — a small self-contained early win. Renumbered 2026-07-09.)

## Abstract

Permit an **expression** in the indexer instead of a literal `#N`. Syntax `#${...}`: the FIR
**evaluates the brane after `$`**, **retrieves its last element** (tail), **expects a number**,
then **searches `#` with that number as the offset** — as usual. Example: `x#${a; b; 3}` →
evaluate `{a;b;3}`, take the tail (`3`), do `x#3`.

## Motivation

The index offset is fixed at parse time today (`IndexFir.offset: i32`), so you cannot index by a
computed value. Computed indexing is routine (index by a counter, by a search result). This adds
it by reusing the tail-retrieval and `as_i64` machinery Foolish already has — a small, low-risk
early win.

## Specification

- **Syntax.** `<anchor>#${<brane>}` — the `#` index operator, then `$`, then a brane.
- **Evaluation.** Evaluate the brane; take its **tail element**; read it as a number
  (`get_value_primitive`/`as_i64`); use that as the `#` offset; run the ordinary index search on
  the anchor.
- **Ordering.** The index brane must **settle before** the offset can be read — so the computed
  index has a genuine wait phase (settle child → extract number → run index).

## FIR Impact

Either a new `DynamicIndexFir` or an extension of `IndexFir` (`fir_kinds.rs:1313`, today
`offset: i32`) with an **optional computed-offset child**. Per the engineering guidance, a thin
new kind is warranted if the Braning-wait phase is cleaner than overloading `IndexFir`. Owes a
`*_nyes_transitions` test.

## UBC Step Impact

- **Parser.** In the `#` postfix path (`parser.rs:643`) and `parse_seek_index` (`parser.rs:894`,
  today parses only a literal integer), branch on the next token — if `$`, parse `${brane}` and
  emit the computed-index AST variant. The `$`/Dollar token exists (`token.rs:20`).
- **Step.** New Braning phase: push the index brane as a task; on settle, take its tail (reuse
  HeadTail/`$` retrieval), read `as_i64` (`fir_kinds.rs:379`), set the offset, run the ordinary
  `SearchPredicate::Index(offset)` scan (unchanged engine).

## Test Plan

- Unit: tail extraction + offset wiring; the Braning-wait progression (`*_nyes_transitions`); tail
  not-a-number path.
- Approval: `x#${a;b;3}` == `x#3`; a computed negative offset; tail is `???` → (NK? — open); the
  index brane as an expression (`x#${1+2}` if arithmetic-in-tail is allowed).

## Rejected Alternatives

### A. Only literal `#N`

The status quo. **Rejected** — no way to index by a computed value; a common need.

### B. `#$expr` without braces

Allow a bare expression after `#$`. **Considered**; the brace form `#${...}` is unambiguous and
matches the "evaluate a brane, take its tail" model. Bare form is an open question.

## Open Questions

- Tail not a number → NK or alarm?
- Negative computed offsets — allowed (like literal `#-N`)?
- Is the brace mandatory (`#${...}`) or is `#$expr` also allowed?
- "Last element" = the tail statement's settled *value*, or the raw tail element? (Assume the
  tail statement's value via `as_i64`.)

## Plan (lean)

- [ ] Unit tests: tail extraction, wait progression, non-number tail.
- [ ] Parser: branch `#` → `$` → `${brane}` → computed-index AST.
- [ ] FIR + step: settle child → tail → `as_i64` → run index; `*_nyes_transitions`.
- [ ] Approval `.foo` cases; comprehensive `foop_53_comprehensive.foo`.
- [ ] Worktree lifecycle per `foop.md`.

## Appendix — notes toward the full spec

- **Self-contained** — reuses `$`/tail, `as_i64`, the Index engine; independent of all other FOOPs.
  Placed early (impl #2) precisely because it's a small standalone win.
- Once **Primitive Characterization** (FOOP-63) exists, "expects a number" becomes a
  characterization demand (`i'`) — align the not-a-number path with that FOOP's WOCONSTANIC-wait
  vs NK then.

## References

- Prior: FOOP-23 (index/seek search), FOOP-63 (Primitive Characterization — number-demand, later).
- Code: `fir_kinds.rs:1313` (`IndexFir.offset`), `:379` (`as_i64`); `parser.rs:643`/`:894` (index
  parse), `token.rs:20` (`Dollar`); HeadTail/`Astn::Tail` (tail retrieval).
- Notes: `NOTES-creation-lineage-and-search-family.md` §8 + Engineering guidance.

## Last Updated

**Date**: 2026-07-09
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes**: Renumbered from FOOP-14 to FOOP-53 (impl-order reorg: computed index moved to #2).
`#${...}` computed index: evaluate the brane, take its tail as a number, run `#` with that offset.
