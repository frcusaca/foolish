---
foop: 36
title: Primitive Characterization — the i'/s'/f' type system
author: Atlas hc.busy@gmail.com
status: Draft
type: Standards
created: 2026-07-09
phase: phase-4
supersedes: []
begun: [ ]
---

# FOOP-63: Primitive Characterization — the `i'`/`s'`/`f'` type system

> Lean draft. Fuller notes in the Appendix and `NOTES-creation-lineage-and-search-family.md` §11.
> (Implementation order: #3. Renumbered 2026-07-09.)

## Abstract

Adopt **characterizations as primitive datatype tags**, turning FOOP-33's (stored-but-inert)
`Characterizations` into an active type system, and deliver the three primitive types + their
operators: **`i'` integer** (exists), **`s'` string** (new), **`f'` float** (new).
`get_value_primitive()` replaces the single hardcoded `as_i64()`, **dispatched by
characterization**. Characterization now has **two roles**: (1) a **type tag on a value** that
governs how its primitive is read and how operators treat it, and (2) a **search demand** — a
search can require a characterization, and a brane that cannot find a properly-characterized
parameter settles **WOCONSTANIC (waiting)**, not failed.

## Motivation

Foolish has one value type (integer) and characterizations that are searchable but semantically
inert. This FOOP makes characterizations *type tags the FVM dispatches on and demands*, and adds
the string and float primitives every non-trivial program needs. `CREATION.md` already sketches
`s = c'⬤`, `f = c'⬤` — string and float as characterizations.

## Specification

### What characterization DOES (the two roles — beyond search)

Characterization was, in FOOP-33, only *searchable*. This FOOP gives it operational meaning:

1. **Type tag on a value (the primitive-selection role).** A value's characterization selects its
   **primitive representation and accessor** via `get_value_primitive()`: `i'` → integer, `s'` →
   string, `f'` → float. This is *compiler/FVM-understood* semantics — the characterization tells
   the FVM which Rust primitive backs the value and which operations are legal. (Contrast with
   FOOP-73 boolean operators, which are boolean-*characterized objects* operated on by Foolish;
   here the primitive tags are read by the FVM itself.)
2. **Operator/verification role.** Operators consult the characterization to decide legality:
   arithmetic demands `i'`/`f'`; a mismatch is handled as a search miss (role 3), not a silent
   coercion.
3. **Search-demand role (the core tension).** A search can *demand* a characterization; see below.

So characterization is no longer inert metadata — it is the **type** of a value, read by
`get_value_primitive()` and by operators, and demandable by searches.

### `get_value_primitive()` — characterization-dispatched value access

Generalizes the hardcoded `as_i64()` (`fir_trait.rs:114`, `fir_kinds.rs:379`): dispatched by
characterization → integer / string / float primitive. Every current `as_i64()` call site (the
value-search matcher `fir_kinds.rs:1738/1739/1767/1768`, arithmetic operands, etc.) becomes
characterization-aware.

### The core: characterization as a search demand (waits, doesn't die)

A candidate with the **wrong characterization is a MISS**, not a match. Not-finding a correctly-
characterized value → **ECONSTANIC** (may gain a value on recoordination, per FOOP-43), and
dependents → **WOCONSTANIC (waiting)**. Characterization is therefore a **`SearchPredicate`
dimension** ("find something characterized `i'`") — the type system *is* search-with-a-
characterization-predicate. **This depends on FOOP-43** (miss → wait is what makes a char-demand
*wait* instead of *die*).

### The three primitives + operators

`StringFir`/`FloatFir` (new, born `Independent`, parallel to `IndepIntFir`); float arithmetic
(`+ - * / %`, `**`); a minimal string operator set (equality first; concat/length TBD); and
characterization verification/demand on every operator.

## FIR Impact

- **New value FIRs `StringFir`, `FloatFir`** — born `Independent`. Per the engineering guidance,
  `constanic_clone_at`'s `Independent` branch returns the same `Rc` for free; do not add deep-copy
  arms. Each owes a `*_nyes_transitions` test.
- **`get_value_primitive()`** on the FIR trait (dispatched by characterization).
- **A characterization gate in the matcher** — likely `SearchPredicate::Char` (or a char-field on
  existing predicates), composing with Name/Value.

## UBC Step Impact

- **Lexer/parser/AST:** string and float literals (`Astn::StringLit`/`Astn::FloatLit`).
- **Value access:** route `as_i64()` call sites through `get_value_primitive()`.
- **Matcher:** a wrong-characterization candidate → `Reject` (miss) → FOOP-43 turns it into
  ECONSTANIC/WOCONSTANIC-wait.
- **Operators:** demand `i'`/`f'`; a char-mismatch is a miss (wait), not necessarily immediate NK.

## Test Plan

- Unit: `StringFir`/`FloatFir` `*_nyes_transitions`; `get_value_primitive` dispatch by `i'`/`s'`/
  `f'`; a search demanding `i'` that finds only `s'` → WOCONSTANIC-wait (not NK); float arithmetic;
  string equality.
- Approval: typed values and operators; `3 + "x"` (char-demand miss → wait/NK per FOOP-43); a
  brane that resolves only once a correctly-characterized value is coordinated in.

## Rejected Alternatives

### A. Characterization as a crude verify-then-NK gate

Check the type; if wrong, NK immediately. **Rejected** (Atlas): a wrong/absent correctly-typed
value is *not-found*, which per FOOP-43 is ECONSTANIC/WOCONSTANIC — the brane **waits**, it
doesn't die.

### B. Type tag lives only on the LHS Identifier

Read the type from the defining statement's `Identifier`. **Tentatively rejected**: the natural
model is the **value FIR carries its own primitive kind** (the FIR *is* the type), with
`i'`/`s'`/`f'` on the LHS as a verified *assertion*. To be confirmed (the KEY open question).

## Open Questions

- **Where the type tag lives** — value FIR vs LHS Identifier (governs the whole FOOP).
- Int/float coercion in mixed arithmetic (`1.5 + 2`) — coerce, or miss/NK?
- Minimal string operator set (literals + equality first, or concat/length too?).
- Does `default_equal` (FOOP-33) extend to string/float equality? Float equality — exact bits.
- Characterization-verification failure → NK, WOCONSTANIC-wait, or a new alarm kind?
- Interaction with null-characterization (FOOP-33 name constants) — `i'`/`s'`/`f'` are normal
  chars alongside the null slot.
- **What characterization means for a NON-primitive (a brane)** — does a `myType'{...}` brane
  carry a user-defined characterization the FVM ignores but searches can demand? (This is the
  general-type-system seed beyond the three primitives.)

## Plan (lean)

- [ ] Resolve the KEY question: type tag on the value FIR vs the LHS Identifier.
- [ ] String + float literals (lexer/parser/AST); `StringFir`/`FloatFir` (born Independent) +
      `*_nyes_transitions`.
- [ ] `get_value_primitive()` dispatched by characterization; route `as_i64()` call sites through it.
- [ ] Characterization gate in the matcher (a `SearchPredicate` dimension); char-mismatch = miss →
      WOCONSTANIC-wait (needs FOOP-43).
- [ ] Float + minimal string operators; characterization demand on operators.
- [ ] Document the two/three roles of characterization (type-tag / verification / search-demand).
- [ ] Approval `.foo` cases; comprehensive `foop_63_comprehensive.foo`.
- [ ] Worktree lifecycle per `foop.md`.

## Appendix — notes toward the full spec

- **Depends on FOOP-33** (characterizations / `Identifier` / creation `s=c'⬤`, `f=c'⬤`) and
  **FOOP-43** (miss → wait). Foundational for all future datatypes.
- **Unblocks:** boolean operators (FOOP-73 — non-boolean args become a char-demand wait), integer
  math (FOOP-83 — typed comparisons), and the search FOOPs' char-gate composition.
- The "search demands a characterization → brane waits" framing is the intellectual core; it
  unifies the type system with the search engine. Write it prominently in the full spec.
- Beyond the three primitives, characterization is the seed of a **general user type system**
  (brane characterizations as user types the FVM stores and searches demand) — scope that as a
  successor, but note the door here.

## References

- Prior: FOOP-33 (characterizations / `Identifier` / `default_equal`), FOOP-43 (miss → ECONSTANIC/
  WOCONSTANIC), `CREATION.md` (`s=c'⬤`, `f=c'⬤`, `0=f'⬤`).
- Code: `fir_trait.rs:114`/`fir_kinds.rs:379` (`as_i64`), `fir_kinds.rs:1738-1768` (value matcher),
  `IndepIntFir` (value-FIR model), `constanic_clone_at:180` (Independent clone).
- Notes: `NOTES-creation-lineage-and-search-family.md` §11 + Engineering guidance.

## Last Updated

**Date**: 2026-07-09
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes**: Renumbered from FOOP-44 to FOOP-63 (impl-order reorg). Fleshed out **what
characterization does beyond search** — the type-tag (primitive-selection) role and the
operator/verification role, alongside the search-demand core. Added the brane/user-type open
question. Depends on FOOP-33 + FOOP-43.
