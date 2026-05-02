# Phase 4.5 — Concatenation + Deferred Features

> Goal: Add the language features deliberately deferred from earlier phases —
> concatenation, forward search, and other constructs that benefit from a stable
> base. Sequential numbering would suggest these come "after" Phase 4, but in
> practice this phase can interleave with Phase 4 once the rhythm of language
> growth is established.

---

## Why Deferred

These features interact with constanic semantics in ways that benefit from a
stable Phase 2 foundation:

- **Concatenation `A B`** introduces a new AB layer. `B`'s unanchored searches
  must walk through A first, then the original AB chain. Getting this right
  requires a clean handle on AB threading inside the evaluator.
- **Forward search `~`** semantically requires a brane that's already Constant
  (you can't forward-search a brane that's still being constructed). Adding it
  before the Constant/Constanic distinction is rock-solid risks subtle bugs.

---

## Features in Scope

### Concatenation `A B`

**Sequential blocking model** (decided in earlier discussion):

```
eval(Concatenation(A, B), ab, ib):
  aResult = eval(A, ab, ib)
  if aResult is not Constant -> whole concat is Constanic (hold AST)
  bResult = eval(B, ab = aResult :: ab, ib)
  return merged brane (A statements followed by B statements)
```

No isolation stage. No cloning. No three-stage process. B's unanchored searches
find things in A first (A is the nearest AB layer), then look further up.

**FIR additions:**
```scala
case class ConcatenationFir(
  elements: List[Fir],
  state:    FirState = FirState.Initialized
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

## Phase 4.5 Exit Criteria

- All concatenation `.foo` tests pass.
- Forward search `~` works on Constant branes; produces NK on Constanic anchors.
- A new concatenation regression test specifically exercises "A is Constanic →
  whole concat is Constanic without partial evaluation of B".

---

## Last Updated

**Date**: 2026-05-01
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.7 (1M Context)
**Changes**: Initial Phase 4.5 outline — concatenation and forward search added
once Phase 1–3 stabilize.
