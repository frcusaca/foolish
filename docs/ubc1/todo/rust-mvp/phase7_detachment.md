# Phase 7 — Detachment, Advanced Features

> Goal: The advanced language constructs — detachment (`[id]{...}`),
> partial application (`[+id]`), forward search liberation (`[~pat]`),
> upward search (`↑`).
>
> **SF and SFF were pulled forward and implemented in Phase 2.**
> See [phase2_sf_sff_seek_insights.md](phase2_sf_sff_seek_insights.md).

---

## Phase 7 Deliverable

Implement the remaining disabled `.foo` test files:

- `detachmentAlarms.foo.disabled`
- `detachmentComplexTests.foo.disabled`
- `detachmentForwardSearch.foo.disabled`
- `detachmentPBrane.foo.disabled`

**Already done in Phase 2:**
- `detachmentSFFMark.foo` — SFF (`<<expr>>`) implemented, 2 approval tests
- `detachmentSFMark.foo` — SF (`<expr>`) implemented, 2 approval tests
- `SFMarkWithoutDetachment.foo` — SF without detachment works

---

## Sub-phases (SF/SFF removed)

| Sub-phase | Feature | Tests |
|-----------|---------|-------|
| P7.1 | Basic `[id]{...}` M-brane | `detachmentAlarms.foo` (test_1) |
| P7.2 | P-brane `[+id]` partial application | `detachmentPBrane.foo` |
| P7.3 | Re-detachment | `detachmentComplexTests.foo` (test_re_detachment) |
| P7.4 | Forward search liberation `[~pat]` | `detachmentForwardSearch.foo` |
| P7.5 | Alarm system | `detachmentAlarms.foo` (test_2, test_3) |
| P7.6 | Complex nested + curry chains | `detachmentComplexTests.foo` (remaining) |

---

## FIR Additions

```rust
// Added to the Fir enum:
DetachmentBrane {
    identifier: String,
    body: FirRef,
    state: Nyes,
},
```

**Already implemented (Phase 2):**
- `StayFoolish` — `<expr>` marker, resolves non-brane searches
- `StayFullyFoolish` — `<<expr>>` marker, blocks all search expansion

---

## Notes on SF / SFF (Already Implemented)

SF and SFF were successfully pulled forward to Phase 2. Key behaviors:
- **SF `<f>`**: resolves all searches EXCEPT those targeting branes (blocks until later coordination)
- **SFF `<<f>>`**: blocks ALL search expansion; acts as stringent quote
- `constanic_clone` strips SFF wrapper and recurses into content
- Both interact with concatenation and arithmetic via `strip_sf_wrapper()`

---

## Parser Notes

The current parser rejects positive unanchored seeks (`#0`, `#1`). The spec
says `#0` should refer to the statement's own line. This parser fix is needed
before Phase 5 can fully test seeks.

## Out of Scope

- `IfExpr` is permanently rejected (FOOP=2).
- `↑` upward search remains a possibility for a future Phase 8.

---

## Last Updated

**Date**: 2026-05-06
**Updated By**: Claude Code; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Removed SF/SFF from Phase 7 scope (implemented in Phase 2).
Reduced sub-phases from 8 to 6. Added parser note about unanchored seek
positive offset restriction. Updated FIR additions section to note
StayFoolish/StayFullyFoolish are already done.

**Date**: 2026-05-05
**Updated By**: Claude Code; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Initial creation — Rust Phase 7 detachment plan.
