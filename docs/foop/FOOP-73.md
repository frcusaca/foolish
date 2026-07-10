---
foop: 37
title: Boolean operators — and, or, not, nor, xor (on boolean-characterized objects)
author: Atlas hc.busy@gmail.com
status: Draft
type: Standards
created: 2026-07-09
phase: phase-4
supersedes: []
begun: [ ]
---

# FOOP-73: Boolean operators — and, or, not, nor, xor

> Lean draft. Fuller notes in the Appendix and `NOTES-creation-lineage-and-search-family.md` §1.
> (Implementation order: #4. Renumbered 2026-07-09.)

## Abstract

Define the five Foolish boolean operators `and`, `or`, `not`, `nor`, `xor` — operating on
**boolean-characterized objects** (`True`/`False`, FIRs created in `system.foo`). **Preferred
design: the operators are ordinary Foolish — each is a brane holding a truth TABLE, applied by a
SEARCH** (value search + contexted continuation + contexted index) that looks up the result row.
This needs **no FVM-privileged computation** and fully honors FOOP-33's "no privileged layer." A
FVM-computed fallback (creations dispatched by `Rc::ptr_eq`) exists only if the table search proves
insufficient. This completes the FOOP-33 creation → booleans story.

> **Not to be confused with the matcher boolean operators `&&`/`||`** (FOOP-93). Those combine
> *matcher Approve/Reject results* inside the search engine and are **compiler-hard-coded** — they
> are not Foolish values. **This** FOOP is about boolean-*characterized objects* operated on by
> **Foolish**. Two different layers; keep them apart.

## Motivation

FOOP-33 creates `True`/`False` but they are inert — you cannot compute with them. Boolean
operators make them useful, and doing it by "declared-as-creation, computed-by-FVM" keeps the
declaration in Foolish (no privileged keyword) while giving real behavior. This is a **deliberate
departure from FOOP-33's "no privileged layer"** at the *behavior* level, stated plainly.

## Specification

### Preferred design — operators are Foolish truth-table SEARCHES (no privileged layer!)

The key realization (Atlas, 2026-07-09): with the search upgrades, **boolean operators can be
defined in Foolish itself as a search over a truth table** — the FVM computes *nothing* special.
An operator is a **brane holding its truth table**, and applying it is a **value search + contexted
continuation + contexted index** that looks up the result row. This fully realizes the Creation
Postulate's "no privileged layer": booleans are creations, operated on by Foolish's own search.

```foolish
{!!system.b.foo
    'True  = ⬤ ;
    'False = ⬤ ;
    AND = {
        A=True,  B=True,  True ;
        A=True,  B=False, False ;
        A=False, B=True,  False ;
        A=False, B=False, False ;
    } ;
    !! Applying AND to (v1, v2): find the row where A=v1 and B=v2, then take the next
    !! statement (the result). Uses value search (~A=), contexted continuation (&~B=),
    !! and contexted index (&#1):
    !!     result = AND~A=v1 &~B=v2 &#1 ;
}
```

`OR`, `NOR`, `XOR` are analogous 4-row tables; `NOT` is a 2-row table (`A=True,False;
A=False,True`) looked up by `NOT~A=v1 &#1`. Equality on `A=v1` is **referential** (FOOP-33
`default_equal` — the created `True`/`False` compared by identity).

**This makes the boolean operators ordinary Foolish**, using exactly the search machinery being
built: value search (`~A=`), the contexted continuation connector (`&~B=`), and contexted index
(`&#1`). **Dependency implication:** the table-search design depends on those search features
(value search from FOOP-23; contexted continuation; contexted index) — see Open Questions on
ordering.

### Application surface

- **Declaration.** `system.foo` (a `system.b.foo` section) defines `AND`/`OR`/`NOT`/`NOR`/`XOR`
  as truth-table branes, plus `'True`/`'False` (null-characterized creations, FOOP-33).
- **Application form.** Either the explicit search (`AND~A=v1 &~B=v2 &#1`) or a sugar
  (`{v1,v2} AND` via RPN concatenation that expands to the table search) — decide the surface
  syntax (Open Questions).
- **Result.** The looked-up `True`/`False` (by identity). A lookup miss (bad args) follows FOOP-43
  (ECONSTANIC/WOCONSTANIC-wait), and a characterization demand once FOOP-63 lands.

### Fallback — FVM-computed (only if the table search proves insufficient)

If the Foolish-native table search cannot be made ergonomic/correct, fall back to declaring the
operators as creations and computing in Rust, dispatched by creation identity (`Rc::ptr_eq`
against the `system.foo` handles), via the `OperatorFir::combine` pattern. This reintroduces a
privileged layer and is therefore the *second* choice — see Rejected Alternatives.

## FIR Impact

**Preferred (table-search) design: NONE** — the operators are Foolish source (truth-table branes
in `system.foo`), applied by existing search FIRs (value search, contexted search, contexted
index). No new FIR, no FVM dispatch. This is the whole point.

**Fallback (FVM-computed) design:** the dispatch would live in the concatenation/application step
following the `OperatorFir::combine` pattern, needing stable `Rc` handles to the `system.foo`
creations. Only if the table search is chosen against.

## UBC Step Impact

**Preferred:** none beyond what the search FOOPs already provide — the operators evaluate as
ordinary searches over the `system.foo` tables.

**Fallback (FVM-computed):** concatenation-application step (`fir_kinds.rs:2162`) recognizes a
boolean-operator creation by `Rc::ptr_eq` and computes via the `OperatorFir::combine` model
(`fir_kinds.rs:483`); evaluator keeps the `system.foo` `Rc`s reachable.

## Test Plan

- Unit: identity dispatch (is-this-`and`?); each truth-table row for all five operators;
  non-boolean args → NK (or wait, post-FOOP-63).
- Approval: `{T,T}and`→T, `{T,F}and`→F, `T not`→F; full `nor`/`xor` tables; `{T,3}and`→NK.
- Comprehensive: booleans interacting with value search and `system.foo` resolution.

## Rejected Alternatives

### A. FVM-computed operators (creations dispatched by `Rc::ptr_eq`)

Declare `and`/`or`/… as creations and compute the logic in Rust, dispatched by creation identity.
**Demoted to fallback** (not outright rejected): it reintroduces a **privileged layer** (the FVM
knows what `and` *means*), which the table-search design avoids entirely. Use only if the Foolish
truth-table search cannot be made ergonomic/correct. This is the *second* choice.

### B. Built-in keyword operators (no creation)

Make `and`/`or`/… reserved keywords. **Rejected**: violates "no privileged layer" at the
*declaration* level — the operators would not be created.

### C. Assertion-defined behavior (`⊦`)

Assert operator behavior (`⊦ {T,T} and`). **Rejected**: `⊦` is de-emphasized and "how a created
symbol acquires operational behavior via assertion" is a large unspecified design. The truth-table
search gives behavior concretely.

## Open Questions

- **Ordering tension:** the preferred table-search design **depends on the search features**
  (value search — FOOP-23; contexted continuation connector; contexted index) it looks the table
  up with. Those are the search FOOPs (FOOP-93/04/14 in the reorg), which come *after* this FOOP
  in the current order. **Resolve:** either move the boolean-table work after the search FOOPs, or
  ship the FVM-computed fallback first and convert to table-search once searches land. (Value
  search itself is FOOP-23, already Draft — so the minimum needed may already be near.)
- Surface syntax: explicit `AND~A=v1 &~B=v2 &#1` vs a sugar (`{v1,v2} AND`).
- Result for non-boolean/insufficient args — NK now, WOCONSTANIC-wait after FOOP-63.
- Are user-written operator bodies (extending/overriding the tables) allowed?
- Short-circuit for `and`/`or` (a table search evaluates both args regardless — matters for
  side-effect-free Foolish, probably fine).

## Plan (lean)

- [ ] Decide the ordering vs the search FOOPs (table-search needs value/contexted search) — see
      Open Questions. Pick: table-search-after-searches, or FVM-fallback-first-then-convert.
- [ ] **Preferred:** write the truth-table branes (`AND`/`OR`/`NOT`/`NOR`/`XOR`) + `'True`/`'False`
      in `system.foo` (`system.b.foo`); define the application (explicit search and/or sugar).
- [ ] Verify the table lookup (`AND~A=v1 &~B=v2 &#1`) resolves correctly with referential equality
      (FOOP-33) on `True`/`False`.
- [ ] *(Fallback only)* identity-dispatch + `OperatorFir::combine` compute, if table-search is
      chosen against.
- [ ] Unit + approval tests: full truth tables via the table search; non-boolean args.
- [ ] Comprehensive `foop_73_comprehensive.foo`.
- [ ] Worktree lifecycle per `foop.md`.

## Appendix — notes toward the full spec

- **Hard dependency on FOOP-33** (True/False, creation identity, `default_equal`, `system.foo`).
- Once **FOOP-63** (Primitive Characterization) lands, non-boolean args become a
  characterization-demand miss → WOCONSTANIC-wait rather than immediate NK — align then.
- **FOOP-83** integer comparisons (`<`/`>`) return these booleans, so this FOOP unblocks typed
  comparisons.
- Deliberate "no privileged layer" departure (behavior FVM-native) — state it in the FOOP body so
  a reviewer doesn't read it as an inconsistency with FOOP-33.
- **The two-booleans distinction** (this FOOP = Foolish objects; FOOP-93 `&&`/`||` = matcher
  results) is the thing most likely to confuse a reader — lead with it.

## References

- Prior: FOOP-33 (creation, identity, `system.foo`, True/False); FOOP-63 (characterization-demand),
  FOOP-93 (the *other* booleans — matcher `&&`/`||`); `docs/why/creation_postulate.md`,
  `CREATION.md`, ADVANCED_FEATURES §Brane Concatenation.
- Code: `fir_kinds.rs:2162` (concatenation step), `:483` (`OperatorFir::combine` model).
- Notes: `NOTES-creation-lineage-and-search-family.md` §1 + Engineering guidance.

## Last Updated

**Date**: 2026-07-09
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes**: Renumbered from FOOP-83 to FOOP-73 (impl-order reorg). Made the two-booleans
distinction explicit (this = Foolish boolean-characterized objects; FOOP-93 `&&`/`||` = matcher
results). **New PREFERRED design (Atlas): operators are Foolish truth-TABLE searches** —
`AND = {A=True,B=True,True; …}`, applied by `AND~A=v1 &~B=v2 &#1` — NO privileged FVM layer,
honoring FOOP-33. FVM-computed dispatch demoted to fallback. Flagged the ordering tension:
table-search depends on value/contexted search (the search FOOPs).
