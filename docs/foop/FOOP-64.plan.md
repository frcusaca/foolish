# FOOP-64.plan — einmo-suite (migrate UBCa snapshots to hierarchical einmo)

Read `docs/foop/FOOP-64.md` before executing. Execute top to bottom. Once work begins, all
updates — including to this plan — happen ONLY in the worktree until merge.

Worktree literals (origin branch is `jia`: this clone has no `alpha`; `jia` is the integration
branch used by the FOOP-62 and FOOP-92 merges):

- Origin branch: `jia`, origin path: `/home/hcbusy/foolish-rust`
- Branch: `foop-64-einmo-suite`
- Worktree: `/home/hcbusy/tmp/foolish-worktrees/foop-64-einmo-suite`

---

- [ ] Begin work: verify `cargo test --workspace` is green on `jia`; commit FOOP-64.md and
      FOOP-64.plan.md; check `begun: [x]` in FOOP-64.md frontmatter
- [ ] Create worktree at /home/hcbusy/tmp/foolish-worktrees/foop-64-einmo-suite with branch
      `foop-64-einmo-suite` (`git worktree add -b foop-64-einmo-suite
      /home/hcbusy/tmp/foolish-worktrees/foop-64-einmo-suite`), from `jia` at
      /home/hcbusy/foolish-rust
- [ ] (read §Suite layout and §Harness of FOOP-64.md)
- [ ] Scaffold `foolish-ubca/einmo_suite/` — `input/` hierarchy dirs, `einmo.toml`
      (`[signing.output]` / `[signing.checked]` passphrase `""`; `verified` unset)
- [ ] Harness: add `einmo` dev-dependency to `foolish-ubca/Cargo.toml`; copy
      `UbcaEvaluatorAdapter` from `zweimomo/src/evaluators.rs` into
      `foolish-ubca/src/ubca_snapshot_tester.rs`; add `einmo_approval_all` test
      (`TestConfig::new(...).foolish_separator().require_correspondence(Stage::Output,
      Stage::Checked)`) — insta `approval_all` stays untouched
- [ ] (read §Placement rules and §Dual-home rule of FOOP-64.md)
- [ ] Attribution pass: `git log --follow` each of the 162 inputs; apply rules R1–R12; write
      `foolish-ubca/einmo_suite/MAPPING.md` (old path → new path, prefix strips, dual-home pairs)
- [ ] Copy all inputs from `foolish-ubca/snapshot_tests/input/` into
      `foolish-ubca/einmo_suite/input/` per MAPPING.md (copy, never move; `snapshot_tests/`
      remains fully intact and green)
- [ ] (read §Proposed new combination tests of FOOP-64.md)
- [ ] Author the eight new combination `.foo` inputs under `input/foop/64/` (+ dual-home
      `lang/usecases/` copies where they read as demonstrations)
- [ ] Write and verify `foolish-ubca/einmo_suite/input/foop/64/comprehensive.foo` (first
      comprehensive at the new reserved path)
- [ ] Generate + self-review: run `einmo_approval_all` to fill `output/`; inspect every new
      `.einmo`; `cargo build -p einmo` then
      `./target/debug/einmo promote "output->checked" foolish-ubca/einmo_suite/` (computer key);
      commit `checked/`
- [ ] Cross-validate: add `#[ignore]`d `cross_validate_einmo_vs_insta` (checked OUTPUT sections
      byte-match approved `.snap` RESULT sections, read-only on `.snap`); run it; fix any
      transport drift until clean
- [ ] Docs: update `foop.md` §Comprehensive FOOP Tests and `AGENTS.md` to the new reserved path
      `foolish-ubca/einmo_suite/input/foop/<NUMBER>/comprehensive.foo`; stamp both Last Updated
      sections
- [ ] Full gates in worktree: `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`,
      `cargo test --workspace` (insta suite AND einmo suite both green)
- [ ] Verify all work is complete in /home/hcbusy/tmp/foolish-worktrees/foop-64-einmo-suite and
      committed to `foop-64-einmo-suite`
- [ ] Merge `foop-64-einmo-suite` to `jia`
  - [ ] Confirm `foop/64/comprehensive.foo` exists, is reviewed, and passes in the einmo suite
  - [ ] Merge breaking changes from `jia` into the worktree first; resolve; re-run full gates
  - [ ] Repair ALL tests on `jia` in /home/hcbusy/foolish-rust after merge
  - [ ] STOP! STOP!! STOP!!! ASK HUMAN to check this box before continuing. UNDER NO
        CIRCUMSTANCES will Agent continue past this point automatically!!
    - [ ] Present human with `cd /home/hcbusy/tmp/foolish-worktrees/foop-64-einmo-suite` and ask
          them to review `einmo_suite/checked/` (diff vs old snaps via
          `cross_validate_einmo_vs_insta`, and `einmo show` on new foop/64 tests) BEFORE checking
          the parent checkbox. Optionally: human signs the corpus with
          `einmo promote "checked->verified" foolish-ubca/einmo_suite/ --interactive`
- [ ] Retirement (human-gated, post-merge; agent may not touch `.snap` files or run the removal)
  - [ ] Human decides retirement timing of `foolish-ubca/snapshot_tests/` + insta `approval_all`
        + insta dev-dependency; agent suggests exact commands with case-inverted first words at
        that time
  - [ ] Delete `cross_validate_einmo_vs_insta` in the same change (agent task, once human
        approves retirement)
- [ ] Cleanup /home/hcbusy/tmp/foolish-worktrees/foop-64-einmo-suite
  - [ ] Check that FOOP-64.plan.md has all but Cleanup checkboxes completed
  - [ ] Remove /home/hcbusy/tmp/foolish-worktrees/foop-64-einmo-suite
        (`git worktree remove ...`)
  - [ ] This is the last sub-task checkbox to be checked in this block
