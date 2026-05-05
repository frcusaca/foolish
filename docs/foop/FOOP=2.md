---
foop: 2
title: Remove if-then-else from the language
author: hc <hc.busy@gmail.com>
status: Final
type: Standards
created: 2026-04-15
phase: phase-1
supersedes: []
implementation: scala-mvp/foolish-scala compiler rejects AST.IfExpr
---

# FOOP=2: Remove if-then-else from the language

## Abstract

Retroactive: documents a decision made on 2026-04-15.

The Foolish language removes the `if expr then expr else expr fi`
construct that existed in the UBC1 grammar. Conditional selection is
expressed through search semantics instead — a search returns the first
matching brane, and that act of selection IS the conditional. The grammar
keeps the `if`/`then`/`else`/`fi` tokens for now (less churn) but the Phase 1
compiler rejects any `AST.IfExpr` it encounters.

## Motivation

In UBC1, `IfFiroe` was the source of an infinite-recursion bug class. The
condition expression's evaluation could re-enter the same brane being
evaluated, producing cycles. Each fix added a counter or a depth check and
moved the bug elsewhere.

More fundamentally, `if-then-else` duplicates a capability that search already
provides. In Foolish, `{a={result=42}, b={result=0}}.a.result` is a
conditional selection: navigation into `a` selects the brane whose result
to use. The right way to compute "if X then Y else Z" in Foolish is to
write a brane with two named statements and search for the right one.

Removing the construct:

- Eliminates the recursion bug class.
- Removes a feature whose semantics overlap a more primitive feature.
- Reduces the surface area of Phase 2's evaluator.

## Specification

### Grammar

The grammar tokens (`if`, `then`, `elif`, `else`, `fi`) and the `ifExpr`
rule remain in `Foolish.g4`. Removing them would be churn; keeping them
parseable lets us produce a clean error message instead of a confusing
parser failure.

### Compiler behavior

`Compiler.compileExpr(NotImplementedAstn(reason))` is called for any
`AST.IfExpr` (the Java→Scala AST conversion in `FoolishAst.fromJava`
produces `NotImplementedAstn("if-then-else removed in UBC2")` for these
nodes).

The compiler MUST throw a `CompilationError` with the message:

> "if-then-else has been removed from Foolish (FOOP=2). Use a brane with
> named statements and search-based selection instead."

### Future grammar removal

A later FOOP MAY remove the tokens from the grammar. This is not done in
Phase 1 because it would require regenerating the parser and provides no
functional benefit.

## FIR Impact

None. There is no `IfFir`. The compiler rejects `IfExpr` before any FIR
is constructed.

## UBC Step Impact

None. The evaluator never sees an `IfFir`.

## Test Plan

A unit test in `CompilerTest`:

```scala
test("rejects if-then-else with FOOP=2 message") {
  val source = "{ if 1 then 2 else 3 fi }"
  val ex = intercept[CompilationError](Compiler.compileToJson(source))
  ex.getMessage should include("FOOP=2")
}
```

## Rejected Alternatives

### A. Keep `if-then-else` and fix the recursion bug properly

Considered. **Rejected**: even with the recursion fixed, the construct
duplicates search-based selection. Two ways to write the same thing leads
to inconsistency in user code.

### B. Remove the tokens from the grammar entirely

Considered. **Rejected for Phase 1**: requires touching `Foolish.g4`,
regenerating the parser, and updating `ASTBuilder.java`. Provides no
functional benefit over rejecting at compile time. May be revisited in
a later phase.

### C. Replace `if-then-else` with `cond` (a Lisp-style multi-clause form)

Considered. **Rejected**: same fundamental issue (alternative path through
the language for what search already does). If a multi-clause selector is
ever needed, write a brane with named statements and search by name —
that IS the multi-clause form.

## Open Questions

None. This was a closed decision before this FOOP was written.

## References

- Original UBC2 design notes (in this docs branch) record the removal
  decision; this FOOP formalizes it.
- `scala-mvp/foolish-scala/foolish-core-scala/src/main/scala/org/foolish/fvm/scubc/FoolishAst.scala`:
  the `case _: AST.IfExpr => NotImplementedAstn(...)` line.
