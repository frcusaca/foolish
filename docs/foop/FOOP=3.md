---
foop: 3
title: Concatenation produces a new brane of constanicCloned elements; further steps delegate to the merged brane
author: hc <hc.busy@gmail.com>
status: Brewing
type: Standards
created: 2026-04-22
phase: phase-3
supersedes: []
---

# FOOP=3: Concatenation produces a new brane of constanicCloned elements; further steps delegate to the merged brane

## Abstract

The Foolish concatenation operator `A B C ...` is represented by a
`ConcatenationFir` holding a **list** of element FIRs (not a binary
tree of pair-wise concatenations).

When the `ConcatenationFir` steps:

1. **Operands are constanic**: by the time concatenation acts, each
   element FIR has already reached a constanic state (CONSTANT,
   INDEPENDENT, ECONSTANIC, or WOCONSTANIC). Implementations may have
   stepped them already or wait depending on driver — Phase 2's
   depth-first ordering ensures it's already done.

2. **Construct the merged brane**: produce a new `NormalBraneFir` whose
   statements are `constanicClone`'d copies of each input element's
   statements, in concatenation order (left-to-right).

3. **Delegate further `step()`s to the merged brane**: the
   `ConcatenationFir` effectively *becomes* the merged brane for
   subsequent stepping. The merged brane is installed at the
   `ConcatenationFir`'s position in the parent. From this point on, the
   merged brane's normal step lifecycle handles all internal
   resolution; constanic clones from one element re-search in the
   merged context, possibly finding names provided by another element.

There is no separate "re-evaluation pass" or "outer rebind" — UBC's
ordinary `step()` cycle, applied to the merged brane, handles
everything.

This supersedes UBC1's three-stage isolate→merge→re-evaluate protocol
and the prior version of FOOP=3 that described "sequential blocking
where A blocks B." The new framing is closer to what UBC2's d0_3
intends, mediated by `constanicClone` (FOOP=7).

## Motivation

### Why a list, not a tree

`A B C` is a single concatenation of three elements, not a binary tree
`(A B) C`. The list shape is operationally what concatenation does —
the result is one brane built from N input branes' statements. A binary
tree representation would force re-evaluation of the inner pair before
the outer one acts, which has no semantic meaning and complicates
constanicCloning.

The compiler must produce `ConcatenationFir(elements: List[Fir])` from
the source `A B C`, regardless of how the AST might nest the
concatenation expressions.

### Why operands are constanic before merging

If we tried to merge while operands were still nigh, the merge would
need to wait somewhere — either inside the merge function (queuing
work) or outside (deferring the whole concatenation). Both are
machinery we don't want in Phase 3.

By requiring operands to be constanic first, the merge step is a pure
structural operation. Phase 2's depth-first ordering makes this
trivially true (every left sibling is at terminal before the right one
starts). Phase 5 (breadth-first) will need to enforce this contract via
its driver.

### Why constanicClone the elements before merging

Each element brane was constructed in its original parent context and
may contain constanic search FIRs that didn't resolve there. Once the
element is placed in the merged brane, those searches need a chance to
re-resolve in the new context (which now includes statements from
sibling elements).

`constanicClone` (FOOP=7) is exactly the mechanism for this: every
constanic descendant gets a fresh re-stepping opportunity in the new
context. If we shared the elements by reference instead of cloning,
re-resolution would mutate the originals, which is wrong (the originals
exist independently at their own source locations).

### Why delegate further steps to the merged brane

Once the merged brane is constructed, the `ConcatenationFir` has no
further work to do. It's not a permanent FIR shape — it's a recipe
for "take these branes and merge them." After the recipe runs, what
remains is just a brane. Subsequent steps go to that brane.

Implementation-wise, this can be done by either:

- Mutating the `ConcatenationFir` in place to hold the merged brane and
  forwarding `step()` calls, OR
- Replacing the `ConcatenationFir` in its parent's statement list with
  the merged brane.

Either is acceptable per FOOP=8 (FIRs are mutable). Implementer's
choice.

### What this leaves out

This FOOP does not specify how SF (`<expr>`) and SFF (`<<expr>>`)
markers interact with concatenation. SF/SFF markers are deferred to
Phase 7 (FOOP for them not yet written). When concatenation is
inside SF/SFF marks, the marks may suppress or alter the
constanicCloning behavior; that's a Phase 7 concern.

## Specification

### FIR shape

```scala
case class ConcatenationFir(
  elements: List[Fir]    // ordered left-to-right; each element should be a NormalBraneFir or a search resolving to one
) extends Fir:
  var state:  Nyes        = Nyes.EMBRYONIC
  var parent: Option[Fir] = None
```

### Step rule

```
step(ConcatenationFir(elements)):
  // Precondition: each element has stepped to constanic terminal.
  // (Phase 2 depth-first guarantees this.)

  if any(elements).state == NK:
    state' = NK; return

  // Construct merged brane: each element's statements, constanicClone'd, in order.
  let mergedStatements = elements.flatMap { elem =>
    let resolvedBrane = derefToBrane(elem)   // follow search chains to the brane FIR
    resolvedBrane.statements.map(stmt => constanicClone(stmt))
  }
  let mergedBrane = NormalBraneFir(characterizations = Nil, statements = mergedStatements)
  mergedBrane.parent = this.parent

  // Set parent on each cloned statement to the merged brane.
  mergedStatements.foreach { stmt => stmt.parent = Some(mergedBrane) }

  // Delegate further steps to mergedBrane: either mutate self to hold it, or
  // replace self in parent. Implementer's choice.
  this.becomeMergedBrane(mergedBrane)
  state' = mergedBrane.state    // typically EMBRYONIC, ready for ordinary stepping
```

After this step, ordinary brane stepping (per `phase2_ubc.md`) takes
over. Constanic clones inside the merged brane re-step in the merged
context. Searches that were ECONSTANIC in their original context may
now find their target. This is recoordination, automatic via the
standard step cycle.

### Element types

For Phase 3 MVP, `elements` is restricted to expressions that resolve
to NormalBraneFirs. Concatenating non-brane FIRs (e.g., `5 7`) is a
compile error; the compiler rejects such cases.

The expression `g f` may have `g` and `f` be either literal brane
expressions (`{...}`) or search FIRs that point at brane FIRs. The
step rule's `derefToBrane` helper follows search chains to the
underlying brane.

### Order matters

Concatenation is **left-to-right**: `A B C` produces a merged brane
whose statements are A's first, then B's, then C's. This matters for
Foolish's backward-search semantics: a search for `x` from a statement
in C's region of the merged brane finds `x` in A or B (if defined
there) but not in any source after C.

This means `A B` and `B A` are NOT equivalent — they produce different
merged orderings. This is intentional and matches Foolish's
positional semantics.

### Characterizations of the merged brane

The merged brane is **uncharacterized** (`characterizations = Nil`). The
input elements may have their own characterizations on their respective
brane FIRs, but the merged result is a new brane with no inherited
characterization. If users need a characterized concatenation, they
write `type'(A B)` (or whatever the future syntax permits).

### Why this is its own phase

Concatenation is being promoted from a deferred Phase 6 feature to a
dedicated Phase 3 (between UBC depth-first and CLI) because:

1. It's the first real exercise of `constanicClone` (FOOP=7) across
   actual context changes — without it, the recoordination machinery is
   only theoretically tested.
2. It's the first feature where Phase 2's depth-first guarantee
   (operands stepped before they're consumed) is operationally
   important.
3. The CLI is more interesting with concatenation working —
   composition is a major reason users would want a CLI.

## FIR Impact

New variant `ConcatenationFir` per the shape above. Roundtrip test
required (FIR test layer 3).

The Phase 1 compiler is updated: `ConcatenationAstn` now compiles to
`ConcatenationFir(elements.map(compileExpr))` instead of being rejected.
Phase 1's P1.11 list (compile-time rejection) removes
`ConcatenationAstn`. Phase 2's deferred-feature list correspondingly
removes it.

## UBC Step Impact

New step rule per the algorithm above. The rule is structurally simple:
construct the merged brane, delegate further stepping to it.

Interaction with constanic coordination: by relying on `constanicClone`
(which is invoked by the merge step on every cloned statement), the
recoordination work is uniform with all other recoordination in the
system. No special concatenation-specific machinery is needed.

## Test Plan

Phase 3 approval tests will revive these `.foo` files (currently in
`foolish-core-scala/src/test/resources/.../inputs/`):

- `concatenationBasics.foo`
- `concatenationResolution.foo`
- `concatenationSearch.foo`
- `concatenationResolutionAdv.foo`

Plus new dedicated tests:

- **p3_concat_simple.foo**: `{a={x=1, y=2}, b={z=3}, c = a b}` →
  c has merged brane `{x=1, y=2, z=3}`.
- **p3_concat_resolution.foo**: `{f = {a = x}, g = {x = 42}, h = g f}` →
  h has merged brane `{x=42, a=42}`. The constanic `a = x` in f, after
  being cloned into the merged brane, finds `x = 42` from g (which
  appears earlier in the merged statement list).
- **p3_concat_order_matters.foo**: same elements as above but
  `h = f g` — merged order is f's statements then g's. The clone of
  `a = x` is now positioned BEFORE `x = 42`, so backward search from `a`
  doesn't find x. h has merged brane `{a=ECONSTANIC, x=42}`. Document
  this as expected behavior.
- **p3_concat_chain.foo**: `A B C` produces a single 3-element merged
  brane (not a nested binary tree).

## Rejected Alternatives

### A. UBC1's three-stage isolate→merge→re-evaluate protocol

The original UBC1 design. **Rejected**: source of the
WOCONSTANIC-during-merge race condition (a transient state where the
merged brane was partly evaluated while children were still resolving),
the cursor cloning machinery, and a substantial fraction of UBC1's
complexity. Repeated attempts to fix bugs introduced more state. The
whole arrangement was an early optimization for parallelism that was
never actually parallel.

### B. The previous version of this FOOP — "A blocks B; B sees A as innermost AB"

Asymmetric sequential blocking. **Rejected after review** (this FOOP
revision): forced step-rule branching between "A is constanic" (block)
and "A is constant" (evaluate B with A as AB), and didn't actually
match what the language wanted. The current model — operands are all
constanic before merging, then the merge produces a fresh brane and
delegates — is closer to UBC2 d0_3 intent.

### C. Two-stage: isolate-then-merge (no recoordination)

Skip the constanicClone step; just merge by reference. **Rejected**:
breaks recoordination semantics. Constanic searches from element A
would not get the chance to re-resolve in the merged context, so
`{f={a=x}, g={x=42}, h=g f}` would leave `a` as ECONSTANIC even though
the merged brane contains `x = 42`.

### D. Parallel evaluation of elements with later reconciliation

The hypothetical reason UBC1 chose three-stage. **Rejected**: never
actually parallel in UBC1. Even if it were, parallel reconciliation
across constanic dependencies is a hard problem and not justified for
the language's current scale.

## Open Questions

- **Does the merged brane retain a record of where each statement came
  from?** (For debugging, "this statement is from A, this from B".) Likely
  via an optional `origin` field on `StatementFir`. To be decided during
  Phase 3 implementation.

- **Concatenation between branes with characterizations**: e.g.,
  `type1'{...} type2'{...}`. The merged brane is uncharacterized per
  this FOOP. Verify this is what users want; revisit if not.

- **Concatenation inside SF/SFF marks**: out of scope. SF/SFF will get
  their own FOOP when Phase 7 is designed; that FOOP will specify
  whether marks suppress or alter concatenation's behavior.

## References

- `scala-mvp/foolish-scala/docs/phase3_concatenation.md`: the phase
  document this FOOP backs.
- UBC1's `ConcatenationFiroe.scala`: the prior (rejected) approach.
- FOOP=7: `constanicClone` is the mechanism that handles recoordination
  during the merge.
- FOOP=8: FIR mutability — the `becomeMergedBrane` operation requires
  in-place mutation or parent-list replacement.
- `docs/ubc1/how/d0_3_concatenation.md` (broader docs branch): UBC2's
  intent that this FOOP approximates.
