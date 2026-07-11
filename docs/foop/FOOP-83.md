---
foop: 38
title: Strengthen integer math — exponent and comparisons
author: Atlas hc.busy@gmail.com
status: Draft
type: Standards
created: 2026-07-09
phase: phase-2
supersedes: []
begun: [ ]
---

# FOOP-83: Strengthen integer math — exponent and comparisons

> Lean draft. Fuller notes in the Appendix and `NOTES-creation-lineage-and-search-family.md` §10.
> (Implementation order: #5. Renumbered 2026-07-09.)

## Abstract

Add the missing integer operators: **exponent** and the four **comparisons** (less-than,
greater-than, and their or-equal forms). Following the unified pattern (booleans FOOP-73,
arithmetic FOOP-63), comparisons are **created lookup-table branes** — e.g. `i'lessthan =
{A=1,B=2,result=True; A=2,B=3,result=True; …}` (a countably-infinite table), looked up by search
`i'lessthan~A=1~B=2#1` — but **implemented in Rust** (the FVM detects the table brane and
shortcuts). They return the boolean `True`/`False` (FOOP-73). **Named with typed-out / snake_case
names** (`lessthan`/`greaterthan`, impls `less_than`/`greater_than`); the `<`/`>`/`<=`/`>=`
**syntactic sugar is deferred** — which sidesteps the `<`/`>` vs SF-marker collision entirely for
now. (Note: `* / %` are already implemented — not part of this FOOP.)

## Motivation

Foolish integers can `+ - * / %` but cannot be raised to a power or compared. Comparison in
particular is table-stakes (and the natural producer of booleans). Using **typed-out names** now
(no `<`/`>` tokens) avoids the SF-marker parse collision and lets sugar be added later once its
disambiguation is designed.

## Specification

Everything here follows the **created-table-brane / FVM-shortcut** pattern shared with FOOP-73
(booleans) and FOOP-63 (arithmetic): declared via the Creation Postulate as a Foolish lookup-table
brane, dispatched by identity, computed natively in `OperatorFir` (`fir_kinds.rs:483`).

- **Exponent** — a `power` (`i'power`) table brane; `a power b` = integer power. Self-contained
  (pure i64). Overflow / negative exponent → NK or alarm (open).
- **Comparisons** — `i'lessthan`, `i'greaterthan`, `i'lessorequal`, `i'greaterorequal` table
  branes. `A lessthan B` looks up `i'lessthan~A=…~B=…#1` → the created **`True`**/**`False`**
  (FOOP-73). So comparisons **depend on FOOP-33 + FOOP-73**. FIR impls use snake_case
  (`less_than`, `greater_than`, `less_or_equal`, `greater_or_equal`).
- **Names now, sugar later.** No `<`/`>`/`<=`/`>=` tokens in this FOOP. A follow-on adds the
  operator sugar once the `<`/`>` vs SF-marker (`<expr>`) collision is resolved (whitespace rule,
  or a different glyph).
- **Already done (out of scope):** `+ - * / %` and unary `-` compute in `OperatorFir::combine`
  (`fir_kinds.rs:531-576`), with `/`/`%` div-by-zero → NK. These *are* the integer table shortcuts.

## FIR Impact

No new FIR kind. New arms in `OperatorFir::combine` for `power` and the four comparison operators,
each dispatched by recognizing the corresponding `system.foo` table-brane creation (like FOOP-73).
The comparison arms return the `system.foo` `True`/`False` by identity.

## UBC Step Impact

- **`system.foo`** declares the table branes: `i'power`, `i'lessthan`, `i'greaterthan`,
  `i'lessorequal`, `i'greaterorequal` (created lookup tables; never enumerated).
- **`OperatorFir::combine`** (`fir_kinds.rs:531`): when applied to one of these table creations,
  shortcut to native Rust: `power` (i64 pow, guard overflow / negative exponent → NK via the
  div-by-zero→NK shape at `fir_kinds.rs:557`); the comparisons compare the two i64s → push
  `system.foo` `True`/`False` by identity.
- **No lexer/parser work.** Named operators go through the existing identifier + application
  (RPN/table-lookup) path — **no `<`/`>`/`<=`/`>=`/`**` tokens are added.** The `<`/`>` vs
  SF-marker collision is thereby avoided; operator sugar is a separate follow-on FOOP.

## Test Plan

- Unit: `power` (incl. overflow / negative exponent → NK); each comparison → `True`/`False` by
  identity; the table-brane dispatch (is-this-`i'lessthan`?).
- Approval: `2 power 8`; `3 lessthan 5`→True, `5 lessthan 3`→False, `5 lessorequal 5`→True;
  `(3 lessthan 5) and (2 lessthan 1)` (composing with FOOP-73 booleans).
- NYES unchanged (operators already have their transitions).

## Rejected Alternatives

### A. Comparisons return `1`/`0` integers (self-contained)

`3 lessthan 5`→`1`. **Rejected** (Atlas): comparisons ARE boolean; returning `1`/`0` wouldn't
compose with the boolean operators and would need a later bridge. Boolean return is correct despite
the FOOP-33/73 dependency.

### B. Ship the `<`/`>` operator sugar now

Add `Lt`/`Gt`/`LtEq`/`GtEq` tokens and parse `a < b`. **Deferred, not rejected**: `<`/`>` collide
with the SF-marker `<expr>` (`parser.rs:938`) and SFF-close/concat-continuation (`parser.rs:375`),
needing a disambiguation (whitespace rule, or a different glyph). Ship the typed-out names now; add
sugar in a follow-on once the collision is resolved.

## Open Questions

- `power` overflow → NK, wrap, or alarm? Negative exponent → NK (integer result)?
- Exact operator names (`lessthan`/`greaterthan`/`lessorequal`/`greaterorequal`? or `lt`/`gt`/…?).
- Comparison of mismatched types once FOOP-63 lands — a characterization demand (`i'` operands).
- (Deferred to the sugar follow-on: the `<`/`>` vs SF-marker disambiguation.)

## Plan (lean)

- [ ] Declare `i'power` + the four comparison table branes in `system.foo` (needs FOOP-33; the
      comparisons need FOOP-73 `True`/`False`).
- [ ] `power` shortcut arm in `OperatorFir::combine` (i64 pow, overflow/neg-exp → NK). *(May ship
      independently — integer-only, no boolean dep.)*
- [ ] Comparison shortcut arms → push `system.foo` `True`/`False` by identity (needs FOOP-73). FIR
      impls snake_case (`less_than`, etc.).
- [ ] Unit + approval `.foo` cases (named operators, no `<`/`>` tokens); comprehensive
      `foop_83_comprehensive.foo`.
- [ ] Worktree lifecycle per `foop.md`.

## Appendix — notes toward the full spec

- **The unified pattern.** Booleans (FOOP-73), arithmetic (FOOP-63), and comparisons (this FOOP)
  are all **created lookup-table branes** — `i'lessthan = {A=1,B=2,result=True; …}` (countably
  infinite), searched `i'lessthan~A=1~B=2#1` — **implemented in Rust** (FVM detects the table brane
  and shortcuts). One mechanism across the whole numeric/logic layer; no privileged layer at the
  *declaration* level.
- **Split is natural:** `power` now (integer-only, no boolean dep); comparisons after FOOP-73.
- **Depends on FOOP-33 + FOOP-73** for comparisons (True/False by identity).
- Once **FOOP-63** exists, operands are characterization-demanded (`i'`), and mixed-type comparison
  aligns with FOOP-63's WOCONSTANIC-wait vs NK.
- **Sugar deferred:** the `<`/`>`/`<=`/`>=` operator forms are a follow-on FOOP once the `<`/`>` vs
  SF-marker collision is resolved (same family as `{*}`/brane, `|`/regex).

## References

- Prior: FOOP-73 (booleans by identity — comparisons return these), FOOP-33 (creation),
  FOOP-63 (typed operands, later).
- Code: `fir_kinds.rs:531-576` (`combine`; `+ - * / %` done, `:557` div-by-zero→NK); `token.rs`
  (`Lt`/`Gt`/`LtEqGt`); `parser.rs:938` (`<` SF-marker), `:375` (concat-continuation).
- Notes: `NOTES-creation-lineage-and-search-family.md` §10 + Engineering guidance.

## Last Updated

**Date**: 2026-07-10
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes (Atlas)**: **Typed-out operator names, sugar deferred.** Exponent and comparisons are now
**created lookup-table branes** (`i'power`, `i'lessthan`, `i'greaterthan`, `i'lessorequal`,
`i'greaterorequal` — countably infinite, searched `i'lessthan~A=1~B=2#1`) **implemented in Rust**
(FVM detects the table brane in `OperatorFir` and shortcuts). Named operators (`lessthan`/…, FIR
impls snake_case) — **no `<`/`>`/`<=`/`>=`/`**` tokens**, which sidesteps the `<`/`>` vs SF-marker
collision entirely; operator sugar deferred to a follow-on. Unifies with FOOP-73 (booleans) and
FOOP-63 (arithmetic) — all created-table-brane / FVM-shortcut.

**Date**: 2026-07-09
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes**: Renumbered from FOOP-34 to FOOP-83 (impl-order reorg). Dependency retargeted to
FOOP-73 (boolean operators). Exponent `**` (self-contained) + comparisons `< > <= >=` returning
True/False; `*`/`%` already done; `<`/`>` SF-marker collision + missing `<=`/`>=` tokens.
