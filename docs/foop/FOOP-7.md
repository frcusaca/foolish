---
foop: 7
title: Constanic Clone — recoordination contract
author: hc <hc.busy@gmail.com>
status: Brewing
type: Standards
created: 2026-05-01
phase: phase-2
supersedes: []
---

# FOOP-7: Constanic Clone — recoordination contract

## Abstract

Defines the **calling contract** for `constanicClone(R)` — the function
invoked at every search-result attachment point. The contract is:

> **Every search result is `constanicClone`'d before being assigned to the
> Search FIR's result field. UBC stepping, applied iteratively, takes care
> of all subsequent state transitions.**

The internal mechanics of `constanicClone` per Nyes state (CONSTANT shared,
ECONSTANIC cloned-and-reset, WOCONSTANIC cloned-with-recursive-children,
etc.) follow the rough idea of UBC2 d0_5. Specifying the multi-step state
transition cascade in prose is impractical — the language is operational,
not declarative. The implementation must be guided by the contract above
and validated by approval tests, not by attempting to predict each step's
intermediate state in English.

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
  sharing is doubly correct.
- NK is terminal; recoordination cannot rescue it.

Only ECONSTANIC and WOCONSTANIC need actual structural clones.
ECONSTANIC clones re-attempt their search in the new context.
WOCONSTANIC clones recursively recoordinate their constanic children.

The d0_5 doc in the broader docs branch describes the original UBC2
algorithm. This FOOP adopts its rough shape but **does not attempt to
prescribe the exact step sequence**, because:

1. The state-transition cascade involves multiple FIRs across multiple
   UBC steps; tracing it in prose loses precision.
2. The sub-cases (WOCONSTANIC search vs WOCONSTANIC expression vs
   WOCONSTANIC brane) interact with the dereferencing chain in ways that
   are easier to implement and test than to describe.
3. The implementer's job is to satisfy the calling contract and the
   approval tests, not to mechanically translate this FOOP into code.

## Specification

### The contract

```scala
// Pseudocode at the search-step site:
val rawResult = scopeWalk(this.pattern, this.parentBrane)
rawResult match
  case None        => this.state = Nyes.ECONSTANIC
  case Some(found) =>
    // CONTRACT: constanicClone is called on every search result.
    // Never assign a raw search result directly to .target.
    this.target = constanicClone(found)
    // The next UBC step will propagate state from this.target into this.
```

The Search FIR's `target` field is **always** the output of `constanicClone`,
never the raw search result.

### The function

```scala
def constanicClone(original: Fir): Fir =
  // Dispatches on original.state. Roughly per UBC2 d0_5:
  //
  //   CONSTANT, INDEPENDENT, NK -> return original (share, do not clone)
  //   ECONSTANIC                -> deep copy, reset to EMBRYONIC
  //   WOCONSTANIC               -> deep copy with recursively-cloned
  //                                constanic children, reset to BRANING
  //   nigh states               -> caller bug, throw
  //
  // The clone's parent pointer is set by the CALLER after this function
  // returns (FIRs are mutable; see FOOP-8 for the parent-pointer
  // representation decision).
  //
  // Detailed multi-step state transitions are implementation-defined.
  // Validate against approval tests, not against prose.
  ???
```

### Caller invariant

Callers MUST step the search result FIR to a constanic terminal state
(or NK) before invoking `constanicClone`. Phase 2's depth-first ordering
makes this trivial: by the time statement N is being stepped, statements
0..N-1 have all completed.

### Per-state intent (rough)

| `original.state` | Intent | Refer to |
|---|---|---|
| CONSTANT | share reference | UBC2 d0_5 "CONSTANT references not cloned" |
| INDEPENDENT | share reference | per FOOP-5 (literals are recoordination-immune) |
| NK | share reference | terminal |
| ECONSTANIC | clone, reset to EMBRYONIC | re-runs search in new context |
| WOCONSTANIC | clone with recursively-cloned constanic children, reset to BRANING | re-steps to recurse into children |
| PREMBRYONIC, EMBRYONIC, BRANING | error (caller bug) | violate caller invariant |

The "intent" column is a guide for the implementer. The exact behavior is
defined by what makes the approval tests pass.

## FIR Impact

`Fir` instances are mutable (per FOOP-8). After `constanicClone` returns,
the caller assigns `.parent` on the returned FIR. The clone's `target`
(if it's a SearchFir) and `state` may continue to mutate as UBC steps
proceed.

Circe serialization excludes `parent` (the parent pointer is not part of
the JSON contract; on deserialization, the consumer re-establishes
parent pointers by traversing the brane tree).

## UBC Step Impact

`constanicClone` is invoked at every site where a search resolves to a
result. See `phase2_ubc.md` per-FIR step rules. The function does NOT
itself perform any stepping — it produces a fresh structural FIR ready
for the UBC driver to step.

## Test Plan

The contract is verified by approval tests. Specifically:

- The worked example in `phase2_ubc.md` (`{y=z, x=y, w=x, v=w+x, u=v+w}`)
  produces the documented final-state table.
- Phase 6 (concatenation) approval tests exercise actual context changes
  — `f = {a=x}; g = {x=42}; h = g f` produces the expected merged result.
- A unit test asserts the calling-site invariant: every Search FIR's
  `target`, after stepping completes, is the output of `constanicClone`
  (verified by post-condition: `target` is never `==` to the raw search
  result for non-CONSTANT/INDEPENDENT/NK results).

Per-state unit tests of `constanicClone` itself are valuable but
**secondary** to the approval-test validation, because the function's
contract is "satisfy the approval tests," not "produce a specific FIR
shape per state."

## Rejected Alternatives

### A. Prescribe the exact multi-step state transition cascade in this FOOP

Tempting because it would make the algorithm look "complete." **Rejected**:
the cascade involves multiple FIRs across multiple UBC steps. Tracing it
in prose loses precision. Foolish exists because English is unsuitable
for this kind of operational specification — let the implementation
speak for itself, validated by approval tests.

### B. Always clone, even CONSTANT

Simplest possible algorithm: every search result becomes a fresh clone.
**Rejected**: O(n) memory blowup on programs with many references to the
same constant. The CONSTANT/INDEPENDENT case being a no-op is an
important optimization that costs essentially nothing in code complexity.

### C. Never clone in Phase 2; defer all cloning to Phase 6 (concatenation)

Saves implementation work in Phase 2. **Rejected**: makes
`constanicClone` an isolated Phase 6 special case rather than a uniform
operation. Forces Phase 6 to retroactively trace every search target in
the FIR tree to clone them, which is more complex than the uniform
"clone at search resolution" rule.

### D. Use a wake-up queue instead of cloning

Don't clone constanic results; instead, when a context change happens,
walk the FIR tree finding constanic FIRs and re-stepping them in place.
**Rejected**: requires the wake-up queue + dependency map machinery that
Phase 2 explicitly defers (FOOP-6). Also fights Foolish's semantics —
each searcher should resolve in *its own* context, so each needs its own
clone, not a shared mutating original.

## Open Questions

- **Equals/hashCode for FIRs**: with `parent` excluded from comparison,
  case classes' default `equals` may need overriding to also exclude
  state fields that change after clone (or `equals` may be redefined
  entirely for FIR comparison purposes). Defer to implementation.

## References

- `scala-mvp/foolish-scala/docs/phase2_ubc.md`: where `constanicClone` is
  invoked (per-FIR step rules).
- `scala-mvp/foolish-scala/docs/00_accumulated_specs.md`: Nyes lifecycle.
- `docs/ubc1/how/d0_5_brane_recoordination.md` (broader docs branch): the
  originating UBC2 design. Use as a reference for intent, not as a
  prescriptive specification.
- FOOP-3: the WOCONSTANIC-during-merge race in concatenation was
  eliminated separately; the WOCONSTANIC state itself remains.
- FOOP-6: depth-first ordering makes the caller invariant trivial in
  Phase 2.
- FOOP-8 (planned): FIR mutability and parent-pointer representation.
