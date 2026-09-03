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
| [FOOP-55](FOOP-55.md) | Project Euler 1 — 'mod, 'or, and the repairs that run the first exercise | Draft | phase-4 | 2026-08-07 | Sisyphus / qwen3.8-max |
| [FOOP-65](FOOP-65.md) | The tail concatenator — backtick application that brings the method name to the front | Draft | phase-2 | 2026-08-07 | Sisyphus / qwen3.8-max |
| [FOOP-75](FOOP-75.md) | Assignment Attached Searches — `LHS =SEARCH_SPEC RHS` as sugar for `LHS = RHS SEARCH_SPEC` | Draft | phase-2 | 2026-08-07 | Sisyphus / claude-opus-5 |
| [FOOP-85](FOOP-85.md) | The einmo Foolish separator collides with Foolish block comments | Draft | meta | 2026-08-07 | Sisyphus / claude-opus-5 |
| [FOOP-95](FOOP-95.md) | Add Embryonic and Resequencing EINMO Sections | Draft | phase-2 | 2026-08-08 | Sisyphus / claude-opus-5 |
| [FOOP-16](FOOP-16.md) | foolish-ubca2 — arena-backed FIR storage via copy-migration | Draft | phase-2 | 2026-08-30 | Claude Code / claude-sonnet-5 |
| [FOOP-26](FOOP-26.md) | Carrying FOOP-55's semantics onto foolish-ubca2 — marks, concatenation-as-operator, and the three-beat step | Draft | phase-4 | 2026-09-01 | Claude Code / claude-opus-5 |
| [FOOP-36](FOOP-36.md) | A Foolish-rendering sequencer for foolish-ubca2 — output that parses back in | Draft | phase-4 | 2026-09-01 | Claude Code / claude-opus-5 |
| [FOOP-46](FOOP-46.md) | BraneConcatOp — a rewritten concatenation operator with phased search resolution | Draft | phase-4 | 2026-09-02 | Claude Code / claude-opus-5 |
| [FOOP-56](FOOP-56.md) | NYES groups — one predicate per group, and "settled" qualified everywhere | Draft | phase-4 | 2026-09-02 | Claude Code / claude-opus-5 |

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
- [FOOP-55](FOOP-55.md) — Project Euler 1: make the first exercise run — `'mod` integer modulo (FOOP-33 §5.1 BodyOverride mechanism, integer result), `'or` boolean OR (FOOP-73 preferred pure-Foolish truth-table design, `'or` only), plus documented platform defects D1–D6 (incl. the leading-`_` lexer workaround via `INTERN_` prefix and the `$=`/`=$` sugar findings, Atlas-directed) and exercise defects E1–E5 (Atlas fixing the file). **Depends on FOOP-65** (exercise rewrite uses backtick application)
- [FOOP-65](FOOP-65.md) — The tail concatenator: backtick `` ` `` — `fn`{p1,p2}` ≡ `{p1,p2} fn`; WEAKEST precedence (weaker than brane concatenation; `$`/search suffixes bind inside operands); within a run concatenation is associative so the chain is flat n-ary reversing source order (`f`g`h`a b c` ≡ `a b c h g f`, i.e. `` a`b`c`d e f `` → exactly TWO ConcatenationFirs, `Concat[tail](Concat[juxt](d,e,f), c, b, a)`). **Revised 2026-08-08: NO separate `TailConcatenationFir`** — a `ConcatProvenance` flag on the existing `ConcatenationFir` instead (the separate-FIR design is now Rejected Alternative C); precedence and the reversal resolve in `build_fir`; the flag affects **sequencing only, never evaluation**, and renders backtick form only while all constituents are embryonic. Prerequisite of FOOP-55; **depends on FOOP-95**; non-regression verified (corpus backticks only in comments)
- [FOOP-85](FOOP-85.md) — einmo's Foolish-suite separator `"!!\n"` (the Foolish **line** comment) collides with Foolish's **block** comment `!!!`: every `!!!` line ends with the separator, and `serialize`'s collision check is a plain substring test, so **any** `.foo` using a block comment is unserializable — which fails the whole UBCa suite, not the one file. Fix is one constant: `"\n!!!EINMO!!!\n"`, newline-wrapped so it matches only a whole line. **Backward compatible** — each `.einmo` records its own separator in its header and `parse` reads it from there, so existing baselines keep verifying. Found while gating FOOP-75; verified (einmo 133 pass, `einmo_gate_output` fixed, workspace 3 failures → 2). Change was reverted to keep a clean build; Appendix A holds the diff verbatim
- [FOOP-95](FOOP-95.md) — Add **EMBRYONIC** and **RESEQUENCED** einmo sections. Every test gains a rendering of the program sequenced **before any step** (EMBRYONIC — the only vantage from which FOOP-65's backtick form is visible, and generally a direct view of parser/precedence structure that today only surfaces as value diffs). Landing last, the **Foolish Resequencer** — a *separate* sequencer emitting **parsable** Foolish from the FIR tree (RESEQUENCED) — plus a new **normalization**, checked by two equalities: fidelity (`normalize(resequence(parse(src))) == normalize(src)`) and idempotence; creation *identity* (`⬤`) is explicitly outside what a round-trip can restore. Repairs a real defect: `ConcatenationFir::stmt_count` **mutates** (forces the concatenation join), disagreeing with its own `stmt_at`/`_search_brane` siblings which both guard and decline → split into pure `stmt_count` + explicit `ensure_joined_stmt_count`, ~20 call sites classified individually. Section order becomes `METADATA, OUTPUT, EMBRYONIC, RESEQUENCED, INPUT, COMMENTS, STAMPS`; `EMBRYONIC`/`RESEQUENCED` MUST join `einmo/src/compare.rs`'s always-compared set or they are pinned in name only. Rewrites EVERY baseline (purely additive + reorder) behind two human-gated inspections
- [FOOP-75](FOOP-75.md) — Assignment Attached Searches: `LHS =SEARCH_SPEC RHS` ≡ `LHS = RHS SEARCH_SPEC` over the trigger set `^ $ ~ ? # .`; an **attached search** must be adjacent to the `=` and **space-terminated** (§5), requiring a new `preceded_by_space` flag on `TokenAndLocation` since `column` does not count skipped whitespace (§5.3). Generalizes and repairs the ad-hoc `=$`/`=^` sugars — three measured defects: `=$` yields the whole brane not the tail, `=^` never settles (leaks `Op^(...)`), and postfix `B$` never re-sugars — all dissolved by routing through the existing `IndexFir` (§7). Sequencer gains an anchor-spine walk (§4). No new FIR kind. Documents the `$`-in-pattern ambiguity and the parenthetical terminator (§6); §9 records shared structure with FOOP-65 (neither blocks the other)

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

- [FOOP-6](FOOP-6.md), [FOOP-7](FOOP-7.md), [FOOP-8](FOOP-8.md), [FOOP-01](FOOP-01.md), [FOOP-11](FOOP-11.md), [FOOP-51](FOOP-51.md), [FOOP-61](FOOP-61.md), [FOOP-32](FOOP-32.md), [FOOP-42](FOOP-42.md), [FOOP-52](FOOP-52.md), [FOOP-62](FOOP-62.md), [FOOP-82](FOOP-82.md), [FOOP-13](FOOP-13.md), [FOOP-23](FOOP-23.md), [FOOP-43](FOOP-43.md), [FOOP-53](FOOP-53.md), [FOOP-83](FOOP-83.md), [FOOP-93](FOOP-93.md), [FOOP-04](FOOP-04.md), [FOOP-14](FOOP-14.md), [FOOP-24](FOOP-24.md), [FOOP-74](FOOP-74.md), [FOOP-84](FOOP-84.md), [FOOP-94](FOOP-94.md), [FOOP-05](FOOP-05.md), [FOOP-65](FOOP-65.md), [FOOP-75](FOOP-75.md), [FOOP-95](FOOP-95.md)

### phase-3

- [FOOP-3](FOOP-3.md), [FOOP-91](FOOP-91.md)

### phase-4

- [FOOP-33](FOOP-33.md), [FOOP-63](FOOP-63.md), [FOOP-73](FOOP-73.md), [FOOP-55](FOOP-55.md), [FOOP-26](FOOP-26.md), [FOOP-36](FOOP-36.md), [FOOP-46](FOOP-46.md), [FOOP-56](FOOP-56.md)

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

### Track 6 — the `foolish-ubca2` chain. Strictly sequential: 56 → 36 → 26.

All three edit `foolish-ubca2`, most of it the same file (`fvm_storage.rs`), so they cannot run
in parallel worktrees. The order is dependency-driven, not preference:

1. **FOOP-56** — NYES groups: one predicate per group (`is_preconstanic` with `is_nye` as its
   alias, `is_constanic`, `is_constantew`, `is_conclusive`) and every bare "settled" qualified
   with the group it means. **First because it is vocabulary the other two are written in**,
   and because it is a ~20-site rename that is far cheaper before the others diverge from
   `jia`. No behaviour change; the crate is 134/134 either side.
2. **FOOP-36** — the Foolish-rendering sequencer: `foolish-ubca2` gets its own sequencer whose
   default mode renders FIR as parseable Foolish, and `einmo_suite2` replaces `einmo_suite` as
   the crate's approval suite. **After 56** because its central rule is stated over
   *conclusive* and *inconclusive constanic*. **Before 26** because 26 moves the same 179
   baselines for *semantic* reasons: with rendering already in Foolish, 26's diffs are readable
   as language rather than as FIR dumps, and its reviewers can predict expected output from its
   own spec. FOOP-36 moves no FIR, no step rule and no step count, so it costs 26 nothing.
3. **FOOP-26** — marks, concatenation-as-operator, the three-beat step. Last: it is the only
   one of the three that changes what programs *mean*, so it should land onto a suite that is
   already readable and a vocabulary that is already settled.

**FOOP-46** (BraneConcatOp) overlaps FOOP-26's concatenation work and FOOP-36 §3.2's
concatenation *rendering*; sequence it against this track rather than in parallel — see its own
references.

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

## Ergonomics

Surface-syntax and developer-experience work: how Foolish *reads* and how a Foolisher sees
what their program actually became. These are not evaluation-semantics changes — every one of
them is sugar, provenance, or visibility, and none alters what a settled program computes.
Grouped here because they share a layer — lexer/parser, the compiler's
Foolish→FIR translation, and the sequencer — rather than the FIR/stepping machinery Tracks 2/3
own.

- **[FOOP-65](FOOP-65.md)** — **the tail concatenator** (backtick). `fn`{p1,p2}` ≡
  `{p1,p2} fn` — the method name comes first, reading like an ordinary call. WEAKEST operator
  (juxtaposition groups inside each operand); a run is a flat n-ary chain reversing source
  order (`f`g`h`a b c` ≡ `a b c h g f`). **No new FIR kind** — a `ConcatProvenance` flag on
  the existing `ConcatenationFir`, set in `build_fir` where the reversal also happens; the flag
  affects **sequencing only, never evaluation**. Backtick form renders only while all
  constituents are embryonic. Prerequisite of FOOP-55; **depends on FOOP-95** for the vantage
  from which that rendering is visible.
- **[FOOP-75](FOOP-75.md)** — **assignment attached searches**. `LHS =SEARCH_SPEC RHS` ≡
  `LHS = RHS SEARCH_SPEC` over `^ $ ~ ? # .`; generalizes and *repairs* the ad-hoc `=$`/`=^`
  sugars (three measured defects). No new FIR kind — routes through the existing `IndexFir`;
  the sequencer gains an anchor-spine walk to re-sugar in the reverse direction.
- **[FOOP-95](FOOP-95.md)** — **add EMBRYONIC and RESEQUENCED einmo sections**. Every test
  gains a rendering of the program *before any step* (EMBRYONIC), and — landing last — the
  **Foolish Resequencer** emitting *parsable* Foolish from the FIR tree (RESEQUENCED), checked
  by two equalities against a new **normalization**: fidelity to the input, and idempotence.
  Turns the whole corpus into a round-trip property test of the parser/FIR-gen phases. Also
  repairs a real defect — `ConcatenationFir::stmt_count` **mutates** (forces the join),
  disagreeing with its own `stmt_at`/`_search_brane` siblings — by splitting it into a pure
  `stmt_count` and an explicit `ensure_joined_stmt_count`.

**Ordering.** FOOP-95 before FOOP-65 (65 needs 95's pre-step vantage to test its rendering;
nothing in 95 depends on 65). FOOP-75 is independent of both — its §9 records the shared
structure with FOOP-65 and confirms neither blocks the other. All three touch the
lexer/parser, so coordinate token additions (FOOP-65 adds `Backtick`; FOOP-75 adds
`preceded_by_space` to `TokenAndLocation`, which FOOP-65's new token arm must populate if 75
lands first).

**Scope caution.** FOOP-95 rewrites every baseline in the einmo suite (new sections + a
section reorder) behind two human-gated inspections, and its §6 (the Resequencer) is flagged
as a clean cut point should it need to become its own FOOP.

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

**Date**: 2026-09-02
**Updated By**: Claude Code / claude-opus-5
**Changes**: Added **FOOP-26**, **FOOP-36**, **FOOP-46** and **FOOP-56** to the table and the
phase-4 list, and added **Track 6 — the `foolish-ubca2` chain**, strictly sequential
**56 → 36 → 26**. FOOP-56 (NYES groups: one predicate per group, every bare "settled"
qualified) goes first because it is the vocabulary the other two are written in and is a
~20-site rename best done before they diverge. FOOP-36 (the Foolish-rendering sequencer, and
`einmo_suite2` replacing `einmo_suite`) goes before FOOP-26 so that FOOP-26's baseline diffs
read as language rather than FIR dumps; FOOP-36 moves no FIR, step rule or step count, so it
costs FOOP-26 nothing. FOOP-46 (BraneConcatOp) overlaps both and should be sequenced against
the track rather than run in parallel. Prior entry: added FOOP-16 (foolish-ubca2 — arena-backed
FIR storage via copy-migration).
