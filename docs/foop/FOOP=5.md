---
foop: 5
title: Compile-time vs evaluation-time work — the FIR contract
author: hc <hc.busy@gmail.com>
status: Final
type: Standards
created: 2026-05-01
phase: phase-1
supersedes: []
implementation: scala-mvp/foolish-scala/docs/phase1_compiler.md and Fir.scala
---

# FOOP=5: Compile-time vs evaluation-time work — the FIR contract

## Abstract

Retroactive: documents a decision made on 2026-05-01.

The Foolish compiler (Phase 1) does **structural translation only**. It
does NOT perform arithmetic, search resolution, or any other
computation. The expression `1 + 2 * 3` compiles to a tree of three
`ConstantIntFir`s under `BinaryOpFir`s, not to a `ConstantIntFir(7)`.
Computation belongs to Phase 2 (the UBC evaluator).

The exception: integer literals compile directly to
`ConstantIntFir(value, state = Constant)` — there is no work to do to
know that `42` evaluates to 42. The `???` literal compiles to
`NKFir(state = NK)` for the same reason.

## Motivation

Mixing compile-time and evaluation-time work was a recurring source of
confusion in earlier Foolish attempts. "Should the compiler fold
constants?" "Should it pre-compute deterministic searches?" Each yes
moves the boundary slightly and makes the compiler harder to reason
about.

The Phase 1 / Phase 2 split fixes the boundary:

- **Compile time**: anything determinable from the AST alone, with no
  reference to other FIRs. (Integer literals: yes. Arithmetic: no — it
  needs both operand values.)
- **Evaluation time**: anything that requires looking at sibling FIRs,
  searching scopes, or stepping state forward.

This split makes the FIR JSON the precise contract between phases. A
JSON FIR can be hand-written or hand-modified; whatever is in the JSON
defines what Phase 2 sees, and the JSON has no compute side-effects.

It also enables a clean test pyramid (see FOOP=6 if/when written for
the test layering decision):

- AST tests verify the parser without involving FIR.
- Compiler tests verify AST→FIR without involving evaluation.
- Roundtrip tests verify Circe codecs without involving anything else.
- Phase 2 approval tests verify the evaluator with a known-good
  compiler upstream.

If anything goes wrong, the layer of failure is unambiguous.

## Specification

### What the compiler does

1. Parses source via ANTLR.
2. Converts Java AST → Scala AST (`FoolishAstn`).
3. For each AST node, produces exactly one FIR according to a fixed
   translation table (see `phase1_compiler.md` for the per-construct
   list).
4. Serializes the resulting FIR tree to JSON via Circe.

### What the compiler does NOT do

- **Constant folding**: `1 + 2` does NOT become `ConstantIntFir(3)`. It
  becomes `BinaryOpFir("+", ConstantIntFir(1), ConstantIntFir(2),
  state = EMBRYONIC)`.
- **Search resolution**: `{a = 1, b = a}` does NOT have its `b = a`
  resolved. The `a` becomes a `SearchFir` in `EMBRYONIC` state.
- **Dead-code elimination**: never.
- **Type checking**: there are no static types in Foolish.
- **Optimization** of any kind.

### Direct-compile cases

Two AST nodes compile directly to a terminal Nyes state because their
final state is determinable from the node alone:

| AST node | FIR produced | Nyes |
|----------|-------------|------|
| `IntLitAstn(v)` | `ConstantIntFir(v)` | `INDEPENDENT` |
| `NKLitAstn` | `NKFir(reason = "??? literal")` | `NK` |

Integer literals are `INDEPENDENT` (not just `CONSTANT`) because no future
context can ever change a literal's value — `42` is `42` everywhere.
This matters for `constanicClone` (FOOP=7), which short-circuits on
`INDEPENDENT` exactly as on `CONSTANT`.

Every other AST node produces a FIR in `EMBRYONIC` state.

### State invariant

After Phase 1 compilation, every FIR in the tree is in exactly one of
these Nyes states: `EMBRYONIC`, `INDEPENDENT` (only for integer literals),
or `NK` (only for `???` literals).

After Phase 2 evaluation reaches a fixed point, every FIR is in exactly
one of: `CONSTANT`, `INDEPENDENT`, `ECONSTANIC`, `WOCONSTANIC`, or `NK`.
`EMBRYONIC` and `BRANING` are the explicit "work remains" signals.

## FIR Impact

Sets the meaning of the `state` field on every FIR variant.

The `state` field is part of the serialized JSON, so the Phase 1 → Phase 2
JSON contract carries the state. A Phase 1 producer that incorrectly
emitted `Constant` on an unresolved expression would mislead Phase 2; the
state field is therefore part of the testing surface for the compiler
(`CompilerTest` checks state alongside structure).

## UBC Step Impact

Phase 2 reads `state == EMBRYONIC` as "work to do here." A correctly
compiled tree has every non-leaf in `EMBRYONIC`. The evaluator's job is
to step `EMBRYONIC → BRANING → ...` through to a constanic terminal state.

A Phase 2 evaluator MAY assume the input tree obeys the Phase 1 state
invariant. It is not required to defend against a tree where, e.g., a
`BinaryOpFir` arrives in `CONSTANT` state — that's a contract violation
and a bug upstream.

## Test Plan

Compiler tests assert state explicitly:

```scala
test("FOOP=5: 1 + 2 compiles to BinaryOpFir(EMBRYONIC) over two INDEPENDENTs") {
  val source = "{1 + 2}"
  val fir = Compiler.compileToFir(source)
  fir shouldBe TopLevelAstAsFir(
    NormalBraneFir(Nil, List(
      StatementFir(None, BinaryOpFir(
        op    = "+",
        left  = ConstantIntFir(1),  // INDEPENDENT
        right = ConstantIntFir(2),  // INDEPENDENT
        state = Nyes.EMBRYONIC      // BinaryOp itself: EMBRYONIC
      ))
    ))
  )
}
```

The state field is checked structurally via `==` on case classes — no
custom matcher needed.

## Rejected Alternatives

### A. Compile-time constant folding

`1 + 2` → `ConstantIntFir(3)` at compile time. **Rejected**:

- Tempting because "obviously correct."
- Becomes a slippery slope: `1 + x` where `x = 2` is also "obviously"
  3, but now the compiler is doing search resolution.
- Better to draw the line at "compile time = no FIR-cross-references."

### B. Make `state` a Phase-2-only field; Phase 1 always emits without it

Cleaner separation in some sense. **Rejected**: adds a separate "input
to evaluator" tree shape that differs from the in-flight FIR tree.
Doubles the schema. The current design — `state` is on every FIR
always, Phase 1 just sets it to `EMBRYONIC` (or `INDEPENDENT` / `NK` for
the two literal cases) — is uniform.

### C. Compile lazily: a thunk-like FIR that re-runs the compiler when stepped

Defers the work but doesn't actually move the boundary. **Rejected**:
makes the FIR JSON not self-contained (it would reference compiler code
to be useful). Defeats the contract.

## Open Questions

None.

## References

- FOOP=2: another "do less at the wrong layer" decision.
- FOOP=4: another "compile uniformly, evaluate uniformly" decision.
- `scala-mvp/foolish-scala/docs/phase1_compiler.md`: the implementation
  step list.
- `scala-mvp/foolish-scala/foolish-core-scala/src/main/scala/org/foolish/fvm/scubc/Fir.scala`:
  the FIR definitions and the `Nyes` enum.
- FOOP=7: the constanic clone algorithm that exploits the EMBRYONIC/INDEPENDENT
  distinction.
