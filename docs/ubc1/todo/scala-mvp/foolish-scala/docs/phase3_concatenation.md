# Phase 3 — Concatenation

> Goal: Implement the concatenation operator `A B C ...` per FOOP-3
> (revised). Concatenation produces a new merged brane of
> `constanicClone`'d copies of each input, in order, and delegates
> further `step()` calls to that merged brane. This is the first phase
> where recoordination across actual context changes happens — earlier
> phases' recoordination is a no-op-equivalent because contexts don't
> change.

> Phase 3 was inserted between Phase 2 (UBC depth-first) and Phase 4
> (CLI) after design discussion. Concatenation deserves its own phase
> because it's the first real exercise of `constanicClone` (FOOP-7) and
> it makes the CLI meaningfully more useful.

Read [FOOP-3](../../../../foop/FOOP-3.md) for the full design rationale
before reading this document.

---

## Phase 3 Deliverable

A `ConcatenationFir` variant in `Fir.scala` plus a step rule that
implements the FOOP-3 algorithm:

1. Each element FIR has reached a constanic terminal state
   (Phase 2's depth-first ordering guarantees this).
2. The step constructs a new `NormalBraneFir` whose statements are
   `constanicClone`'d copies of each input element's statements, in
   left-to-right concatenation order.
3. Subsequent `step()` calls on the `ConcatenationFir` delegate to the
   merged brane (the `ConcatenationFir` either mutates to hold the
   merged brane or is replaced in its parent's statement list — implementer's
   choice per FOOP-8).

The Phase 1 compiler is updated:
- `ConcatenationAstn` now compiles to `ConcatenationFir(elements.map(compileExpr))`
- It is removed from Phase 1's compile-time rejection list (P1.11).

---

## Why this phase exists

Concatenation interacts with constanic semantics in ways that benefit
from a stable Phase 2 (depth-first UBC) foundation, but has its own
value as a milestone:

- **First real recoordination**: in Phases 1–2, `constanicClone` is
  invoked but the cloned context is the same as the original (since
  there are no actual context changes). Concatenation introduces actual
  context changes — element A's constanic searches get re-stepped in a
  merged brane that may now include statements from element B.
- **CLI dependency**: the CLI in Phase 4 should support concatenation
  for users to compose Foolish snippets meaningfully.

---

## Step Algorithm (per FOOP-3)

```scala
step(c: ConcatenationFir):
  // Precondition: each element c.elements(i) has stepped to constanic
  // terminal. Phase 2's depth-first left-to-right ordering ensures this.

  // NK propagation
  if c.elements.exists(_.state == Nyes.NK):
    c.state = Nyes.NK
    return

  // Construct merged statements: each element's statements,
  // constanicClone'd, in concatenation order.
  val mergedStatements: List[StatementFir] = c.elements.flatMap { elem =>
    val resolvedBrane: NormalBraneFir = derefToBrane(elem)
    resolvedBrane.statements.map { stmt =>
      val cloned = constanicClone(stmt)   // FOOP-7
      cloned.asInstanceOf[StatementFir]
    }
  }

  val mergedBrane = NormalBraneFir(
    characterizations = Nil,
    statements        = mergedStatements
  )
  mergedBrane.parent = c.parent

  // Set parent on each cloned statement to the merged brane.
  mergedStatements.foreach { stmt => stmt.parent = Some(mergedBrane) }

  // Delegate further steps to the merged brane.
  c.becomeMergedBrane(mergedBrane)
  c.state = mergedBrane.state    // typically EMBRYONIC, ready for ordinary stepping
```

After this step, ordinary brane stepping (per phase2_ubc.md) takes
over. Constanic clones inside the merged brane re-step in the merged
context. Searches that were ECONSTANIC in their original context may
now find their target.

The `derefToBrane` helper follows search chains to the underlying brane
FIR. The element may be a literal brane (`{...}`) or a search FIR that
resolves to one.

The `becomeMergedBrane` helper is implementation-defined. Two valid
approaches:
- **Mutate self**: `ConcatenationFir` carries an optional `merged:
  Option[NormalBraneFir]` field; once set, all `step()` calls forward
  to it. Read access (e.g., from sequencer) follows the indirection.
- **Replace in parent**: locate this `ConcatenationFir` in its parent's
  statement list and replace it with the merged brane. Cleaner final
  state but requires parent-mutation access.

---

## FIR Additions

```scala
case class ConcatenationFir(
  elements: List[Fir]    // ordered left-to-right
) extends Fir:
  var state:  Nyes        = Nyes.EMBRYONIC
  var parent: Option[Fir] = None
  var merged: Option[NormalBraneFir] = None   // populated by becomeMergedBrane
```

Roundtrip test required (Phase 1 test layer 3): construct a
`ConcatenationFir`, encode to JSON, decode, compare structurally
(per FOOP-8: parent and merged are excluded from JSON; equality is
structural).

---

## Tests to Enable

Existing `.foo` files (currently in
`foolish-core-scala/src/test/resources/.../inputs/`):

- `concatenationBasics.foo`
- `concatenationResolution.foo`
- `concatenationSearch.foo`
- `concatenationResolutionAdv.foo`

These were deferred from Phase 2's approval suite. Move them into
Phase 3's approval test runner.

New dedicated tests (per FOOP-3 §Test Plan):

- **p3_concat_simple.foo**: `{a={x=1, y=2}, b={z=3}, c = a b}` →
  c is a merged brane with statements x=1, y=2, z=3 in order.
- **p3_concat_resolution.foo**: `{f = {a = x}, g = {x = 42}, h = g f}` →
  h is the merged brane `{x=42, a=42}`. The constanic `a = x` in f,
  after being cloned into the merged brane, finds `x = 42` from g.
- **p3_concat_order_matters.foo**: same elements as above but `h = f g`
  — `a = x` is cloned into a position BEFORE `x = 42`, so backward
  search doesn't find x. h is `{a=ECONSTANIC, x=42}`. Document this
  as expected behavior of left-to-right concatenation.
- **p3_concat_chain.foo**: `A B C` produces a single 3-element merged
  brane (not a nested binary tree). Verify the compiler produces
  `ConcatenationFir(elements=[A, B, C])`, not nested binary.

---

## What This Phase Does NOT Do

- **No forward search `~`** — deferred (likely a follow-up phase).
- **No detachment, SF, SFF** — Phase 7.
- **No concatenation-with-non-brane elements** — `5 7` is a compile
  error in this phase; only brane-resolving expressions are valid
  elements.
- **No recoordination beyond what `constanicClone` already provides** —
  the implementation does not need a separate "outer rebind" or
  "resolve again" pass. The standard UBC step cycle handles it.

---

## Phase 3 Exit Criteria

- All 4 concatenation `.foo` tests pass.
- All 4 new dedicated tests pass.
- The Phase 1 compiler successfully compiles `ConcatenationAstn`; the
  `ConcatenationFir` roundtrips through Circe.
- A test specifically demonstrates recoordination producing different
  results than the original (e.g., the `p3_concat_resolution.foo`
  example resolves `a = 42` after concatenation).
- A test demonstrates left-to-right order sensitivity
  (`p3_concat_order_matters.foo`).

---

## Open Questions

- **`StatementFir.origin` debugging field**: should each cloned
  statement carry a record of which input element it came from? Useful
  for debugging concatenation issues. Decide during implementation.
- **Concatenation between branes with characterizations**: `type1'{...}
  type2'{...}` — the merged brane is uncharacterized per FOOP-3.
  Verify users are okay with that; revisit if not.

---

## Last Updated

**Date**: 2026-05-04
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.7 (1M Context)
**Changes**: Promoted concatenation from Phase 6 to Phase 3 (its own
dedicated phase between UBC depth-first and CLI). Rewrote per FOOP-3
revised algorithm (constanicClone'd merge + delegate-to-merged-brane).
Removed forward search and other deferred features from this phase's
scope.
