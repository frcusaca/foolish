# Phase 5 — Detachment, SF/SFF, Advanced Features

> Goal: The advanced language constructs that make Foolish a real programming
> language — detachment (`[id]{...}`), Stay-Foolish markers (`<expr>`),
> Stay-Fully-Foolish (`<<expr>>`), upward search (`↑`).

---

## Phase 5 Deliverable

Implement the 7 currently-disabled `.foo` test files:

- `detachmentAlarms.foo.disabled`
- `detachmentComplexTests.foo.disabled`
- `detachmentForwardSearch.foo.disabled`
- `detachmentPBrane.foo.disabled`
- `detachmentSFFMark.foo.disabled`
- `detachmentSFMark.foo.disabled`
- `SFMarkWithoutDetachment.foo.disabled`

---

## Sub-phases

| Sub-phase | Feature | Tests |
|-----------|---------|-------|
| P5.1 | Basic `[id]{...}` M-brane | `detachmentAlarms.foo` (test_1) |
| P5.2 | P-brane `[+id]` partial application | `detachmentPBrane.foo` |
| P5.3 | Re-detachment | `detachmentComplexTests.foo` (test_re_detachment) |
| P5.4 | Forward search liberation `[~pat]` | `detachmentForwardSearch.foo` |
| P5.5 | SF mark `<expr>` | `detachmentSFMark.foo`, `SFMarkWithoutDetachment.foo` |
| P5.6 | SFF mark `<<expr>>` | `detachmentSFFMark.foo` |
| P5.7 | Alarm system | `detachmentAlarms.foo` (test_2, test_3) |
| P5.8 | Complex nested + curry chains | `detachmentComplexTests.foo` (remaining) |

---

## FIR Additions

```scala
case class DetachmentBraneFir(
  identifier: String,
  body:       Fir,
  state:      FirState = FirState.Initialized
) extends Fir

case class StayFoolishFir(expr: Fir,  state: FirState = FirState.Initialized) extends Fir
case class StayFullyFoolishFir(expr: Fir, state: FirState = FirState.Initialized) extends Fir
```

Each addition needs a roundtrip test in `FirRoundtripTest` and an AST→FIR test in
`CompilerTest` before being wired into the evaluator.

---

## Notes on SF / SFF

- **SF `<f>`**: resolves own symbols only, no child forwarding, does not step found
  results.
- **SFF `<<f>>`**: skips straight to Constanic.
- Both interact with concatenation to enable late binding and partial application.

---

## Out of Scope

- `IfExpr` is permanently rejected (UBC2 design removed it; search-based selection
  replaces it). Phase 5 does *not* re-introduce it.
- `↑` upward search remains a possibility for a future Phase 6 if needed.

---

## Last Updated

**Date**: 2026-05-01
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.7 (1M Context)
**Changes**: Initial Phase 5 outline — detachment, SF/SFF, and remaining advanced
language features.
