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
- `foolish-parser/src/token.rs`, `lexer.rs`, `parser.rs`, `ast.rs` — `~=`/`?=` tokens, value
  search grammar (A.2), AST node(s)
- `foolish-ubca/src/fir_kinds.rs` — `ValueSearchFir`, `FoolRefFir`, `IndexFir`/`SearchFir`
  search-anchored dispatch
- `foolish-ubca/src/proto_brane.rs` — `push_search_result` two-entry invariant
- `foolish-ubca/src/compiler.rs` — AST → FIR for value search
- `foolish-ubca/snapshot_tests/input/` — new approval inputs (A.5, B.1, C.4)
- alarm plumbing for `VALUE-SEARCH-UNSUPPORTED-PATTERN`

**Files NOT touched:** approved `.snap` files (human workflow only), foolish-core evaluation
semantics, UBC (retired).

---

## Phase 0: Setup

- [ ] Verify all workspace tests pass on `jia` before starting (hard project rule)
- [ ] Create worktree at /home/hcbusy/tmp/foolish-worktrees/foop-23-value-search with branch
      `foop-23-value-search` from `jia`:
      `git worktree add -b foop-23-value-search /home/hcbusy/tmp/foolish-worktrees/foop-23-value-search`
- [ ] Check the `begun` box in FOOP-23.md frontmatter and commit in the origin directory stating
      work has commenced; from this point ALL work (including FOOP/plan edits) happens ONLY in
      the worktree

## Phase A: Operator family, integer-literal equality (spec §A)

### A-tests (written first)

- [ ] Unit tests for `ValueSearchFir` scanning: forward finds first match, backward finds last
      match (pin found statement INDEX, not just value), name-gate (forms 4–6), non-integer
      candidate skipped, nye candidate suspends (BRANING retained), NK candidate → search NK,
      anchored miss → NK, unanchored miss → ECONSTANIC
- [ ] `value_search_fir_nyes_transitions` unit test via `assert_progression` (mandatory:
      AGENTS.md NYES-transition rule)
- [ ] Add approval inputs from spec §A.5: `value_search_forward_and_backward.foo`,
      `value_search_name_and_value.foo`, `value_search_unanchored.foo`,
      `value_search_pattern_error.foo`

### A-implementation

- [ ] Lexer: `TildeEquals` (`~=`), `QuestionEquals` (`?=`) tokens
- [ ] Parser: `value_search_suffix` grammar per §A.2 (suffix forms 1/2/4/5, unanchored prefix
      forms 3/6); value_pattern at arith precedence, NO trailing search suffixes inside pattern
- [ ] AST node(s) for value search (anchored?, forward?, name_pattern?, value_pattern expr)
- [ ] `ValueSearchFir` in `fir_kinds.rs` per spec §FIR Impact; compiler lowering
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

## Phase C: FoolRefFir and search-anchored anchoring (spec §C)

### C1: two-child result bookkeeping (behavior-neutral)

- [ ] Unit tests first: resolved search has `ubc_children == [clone, FoolRefFir]`; FoolRefFir
      referent is the ORIGINAL statement (identity, original parent chain); referent survives
      original brane drop (strong ref); no mutation path through FoolRefFir
- [ ] `fool_ref_fir_nyes_transitions` unit test (born CONSTANT, terminal)
- [ ] Implement `FoolRefFir`; extend `handle_found` (SearchFir + ValueSearchFir) to push the
      pair; replace `push_search_result` single-entry assertion with paired invariant
- [ ] Audit every `ubc_children` reader for the ≤1 assumption (spec lists:
      `settle_from_ubc_result`, `deepest_econstanic_in_chain`, evaluator result extraction,
      constanic-clone of searches, sequencer) — all must read `[0]` only
- [ ] Full snapshot run: **zero diffs** required (C1 is bookkeeping only); fmt/clippy/tests
      green
- [ ] Commit C1

### C2: chained sequencing + positional anchoring

- [ ] Unit tests first: chain waits per §C.1 (anchor nigh → BRANING; anchor NK → NK; anchor
      ECONSTANIC → ECONSTANIC); `#0` = found statement; `#±n` addressing via
      `index_into_brane_relative`; out-of-range → NK; `?`/`~` anchored on search scan
      backward/forward from original position; scans clipped to home brane (escape → NK)
- [ ] Add approval inputs from spec §C.4: `chained_search_sequencing.foo`,
      `search_anchored_index.foo`, `search_anchored_search.foo`,
      `value_search_positional_payoff.foo`
- [ ] `IndexFir`: Search-kind anchor dispatch (via `ubc_children[1]` referent +
      `find_stmt_index` + `index_into_brane_relative`)
- [ ] `SearchFir`/`ValueSearchFir`: Search-kind anchor dispatch, positional scan with
      home-brane bounds
- [ ] fmt/clippy/tests green; generate snapshots, present to human
- [ ] Commit C2

## Phase D: Documentation

- [ ] Update `docs/vintage_legacy/NAMES_SEARCHES_N_BOUNDS.md` §Value Search and
      `ADVANCED_FEATURES.md` with superseded-by-FOOP-23 pointers (do not delete vintage text)
- [ ] Update README operator list if it still shows `:` value search
- [ ] Resolve FOOP-23 Open Questions with BDFL; edit spec body accordingly
- [ ] Update Last Updated sections of every touched .md

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

**Date**: 2026-07-04
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Initial plan: Phases 0/A/B/C(C1,C2)/D/E with tests-first ordering, expanded
worktree values (origin `jia`), zero-diff gate on C1 bookkeeping, human STOP before merge.
