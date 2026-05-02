# FOOP Index

Canonical list of all Foolish Optimization Process documents.

FOOP numbers are little-endian: FOOP-`abcd` sorts by numerical value `dcba`.
Sort by hand when needed.

---

| FOOP | Title | Status | Phase | Created | Author |
|------|-------|--------|-------|---------|--------|
| [FOOP-1](FOOP-1.md) | FOOP Purpose, Process, and Format | Final | meta | 2026-05-01 | hc |
| [FOOP-2](FOOP-2.md) | Remove if-then-else from the language | Final | phase-1 | 2026-04-15 | hc |
| [FOOP-3](FOOP-3.md) | Sequential blocking concatenation | Brewing | phase-6 | 2026-04-22 | hc |
| [FOOP-4](FOOP-4.md) | Bare identifiers compile to anchored regex SearchFirs | Final | phase-1 | 2026-05-01 | hc |
| [FOOP-5](FOOP-5.md) | Compile-time vs evaluation-time work — the FIR contract | Final | phase-1 | 2026-05-01 | hc |
| [FOOP-6](FOOP-6.md) | Phase 2 evaluator is depth-first; breadth-first deferred to Phase 4 | Brewing | phase-2 | 2026-05-01 | hc |
| [FOOP-7](FOOP-7.md) | Constanic Clone — recoordination contract | Brewing | phase-2 | 2026-05-01 | hc |
| [FOOP-8](FOOP-8.md) | FIRs are mutable; parent pointers are post-clone; Circe excludes parent | Brewing | phase-2 | 2026-05-02 | hc |

---

## By Status

### Final

- [FOOP-1](FOOP-1.md) — FOOP Purpose, Process, and Format
- [FOOP-2](FOOP-2.md) — Remove if-then-else from the language
- [FOOP-4](FOOP-4.md) — Bare identifiers compile to anchored regex SearchFirs
- [FOOP-5](FOOP-5.md) — Compile-time vs evaluation-time work

### Brewing

- [FOOP-3](FOOP-3.md) — Sequential blocking concatenation (targets Phase 6)
- [FOOP-6](FOOP-6.md) — Phase 2 depth-first; Phase 4 breadth-first
- [FOOP-7](FOOP-7.md) — Constanic Clone recoordination contract
- [FOOP-8](FOOP-8.md) — FIRs are mutable; parent pointers post-clone; Circe excludes parent

### Implementing

(none yet)

### Withdrawn / Rejected / Superseded

(none yet)

---

## By Phase

### meta

- [FOOP-1](FOOP-1.md)

### phase-1

- [FOOP-2](FOOP-2.md), [FOOP-4](FOOP-4.md), [FOOP-5](FOOP-5.md)

### phase-2

- [FOOP-6](FOOP-6.md), [FOOP-7](FOOP-7.md), [FOOP-8](FOOP-8.md)

### phase-6

- [FOOP-3](FOOP-3.md)

---

## Process

To submit a new FOOP:

1. Find the next available number (highest existing + 1).
2. Copy `FOOP-template.md` to `FOOP-N.md`.
3. Fill in the YAML front matter and body sections.
4. Add a row to this index (and update the by-status / by-phase sections).
5. Commit. Initial status is `Draft` if you want feedback before
   submitting, or `Brewing` if it's ready for BDFL review.

See [FOOP-1](FOOP-1.md) for the full process specification.

---

## Last Updated

**Date**: 2026-05-02
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.7 (1M Context)
**Changes**: Added FOOP-8 (FIR mutability + parent pointers + Circe handling).
FOOP-7 retitled "recoordination contract" — its body was simplified to a
contract-only spec instead of attempting to specify state transitions in prose.
