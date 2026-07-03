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
| [FOOP-9](FOOP-9.md) | Operators are brane-like FIRs with positional unnamed operands and no search boundary | Implementing | phase-1 | 2026-05-04 | hc |
| [FOOP-01](FOOP-01.md) | Anchored search through constanic anchors — dereference searches, NK on missing brane names | Brewing | phase-2 | 2026-05-04 | hc |
| [FOOP-11](FOOP-11.md) | Search stops at NK; search result becomes NK | Brewing | phase-2 | 2026-05-04 | hc |
| [FOOP-21](FOOP-21.md) | Alarms — diagnostic levels emitted by compiler and evaluator | Implementing | phase-1 | 2026-05-04 | hc |
| [FOOP-31](FOOP-31.md) | SPA1 — UBC reference implementation (depth-first) | Draft | meta | 2026-05-07 | hc |
| [FOOP-41](FOOP-41.md) | UBCb — Message-passing brane computer variant; SPA1 parity plan | Draft | meta | 2026-05-07 | hc |
| [FOOP-51](FOOP-51.md) | AB list, name resolution, search_result, and short-circuit accumulation | Brewing | phase-2 | 2026-05-08 | hc |
| [FOOP-61](FOOP-61.md) | UBCb State Machine — Per-Variant NYES Table | Draft | phase-2 | 2026-05-09 | hc |
| [FOOP-71](FOOP-71.md) | Snapshot testing with cargo-insta for UBCb — approval testing infrastructure | Draft | meta | 2026-05-15 | Sisyphus |
| [FOOP-81](FOOP-81.md) | Enhanced SnapshotSuite with HumanizingSequencer and SequenceableFir | Superseded | meta | 2026-05-15 | Sisyphus |
| [FOOP-91](FOOP-91.md) | Rename all_terminal to all_constanic in UBCb | Draft | phase-3 | 2026-05-17 | opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4 |
| [FOOP-22](FOOP-22.md) | Multi-signer snapshot signatures with appended utility signing and entire-file integrity | Draft | meta | 2026-06-01 | Sisyphus |
| [FOOP-32](FOOP-32.md) | Repair rudimentary FVM evaluation and Sequencer formatting bugs found in snapshot review | Final | phase-2 | 2026-06-01 | Sisyphus |
| [FOOP-42](FOOP-42.md) | Humanizing FIR Sequencer formatting specification | Draft | phase-2 | 2026-06-03 | opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4 |
| [FOOP-52](FOOP-52.md) | Repair FVM evaluation bugs found in snapshot review round 2 | Draft | phase-2 | 2026-06-06 | opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4 |
| [FOOP-62](FOOP-62.md) | UBCa — Two-Store ProtoBrane Tree and Uniform Two-Phase Stepping | Final | phase-2 | 2026-06-09 | Atlas |
| [FOOP-72](FOOP-72.md) | Foolish Numbering System (FNS) and Snapshot Test Organization | Draft | phase-0 | 2026-06-17 | Sisyphus |
| [FOOP-82](FOOP-82.md) | UBCa Code Review — Findings and Recommended Changes | Draft | phase-2 | 2026-06-23 | Sisyphus |
| [FOOP-92](FOOP-92.md) | Einmo — directory-based signed-snapshot testing with staged promotion | Draft | meta | 2026-06-26 | Sisyphus |
| [FOOP-03](FOOP-03.md) | Repository Cleanup — Remove Dead Code, Flatten Workspace, Establish UBCa as Reference Implementation, Rename Main to jia | Draft (blocked on FOOP-62) | meta | 2026-07-01 | Sisyphus / mimo-v2.5-pro |

---

## By Status

### Final

- [FOOP-1](FOOP-1.md) — FOOP Purpose, Process, and Format
- [FOOP-2](FOOP-2.md) — Remove if-then-else from the language
- [FOOP-4](FOOP-4.md) — Bare identifiers compile to anchored regex SearchFirs
- [FOOP-5](FOOP-5.md) — Compile-time vs evaluation-time work
- [FOOP-32](FOOP-32.md) — Repair rudimentary FVM evaluation and Sequencer formatting bugs

### Draft

- [FOOP-31](FOOP-31.md) — SPA1 milestone (UBC reference implementation)
- [FOOP-41](FOOP-41.md) — UBCb message-passing variant; SPA1 parity plan
- [FOOP-52](FOOP-52.md) — Repair FVM evaluation bugs found in snapshot review round 2
- [FOOP-61](FOOP-61.md) — UBCb State Machine — Per-Variant NYES Table
- [FOOP-71](FOOP-71.md) — Snapshot testing with cargo-insta for UBCb (approval testing infrastructure)
- [FOOP-91](FOOP-91.md) — Rename all_terminal to all_constanic in UBCb
- [FOOP-22](FOOP-22.md) — Multi-signer snapshot signatures with appended utility signing
- [FOOP-42](FOOP-42.md) — Humanizing FIR Sequencer formatting specification
- [FOOP-72](FOOP-72.md) — Foolish Numbering System (FNS) and Snapshot Test Organization
- [FOOP-82](FOOP-82.md) — UBCa Code Review — Findings and Recommended Changes
- [FOOP-92](FOOP-92.md) — Einmo — directory-based signed-snapshot testing with staged promotion
- [FOOP-03](FOOP-03.md) — Repository Cleanup — dead code removal, workspace flatten, `jia` rename (blocked, see FOOP-62)

### Brewing

- [FOOP-3](FOOP-3.md) — Concatenation algorithm (targets Phase 3)
- [FOOP-6](FOOP-6.md) — Phase 2 depth-first; Phase 5 breadth-first
- [FOOP-7](FOOP-7.md) — Constanic Clone recoordination contract (revised 2026-05-08 to consume AB)
- [FOOP-8](FOOP-8.md) — FIRs are mutable; parent pointers post-clone; Circe excludes parent
- [FOOP-01](FOOP-01.md) — Anchored search through constanic anchors
- [FOOP-11](FOOP-11.md) — Search stops at NK
- [FOOP-51](FOOP-51.md) — AB list, name resolution, search_result, short-circuit accumulation
- [FOOP-62](FOOP-62.md) — UBCa Two-Store ProtoBrane Tree and Uniform Two-Phase Stepping (**Final** 2026-07-03; UBC retired, UBCa is the sole engine, merged to `jia`)

### Implementing

- [FOOP-9](FOOP-9.md) — Unified OperatorFir (was BinaryOpFir/UnaryOpFir)
- [FOOP-21](FOOP-21.md) — Alarms (compiler + evaluator diagnostic levels)

### Withdrawn / Rejected / Superseded

- [FOOP-81](FOOP-81.md) — Enhanced SnapshotSuite with HumanizingSequencer and SequenceableFir (Superseded)

---

## By Phase

### meta

- [FOOP-1](FOOP-1.md), [FOOP-31](FOOP-31.md), [FOOP-41](FOOP-41.md), [FOOP-71](FOOP-71.md), [FOOP-81](FOOP-81.md), [FOOP-22](FOOP-22.md), [FOOP-92](FOOP-92.md), [FOOP-03](FOOP-03.md)

### phase-0

- [FOOP-72](FOOP-72.md)

### phase-1

- [FOOP-2](FOOP-2.md), [FOOP-4](FOOP-4.md), [FOOP-5](FOOP-5.md), [FOOP-9](FOOP-9.md), [FOOP-21](FOOP-21.md)

### phase-2

- [FOOP-6](FOOP-6.md), [FOOP-7](FOOP-7.md), [FOOP-8](FOOP-8.md), [FOOP-01](FOOP-01.md), [FOOP-11](FOOP-11.md), [FOOP-51](FOOP-51.md), [FOOP-61](FOOP-61.md), [FOOP-32](FOOP-32.md), [FOOP-42](FOOP-42.md), [FOOP-52](FOOP-52.md), [FOOP-62](FOOP-62.md), [FOOP-82](FOOP-82.md)

### phase-3

- [FOOP-3](FOOP-3.md), [FOOP-91](FOOP-91.md)

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

**Date**: 2026-07-03
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes**: FOOP-62 CLOSED — status Brewing (backburnered) → **Final** in both the main table
and the By Status list. UBC retired; UBCa is the sole reference engine; merged to `jia`
(`e691b472`).

**Date**: 2026-07-02
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes**: FOOP-03 cleanup — added the 8 FOOPs missing from this index
(FOOP-81, FOOP-91, FOOP-42, FOOP-62, FOOP-72, FOOP-82, FOOP-92, FOOP-03) to
the main table, By Status, and By Phase sections (added a new `phase-0`
subsection for FOOP-72; added a `Withdrawn / Rejected / Superseded` entry
for FOOP-81). Noted FOOP-62 as backburnered and FOOP-03 as blocked on it.

**Date**: 2026-06-06
**Updated By**: opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Added FOOP-52 (repair FVM evaluation bugs round 2). Added
to main index table, Draft status section, and phase-2 section.

**Date**: 2026-06-03
**Updated By**: opencode / xiaomi/mimo-v2.5
**Changes**: Promoted FOOP-32 from Draft to Final (all bugs fixed).

**Date**: 2026-06-01
**Updated By**: opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Added FOOP-22 (multi-signer snapshot signatures) and FOOP-32
(repair rudimentary FVM evaluation and Sequencer formatting bugs). Added
to main index table, Draft status section, and respective phase sections.

**Date**: 2026-05-08
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.7 xHigh effort
**Changes**: Cleaned up FOOP numbering inconsistencies. Deleted
duplicate FOOP012.md (kept canonical FOOP-21.md as the alarms FOOP).
Identifier convention: filenames ARE the FOOP identifiers (FOOP-21,
FOOP-31, FOOP-41, etc. — written in little-endian digits per
AGENTS.md), and the `foop:` frontmatter is a numeric sort key (the
digits reversed to decimal). Fixed FOOP-31 (SPA1) sort key from
foop:31 to foop:13, matching the FOOP-01/11/21 → foop:10/11/12
pattern. Fixed FOOP-41 (UBCb) sort key from foop:32 to foop:14.
Updated INDEX entries to use identifier form (FOOP-21 not FOOP-12,
FOOP-01 not FOOP-10). Added FOOP-51 (AB list, name resolution,
search_result, short-circuit accumulation; sort key foop:15). Noted
FOOP-7 major revision in this session.

**Date**: 2026-05-08
**Updated By**: cyankiwi/Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Promoted FOOP-9 and FOOP-21 from Brewing to Implementing status.

**Date**: 2026-05-07
**Updated By**: opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Added FOOP-31 (SPA1 — UBC reference milestone) and FOOP-41
(UBCb — message-passing variant parity plan). Added Draft status section.

**Date**: 2026-05-04
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.7 (1M Context)
**Changes**: Added FOOP-21 (alarm system — diagnostic levels emitted by
compiler and evaluator, INFO/WARN/MILD/PANIC). Earlier same-day:
FOOPs 9-11 added; FOOP-3 retitled and rephased to phase-3.

**Date**: 2026-05-06
**Updated By**: Claude Code; cyankiwi/Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Renamed all FOOP-*.md files. Updated all internal
references from FOOP= to FOOP-. Added ls|rev|sort -V|rev sort command.
