---
foop: 41
title: All-results (find-all) search — doubled operators
author: Atlas hc.busy@gmail.com
status: Draft
type: Standards
created: 2026-07-09
phase: phase-2
supersedes: []
begun: [ ]
---

# FOOP-14: All-results (find-all) search — doubled operators

> Lean draft. Fuller notes in the Appendix and `NOTES-creation-lineage-and-search-family.md` §5.
> (Implementation order: #8 — Search FOOP C of three. Renumbered 2026-07-09.)

## Abstract

**Doubling a search operator makes it find-all** — it collects **every** matching statement into
a brane, instead of returning the single first match. `a~~tmp.*` = all statements in `a` whose
name matches `tmp.*`, returned in a brane. The convention is general: `~~` (forward name), `??`
(backward name), and by extension the value forms and combinations with `&`, `!`, and the boolean
matcher operators. FOOP-23 explicitly reserved find-all and *designed the hook*: "a find-all
Matcher that collects instead of stopping runs over the same Navigator."

## Motivation

Foolish can find *one* match; it cannot collect *all* matches. Find-all is a routine need (list
everything named `tmp_*`, sum all values matching a pattern). It is cheap here because the tokens
are already lexed and the engine already anticipated a collect mode.

## Specification

- **Operators.** `~~` forward-name find-all, `??` backward-name find-all; extends to value forms
  (`~~=`/`??=`) and composes with `&` (contexted) and the predicate operators (`!`, `&&`/`||` —
  FOOP-93).
- **Result.** A **brane whose children are the matches, in scan order.** Each entry reuses the
  existing single-search result structure (a value + a `FoolRefFir` position, per
  `push_search_result_pair`), so every entry carries its position → a following `&`-search chains
  off any entry (FOOP-23 composition).
- **Empty result.** An empty brane (not NK) when nothing matches. (Confirm against FOOP-43.)

## FIR Impact

No new FIR kind — reuse `SearchFir` with a **find-all flag** (or a distinct `Astn` variant that
compiles to a find-all `SearchFir`). The result is an ordinary brane of result-pairs.

## UBC Step Impact

- **Parser.** `TildeTilde`/`QuestionQuestion`/`DotDot` are **already lexed** (`lexer.rs:140-156`,
  tokens in `token.rs`); the parser uses them only for `Display` today (`parser.rs:1075-1082`).
  Wire them into `parse_postfix_expr` (`parser.rs:573`) as find-all variants.
- **Collect-mode scan.** Add a scan alongside `contextful_search_scan` (`fir_kinds.rs:1973`):
  same Navigator, same Predicate, but on `Approve` **push into a results Vec and continue** rather
  than `return Found`. Assemble the collected statements (as result-pairs) into a result brane.

## Test Plan

- Unit: collect-mode scan returns all `Approve`s in scan order; empty → empty brane.
- Approval: `a~~tmp.*` → brane of `tmp_*`; `~~` vs `??` ordering; find-all + `&`; find-all + `!`
  (`a!~~tmp.*`); find-all over a `(=10||=4)` combined predicate.

## Rejected Alternatives

### A. Vintage `B??`/`B//` spellings

`search-semantics.md` used `??` (local find-all) and `//` (global). **Rejected** in favor of the
systematic doubling convention, consistent with `&`/`!` as modifiers. `//` also clashes with
division.

### B. A separate find-all engine

**Rejected**: FOOP-23's one-engine model already anticipated a collect-mode Matcher over the same
Navigator; a second engine would duplicate traversal.

## Open Questions

- `//` (forward find-all + parents) clashes with division — drop or respell.
- Does find-all cross into parent branes (the vintage "global" idea) or stay local?
- Result ordering guarantee (scan order — forward vs backward per operator).
- Empty result: empty brane vs NK (lean: empty brane).

## Plan (lean)

- [ ] Unit: collect-mode scan (order, empty).
- [ ] Wire `~~`/`??` into `parse_postfix_expr` (tokens already lexed).
- [ ] Add the collect-mode scan + result-brane assembly (reuse result-pair shape).
- [ ] Approval cases (incl. compose with `!` and `&&`/`||`); comprehensive `foop_14_comprehensive.foo`.
- [ ] Worktree lifecycle per `foop.md`.

## Appendix — notes toward the full spec

- **Search FOOP C of three** (A = predicates FOOP-93, B = cascading `|` FOOP-04). A scan-mode
  change, orthogonal to the predicate/control-flow work.
- Composes with **FOOP-93** (`a!~~x`, find-all over a combined predicate) and depends conceptually
  on **FOOP-43** for miss/empty semantics.
- Result-entry shape decision (value vs statement vs `FoolRefFir`-bearing) is the main design
  call; leaning on the existing `push_search_result_pair` pair per match.
- Engineering guidance: canonical "scan-mode change, not a new engine" case.

## References

- Prior: FOOP-23 (reserved find-all; one-engine model; FOOP-23.md:600 collect-instead-of-stop),
  FOOP-43 (empty/miss), FOOP-93 (predicate composition).
- Code: `lexer.rs:140-156` (doubled tokens), `parser.rs:573` (postfix), `fir_kinds.rs:1973`
  (scan loop), `push_search_result_pair` (result shape).
- Notes: `NOTES-creation-lineage-and-search-family.md` §5 + Engineering guidance.

## Last Updated

**Date**: 2026-07-09
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes**: Renumbered from FOOP-73 to FOOP-14 (impl-order reorg; Search FOOP C of three).
Doubled search operators (`~~`/`??`) collect all matches into a brane; tokens already lexed;
collect-mode scan reusing the one-engine model.
