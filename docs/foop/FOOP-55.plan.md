# FOOP-55.plan — project-euler-1

Read `FOOP-55.md` FIRST — this plan assumes the specification. Execute top
to bottom. Variables are already expanded to literals (per `foop.md`):

```
WORKTREE_ORIGIN_BRANCH=jia
WORKTREE_ORIGIN_PATH=/storage1/human/hcbusy/foolish
WORKTREE_BRANCH_NAME=foop-55-project-euler-1
WORKTREE_FULL_FS_PATH=/storage1/human/hcbusy/foolish/../foolish_worktrees/foop-55-project-euler-1
```

Note: the shell environment needs `RUSTUP_TOOLCHAIN=stable` exported (no
default toolchain is configured on this machine).

**Dependency gate: FOOP-65 (tail concatenator) must be merged to `jia`
before this plan starts** — the exercise rewrite (FOOP-55 E4) is written
in backtick application form. If FOOP-65 is not merged, STOP; do not
begin, and do not rewrite the exercise in juxtaposition form as a
stopgap.

## Phase 0 — Begin, baseline, and prerequisites

- [ ] Begin work: commit FOOP-55.md and FOOP-55.plan.md to origin (`jia`),
      check `begun: [x]` in FOOP-55.md frontmatter
- [ ] Create worktree at /storage1/human/hcbusy/foolish/../foolish_worktrees/foop-55-project-euler-1 with branch `foop-55-project-euler-1`
      (`git worktree add -b foop-55-project-euler-1 /storage1/human/hcbusy/foolish/../foolish_worktrees/foop-55-project-euler-1`
      from `jia` at /storage1/human/hcbusy/foolish; ALL subsequent work —
      including edits to FOOP-55.md and this plan — happens ONLY in the
      worktree until merge)
- [ ] Read `rust_instructions.md` in full (mandatory before any Rust;
      especially §"Phase-by-phase testing discipline")
- [ ] Read FOOP-55.md §Findings (D1–D6, E1–E5) and §Specification in full,
      plus FOOP-65.md §1–§3 (backtick equivalence, precedence, flat chain)
- [ ] Read FOOP-33.md §5.0 and §5.1 (the as-built comparison mechanism
      that `'mod` mirrors — including the three implementation decisions)
- [ ] VERIFY FOOP-65 is merged to `jia` (backtick tail concatenator
      implemented: `Token::Backtick`, `Astn::TailConcatenation`,
      `TailConcatenationFir`, einmo `foop/65/` baselines present). If not:
      STOP and wait — this plan does not proceed without it.
- [ ] VERIFY Atlas's exercise fixes have landed in
      `foolish-ubca/einmo_suite/input/exercises/project_euler/1.foolish`:
      (E3) `INTERN_` prefix replaces every leading `_` name; (E1) the
      accumulator is consistently `sum35` (or consistently `sum`); (E2)
      line 13's three `<<-N>>` are `<<#-N>>` index searches; (E4) the
      application sites use the backtick form per FOOP-55 E4's mapping
      table (e.g. `('cmod`{lv,3,0})$`, `('ite`{cond1, sum35+lv, sum35})$`,
      `(loop`{loop})$`); (E5) no `$=` remains — result extraction is
      explicit parentheses + `$`. If ANY is missing: STOP and ask Atlas —
      do NOT edit the exercise yourself.
- [ ] Baseline run in the worktree:
      `RUSTUP_TOOLCHAIN=stable cargo run -q -p foolish-cli -- run foolish-ubca/einmo_suite/input/exercises/project_euler/1.foolish`
      — record the failure mode in this plan (it should now be an
      evaluation/semantic failure, NOT a parse error; a parse error means
      the exercise fixes are incomplete or a new problem appeared)
- [ ] Run all tests — old and new — and make sure they all pass correctly.

## Phase 1 — `'mod` (integer modulo system operator)

- [ ] Read FOOP-55.md §Specification §1; read `foolish-ubca/src/system_foo.rs`
      (ComparisonFir 158-341, `operand_is_unevaluated_here` 351-358,
      `comparison_body` 384-391, `comparison_nyes_transitions` 627-669) and
      `foolish-ubca/src/compiler.rs` (`BodyOverride` 468-505)
- [ ] Write the unit tests FIRST (they fail until implemented):
  - [ ] `modulo_nyes_transitions` in `system_foo.rs` tests — REQUIRED by
        AGENTS.md for the new FIR kind; pins all three terminals exactly as
        `comparison_nyes_transitions` does (ECONSTANIC inside system.foo;
        CONSTANT with integer neighbours; NK with a non-integer neighbour)
  - [ ] Modulo semantics tests via `compose_program_with_system`:
        `{7, 3, 'mod}$` → 1; `{0, 5, 'mod}$` → 0; truncation pinned:
        `{(-7), 3, 'mod}$` and `{7, (-3), 'mod}$` per Rust `%`;
        `{7, 0, 'mod}$` → NK reason "division by zero";
        brane operand → NK reason "modulo: non-integer operand"
- [ ] Implement `'mod` per FOOP-55 §1:
  - [ ] `system/system.foo`: add `'mod = ⬤` with the `!! modulo:` comment
  - [ ] New FIR kind in `system_foo.rs` (enum-parameterized shape
        RECOMMENDED: `ArithOp::Mod`), two SFF operand lookups `<<#-2>>` /
        `<<#-1>>` via `build_operand`/`push_foolish_child_sff_marked`,
        two-phase `fir_op_step`, `combine` rules in FOOP-55 §1 order
        (ECONSTANIC-first — the load-bearing rule)
  - [ ] Generalize the `BodyOverride` hook name table (`comparison_body` →
        system name table covering `ComparisonOp::ALL` + `'mod`); hook
        stays scoped to `system.foo`'s own top-level statements
  - [ ] New `FirKind` arm + `constanic_clone_at` recoordination arm +
        display/searchable name `'mod`
- [ ] Unit tests pass (`cargo test -p foolish-ubca --lib -- modulo` and the
      full `cargo test -p foolish-ubca --lib`)
- [ ] Einmo inputs `foolish-ubca/einmo_suite/input/foop/55/mod_basic.foo`
      and `mod_edge.foo` (cover the semantics table incl. both NK reasons);
      run `run_einmo_tests`, review the OUTPUT, justify EVERY line per
      AGENTS.md step 4, then promote ONLY these foop/55 baselines
- [ ] Run all tests — old and new — and make sure they all pass correctly.

## Phase 2 — `'or` (pure-Foolish truth-table boolean OR)

- [ ] Read FOOP-55.md §Specification §2 (incl. the four-row trace table and
      the four preconditions) and FOOP-73.md §Preferred design
- [ ] Write the tests FIRST:
  - [ ] Unit: all four `{A, B, 'or}$` rows via
        `compose_program_with_system`, asserting the result is the SAME
        `Rc` as `system.foo`'s `'True`/`'False` (identity, not just
        display); non-boolean argument (e.g. `{3, 'True, 'or}$`) → NK
  - [ ] Einmo input `foolish-ubca/einmo_suite/input/foop/55/or_table.foo`
        (four rows + the non-boolean miss)
- [ ] Implement: add the `'or` truth-table brane to `system/system.foo`
      EXACTLY as FOOP-55 §2 (flat 12-statement table, row grouping is
      load-bearing; `A = <<#-2>>; B = <<#-2>>` both `#-2`)
- [ ] Verify the four preconditions empirically on the live FVM (flat
      splice of brane values in concatenation; search-as-value-pattern;
      referential creation equality in the matcher; `&#1` landing on the
      row result). Record the evidence (test names) in this plan
- [ ] SANITY CHECK — if the table search proves insufficient or wrong
      despite the trace: STOP and consult Atlas before taking FOOP-73's
      FVM-computed fallback (it reintroduces a privileged layer); record
      the decision in FOOP-55.md §2 and update FOOP-73.md accordingly
- [ ] Einmo `or_table.foo` OUTPUT reviewed (every line justified) and
      promoted; unit tests pass
- [ ] Run all tests — old and new — and make sure they all pass correctly.

## Phase 3 — Integration: make the exercise run

- [ ] Read FOOP-55.md §4 (integration risks); load the `foolish-debugging`
      skill — all diagnosis below is unit-test-driven
      (`temporary_reproduce_to_debug_*` tests, promoted to regression tests
      or deleted per the skill)
- [ ] Run the exercise; record the first failure; debug incrementally.
      Expected fronts (each fix gets its own regression test):
  - [ ] `'cmod` composition: `{lv,3,0}'cmod` settles to the right boolean
        (exercises flat splice + `#-3` bindings + `'eq` + `'mod` together)
        — einmo input `foop/55/cmod.foo`
  - [ ] `'ite` mechanism: `{C, T, F, 'ite}` selects T/F correctly — einmo
        input `foop/55/ite.foo` (small, hand-checkable cases)
  - [ ] Recursion: `<self loop>` / `continue` / `{loop} loop` entry shape
  - [ ] MEASURE the depth/budget question (FOOP-55 §4 risks 1-2): does the
        ~1000-deep recursion hit `MAX_DEPTH` (fir_trait.rs:395) or the
        iteration cap (evaluator.rs:168)? If yes: consult Atlas if the
        remedy is semantic; implement the chosen remedy with tests; record
        the decision in FOOP-55.md §4
  - [ ] `lv = lv+1` reassignment reads the parent-context value correctly
- [ ] Einmo input `foop/55/euler_small.foo` — the exercise's algorithm with
      the bound 10 instead of 1000; expected answer 23 (hand-checkable:
      3+5+6+9); review + justify + promote
- [ ] The exercise itself:
      `RUSTUP_TOOLCHAIN=stable cargo run -q -p foolish-cli -- run foolish-ubca/einmo_suite/input/exercises/project_euler/1.foolish`
      evaluates to constanic and the answer reads **233168**
- [ ] Run all tests — old and new — and make sure they all pass correctly.

## Phase 4 — Comprehensive test and the exercise baseline

- [ ] Write and verify `foolish-ubca/einmo_suite/input/foop/55/comprehensive.foo`
      (reserved name; mix `'mod`/`'or` with comparisons, value search,
      contexted search, concatenation, SF/SFF markers; at least one path
      through every new behavior this FOOP adds; slight repetition of
      earlier tests is fine when it serves coverage)
- [ ] Run all tests — old and new — and make sure they all pass correctly.
- [ ] Exercise einmo baseline: review the OUTPUT of
      `input/exercises/project_euler/1.foolish` (answer 233168, plus alarms
      and step count), justify every line, then present it to Atlas for
      human review and checked-stage promotion — final approval and any
      verified-stage signature require the human (einmo.toml leaves
      `verified` unconfigured on purpose). NEVER auto-promote; never
      promote over any foreign FOOP's baseline
- [ ] Run all tests — old and new — and make sure they all pass correctly.

## Merge

- [ ] Verify all work is complete in /storage1/human/hcbusy/foolish/../foolish_worktrees/foop-55-project-euler-1 and committed to `foop-55-project-euler-1`
- [ ] Merge `foop-55-project-euler-1` to `jia`
  - [ ] Run all tests — old and new — and make sure they all pass correctly.
  - [ ] Check and make sure current foop has, and passes, its comprehensive
        snaptest (`einmo_suite/input/foop/55/comprehensive.foo`)
  - [ ] Run all tests — old and new — and make sure they all pass correctly.
  - [ ] (If complex merge situation: repair work sub-tasks land here, each
        timestamped)
  - [ ] STOP! STOP!! STOP!!! ASK HUMAN to check this box before continuing.
        UNDER NO CIRCUMSTANCES will Agent continue past this point
        automatically!!
    - [ ] Present human with the
          `cd /storage1/human/hcbusy/foolish/../foolish_worktrees/foop-55-project-euler-1`
          command and ask them to review snapshots BEFORE checking the
          parent checkbox.
  - [ ] Cleanup /storage1/human/hcbusy/foolish/../foolish_worktrees/foop-55-project-euler-1
    - [ ] Check that FOOP-55.plan.md has all but Cleanup checkboxes completed
    - [ ] Remove /storage1/human/hcbusy/foolish/../foolish_worktrees/foop-55-project-euler-1
          (`git worktree remove` + branch deletion after merge)
    - [ ] This is the last sub-task checkbox to be checked in this block of
          subtasks
