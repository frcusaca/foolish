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
- [ ] foolish-core migration (same treatment, per FOOP-64.md §foolish-core migration):
      scaffold `foolish-core/einmo_suite/` (same hierarchy + einmo.toml); inventory
      `foolish-core/snapshot_tests/` — determine the post-FOOP-62 evaluator for its inputs and
      `einmo flag` stale inputs with reasons for the human; attribution pass → its own
      MAPPING.md; copy inputs per rules R1–R12 (dual-home rule applies); add its
      `einmo_approval_all`; generate, self-review, promote `output->checked`; cross-validate
      against its approved `.snap` corpus
- [x] SANITY CHECK (done up-front 2026-07-14, before implementation): does einmo actually fail
      when a demanded checked/verified file is missing? **Yes** — `compare --require-match`
      exits 1 with `only-in-output` + burden message for both stages, and the library's
      `require_correspondence` reports `correspondence_failures` (einmo's own
      `correspondence_failure_reported_until_promoted` test pins it). **But two vacuous passes
      found**: an empty suite passes `compare --require-match` (exit 0), and
      `confirm-signatures --require-all` over an empty `verified/` passes (exit 0). Both gate
      tests must assert non-emptiness — recorded in FOOP-64.md §Two-tier signing gate.
      (2026-07-14 22:41)
- [ ] Two-tier gate, development tier ("feature-complete test suite"): the `einmo_approval_all`
      tests (both suites) are the checked-stage gate — confirm each fails on any
      output↔checked divergence; **assert `!results.files.is_empty()`** (anti-vacuity, per the
      sanity check above)
- [ ] Two-tier gate, merge tier ("merge-ready test suite"): add `einmo_verified_gate` test
      (`#[ignore]` locally), covering BOTH suites (`foolish-ubca/einmo_suite/`,
      `foolish-core/einmo_suite/`): output ↔ `verified/` correspondence (`--require-match`
      semantics) + `confirm_signatures(verified, <human-key-prefix>) --require-all` +
      zero-computer-key scan on `stage:verified` stamps; **assert `verified/` is non-empty AND
      its file count == `output/` count BEFORE the key assertions** (anti-vacuity, per the
      sanity check — `--require-all` passes trivially on an empty directory)
  - [ ] Obtain the human reviewer's public-key hex prefix (human derives it once; passphrase
        never recorded); embed as the gate constant
- [ ] Add `.github/workflows/einmo-gates.yml`: on PR — run `einmo_verified_gate`; on push —
      `einmo verify --all` over the suite (absorbed FOOP-92 Phase 11)
- [ ] STOP: ASK HUMAN to (a) run the initial verified signing sessions —
      `einmo promote "checked->verified" foolish-ubca/einmo_suite/ --interactive` and
      `einmo promote "checked->verified" foolish-core/einmo_suite/ --interactive` over the
      reviewed corpora (the merge-ready tier cannot pass without them) — and (b) enable the
      GitHub branch-protection setting making the einmo-gates workflow a required status check
      (repository settings are human-only)
- [ ] Docs & skills (in-worktree): update `AGENTS.md` — Development Rules restated as "codebase
      must pass the einmo checked stage (output matches signed checked einmos)"; Build Commands
      → einmo flow; ⚠️ CRITICAL section rewritten in einmo terms (AI may promote
      output->checked after review; AI must NEVER produce a verified stamp; insta-specific
      prohibitions retired with insta). Update `foop.md` — comprehensive path re-homing + FOOP
      merge criteria include the checked-stage gate. Update the three skills
      (`.opencode/skills/{foop-write-plan,foop-use-maintain,foolish-debugging}/SKILL.md`) —
      insta/.snap references → einmo flow. Stamp all Last Updated sections
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
          the parent checkbox. (The verified signing session already happened at the two-tier
          gate STOP above — confirm `einmo_verified_gate` passes before merging.)
- [ ] Retirement — **completion-blocking**: this FOOP is not complete until the repository has
      securely migrated off insta snapshots (insta removed from all dependencies)
  - [ ] Inventory remaining insta usage in `foolish-parser` (inline snapshot assertions);
        migrate to whatever einmo shape fits (small suite or unit-test rewrites) —
        `foolish-core` is already migrated by its dedicated task above
  - [ ] Cross-validate the parser migration the same way (content byte-match where applicable)
  - [ ] STOP: human deletes the `.snap` corpora (`foolish-ubca/snapshot_tests/`, and the
        parser/core equivalents) — agents may not move or alter `.snap` files; agent suggests
        the exact commands with case-inverted first words at that time
  - [ ] Agent: delete the insta test code (`approval_all` and parser/core insta tests) and
        `cross_validate_einmo_vs_insta`; remove `insta` from `foolish-ubca/Cargo.toml`,
        `foolish-parser/Cargo.toml`, `foolish-core/Cargo.toml`, and `[workspace.dependencies]`
  - [ ] Acceptance: `cargo tree -i insta` reports "package not found"; `cargo test --workspace`
        green; `foolish_review.sh` / `accept_approved.sh` disposition per the Open Question
- [ ] Cleanup /home/hcbusy/tmp/foolish-worktrees/foop-64-einmo-suite
  - [ ] Check that FOOP-64.plan.md has all but Cleanup checkboxes completed
  - [ ] Remove /home/hcbusy/tmp/foolish-worktrees/foop-64-einmo-suite
        (`git worktree remove ...`)
  - [ ] This is the last sub-task checkbox to be checked in this block
