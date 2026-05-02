---
foop: 3
title: Sequential blocking concatenation
author: hc <hc.busy@gmail.com>
status: Brewing
type: Standards
created: 2026-04-22
phase: phase-5
supersedes: []
---

# FOOP-3: Sequential blocking concatenation

## Abstract

Retroactive: documents a decision made on 2026-04-22.

The Foolish concatenation operator `A B` (two adjacent branes on the same
line) evaluates with **sequential blocking semantics**: A must reach
`Constant` state before B begins evaluation, and B then evaluates with
A prepended to its Ancestral Brane chain. There is no isolation stage,
no cloning, no three-stage protocol. This supersedes the UBC1
three-stage isolate→merge→re-evaluate approach.

Status is `Brewing` (not `Final`) because the implementation does not yet
exist — it is targeted at Phase 5.

## Motivation

UBC1 implemented concatenation with a three-stage protocol:

1. **Isolate**: clone A and B into independent evaluation contexts.
2. **Merge**: combine results.
3. **Re-evaluate**: walk the merged structure resolving cross-references.

This was the source of the WOCONSTANIC state, the cursor cloning
machinery, and a substantial fraction of UBC1's complexity. Repeated
attempts to fix bugs in concatenation introduced more state. The whole
arrangement was an early optimization for parallelism that was never
actually parallel.

The simpler model: A blocks B. If A is `Constanic`, the concatenation is
`Constanic` and we hold the AST. If A is `Constant`, B sees A's
statements as if they were the innermost layer of B's surrounding scope.
B's unanchored searches walk A first, then up the original AB chain.

This eliminates:

- The WOCONSTANIC state (no longer needed)
- The three-stage cursor protocol
- Cross-context cloning
- Most of the "concatenation race" bugs

It retains everything actually needed for concatenation semantics:
B can refer to names defined in A; B's inability to resolve a name
makes it Constanic; A being Constanic makes the whole concatenation
Constanic.

## Specification

### Evaluation rule

```
eval(ConcatenationFir(elements), ab, ib):
  let head = elements.head
  let tail = elements.tail
  if elements.size == 1: eval(head, ab, ib)
  else:
    let aResult = eval(head, ab, ib)
    if aResult.state != Constant:
      ConcatenationFir(elements, state = Constanic)  // hold the AST
    else:
      let bResult = eval(ConcatenationFir(tail), ab = aResult :: ab, ib)
      mergeBranes(aResult, bResult)
```

### Left-associativity

`A B C` parses as `(A B) C`. The rule above naturally produces this
behavior because the recursion processes `head :: tail` left to right.

### Merging

`mergeBranes(a, b)` produces a single brane whose statements are A's
statements followed by B's statements. Names defined in both A and B
follow the standard shadowing rule (B shadows A from B's statement onward;
backward search from after B's statements finds B's definition first).

## FIR Impact

New variant in `Fir.scala`:

```scala
case class ConcatenationFir(
  elements: List[Fir],
  state:    FirState = FirState.Initialized
) extends Fir
```

Roundtrip test required (FIR test layer 3).

The Phase 1 compiler MUST reject `ConcatenationAstn` with a message
naming this FOOP and the target phase (Phase 5).

## UBC Step Impact

New step rule per the algorithm in §Specification.

Interaction with constanic coordination: when A is Constanic on
identifier X, the whole concatenation is Constanic on X. If a later
context (e.g., REPL line) defines X, the concatenation must be
re-stepped. This is the same dependency-tracking machinery any
Constanic FIR needs (see `phase2_ubc.md`).

## Test Plan

Phase 5 will revive these `.foo` files (currently in
`foolish-core-scala/src/test/resources/.../inputs/`):

- `concatenationBasics.foo`
- `concatenationResolution.foo`
- `concatenationSearch.foo`
- `concatenationResolutionAdv.foo`

Plus new tests:

- `p4_5_concat_blocked_left.foo`: A is Constanic → whole concat Constanic
  WITHOUT partial evaluation of B.
- `p4_5_concat_chain.foo`: `A B C` left-associativity.
- `p4_5_concat_late_resolve.foo`: A becomes Constant after later context
  arrives → B can re-evaluate.

## Rejected Alternatives

### A. UBC1's three-stage protocol

Already discussed in Motivation. Source of complexity, no benefit
that simpler model doesn't provide.

### B. Two-stage: isolate-then-merge (no re-evaluation)

Half the complexity of UBC1's three-stage. **Rejected**: still requires
isolation/cloning, which is the expensive part. The "re-evaluation" stage
in UBC1 was actually doing useful work (cross-resolution); removing it
without replacing it produces wrong answers. Better to remove the
isolation entirely (this FOOP) than to try to keep partial machinery.

### C. Parallel evaluation of A and B with later reconciliation

The hypothetical reason UBC1 chose three-stage. **Rejected**: never
actually parallel in UBC1. Even if it were, parallel reconciliation
across constanic dependencies is a hard problem and not justified for
the language's current scale.

## Open Questions

- **Does the merged brane retain a record of where each statement came
  from?** (For debugging, "this statement is from A, this from B".) Likely
  yes via an optional `origin` field on `StatementFir`. To be decided
  during Phase 5 implementation.

- **What happens if the concatenation is between branes with conflicting
  characterizations?** E.g., `type1'{...} type2'{...}`. To be decided.

## References

- `scala-mvp/foolish-scala/docs/phase5_concatenation.md`: the phase
  document this FOOP backs.
- UBC1's `ConcatenationFiroe.scala`: the prior (rejected) approach.
