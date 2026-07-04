---
foop: 9
title: Operators are brane-like FIRs with positional unnamed operands and no search boundary
author: hc <hc.busy@gmail.com>
status: Deprecated
type: Standards
created: 2026-05-04
phase: phase-1
supersedes: []
---

# FOOP-9: Operators are brane-like FIRs with positional unnamed operands and no search boundary

> **Status: Deprecated** (2026-07-03 18:23)
>
> Canceled as it stands. This feature should be later respecified and reimplemented.

## Abstract

Replaces the current `BinaryOpFir(op, left, right)` / `UnaryOpFir(op, expr)`
tree model with a single `OperatorFir(op, operands: List[Fir])` shape.
Operators are structurally analogous to `NormalBraneFir` — a container
holding an ordered list of child FIRs that get stepped left-to-right —
but with two differences:

1. Children are **positional and unnamed** (operator operands have no
   identifier; their meaning is by position, like function arguments).
2. The operator's own self-step, after all children reach CONSTANT, is a
   **scalar reduction** (e.g., `+` reduces two integers to their sum)
   rather than returning the container itself.

Critically, the operator FIR has **no search boundary**. A search inside
an operand searches starting at the operator's *parent* brane, not
inside the operator. This matters most for unanchored seek
(`#-N`) — `b` in `a + b` is the second statement of the enclosing brane,
not the second statement inside `+`.

## Motivation

### Why share children-stepping with branes

Previous Foolish iterations always reused the same children-evaluation
implementation for branes and operators. Every iteration that tried to
keep them separate eventually merged them after the second or third bug
in operator-state propagation. Sharing the implementation has two
concrete payoffs:

1. **One bug fix, two beneficiaries**: a fix to "how do I propagate
   constanic state from children to parent" applies to both branes and
   operators automatically.

2. **One Phase 4 lift**: when we switch from depth-first (Phase 2) to
   breadth-first (Phase 5), only one children-stepping implementation
   needs rewriting. Operators come along for free.

### Why drop BinaryOpFir / UnaryOpFir

The tree model `BinaryOpFir(op, left, right)` is convenient for binary
operators but doesn't generalize:

- N-ary operators (e.g., variadic comparisons, future language additions)
  need a different shape.
- The arity is encoded in the variant name, doubling the FIR algebra.
- Phase 2's step rule for `BinaryOpFir` and `NormalBraneFir` would be
  near-duplicates with one substantive difference (the scalar
  reduction).

A single `OperatorFir(op, operands: List[Fir])` handles unary, binary,
and any future N-ary uniformly. The implementation is one container.

### Why no search boundary

Consider:

```
{
  a = 5,
  b = 7,
  c = a + b
}
```

The `+` is the operator. Inside it, two operands: a search for `a` and a
search for `b`. Where do those searches start?

If `+` were a search boundary (like a brane), the search for `a` would
look inside `+` first — find no `a` — then walk up to `+`'s parent brane
to find `a = 5`. That's wasted work but produces the right answer in
this case.

But consider:

```
{
  x = 5,
  y = 7,
  z = #-2 + #-1
}
```

`#-N` is unanchored backward seek: "the statement N positions before me
in my immediate brane." If `+` were a search boundary, the operand
brane for `+` would be the seek's "immediate brane," and the seek would
look inside `+`'s operands. The `#-1` would find the `#-2` (the previous
operand position). That is **not** what Foolish means by `#-1` in
this context — the user intends `#-1` to find the statement immediately
before `z = ...`.

Making operators **transparent** (no search boundary) means the operand
sees the enclosing brane as its immediate brane. `#-1` finds `y = 7`,
`#-2` finds `x = 5`, the result is `5 + 7 = 12`. This matches user
intent and matches UBC2 d0_2's "no-boundary" classification of System
Operator FIRs.

The general rule: **operators are transparent to search; they pass
through to their parent brane**. This applies to all search forms:
unanchored identifiers, anchored searches, head/tail (`^`/`$`), and
seeks (`#N`, `#-N`).

## Specification

### FIR algebra change

Remove from `Fir.scala`:

```scala
case class BinaryOpFir(op: String, left: Fir, right: Fir, state: Nyes = ...) extends Fir
case class UnaryOpFir(op: String, expr: Fir, state: Nyes = ...) extends Fir
```

Add:

```scala
case class OperatorFir(
  op:       String,         // "+", "-", "*", "/", "%", or unary forms encoded as op + "@unary"
  operands: List[Fir]       // ordered, positional, unnamed
) extends Fir:
  var state:  Nyes        = Nyes.EMBRYONIC
  var parent: Option[Fir] = None
  // No `target` — operators don't dereference; they reduce.
```

(Per FOOP-8: `state` and `parent` are mutable fields.)

The `op` string disambiguates unary from binary by either:
- Storing operands.length (unary = 1, binary = 2) implicitly, OR
- Tagging the op string explicitly (`"+@unary"` vs `"+"`)

Implementation choice; document in `Fir.scala`. The simpler approach is
arity-by-list-length since unary `-` (`UnaryExprAstn`) has exactly one
operand and binary `+` has exactly two.

### Children stepping shared with NormalBraneFir

The implementer should extract a shared trait or helper:

```scala
trait Container extends Fir:
  def children: List[Fir]
  // Step children left-to-right to constanic terminal.
  // Used by both NormalBraneFir (statements as children) and OperatorFir
  // (operands as children).
```

Both `NormalBraneFir.step` and `OperatorFir.step` delegate the
children-stepping pass to this shared logic. Their self-step differs:

- **NormalBraneFir.selfStep**: aggregate child states; brane reaches
  CONSTANT only when all statements are CONSTANT/INDEPENDENT.
- **OperatorFir.selfStep**: when all operands are CONSTANT/INDEPENDENT,
  apply the operator's reduction (e.g., `+` adds the two integer values)
  and produce a `ConstantIntFir(result)` to replace the container as the
  operator FIR's resolved value. State transitions to CONSTANT (the
  reduced value, not the container, is what propagates).

### Search transparency rule

When a search FIR inside an operator's operand subtree walks its scope,
the operator FIR is **skipped**. Specifically:

- The search's "immediate brane" (IB) is determined by walking the
  parent chain UP through any operator FIRs to find the nearest
  containing `NormalBraneFir` (or future ConcatenationBrane,
  DetachmentBrane).
- Anchored searches still resolve against their explicit anchor; the
  transparency rule only affects how IB is identified for unanchored
  searches and seeks.

In code, this is naturally implemented as:

```scala
def immediateBrane(fir: Fir): NormalBraneFir =
  fir.parent match
    case Some(op: OperatorFir)        => immediateBrane(op)
    case Some(brane: NormalBraneFir)  => brane
    case Some(other)                  => immediateBrane(other)  // recurse upward
    case None                         => sys.error("no enclosing brane")
```

### Compiler change (Phase 1 P1.5)

`Compiler.compileExpr` is updated:

```scala
case BinaryExprAstn(op, l, r) => OperatorFir(op,         List(compileExpr(l), compileExpr(r)))
case UnaryExprAstn(op, e)     => OperatorFir(s"$op@unary", List(compileExpr(e)))
```

(Or use the arity-by-length scheme without the `@unary` tag — implementer's
choice.)

### Compile-time / evaluation-time work (FOOP-5 still applies)

The compiler still does NO arithmetic. `1 + 2` compiles to:

```scala
OperatorFir("+", List(
  ConstantIntFir(1),  // INDEPENDENT
  ConstantIntFir(2)   // INDEPENDENT
))  // state = EMBRYONIC
```

Phase 2 evaluator's step on this `OperatorFir`:

1. Step operands: both are already INDEPENDENT, no work.
2. All operands CONSTANT/INDEPENDENT → reduce. Compute `1 + 2 = 3`.
3. The OperatorFir's value becomes `ConstantIntFir(3)`. The container
   transitions to CONSTANT and effectively resolves to its scalar
   value via the standard search-result mechanism (when a search finds
   this OperatorFir's enclosing statement, the search target ultimately
   resolves to the ConstantIntFir(3)).

Implementation detail (deferrable): does the OperatorFir literally
*become* a ConstantIntFir (mutate its identity), or does it stay an
OperatorFir in CONSTANT state with a `result` field holding the
ConstantIntFir? Likely the latter — keeps the FIR algebra simple. The
field name (`result`, `value`, etc.) is for the implementer to pick.

### Why position-based, not name-based

Operands have no names because the operator's semantics give them
position-based meaning (`+`'s left is added to `+`'s right). If we tried
to name them (`+`'s "addend1" and "addend2"), users could accidentally
write `addend1` inside the operand and trigger a search — but for what?
Operands are not statements; they're not searchable. Keeping them
unnamed prevents the confusion.

## FIR Impact

- Remove `BinaryOpFir`, `UnaryOpFir` from `Fir.scala`.
- Add `OperatorFir`.
- Optionally add a `Container` trait abstracting "step my children
  left-to-right then self-resolve."
- Circe codecs adjust to the new variant.
- Roundtrip tests add an `OperatorFir` case.

## UBC Step Impact

`OperatorFir.step` delegates the children-stepping pass to the shared
`Container` logic, then performs the scalar reduction when all operands
are CONSTANT/INDEPENDENT. The state propagation rules for ECONSTANIC /
WOCONSTANIC operands match `NormalBraneFir` (any constanic operand →
operator is WOCONSTANIC; any NK operand → operator is NK).

The search-transparency rule changes how `Ubc.step` for SearchFir
identifies the searcher's immediate brane: walk up through OperatorFirs
to find the nearest NormalBraneFir.

## Test Plan

Phase 1 unit tests (Layer 2 — AST → FIR):

```scala
test("FOOP-9: 1 + 2 compiles to OperatorFir(+, [Const(1), Const(2)])") {
  val ast = BinaryExprAstn("+", IntLitAstn(1), IntLitAstn(2))
  Compiler.compileExpr(ast) shouldBe OperatorFir("+", List(
    ConstantIntFir(1),
    ConstantIntFir(2)
  ))
}

test("FOOP-9: -42 compiles to OperatorFir(-@unary, [Const(42)])") {
  val ast = UnaryExprAstn("-", IntLitAstn(42))
  Compiler.compileExpr(ast) shouldBe OperatorFir("-@unary", List(
    ConstantIntFir(42)
  ))
}
```

Phase 1 unit tests (Layer 3 — Roundtrip):

```scala
test("OperatorFir roundtrips") {
  roundtrip(OperatorFir("+", List(ConstantIntFir(1), ConstantIntFir(2))))
}
```

Phase 2 approval tests (after FOOP-9 is implemented):

- `simpleAdditionIsApproved.foo` etc. — verify arithmetic still works.
- `operatorPrecedenceIsApproved.foo` — confirm precedence is intact
  (handled by parser, not affected by this FOOP).
- New approval test `operatorSearchTransparency.foo`:
  ```
  {
    x = 5,
    y = 7,
    z = #-2 + #-1
  }
  ```
  expected: `z = 12`. If z were 10 (i.e., `#-1` had found `#-2`), the
  search transparency rule is broken.

## Rejected Alternatives

### A. Keep BinaryOpFir / UnaryOpFir

The current design. **Rejected**: doubles the operator FIR variants for
no benefit; makes Phase 4 (breadth-first) require two parallel rewrites
of the children-stepping logic.

### B. Operators are search-bounded (transparent search disabled)

Treat operators as search boundaries — operands' searches start inside
the operator's children. **Rejected**: breaks `#-N` semantics inside
operators (the `#-2 + #-1` case above). Forces every operand to fully
qualify its scope, which is unnatural and verbose.

### C. Compile to UBC2's literal `{🧠1, 🧠2, 🧠+}` brane shape

UBC2 d0_2 desugars `1 + 2` into a brane with three statements: literals
followed by an operator FIR that consumes from sibling positions.
**Rejected for MVP**: requires named operator FIRs (`🧠+`, `🧠-`, etc.)
to be searchable as siblings, which adds a layer of indirection
(operator-as-statement vs operator-as-container). The container model
is closer to what users expect from a parsed expression tree.

This rejection may be revisited if a future Foolish feature requires
operators to be named or searchable.

### D. Use Java-style `apply(args: Any*)` polymorphism

Pretend the operator is a function and the operands are arguments.
**Rejected**: imports function-call semantics that Foolish doesn't have.
Operators are not functions — they're a primitive language construct.
Don't muddy the model.

## Open Questions

- **Operator-as-FIR identity after reduction**: does the reduced
  `OperatorFir` mutate its identity to become a `ConstantIntFir`, or
  does it keep its OperatorFir identity with a result field? The
  implementer picks; document the choice in `Fir.scala`.
- **Future N-ary operators**: when (if) the language adds variadic
  comparisons or other N-ary forms, does `OperatorFir` need an
  arity-validation pass? Defer.

## References

- `scala-mvp/foolish-scala/foolish-core-scala/src/main/scala/org/foolish/fvm/scubc/Fir.scala`:
  the file to update.
- `scala-mvp/foolish-scala/docs/phase1_compiler.md`: P1.5 step needs
  rewrite.
- `scala-mvp/foolish-scala/docs/phase2_ubc.md`: per-FIR step rules need
  updates for OperatorFir; search step rule needs the immediate-brane
  walk-through-operators logic.
- `docs/ubc1/how/d0_2_system_operator.md` (broader docs branch):
  UBC2's no-boundary classification and the desugaring approach we did
  NOT adopt (Rejected Alternative C).
- FOOP-5: compile-time vs evaluation-time work — still applies; the
  compiler still does no arithmetic.
- FOOP-8: FIRs are mutable; `OperatorFir.state` and `OperatorFir.parent`
  are mutable per that FOOP.

---

## Last Updated

**Date**: 2026-07-04
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Status -> Deprecated. Canceled as it stands per user request; feature should be later respecified and reimplemented. Added Deprecation Notice section.
