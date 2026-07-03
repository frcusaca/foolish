# FOOP-13 Plan — MAX_BRANE_SIZE auto-sizing

This plan executes [FOOP-13](FOOP-13.md). **Read the specification first** — the plan assumes its
context. Scope is small: one config struct, one AST→AST rewrite in `foolish-ubca/src/compiler.rs`,
one field on `UbcaEvaluator`, and unit tests. No new FIR kind, no NYES change (so no
`*_nyes_transitions` additions), no `.foo` approval tests, no `.snap` churn — the default
configuration is byte-identical to current behavior.

Tests are written FIRST (project rule), asserting the specification, then the implementation makes
them pass.

Branch and worktree lifecycle (per `foop.md`):

```
WORKTREE_ORIGIN_BRANCH=jia
WORKTREE_ORIGIN_PATH=/home/hcbusy/foolish-rust
WORKTREE_BRANCH_NAME=foop-13-max-brane-size
WORKTREE_FULL_FS_PATH=/home/hcbusy/tmp/foolish-worktrees/foop-13-max-brane-size
```

Once work begins, ALL updates — including to this plan and the FOOP spec — happen ONLY in the
worktree, until merge time.

## Phase 0 — Preconditions and worktree

- [ ] Verify all tests pass on `jia` in /home/hcbusy/foolish-rust (`cargo test --workspace`).
      Do not begin while any test is broken (Development Rules).
- [ ] Check the `begun: [ ]` box in FOOP-13.md frontmatter in /home/hcbusy/foolish-rust and commit
      FOOP-13.md + FOOP-13.plan.md + INDEX.md row on `jia`, stating work has commenced.
- [ ] Create worktree at /home/hcbusy/tmp/foolish-worktrees/foop-13-max-brane-size with branch
      `foop-13-max-brane-size` off `jia`:

      ```bash
      cd /home/hcbusy/foolish-rust
      git worktree add -b foop-13-max-brane-size /home/hcbusy/tmp/foolish-worktrees/foop-13-max-brane-size
      cd /home/hcbusy/tmp/foolish-worktrees/foop-13-max-brane-size
      ```

## Phase 1 — Tests first (they will not compile / will fail until Phase 2)

- [ ] Add `UbcaConfig` unit tests + auto-sizing compiler tests in the tests module of
      `foolish-ubca/src/compiler.rs`, per the spec's Test Plan:
  - [ ] `unlimited_config_is_identity` — `compile_with(src, &UbcaConfig::default())` yields the
        same FIR kinds/shape as `compile(src)`; no `FirKind::Concatenation` appears for a source
        with none.
  - [ ] `brane_at_or_under_max_is_not_split` — nested brane with exactly `k` statements stays a
        single `FirKind::Brane`.
  - [ ] `oversized_brane_splits_into_chunked_concatenation` — nested brane with 5 statements,
        `k = 2` → `FirKind::Concatenation` with 3 `FirKind::Brane` children of sizes 2, 2, 1;
        statement names in original order across chunks.
  - [ ] `root_brane_is_never_split` — root brane with `n > k` compiles to a root
        `FirKind::Brane`.
  - [ ] `characterized_brane_is_never_split` — characterized oversized nested brane stays whole.
  - [ ] `oversized_brane_inside_explicit_concatenation` — `a {big} b` with oversized `{big}`
        yields a nested `FirKind::Concatenation` element.
- [ ] Add settle-and-compare test `split_brane_settles_to_same_result_as_unsplit` (where the
      step-to-settled test helpers live, e.g. the tests module of `foolish-ubca/src/fir_kinds.rs`
      or `evaluator.rs`): compile one program with unlimited vs `k = 2` config, step both roots to
      settled, assert identical sequenced output. Program MUST include a statement referencing a
      name defined in an earlier chunk (cross-chunk resolution pin).
- [ ] Commit the failing tests: "FOOP-13: tests first for MAX_BRANE_SIZE auto-sizing".

## Phase 2 — Implementation

- [ ] Add `UbcaConfig { max_brane_size: Option<NonZeroUsize> }` (`Debug, Clone, Default`) in
      `foolish-ubca` (new small module or `compiler.rs`), exported from `lib.rs`.
- [ ] Add the recursive AST→AST rewrite `auto_size_astn(ast: Astn, k: NonZeroUsize) -> Astn` in
      `foolish-ubca/src/compiler.rs`: recurse structurally; on `Astn::Brane`, after recursing into
      statements, if `statements.len() > k` and `characterizations.is_empty()`, chunk into
      consecutive `Brane`s of ≤ `k` statements wrapped in `Astn::Concatenation`. Root exemption:
      the top-level call rewrites the root's statements but never the root brane itself.
- [ ] Add `Compiler::compile_with(source: &str, config: &UbcaConfig)`; apply the rewrite between
      `validate_astn` and `build_fir` in `compile_standalone` (thread the config through, root
      exempt). `Compiler::compile` delegates with `UbcaConfig::default()`.
- [ ] Change `UbcaEvaluator` to `pub struct UbcaEvaluator { pub config: UbcaConfig }` with
      `Default`; `evaluate` calls `Compiler::compile_with(source, &self.config)`. Fix any
      construction sites (`UbcaEvaluator` → `UbcaEvaluator::default()`).
- [ ] All Phase 1 tests pass: `cargo test -p foolish-ubca`.
- [ ] Commit: "FOOP-13: implement MAX_BRANE_SIZE auto-sizing rewrite".

## Phase 3 — Gates and no-regression proof

- [ ] `cargo fmt --all` — clean.
- [ ] `cargo clippy --workspace -- -D warnings` — clean.
- [ ] `cargo test --workspace` — all green.
- [ ] Run the UBCa snapshot suite (`cargo insta test -p foolish-ubca --lib`) and confirm ZERO
      `.snap.new` files are produced (default config is byte-identical; NEVER accept snapshots —
      if any `.snap.new` appears, that is a bug in the rewrite plumbing: stop and fix).
- [ ] Update FOOP-13.md status `Draft` → `Implementing`→ (when merged) leave for human to set
      `Final`; refresh both files' Last Updated sections (in the WORKTREE).
- [ ] Commit: "FOOP-13: gates green, snapshots untouched".

## Phase 4 — Merge and cleanup

- [ ] Verify all work is complete in /home/hcbusy/tmp/foolish-worktrees/foop-13-max-brane-size and
      committed to `foop-13-max-brane-size`.
- [ ] Merge `foop-13-max-brane-size` to `jia` in /home/hcbusy/foolish-rust (git merge, not
      rebase); repair any conflicts and re-run the Phase 3 gates on `jia`.
- [ ] Update `docs/foop/INDEX.md` row for FOOP-13 status on `jia`.
- [ ] STOP! STOP!! STOP!!! ASK HUMAN to check this box before continuing. UNDER NO CIRCUMSTANCES
      will Agent continue past this point automatically!!
- [ ] Cleanup /home/hcbusy/tmp/foolish-worktrees/foop-13-max-brane-size.
  - [ ] Check that FOOP-13.plan.md has all but Cleanup checkboxes completed.
  - [ ] Remove /home/hcbusy/tmp/foolish-worktrees/foop-13-max-brane-size
        (`git worktree remove ...` from /home/hcbusy/foolish-rust).
  - [ ] This is the last sub-task checkbox to be checked in this block of subtasks.

## Last Updated

**Date**: 2026-07-03
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Initial plan — tests-first phases for the MAX_BRANE_SIZE auto-sizing rewrite,
worktree lifecycle off `jia`, gates including a zero-`.snap.new` no-regression check.
