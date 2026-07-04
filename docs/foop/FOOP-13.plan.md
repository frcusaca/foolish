# FOOP-13 Plan — ConcatBrane upgrade, then MAX_BRANE_SIZE auto-sizing

This plan executes [FOOP-13](FOOP-13.md). **Read the specification first** — the plan assumes
its context, above all the Equivalence Law (a settled ConcatBrane is observationally identical
to the one big brane holding every statement of its elements, in order, never materialized) and
the two-phase structure:

- **PHASE A — ConcatBrane upgrade**: `ConcatenationFir` stops merging; hidden k-ary storage
  tree of bags; capability dispatch; global line numbers; true constanic cloning with
  recoordination. Semantic repair of source-level concatenation; NO configuration involved.
- **PHASE B — MAX_BRANE_SIZE**: `UbcaConfig` + the iterative AST rewrite (chunk statements,
  then group element arrays > k recursively until every node fits).

Phase A produces expected `.snap.new` churn (step counts change for all concatenation programs;
cross-element references may newly resolve). That churn is **reviewed by the human between the
phases** — NEVER auto-accepted, never `cargo insta accept`, never `INSTA_UPDATE=always`. Phase B
must produce ZERO further churn under the default (unlimited) configuration.

Tests are written FIRST in each phase (project rule), asserting the specification, then the
implementation makes them pass. No new FIR kind and no new NYES state; only
`concatenation_nyes_transitions` is extended.

Branch and worktree lifecycle (per `foop.md`):

```
WORKTREE_ORIGIN_BRANCH=jia
WORKTREE_ORIGIN_PATH=/home/hcbusy/foolish-rust
WORKTREE_BRANCH_NAME=foop-13-concat-brane-max-size
WORKTREE_FULL_FS_PATH=/home/hcbusy/tmp/foolish-worktrees/foop-13-concat-brane-max-size
```

Once work begins, ALL updates — including to this plan and the FOOP spec — happen ONLY in the
worktree, until merge time.

## Phase 0 — Preconditions and worktree

- [ ] Verify all tests pass on `jia` in /home/hcbusy/foolish-rust (`cargo test --workspace`).
      Do not begin while any test is broken (Development Rules).
- [ ] Check the `begun: [ ]` box in FOOP-13.md frontmatter in /home/hcbusy/foolish-rust and
      commit FOOP-13.md + FOOP-13.plan.md + INDEX.md on `jia`, stating work has commenced.
- [ ] Create worktree at /home/hcbusy/tmp/foolish-worktrees/foop-13-concat-brane-max-size with
      branch `foop-13-concat-brane-max-size` off `jia`:

      ```bash
      cd /home/hcbusy/foolish-rust
      git worktree add -b foop-13-concat-brane-max-size /home/hcbusy/tmp/foolish-worktrees/foop-13-concat-brane-max-size
      cd /home/hcbusy/tmp/foolish-worktrees/foop-13-concat-brane-max-size
      ```

## PHASE A — ConcatBrane upgrade

### A1 — Tests first: SNAPSHOTS before unit tests (will fail / not settle until A3)

- [ ] Create the `concat_brane_*` snapshot inputs FIRST by copying the fenced `foolish` blocks
      from spec §Test Plan VERBATIM (byte-for-byte — the spec is the source of truth) into
      `foolish-ubca/snapshot_tests/input/`. These replace the existing `concatenation_*`
      family (retirement of the old approved `.snap` files is HUMAN-gated in A4):
  - [ ] `concat_brane_test_basic.foo` — small literal/named concatenations; output shows the
        cross-element repair (`c=3`).
  - [ ] `concat_brane_foolish_concatenations.foo` — deeply nested written concatenations
        (`{ { { {..}{...}{..} }{..}{ {..}{ {..}{..} } } } ...}`); output is one flat brane in
        source order.
  - [ ] `concat_brane_split_long_brane.foo` — `{a1=1, …, a200=200}`. Settles UNSPLIT in
        Phase A; by Sequencing k-invariance the SAME approved snap must survive Phase B's
        k=13 splitting unchanged (the Phase B sentinel).
  - [ ] `concat_brane_nested_shadowed_resolution.foo` — the exhaustive shadowed-resolution
        matrix: prefix-constituent vs parent-context vs original-context (copied-in constants)
        vs `<{…}>` (resolves from the concat's own statement position).
- [ ] Equivalence Law and search tests (tests module of `foolish-ubca/src/fir_kinds.rs`):
  - [ ] `concat_equals_big_brane` — same statements as `{s₁…sₙ}` vs `{s₁…s₅}{s₆…sₙ}` settle to
        identical sequenced output.
  - [ ] `concat_search_brane_translates_global_indices` — forward and reverse `_search_brane`
        hits in first, middle, and last bag return correct global indices.
  - [ ] `concat_ib_search_crosses_segments` — `{a=10}{b=a}` resolves `b` to `10`.
  - [ ] `concat_ab_search_reaches_outward` — a statement inside a ConcatBrane resolves a name
        defined in the enclosing brane.
- [ ] Indexing tests:
  - [ ] `concat_index_spans_segments` — `#9` into 5+5 finds the last statement; `#-1` the same;
        head/tail across a bag boundary; out-of-range → NK.
  - [ ] `concat_find_stmt_index_is_global`.
- [ ] Structure, value, and clone tests:
  - [ ] `concat_statement_parents_point_at_top_concat` — parents bypass the whole storage tree
        at every depth; bags never surface via `get_my_brane`.
  - [ ] `concat_value_is_itself` — `settled_result()` is None so `value()` of a settled
        ConcatBrane is itself; `as_i64` is None via the unmodified default (no override).
  - [ ] `concat_constanic_clone_rewires_and_recoordinates` — clone-of-concat as a search result:
        storage tree deep-cloned via `skip_foolish_children = true`, parents rewired to the
        clone, numbering and arrangement preserved; NO element FIRs cloned, element searches
        never re-run (value semantics).
  - [ ] `settled_search_clone_skips_foolish_children` — settled SearchFir clone with the option
        drops the anchor subtree; behavior otherwise identical.
  - [ ] `concat_arrangement_is_function_of_n_and_k` — nested ConcatBrane element contributes
        its lines like any brane (no inherited shape); unlimited k → single SubBrane; n=k² and
        n=k²+1 boundaries.
  - [ ] Empty-brane elements contribute zero lines
        (`concatenation_of_empty_branes` semantics preserved).
- [ ] Protocol tests (element typing, auto-wrapping, copy-and-coordinate):
  - [ ] `concat_element_typing_rejects_non_brane` — non-brane/non-search direct element →
        alarm + NK at construction; element resolving to a non-brane at settle time →
        alarm + NK.
  - [ ] `concat_construction_auto_wraps` — literal elements SFF-wrapped (searches BORN
        ECONSTANIC), search elements SF-wrapped.
  - [ ] `concat_cross_element_reference_resolves` — `{cb = {a=1, b=2} {c = a + b};}` → `c = 3`
        (pins the semantic repair; current evaluator leaves `c` unresolved, verified
        2026-07-04).
  - [ ] `concat_sff_born_searches_revive_embryonic` — copy transforms ECONSTANIC → EMBRYONIC in
        position with correct parents.
  - [ ] `concat_sf_on_search_is_noop` — explicit `<search>` element ≡ bare search element.
  - [ ] `concat_sf_marked_literal_prepares_locally` — explicit SF on a literal OVERRIDES the
        auto-SFF: innards resolve BEFORE copy, from the concat's own statement context; settled
        lines copy with standard recoordination.
  - [ ] `concat_explicit_sff_element_is_error` — `<<{…}>>` element → alarm + NK, never steps.
- [ ] Extend `concatenation_nyes_transitions` in `foolish-ubca/src/fir_kinds.rs` for the
      populate-then-drain progression (assert_progression: PREMBRIONIC start, monotone,
      constanic terminal).
- [ ] Commit: "FOOP-13 A1: tests first for the non-merging ConcatBrane".

### A2 — Capability plumbing (behavior-neutral refactor; gates stay green)

- [ ] Add `Fir` trait methods with behavior-preserving defaults: `stmt_count()` (Brane:
      `foolish_children.len()`; default None), `stmt_at(idx)`, and `settled_result()` (default:
      first ubc child — today's behavior for the result-style kinds).
- [ ] Convert kind-match sites from `FirKind::Brane` to the capability
      (`stmt_count().is_some()` / `is_brane_like()`): `get_my_brane` and `step_inner`
      `current_brane` in `foolish-ubca/src/fir_trait.rs`; `find_parent_brane` and the SearchFir
      anchored arm in `foolish-ubca/src/fir_kinds.rs`; the `proto_to_core_fir` bridge sites in
      `foolish-ubca/src/evaluator.rs`. Leave construction-site matches alone.
- [ ] Re-express `FirRefNavExt::index_into`, `find_stmt_index`, and
      `index_into_brane_relative` over `stmt_count`/`stmt_at`.
- [ ] Re-express `FirRefExt::value` over `settled_result()` (Some → recurse into the result;
      None → the FIR is its value; the constanic gate lives INSIDE `settled_result()`, which
      never returns self — the caller holding the handle substitutes self).
- [ ] Doctrine correction (spec §"Doctrine correction"): rewrite the `FirRefExt::value` and
      `Fir::as_i64` doc comments in `foolish-ubca/src/fir_trait.rs` and scope the `(result=)`
      aside on `ProtoBrane::all_children` in `foolish-ubca/src/proto_brane.rs`; sweep
      `foolish-ubca` code comments and `docs/` for any other "ubc_children = result" phrasing
      and correct it (leave `push_search_result`'s search-scoped invariant and FOOP-62 §8
      untouched — they are already correctly scoped).
- [ ] Gates: `cargo fmt --all`; `cargo clippy --workspace -- -D warnings`;
      `cargo test --workspace`; `cargo insta test -p foolish-ubca --lib` produces ZERO
      `.snap.new` (this step must be observably inert).
- [ ] Commit: "FOOP-13 A2: capability dispatch plumbing (behavior-neutral)".

### A3 — The non-merging ConcatBrane

- [ ] Implement the ConcatBrane protocol (spec §"The ConcatBrane protocol") in
      `ConcatenationFir` construction + `fir_op_step`:
  - [ ] Construction: element typing check (literal brane | search | SF-marked form of either;
        explicit SFF element or any other kind → alarm + NK); auto-wrap SFF around bare literal
        branes (reuse the `under_sff` build rule) and SF around bare searches; explicit SF on a
        literal overrides the auto-SFF (local preparation — element built WITHOUT `under_sff`,
        steps in the concat's own context before copy).
  - [ ] Step: elements to constanic; settle-time typing check (every value a brane, else
        alarm + NK).
  - [ ] Count lines; compute the arrangement as the pure function of (n, k): SubBranes ≤ k
        under stacked ConcatBranes of fan-out ≤ k (unlimited k → single SubBrane). Any
        brane-like value contributes lines through the trait surface (seam stays open for
        future brane kinds).
  - [ ] Constanic-copy each line to its SubBrane position and coordinate there: parent bypass
        to the top ConcatBrane, global line numbers, SFF-born ECONSTANIC → EMBRYONIC revival;
        push non-constanic copies as tasks; settle by the existing terminal rule.
- [ ] Add the general `skip_foolish_children` option to the constanic-clone path
      (`foolish-ubca/src/proto_brane.rs`): clone `ubc_children` only. Use it for the settled
      ConcatBrane clone (value semantics — element FIRs not cloned, element searches never
      re-run), then adopt it in the settled-SearchFir/IndexFir clone path (result lives in
      ubc_children; the anchor subtree is dead weight).
- [ ] Implement `ConcatenationFir` overrides: `stmt_count`, `stmt_at` (tree descent via prefix
      sums), `_search_brane` (direction-aware global range over the tree), `_ab_search` (shared
      logic with `BraneFir`, not duplicated), `is_own_value` → true, `as_i64` → None.
- [ ] Teach the constanic-clone path (`foolish-ubca/src/proto_brane.rs`) the ConcatBrane arm:
      deep-clone storage tree, rewire, preserve numbering/shape, standard NYES transform.
- [ ] Update `proto_to_core_fir` to render a settled ConcatBrane as ONE flat brane in global
      order (Equivalence Law) — byte-identical rendering where semantics are unchanged.
- [ ] All A1 tests pass: `cargo test -p foolish-ubca`.
- [ ] Commit: "FOOP-13 A3: non-merging ConcatBrane with hidden storage tree".

### A4 — Phase A gates and HUMAN SNAPSHOT REVIEW

- [ ] `cargo fmt --all`; `cargo clippy --workspace -- -D warnings`; `cargo test --workspace`.
- [ ] `cargo clean -p foolish-ubca && cargo insta test -p foolish-ubca --lib` — collect the
      `.snap.new` files. Expected churn ONLY in concatenation-related snapshots: step-count
      drift, plus cross-element references newly resolving (the `{a=10}{b=a}` class). Any OTHER
      churn is a regression: stop and fix.
- [ ] Write a churn summary (which snaps, step-count-only vs semantic) for the reviewer.
- [ ] Propose retirement of the superseded `concatenation_*` snapshot inputs and approved
      `.snap` files (list them explicitly for the reviewer). AI may remove the superseded
      `.foo` INPUT files once the human agrees; the approved `.snap` files themselves are
      removed by the HUMAN only.
- [ ] STOP! STOP!! STOP!!! ASK HUMAN to review Phase A snapshots with ./foolish_review.sh
      foolish-ubca and ./accept_approved.sh foolish-ubca, and to check this box before Phase B.
      UNDER NO CIRCUMSTANCES will Agent continue past this point automatically!!
- [ ] Address any `.snap.new.check` files flagged with `@agent` comments.
- [ ] Commit: "FOOP-13 A4: Phase A green; snapshots human-reviewed".

## PHASE B — MAX_BRANE_SIZE

### B1 — Tests first (compiler tests module, `foolish-ubca/src/compiler.rs`)

- [ ] `unlimited_config_is_identity` — default config compiles byte-identically to `compile`.
- [ ] `brane_at_or_under_max_is_not_split` — exactly `k` statements stays a single BraneFir.
- [ ] `oversized_brane_splits_into_chunked_concatenation` — 5 statements, k=2 → ConcatenationFir
      of 3 BraneFir chunks sized 2, 2, 1; statement names/order preserved.
- [ ] `concat_brane_split_long_brane_hierarchy` — companion to the Phase A snap: under k=13
      the a1…a200 storage tree is the pure function of (200, 13) — 16 SubBranes of ≤ 13 lines
      under stacked ConcatBranes, every node fan-out ≤ 13, balanced per the arrangement rule,
      global indices 0..199 in order.
- [ ] `iterative_grouping_bounds_every_node` — n=30, k=3 (10 chunks > k → grouping iterates):
      NO node holds more than 3 children (statements or elements); order preserved; settles
      identically to the unlimited compile. Boundary cases n=k² and n=k²+1.
- [ ] `root_brane_is_never_split`; `characterized_brane_is_never_split`.
- [ ] `split_brane_settles_to_same_result_as_unsplit` — unlimited vs k=2, identical sequenced
      output, including a cross-chunk name reference.
- [ ] Commit: "FOOP-13 B1: tests first for MAX_BRANE_SIZE auto-sizing".

### B2 — Configuration surface

- [ ] Add `UbcaConfig { max_brane_size: Option<NonZeroUsize> }` (`Debug, Clone, Default`),
      exported from `foolish-ubca/src/lib.rs`.
- [ ] `Compiler::compile_with(source, &UbcaConfig)`; `Compiler::compile` delegates with default.
- [ ] `UbcaEvaluator` gains `pub config: UbcaConfig` with `Default`; `evaluate` uses
      `compile_with`; fix construction sites.
- [ ] Commit: "FOOP-13 B2: UbcaConfig and compile_with".

### B3 — The iterative auto-sizing rewrite

- [ ] Implement the AST→AST rewrite in `foolish-ubca/src/compiler.rs` between `validate_astn`
      and `build_fir`: recurse into statements; chunk oversized branes into ≤ k-statement chunk
      branes; then WHILE the element array exceeds k, group consecutive runs of ≤ k elements
      into nested `Astn::Concatenation`s (the k-ary tree). Root and characterized branes exempt.
- [ ] All B1 tests pass: `cargo test -p foolish-ubca`.
- [ ] Commit: "FOOP-13 B3: iterative auto-sizing rewrite".

### B4 — Phase B gates

- [ ] `cargo fmt --all`; `cargo clippy --workspace -- -D warnings`; `cargo test --workspace`.
- [ ] Set the UBCa snapshot suite to `max_brane_size = 13` (`ubca_snapshot_tester.rs`
      constructs its `UbcaEvaluator` with `UbcaConfig { max_brane_size: NonZeroUsize::new(13) }`).
      13 forces real splitting in the targeted inputs (200 > 13² = 169 → three-level tree)
      while leaving small-brane snapshots untouched. Verify no approved snapshot contains a
      brane exceeding 13 statements other than the concat_brane family; list any that do for
      the human before proceeding.
- [ ] `cargo insta test -p foolish-ubca --lib` — ZERO `.snap.new` relative to the Phase A
      approved state (default config is unlimited; Phase B must be invisible to snapshots).
- [ ] Update FOOP-13.md status `Draft` → `Implementing` and refresh both files' Last Updated
      sections (in the WORKTREE).
- [ ] Commit: "FOOP-13 B4: gates green; default config snapshot-invisible".

## Phase 5 — Merge and cleanup

- [ ] Verify all work is complete in
      /home/hcbusy/tmp/foolish-worktrees/foop-13-concat-brane-max-size and committed to
      `foop-13-concat-brane-max-size`.
- [ ] Merge `foop-13-concat-brane-max-size` to `jia` in /home/hcbusy/foolish-rust (git merge,
      not rebase); repair any conflicts and re-run all gates on `jia`.
- [ ] Update `docs/foop/INDEX.md` row for FOOP-13 status on `jia`.
- [ ] STOP! STOP!! STOP!!! ASK HUMAN to check this box before continuing. UNDER NO CIRCUMSTANCES
      will Agent continue past this point automatically!!
- [ ] Cleanup /home/hcbusy/tmp/foolish-worktrees/foop-13-concat-brane-max-size.
  - [ ] Check that FOOP-13.plan.md has all but Cleanup checkboxes completed.
  - [ ] Remove /home/hcbusy/tmp/foolish-worktrees/foop-13-concat-brane-max-size
        (`git worktree remove ...` from /home/hcbusy/foolish-rust).
  - [ ] This is the last sub-task checkbox to be checked in this block of subtasks.

## Last Updated

**Date**: 2026-07-04
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Spec seventh revision synced: A1 restructured snapshot-first (author the four
`concat_brane_*` inputs before unit tests; old `concatenation_*` family superseded); A4 gains
the human-gated retirement task; B1 gains `concat_brane_split_long_brane_hierarchy`; B4 gains
the suite max_brane_size=13 task with the pre-flight check for oversized branes in existing
snaps.

**Date**: 2026-07-04
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Spec sixth revision synced: A3 gains the `skip_foolish_children` clone-option task
(ConcatBrane value-semantics clone + settled-search adoption); clone tests updated (no element
FIRs in the clone, element searches never re-run; `settled_search_clone_skips_foolish_children`
added).

**Date**: 2026-07-04
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Spec fifth revision synced: A1 gains the SF-element tests (SF-on-search NOOP,
SF-on-literal override/local preparation, explicit-SFF error); A3 construction sub-task now
covers the three accepted element forms and the SF override built without `under_sff`.

**Date**: 2026-07-04
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: A1/A3 updated for the ConcatBrane protocol (spec fourth revision): protocol test
block added (element typing NK, auto-wrapping, cross-element resolution `c=3`, ECONSTANIC →
EMBRYONIC revival); A3 rewritten as construction typing + auto-SFF/SF + settle + count-and-
arrange (pure function of (n, k)) + copy-and-coordinate sub-tasks.

**Date**: 2026-07-03
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: A2 updated per Atlas's correction: `settled_result()` replaces the `is_own_value()`
hook, `value()` re-expressed over it, and a doctrine-correction task added (fix doc comments
that overstate "ubc_children = result"; no `as_i64` override). A1 test renamed to
`concat_value_is_itself`.

**Date**: 2026-07-03
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Rewritten for the revised specification: two implementation phases (A: non-merging
ConcatBrane with hidden k-ary storage tree, capability dispatch, constanic-clone repair; B:
UbcaConfig + iterative auto-sizing rewrite), with a mandatory human snapshot-review STOP between
the phases and a zero-churn gate on Phase B. Worktree renamed to foop-13-concat-brane-max-size.
