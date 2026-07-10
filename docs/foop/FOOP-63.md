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
operators: **`i'` integer** (exists), **`s'` string** (new), **`f'` float** (new). The type tag
lives on the **LHS characterization** and typing is **enforced through characterized search**: a
search asking for `~b'x` matches only a candidate whose LHS carries `b'`; a plain `x` does not
qualify → the search settles **ECONSTANIC/WOCONSTANIC (waiting)**, not failed. Typed value
accessors (`get_value_int`/`_float`/`_string`) read a value's backing primitive. Arithmetic is
"faked" via operation-table branes the FVM detects and shortcuts to native math (like FOOP-73
booleans).

## Motivation

Foolish has one value type (integer) and characterizations that are searchable but semantically
inert. This FOOP makes characterizations *type tags the FVM dispatches on and demands*, and adds
the string and float primitives every non-trivial program needs. `CREATION.md` already sketches
`s = c'⬤`, `f = c'⬤` — string and float as characterizations.

## Specification

### The model — characterization is a SEARCH-LEVEL demand, not a value-carried tag (Atlas)

The type tag lives where FOOP-33 already put it — on the **LHS characterization** of a statement —
and typing is **enforced through search matching**, not by a kind stored on the value FIR.

- **The LHS characterization does two things:** it **declares** "this thing is characterized `b'`,
  find it here," and (by symmetry) a *searcher* asking for `b'` **demands** that characterization.
- **A characterized search matches only characterized candidates.** `~b'x` looks in the candidate
  list for a candidate whose **LHS is labeled `b'x`**. A candidate named `x` *without* the `b'`
  characterization **does not match** — so the search finds nothing and (per FOOP-43) settles
  **ECONSTANIC** (may recoordinate to a properly-characterized `x` later). This is the whole typing
  mechanism: the matcher's characterization gate (the `Char` predicate) *is* the type check.
- **First pass (this FOOP):** `~b'x` = "find an `x` whose LHS carries `b'`; plain `x` does not
  qualify." Only LHS characterization is consulted.
- **Deferred extension:** if the LHS lacks the characterization, *also* check the same search path
  for the **RHS** carrying the `b'` characterization (a value that is itself `b'`-typed). The exact
  rule needs thinking-through — **out of scope for the first pass**; noted in Open Questions.

So characterization is not a kind field on the value FIR; it is **metadata on the LHS that the
search matcher gates on**. the typed accessors (`get_value_int`/`_float`/`_string`, below) still read a *primitive's* backing
representation, but *which values are eligible for a typed operation* is decided by
characterized search, not by inspecting the value.

> **@human — coordination interaction (to think through).** Because coordination strips search
> context (FOOP-43 Component 2) and the marker on constanic clone (FOOP-24), we must decide what a
> *coordinated* characterized value looks like: does the characterization travel with the
> coordinated value, or is it (like position) shed? Since the tag is LHS metadata read by search
> — not value-carried — a coordinated *value* is plausibly characterization-free, and re-demanding
> `b'` at the new site re-checks the new LHS. Confirm during implementation.

### Typed value accessors — `get_value_int()` / `get_value_float()` / `get_value_string()`

"`get_value_primitive()`" is shorthand for a **family of typed accessors** (Atlas), one per
primitive — **not** a single polymorphic method:

```rust
fn get_value_int(&self)    -> Option<i64>;     // generalizes today's as_i64()
fn get_value_float(&self)  -> Option<f64>;
fn get_value_string(&self) -> Option<&str>;
```

Each returns `Some` iff the value is that primitive, else `None`. This generalizes the single
hardcoded `as_i64()` (`fir_trait.rs:114`, `fir_kinds.rs:379`). Every current `as_i64()` call site
(the value-search matcher `fir_kinds.rs:1738/1739/1767/1768`, arithmetic operands, etc.) calls the
appropriate typed accessor. (A caller that needs "whichever primitive this is" dispatches on the
FIR kind / characterization and calls the matching accessor — the family, not one method, is the
interface.)

### The core: characterization as a search demand (waits, doesn't die)

A candidate with the **wrong characterization is a MISS**, not a match. Not-finding a correctly-
characterized value → **ECONSTANIC** (may gain a value on recoordination, per FOOP-43), and
dependents → **WOCONSTANIC (waiting)**. Characterization is therefore a **`SearchPredicate`
dimension** ("find something characterized `i'`") — the type system *is* search-with-a-
characterization-predicate. **This depends on FOOP-43** (miss → wait is what makes a char-demand
*wait* instead of *die*).

### The three primitives + operators

`StringFir`/`FloatFir` (new, born `Independent`, parallel to `IndepIntFir`); a minimal string
operator set (equality first; concat/length TBD); and characterization verification/demand on every
operator.

### Arithmetic — "fake it" with operation tables, detect them in `Op+` (Atlas)

Rather than build a coercion lattice, arithmetic follows the **same declared-in-Foolish /
computed-by-FVM pattern as the booleans (FOOP-73)**. Define the arithmetic operators as **branes
holding operation tables** — conceptually a real-addition table and an integer-addition table
(cardinality ℝ² and ℂ² respectively, i.e. "infinite" tables you never actually enumerate) — and the
Foolish-level meaning of `a + b` is a **search over the appropriate table**. In practice the FVM
**detects use of these specific table branes inside `OperatorFir` (`Op+`, `fir_kinds.rs:483`) and
substitutes native IEEE-float / integer arithmetic** — it never enumerates the table.

- So `+` (and the other arithmetic ops) are Foolish objects (declared as table branes in
  `system.foo`), dispatched by identity (like the boolean operators), and the FVM shortcuts the
  table search to a Rust op. No implicit-coercion lattice: the *choice of table* (int vs real)
  encodes the type semantics, and a mismatch is a characterization-demand miss, not a coercion.
- The genuine "upgrade" is the search logic that *would* look a result up in the table; the
  practical implementation is the detection-and-shortcut in `Op+`. This keeps "no privileged layer"
  at the declaration level while the FVM does the real math.
- `float arithmetic (+ - * / %, **)` and integer arithmetic are the two backed table-families for
  this FOOP; the existing integer `+ - * / %` (already in `combine`) is the integer table's shortcut.

## FIR Impact

- **New value FIRs `StringFir`, `FloatFir`** — born `Independent`. Per the engineering guidance,
  `constanic_clone_at`'s `Independent` branch returns the same `Rc` for free; do not add deep-copy
  arms. Each owes a `*_nyes_transitions` test.
- **Typed value accessors** `get_value_int`/`get_value_float`/`get_value_string` on the FIR trait (one per primitive; each `Option<T>`).
- **A characterization gate in the matcher** — a `SearchPredicate::Char { … }` leaf.
  > **Co-design note (coherence review):** FOOP-93 defines the `SearchPredicate` **tree**
  > (`And`/`Or` + `negate`). Add `Char` as **another leaf of that tree** — do **not** invent a
  > parallel combination mechanism. A char-demand then composes for free: `(i' && =10)` etc. Land
  > FOOP-93 first. See the coherence review in the NOTES doc.

## UBC Step Impact

- **Lexer/parser/AST:** string and float literals (`Astn::StringLit`/`Astn::FloatLit`).
- **Value access:** route `as_i64()` call sites through the matching typed accessor.
- **Matcher:** a wrong-characterization candidate → `Reject` (miss) → FOOP-43 turns it into
  ECONSTANIC/WOCONSTANIC-wait. When such a demand settles ECONSTANIC, set
  `EconstanicReason::CharDemand` (adds that variant to FOOP-43's enum, Component 3).
- **Operators:** demand `i'`/`f'`; a char-mismatch is a miss (wait), not necessarily immediate NK.

## Test Plan

- Unit: `StringFir`/`FloatFir` `*_nyes_transitions`; typed accessors return `Some` only for their
  type (`get_value_int` on an `s'` value → `None`); characterized-search matching by `i'`/`s'`/
  `f'`; a search demanding `i'` that finds only `s'` → WOCONSTANIC-wait (not NK); float arithmetic;
  string equality.
- Approval: typed values and operators; `3 + "x"` (char-demand miss → wait/NK per FOOP-43); a
  brane that resolves only once a correctly-characterized value is coordinated in.

## Rejected Alternatives

### A. Characterization as a crude verify-then-NK gate

Check the type; if wrong, NK immediately. **Rejected** (Atlas): a wrong/absent correctly-typed
value is *not-found*, which per FOOP-43 is ECONSTANIC/WOCONSTANIC — the brane **waits**, it
doesn't die.

### B. Value FIR carries its own primitive kind

Give each value FIR a kind field (the FIR *is* the type), reading `i'`/`s'`/`f'` on the LHS only as
a verified assertion. **Rejected** (Atlas): the chosen model is the reverse — the tag is **LHS
metadata enforced through characterized search** (`~b'x` matches only `b'`-characterized `x`), not a
kind on the value. Search *is* the type check; this keeps everything in the one-engine matcher and
avoids a parallel value-kind mechanism. (It also means a coordinated value is plausibly
characterization-free — the tag was never on the value.)

## Open Questions

- **RESOLVED:** the type tag is **LHS metadata, enforced by characterized search** (§The model),
  not a value-carried kind. **RESOLVED:** mixed arithmetic uses the **table-detection** trick (see
  §Arithmetic) — no coercion lattice.
- The **RHS-characterization fallback** ("if no LHS `b'x`, check the search path for RHS carrying
  `b'`") — deferred to a later pass; needs a precise rule.
- Minimal string operator set (literals + equality first, or concat/length too?).
- Does `default_equal` (FOOP-33) extend to string/float equality? Float equality — exact bits.
- Coordination interaction: does a *coordinated* characterized value keep or shed its
  characterization (parallels FOOP-43-C2 position-strip and FOOP-24 marker-strip)? (Lean: shed —
  tag is LHS metadata, re-demanded at the new site.)
- Interaction with null-characterization (FOOP-33 name constants) — `i'`/`s'`/`f'` are normal
  chars alongside the null slot.
- **What characterization means for a NON-primitive (a brane)** — a `myType'{...}` brane the FVM
  doesn't back with a primitive but searches can demand (`~myType'thing`). This is the
  general-user-type-system seed — the same LHS-metadata-via-search model extends to it for free.
  Scope as a successor, but the model already accommodates it.

## Plan (lean)

- [ ] Resolve the KEY question: type tag on the value FIR vs the LHS Identifier.
- [ ] String + float literals (lexer/parser/AST); `StringFir`/`FloatFir` (born Independent) +
      `*_nyes_transitions`.
- [ ] Add typed accessors `get_value_int`/`_float`/`_string`; route `as_i64()` call sites through
      the matching one.
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
