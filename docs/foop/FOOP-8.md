---
foop: 8
title: FIRs are mutable; parent pointers are post-clone; Circe excludes parent
author: hc <hc.busy@gmail.com>
status: Superseded
type: Standards
created: 2026-05-02
phase: phase-2
supersedes: []
---

# FOOP-8: FIRs are mutable; parent pointers are post-clone; Circe excludes parent

> **Superseded 2026-07-14.** The semantics (FIRs mutable, parents set post-clone, parent
> excluded from serialization) hold, but the mechanism was replaced by UBCa's interior
> mutability (`Cell`/`RefCell` in `ProtoBrane`) + `Weak` parent links (FOOP-62/FOOP-52
> architecture). Early UBC-era text; do not cite for current mechanics.

## Abstract

`Fir` instances are **mutable**. Specifically:

- `state: Nyes` is a mutable field. UBC `step()` mutates it in place.
- `parent: Option[Fir]` is a mutable field. After `constanicClone(R)`
  returns the cloned FIR, the caller assigns `clone.parent = newParent`.
- `target: Option[Fir]` (on `SearchFir`) is mutable, populated by the
  search step.
- Other constanic-bookkeeping fields (TBD by implementation) may also be
  mutable.

**Circe serialization excludes `parent`**. The parent pointer is implicit
in the brane tree's nesting — a deserializer reconstructs parent
references by traversing the tree once after `decode`. JSON FIR documents
remain acyclic.

This FOOP supersedes the implicit assumption from FOOP-5 that all FIR
fields are immutable case-class fields fully covered by Circe generic
derivation.

## Motivation

The recoordination algorithm (FOOP-7) requires that a cloned FIR be
re-parented to its new context. Previous design held the assumption that
FIRs were pure immutable case classes covered by Circe's `deriveEncoder`
/ `deriveDecoder`. That assumption breaks `constanicClone` for two
reasons:

1. **Cycles**: a `parent` reference creates an upward link, breaking the
   tree's acyclicity. Circe generic derivation cannot serialize
   self-referential graphs.
2. **Stepwise mutation**: UBC `step()` advances FIRs through the Nyes
   lifecycle. Returning a new immutable FIR from each step would force
   the entire ancestor chain to be reconstructed for every change, which
   is impractical.

The alternative — threading parent context through every `step()` call —
is verbose and pollutes every step rule's signature with scope-chain
plumbing that's never used by most rules. UBC2's reference design uses
explicit parent back-pointers for the same reason.

Trade-off accepted: lose Circe generic derivation simplicity; gain
implementation tractability for recoordination and stepwise evaluation.

## Specification

### Mutable fields

```scala
sealed trait Fir:
  var state:  Nyes
  var parent: Option[Fir] = None
```

Each variant declares additional mutable fields as needed. Examples:

```scala
case class SearchFir(
  pattern:   String,
  direction: SearchDirection,
  anchored:  Boolean
) extends Fir:
  var state:  Nyes        = Nyes.EMBRYONIC
  var parent: Option[Fir] = None
  var target: Option[Fir] = None
  // anchor field (for anchored searches) similarly mutable if needed
```

Implementation detail: the exact split between constructor parameters
and mutable fields is up to the Phase 2 implementer. The constraint is
that `state`, `parent`, and `target` (where applicable) MUST be mutable.

### Parent assignment after constanicClone

```scala
// At every search-step site:
val cloned = constanicClone(rawSearchResult)
cloned.parent = Some(this.parentBrane)   // or whatever the appropriate parent is
this.target   = Some(cloned)
```

The `cloned.parent = ...` assignment is part of the calling protocol.
`constanicClone` itself does not set `parent`; it returns the bare clone
and the caller wires up the parent.

### Circe serialization

`parent` is excluded from JSON. Two approaches are acceptable:

**Approach 1**: custom Circe codec that omits `parent` on encode and
defaults it to `None` on decode. After decode, the consumer walks the
tree once to populate parent pointers from the structural nesting.

**Approach 2**: use Circe's `@JsonExclude`-equivalent or define field
codecs that explicitly skip `parent`.

The implementer chooses; the JSON contract is what matters.

### Equality

With mutable fields, default case-class `equals` becomes unreliable
(two FIRs with the same structure but different mutation history would
compare equal one moment and not the next). Recommendations:

- Override `equals` and `hashCode` to compare only structural fields
  (pattern, direction, characterizations, etc.) plus `state`.
- OR: do not rely on `==` for FIR comparison; use a dedicated
  `structurallyEquivalent` method for tests.

Implementation choice; document in `Fir.scala`.

### Roundtrip test invariant

Roundtrip tests (FirRoundtripTest) must compare structurally, not by
default equals. Suggested form:

```scala
def roundtrip(fir: Fir): Unit =
  val json    = fir.asJson.noSpaces
  val decoded = decode[Fir](json).getOrElse(fail("decode failed"))
  decoded.structurallyEquivalent(fir) shouldBe true
```

After roundtrip, parent pointers on `decoded` are absent (None);
parent pointers on the original FIR may also be None (if it was never
attached) or Some (if it was). The structural equivalence check must
ignore parent.

## FIR Impact

`Fir.scala` must be rewritten to use mutable fields. Existing case-class
defaults (`state: Nyes = Nyes.EMBRYONIC` as a constructor parameter)
become `var state: Nyes = Nyes.EMBRYONIC` in the body.

The Circe codec must exclude parent. The `FirRoundtripTest` must use
structural equivalence.

## UBC Step Impact

`step()` mutates the FIR in place. Step rules return the same FIR
instance, possibly with updated `state` / `target` / etc. There is no
"return a new FIR" step pattern.

## Test Plan

- Existing `FirRoundtripTest` continues to pass after the mutability
  refactor. Tests are updated to use `structurallyEquivalent`.
- A new test asserts `parent` is excluded from JSON: encode a FIR with
  parent set, decode, parent should be None.
- A new test asserts `step()` mutates in place: `val fir = ...; val
  before = fir; Ubc.step(fir); fir should be theSameInstanceAs before`.

## Rejected Alternatives

### A. Keep FIRs immutable; thread context through `step()`

```scala
def step(fir: Fir, ancestralBrane: List[Fir]): Fir
```

Every step rule signature carries `ancestralBrane`. The function returns
a new FIR per call, requiring the caller to thread it back into the
parent's structure.

**Rejected**: pollutes every step rule with scope plumbing; forces
parent-of-parent reconstruction on every step; multiplies allocations.
The mutable approach is what UBC2 reference designs use and what the
implementation effort can actually carry.

### B. Wrap each FIR with a context object

```scala
case class FirInContext(fir: Fir, parent: Option[FirInContext], scope: List[Fir])
```

Wrapping preserves immutability of `Fir` itself. **Rejected**: the
wrapper structure must be reconstructed on every step, and the
relationship between a wrapper and its `fir` is just "another mutable
back-pointer in disguise" — solving nothing. Adds an indirection layer.

### C. Mutable parent only; keep state immutable

Halfway design. **Rejected**: `state` is the field that changes most
often during stepping; making everything else mutable but `state`
forces step rules to construct a new FIR each call anyway, defeating
the simplification.

### D. Reference cells (Scala `Ref` / `AtomicReference`)

Wrap mutable fields in cells. **Rejected**: pure-FP overkill for an MVP;
Phase 2 is single-threaded depth-first; the cell wrappers add ceremony
without buying anything until Phase 4 introduces concurrency (and even
there, the cell decision is best made at that time, not now).

## Open Questions

- **Parent pointer cycle detection**: what guards prevent a malformed FIR
  tree (where the parent chain loops) from causing infinite scope walks?
  Possibly handled by depth limiting (FOOP-?). Defer.
- **Equals/hashCode policy**: structural-only, structural-plus-state, or
  a separate `structurallyEquivalent` method. Defer to implementation.

## References

- `scala-mvp/foolish-scala/foolish-core-scala/src/main/scala/org/foolish/fvm/scubc/Fir.scala`:
  the file to rewrite.
- FOOP-5: the original "FIRs are immutable case classes" assumption,
  now relaxed.
- FOOP-7: the recoordination contract that requires post-clone parent
  assignment.
- d0_5: UBC2 reference design also uses parent back-pointers.
