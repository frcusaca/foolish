# Phase 7 — Detachment, SF/SFF, Advanced Features

> Goal: The advanced language constructs — detachment (`[id]{...}`),
> Stay-Foolish markers (`<expr>`), Stay-Fully-Foolish (`<<expr>>`),
> upward search (`↑`).

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

```rust
// Added to the Fir enum:
DetachmentBrane {
    identifier: String,
    body: FirRef,
    state: Nyes,
},
StayFoolish {
    expr: FirRef,
    state: Nyes,
},
StayFullyFoolish {
    expr: FirRef,
    state: Nyes,
},
```

Each addition needs a roundtrip test and an AST→FIR test before being wired
into the evaluator.

---

## Notes on SF / SFF

- **SF `<f>`**: resolves own symbols only, no child forwarding, does not step found results.
- **SFF `<<f>>`**: skips straight to ECONSTANIC.
- Both interact with concatenation to enable late binding and partial application.

---

## Out of Scope

- `IfExpr` is permanently rejected (FOOP=2).
- `↑` upward search remains a possibility for a future Phase 8.

---

## Last Updated

**Date**: 2026-05-05
**Updated By**: Claude Code; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Initial creation — Rust Phase 7 detachment plan. Adapted from Scala
version with Rust enum variants and Rc<RefCell<FirRef>> patterns.
