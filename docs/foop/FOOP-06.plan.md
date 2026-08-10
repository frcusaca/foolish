# FOOP-06.plan — output-gen-stage

Implementation plan for [FOOP-06](FOOP-06.md): generate to `output.gen/`, promote `output/` to a
committed baseline, and make each validation level compare against its immediate predecessor only.

**Read `FOOP-06.md` in full before executing this plan.** Section pointers appear inline below.

Worktree variables (already expanded to literals throughout):

```
WORKTREE_ORIGIN_BRANCH=jia
WORKTREE_ORIGIN_PATH=/storage1/human/hcbusy/foolish
WORKTREE_BRANCH_NAME=foop-06-output-gen-stage
WORKTREE_FULL_FS_PATH=/storage1/human/hcbusy/foolish/../foolish_worktrees/foop-06-output-gen-stage
```

---

## Phase 0 — Resolve blocking design questions

The Open Questions in FOOP-06 §Open Questions are **not** all deferrable. The signing question in
particular decides the shape of the core comparison, so it is answered before any code is written.

- [ ] Read §Specification and §Open Questions of FOOP-06.md
- [ ] **RESOLVE (blocking): does `output.gen/` get signed, and does `output.gen ≡ output` compare
      whole files or only `MatchSections`?** Metadata headers (`suite:` path, `producer:` commit,
      `generated:` timestamp, einmo binary hash) legitimately differ between runs, so a whole-file
      comparison would fail spuriously. Present the options and a recommendation to the human;
      record the decision in FOOP-06.md §Specification before proceeding.
- [ ] **RESOLVE: migration of the existing 174 `output/` artifacts** — accept today's committed
      content as the baseline, or regenerate first? Note the tracked files carry stale `suite:`
      paths from the repo move. Record the decision in FOOP-06.md.
- [ ] **RESOLVE: `output.gen/` vs `output.new/`** as the directory name. Record it.
- [ ] **RESOLVE: does divergence share a level with runtime error** (3 badges) or split (4 badges)?
      Record it.
- [ ] Update FOOP-06.md with all four resolutions; remove them from §Open Questions
- [ ] STOP! ASK HUMAN to confirm the four resolutions before implementation begins.

---

## Phase 1 — Begin work and set up the worktree

- [ ] Begin work: commit FOOP-06.md and FOOP-06.plan.md to origin, check `begun: [x]` in frontmatter
- [ ] Create worktree at /storage1/human/hcbusy/foolish/../foolish_worktrees/foop-06-output-gen-stage with branch `foop-06-output-gen-stage`
- [ ] Confirm the suite is green at HEAD before changing anything:
      `cargo test --workspace` (expect 583 passed / 0 failed)

---

## Phase 2 — `output.gen/` as the generation stage

- [ ] Read §Specification "Evaluation writes only to `output.gen/`" of FOOP-06.md
- [ ] Add the `output.gen` stage to einmo's stage model (`einmo/src/config.rs` `StageNames`,
      `einmo/src/einmo_suite.rs` `Stage`)
- [ ] Point `evaluate_all` at `output.gen/` — it must never write `output/`
- [ ] Move catastrophe-crumb creation into `output.gen/` so a crash leaves no committed stage dirty
- [ ] **ASK HUMAN to add `output.gen/` to `.gitignore`** — agents must not edit `.gitignore`
      (`AGENTS.md` §Important Safety Guide Rails). Supply the exact line to add.
- [ ] Unit tests: generation writes only to `output.gen/`; `output/` is untouched by evaluation
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 3 — Redefine the validation levels as predecessor-only links

- [ ] Read §Specification "Each level compares against its immediate predecessor only" of FOOP-06.md
- [ ] Replace the cumulative `ValidationLevel` with `Generation` / `Checked` / `Verified` per the
      spec's API shape
- [ ] `Generation`: evaluate into `output.gen/`, then require `output.gen ≡ output` (per the Phase-0
      comparison decision)
- [ ] Add distinct `Problem` variants for runtime error vs divergence
- [ ] `Checked` and `Verified`: express in terms of `compare()` — **must not evaluate and must not
      write**. Assert this in tests, not only in review.
- [ ] Unit tests for each level, including the no-evaluation / no-write assertions
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 4 — The `output.gen → output` promotion

- [ ] Read §Specification "The new promotion" of FOOP-06.md
- [ ] Implement `einmo promote output.gen to output` (`einmo/src/cli.rs`), producing correctly
      signed `output/` artifacts
- [ ] Ensure the CLI help text states the **weak** claim: "ran without error, output is reasonable
      — not a semantic or stylistic review". It must read as clearly weaker than
      `promote output to checked`.
- [ ] Verify `promote output to checked` and `promote checked to verified` still behave as before
- [ ] Unit tests for the new promotion
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 5 — UBCa gates and the mutex

- [ ] Read §Test Plan of FOOP-06.md
- [ ] Rename the three gates in `foolish-ubca/src/ubca_snapshot_tester.rs` to match the levels
- [ ] Keep `GATE_LOCK` on the generation gate only (the sole writer); **update — do not delete —
      the module docs added in commit `0a356f88`** so the race hazard stays recorded for whoever
      adds the next writing gate
- [ ] Add a test that runs the three gates concurrently and asserts all pass (the regression test
      for the race that motivated the lock)
- [ ] Confirm `cargo test --workspace` passes with default parallelism and no `--test-threads=1`
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 6 — Migration of the existing baseline

- [ ] Execute the migration decided in Phase 0
- [ ] Generate into `output.gen/` and diff against the committed `output/`
- [ ] **Report the diff to the human in ONE statement** before promoting: how many artifacts differ,
      and whether any *evaluated OUTPUT* differs (as opposed to metadata headers only). Evaluated
      OUTPUT differing at this stage means this FOOP changed behavior, which §UBC Step Impact says
      it must not — that is a bug to fix, not a diff to accept.
- [ ] Promote `output.gen → output` once the human accepts
- [ ] Confirm all three levels green
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 7 — Documentation and badges

- [ ] Update `README.md` with the three badges (`output.gen ≡ output`, `output ↔ checked`,
      `checked ↔ verified`) and a sentence on transitivity
- [ ] Update `AGENTS.md` §"Approval Tests (einmo)" — four stages, the new promotion, and the
      distinction between the weak `output.gen → output` claim and the `output → checked` gate
- [ ] Update `rust_instructions.md` §"Phase-by-phase testing discipline" — the three-stage contract
      becomes four; state which stage is the work file
- [ ] Update `foop.md` §"Promotion Review Gate" — confirm it still reads as governing
      `output → checked` **only**, and is not weakened by the new sibling promotion
- [ ] Update both FOOP skills (`foop-write-plan`, `foop-use-maintain`) for the new stage and commands
- [ ] Update FOOP-64.md with a pointer noting FOOP-06 amends its stage model (do not rewrite its
      history; add a forward reference)
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 8 — Comprehensive test, merge, and cleanup

- [ ] Write and verify `foolish-ubca/einmo_suite/input/foop/06/comprehensive.foo`
      (exercises the new stage model end to end; see `foop.md` §Comprehensive FOOP Tests)
- [ ] Run all tests — old and new — and make sure they all pass correctly.
- [ ] Review and promote `output` → `checked` for `foop/06/comprehensive`
  - [ ] Confirm the rest of the suite is green — no foreign-FOOP baseline diverges
        (a foreign divergence is a regression I introduced: fix the code, do not promote)
  - [ ] Confirm the case has no `verified/` twin (if it does: STOP, ask the human)
  - [ ] Re-read the in-force specification for the feature under test: FOOP-06.md §Specification
  - [ ] Review `foop/06/comprehensive` — every OUTPUT statement justified
  - [ ] Write the justification summary into the plan or commit message: what it demonstrates and
        why its result is spec-correct
  - [ ] Report ALL accumulated doubts to the human in ONE statement — or record "no doubts".
        Blocking doubts stop here; non-blocking ones are reported alongside.
  - [ ] Run `einmo promote output to checked foolish-ubca/einmo_suite`
  - [ ] Re-run the checked gate — must exit 0
- [ ] Run all tests — old and new — and make sure they all pass correctly.
- [ ] Verify all work is complete in /storage1/human/hcbusy/foolish/../foolish_worktrees/foop-06-output-gen-stage and committed to `foop-06-output-gen-stage`
- [ ] Merge `foop-06-output-gen-stage` to `jia`
  - [ ] Run all tests — old and new — and make sure they all pass correctly.
  - [ ] Repair ALL tests in `jia` in /storage1/human/hcbusy/foolish
  - [ ] STOP! STOP!! STOP!!! ASK HUMAN to check this box before continuing. UNDER NO CIRCUMSTANCES will Agent continue past this point automatically!!
    - [ ] Present human with the `cd /storage1/human/hcbusy/foolish/../foolish_worktrees/foop-06-output-gen-stage` command and ask them to review snapshots BEFORE checking the parent checkbox.
  - [ ] Cleanup /storage1/human/hcbusy/foolish/../foolish_worktrees/foop-06-output-gen-stage
    - [ ] Check that FOOP-06.plan.md has all but Cleanup checkboxes completed
    - [ ] Remove /storage1/human/hcbusy/foolish/../foolish_worktrees/foop-06-output-gen-stage
    - [ ] This is the last sub-task checkbox to be checked in this block

---

## Last Updated

**Date**: 2026-08-09
**Updated By**: Claude Code / claude-opus-5
**Changes**: Initial plan. Phase 0 front-loads four blocking design resolutions (chiefly whether
`output.gen ≡ output` compares whole files or only `MatchSections`, since metadata headers
legitimately differ between runs) behind a human STOP, before any code is written.
