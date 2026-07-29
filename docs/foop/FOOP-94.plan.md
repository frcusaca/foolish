# FOOP-94.plan — brane-nk (brane NK only when all constituents are NK)

Read `docs/foop/FOOP-94.md` before executing. Execute top to bottom. Once work begins, all
updates — including to this plan — happen ONLY in the worktree until merge.

Worktree literals (origin branch `jia`; this clone has no `alpha`):

- Origin branch: `jia`, origin path: `/home/hcbusy/foolish-rust`
- Branch: `foop-94-brane-nk`
- Worktree: `/home/hcbusy/tmp/foolish-worktrees/foop-94-brane-nk`

---

- [ ] Begin work: verify `cargo test --workspace` is green on `jia`; commit FOOP-94.md and
      FOOP-94.plan.md; check `begun: [x]` in FOOP-94.md frontmatter
- [ ] Create worktree at /home/hcbusy/tmp/foolish-worktrees/foop-94-brane-nk with branch
      `foop-94-brane-nk` (`git worktree add -b foop-94-brane-nk
      /home/hcbusy/tmp/foolish-worktrees/foop-94-brane-nk`), from `jia` at
      /home/hcbusy/foolish-rust
- [ ] (read §Investigation questions of FOOP-94.md)
- [ ] Investigation: audit every consumer of a brane's NYES — grep `get_nyes()` call sites and
      classify receivers; confirm no code branches on a *brane* being NK (other than the
      classifier itself and display); record findings in FOOP-94.md §Investigation questions
- [ ] Investigation: confirm all four `_decide_nyes_due_to_children` call sites
      (`fir_kinds.rs:801`, `:2170`, `:2237`, `:2448`) want the new rule, especially the concat
      path; record findings in FOOP-94.md
- [ ] Tests first (failing): rename `brane_with_nk_child_classifies_nk` →
      `brane_with_nk_child_classifies_constant`; add `brane_all_nk_children_classifies_nk` and
      `brane_single_nk_child_classifies_nk`; add concat equivalents; add the four
      preserved-invariant search tests (§Preserved invariants of FOOP-94.md)
- [ ] Flip the rule in `_decide_nyes_due_to_children` (`fir_kinds.rs:11`) per the cascade table
      in FOOP-94.md §Specification; update (or delegate) the test-helper brane in
      `fir_trait.rs` (~`:476`) to match
- [ ] Extend `brane_nyes_transitions` (and concat variants) with mixed-children and all-NK
      progressions per the AGENTS.md NYES-transition-test REQUIREMENT
- [ ] `cargo test -p foolish-ubca` green (unit tests)
- [ ] `cargo insta test -p foolish-ubca --lib` — inventory the `.snap.new` set (~expect the
      mixed-NK subset of the 34 brane-NK snapshots); verify EACH diff is exactly a
      container-state change (members and alarms byte-identical); fix any deviation
- [ ] STOP! Present the `.snap.new` set to the human for review (`./foolish_review.sh
      foolish-ubca`, then `./accept_approved.sh foolish-ubca`). NEVER auto-accept. Do not
      proceed until the human has signed the snapshots.
- [ ] Write and verify `foolish-ubca/snapshot_tests/input/foop_94_comprehensive.foo` (mixed
      NK/value branes: nested, searched — anchored/contexted/value — concatenated, and fed
      through operators); human signs it with the same review flow. (If FOOP-64's einmo suite
      has merged by now, use `foolish-ubca/einmo_suite/input/foop/94/comprehensive.foo` and the
      einmo promote flow instead.)
- [ ] Resolve FOOP-94.md §Open Questions (Independent+NK classification; helper delegation;
      sequencer rendering) — edit the spec body, empty the section
- [ ] Full gates in worktree: `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`,
      `cargo test --workspace`
- [ ] Verify all work is complete in /home/hcbusy/tmp/foolish-worktrees/foop-94-brane-nk and
      committed to `foop-94-brane-nk`
- [ ] Merge `foop-94-brane-nk` to `jia`
  - [ ] Merge breaking changes from `jia` into the worktree first; resolve; re-run full gates
  - [ ] Repair ALL tests on `jia` in /home/hcbusy/foolish-rust after merge
  - [ ] STOP! STOP!! STOP!!! ASK HUMAN to check this box before continuing. UNDER NO
        CIRCUMSTANCES will Agent continue past this point automatically!!
    - [ ] Present human with `cd /home/hcbusy/tmp/foolish-worktrees/foop-94-brane-nk` and ask
          them to review the signed snapshots and the classifier diff BEFORE checking the
          parent checkbox.
- [ ] Cleanup /home/hcbusy/tmp/foolish-worktrees/foop-94-brane-nk
  - [ ] Check that FOOP-94.plan.md has all but Cleanup checkboxes completed
  - [ ] Remove /home/hcbusy/tmp/foolish-worktrees/foop-94-brane-nk
        (`git worktree remove ...`)
  - [ ] This is the last sub-task checkbox to be checked in this block
