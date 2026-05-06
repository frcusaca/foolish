---
foop: 10
title: Anchored search through constanic anchors — dereference searches, NK on missing brane names
author: hc <hc.busy@gmail.com>
status: Brewing
type: Standards
created: 2026-05-04
phase: phase-2
supersedes: []
---

# FOOP-10: Anchored search through constanic anchors — dereference searches, NK on missing brane names

## Abstract

Defines how anchored searches (`brane.name`, `brane?pat`, `brane^`,
`brane$`, `brane#N`) behave when the anchor is itself in a constanic
state. The rule splits by what kind of FIR the anchor resolves to:

- **Anchor resolves to a constanic search FIR** (ECONSTANIC or
  WOCONSTANIC): the search **dereferences through the anchor's chain**
  to the underlying ECONSTANIC or to a CONSTANT brane, and performs the
  anchored search there. State propagates: if the dereferenced result is
  ECONSTANIC, the anchored search becomes WOCONSTANIC.
- **Anchor resolves to a constanic brane FIR** (a brane in BRANING or
  WOCONSTANIC state, where the brane's structure is known but some
  values are still resolving): the anchored search proceeds against the
  brane's statement list. If the name is **not present** in that
  statement list, the result is **NK** — permanent failure, like asking
  for `square.circumference`.

This corrects an inconsistency between `phase2_ubc.md` (which previously
said "anchored search on constanic anchor → NK") and
`00_accumulated_specs.md` (which said anchored search on constanic anchor
returns `🧠??`, i.e., constanic).

## Motivation

Anchored search semantics need to handle two distinct "constanic anchor"
situations that prior wording lumped together:

1. **The anchor is a chain of indirections that hasn't fully resolved.**
   Example: `b = some_brane; c = b.x`. When `c = b.x` is evaluated, `b`
   may resolve to a search FIR (a search for `some_brane`) that is
   WOCONSTANIC because `some_brane` is missing. The anchored `.x`
   search should **wait on the chain**, not declare NK. If the chain
   eventually resolves to a brane, the search proceeds; if it resolves
   to ECONSTANIC, the anchored search is WOCONSTANIC too.

2. **The anchor IS a brane structurally, but the brane is constanic
   because its values aren't all in.** Example:
   ```
   {
     b = {x = unknown, y = 5},
     c = b.y
   }
   ```
   `b`'s body is a brane with two statements. `x = unknown` is
   ECONSTANIC, so the brane `b` is WOCONSTANIC. But `b.y` should still
   succeed: `y` is a known statement of the brane, its value is 5.
   The brane's *structure* (which names are defined) is fixed even when
   values aren't.

The problem from previous wording: treating both situations as "anchor
constanic → NK" would break case 2 — `b.y` should resolve to 5, not NK.

The other half of the rule: if `b.q` is asked and `q` is NOT a
statement of `b`'s brane, the answer IS NK (permanent). The brane's
structure is known; `q` is provably not there. No future recoordination
of `b` (it's already a brane structurally, just with constanic values)
will make `q` appear. Compare: asking for `square.circumference` — the
square is fully defined as a shape; asking for a circle's property of
it is a category error, not a "wait and see."

## Specification

### Anchor resolution first

For any anchored search FIR, the first step is to resolve the anchor:

```scala
val anchorResolved = stepToCompletionInPhase2OrTerminalInPhase4(anchor)
```

The anchor is itself a FIR (a search FIR, an OperatorFir, or directly a
brane). After it reaches a terminal Nyes state, dispatch on what it is.

### Anchor is a CONSTANT brane

```scala
case anchor: NormalBraneFir if anchor.state == Nyes.CONSTANT =>
  searchLocally(anchor, pattern, direction) match
    case Some(found) => target = constanicClone(found); state' = ...
    case None        => state' = Nyes.NK     // miss on CONSTANT brane → NK
```

This case is unchanged from prior spec.

### Anchor is a constanic brane (BRANING or WOCONSTANIC)

The brane's statements list is known; the brane's *values* are still
resolving. The anchored search proceeds against the statements list:

```scala
case anchor: NormalBraneFir if anchor.state == Nyes.WOCONSTANIC || anchor.state == Nyes.BRANING =>
  searchLocally(anchor, pattern, direction) match
    case Some(found) =>
      // The named statement exists. Found's body may be CONSTANT, ECONSTANIC,
      // WOCONSTANIC, or NK — propagation works as usual via constanicClone.
      target  = constanicClone(found)
      state'  = ...   // propagated from target
    case None =>
      // The named statement does NOT exist in this brane's structure.
      // The brane may not be CONSTANT yet, but its statement-list IS known
      // (statements are added at parse time, not at eval time). Missing name →
      // permanent NK, just like missing on a CONSTANT brane.
      state' = Nyes.NK
```

The "permanent NK" here is the key correction: the brane's *shape*
(which names exist) is fixed once the brane FIR is constructed.
WOCONSTANIC just means values are still resolving, not that new
statements might arrive. Missing names are permanently missing.

### Anchor is a constanic search FIR (ECONSTANIC or WOCONSTANIC)

The anchor isn't a brane — it's a search that hasn't (yet, or ever)
dereferenced to a brane. Dereference through the chain:

```scala
case anchor: SearchFir if Nyes.isConstanic(anchor.state) =>
  // Walk the search's target chain to find the underlying non-search FIR.
  val derefed = dereference(anchor)  // follows .target through WOCONSTANIC chains
  derefed match
    case b: NormalBraneFir =>
      // The chain ended at a brane. Recurse into the brane case above.
      stepAnchoredSearchAgainst(b, pattern, direction)
    case ec: SearchFir if ec.state == Nyes.ECONSTANIC =>
      // Chain ended at an unresolved search. The anchor isn't actually
      // a brane (yet). Wait on it.
      state' = Nyes.WOCONSTANIC
    case nk if nk.state == Nyes.NK =>
      state' = Nyes.NK            // chain ended in NK
    case _ =>
      // Chain ended at something that isn't a brane — e.g., an integer.
      // Anchored searches require a brane anchor; non-brane → NK.
      state' = Nyes.NK
```

### Anchor is NK

```scala
case anchor if anchor.state == Nyes.NK =>
  state' = Nyes.NK    // NK propagates
```

### Anchor is in a nigh state

```scala
case anchor if Nyes.isNigh(anchor.state) =>
  // Phase 2 should have stepped the anchor to terminal already (depth-first).
  // Phase 4 (breadth-first) may need to wait.
  state' = ...    // implementation-defined: BRANING / wait
```

In Phase 2, this case shouldn't arise because of left-to-right depth-first
ordering. In Phase 4, it's a real case that requires the cooperative
scheduler.

### Summary of the rules

| Anchor's resolved kind & state | Search behavior |
|---|---|
| CONSTANT brane | local search; miss → NK |
| WOCONSTANIC or BRANING brane | local search against statement-list; miss → NK (statement-list shape is fixed); hit → constanicClone result |
| WOCONSTANIC search | dereference chain; recurse on what chain ends at |
| ECONSTANIC search | result is WOCONSTANIC (waiting on the chain) |
| NK | NK |
| nigh | wait (Phase 4) / shouldn't arise (Phase 2) |

### Why the brane case matches even WOCONSTANIC anchors

Branes have their statement-list determined at construction time. The
parser produces `NormalBraneFir(statements = List(...))` with statements
present immediately. The brane's Nyes state changes as values resolve,
but no statement is ever *added* during stepping. So "is statement N in
this brane?" is answerable as soon as the brane FIR exists, regardless
of state.

This is why missing-on-WOCONSTANIC-brane is permanent NK rather than
"wait and see." There's nothing to wait for — the answer is already
determinable.

## FIR Impact

No new FIR variants. Anchored `SearchFir`'s step rule changes per
above.

## UBC Step Impact

Phase 2's anchored `SearchFir` step rule needs to dispatch on the
anchor's type (brane vs search) in addition to its state. Add the
`dereference` helper that walks `.target` chains.

The implementation should be careful that "miss on WOCONSTANIC brane →
NK" only fires after the local search has been run against the brane's
full statement list. (Don't short-circuit "anchor is constanic →
NK" before checking.)

## Test Plan

Phase 2 approval tests:

```
{
  b = {x = unknown, y = 5},
  c = b.y    !! should resolve to 5
}
```

```
{
  b = {x = unknown, y = 5},
  c = b.q    !! should be NK — q is not a statement of b
}
```

```
{
  b = unknown,
  c = b.x    !! b is ECONSTANIC; c should be WOCONSTANIC, waiting on the chain
}
```

```
{
  b = {x = 5, y = 6},
  c = b.q    !! q not in b's statements → NK; b is CONSTANT so prior rule applies
}
```

The existing `anchoredSearchOnConstanic.foo` and
`anchoredSearchFailsOnConstant.foo` tests should be reviewed against
this clarification.

## Rejected Alternatives

### A. Treat all constanic anchors uniformly as NK

The simplest rule: any constanic anchor → NK. **Rejected**: breaks the
case where the brane structurally has the name but its other values
aren't in yet. `{b = {x = unknown, y = 5}, c = b.y}` should produce 5,
not NK.

### B. Treat all constanic anchors uniformly as WOCONSTANIC ("wait")

The other simplification: any constanic anchor → wait. **Rejected**:
breaks the permanent-NK case. `{b = {x = 5, y = 6}, c = b.q}` should be
NK because q isn't there, regardless of whether b is "fully done."

### C. Make brane statement-lists themselves mutable (statements can be added during stepping)

Would force "wait" semantics because we couldn't know if a name might
later appear. **Rejected**: contradicts Foolish's writing-order rule (a
brane's statements come from the source text; they're added at parse
time; stepping resolves their bodies, not their existence). This would
be a major language change and isn't motivated by any current
requirement.

## Open Questions

- The exact behavior when an anchor's chain dereferences to a
  non-brane non-search non-NK FIR (e.g., the chain ends at an
  OperatorFir). Currently spec says "→ NK." This may need refinement if
  language additions allow operator FIRs to be searchable. Defer.

## References

- `scala-mvp/foolish-scala/docs/phase2_ubc.md`: anchored SearchFir step
  rule needs update.
- `scala-mvp/foolish-scala/docs/00_accumulated_specs.md`: lifecycle
  table is consistent with this FOOP; the symbol mapping line is
  correct.
- FOOP-7: constanicClone is invoked on the search result as usual.
- FOOP-8: FIR mutability — anchor's `.state` is read directly.
- FOOP-11: search-stops-at-NK rule is consistent with this FOOP's NK
  propagation.
