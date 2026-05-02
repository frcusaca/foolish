# Phase 7 — Detachment, SF/SFF, Advanced Features

> Goal: The advanced language constructs that make Foolish a real programming
> language — detachment (`[id]{...}`), Stay-Foolish markers (`<expr>`),
> Stay-Fully-Foolish (`<<expr>>`), upward search (`↑`).

---

## Phase 7 Deliverable

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
| P7.1 | Basic `[id]{...}` M-brane | `detachmentAlarms.foo` (test_1) |
| P7.2 | P-brane `[+id]` partial application | `detachmentPBrane.foo` |
| P7.3 | Re-detachment | `detachmentComplexTests.foo` (test_re_detachment) |
| P7.4 | Forward search liberation `[~pat]` | `detachmentForwardSearch.foo` |
| P7.5 | SF mark `<expr>` | `detachmentSFMark.foo`, `SFMarkWithoutDetachment.foo` |
| P7.6 | SFF mark `<<expr>>` | `detachmentSFFMark.foo` |
| P7.7 | Alarm system | `detachmentAlarms.foo` (test_2, test_3) |
| P7.8 | Complex nested + curry chains | `detachmentComplexTests.foo` (remaining) |

---

## FIR Additions

```scala
case class DetachmentBraneFir(
  identifier: String,
  body:       Fir,
  state:      Nyes = Nyes.EMBRYONIC
) extends Fir

case class StayFoolishFir(expr: Fir,  state: Nyes = Nyes.EMBRYONIC) extends Fir
case class StayFullyFoolishFir(expr: Fir, state: Nyes = Nyes.EMBRYONIC) extends Fir
```

Each addition needs a roundtrip test in `FirRoundtripTest` and an AST→FIR test in
`CompilerTest` before being wired into the evaluator.

---

## Notes on SF / SFF

- **SF `<f>`**: resolves own symbols only, no child forwarding, does not step found
  results.
- **SFF `<<f>>`**: skips straight to ECONSTANIC.
- Both interact with concatenation to enable late binding and partial application.

---

## Out of Scope

- `IfExpr` is permanently rejected (FOOP-2). Phase 7 does *not* re-introduce it.
- `↑` upward search remains a possibility for a future Phase 8 if needed.

---

## Last Updated

**Date**: 2026-05-01
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.7 (1M Context)
**Changes**: Renumbered Phase 6 → Phase 7 (after inserting Phase 4 breadth-first).
Adopted Nyes terminology in FIR snippets. SFF "skips to Constanic" → "skips to
ECONSTANIC" for terminology precision.
