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
| [FOOP-3](FOOP-3.md) | Concatenation produces a new brane of constanicCloned elements; further steps delegate to the merged brane | Superseded | phase-3 | 2026-04-22 | hc |
| [FOOP-4](FOOP-4.md) | Bare identifiers compile to anchored regex SearchFirs | Final | phase-1 | 2026-05-01 | hc |
| [FOOP-5](FOOP-5.md) | Compile-time vs evaluation-time work — the FIR contract | Final | phase-1 | 2026-05-01 | hc |
| [FOOP-6](FOOP-6.md) | Phase 2 evaluator is depth-first; breadth-first deferred to Phase 5 | Superseded | phase-2 | 2026-05-01 | hc |
| [FOOP-7](FOOP-7.md) | Constanic Clone — recoordination contract | Superseded | phase-2 | 2026-05-01 | hc |
| [FOOP-8](FOOP-8.md) | FIRs are mutable; parent pointers are post-clone; Circe excludes parent | Superseded | phase-2 | 2026-05-02 | hc |
| [FOOP-9](FOOP-9.md) | Operators are brane-like FIRs with positional unnamed operands and no search boundary | Deprecated | phase-1 | 2026-05-04 | hc |
| [FOOP-01](FOOP-01.md) | Anchored search through constanic anchors — dereference searches, NK on missing brane names | Deprecated | phase-2 | 2026-05-04 | hc |
| [FOOP-11](FOOP-11.md) | Search stops at NK; search result becomes NK | Deprecated | phase-2 | 2026-05-04 | hc |
| [FOOP-21](FOOP-21.md) | Alarms — diagnostic levels emitted by compiler and evaluator | Deprecated | phase-1 | 2026-05-04 | hc |
| [FOOP-31](FOOP-31.md) | SPA1 — UBC reference implementation (depth-first) | Deprecated | meta | 2026-05-07 | hc |
| [FOOP-41](FOOP-41.md) | UBCb — Message-passing brane computer variant; SPA1 parity plan | Deprecated | meta | 2026-05-07 | hc |
| [FOOP-51](FOOP-51.md) | AB list, name resolution, search_result, and short-circuit accumulation | Deprecated | phase-2 | 2026-05-08 | hc |
| [FOOP-61](FOOP-61.md) | UBCb State Machine — Per-Variant NYES Table | Deprecated | phase-2 | 2026-05-09 | hc |
| [FOOP-71](FOOP-71.md) | Snapshot testing with cargo-insta for UBCb — approval testing infrastructure | Deprecated | meta | 2026-05-15 | Sisyphus |
| [FOOP-81](FOOP-81.md) | Enhanced SnapshotSuite with HumanizingSequencer and SequenceableFir | Superseded | meta | 2026-05-15 | Sisyphus |
| [FOOP-91](FOOP-91.md) | Rename all_terminal to all_constanic in UBCb | Deprecated | phase-3 | 2026-05-17 | opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4 |
| [FOOP-02](FOOP-02.md) | Consolidate FIR formatting; unify approval testing | Deprecated | phase-3 | 2026-05-17 | opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4 |
| [FOOP-22](FOOP-22.md) | Multi-signer snapshot signatures with appended utility signing and entire-file integrity | Deprecated | meta | 2026-06-01 | Sisyphus |
| [FOOP-32](FOOP-32.md) | Repair rudimentary FVM evaluation and Sequencer formatting bugs found in snapshot review | Final | phase-2 | 2026-06-01 | Sisyphus |
| [FOOP-42](FOOP-42.md) | Humanizing FIR Sequencer formatting specification | Deprecated | phase-2 | 2026-06-03 | opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4 |
| [FOOP-52](FOOP-52.md) | Repair FVM evaluation bugs found in snapshot review round 2 | Superseded (by FOOP-62) | phase-2 | 2026-06-06 | opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4 |
| [FOOP-62](FOOP-62.md) | UBCa — Two-Store ProtoBrane Tree and Uniform Two-Phase Stepping | Final | phase-2 | 2026-06-09 | Atlas |
| [FOOP-72](FOOP-72.md) | Foolish Numbering System (FNS) and Snapshot Test Organization | Draft | phase-0 | 2026-06-17 | Sisyphus |
| [FOOP-82](FOOP-82.md) | UBCa Code Review — Findings and Recommended Changes | Draft | phase-2 | 2026-06-23 | Sisyphus |
| [FOOP-92](FOOP-92.md) | Einmo — directory-based signed-snapshot testing with staged promotion | Complete | meta | 2026-06-26 | Sisyphus |
| [FOOP-03](FOOP-03.md) | Repository Cleanup — Remove Dead Code, Flatten Workspace, Establish UBCa as Reference Implementation, Rename Main to jia | Draft (blocked on FOOP-62) | meta | 2026-07-01 | Sisyphus / mimo-v2.5-pro |
| [FOOP-13](FOOP-13.md) | MAX_BRANE_SIZE — auto-sizing via a non-merging ConcatBrane equivalent to the merged brane | Brewing | phase-2 | 2026-07-03 | Atlas |
| [FOOP-23](FOOP-23.md) | Value search and contexted (&-prefixed) search — value equality, expression patterns, and searching from a statement's position | Complete | phase-2 | 2026-07-04 | Atlas |
| [FOOP-33](FOOP-33.md) | The Creation Postulate — ⬤, universal characterizations, and Booleans | Final | phase-4 | 2026-07-07 | Atlas |
| [FOOP-43](FOOP-43.md) | Search settlement — miss settles by anchoring; SFF-marked → ECONSTANIC (foundational keystone) | Superseded (merged into FOOP-93) | phase-2 | 2026-07-09 | Atlas |
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
| [FOOP-54](FOOP-54.md) | Einmo — comparison arm of FOOP-92 (mimo-opencode vs claude-code implementation comparison) | Complete | meta | 2026-06-26 | Sisyphus |
| [FOOP-64](FOOP-64.md) | Migrate UBCa snapshot tests to a hierarchical einmo suite | Draft | meta | 2026-07-14 | Atlas |
| [FOOP-74](FOOP-74.md) | FIRID — atomic per-Fir identity for constanic-clone cycle detection | Draft | phase-2 | 2026-07-11 | Atlas |
| [FOOP-84](FOOP-84.md) | Search Engine Refactor — the authoritative search specification, and detachment on it | Draft | phase-2 | 2026-07-28 | Atlas |
| [FOOP-94](FOOP-94.md) | Brane NK only when all constituents are NK — remove any-NK contamination | Draft | phase-2 | 2026-07-14 | Atlas |
| [FOOP-05](FOOP-05.md) | fir module decomposition — fir_base, fir_search_base, one file per FIR kind | Draft | phase-2 | 2026-07-14 | Atlas |
| [FOOP-15](FOOP-15.md) | Secured interactive einmo review — attested inspection of einmos and their perspectives | Draft | meta | 2026-07-14 | Atlas |
| [FOOP-25](FOOP-25.md) | EinmoReview — a thread-safe review-session object; thin bash, server, and dhtml frontends | Superseded (by einmo repo EIMP-1) | meta | 2026-07-19 | Atlas |
| [FOOP-35](FOOP-35.md) | Ship einmo — own repository, crates.io registration, and a working `cargo einmo test` | Draft | meta | 2026-07-19 | Atlas |
| [FOOP-45](FOOP-45.md) | Deadbrane — useless-element detection and FirID cloning semantics (renumbered from FOOP-84) | Draft | phase-2 | 2026-07-14 | Hephaestus |

---

## By Status

### Complete

- [FOOP-92](FOOP-92.md) — Einmo (marked Complete as it stands 2026-07-14: MVP + hardening merged
  at 9bbdaf43; gates + console-review re-homed into FOOP-64; serve/SPA, MCP, algorithm corpus,
  use-case validation deferred to future FOOP(s))
- [FOOP-23](FOOP-23.md) — Value search + contexted `&`-searches. Verified against code 2026-07-14:
  one-engine `ContextfulSearch` (all six predicates), `FoolRefFir` two-child invariant, `&`
  parsing, D-phase backfits — all merged on `jia` with approved `value_search_*`/`contexted_*`/
  `foop_23_comprehensive` snapshots. Plan checkboxes were never maintained; completion attested
  by code + signed corpus.
- [FOOP-54](FOOP-54.md) — Einmo comparison arm: identical spec to FOOP-92, implemented by
  mimo-opencode while claude-code (Claude Opus 4.8) implemented FOOP-92 (both under two hours);
  neutral-agent analysis in FOOP-54.md §"Post implementation comparison" chose FOOP-92, which was
  hardened and merged (9bbdaf43). Follow-up completed 2026-07-14: the §9 Best Practices Review is
  folded into `rust_instructions.md` (all 15 recommendations + task-indexed §2 Task guides). No
  open items.

### Final

- [FOOP-1](FOOP-1.md) — FOOP Purpose, Process, and Format
- [FOOP-2](FOOP-2.md) — Remove if-then-else from the language
- [FOOP-4](FOOP-4.md) — Bare identifiers compile to anchored regex SearchFirs
- [FOOP-5](FOOP-5.md) — Compile-time vs evaluation-time work
- [FOOP-32](FOOP-32.md) — Repair rudimentary FVM evaluation and Sequencer formatting bugs
- [FOOP-33](FOOP-33.md) — Creation Postulate → Booleans — `⬤` creation (ASCII alias `{*}`), three-valued default equality via value search (`Equality::{Equal,NotEqual,Unknowable}`), `Identifier`/minimal `Characterizations`, null-characterized name constants (`get_value()`→`NK("'…redefined")`), `system.foo` as the built-in root brane defining `'True`/`'False` (ready to implement)

### Draft

- [FOOP-72](FOOP-72.md) — Foolish Numbering System (FNS) and Snapshot Test Organization
- [FOOP-82](FOOP-82.md) — UBCa Code Review — Findings and Recommended Changes
- [FOOP-03](FOOP-03.md) — Repository Cleanup — dead code removal, workspace flatten, `jia` rename (blocked, see FOOP-62)
- [FOOP-23](FOOP-23.md) — Value search + contexted `&`-searches — `~=`/`?=`, expression patterns, `&`-prefix navigation from a statement (`FoolRefFir`). **Semantics superseded by the Search Engine Refactor FOOP** (2026-07-28) — FOOP-23 remains authoritative only for grammar productions, the approval-test-input catalog, Rejected Alternatives, and its bug-fix Appendix.
*(Implementation-ordered batch built on FOOP-33; renumbered 2026-07-09 so number ≈ impl order.
**Search-engine sub-ordering corrected 2026-07-28 — see the explicit list immediately below.**)*
- [FOOP-43](FOOP-43.md) — Search settlement: **anchored miss → NK** (unchanged — an anchored search proves absence), **unanchored miss → ECONSTANIC** (unchanged), **SFF-marked search → ECONSTANIC regardless of anchoring** (the actual change — withheld candidates leave a search deferrable, not dead) + the NK-propagation fix (a deepen on an *unresolved* anchor waits instead of forcing NK) + **coordination removes search context** + the `EconstanicReason` tag (foundational keystone; prereq for FOOP-63/73/34 and the Search Engine Refactor)
- [FOOP-53](FOOP-53.md) — Computed index `#${...}` (evaluate brane, tail as number, run `#`; self-contained early win)
- [FOOP-63](FOOP-63.md) — Primitive Characterization: `i'`/`s'`/`f'` type system; characterization = type-tag + search-demand (brane WOCONSTANIC-waits); needs FOOP-33 + FOOP-43
- [FOOP-73](FOOP-73.md) — Boolean operators and/or/not/nor/xor as **Foolish truth-table searches** (no privileged layer; FVM-compute fallback); needs FOOP-33
- [FOOP-83](FOOP-83.md) — Integer math: exponent `**` + comparisons `< > <= >=` returning True/False (needs FOOP-33/73; `*`/`%` already done)
- [FOOP-84](FOOP-84.md) — **Search Engine Refactor — now the required foundation for the rest of this search-family batch, land it before FOOP-93/FOOP-04/FOOP-14/FOOP-24.** Supersedes FOOP-23/FOOP-24 as the authoritative search spec (absorbs the full operator table, `FoolRefFir` shape, name+value atomicity rule, and cursor-source×predicate framing so downstream FOOPs can cite it alone); unifies `ab_search_with_engine`/`BraneFir::_ab_search` into one `AncestralNavigator`; introduces per-candidate, innermost-to-outward boundary evaluation (`CopyMode`/`BoundaryEffect`, replacing `Scope.has_ancestral_sfm`) that FOOP-24 builds on; **§2.2.0 scope rule** — a marker affects only a backward/ancestral search originating inside it, only at the outward boundary crossing (never contexted `&` searches, never locally-resolving searches); documents `contexted ⟹ anchored` as permanent policy. **Two halves with different risk: Part 1 + §2.2 are behavior-preserving (no snapshot may change); §2.3–§2.5 are a deliberate semantic change with expected SF/SFF snapshot churn** — land as separate commits. **Hard dependency on FOOP-43** (§1.5 settlement rule and Component 3's `EconstanicReason::Detached`, which §2.4.1 now requires).
- [FOOP-93](FOOP-93.md) — Search predicates: inverse matcher `!` + matcher boolean operators `&&`/`||` (compiler-hard-coded matcher-outcome ops; SearchPredicate `And`/`Or`/negate). **Needs FOOP-84** (extends its `SearchPredicate`/de-duplicated Navigator; the old "shares a locus with detachment" note is corrected — orthogonal to FOOP-24, no relative ordering needed between the two).
- [FOOP-04](FOOP-04.md) — Cascading connector `|` (fallback between whole searches; `CascadingSearchFir`, shared-fixed-anchor semantics; needs FOOP-43). **Needs FOOP-84** (builds on its `FoolRefFir`/contexted-resume restatement and de-duplicated Navigator).
- [FOOP-14](FOOP-14.md) — All-results `~~`/`??` (doubled operators collect into a brane; tokens already lexed). **Needs FOOP-84** (collect-mode scan runs over its `AncestralNavigator`; composes for free with FOOP-24's `Detach` filtering — no special handling needed, see FOOP-14's Composition note).
- [FOOP-24](FOOP-24.md) — **Coordination detachment** = parameterized SF/SFF marker (`[patterns]<...>` / `[patterns]<<...>>`; the `Detachment` struct, `decide_to_detach`, `[patterns]` parsing). **The live spec for the feature** — renamed from plain "Detachment" per FOOP-84 §Part 3. FOOP-84 supersedes its *mechanism* only (Implementation Plan → Phase A's `_ab_search`-override, and the scan-loop prefilter locus) — build on FOOP-84 Part 2's `resolve_boundary_effect`/`CopyMode` instead. "Nested markers" resolved (FOOP-84 §2.3/§2.6); "Exclusive detachment" reframed under "Required Searches". Scope is narrow (FOOP-84 §0.6/§2.2.4): **affects only descendant searches of the marker, as they cross the marker's boundary outward** — never contexted (`&`) searches, never locally-resolving ones. **Needs FOOP-84** (hard dependency) **and FOOP-43** (its SFF-marked→ECONSTANIC rule + `EconstanicReason::Detached`, so full-detachment exhaustion defers rather than settling NK). Independent of FOOP-93/FOOP-04/FOOP-14.
- [FOOP-34](FOOP-34.md) — Recursion Upgrades (**standalone research**; write ~1–2 dozen algorithms first; after the full search suite — FOOP-84/93/04/14/85; `↑`; no cycle detection)
- [FOOP-44](FOOP-44.md) — Macros (**standalone research**; brane-transforms-brane vs expansion phase; leans on FOOP-14 + characterizations)
- [FOOP-64](FOOP-64.md) — Migrate UBCa snapshot tests to `foolish-ubca/einmo_suite/` (hierarchy `foop/<N>/`, `lang/…`, `regression/`; signed `.einmo` per FOOP-92; dual-home rule; 9 new combination tests; fills the sort-key-46 gap left by FOOP-74)
- [FOOP-74](FOOP-74.md) — FIRID (atomic per-Fir instance counter) + thread-local in-flight clone stack; `eprintln!` alarm when `constanic_clone_at` re-enters an already-in-progress FIRID (detection/visibility only, not a language semantic — distinct from FOOP-34's "no recursion-cycle detection" language-design stance)
- [FOOP-45](FOOP-45.md) — Deadbrane (useless-element detection: directly useless, transitively
  useless, fixed-point algorithm) + FirID cloning semantics (pins Constant/Independent →
  Rc::clone identity-sharing, non-constanic → new FIRID). **Renumbered from FOOP-84 on
  2026-07-29** — it collided with the Search Engine Refactor, which keeps 84.
- [FOOP-05](FOOP-05.md) — fir module decomposition (Track 1, right after FOOP-64): mechanical
  zero-behavior-change split of `fir_kinds.rs`/`fir_trait.rs` into `fir_base.rs`,
  `fir_ref.rs`, `fir_search_base.rs`, and one `firs/<kind>_fir.rs` per FIR kind — unblocks
  Tracks 2 ∥ 3
- [FOOP-15](FOOP-15.md) — Secured interactive einmo review (re-homes FOOP-92's deferred
  serve/SPA + MCP + perspective rendering; phased R1 read-only → R2 attested actions → R3
  perspectives → R4 MCP; build-up goal)
- [FOOP-25](FOOP-25.md) — **Superseded, 2026-07-29**: einmo was extracted into its own repository
  (`~/yolo/einmo`); this unimplemented design was ported there as `EIMP-1` (adapted for einmo's
  own EIMP process — no worktree/`jia` mechanics). Implement against the einmo repo's `EIMP-1`
  plan, not this one. Original abstract, kept for the historical record: EinmoReview session
  object (thread-safe review state: per-reviewer replace-not-stack decisions, single-flight
  verified cache, journal; signing deliberately a SEPARATE `Signer` object — individual or batch
  from one passphrase entry; server API over UDS; poor_einmo.sh reduced to a thin client; first
  dhtml frontend; the session layer FOOP-15 attaches to). §S.11 adds a LAYERED post-quantum
  section attestation in its own `CorpusSigner` object (conservative SPHINCS+/SLH-DSA over a
  stage's manifest+byte-joined files, same passphrase as the Ed25519 stamps, ON TOP of them not
  replacing; default massively-parallel one-buffer read + a tested streaming alternative; crypto
  core + tests only this FOOP)
- [FOOP-35](FOOP-35.md) — Ship einmo as a dual product: library (`einmo = "0.1"`, audited pub API,
  missing_docs-clean) + installable cargo command (`cargo install einmo` → `einmo`/`cargo-einmo`,
  new `einmo test` verb with checked/verified levels and CI exit codes); sequential walkthrough of
  the decisions/registrations (name check, own repo via filter-repo, crates.io account/publish,
  docs.rs) plus the Rust testing battery (proptest, cargo-fuzz on untrusted parsers, mutants,
  deny/audit, MSRV, coverage)
- [FOOP-94](FOOP-94.md) — Brane NK only when ALL constituents are NK (flip `_decide_nyes_due_to_children` cascade: any-NK+rest-constant → CONSTANT, not NK; operator NK propagation and search semantics untouched; ~34 brane-NK snapshots to re-review)

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

- [FOOP-62](FOOP-62.md) — UBCa Two-Store ProtoBrane Tree and Uniform Two-Phase Stepping (**Final** 2026-07-03; UBC retired, UBCa is the sole engine, merged to `jia`)

### Implementing

(none — FOOP-9 and FOOP-21 deprecated 2026-07-03; see Deprecated section)

### Deprecated

- [FOOP-41](FOOP-41.md) — UBCb parity plan (target removed by FOOP-03; deprecated 2026-07-14)
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

- [FOOP-43](FOOP-43.md) — Search settlement (merged into FOOP-93 as components 43-C1/C2/C3; full
  discussion preserved in the file)
- [FOOP-3](FOOP-3.md), [FOOP-6](FOOP-6.md), [FOOP-7](FOOP-7.md), [FOOP-8](FOOP-8.md) — early
  UBC-era semantics docs (2026-05, least-reliable provenance tier): substance realized in UBCa
  (ConcatBrane/FOOP-13A; depth-first stepping; constanic-clone contract; interior mutability +
  Weak parents). Superseded 2026-07-14 with per-file banners — do not cite for current mechanics
- [FOOP-81](FOOP-81.md) — Enhanced SnapshotSuite with HumanizingSequencer and SequenceableFir (Superseded)
- [FOOP-52](FOOP-52.md) — FVM scope/search rework + snapshot bug repairs (Superseded by FOOP-62:
  plan targets the retired UBC engine; its architecture and bug fixes are realized in UBCa and
  pinned by the approved corpus)

---

## By Phase

### meta

- [FOOP-1](FOOP-1.md), [FOOP-31](FOOP-31.md), [FOOP-41](FOOP-41.md), [FOOP-71](FOOP-71.md), [FOOP-81](FOOP-81.md), [FOOP-22](FOOP-22.md), [FOOP-92](FOOP-92.md), [FOOP-03](FOOP-03.md), [FOOP-54](FOOP-54.md), [FOOP-64](FOOP-64.md), [FOOP-15](FOOP-15.md), [FOOP-25](FOOP-25.md), [FOOP-35](FOOP-35.md)

### phase-0

- [FOOP-72](FOOP-72.md)

### phase-1

- [FOOP-2](FOOP-2.md), [FOOP-4](FOOP-4.md), [FOOP-5](FOOP-5.md), [FOOP-9](FOOP-9.md), [FOOP-21](FOOP-21.md)

### phase-2

- [FOOP-6](FOOP-6.md), [FOOP-7](FOOP-7.md), [FOOP-8](FOOP-8.md), [FOOP-01](FOOP-01.md), [FOOP-11](FOOP-11.md), [FOOP-51](FOOP-51.md), [FOOP-61](FOOP-61.md), [FOOP-32](FOOP-32.md), [FOOP-42](FOOP-42.md), [FOOP-52](FOOP-52.md), [FOOP-62](FOOP-62.md), [FOOP-82](FOOP-82.md), [FOOP-13](FOOP-13.md), [FOOP-23](FOOP-23.md), [FOOP-43](FOOP-43.md), [FOOP-53](FOOP-53.md), [FOOP-83](FOOP-83.md), [FOOP-93](FOOP-93.md), [FOOP-04](FOOP-04.md), [FOOP-14](FOOP-14.md), [FOOP-24](FOOP-24.md), [FOOP-74](FOOP-74.md), [FOOP-84](FOOP-84.md), [FOOP-94](FOOP-94.md), [FOOP-05](FOOP-05.md)

### phase-3

- [FOOP-3](FOOP-3.md), [FOOP-91](FOOP-91.md)

### phase-4

- [FOOP-33](FOOP-33.md), [FOOP-63](FOOP-63.md), [FOOP-73](FOOP-73.md)

### phase-5

- [FOOP-34](FOOP-34.md)

### phase-6

- [FOOP-44](FOOP-44.md)

---

## Implementation Roadmap (2026-07-14, Atlas + Claude Code)

The open backlog, organized into **tracks**. A track is internally *sequential* (its FOOPs touch
the same code, so parallel worktrees inside a track would merge-conflict); distinct tracks are
*parallelizable* (different files or different layers). Ordering below states the reason for
every dependency — feature-level (semantics one FOOP assumes from another) or code-level (files
both would edit).

**Provenance calibration rule (Atlas):** FOOPs were written by many agents of differing ability.
Modern FOOPs (the 2026-07 batch and later) are reliable and believable; early UBC-era FOOPs
(2026-05, hc-era) are not to be cited for current mechanics — they are Superseded with banners.
When an old FOOP and the UBCa code disagree, the code and the modern FOOPs win.

### Track 0 — the gate (FOOP-64). REQUIRED BEFORE ANYTHING ELSE.

**Why first, and why alone:** the insta `approval_all` gate is structurally red — generation
embeds a wall-clock `generated:` timestamp inside the signed, byte-compared content, so a fresh
run can never byte-match the stored corpus (verified 2026-07-14: 161/161 snaps diverge,
signature-envelope only, content byte-identical; signing-key drift `eb9604b1…`→`dc5f586c…`).
The project's own Development Rules forbid starting Phase+ work on a red suite, so every other
track is blocked by definition. Einmo fixes this structurally: its `compare` checks INPUT/OUTPUT
sections only — STAMPS and metadata (where timestamps and keys live) are excluded — so churn
cannot redden the gate. FOOP-64 also absorbs FOOP-92's Phase 11 (gate glue) and Phase 12
(`console-review`), which the migration needs operationally (the initial ~162-file promotion is
exactly a console-review workload). Deliverables: `run_einmo_tests` green, hierarchical
`einmo_suite/` (`foop/<N>/`, `lang/…`, `regression/`), cross-validation against the old corpus,
**foolish-core's corresponding `einmo_suite/`** with the same organizational rules (as part of
this FOOP), the **two-tier signing gate** over both suites — the **feature-complete test
suite** (development: output↔**checked** correspondence, computer key acceptable) and the
**merge-ready test suite** (PR merge: output↔**verified** correspondence via
`einmo_verified_gate` + `.github/workflows/einmo-gates.yml` + human-enabled branch protection,
human reviewer key required and computer-key bypass scanned — same suite directories, different
requirements, different public keys) — AGENTS.md/foop.md/skills updated to mandate the
checked-stage gate, and — **completion-blocking** — secure migration off insta entirely:
`foolish-parser`'s inline insta usage migrated too, `.snap` corpora deleted (human act),
`insta` removed from every `Cargo.toml` (`cargo tree -i insta` finds nothing).

### Track 1 — fir module decomposition (FOOP-05). Right after Track 0, before Tracks 2/3 fork.

Resolves P4 (Atlas-approved 2026-07-14): mechanically split `fir_kinds.rs` and `fir_trait.rs`
into idiomatic modules — shared default behavior in `fir_base.rs` (the "base class"), the
search engine in `fir_search_base.rs`, `FirRef` + extension traits + stepping driver in
`fir_ref.rs`, and one `firs/<kind>_fir.rs` per FIR kind. Move-only, one module per commit,
tests green per commit, snapshot-invisible (Track 0's einmo gate is the byte-identity oracle).
After this, Tracks 2 and 3 edit disjoint files and run genuinely parallel.

### Track 2 — the search family. One track, strictly sequential, four FOOPs.

All five edit the same code: the `contextful_search` module in `fir_kinds.rs` (engine,
`SearchPredicate`, `BraneNavigator`), `SearchFir` settle paths, lexer/parser tokens, compiler
lowering. **Internal order and why:**

1. **FOOP-93** — settlement semantics (absorbed FOOP-43: miss→ECONSTANIC everywhere;
   coordination strips the `FoolRefFir` position; `EconstanicReason` tag) + predicate algebra
   (`!`, `&&`, `||`). Settlement goes first *inside* the FOOP because the predicate tests assume
   the new miss outcomes. This is the family's one big snapshot-flipping review session.
   *Encapsulation:* `SearchPredicate` stays `pub(crate)`; algebra lands as composite variants
   (`Not(Box<SearchPredicate>)`, `And/Or(Box<_>, Box<_>)`) so `matches(candidate, ctx)` remains
   the single evaluation door.
2. **FOOP-04** — cascading `|`. Feature-depends on 93: a cascade falls through on
   Miss-*reason* ECONSTANIC (reads 43-C3's tag) and does NOT fall through on found-NK — the
   distinction only exists after 93.
3. **FOOP-14** — all-results `~~`/`??`. After 04 so cascade semantics are settled over
   single-result searches before multi-result shapes exist. **P2 (design problem found, resolve
   in spec before coding):** the FOOP-23 two-child invariant (`[clone, FoolRefFir]`) is
   single-result; all-results must define its result shape (per-element position pairs vs a
   positionless value brane per 43-C2) and how a following `&`-search reads it.
4. **FOOP-53** — computed index `#${…}`. Small; reuses the value-pattern child-settling
   machinery FOOP-23 shipped, feeding `SearchPredicate::Index`. Slot anywhere after 93; placed
   here to keep review sessions rhythm.
*(Removed 2026-07-14: the fruit-picker `$*` FOOP was withdrawn by Atlas before commit — too
many unsettled semantics. Its companion idea, the **mature step-budget**, is parked with its
corrected design recorded: two-dimensional control — a flowing per-call step budget AND the
existing `MAX_DEPTH`, kept separately; exhaustion does NOT settle NK — FIRs simply remain
BRANING and the FVM returns an **incomplete status** for the calling program to handle.
Revive as its own FOOP when a consumer arrives; FOOP-34 recursion is the expected one.)*

### Track 3 — the state machine. Sequential: 94 → 74 → 84 → 24.

Touches `_decide_nyes_due_to_children`, `ProtoBrane`, `constanic_clone_at` — different sites
than Track 2's engine module, but the **same file** (`fir_kinds.rs`), so Tracks 2 and 3 in
parallel worktrees will merge with friction (see P4). Order and why:

1. **FOOP-94** — brane NK only when all constituents are NK. Quick, isolated to the shared
   classifier; do first to get its ~mixed-NK snapshot review out of the way.
2. **FOOP-74** — FIRID, **by itself** (Atlas 2026-07-14): standalone diagnostic
   infrastructure, no other FOOP gates on it.
3. **FOOP-45** — Deadbrane (renumbered from FOOP-84 on 2026-07-29), **without any FIRID
   dependency** (its FirID-cloning-semantics component is removed/deferred — see the banner in
   FOOP-45.md). Useless-element detection only.
4. **FOOP-24** — detachment (parameterized SF/SFF markers). Last because it straddles both
   tracks: a search *prefilter* (Track 2's engine) plus clone/recoordination machinery
   (Track 3), and it consumes 93's `EconstanicReason::Detached`. Schedule after both tracks'
   cores merge.

### Track 4 — independent, parallel with Tracks 2/3.

- **FOOP-13 Phase B** — MAX_BRANE_SIZE auto-sizing. Phase A (non-merging ConcatBrane) is merged;
  Phase B (`UbcaConfig`, `compile_with`, the AST→AST chunking rewrite) lives in `compiler.rs` /
  `evaluator.rs` — nearly zero overlap with the fir_kinds tracks, so it can run in parallel any
  time after Track 0. (`UbcaConfig` is also the future home of the parked step-budget's
  allowance knob — one config type, fields added as features land.)
- **FOOP-82** — UBCa code review findings. **Split disposition:** (a) re-triage the correctness
  findings first — several (HeadTail/Index settle path) may already be fixed by FOOP-23's
  backfits (the review predates them; calibrate reliance accordingly); still-live bug fixes are
  claimed by the track that owns the touched site (settle paths → Track 2/3). (b) The
  architectural findings become the optional **fir_kinds decomposition** (see P4).

### Track 5 — phase-4 language features. Sequential: 33 → 83 → 73 → 63.

Parser/lexer + new `Characterizations` machinery + `system.foo` bootstrapping — different layer
from Tracks 2/3 (minor lexer-token merge friction only; coordinate token additions). Order:
**FOOP-33** (⬤ creation, characterizations, Booleans — Final, design frozen) enables
**FOOP-83** (comparisons must produce `'True`/`'False`), which enables **FOOP-73** (boolean
operators as truth-table searches — also leans on Track 2's value-search, already Complete via
FOOP-23), then **FOOP-63** (primitive characterization `i'`/`s'`/`f'`, consumes 33's
`Characterizations` and 93's `EconstanicReason::CharDemand`).

### Parked (research; no track until their prerequisites exist)

**FOOP-34** (recursion — wants Track 2 complete + FOOP-05's budget + FOOP-24), **FOOP-44**
(macros — wants FOOP-14 + characterizations), **FOOP-72** (FNS/snapshot docs — its snapshot-org
half is absorbed by FOOP-64's MAPPING/hierarchy; the FNS half is a docs pass, do opportunistically),
**FOOP-03** (repo-cleanup bookkeeping — finish opportunistically), and **FOOP-15** (secured
interactive einmo review — re-homes FOOP-92's deferred serve/SPA + MCP + perspective
rendering; touches only the einmo crate family so it may run parallel to the FVM tracks any
time after Track 0; phased R1–R4 to build up to it; FOOP-92's algorithm-corpus and use-case
validation remain unhomed until wanted).

### Problems found during this audit (fix-before-95%-done list)

- **P1 — red gate**: see Track 0. Found 2026-07-14; the reason Track 0 exists.
- **P2 — all-results result shape** vs the two-child invariant: design in FOOP-14's spec before
  any code (noted in its file).
- **P3 — (retired with the fruit-picker withdrawal)** identity-based fixpoint termination
  remains a design note for whenever `$*` returns.
- **P4 — RESOLVED as Track 1 (FOOP-05)**: the fir module decomposition, Atlas-approved
  2026-07-14. Line references in open FOOPs predate it; expect one-time churn.
- **P5 — comprehensive-test path re-homing**: FOOP-64 moves the reserved path to
  `einmo_suite/input/foop/<N>/comprehensive.foo`; every open plan (94, 05, …) carries the
  conditional path note until 64 merges.
- **P6 — einmo compare excludes STAMPS/metadata** (verified in `einmo.README.md` and compare
  semantics): this is the property that makes Track 0 the durable fix — record it so nobody
  "fixes" compare into byte-exactness and re-reddens the gate.
- **P7 — checkbox ledgers are unreliable**: FOOP-23 shipped with 10/75 boxes ticked; FOOP-92
  with 2/169. Completion is attested by code + signed corpus, not plans. Future execution
  should tick boxes as it goes (the skills mandate it), but audits must diff spec-named
  artifacts against the tree.

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

**Date**: 2026-07-29 (2)
**Updated By**: Claude Code (Sonnet 5)
**Changes**: **FOOP-25 superseded.** einmo was extracted into its own repository (`~/yolo/einmo`);
FOOP-25's design (`EinmoReview`, unimplemented) was ported there as `EIMP-1`, adapted for
einmo's own EIMP process (no worktree/`jia` mechanics). Updated FOOP-25's frontmatter
(`status: Superseded`, `superseded_by: einmo repo EIMP-1`), added a superseded banner to
`FOOP-25.md`, and formally cancelled `FOOP-25.plan.md` (`[x] Canceled` + `[-]` on every
checkbox, per `foop.md`'s cancellation procedure). Updated this index's table row and Draft-list
prose entry to match.

**Date**: 2026-07-29
**Updated By**: Claude Code (Opus 5)
**Changes**: **Resolved a FOOP-84 number collision** discovered while merging `origin/jia` into
`jia`. Two unrelated specifications had independently claimed FOOP-84: **Search Engine Refactor**
(Atlas, `foop: 48`, present at the merge-base and cited as FOOP-84 by 44 references across
FOOP-04/14/24/93 on both sides) and **Deadbrane** (Hephaestus, `foop: D48`), which landed on
`origin/jia` and overwrote the file — leaving those 44 cross-references dangling on that branch.
Search Engine Refactor **keeps FOOP-84** as the incumbent; Deadbrane is **renumbered to FOOP-45**
(`foop: D54`, per `foop_check.py gen_next`) in new `FOOP-45.md` / `FOOP-45.plan.md`. Deadbrane's
plan had not begun (no checkboxes checked), so its worktree metadata was also brought to current
convention: branch `foop-45-deadbrane` (bare, no `foop/` prefix) and origin/merge target `jia`
rather than `alpha`. Updated the referring FOOP-94 reference, the main table, and the Draft list.
Also merged both sides' additive Draft-section entries (ours: the explicit search-engine
sub-batch order; theirs: FOOP-05/15/25/35/94) and set FOOP-43's row to **Superseded (merged into
FOOP-93)** — matching the banner in FOOP-43.md itself — while keeping its newer, more accurate
title.

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
(3) **Marker scope rule** (FOOP-84 §0.6/§2.2.4) recorded on the FOOP-84 and FOOP-85 entries: a marker
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

**Date**: 2026-07-19
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5
**Changes**: Added FOOP-35 (ship einmo as library + cargo command: `einmo test` verb, own repo via
filter-repo, crates.io registration walkthrough, Rust testing battery; plans target `jia`). Also
FOOP-25 plan origin confirmed as `jia`. Earlier same day: added FOOP-25 (EinmoReview session object —
thread-safe review state with a separate `Signer` for individual-or-batch signing, review server,
thin poor_einmo.sh client, dhtml frontend; the session layer for FOOP-15). Both Draft, `begun: [ ]`.

**Date**: 2026-07-14
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5
**Changes**: FOOP-64 scope extension (Atlas): added the **two-tier signing gate** (development
tier = einmo **checked**-stage correspondence, mandated in AGENTS.md/foop.md/skills; PR-merge
tier = einmo **verified**-stage correspondence via `einmo_verified_gate` + GitHub workflow +
human-enabled branch protection — same `einmo_suite/` directory, different requirements and
public keys, with computer-key-bypass scan), the mandatory human verified-signing session, and
the hard completion criterion: FOOP-64 is not complete until the repo is securely off insta —
`foolish-parser`/`foolish-core` insta tests migrated too, `.snap` corpora deleted (human act),
`insta` gone from all Cargo.tomls (`cargo tree -i insta` empty). Track 0 deliverables updated.
Second extension same session: **foolish-core's snapshot_tests migrate to a corresponding
`foolish-core/einmo_suite/` as part of this FOOP** (same hierarchy/placement/dual-home rules,
own MAPPING.md, stale-input flagging via `einmo flag`), and both suites are covered by both
tiers — named the **feature-complete test suite** (checked) and the **merge-ready test suite**
(verified); the human signing session covers both corpora.

**Date**: 2026-07-14
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5
**Changes**: Roadmap round 2 (Atlas corrections). **Fruit-picker `$*` FOOP withdrawn** before
commit; its number reissued: **FOOP-05 is now the fir module decomposition** (Track 1, right
after FOOP-64: `fir_base.rs` / `fir_ref.rs` / `fir_search_base.rs` / one `firs/<kind>_fir.rs`
per kind; move-only; unblocks Tracks 2 ∥ 3 — P4 resolved). Step-budget parked with corrected
design recorded in Track 2: two-dimensional control (budget AND `MAX_DEPTH`, kept separately);
exhaustion → FIRs remain BRANING, FVM returns **incomplete status** (never NK). **FOOP-74
(FIRID) scheduled by itself; FOOP-84 (Deadbrane) stripped of its FirID dependency** (banner in
file; FirID cloning semantics deferred). **New FOOP-15** — secured interactive einmo review
(re-homes FOOP-92's deferred serve/SPA + MCP + perspective rendering; phased R1–R4; build-up
goal). Track 0 (FOOP-64) confirmed P0 — dispatched for immediate execution; no interim insta
re-sign.

**Date**: 2026-07-14
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5
**Changes**: Backlog reorganization (Atlas-directed). Added the **Implementation Roadmap**
section: Track 0 (FOOP-64 einmo migration — required before anything else; absorbs FOOP-92's
gates + console-review), Track 2 (search family, sequential: 93 → 04 → 14 → 53 → 05), Track 3
(state machine: 94 → 74 → 84 → 24), Track 4 (13-B + 82 triage, parallel), Track 5 (33 → 83 →
73 → 63), parked research, problems P1–P7, and the provenance-calibration rule (modern FOOPs
reliable; early UBC-era FOOPs not citable). **FOOP-92 → Complete** (as it stands; remnants
re-homed/deferred per its plan note). **FOOP-43 → Superseded** (merged into FOOP-93 as
43-C1/C2/C3; C2 packaging is an agent decision vetoable at BDFL review). **FOOP-3/6/7/8 →
Superseded** and **FOOP-41 → Deprecated** (early-era; per-file banners state what UBCa
realized). **New FOOP-05** — fruit picker `$*` + mature flowing step-budget (sort key 50).
Roadmap notes added to FOOP-04/14/53; FOOP-64 gained the Track-0 banner and absorbed scope.

**Date**: 2026-07-14
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5
**Changes**: Completion audit of FOOPs 23/13/52/92 against code and tests. **FOOP-23 → Complete**
(one-engine search fully merged; plan checkboxes were never maintained — completion attested by
code + approved corpus). **FOOP-52 → Superseded** by FOOP-62 (plan targets the retired UBC
engine; substance realized in UBCa). FOOP-13 remains Brewing (Phase A ConcatBrane merged; the
title feature — Phase B MAX_BRANE_SIZE auto-sizing/`UbcaConfig` — is unimplemented). FOOP-92
remains open pending re-homing of post-MVP remnants (gates/console-review/serve/MCP/algorithm
corpus). Audit also found `approval_all` structurally red: all 161 UBCa snaps diverge
signature-only (embedded `generated:` wall-clock + key drift `eb9604b1…`→`dc5f586c…`); content
byte-identical — motivates prioritizing FOOP-64 (einmo migration).

**Date**: 2026-07-14
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5
**Changes**: Added **FOOP-94** — brane NK only when ALL constituents are NK: flip the shared
`_decide_nyes_due_to_children` cascade so a settled brane with mixed NK/value members classifies
CONSTANT instead of NK (all-NK still → NK; empty brane still → CONSTANT; operator NK propagation
`5 + NK → NK` and all search semantics untouched and pinned by new tests). Quick
investigate-and-flip FOOP; ~34 approved snapshots carry brane-level NK and the mixed-content
subset will need human re-review. Fills sort key 49. Added to main table, By-Status (Draft),
By-Phase (phase-2).

**Date**: 2026-07-14
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5
**Changes**: Registered **FOOP-54** (previously unindexed) as **Complete** — the mimo-opencode
comparison arm of FOOP-92 (identical spec, two agents, both under two hours; neutral-agent
analysis in FOOP-54.md chose FOOP-92 for hardening + merge). Added to main table, new By-Status
(Complete) section, and By-Phase (meta). Its plan is closed with `[-]` cancellations; sole open
item is folding the §9 Best Practices Review into `rust_instructions.md`.

**Date**: 2026-07-14
**Updated By**: Hephaestus / xiaomi/mimo-v2.5-pro
**Changes**: Added **FOOP-84** — Deadbrane (useless-element detection: directly useless, transitively useless, fixed-point algorithm) + FirID cloning semantics refinement (pins Constant/Independent → Rc::clone identity-sharing, non-constanic → new FIRID). Added to main table, By-Status (Draft), and By-Phase (phase-2).

**Date**: 2026-07-14
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5
**Changes**: Added **FOOP-64** — migrate the 162 flat UBCa insta snapshots to a new hierarchical
einmo suite at `foolish-ubca/einmo_suite/` (`foop/<NUMBER>/…` incl. `comprehensive.foo`,
`lang/<category>/…`, `lang/usecases/…`, `regression/…`), signed `.einmo` format per FOOP-92,
dual-home rule for near-identical tests, cross-validation against approved `.snap` RESULTs,
comprehensive-test path re-homed to `einmo_suite/input/foop/<N>/comprehensive.foo`, and nine
proposed feature-combination tests. Fills the deliberate sort-key-46 gap left by FOOP-74, so
`foop_check.py check` passes again. Added to main table, By-Status (Draft), and By-Phase (meta).

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
