---
foop: 39
title: Search predicates — inverse matcher and matcher boolean operators
author: Atlas hc.busy@gmail.com
status: Draft
type: Standards
created: 2026-07-09
phase: phase-2
supersedes: []
begun: [ ]
---

# FOOP-93: Search predicates — inverse matcher `!` and matcher boolean operators `&&`/`||`

> Lean draft. Fuller notes in the Appendix and `NOTES-creation-lineage-and-search-family.md` §4/§9.
> (Implementation order: #6 — Search FOOP A of three. Renumbered 2026-07-09.)

## Abstract

The **predicate** layer of the search family — all done at one locus (`SearchPredicate::matches`):

- **`!` — inverse matcher.** Negates a search's predicate; matches exactly what the un-negated
  search rejects. Per-gate in combined forms. Placement: after `&`, before the op.
- **`&&` / `||` — matcher boolean operators.** Boolean ops on **matcher Approve/Reject results** —
  a combining matcher. `(=10 || =4)~a.*` = "name starts with `a` **and** (value 10 **or** 4)."
  (`NkStop` in either branch halts the scan.)
- **Value-pattern-as-expression** — the value side of a value search may be any expression,
  **including a search**, evaluated to constanic first (`_TABLE~A=A`). Implemented/tested here
  because the self-hosting operators (FOOP-73/63/83) depend on it.

The first two are **compiler-hard-coded matcher semantics** (they operate on match *outcomes*, not
on Foolish values — distinct from the Foolish boolean operators in FOOP-73). They compose and share
the matcher locus, so they belong in one FOOP.

## Motivation

Foolish searches gate on a single predicate (or the atomic `~name=value` conjunction). Real
queries want "not matching X," "value 10 or 4," and "name A and value B." `!` adds negation;
`&&`/`||` add boolean combination — both extend the *predicate*, cheaply, at the same place.

## Specification

### Inverse matcher `!`

- **Placement.** After an optional `&`, before the search operator: `a&!?x`, `b!~a.*`, `!=5`.
- **Semantics.** Invert the predicate outcome: `Approve`↔`Reject`; `NkStop` stays `NkStop` (an NK
  candidate is *incomparable*, not "matched").
- **Per-gate.** In `~name=value`, a `!` may sit on the name gate and independently on the value
  gate: `b!~a.*!=5` negates both.

### Matcher boolean operators `&&` / `||`

- Combine matcher results into a composite matcher tested per candidate: `And` = Approve iff both
  branches Approve; `Or` = Approve iff either. Generalizes the atomic `NameValue`
  (`fir_kinds.rs:1745`).
- **`NkStop` wins over everything, in both `And` and `Or` (Atlas).** If *either* branch returns
  `NkStop` on a candidate, the composite returns `NkStop` — halting the scan with NK — regardless
  of the other branch (even if the other branch `Approve`s). NK is contagious in the matcher, the
  same way NKs never compare equal (FOOP-33). So `(<NK-valued> || =4)` on an NK candidate halts,
  it does not fall through to the `4` branch. The three-valued truth table:

  | `And`/`Or` | any branch `NkStop` | else `And`(∧) | else `Or`(∨) |
  |------------|---------------------|---------------|--------------|
  | result     | **`NkStop`**        | Approve iff both Approve | Approve iff either Approves |

- **Bare leading value predicate `=N`** — today value search is only `~=`/`?=`; `(=10||=4)` uses
  bare `=10`. New.
- **Placement / parseability.** The combine-block `(...)` **leads** (anchor/expression position):
  `(=10||=4)~a.*`. A *leading* `(...)` is in expression position, so it does **not** hit the
  regex-group path (`parser.rs:797`, where `(` inside a *pattern* is a regex group).

### Value-pattern is an expression (may itself be a search) — enhancement (Atlas)

The value side of a value search is already an **expression** (`value_pattern := arith_expr`,
`parser.rs:410`) — so `a~=1+2` compares against the arithmetic result. This FOOP makes that
uniform: **the value-pattern may be any expression, including a search**, and it is **evaluated to
constanic first, then compared**. So `_TABLE~A=A` means "find a candidate whose value equals *the
value that searching for `A` yields*" — the RHS `A` is an identifier → a `SearchFir` (FOOP-4) that
resolves to the bound arg, and its settled value is the comparison target.

- The matcher already settles the value-child before comparing (`value_child`, `build_value_predicate`
  require constanic — `fir_kinds.rs:967-978`). This enhancement confirms and *tests* that a search
  in the value-pattern resolves the same way an arithmetic expression does.
- **This FOOP implements/tests it first** — the boolean/arithmetic operators (FOOP-73/63/83) depend
  on it for their self-hosting table lookups (`_TABLE~A=A&~B=B#1`). Land it here.
- If the value-pattern's own search settles ECONSTANIC/NK (FOOP-43), the enclosing value search
  waits/NKs accordingly (it can't compare against an unresolved value).

## FIR Impact

No new FIR kind. Extend `SearchPredicate` (`fir_kinds.rs:1679`):
- **Combination:** `And(Box<_>, Box<_>)` / `Or(Box<_>, Box<_>)` — the **recursive tree** shape.
- **Negation:** a `negate: bool` on the leaf variants, or (for `NameValue`) `negate_name` /
  `negate_value` for per-gate control.
`Astn`'s search variants gain the negated flag; the combine-block is a new leading-`(...)` parse.

> **Co-design note (coherence review):** this FOOP **defines the `SearchPredicate` tree shape**
> (`And`/`Or` outer, `negate` on leaves). FOOP-63 (Primitive Characterization) later adds a `Char`
> predicate — it must slot in as **another leaf** of *this* tree, **not** a parallel combination
> mechanism. Land this FOOP's tree first. See the coherence review in the NOTES doc.

## UBC Step Impact

- **`SearchPredicate::matches`** (`fir_kinds.rs:1709`): for a negated gate, swap `Approve`/`Reject`
  (leave `NkStop`); for `And`/`Or`, combine branch outcomes — **`NkStop` in either branch → `NkStop`
  (halt)**; else `And`=∧, `Or`=∨ on Approve/Reject.
- **Value-pattern evaluation.** Ensure the value-child (which may be a search) is settled to
  constanic before comparison (already required at `fir_kinds.rs:967-978`); add tests that a search
  in the value-pattern resolves like an arithmetic expression.
- **Parser** (`parse_postfix_expr`, `parser.rs:573`): consume `!` after optional `&` before the
  op; parse the leading `(...)` combine-block and bare `=N`.
- **Lexer:** a `Bang` token for a single `!` **not** part of `!!`/`!!!`/`#!` (comment markers,
  `lexer.rs:100`); new tokens `&&`/`||`.

## Test Plan

- Unit: negated `SearchPredicate` outcomes (Approve↔Reject; NkStop unchanged; per-gate
  `NameValue`); `And`/`Or` truth table with **`NkStop`-halts** (`(NK||Approve)` → NkStop); bare `=N`.
- **Value-pattern-as-search:** `_TABLE~A=A` where `A` is a bound value resolves and compares;
  value-pattern search that settles ECONSTANIC/NK propagates.
- Lexer/parser: `!~`, `&!~`, `!=`, `!` placement; `!!` still a comment; the leading `(...)`.
- Approval: `b!~a.*`; `b!~a.*!=5`; `(=10||=4)~a.*`; `(=10 && ~a.*)`; `!` composed with `&&`/`||`.

## Rejected Alternatives

### A. A `Not(Box<SearchPredicate>)` wrapper

**Rejected**: flatter to add `negate` flags, and the combined form needs *per-gate* negation which
two bools express directly; a wrapper can't negate one gate of a `NameValue`.

### B. Do the boolean combination as Foolish (like FOOP-73 operators)

Express `&&`/`||` via a Foolish truth table. **Rejected**: these operate on *matcher outcomes*
inside the engine, not on Foolish values — they must be compiler-hard-coded. (This is the
two-booleans distinction.)

## Open Questions

- **RESOLVED:** `NkStop` in either `And`/`Or` branch → halt with NK (Atlas). Value-pattern may be a
  search, evaluated to constanic first (implemented/tested here).
- Negated anchored-search miss — defer to FOOP-43's rules.
- `!` before a positional/head-tail op (`!#3`, `!^`) — meaningful or error?
- Does the combine-block appear only leading, or also mid-chain?
- Bare `=N` — anchored or unanchored default?

## Plan (lean)

- [ ] Add `Bang`/`&&`/`||` tokens; lex a lone `!` distinct from `!!`.
- [ ] Thread a `negated` flag through the `Astn` search variants + `parse_postfix_expr`; parse the
      leading `(...)` combine-block + bare `=N`.
- [ ] Extend `SearchPredicate` with negation flags + `And`/`Or`; update `matches` (**`NkStop`
      halts** in both `And`/`Or`).
- [ ] **Value-pattern-as-expression:** ensure a value-pattern that is a search settles to constanic
      before comparison; unit-test it resolves like arithmetic. (FOOP-73/63/83 depend on this.)
- [ ] Unit + approval cases; comprehensive `foop_93_comprehensive.foo`.
- [ ] Worktree lifecycle per `foop.md`.

## Appendix — notes toward the full spec

- **Search FOOP A of three:** A = predicates (`!` + `&&`/`||`, this FOOP), B = cascading connector
  `|` (FOOP-04), C = all-results `~~`/`??` (FOOP-14). Split by *kind*: predicate combination vs
  control-flow cascade vs scan-mode.
- Shares the prefilter locus with **detachment** (FOOP-24) — both act at
  `SearchPredicate::matches` / the scan loop; do this first, detachment reuses the seam.
- Composes with **FOOP-14** (`a!~~x` = all names not matching) and enables the boolean-table
  lookups of **FOOP-73** (value + contexted search).
- Engineering guidance: canonical "extend a predicate, don't add a FIR" case.

## References

- Prior: FOOP-23 (combined `~name=value`; `NameValue`; the one-engine model), FOOP-43 (miss
  outcome), FOOP-73 (the *other* booleans — Foolish objects).
- Code: `fir_kinds.rs:1679` (`SearchPredicate`), `:1745` (`NameValue`), `:1709` (`matches`);
  `parser.rs:573` (postfix), `:797` (regex-group in pattern); `lexer.rs:100` (`!!` comment).
- Notes: `NOTES-creation-lineage-and-search-family.md` §4/§9 + Engineering guidance.

## Last Updated

**Date**: 2026-07-09
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes**: New FOOP-93 = Search predicates (merged inverse matcher `!` from old FOOP-53 + the
matcher boolean operators `&&`/`||` from old FOOP-24). Both are compiler-hard-coded matcher-outcome
operators (distinct from FOOP-73's Foolish booleans), sharing the `SearchPredicate::matches` locus.
