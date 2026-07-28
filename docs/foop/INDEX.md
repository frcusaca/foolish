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
| [FOOP-9](FOOP-9.md) | Operators are brane-like FIRs with positional unnamed operands and no search boundary | Deprecated | phase-1 | 2026-05-04 | hc |
| [FOOP-01](FOOP-01.md) | Anchored search through constanic anchors — dereference searches, NK on missing brane names | Deprecated | phase-2 | 2026-05-04 | hc |
| [FOOP-11](FOOP-11.md) | Search stops at NK; search result becomes NK | Deprecated | phase-2 | 2026-05-04 | hc |
| [FOOP-21](FOOP-21.md) | Alarms — diagnostic levels emitted by compiler and evaluator | Deprecated | phase-1 | 2026-05-04 | hc |
| [FOOP-31](FOOP-31.md) | SPA1 — UBC reference implementation (depth-first) | Deprecated | meta | 2026-05-07 | hc |
| [FOOP-41](FOOP-41.md) | UBCb — Message-passing brane computer variant; SPA1 parity plan | Draft | meta | 2026-05-07 | hc |
| [FOOP-51](FOOP-51.md) | AB list, name resolution, search_result, and short-circuit accumulation | Deprecated | phase-2 | 2026-05-08 | hc |
| [FOOP-61](FOOP-61.md) | UBCb State Machine — Per-Variant NYES Table | Deprecated | phase-2 | 2026-05-09 | hc |
| [FOOP-71](FOOP-71.md) | Snapshot testing with cargo-insta for UBCb — approval testing infrastructure | Deprecated | meta | 2026-05-15 | Sisyphus |
| [FOOP-81](FOOP-81.md) | Enhanced SnapshotSuite with HumanizingSequencer and SequenceableFir | Superseded | meta | 2026-05-15 | Sisyphus |
| [FOOP-91](FOOP-91.md) | Rename all_terminal to all_constanic in UBCb | Deprecated | phase-3 | 2026-05-17 | opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4 |
| [FOOP-02](FOOP-02.md) | Consolidate FIR formatting; unify approval testing | Deprecated | phase-3 | 2026-05-17 | opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4 |
| [FOOP-22](FOOP-22.md) | Multi-signer snapshot signatures with appended utility signing and entire-file integrity | Deprecated | meta | 2026-06-01 | Sisyphus |
| [FOOP-32](FOOP-32.md) | Repair rudimentary FVM evaluation and Sequencer formatting bugs found in snapshot review | Final | phase-2 | 2026-06-01 | Sisyphus |
| [FOOP-42](FOOP-42.md) | Humanizing FIR Sequencer formatting specification | Deprecated | phase-2 | 2026-06-03 | opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4 |
| [FOOP-52](FOOP-52.md) | Repair FVM evaluation bugs found in snapshot review round 2 | Draft | phase-2 | 2026-06-06 | opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4 |
| [FOOP-62](FOOP-62.md) | UBCa — Two-Store ProtoBrane Tree and Uniform Two-Phase Stepping | Final | phase-2 | 2026-06-09 | Atlas |
| [FOOP-72](FOOP-72.md) | Foolish Numbering System (FNS) and Snapshot Test Organization | Draft | phase-0 | 2026-06-17 | Sisyphus |
| [FOOP-82](FOOP-82.md) | UBCa Code Review — Findings and Recommended Changes | Draft | phase-2 | 2026-06-23 | Sisyphus |
| [FOOP-92](FOOP-92.md) | Einmo — directory-based signed-snapshot testing with staged promotion | Draft | meta | 2026-06-26 | Sisyphus |
| [FOOP-03](FOOP-03.md) | Repository Cleanup — Remove Dead Code, Flatten Workspace, Establish UBCa as Reference Implementation, Rename Main to jia | Draft (blocked on FOOP-62) | meta | 2026-07-01 | Sisyphus / mimo-v2.5-pro |
| [FOOP-13](FOOP-13.md) | MAX_BRANE_SIZE — auto-sizing via a non-merging ConcatBrane equivalent to the merged brane | Brewing | phase-2 | 2026-07-03 | Atlas |
| [FOOP-23](FOOP-23.md) | Value search and contexted (&-prefixed) search — value equality, expression patterns, and searching from a statement's position | Draft | phase-2 | 2026-07-04 | Atlas |
| [FOOP-33](FOOP-33.md) | The Creation Postulate — ⬤, universal characterizations, and Booleans | Final | phase-4 | 2026-07-07 | Atlas |
| [FOOP-43](FOOP-43.md) | Search settlement — miss settles by anchoring; SFF-marked → ECONSTANIC (foundational keystone) | Draft | phase-2 | 2026-07-09 | Atlas |
| [FOOP-53](FOOP-53.md) | Computed index — `#${...}` | Draft | phase-2 | 2026-07-09 | Atlas |
| [FOOP-63](FOOP-63.md) | Primitive Characterization — the `i'`/`s'`/`f'` type system | Draft | phase-4 | 2026-07-09 | Atlas |
| [FOOP-73](FOOP-73.md) | Boolean operators — and, or, not, nor, xor (Foolish truth-table searches) | Draft | phase-4 | 2026-07-09 | Atlas |
| [FOOP-83](FOOP-83.md) | Strengthen integer math — exponent `**` and comparisons | Draft | phase-2 | 2026-07-09 | Atlas |
| [FOOP-93](FOOP-93.md) | Search predicates — inverse matcher `!` and matcher boolean operators `&&`/`\|\|` | Draft | phase-2 | 2026-07-09 | Atlas |
| [FOOP-04](FOOP-04.md) | Cascading connector for search — the `\|` fallback operator | Draft | phase-2 | 2026-07-09 | Atlas |
| [FOOP-14](FOOP-14.md) | All-results (find-all) search — doubled operators `~~`/`??` | Draft | phase-2 | 2026-07-09 | Atlas |
| [FOOP-24](FOOP-24.md) | Coordination detachment — parameterized stay-foolish markers | Draft | phase-2 | 2026-07-09 | Atlas |
| [FOOP-34](FOOP-34.md) | Recursion Upgrades (standalone research; write algorithms first) | Draft | phase-5 | 2026-07-09 | Atlas |
| [FOOP-44](FOOP-44.md) | Macros — research and design (standalone research) | Draft | phase-6 | 2026-07-09 | Atlas |
| [FOOP-74](FOOP-74.md) | FIRID — atomic per-Fir identity for constanic-clone cycle detection | Draft | phase-2 | 2026-07-11 | Atlas |
| [FOOP-84](FOOP-84.md) | Search Engine Refactor — the authoritative search specification, and detachment on it | Draft | phase-2 | 2026-07-28 | Claude Code (Sonnet 5) |

---

## By Status

### Final

- [FOOP-1](FOOP-1.md) — FOOP Purpose, Process, and Format
- [FOOP-2](FOOP-2.md) — Remove if-then-else from the language
- [FOOP-4](FOOP-4.md) — Bare identifiers compile to anchored regex SearchFirs
- [FOOP-5](FOOP-5.md) — Compile-time vs evaluation-time work
- [FOOP-32](FOOP-32.md) — Repair rudimentary FVM evaluation and Sequencer formatting bugs
- [FOOP-33](FOOP-33.md) — Creation Postulate → Booleans — `⬤` creation (ASCII alias `{*}`), three-valued default equality via value search (`Equality::{Equal,NotEqual,Unknowable}`), `Identifier`/minimal `Characterizations`, null-characterized name constants (`get_value()`→`NK("'…redefined")`), `system.foo` as the built-in root brane defining `'True`/`'False` (ready to implement)

### Draft

- [FOOP-41](FOOP-41.md) — UBCb message-passing variant; SPA1 parity plan
- [FOOP-52](FOOP-52.md) — Repair FVM evaluation bugs found in snapshot review round 2
- [FOOP-72](FOOP-72.md) — Foolish Numbering System (FNS) and Snapshot Test Organization
- [FOOP-82](FOOP-82.md) — UBCa Code Review — Findings and Recommended Changes
- [FOOP-92](FOOP-92.md) — Einmo — directory-based signed-snapshot testing with staged promotion
- [FOOP-03](FOOP-03.md) — Repository Cleanup — dead code removal, workspace flatten, `jia` rename (blocked, see FOOP-62)
- [FOOP-23](FOOP-23.md) — Value search + contexted `&`-searches — `~=`/`?=`, expression patterns, `&`-prefix navigation from a statement (`FoolRefFir`). **Semantics superseded by FOOP-84** (2026-07-28) — FOOP-23 remains authoritative only for grammar productions, the approval-test-input catalog, Rejected Alternatives, and its bug-fix Appendix.
*(Implementation-ordered batch built on FOOP-33; renumbered 2026-07-09 so number ≈ impl order.
**Search-engine sub-ordering corrected 2026-07-28 — see the explicit list immediately below.**)*
- [FOOP-43](FOOP-43.md) — Search settlement: **anchored miss → NK** (unchanged — an anchored search proves absence), **unanchored miss → ECONSTANIC** (unchanged), **SFF-marked search → ECONSTANIC regardless of anchoring** (the actual change — withheld candidates leave a search deferrable, not dead) + the NK-propagation fix (a deepen on an *unresolved* anchor waits instead of forcing NK) + **coordination removes search context** + the `EconstanicReason` tag (foundational keystone; prereq for FOOP-63/73/84/85/34)
- [FOOP-53](FOOP-53.md) — Computed index `#${...}` (evaluate brane, tail as number, run `#`; self-contained early win)
- [FOOP-63](FOOP-63.md) — Primitive Characterization: `i'`/`s'`/`f'` type system; characterization = type-tag + search-demand (brane WOCONSTANIC-waits); needs FOOP-33 + FOOP-43
- [FOOP-73](FOOP-73.md) — Boolean operators and/or/not/nor/xor as **Foolish truth-table searches** (no privileged layer; FVM-compute fallback); needs FOOP-33
- [FOOP-83](FOOP-83.md) — Integer math: exponent `**` + comparisons `< > <= >=` returning True/False (needs FOOP-33/73; `*`/`%` already done)
- [FOOP-84](FOOP-84.md) — **Search Engine Refactor — now the required foundation for the rest of this search-family batch, land it before FOOP-93/FOOP-04/FOOP-14/FOOP-24.** Supersedes FOOP-23/FOOP-24 as the authoritative search spec (absorbs the full operator table, `FoolRefFir` shape, name+value atomicity rule, and cursor-source×predicate framing so downstream FOOPs can cite it alone); unifies `ab_search_with_engine`/`BraneFir::_ab_search` into one `AncestralNavigator`; introduces per-candidate, innermost-to-outward boundary evaluation (`CopyMode`/`BoundaryEffect`, replacing `Scope.has_ancestral_sfm`) that FOOP-24 builds on; **§2.2.0 scope rule** — a marker affects only a backward/ancestral search originating inside it, only at the outward boundary crossing (never contexted `&` searches, never locally-resolving searches); documents `contexted ⟹ anchored` as permanent policy. **Two halves with different risk: Part 1 + §2.2 are behavior-preserving (no snapshot may change); §2.3–§2.5 are a deliberate semantic change with expected SF/SFF snapshot churn** — land as separate commits. **Hard dependency on FOOP-43** (§1.5 settlement rule and Component 3's `EconstanicReason::Detached`, which §2.4.1 now requires).
- [FOOP-93](FOOP-93.md) — Search predicates: inverse matcher `!` + matcher boolean operators `&&`/`||` (compiler-hard-coded matcher-outcome ops; SearchPredicate `And`/`Or`/negate). **Needs FOOP-84** (extends its `SearchPredicate`/de-duplicated Navigator; the old "shares a locus with detachment" note is corrected — orthogonal to FOOP-24, no relative ordering needed between the two).
- [FOOP-04](FOOP-04.md) — Cascading connector `|` (fallback between whole searches; `CascadingSearchFir`, shared-fixed-anchor semantics; needs FOOP-43). **Needs FOOP-84** (builds on its `FoolRefFir`/contexted-resume restatement and de-duplicated Navigator).
- [FOOP-14](FOOP-14.md) — All-results `~~`/`??` (doubled operators collect into a brane; tokens already lexed). **Needs FOOP-84** (collect-mode scan runs over its `AncestralNavigator`; composes for free with FOOP-24's `Detach` filtering — no special handling needed, see FOOP-14's Composition note).
- [FOOP-24](FOOP-24.md) — **Coordination detachment** = parameterized SF/SFF marker (`[patterns]<...>` / `[patterns]<<...>>`; the `Detachment` struct, `decide_to_detach`, `[patterns]` parsing). **The live spec for the feature** — renamed from plain "Detachment" per FOOP-84 §Part 3. FOOP-84 supersedes its *mechanism* only (Implementation Plan → Phase A's `_ab_search`-override, and the scan-loop prefilter locus) — build on FOOP-84 Part 2's `resolve_boundary_effect`/`CopyMode` instead. "Nested markers" resolved (FOOP-84 §2.3/§2.6); "Exclusive detachment" reframed under "Required Searches". Scope is narrow (FOOP-84 §2.2.0): **affects only descendant searches of the marker, as they cross the marker's boundary outward** — never contexted (`&`) searches, never locally-resolving ones. **Needs FOOP-84** (hard dependency) **and FOOP-43** (its SFF-marked→ECONSTANIC rule + `EconstanicReason::Detached`, so full-detachment exhaustion defers rather than settling NK). Independent of FOOP-93/FOOP-04/FOOP-14.
- [FOOP-34](FOOP-34.md) — Recursion Upgrades (**standalone research**; write ~1–2 dozen algorithms first; after the full search suite — FOOP-84/93/04/14/85; `↑`; no cycle detection)
- [FOOP-44](FOOP-44.md) — Macros (**standalone research**; brane-transforms-brane vs expansion phase; leans on FOOP-14 + characterizations)
- [FOOP-74](FOOP-74.md) — FIRID (atomic per-Fir instance counter) + thread-local in-flight clone stack; `eprintln!` alarm when `constanic_clone_at` re-enters an already-in-progress FIRID (detection/visibility only, not a language semantic — distinct from FOOP-34's "no recursion-cycle detection" language-design stance)

**Explicit search-engine sub-batch implementation order (2026-07-28 correction):** the discovery
that FOOP-24's detachment design needed a real engine refactor (now FOOP-84) means the previous
flat listing above (FOOP-93 → FOOP-04 → FOOP-14 → FOOP-24) no longer reflects buildable order.
Corrected order:

1. **FOOP-43** (keystone) — miss settles by anchoring (anchored→NK, unanchored→ECONSTANIC),
   **SFF-marked→ECONSTANIC**, the NK-propagation fix, and the `EconstanicReason` tag. Prerequisite
   for everything below; FOOP-84 §1.5/§2.4.1 depend on it directly.
2. **FOOP-84** (new keystone for the search-engine internals) — `AncestralNavigator`,
   `CopyMode`/`BoundaryEffect`, the absorbed operator-table reference. Nothing below can be
   cleanly built without this landing first — it de-duplicates the two existing search code paths
   that FOOP-93/FOOP-04/FOOP-14/FOOP-24 would otherwise each have to individually reconcile with.
3. **FOOP-93, FOOP-04, FOOP-14, FOOP-24** — mutually independent (orthogonal collaborators: two
   extend `SearchPredicate`/scan-mode, one is a standalone wrapper FIR, one extends the Navigator
   via marker configuration); may land in any order or in parallel once FOOP-84 is done. FOOP-24
   is the only one with an additional prerequisite (FOOP-43's SFF-marked→ECONSTANIC rule, for
   full-detachment exhaustion — already satisfied by step 1).
4. **FOOP-34** (recursion) — after the full search suite (all of the above), per its own spec.

### Brewing

- [FOOP-13](FOOP-13.md) — MAX_BRANE_SIZE — auto-sizing via a non-merging ConcatBrane (two phases: ConcatBrane upgrade, then the limit; ready for BDFL review 2026-07-03)

- [FOOP-3](FOOP-3.md) — Concatenation algorithm (targets Phase 3)
- [FOOP-6](FOOP-6.md) — Phase 2 depth-first; Phase 5 breadth-first
- [FOOP-7](FOOP-7.md) — Constanic Clone recoordination contract (revised 2026-05-08 to consume AB)
- [FOOP-8](FOOP-8.md) — FIRs are mutable; parent pointers post-clone; Circe excludes parent
- [FOOP-62](FOOP-62.md) — UBCa Two-Store ProtoBrane Tree and Uniform Two-Phase Stepping (**Final** 2026-07-03; UBC retired, UBCa is the sole engine, merged to `jia`)

### Implementing

(none — FOOP-9 and FOOP-21 deprecated 2026-07-03; see Deprecated section)

### Deprecated

Canceled as they stand; each may be respecified and reimplemented later. See
`## Deprecation Notice` in each file for rationale.

- [FOOP-9](FOOP-9.md) — Unified OperatorFir (was BinaryOpFir/UnaryOpFir)
- [FOOP-01](FOOP-01.md) — Anchored search through constanic anchors
- [FOOP-11](FOOP-11.md) — Search stops at NK
- [FOOP-21](FOOP-21.md) — Alarms (compiler + evaluator diagnostic levels)
- [FOOP-31](FOOP-31.md) — SPA1 milestone (UBC reference implementation)
- [FOOP-51](FOOP-51.md) — AB list, name resolution, search_result, short-circuit accumulation
- [FOOP-61](FOOP-61.md) — UBCb State Machine — Per-Variant NYES Table
- [FOOP-71](FOOP-71.md) — Snapshot testing with cargo-insta for UBCb (approval testing infrastructure)
- [FOOP-91](FOOP-91.md) — Rename all_terminal to all_constanic in UBCb
- [FOOP-02](FOOP-02.md) — Consolidate FIR formatting; unify approval testing
- [FOOP-22](FOOP-22.md) — Multi-signer snapshot signatures with appended utility signing (superseded by FOOP-92)
- [FOOP-42](FOOP-42.md) — Humanizing FIR Sequencer formatting specification

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

- [FOOP-6](FOOP-6.md), [FOOP-7](FOOP-7.md), [FOOP-8](FOOP-8.md), [FOOP-01](FOOP-01.md), [FOOP-11](FOOP-11.md), [FOOP-51](FOOP-51.md), [FOOP-61](FOOP-61.md), [FOOP-32](FOOP-32.md), [FOOP-42](FOOP-42.md), [FOOP-52](FOOP-52.md), [FOOP-62](FOOP-62.md), [FOOP-82](FOOP-82.md), [FOOP-13](FOOP-13.md), [FOOP-23](FOOP-23.md), [FOOP-43](FOOP-43.md), [FOOP-53](FOOP-53.md), [FOOP-83](FOOP-83.md), [FOOP-93](FOOP-93.md), [FOOP-04](FOOP-04.md), [FOOP-14](FOOP-14.md), [FOOP-24](FOOP-24.md), [FOOP-74](FOOP-74.md), [FOOP-84](FOOP-84.md)

### phase-3

- [FOOP-3](FOOP-3.md), [FOOP-91](FOOP-91.md)

### phase-4

- [FOOP-33](FOOP-33.md), [FOOP-63](FOOP-63.md), [FOOP-73](FOOP-73.md)

### phase-5

- [FOOP-34](FOOP-34.md)

### phase-6

- [FOOP-44](FOOP-44.md)

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

**Date**: 2026-07-28 (4)
**Updated By**: Claude Code (Opus 5)
**Changes**: **FOOP-85 withdrawn; coordination detachment stays in FOOP-24.** The 2026-07-28 (2)
revision reserved a separate FOOP-85 for implementing what FOOP-24 already specifies. That was
wrong on two counts: (a) coordination detachment **is** FOOP-24's feature — FOOP-84 Part 3 renamed
it, it did not fork a new one — so reserving a second number split a live feature from its own
specification and demoted FOOP-24 to a "historical" document while its semantics, cross-tabulation
table, pattern types, and test plan all remained current; (b) 85 is not a valid next number under
little-endian numbering — `foop_check.py gen_next` yields **FOOP-94** (sort key 49). Removed the
FOOP-85 bullet, folded its content into a rewritten FOOP-24 entry (retitled "Coordination
detachment"), and redirected every live FOOP-85 reference across FOOP-84, FOOP-93, FOOP-43, and
this index to FOOP-24. What FOOP-84 supersedes in FOOP-24 is **mechanism only** (the Phase A
`_ab_search`-override design and the scan-loop prefilter locus), not the feature. FOOP-94 remains
free for genuinely new work.

**Date**: 2026-07-28 (3)
**Updated By**: Claude Code (Opus 5)
**Changes**: Propagated three FOOP-43/FOOP-84 corrections settled with Atlas this session.
(1) **FOOP-43's rule changed**: anchored miss stays **NK** (not ECONSTANIC as its prior draft
proposed); unanchored miss stays ECONSTANIC; the real change is **SFF-marked searches →
ECONSTANIC regardless of anchoring**, plus the NK-propagation fix for deepens on unresolved
anchors. Retitled FOOP-43 and rewrote both index entries. (2) **FOOP-84's behavior-preserving
claim split**: Part 1 + §2.2 preserve behavior (no snapshot may change), §2.3–§2.5 are a
deliberate semantic change with expected SF/SFF churn; upgraded its FOOP-43 dependency from
"relies on, does not modify" to a hard dependency (§2.4.1 now needs `EconstanicReason::Detached`).
(3) **Marker scope rule** (FOOP-84 §2.2.0) recorded on the FOOP-84 and FOOP-85 entries: a marker
affects only backward/ancestral searches originating inside it, only at the outward boundary
crossing — never contexted (`&`) searches, never locally-resolving ones. Corrected FOOP-85's
prerequisite description to name the SFF-marked rule rather than anchored-miss recoordination.

**Date**: 2026-07-28 (2)
**Updated By**: Claude Code (Sonnet 5)
**Changes**: Documented the correct implementation order for the search-engine sub-batch now that
FOOP-84 (added earlier the same day) is a load-bearing prerequisite for FOOP-93/FOOP-04/FOOP-14/
FOOP-85, not a peer alongside them. Rewrote the Draft-status listing for this batch with explicit
"Needs FOOP-84" annotations on FOOP-93/FOOP-04/FOOP-14, added a **FOOP-85 (reserved, not yet
created)** entry documenting what it must build (the `Detachment` struct/parser on top of FOOP-84
Part 2, plus FOOP-84 Part 3's terminology), and added a short explicit numbered order
(FOOP-43 → FOOP-84 → {FOOP-93, FOOP-04, FOOP-14, FOOP-85 in any order/parallel} → FOOP-34) at the
end of the Draft section. Corrected the FOOP-84 changelog entry below: `CopyMode` is a two-variant
type (`Normal`/`SfCopy`) that only ever reaches the scan loop for yielded candidates — `Detach` is
a pre-yield filter internal to the Navigator's own iteration (`next_candidate()` simply never
returns a Detached candidate), not a third value riding the same channel as originally described
here; see FOOP-84 §2.3's own Last-Updated entry for the full correction. Also updated FOOP-93,
FOOP-04, and FOOP-14 in place with "Builds on FOOP-84" banners and redirected references (FOOP-93
additionally corrected a stale claim that it "shares a locus" with detachment — it doesn't, per
FOOP-84 §2.3, since detachment now acts in the Navigator, not the Predicate).

**Date**: 2026-07-28
**Updated By**: Claude Code (Sonnet 5)
**Changes**: Added **FOOP-84** (Search Engine Refactor — the authoritative search specification,
and detachment on it), Draft, phase-2, sort key 48. Grew out of a design review of FOOP-24
(Detachment) that found the search engine had no representation for "the sequence of candidates
visible to a search, crossing brane boundaries" — the missing piece detachment needs. FOOP-84
supersedes FOOP-23/FOOP-24 on search-mechanism specifics: unifies `ab_search_with_engine` and
`BraneFir::_ab_search` into one `AncestralNavigator`; replaces `Scope.has_ancestral_sfm` with a
per-candidate, innermost-to-outward `CopyMode` resolution (`Normal`/`SfCopy`/`Detach`) that
resolves FOOP-24's previously-UNDECIDED "nested markers" question; documents `contexted ⟹
anchored` as permanent policy (not a gap); renames FOOP-24's "Detachment" to "Coordination
detachment" and reframes "Exclusive detachment" as one mechanism under a new "Required Searches"
future feature. Deliberately behavior-preserving (no new syntax, no snapshot churn expected) —
implementation of coordination detachment itself is reserved as **FOOP-85** (not yet created).
Added FOOP-24's PARTIALLY SUPERSEDED banner cross-reference. Added to main table, By-Status
(Draft), and By-Phase (phase-2).

**Date**: 2026-07-11
**Updated By**: Claude Code 2.1.119 (Claude Code); Sonnet 5
**Changes**: Added **FOOP-74** — FIRID (atomic per-Fir instance counter on `ProtoBrane`) +
thread-local in-flight clone stack + `eprintln!` alarm when `constanic_clone_at` re-enters an
already-in-progress FIRID. Diagnostic tooling only (no semantic/evaluation change), written
directly from a FOOP-13 triage session that found a genuine constanic-clone cycle by hand
(`concat_sf_f_more.foo`'s `f1`: `a`'s search for `"b"` clones `f1.b`; the clone's revived
`<<b + c>>` search finds the same original `f1.b` again, nesting without bound). Numbered 74
(sort key 47) at Atlas's explicit request, deliberately leaving a gap at sort key 46 — noted in
the FOOP itself as accepted, not an error. Added to main table, By-Status (Draft), and By-Phase
(phase-2).

**Date**: 2026-07-09
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes**: Added **11 new Draft FOOPs** (a coherent language-feature batch built on FOOP-33),
flushed from `docs/foop/NOTES-creation-lineage-and-search-family.md`: FOOP-43 (search miss →
ECONSTANIC — the keystone), 53 (inverse `!`), 63 (detachment = parameterized SF/SFF marker),
73 (all-results `~~`/`??`), 83 (boolean operators), 93 (Recursion Upgrades), 04 (macros research),
14 (computed index `#${...}`), 24 (beefy search `&&`/`||`/`|`), 34 (integer math `**`/comparisons),
44 (Primitive Characterization `i'`/`s'`/`f'`). Dependency graph verified acyclic (FOOP-43 is the
keystone; 63/24/44/93 depend on it; 83/34/44 need FOOP-33). Added to main table, By-Status (Draft),
and By-Phase (phase-2/4/5/6). Numbering consecutive 1–44.

**Date**: 2026-07-08
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes**: FOOP-33 status Draft → **Final** (design frozen, ready to implement) after several
review rounds with Atlas. Moved its By-Status entry Draft → Final and refreshed the summary
(three-valued equality, `Identifier`, null-const `get_value()`→NK, `system.foo` as built-in root).

**Date**: 2026-07-07
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes**: Added FOOP-33 (The Creation Postulate — `⬤`, universal characterizations, and
Booleans), Draft, phase-4, with plan (FOOP-33.plan.md). Adds `⬤` creation with a global identity
map, referential equality via value search, a first-class `Characterizations` struct with
null-characterized name constants (enforced at brane step and concatenation), and `system.foo` as
the ancestral prelude defining `'True`/`'False`. Added to main table, By-Status (Draft), and a new
By-Phase phase-4 section.

**Date**: 2026-07-05
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Retitled FOOP-23 to "Value search and contexted (`&`-prefixed) search" after design
revision: search now splits into a contextless family (existing `.` `?` `~` `#` `^` `$` + value
`~=`/`?=` — deepen, demand a brane) and a contexted family (`&`-prefixed twins — navigate from a
statement's position within its home brane). Updated the main-table title and By-Status entry.

**Date**: 2026-07-04
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Added FOOP-23 (Value search — `~=`/`?=` operator family with six forms, expression
patterns, chained-search sequencing, and search-anchored `#`/`?`/`~` via the new `FoolRefFir`
strong original-statement reference), Draft, phase-2, with implementation plan
(FOOP-23.plan.md). Added to main table, By Status (Draft), and By Phase (phase-2).

**Date**: 2026-07-03
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: FOOP-13 status Draft → Brewing (design converged; ready for BDFL review). Moved its
By Status bullet from Draft to Brewing.

**Date**: 2026-07-03
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Deprecated 12 FOOPs at user request: FOOP-9, FOOP-01, FOOP-11, FOOP-21, FOOP-31,
FOOP-51, FOOP-61, FOOP-71, FOOP-91, FOOP-02, FOOP-42 (status → Deprecated, canceled as they
stand, to be later respecified and reimplemented), and FOOP-22 (status → Deprecated, superseded
by FOOP-92). Added a Deprecation Notice to each spec's body and canceled every outstanding
checkbox in each `.plan.md` (already-completed checkboxes left untouched as a record of prior
progress). Added missing FOOP-02 row to the main table (previously absent). Added new
"Deprecated" subsection under By Status; removed the 12 FOOPs from Draft/Brewing/Implementing.

**Date**: 2026-07-03
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Added FOOP-13 (MAX_BRANE_SIZE — auto-sizing via a non-merging ConcatBrane), Draft,
phase-2. Updated By Status (Draft) and By Phase (phase-2) sections. Later same day: retitled
after design revision (two phases: ConcatBrane upgrade with hidden k-ary storage tree, then the
MAX_BRANE_SIZE limit with iterative chunk grouping).

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
