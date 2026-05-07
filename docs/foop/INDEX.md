# FOOP Index

Canonical list of all Foolish Optimization Process documents.

FOOP numbers are little-endian: FOOP-`abcd` sorts by numerical value `dcba`.
Sort the directory with:

```bash
ls | rev | sort -V | rev
```

---

| FOOP | Title | Status | Phase | Created | Author |
|------|-------|--------|-------|---------|--------|
| [FOOP-1](FOOP-1.md) | FOOP Purpose, Process, and Format | Final | meta | 2026-05-01 | hc |
| [FOOP-2](FOOP-2.md) | Remove if-then-else from the language | Final | phase-1 | 2026-04-15 | hc |
| [FOOP-3](FOOP-3.md) | Concatenation produces a new brane of constanicCloned elements; further steps delegate to the merged brane | Brewing | phase-3 | 2026-04-22 | hc |
| [FOOP-4](FOOP-4.md) | Bare identifiers compile to anchored regex SearchFirs | Final | phase-1 | 2026-05-01 | hc |
| [FOOP-5](FOOP-5.md) | Compile-time vs evaluation-time work — the FIR contract | Final | phase-1 | 2026-05-01 | hc |
| [FOOP-6](FOOP-6.md) | Phase 2 evaluator is depth-first; breadth-first deferred to Phase 5 | Brewing | phase-2 | 2026-05-01 | hc |
| [FOOP-7](FOOP-7.md) | Constanic Clone — recoordination contract | Brewing | phase-2 | 2026-05-01 | hc |
| [FOOP-8](FOOP-8.md) | FIRs are mutable; parent pointers are post-clone; Circe excludes parent | Brewing | phase-2 | 2026-05-02 | hc |
| [FOOP-9](FOOP-9.md) | Operators are brane-like FIRs with positional unnamed operands and no search boundary | Brewing | phase-1 | 2026-05-04 | hc |
| [FOOP01](FOOP01.md) | Anchored search through constanic anchors — dereference searches, NK on missing brane names | Brewing | phase-2 | 2026-05-04 | hc |
| [FOOP-11](FOOP-11.md) | Search stops at NK; search result becomes NK | Brewing | phase-2 | 2026-05-04 | hc |
| [FOOP-12](FOOP-12.md) | Alarms — diagnostic levels emitted by compiler and evaluator | Brewing | phase-1 | 2026-05-04 | hc |
| [FOOP-13](FOOP-31.md) | SPA1 — UBC reference implementation (depth-first) | Draft | meta | 2026-05-07 | hc |
| [FOOP-14](FOOP-32.md) | UBCb — Message-passing brane computer variant; SPA1 parity plan | Draft | meta | 2026-05-07 | hc |

---

## By Status

### Final

- [FOOP-1](FOOP-1.md) — FOOP Purpose, Process, and Format
- [FOOP-2](FOOP-2.md) — Remove if-then-else from the language
- [FOOP-4](FOOP-4.md) — Bare identifiers compile to anchored regex SearchFirs
- [FOOP-5](FOOP-5.md) — Compile-time vs evaluation-time work

### Draft

- [FOOP-13](FOOP-31.md) — SPA1 milestone (UBC reference implementation)
- [FOOP-14](FOOP-32.md) — UBCb message-passing variant; SPA1 parity plan

### Brewing

- [FOOP-3](FOOP-3.md) — Concatenation algorithm (targets Phase 3)
- [FOOP-6](FOOP-6.md) — Phase 2 depth-first; Phase 5 breadth-first
- [FOOP-7](FOOP-7.md) — Constanic Clone recoordination contract
- [FOOP-8](FOOP-8.md) — FIRs are mutable; parent pointers post-clone; Circe excludes parent
- [FOOP-9](FOOP-9.md) — Operators are brane-like FIRs with positional unnamed operands
- [FOOP01](FOOP01.md) — Anchored search through constanic anchors
- [FOOP-11](FOOP-11.md) — Search stops at NK
- [FOOP-12](FOOP-12.md) — Alarms (compiler + evaluator diagnostic levels)

### Implementing

(none yet)

### Withdrawn / Rejected / Superseded

(none yet)

---

## By Phase

### meta

- [FOOP-1](FOOP-1.md), [FOOP-13](FOOP-31.md), [FOOP-14](FOOP-32.md)

### phase-1

- [FOOP-2](FOOP-2.md), [FOOP-4](FOOP-4.md), [FOOP-5](FOOP-5.md), [FOOP-9](FOOP-9.md), [FOOP-12](FOOP-12.md)

### phase-2

- [FOOP-6](FOOP-6.md), [FOOP-7](FOOP-7.md), [FOOP-8](FOOP-8.md), [FOOP01](FOOP01.md), [FOOP-11](FOOP-11.md)

### phase-3

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

**Date**: 2026-05-07
**Updated By**: opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Added FOOP-13 (SPA1 — UBC reference milestone) and FOOP-14
(UBCb — message-passing variant parity plan). Added Draft status section.

**Date**: 2026-05-04
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.7 (1M Context)
**Changes**: Added FOOP-12 (alarm system — diagnostic levels emitted by
compiler and evaluator, INFO/WARN/MILD/PANIC). Earlier same-day:
FOOPs 9-11 added; FOOP-3 retitled and rephased to phase-3.

**Date**: 2026-05-06
**Updated By**: Claude Code; cyankiwi/Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Renamed all FOOP-*.md files. Updated all internal
references from FOOP= to FOOP-. Added ls|rev|sort -V|rev sort command.
