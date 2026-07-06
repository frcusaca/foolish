---
foop: 32
title: FOOP-23 Implementation Plan — Value search (~=/?=, expression patterns, search-anchored search)
status: Draft
created: 2026-07-04
---

# FOOP-23 Implementation Plan

**Read `FOOP-23.md` first — this plan assumes the specification's context.**

## Worktree

All values expanded (per foop.md):

```
WORKTREE_ORIGIN_BRANCH=jia
WORKTREE_ORIGIN_PATH=/home/hcbusy/foolish-rust
WORKTREE_BRANCH_NAME=foop-23-value-search
WORKTREE_FULL_FS_PATH=/home/hcbusy/tmp/foolish-worktrees/foop-23-value-search
```

## Scope

Parts A → B → C of FOOP-23, strictly ordered. Each part is reviewable on its own (tests +
snapshots) before the next begins.

**Files to modify:**
- `foolish-parser/src/token.rs`, `lexer.rs`, `parser.rs`, `ast.rs` — `~=`/`?=` tokens, the `&`
  contexted-prefix token, value-search grammar (A.2), contexted-search grammar (C.3), AST node(s)
- `foolish-ubca/src/fir_kinds.rs` — the `ContextfulSearch` engine (generalized `SearchFir` with
  `CursorSource` + `SearchPredicate`), `FoolRefFir`; legacy `IndexFir`/`HeadTailFir`/`SearchFir`
  scans backfitted onto the engine in Phase C-backfit
- `foolish-ubca/src/proto_brane.rs` — `push_search_result` two-entry invariant
- `foolish-ubca/src/compiler.rs` — AST → FIR for value search and contexted search
- `foolish-ubca/src/fir_trait.rs` — `get_my_brane` doc-comment ("home brane" ≡ "brane of")
- `foolish-ubca/snapshot_tests/input/` — new approval inputs (A.5, B.1, C.4)
- alarm plumbing for `VALUE-SEARCH-UNSUPPORTED-PATTERN`
- Docs (Phase D): `AGENTS.md`, `README.md`, `docs/howto/01_howto_foolish.foo`,
  `02_howto_foolish_more.foo`, vintage `NAMES_SEARCHES_N_BOUNDS.md` / `ADVANCED_FEATURES.md`
  (superseded banners)

**Out of scope (deliberately NOT added):** the `&.` operator (Rejected Alt. F — `.` already
deepens); any new search-result naming sugar (Rejected Alt. G — keep only the existing
`a$=b`/`a^=b`; new forms need Humanizing-Sequencer work). No Humanizing-Sequencer changes are
required by this FOOP.

**Files NOT touched:** approved `.snap` files (human workflow only), foolish-core evaluation
semantics, UBC (retired), the Humanizing Sequencer (no new naming sugar).

---

## Phase 0: Setup

- [ ] Verify all workspace tests pass on `jia` before starting (hard project rule)
- [ ] Create worktree at /home/hcbusy/tmp/foolish-worktrees/foop-23-value-search with branch
      `foop-23-value-search` from `jia`:
      `git worktree add -b foop-23-value-search /home/hcbusy/tmp/foolish-worktrees/foop-23-value-search`
- [ ] Check the `begun` box in FOOP-23.md frontmatter and commit in the origin directory stating
      work has commenced; from this point ALL work (including FOOP/plan edits) happens ONLY in
      the worktree

## Implementation strategy (Atlas): new-feature-first, backfit last

Per Atlas, the cleanest route is **one matching engine that receives candidates** — a
`ContextfulSearch` engine (spec §C.3.2: cursor-source × predicate, feeding candidates to a match
predicate). Build and prove it with the **new** features (value search, then contexted search),
which are free of legacy snapshots and so can drive the engine's shape without risk. **Only
after** the new features pass do we **backfit**: replace the internals of the existing
contextless `SearchFir`/`IndexFir`/`HeadTailFir` with delegations to the engine, **piecewise,
keeping all tests green at each step**. This inverts the naive order (don't refactor the working
contextless code up front); the backfit is a behavior-preserving refactor gated on **zero
snapshot diffs**.

Phase order: A0 (engine skeleton) → A (value predicate) → B (expression patterns) → C0/C1
(sequencing + FoolRefFir) → C2 (contexted cursor-source) → **C-backfit** (migrate legacy scans) →
D (docs) → E (merge).

## Phase A0: The `ContextfulSearch` engine — Navigator + Matcher (skeleton)

The engine is a single loop over a candidate stream, split into two collaborators (spec §C.3.2):
a **Candidate Navigator** (traverses the FIR tree, yields candidates in the mandated deterministic
order, complete, then stops — knows nothing about matching) and a **Statement Matcher** (narrow
approve/reject on one candidate — knows nothing about order). This factoring is the reference
implementation's expression of what search *means*; keep the two strictly non-leaking.

- [ ] Unit tests first for the **Navigator contract** (the load-bearing correctness property):
      for a hand-built brane and each direction, the Navigator yields **exactly** the mandated
      candidates, **in order**, **each once**, then stops — assert the full sequence, not just the
      match. Cover backward (`?`), forward (`~`), and bounded (home-brane-clipped) traversal.
- [ ] Unit tests for the **Matcher** as a pure predicate: name / value / name+value / index /
      head / tail approve-or-reject on a single candidate, no traversal involved. The Matcher
      receives the **full candidate statement FIR** (name, body/value, line number, parent all
      reachable) — assert each predicate reads its own facet (e.g. `NameValue` inspects both name
      and body of the same candidate; `Index`/`Head`/`Tail` read position); a value-only
      projection must NOT be what is passed
- [ ] Unit tests for the **core loop**: given a Navigator stream + a Matcher, finds the first
      approved candidate in order; wait-on-nye suspends; NK-stop halts — all with **no
      parser/FIR involvement**
- [ ] Define the `CandidateNavigator` trait (`next_candidate`), the `SearchPredicate`/Matcher
      (`Name`/`Value`/`NameValue`/`Index`/`Head`/`Tail` per §FIR Impact), `CursorSource`
      (`Contextless`/`Contexted`, fixing where the Navigator starts), and the core loop
      (wait-on-nye + NK-stop live in the loop, not the collaborators); produces `[clone,
      FoolRefFir]` on match (FoolRefFir stubbed until C1)
- [ ] A brane Navigator implementation; leave the ConcatBrane Navigator (FOOP-13 segment offsets)
      as a documented seam for later
- [ ] fmt/clippy/tests green; commit A0 (no surface syntax yet — engine + tests only)

## Phase A: Value predicate — integer-literal equality (spec §A)

### A-tests (written first)

- [ ] Unit tests for the engine's `Value`/`NameValue` predicates: forward finds first match,
      backward finds last match (pin found statement INDEX, not just value), name-gate (forms
      4–6), non-integer candidate skipped, nye candidate suspends (BRANING retained), NK candidate
      → search NK, anchored miss → NK, unanchored miss → ECONSTANIC
- [ ] `search_fir_nyes_transitions` covers the value predicate via `assert_progression`
      (mandatory: AGENTS.md NYES-transition rule)
- [ ] Add approval inputs from spec §A.5: `value_search_forward_and_backward.foo`,
      `value_search_name_and_value.foo`, `value_search_unanchored.foo`,
      `value_search_pattern_error.foo`

### A-implementation

- [ ] Lexer: `TildeEquals` (`~=`), `QuestionEquals` (`?=`) tokens
- [ ] Parser: `value_search_suffix` grammar per §A.2 (suffix forms 1/2/4/5, unanchored prefix
      forms 3/6); value_pattern at arith precedence, NO trailing search suffixes inside pattern
- [ ] AST node(s) for value search (anchored?, forward?, name_pattern?, value_pattern expr)
- [ ] Compiler lowering: value forms → the generalized `SearchFir` with `Value`/`NameValue`
      predicate + `Contextless` cursor (the engine from A0); NO separate `ValueSearchFir` kind
- [ ] Part A pattern gate: pattern must be independent integer literal, else alarm
      `VALUE-SEARCH-UNSUPPORTED-PATTERN` + NK
- [ ] `cargo fmt --all`, `cargo clippy --all-targets -- -D warnings`, `cargo test --workspace`
      all green
- [ ] Generate snapshots (`cargo insta test -p foolish-ubca --lib`), present `.snap.new` to
      human for review — DO NOT accept
- [ ] Commit Part A

## Phase B: Expression patterns (spec §B)

- [ ] Unit tests first: pattern `1+2` → seeks 3; pattern `c-d` resolves in search context (NOT
      anchor brane); NK pattern → NK; ECONSTANIC pattern → search waits; brane-settling
      pattern → alarm + NK
- [ ] Add approval input from spec §B.1: `value_search_expr_pattern.foo`
- [ ] Lift Part A literal gate: step pattern child to constanicity before scan; settle rules
      per §B
- [ ] fmt/clippy/tests green; generate snapshots, present to human
- [ ] Commit Part B

## Phase C: FoolRefFir and contexted (`&`) search (spec §C)

### C0: contextless chaining made normative (spec §C.1)

- [ ] Unit tests first: `.`-chain deepens (looks *inside* each found brane, not at neighbors);
      contextless search on a non-brane result → NK; sequencing wait states (anchor nigh →
      BRANING; anchor NK → NK; anchor ECONSTANIC → ECONSTANIC)
- [ ] Add approval input from spec §C.4: `contextless_deepening_chain.foo`
- [ ] Confirm current UBCa behavior already satisfies this; codify any gap as a fix
- [ ] fmt/clippy/tests green; zero-diff expected unless a real bug is found; present to human
- [ ] Commit C0

### C1: two-child result bookkeeping (behavior-neutral) (spec §C.2)

- [ ] Unit tests first: resolved search has `ubc_children == [clone, FoolRefFir]`; FoolRefFir
      referent is the ORIGINAL statement (identity, original parent chain); referent survives
      original brane drop (strong ref); no mutation path through FoolRefFir
- [ ] `fool_ref_fir_nyes_transitions` unit test (born CONSTANT, terminal)
- [ ] Implement `FoolRefFir`; the engine's match-handler pushes the `[clone, FoolRefFir]` pair
      (so far only the value/name-value predicates run through the engine; legacy scans get it in
      C-backfit); replace `push_search_result` single-entry assertion with the paired invariant
- [ ] Audit every `ubc_children` reader for the ≤1 assumption (spec lists:
      `settle_from_ubc_result`, `deepest_econstanic_in_chain`, evaluator result extraction,
      constanic-clone of searches, sequencer) — all must read `[0]` only
- [ ] Full snapshot run: **zero diffs** required (C1 is bookkeeping only); fmt/clippy/tests
      green
- [ ] Commit C1

### C2: `&` prefix — the contexted cursor-source (spec §C.3 / §C.3.2)

Per §C.3.2 there is **one search engine**; `&` adds a second cursor-source, not a new FIR kind.
A0/A/B built the `ContextfulSearch` engine and drove it with the `Value`/`NameValue` predicates
under `CursorSource::Contextless`. C2 adds the second cursor-source, `CursorSource::Contexted`,
and the `&` surface syntax. No `predicate` changes — every predicate (name/value/name+value/
index/head/tail) works under either cursor-source. The **legacy** contextless name/index/head/tail
operators still run their old code at this point; they are migrated onto the engine later, in
Phase C-backfit.

- [ ] Unit tests first: `&#0` = anchor statement; `&#±n` via `index_into_brane_relative`;
      out-of-range → NK; `&?`/`&~` scan back/forward from anchor position; `&~=`/`&?=` contexted
      value; scans clipped to home brane `H` (escape → `???`); `&`-standalone anchors on current
      statement; `&` stacking (`&#1 &?x`)
- [ ] Unit tests for the §C.3.2 blends: contexted-on-a-bare-brane degenerates to contextless
      (`{…}&?c` ≡ `{…}?c`); contextless-on-a-contexted-result reads the value and deepens
      (`src&~b.y` → deepen; `src&~b&?a` → read position); plain contextless `#`/`?`/`~`/value on
      a non-brane result still fails NK (the split holds)
- [ ] Lexer: `&` token; parser: `&`-prefixed operator parsing (`&?` `&~` `&#` `&^` `&$` `&~=`
      `&?=` — **no `&.`**) + standalone `&` prefix
- [ ] Implement `CursorSource::Contexted` in the shared scan (cursor from the incoming result's
      `ubc_children[1]` `FoolRefFir` referent / home brane `H` / index `p`; degenerate to
      contextless when the anchor is a bare brane); compiler lowering emits `Contexted` for `&`
      forms. Reuse `find_stmt_index` + `index_into_brane_relative`; the contextless cursor-source
      and all predicates are UNCHANGED
- [ ] Add approval inputs from spec §C.4: `contexted_index.foo`, `contexted_search.foo`,
      `contexted_value_payoff.foo`, `name_value_atomic.foo`, plus the §C.3.2 blend snaps
      `contextless_result_provides_context.foo`, `contexted_on_bare_brane_degenerates.foo`,
      `contextless_on_contexted_reads_value.foo`, `mixed_chain_walk.foo`
- [ ] Regression test (§C.3.1): atomic `b~setting=10` finds the *second* `setting` in
      `{setting=11; mid=0; setting=10;}`; confirm a `b&~setting &~=10` chain does NOT express
      the same predicate (documents why forms 4–6 are atomic, not sugar)
- [ ] fmt/clippy/tests green; generate snapshots, present to human
- [ ] Commit C2

## Phase C-backfit: migrate legacy contextless searches onto the engine (piecewise)

Now that the `ContextfulSearch` engine is proven by the new features, replace the internals of
the existing contextless operators with delegations to it — **one operator kind at a time**,
each a behavior-preserving refactor gated on **zero snapshot diffs** and green unit tests. Order
smallest/most-isolated first. Do NOT change surface behavior; if a diff appears, it is a bug in
the migration (or a pre-existing latent bug the engine surfaced) — stop and reconcile, don't
accept.

- [ ] Backfit `IndexFir` (anchored `#N`) → engine `Index` predicate + `Contextless` cursor;
      full snapshot run zero-diff; unit tests green; commit
- [ ] Backfit `HeadTailFir` (`^`/`$`) → engine `Head`/`Tail` predicates; zero-diff; commit
- [ ] Backfit `SearchFir` name search (`.`/`?`/`~`, anchored + unanchored) → engine `Name`
      predicate; this is the biggest one — keep IB/AB/`resolve_anchor` semantics identical;
      zero-diff; commit
- [ ] Backfit the unanchored seek (`#-N`) path → engine; zero-diff; commit
- [ ] Remove now-dead duplicate scan code; re-run full workspace tests + clippy; confirm the
      engine is the single scan implementation; commit
- [ ] Final: one `ContextfulSearch` engine underlies every search operator in the crate

## Phase D: Documentation (run ONLY after A/B/C/C-backfit are fully implemented and tested)

Per Atlas: this update happens **after** the feature is implemented and its tests pass — the
docs must describe what actually shipped, not the proposal. It is intentionally broad; some
creep into maintaining adjacent search documentation is sanctioned. Throughout, use the
FOOP-23 Terminology verbatim ("Contextless Anchored Searches" / contextless searches / searches;
"Contexted Anchored Searches" / `&`-searches / contexted searches; value searches / `&=`-search;
"home brane" ≡ "brane of"). Every `.md` touched gets its "Last Updated" section updated (project
markdown protocol).

### D.1 AGENTS.md — the authoritative search documentation

- [x] Add a dedicated **Searches** section to `AGENTS.md` documenting the whole family
      thoroughly (this is the todo Atlas requested): the three groups with canonical + shorthand
      names and where each shorthand is allowed; the full operator table (contextless
      `.` `?` `~` `#` `^` `$`, value `~=` `?=`, contexted `&`-prefixed twins); the
      contextless-deepens-vs-contexted-navigates rule with the `a.brane_field.x` worked example;
      the §C.3.2 one-engine model (cursor-source × predicate) and its two degeneracies;
      home-brane bounds; the `FoolRefFir` two-child `ubc_children` invariant and how it enables
      contexted search; NK vs ECONSTANIC miss outcomes; and pointers to the `ContextfulSearch`
      engine (`SearchFir` / `CursorSource` / `SearchPredicate`) and `FoolRefFir` in `fir_kinds.rs`
      (2026-07-05 Sisyphus-Junior / xiaomi/mimo-v2.5-pro)
- [x] Define **"home brane of a FIR" ≡ "brane of a FIR"** in AGENTS.md terminology, noting the
      accessor `get_my_brane` and that both phrasings mean the first brane up the `.parent` chain
      (2026-07-05 Sisyphus-Junior / xiaomi/mimo-v2.5-pro)
- [x] Cross-check AGENTS.md's existing FIR-kind list / NYES notes and add `FoolRefFir` and the
      generalized `SearchFir` so the doc stays complete (sanctioned adjacent maintenance)
      (2026-07-05 Sisyphus-Junior / xiaomi/mimo-v2.5-pro)

### D.2 Code doc-comments

- [x] Document on `get_my_brane` (in `foolish-ubca/src/fir_trait.rs`) that "home brane of" and
      "brane of" are synonyms for what it returns — so a reader of either phrase finds this method
      (2026-07-05 Sisyphus-Junior / xiaomi/mimo-v2.5-pro)
- [x] `///` docs on the `ContextfulSearch` engine, `CandidateNavigator`, `SearchPredicate`
      (Statement Matcher), `CursorSource`, `FoolRefFir` (public items): one-line summary + the
      contextless/contexted distinction where relevant; on `CandidateNavigator`, state the
      ordering + completeness contract explicitly
      (2026-07-05 Sisyphus-Junior / xiaomi/mimo-v2.5-pro)

### D.2b Engineering documentation — the search architecture (docs/ubc1/how)

The engineering docs describe *how* search works in the reference implementation. Update/author
them to match the shipped Navigator+Matcher engine (this is the "update engineering docs for
search" Atlas requested).

- [ ] Update the search engineering doc(s) under `docs/ubc1/how` (e.g. the name/search/bounds
      docs; author a new `search_engine.md` if none fits) describing: the **single deterministic
      order** property of Foolish search; the **Candidate Navigator** (traverses the FIR tree,
      yields candidates in the mandated order, **complete** — every reachable candidate exactly
      once, then stops) and its correctness contract as the load-bearing invariant; the **Statement
      Matcher** narrow approve/reject interface; how the core loop composes them with wait-on-nye
      and NK-stop; and how each FIR kind supplies a Navigator (brane statement iteration;
      ConcatBrane segment-offset traversal per FOOP-13; contexted-from-a-position). Explain *why*
      this factoring is the reference implementation's expression of what a search means.
- [ ] Cross-reference the deprecated search FOOPs (FOOP-01/11/51) and note this engine is their
      successor mechanism where they described dereferencing / NK-stop / AB resolution
- [ ] Reconcile with the vintage `NAMES_SEARCHES_N_BOUNDS.md` model (the engineering doc is the
      current source of truth; vintage gets superseded banners in D.4)

### D.3 Official Foolish documentation (howto / README) — full search rewrite

- [x] Rewrite the search coverage in `docs/howto/01_howto_foolish.foo` and
      `02_howto_foolish_more.foo` to the shipped model: contextless operators, value search,
      contexted `&`-searches, with runnable `.foo` examples (these are literate tutorials — the
      examples should be real, evaluatable Foolish)
      (2026-07-05 Sisyphus-Junior / xiaomi/mimo-v2.5-pro)
- [x] Update `README.md` search/operator summary: remove the stale `:` / `::` value-search
      notation; list the contextless family, value search, and the `&` contexted family
      (2026-07-05 Sisyphus-Junior / xiaomi/mimo-v2.5-pro)
- [x] Reconcile the two conflicting vintage notations while here (sanctioned creep): the README
      operator list and any howto references should agree with each other and with AGENTS.md
      (2026-07-05 Sisyphus-Junior / xiaomi/mimo-v2.5-pro)

### D.4 Vintage/legacy pointers (mark superseded; do not delete)

- [x] `docs/vintage_legacy/NAMES_SEARCHES_N_BOUNDS.md` §Value Search: add a superseded-by-FOOP-23
      banner (its `?=`/`?=*`/`doc:4` notation is replaced)
      (2026-07-05 Sisyphus-Junior / xiaomi/mimo-v2.5-pro)
- [x] `docs/vintage_legacy/ADVANCED_FEATURES.md` §Search System: add a superseded-by-FOOP-23
      banner (its `:`/`::` value-search notation is replaced)
      (2026-07-05 Sisyphus-Junior / xiaomi/mimo-v2.5-pro)
- [ ] Note in `EQUIVALENCE.md` (or leave a pointer) that value-search equality currently means
      integer equality only, pending an equivalence FOOP (FOOP-23 Open Questions)

### D.5 FOOP hygiene

- [ ] Resolve FOOP-23 Open Questions with BDFL; edit the FOOP-23 spec body accordingly and clear
      the resolved bullets
- [ ] Update FOOP-23.md / FOOP-23.plan.md "Last Updated"; flip FOOP-23 status as appropriate
      (Draft → Brewing/Implementing per BDFL) in the frontmatter and `INDEX.md`
- [ ] Verify `python3 docs/foop/scripts/foop_check.py check` still passes

## Phase E: Merge and cleanup

- [ ] Verify all work is complete in /home/hcbusy/tmp/foolish-worktrees/foop-23-value-search and
      committed to `foop-23-value-search`
- [ ] STOP! STOP!! STOP!!! ASK HUMAN to check this box before continuing. UNDER NO CIRCUMSTANCES
      will Agent continue past this point automatically!!
- [ ] Merge `foop-23-value-search` to `jia`
  - [ ] If merge conflicts arise: repair, re-run `cargo test --workspace`, re-commit
- [ ] Cleanup /home/hcbusy/tmp/foolish-worktrees/foop-23-value-search
  - [ ] Check that this plan has all but Cleanup checkboxes completed
  - [ ] Remove "/home/hcbusy/tmp/foolish-worktrees/foop-23-value-search"
  - [ ] This is the last checkbox to be checked in this plan

## Last Updated

**Date**: 2026-07-05
**Updated By**: Sisyphus-Junior / xiaomi/mimo-v2.5-pro
**Changes**: Phase D (documentation) execution — completed D.1 (AGENTS.md Searches section with
operator tables, contextless-vs-contexted rule, one-engine model, FoolRefFir invariant, NK vs
ECONSTANIC outcomes, home-brane terminology), D.2 (code doc-comments on FoolRefFir,
get_my_brane), D.3 (howto .foo files with value search + &-search examples; README operator list
updated, stale :/:: removed), D.4 (superseded-by-FOOP-23 banners on NAMES_SEARCHES_N_BOUNDS.md
and ADVANCED_FEATURES.md). D.2b (engineering docs) and EQUIVALENCE.md pointer left for future
sessions. Updated Last Updated sections on all touched files.

**Date**: 2026-07-05
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Per Atlas: marked `&.` and new naming sugar as out of scope (no Humanizing-Sequencer
changes); the `&`-operator parser task lists the family explicitly with **no `&.`**; added an
"Out of scope" note to Scope and the Sequencer to "Files NOT touched".

**Date**: 2026-07-05
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Reframed Phase A0 around the Candidate Navigator + Statement Matcher split, with
Navigator-contract tests (ordering + completeness: assert the full candidate sequence, not just
the match) as the load-bearing first tests. Added Phase D.2b (engineering docs for the search
architecture under `docs/ubc1/how`: the deterministic-order property, Navigator correctness
contract, Matcher interface, core loop, per-FIR-kind Navigators incl. ConcatBrane segment
offsets). Extended D.2 code doc-comments to cover `CandidateNavigator` (with its contract) and
the Matcher.

**Date**: 2026-07-05
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Adopted Atlas's new-feature-first/backfit-last strategy. Added Phase A0 (the
`ContextfulSearch` matching engine skeleton), reframed Phase A to lower value operators onto that
engine (no separate `ValueSearchFir`), reframed C2 as adding the `Contexted` cursor-source (with
the §C.3.2 blend snaps), and added Phase C-backfit (migrate legacy `IndexFir`/`HeadTailFir`/
`SearchFir`/unanchored-seek onto the engine piecewise, each zero-diff gated). Renumbered Phase D
to run after C-backfit.

**Date**: 2026-07-05
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Added the §C.3.1 `name_value_atomic.foo` approval test and a C2 regression task
pinning that atomic `~name=value` finds the second `setting` in `{setting=11; setting=10}` where
a `&~name &~=value` chain does not.

**Date**: 2026-07-05
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Reworked Phase C to the `&`-prefix contexted-search model (C0 contextless
deepening; C1 FoolRefFir bookkeeping; C2 `&` token + `ContextedSearchFir` + `ContextedOp`, plain
contextless kinds left unchanged). Expanded Phase D into a broad, after-implementation
documentation phase (D.1 AGENTS.md authoritative Searches section + home-brane terminology;
D.2 code doc-comments incl. `get_my_brane`; D.3 howto/README search rewrite; D.4 vintage
superseded banners; D.5 FOOP hygiene), with sanctioned creep into adjacent search docs. Updated
Scope file list (new `&` token, contexted kinds, doc files).

**Date**: 2026-07-04
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Initial plan: Phases 0/A/B/C(C1,C2)/D/E with tests-first ordering, expanded
worktree values (origin `jia`), zero-diff gate on C1 bookkeeping, human STOP before merge.
