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

Add the missing integer operators: **exponent `**`** and the four **comparisons `<` `>` `<=`
`>=`**. Comparisons **return boolean `True`/`False`** (the `system.foo` creations), so they
compose with the Foolish boolean operators (`(3<5) and (x<y)`) and depend on FOOP-33 + FOOP-73
(boolean operators). Exponent is self-contained integer arithmetic. (Note: `*` multiply and `%`
modulus are **already implemented** — not part of this FOOP.)

## Motivation

Foolish integers can `+ - * / %` but cannot be raised to a power or compared. Comparison in
particular is table-stakes (and the natural producer of booleans). This is a small, concrete
strengthening of the integer library.

## Specification

- **Exponent `**`** — `a ** b` = integer power. Self-contained (pure i64). Overflow / negative
  exponent → NK or alarm (open).
- **Comparisons `<` `>` `<=` `>=`** — `a < b` etc. evaluate to the created **`True`** or
  **`False`** (`system.foo`, by identity — same mechanism as FOOP-73). So comparisons **depend on
  FOOP-33 + FOOP-73** and order after them.
- **Already done (out of scope):** `+ - * / %` and unary `-` compute in `OperatorFir::combine`
  (`fir_kinds.rs:531-576`), with `/`/`%` div-by-zero → NK.
- **Note (FOOP-63 arithmetic model):** under FOOP-63, arithmetic operators are conceptually Foolish
  **operation-table branes** the FVM detects and shortcuts to native math (no coercion lattice).
  The existing integer `combine` arms *are* the integer table's shortcut; `**` here adds to that
  same integer path. Float arithmetic and the int/float typing live in FOOP-63. This FOOP only
  extends the integer shortcut (`**`) and adds comparisons — it does not build the table machinery.

## FIR Impact

No new FIR kind. New arms in `OperatorFir::combine` for `**` and the four comparisons.

## UBC Step Impact

- **`OperatorFir::combine`** (`fir_kinds.rs:531`): add `**` (i64 pow, guard overflow / negative
  exponent) and `<`/`>`/`<=`/`>=` (compare the two i64s → push the `system.foo` `True`/`False`
  creation by identity, like FOOP-73). Reuse the div-by-zero→NK shape (`fir_kinds.rs:557`) for
  guarded results.
- **Parser / lexer:**
  - **`<` collides with the SF-marker `<expr>`** (`parser.rs:938`, `<` … expect `>`) and `>` with
    SFF-close / concat-continuation (`parser.rs:375`). `a < b` (comparison) vs `<b>` (SF-marker)
    needs disambiguation (whitespace-sensitive? require spaces around comparison?).
  - **`<=` / `>=` tokens do not exist** (`token.rs` has `Lt`/`Gt`/`LtEqGt`=`<=>`/`LtLt`/`GtGt`,
    not `LtEq`/`GtEq`) — add them.
  - Add `**` token (or reuse `Mul Mul`) at arithmetic precedence.

## Test Plan

- Unit: `**` (incl. overflow / negative exponent); each comparison → `True`/`False` by identity.
- Approval: `2**8`; `3<5`→True, `5<3`→False, `5<=5`→True; `(3<5) and (2<1)`.
- NYES unchanged (operators already have their transitions).

## Rejected Alternatives

### A. Comparisons return `1`/`0` integers (self-contained)

`3<5`→`1`. **Rejected** (Atlas): comparisons ARE boolean; returning `1`/`0` wouldn't compose with
the boolean operators and would need a later bridge. Boolean return is correct despite the
FOOP-33/73 dependency.

### B. Ship nothing / defer exponent too

**Rejected**: exponent is trivial and self-contained; there's no reason to defer it. (It *may*
ship before the comparisons, which wait on FOOP-73.)

## Open Questions

- `**` overflow → NK, wrap, or alarm? Negative exponent → NK (integer result)?
- The `<`/`>` SF-marker disambiguation (whitespace-sensitive comparison? mandatory spaces?).
- Comparison of mismatched types once FOOP-63 (Primitive Characterization) lands (a characterization
  demand?).

## Plan (lean)

- [ ] `**` arm in `OperatorFir::combine` + token + parser; unit tests (overflow/neg-exp). *(May
      ship independently, before comparisons.)*
- [ ] Add `LtEq`/`GtEq` tokens; resolve `<`/`>` vs SF-marker in the parser.
- [ ] Comparison arms in `combine` → push `system.foo` `True`/`False` (needs FOOP-73).
- [ ] Approval `.foo` cases; comprehensive `foop_83_comprehensive.foo`.
- [ ] Worktree lifecycle per `foop.md`.

## Appendix — notes toward the full spec

- **Split is natural:** exponent now (self-contained); comparisons after FOOP-73 (booleans).
- **Depends on FOOP-33 + FOOP-73** for comparisons (True/False by identity).
- Once **FOOP-63** exists, operands become characterization-demanded (`i'`), and mixed-type
  comparison aligns with that FOOP's WOCONSTANIC-wait vs NK.
- The `<`/`>` vs SF-marker collision is the real parseability work — same family as `{*}`/brane
  and `|`/regex; decide the disambiguation rule early.

## References

- Prior: FOOP-73 (booleans by identity — comparisons return these), FOOP-33 (creation),
  FOOP-63 (typed operands, later).
- Code: `fir_kinds.rs:531-576` (`combine`; `+ - * / %` done, `:557` div-by-zero→NK); `token.rs`
  (`Lt`/`Gt`/`LtEqGt`); `parser.rs:938` (`<` SF-marker), `:375` (concat-continuation).
- Notes: `NOTES-creation-lineage-and-search-family.md` §10 + Engineering guidance.

## Last Updated

**Date**: 2026-07-09
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes**: Renumbered from FOOP-34 to FOOP-83 (impl-order reorg). Dependency retargeted to
FOOP-73 (boolean operators). Exponent `**` (self-contained) + comparisons `< > <= >=` returning
True/False; `*`/`%` already done; `<`/`>` SF-marker collision + missing `<=`/`>=` tokens.
