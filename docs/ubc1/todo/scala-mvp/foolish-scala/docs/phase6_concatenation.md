# Phase 6 — Concatenation + Deferred Features

> Goal: Add the language features deliberately deferred from earlier phases —
> concatenation, forward search, and other constructs that benefit from a stable
> base. Concatenation is the first feature where `constanicClone` actually
> recoordinates across context changes (in earlier phases recoordination is
> a no-op-equivalent because contexts don't change).

---

## Why Deferred

These features interact with constanic semantics in ways that benefit from a
stable Phase 2 + Phase 4 foundation:

- **Concatenation `A B`** introduces a new AB layer. `B`'s unanchored searches
  must walk through A first, then the original AB chain. The `constanicClone`
  recoordination (FOOP-7) becomes operationally meaningful here for the first
  time, because cloning into the merged context can produce different search
  results than the original context.
- **Forward search `~`** semantically requires a brane that's already CONSTANT
  (you can't forward-search a brane that's still being constructed). Adding it
  before the Nyes lifecycle is rock-solid risks subtle bugs.

---

## Features in Scope

### Concatenation `A B`

**Sequential blocking model** (FOOP-3, refined here):

```
step(ConcatenationFir(elements), ib):
  // Step elements left-to-right; later elements see earlier ones as AB layers.
  for i in 0 until elements.length:
    elements(i) = stepToCompletion(elements(i), ab = elements[0..i-1] :: outer_ab, ib)
    if elements(i).state == NK -> propagate NK
  // Merge: produce a single brane whose statements are all elements' statements
  // concatenated in order. Elements that are constanic remain so in the merged
  // form; recoordination via constanicClone happens during the per-element step.
  return mergedBrane(elements)
```

Per FOOP-3 amended, this does NOT introduce a separate WOCONSTANIC concatenation
race — the per-element stepping uses the standard constanicClone mechanism from
Phase 2.

**FIR additions:**
```scala
case class ConcatenationFir(
  elements: List[Fir],
  state:    Nyes = Nyes.EMBRYONIC
) extends Fir
```

### Forward search `~`

**FIR additions:**
- `SearchFir.direction = Forward` is already in the Phase 1 model.
- The compiler enables `RegexSearchAstn(_, REGEXP_FORWARD_LOCAL, _)` (was rejected
  in P1.11).

### `^` head and `$` tail on the unanchored side

Currently anchored-only. May be useful as `^` in concatenation context.

---

## Tests to Enable

These `.foo` files are deferred from Phase 2:
- `concatenationBasics.foo`
- `concatenationResolution.foo`
- `concatenationSearch.foo`
- `concatenationResolutionAdv.foo`
- `testTilde.foo`

---

## Phase 6 Exit Criteria

- All concatenation `.foo` tests pass.
- Forward search `~` works on CONSTANT branes; produces NK on constanic anchors.
- A new concatenation regression test specifically exercises "A is constanic →
  whole concat is WOCONSTANIC, B is not partially-evaluated past `constanicClone`."
- A concatenation test specifically exercises recoordination producing a
  different result than the original (e.g., `f = {a = x}` cloned into a context
  containing `x = 42` resolves `a` to 42).

---

## Last Updated

**Date**: 2026-05-01
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.7 (1M Context)
**Changes**: Renumbered Phase 5 → Phase 6 (after inserting Phase 4 breadth-first).
Adopted Nyes terminology. Updated FIR snippet to use `Nyes.EMBRYONIC`. Concatenation
algorithm now references constanicClone (FOOP-7) instead of standalone three-stage
process. Added recoordination regression test to exit criteria.
