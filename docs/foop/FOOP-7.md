---
foop: 7
title: Constanic Clone — recoordination algorithm
author: hc <hc.busy@gmail.com>
status: Brewing
type: Standards
created: 2026-05-01
phase: phase-2
supersedes: []
---

# FOOP-7: Constanic Clone — recoordination algorithm

## Abstract

Defines `constanicClone(R, newParent)`, the function called whenever a
search resolves to a result `R` that needs to be installed in a new context
(the searcher's brane). The function dispatches on `R`'s `Nyes` state:

- `CONSTANT` / `INDEPENDENT` / `NK`: return `R` (share, do not clone)
- `ECONSTANIC`: deep copy with state reset to `EMBRYONIC`, re-parent
- `WOCONSTANIC`: deep copy with cloned children, state reset to `BRANING`,
  re-parent
- nigh states (`PREMBRYONIC`, `EMBRYONIC`, `BRANING`): caller bug

This algorithm is invoked uniformly by both Phase 2 (depth-first) and
Phase 4 (breadth-first) UBC drivers, and by Phase 6 (concatenation) when
elements are merged into a new context. The function's smarts are
internal; callers don't branch on `R`'s state.

## Motivation

Foolish's recoordination semantics (a constanic FIR may resolve when
placed in a new context) require that constanic search results be
*cloned* into the searcher's context rather than shared by reference. If
shared, a future context change to the original search result would
affect every searcher that pointed at it — wrong, because each search
should resolve in its own context.

But not all results need cloning:
- CONSTANT FIRs are immutable; sharing is correct and saves memory.
- INDEPENDENT FIRs are CONSTANTs that no context could change (literals);
  sharing is even more obviously correct.
- NK is terminal; recoordination cannot rescue it.

Only ECONSTANIC and WOCONSTANIC need actual clones. ECONSTANIC clones
re-attempt their search in the new context. WOCONSTANIC clones
recursively recoordinate their constanic children.

The d0_5 doc in the broader docs branch describes this algorithm. It was
written assuming UBC2's full state machine and message-passing model; this
FOOP refactors it for our depth-first sequential evaluator (Phase 2) and
ensures Phase 4's breadth-first driver inherits the same algorithm.

## Specification

### Algorithm

```scala
def constanicClone(original: Fir, newParent: Fir): Fir = original.state match
  case Nyes.CONSTANT | Nyes.INDEPENDENT =>
    // Immutable values are safe to share. INDEPENDENT additionally cannot be
    // changed by recoordination, so sharing is doubly correct.
    original

  case Nyes.NK =>
    // Terminal failure. Recoordination cannot rescue an NK.
    original

  case Nyes.ECONSTANIC =>
    // A search FIR that found nothing in the original context. Clone it
    // (preserving the search's pattern, direction, anchored flag) and reset
    // state to EMBRYONIC so the next UBC step retries the search in the
    // new context.
    val clone = original.copyWith(parent = newParent, state = Nyes.EMBRYONIC)
    clone

  case Nyes.WOCONSTANIC =>
    // An expression or brane whose evaluation depends on at least one
    // ECONSTANIC. Clone the structure, recursively recoordinate the
    // children, reset state to BRANING so the next UBC step recurses in.
    val clonedChildren = original.children.map(c => constanicClone(c, /* the cloned self */))
    val clone          = original.copyWith(
                          children = clonedChildren,
                          parent   = newParent,
                          state    = Nyes.BRANING
                        )
    clone

  case nigh @ (Nyes.PREMBRYONIC | Nyes.EMBRYONIC | Nyes.BRANING) =>
    // Caller bug: never call constanicClone on a non-constanic FIR.
    sys.error(s"constanicClone called on nigh FIR (state=$nigh)")
```

### Per-state behavior table

| `original.state` | Cloned? | Clone's initial Nyes | Why |
|---|---|---|---|
| CONSTANT | No (share) | n/a | immutable |
| INDEPENDENT | No (share) | n/a | immutable + recoordination-immune |
| NK | No (share) | n/a | terminal failure |
| ECONSTANIC | Yes | EMBRYONIC | re-runs search in new context |
| WOCONSTANIC | Yes (with cloned children) | BRANING | re-steps to recurse into children |
| PREMBRYONIC | error | n/a | caller bug |
| EMBRYONIC | error | n/a | caller bug |
| BRANING | error | n/a | caller bug |

### Caller invariant

Callers MUST step the search result FIR to a constanic terminal state
(or NK) before invoking `constanicClone`. Phase 2's depth-first ordering
makes this trivial: the result is always at completion before the search
that finds it is evaluated.

### Recursion shape (WOCONSTANIC case)

When cloning a WOCONSTANIC FIR, the function recursively applies
`constanicClone` to each child. CONSTANT children are shared; constanic
children are cloned. The recursion bottoms out at:
- CONSTANT/INDEPENDENT/NK leaves (shared)
- ECONSTANIC leaves (cloned, set to EMBRYONIC)

The clone tree mirrors the original tree's structure but every constanic
node is fresh (with a fresh state ready for re-stepping).

### Interaction with search short-circuiting

After Phase 2 runs `step()` on a freshly cloned WOCONSTANIC search,
search short-circuiting (per `phase2_ubc.md` §"The Hard Part") rewires the
clone's `target` field to point directly at the underlying ECONSTANIC,
collapsing chains of WOCONSTANIC searches.

## FIR Impact

Each FIR variant must support a `copyWith` operation that takes an
optional new state, optional new children, optional new parent. Scala
case classes get this for free via `copy(...)`.

The `Fir` trait may benefit from a method:

```scala
sealed trait Fir:
  def state: Nyes
  def deepCopyWithState(newState: Nyes, newParent: Fir): Fir
```

Or `constanicClone` may be implemented as a per-variant case statement
that uses `copy(...)` directly. Implementation choice deferred to Phase 2.

## UBC Step Impact

`constanicClone` is invoked from these step rules (see `phase2_ubc.md`):

1. `step(SearchFir, unanchored)` — when the scope walk finds a result.
2. `step(SearchFir, anchored)` — when the local search finds a result.
3. `step(IndexFir)` and `step(HeadTailFir)` — when the position is
   resolved.
4. (Phase 6) `step(ConcatenationFir)` — when an element produces a
   result that subsequent elements may reference.

In all cases, the call site is identical:

```scala
val cloned = constanicClone(rawResult, this.parent)
this.target = cloned
```

## Test Plan

Unit tests for `constanicClone`:

```scala
test("constanicClone shares CONSTANT") {
  val c = ConstantIntFir(42)  // CONSTANT or INDEPENDENT
  constanicClone(c, anyParent) should be theSameInstanceAs c
}

test("constanicClone shares INDEPENDENT") { ... }
test("constanicClone shares NK") { ... }

test("constanicClone clones ECONSTANIC, sets to EMBRYONIC") {
  val original = SearchFir("^missing$", Backward, false, None, Nyes.ECONSTANIC)
  val cloned   = constanicClone(original, newParent)
  cloned should not be theSameInstanceAs(original)
  cloned.state shouldBe Nyes.EMBRYONIC
  cloned.asInstanceOf[SearchFir].pattern shouldBe "^missing$"
}

test("constanicClone clones WOCONSTANIC, sets to BRANING, recurses children") {
  val ec      = SearchFir("^x$", Backward, false, None, Nyes.ECONSTANIC)
  val woRef   = SearchFir("^y$", Backward, false, None, Nyes.WOCONSTANIC, target = ec)
  val cloned  = constanicClone(woRef, newParent)
  cloned.state shouldBe Nyes.BRANING
  cloned.target should not be theSameInstanceAs(ec)
  cloned.target.state shouldBe Nyes.EMBRYONIC
}

test("constanicClone errors on nigh state") {
  val nigh = SearchFir(..., state = Nyes.EMBRYONIC)
  intercept[RuntimeException] { constanicClone(nigh, anyParent) }
}
```

Integration tests via `.foo` approval tests:

- `{a = unknown, y = a}`: y's body's target should be a clone of a's body
  (not the same instance), both pointing at the same recursive clone of
  the missing-name search.
- The worked example from `phase2_ubc.md` (`{y=z, x=y, w=x, v=w+x, u=v+w}`)
  should produce the documented final-state table.

## Rejected Alternatives

### A. Always clone, even CONSTANT

Simplest possible algorithm: every search result becomes a fresh clone.
**Rejected**: O(n) memory blowup on programs with many references to the
same constant. The CONSTANT/INDEPENDENT case being a no-op is an
important optimization that costs essentially nothing in code complexity.

### B. Never clone in Phase 2; defer all cloning to Phase 6 (concatenation)

Saves implementation work in Phase 2. **Rejected**: makes the
`constanicClone` an isolated Phase 6 special case rather than a uniform
operation. Forces Phase 6 to retroactively trace every search target in
the FIR tree to clone them, which is more complex than the uniform
"clone at search resolution" rule.

### C. Use a wake-up queue instead of cloning

Don't clone constanic results; instead, when a context change happens,
walk the FIR tree finding constanic FIRs and re-stepping them in place.
**Rejected**: requires the wake-up queue + dependency map machinery that
Phase 2 explicitly defers (FOOP-6). Also fights Foolish's semantics —
each searcher should resolve in *its own* context, so each needs its own
clone, not a shared mutating original.

### D. Lazy cloning: defer until the clone is actually re-stepped

Clone metadata only; do the actual deep-copy work on first re-step.
**Rejected**: micro-optimization. Saves negligible work in the common
case (most clones are immediately re-stepped). Adds complexity to the
clone representation. Not justified for the language's current scale.

## Open Questions

- **Parent pointer representation**: the algorithm sketch passes
  `newParent` but our current `Fir` case classes don't have an explicit
  `parent` field. Implementation must decide whether to add one
  (mutable reference for back-pointers) or to thread parent context
  through `step()` calls. Likely the latter; deferred to Phase 2
  implementation.
- **WOCONSTANIC search FIRs after short-circuiting**: when a search has
  been short-circuited (its `target` points at a non-search FIR), is it
  still WOCONSTANIC, or some other state? The algorithm assumes
  WOCONSTANIC. Phase 2 implementation may surface a need to distinguish
  "WOCONSTANIC search with intact chain" from "WOCONSTANIC search whose
  target is now a non-search FIR." Defer to implementation.

## References

- `scala-mvp/foolish-scala/docs/phase2_ubc.md`: where `constanicClone` is
  invoked (per-FIR step rules + the worked example).
- `scala-mvp/foolish-scala/docs/00_accumulated_specs.md`: Nyes lifecycle
  definitions.
- `docs/ubc1/how/d0_5_brane_recoordination.md` (in the broader docs branch):
  the originating UBC2 design for constanic cloning.
- FOOP-3: amended to clarify that the WOCONSTANIC state itself is retained
  (this FOOP defines its lifecycle); only the WOCONSTANIC-during-merge race
  in concatenation was eliminated.
- FOOP-6: depth-first sequential ordering, which makes the caller invariant
  trivially satisfied in Phase 2.
