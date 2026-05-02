---
foop: 4
title: Bare identifiers compile to anchored regex SearchFirs
author: hc <hc.busy@gmail.com>
status: Final
type: Standards
created: 2026-05-01
phase: phase-1
supersedes: []
implementation: Compiler.compileExpr step P1.6 in phase1_compiler.md
---

# FOOP-4: Bare identifiers compile to anchored regex SearchFirs

## Abstract

Retroactive: documents a decision made on 2026-05-01.

When the compiler encounters a bare identifier such as `a_config`, it
produces a `SearchFir` with a regex pattern of `^a_config$`, not a
plain-name search. This means:

- The `SearchFir` schema has only one search mode (regex), not two.
- Phase 2's evaluator has only one search-matching path to implement.
- A search engine reading the FIR JSON does not need to branch on "is
  this a name or a pattern" — it always runs a regex match.

## Motivation

In UBC1, name searches and regex searches were two different code paths:
`NameSearchFiroe` and `RegexpSearchFiroe`. Each had its own caching, its
own canonicalization, and its own bugs around shadowing and Unicode
identifier separators.

A bare identifier can be expressed as a fully-anchored regex:
`a_config` is exactly the regex `^a_config$`. The compilation step does
this conversion once. The runtime sees only one kind of search.

Benefits:

- **Single code path** in the evaluator → one set of bugs to find, one
  cache to maintain, one set of tests to write.
- **No "is this a name or pattern" branching** in serialized FIR readers.
- **No information loss**: the regex `^name$` is precisely equivalent to
  an exact name match.

Cost: a small amount of extra regex compilation work at evaluator startup.
At Foolish's scale, negligible.

## Specification

### Compiler rule

`Compiler.compileExpr(IdentifierAstn(Nil, id))` produces:

```scala
SearchFir(
  pattern   = "^" + id + "$",
  direction = SearchDirection.Backward,
  anchored  = false,
  anchor    = None,
  state     = FirState.Initialized
)
```

The identifier `id` has already been canonicalized by `ASTBuilder` (the
three separator forms — `_`, `ˍ`, narrow no-break space — are all
normalized to `ˍ`). The compiler does NOT re-canonicalize and does NOT
escape regex metacharacters: Foolish identifiers cannot contain regex
metacharacters by grammar (the lexer's `IDENTIFIER` rule is letters +
digits + the three separators, none of which are regex special).

### Anchored case

`Compiler.compileExpr(DotSearchAstn(anchor, name))` produces:

```scala
SearchFir(
  pattern   = "^" + name.id + "$",
  direction = SearchDirection.Backward,
  anchored  = true,
  anchor    = Some(compileExpr(anchor)),
  state     = FirState.Initialized
)
```

### Characterized identifiers

`type'name` does NOT use `SearchFir` because the characterization carries
semantics beyond name matching (it's a type-like qualifier). It uses
`CharacterizedRefFir` instead. See `Fir.scala`.

### Already-regex searches

`RegexSearchAstn(anchor, REGEXP_LOCAL, pattern)` produces a `SearchFir`
with the user-supplied pattern and `anchored = true`. The user is
responsible for any anchoring they want.

## FIR Impact

`SearchFir` already exists in `Fir.scala` (introduced in the Phase 1
scaffold). This FOOP fixes the field semantics:

- `pattern: String` is always a regex
- Bare identifiers and dot-access produce `^name$`
- Regex searches (`?pat`, `~pat`) produce the user-supplied pattern as-is

## UBC Step Impact

The Phase 2 evaluator runs `Pattern.compile(fir.pattern).matcher(name).matches()`
to check each candidate statement name. There is no separate "exact match"
fast path. Cache the compiled `Pattern` per `SearchFir` instance.

## Test Plan

Phase 1 unit tests in `CompilerTest.scala`:

```scala
test("FOOP-4: bare identifier compiles to anchored backward search regex") {
  val ast = IdentifierAstn(Nil, "aˍconfig")
  Compiler.compileExpr(ast) shouldBe SearchFir(
    pattern   = "^aˍconfig$",
    direction = SearchDirection.Backward,
    anchored  = false,
    anchor    = None
  )
}

test("FOOP-4: dot access compiles to anchored backward search regex") {
  val ast = DotSearchAstn(IdentifierAstn(Nil, "b"), IdentifierAstn(Nil, "x"))
  // ... shouldBe SearchFir("^x$", Backward, anchored = true, Some(SearchFir("^b$", ...)))
}
```

Phase 2 will add approval tests covering identifier search semantics; they
exercise the unified regex path.

## Rejected Alternatives

### A. Two-mode `SearchFir` with a `mode: Exact | Regex` field

The straightforward design. **Rejected**: forces the evaluator to branch
on every search. Doubles the test surface for "did the right path get
hit?" Provides no observable benefit over uniformly using regex.

### B. Two FIR variants: `NameSearchFir` and `RegexSearchFir`

Even more explicit. **Rejected**: same problem as A but worse — now JSON
deserializers must branch too, and refactoring affects both variants
in lockstep.

### C. Lazy compilation: store the bare name, compile to regex at evaluation
time

Saves a few bytes of JSON. **Rejected**: pushes the conversion logic out
of the (well-tested) compiler into the evaluator, where it has to be
re-run for every search. Wrong direction.

## Open Questions

None. Implementation directly follows from the rule in §Specification.

## References

- `scala-mvp/foolish-scala/docs/phase1_compiler.md`: P1.6 (bare
  identifier) and P1.8 (anchored search).
- `scala-mvp/foolish-scala/foolish-core-scala/src/main/scala/org/foolish/fvm/scubc/Fir.scala`:
  `SearchFir` definition.
- FOOP-2: another simplification (one fewer code path).
