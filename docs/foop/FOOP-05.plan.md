# FOOP-05.plan — fir-decomposition (fir_base / fir_search_base / firs/*)

Read `docs/foop/FOOP-05.md` first. Track 1: execute immediately after FOOP-64 merges (green
einmo gate is the byte-identity oracle), before Tracks 2/3 fork into parallel worktrees.

- Origin branch: `jia`, origin path: `/home/hcbusy/foolish-rust`
- Branch: `foop-05-fir-decomposition`
- Worktree: `/home/hcbusy/tmp/foolish-worktrees/foop-05-fir-decomposition`

---

- [ ] Begin work: verify workspace tests green on `jia` (requires FOOP-64 merged); commit
      FOOP-05.md + FOOP-05.plan.md; check `begun: [x]`
- [ ] Create worktree at /home/hcbusy/tmp/foolish-worktrees/foop-05-fir-decomposition with
      branch `foop-05-fir-decomposition` from `jia`
- [ ] Inventory: enumerate `FirKind` variants and map every item in `fir_kinds.rs` and
      `fir_trait.rs` to its target file (spec §Specification layout); record the map in this
      plan as sub-items before moving anything
- [ ] Move 1: `fir_base.rs` (trait Fir + defaults, FirKind, StepReport, UbcError, Scope);
      tests green; commit
- [ ] Move 2: `fir_ref.rs` (FirRef alias, FirRefExt/FirRefNavExt, step_inner + borrow docs);
      tests green; commit
- [ ] Move 3: `fir_search_base.rs` (contextful_search module + `_decide_nyes_due_to_children`);
      tests green; commit
- [ ] Moves 4..N: one `firs/<kind>_fir.rs` per commit, each with its private helpers and its
      `#[cfg(test)]` tests incl. `*_nyes_transitions`; tests green per commit
- [ ] Delete the emptied `fir_kinds.rs` / `fir_trait.rs`; `lib.rs` module map doc-comment;
      confirm the crate's public re-export surface is byte-unchanged (`cargo public-api` or
      manual diff of `lib.rs` pub use)
- [ ] Full gates: `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`,
      `cargo test --workspace`; einmo suite byte-identical (zero correspondence diffs)
- [ ] Verify all work complete and committed to `foop-05-fir-decomposition`
- [ ] Merge `foop-05-fir-decomposition` to `jia`
  - [ ] Merge breaking changes from `jia` first; re-run gates
  - [ ] STOP! STOP!! STOP!!! ASK HUMAN to check this box before continuing. UNDER NO
        CIRCUMSTANCES will Agent continue past this point automatically!!
    - [ ] Present `cd /home/hcbusy/tmp/foolish-worktrees/foop-05-fir-decomposition`; the review
          is `git log --follow` spot checks + the zero-diff einmo gate, not snapshots.
- [ ] Cleanup /home/hcbusy/tmp/foolish-worktrees/foop-05-fir-decomposition
  - [ ] Check that FOOP-05.plan.md has all but Cleanup checkboxes completed
  - [ ] Remove the worktree (`git worktree remove ...`)
  - [ ] This is the last sub-task checkbox to be checked in this block
