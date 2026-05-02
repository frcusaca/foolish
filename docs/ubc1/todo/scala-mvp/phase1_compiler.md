# Phase 1 — Compiler: Source to FIR JSON

> Goal: Parse `.foo` source, translate AST to Foolish Internal Representation (FIR),
> serialize as JSON via Circe. **No evaluation.** Every FIR comes out in
> `EMBRYONIC` state (the Nyes lifecycle name for "freshly initialized, not yet
> stepped"), except integer literals (compile to `INDEPENDENT`) and `???`
> (compiles to `NK`).

---

## Phase 1 Deliverable

A `Compiler.compileToJson(source: String): String` function that:

1. Parses Foolish source via the existing ANTLR grammar
2. Walks the Java AST, converts to Scala AST (`FoolishAstn` types — done by `FoolishAst.fromProgram`)
3. Translates Scala AST to FIR (the work of this phase)
4. Serializes the FIR tree to JSON via Circe

The FIR JSON is the contract between Phase 1 and Phase 2. Phase 2's evaluator reads
this JSON, parses it back to FIR, and steps the states forward.

---

## Why This Approach

**The compiler is the parser plus a structural translator.** No arithmetic is
performed. `1 + 2 * 3` compiles to:

```scala
BinaryOpFir("+",
  ConstantIntFir(1),
  BinaryOpFir("*", ConstantIntFir(2), ConstantIntFir(3))
)
```

All three integer literals are already `Constant` (no work needed to know their
value), but the binary expression itself is `EMBRYONIC` until Phase 2's evaluator
visits it.

**Bare identifiers compile to fully-configured search FIRs.** `a_config` becomes:

```scala
SearchFir(
  pattern   = "^a_config$",
  direction = SearchDirection.Backward,
  anchored  = false,
  anchor    = None,
  state     = Nyes.EMBRYONIC
)
```

The pattern is a regex with `^...$` anchors so a search engine reading this can
match it directly, with no extra work to convert "this is a name" to "this is a
regex matching the name." This was an explicit design call from the user.

---

## Three Test Layers

Phase 1 has three test files, one per failure mode:

### Layer 1: `FoolishAstTest.scala` — parser correctness

Per-construct unit tests. Each test:
- inputs a small Foolish source string
- parses + converts via `FoolishAst.fromProgram`
- asserts the resulting `FoolishAstn` tree equals an inline-constructed expected value

Example:
```scala
test("parses bare identifier") {
  val ast = parseAndConvert("{a_config}")
  ast shouldBe ProgramAstn(TopLevelAstn(List(
    BraneAstn(Nil, List(IdentifierAstn(Nil, "aˍconfig")))
  )))
}
```

(Note the canonicalized identifier — `_` becomes the modifier letter low line `ˍ`
in the AST. See `02_implementor_reference.md` for the canonicalization rules.)

### Layer 2: `CompilerTest.scala` — AST → FIR translation correctness

Per-construct unit tests for the Scala AST → FIR step. Each test:
- inputs a `FoolishAstn` value (Scala literal, no parsing)
- runs the AST → FIR compiler
- asserts the resulting FIR equals an inline-constructed expected value

Example:
```scala
test("compiles bare identifier to backward unanchored search") {
  val ast = IdentifierAstn(Nil, "aˍconfig")
  val fir = Compiler.compileExpr(ast)
  fir shouldBe SearchFir(
    pattern   = "^aˍconfig$",
    direction = SearchDirection.Backward,
    anchored  = false,
    anchor    = None,
    state     = Nyes.EMBRYONIC
  )
}
```

### Layer 3: `FirRoundtripTest.scala` — Circe JSON roundtrip

For every FIR variant, assert `fir == decode[Fir](fir.asJson.noSpaces).right.get`.
Tests are inline (no `.foo`).

A new variant added to `Fir.scala` must add a roundtrip test here.

**These three layers are independent.** A failing Phase 1 test points unambiguously
at which layer is broken: the parser, the translator, or the JSON codec.

---

## Phase 1 Implementation Steps

The implementation steps below grow out of the language scope. Each step adds one
or two FIR variants, populates the corresponding `compileExpr` cases, and adds tests
to all three layers.

### P1.1 — Skeleton

Land the empty stub Compiler that parses, converts to AST, and emits `NKFir` for
every input. Verify `mvn test` runs the existing `FirRoundtripTest`.

**Already done** in the assembled scaffold.

### P1.2 — Integer literals

| Add to `compileExpr` | Test in |
|---|---|
| `IntLitAstn(v) -> ConstantIntFir(v)` | `CompilerTest`, `FirRoundtripTest` (already), `FoolishAstTest` |

### P1.3 — Empty brane and brane with anonymous statements

| Add to `compileExpr` | Test in |
|---|---|
| `BraneAstn(chars, stmts) -> NormalBraneFir(chars, stmts.map(compileStatement))` | all 3 layers |
| Anonymous statement: `StatementFir(name = None, body = compileExpr(stmt))` | |

### P1.4 — Identification

| Add to `compileExpr` | Test in |
|---|---|
| `AssignmentAstn(id, Normal, rhs) -> StatementFir(name = Some(id.id), body = compileExpr(rhs))` | all 3 layers |

### P1.5 — Arithmetic (tree only, no compute)

| Add to `compileExpr` | Test in |
|---|---|
| `BinaryExprAstn(op, l, r) -> BinaryOpFir(op, compileExpr(l), compileExpr(r))` | all 3 layers |
| `UnaryExprAstn(op, e) -> UnaryOpFir(op, compileExpr(e))` | |

### P1.6 — Bare identifiers (unanchored search)

| Add to `compileExpr` | Test in |
|---|---|
| `IdentifierAstn(Nil, id) -> SearchFir("^"+id+"$", Backward, anchored=false, None)` | all 3 layers |
| `IdentifierAstn(chars, id) -> CharacterizedRefFir(chars, "^"+id+"$")` | |

### P1.7 — `#-N` unanchored seek

| Add to `compileExpr` | Test in |
|---|---|
| `UnanchoredSeekAstn(offset) -> IndexFir(offset, anchored=false, None)` | all 3 layers |

### P1.8 — Anchored search operators

| Add to `compileExpr` | Test in |
|---|---|
| `DotSearchAstn(anchor, name)` and `name.id` is exact match → `SearchFir("^"+name.id+"$", Backward, anchored=true, Some(compileExpr(anchor)))` | all 3 layers |
| `RegexSearchAstn(anchor, REGEXP_LOCAL, pat) -> SearchFir(pat, Backward, anchored=true, Some(...))` | |
| `RegexSearchAstn(anchor, REGEXP_FORWARD_LOCAL, pat)` — **defer to Phase 6** | reject in compileExpr |
| `IndexAccessAstn(anchor, n) -> IndexFir(n, anchored=true, Some(compileExpr(anchor)))` | |
| `OneShotSearchAstn(anchor, HEAD) -> HeadTailFir(true, anchored=true, Some(compileExpr(anchor)))` | |
| `OneShotSearchAstn(anchor, TAIL) -> HeadTailFir(false, anchored=true, Some(compileExpr(anchor)))` | |

### P1.9 — Assignment sugar

| Add to `compileExpr` | Test in |
|---|---|
| `AssignmentAstn(id, TailSugar, rhs)` → `StatementFir(Some(id.id), compileExpr(OneShotSearchAstn(rhs, TAIL)))` | all 3 layers |
| `AssignmentAstn(id, HeadSugar, rhs)` → similar with HEAD | |

### P1.10 — `???` literal

| Add to `compileExpr` | Test in |
|---|---|
| `NKLitAstn -> NKFir(reason = "??? literal")` | all 3 layers |

### P1.11 — Reject MVP3-deferred constructs

For these AST node types, `compileExpr` must throw a clear compilation error:

- `NotImplementedAstn(reason)` (Phase 7 features)
- `ConcatenationAstn(_)` — Phase 6
- `RegexSearchAstn(_, REGEXP_FORWARD_LOCAL | REGEXP_GLOBAL | REGEXP_FORWARD_GLOBAL, _)` — Phase 6

The error should name the construct and the phase that adds it.

---

## What's Out of Phase 1

- Any actual evaluation. `BinaryOpFir(+, ConstantIntFir(1), ConstantIntFir(2))`
  stays exactly that — Phase 2 collapses it to `ConstantIntFir(3)`.
- Concatenation, forward search, detachment, SF/SFF — see overview.
- The 60 `.foo` approval tests live in Phase 2.

---

## Phase 1 Exit Criteria

- All three test layers pass for every language construct in scope.
- Every FIR variant has a roundtrip test.
- `Compiler.compileToJson(source)` runs end-to-end on at least 5 representative
  `.foo` source files (chosen from `foolish-core-scala/src/test/resources/.../inputs/`)
  without throwing — output is hand-inspected, no automated approval yet.
- `phase2_ubc.md` has been read and any open questions about the FIR schema have
  been resolved (likely some FIR variants need adjustment to support evaluation
  cleanly — make those changes in Phase 2 with passes through Phase 1's tests).

---

## Last Updated

**Date**: 2026-05-01
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.7 (1M Context)
**Changes**: Initial Phase 1 detail document — compiler with three test layers.
