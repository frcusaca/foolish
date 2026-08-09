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

- [x] Begin work: commit FOOP-55.md and FOOP-55.plan.md to origin (`jia`),
      check `begun: [x]` in FOOP-55.md frontmatter
      (2026-08-09 13:40)
- [x] Create worktree at /storage1/human/hcbusy/foolish/../foolish_worktrees/foop-55-project-euler-1 with branch `foop-55-project-euler-1`
      (2026-08-09 13:41)
- [ ] Read `rust_instructions.md` in full (mandatory before any Rust;
      especially §"Phase-by-phase testing discipline")
- [ ] Read FOOP-55.md §Findings (D1–D6, E1–E5) and §Specification in full,
      plus FOOP-65.md §1–§3 (backtick equivalence, precedence, flat chain)
- [ ] Read FOOP-33.md §5.0 and §5.1 (the as-built comparison mechanism
      that `'mod` mirrors — including the three implementation decisions)
- [x] ~~VERIFY FOOP-65 is merged to `jia`~~ — **DEPENDENCY DROPPED
      (2026-08-09).** FOOP-65 (the backtick tail concatenator) is **not
      required**. Verified live on `jia` after the FOOP-75 merge: the
      exercise's juxtaposition application form (`{p=1} f`) evaluates
      correctly as written. FOOP-65 would only let the call sites be
      *rewritten* more readably (E4's mapping table) — it unblocks nothing.
      Proceed without it; E4 becomes optional polish, not a precondition.
- [x] VERIFY the exercise's own defects are fixed in
      `future_exercise_inputs/project_euler/1.foo.disabled` (moved out of
      the einmo input tree; restore it to
      `foolish-ubca/einmo_suite/input/exercises/project_euler/1.foo` when
      it runs):
      (E3) `INTERNAL_` prefix replaces every leading `_` name; (E1) accumulator
      consistently `sum35`; (E2) line 13's `<<#-N>>` index searches; (E5) six
      `$=` lines rewritten as explicit `(X)$` (FOOP-75 merged, attached search
      available; using explicit form for clarity). E4 backtick rewrite skipped
      (FOOP-65 not on jia).
      (2026-08-09 13:45)
- [x] **Requirements survey (2026-08-09, on `jia` post-FOOP-75-merge).**
      Each feature the exercise needs was probed live through
      `UbcaEvaluator` — parse *and* value, since several parse cleanly yet
      settle ECONSTANIC:

      | Need | Status |
      |---|---|
      | `=$` attached search | **works** (FOOP-75, merged) |
      | `'eq`, `'lt` | work → `'True` |
      | `<<#-N>>` SFF, `<name>` SF | work |
      | `~cond=(...)` value search | works |
      | `&#1` contexted index | works |
      | `{args} fn` juxtaposition | works |
      | **`'mod`** | **MISSING** — `{7,3,'mod}$` → `?(pattern='^'mod$', UNANCHORED, ECONSTANIC)` |
      | **`'or`** | **MISSING** — same shape |
      | `$=` (six lines) | does not parse — rewrite to `=$` (E5) |
      | `_name` | does not lex — `INTERN_` workaround (E3), decision in Phase 5 |

      `system/system.foo` declares only `'True 'False 'lt 'gt 'le 'ge 'eq`.
      So **`'mod` and `'or` are the only two features to build**; everything
      else is a mechanical edit to the exercise.
- [ ] Baseline run in the worktree:
      `RUSTUP_TOOLCHAIN=stable cargo run -q -p foolish-cli -- run foolish-ubca/einmo_suite/input/exercises/project_euler/1.foolish`
      — record the failure mode in this plan (it should now be an
      evaluation/semantic failure, NOT a parse error; a parse error means
      the exercise fixes are incomplete or a new problem appeared)
- [ ] Run all tests — old and new — and make sure they all pass correctly.

## Phase 1 — `'mod` (integer modulo system operator)

- [x] Read FOOP-55.md §Specification §1; read `foolish-ubca/src/system_foo.rs`
      (ComparisonFir 158-341, `operand_is_unevaluated_here` 351-358,
      `comparison_body` 384-391, `comparison_nyes_transitions` 627-669) and
      `foolish-ubca/src/compiler.rs` (`BodyOverride` 468-505)
      (2026-08-09 13:30)
- [x] Write the unit tests FIRST (they fail until implemented):
  - [x] `modulo_nyes_transitions` in `system_foo.rs` tests — REQUIRED by
        AGENTS.md for the new FIR kind; pins all three terminals exactly as
        `comparison_nyes_transitions` does (ECONSTANIC inside system.foo;
        CONSTANT with integer neighbours; NK with a non-integer neighbour)
        (2026-08-09 13:35)
  - [x] Modulo semantics tests via `compose_program_with_system`:
        `{7, 3, 'mod}$` → 1; `{0, 5, 'mod}$` → 0; truncation pinned:
        `{(-7), 3, 'mod}$` and `{7, (-3), 'mod}$` per Rust `%`;
        `{7, 0, 'mod}$` → NK reason "division by zero";
        brane operand → NK reason "modulo: non-integer operand"
        (2026-08-09 13:35)
- [x] Implement `'mod` per FOOP-55 §1:
  - [x] `system/system.foo`: add `'mod = ⬤` with the `!! modulo:` comment
        (2026-08-09 13:30)
  - [x] New FIR kind in `system_foo.rs` (enum-parameterized shape
        RECOMMENDED: `ArithOp::Mod`), two SFF operand lookups `<<#-2>>` /
        `<<#-1>>` via `build_operand`/`push_foolish_child_sff_marked`,
        two-phase `fir_op_step`, `combine` rules in FOOP-55 §1 order
        (ECONSTANIC-first — the load-bearing rule)
        (2026-08-09 13:35)
  - [x] Generalize the `BodyOverride` hook name table (`comparison_body` →
        system name table covering `ComparisonOp::ALL` + `'mod`); hook
        stays scoped to `system.foo`'s own top-level statements
        (2026-08-09 13:35)
  - [x] New `FirKind` arm + `constanic_clone_at` recoordination arm +
        display/searchable name `'mod`
        (2026-08-09 13:35)
- [x] Unit tests pass (`cargo test -p foolish-ubca --lib -- modulo` and the
      full `cargo test -p foolish-ubca --lib`)
      (2026-08-09 13:40)
- [x] Einmo inputs `foolish-ubca/einmo_suite/input/foop/55/mod_basic.foo`
      and `mod_edge.foo` (cover the semantics table incl. both NK reasons)
      (2026-08-09 13:30)
- [x] Run all tests — old and new — and make sure they all pass correctly.
      (2026-08-09 13:40)

## Phase 2 — `'or` (FVM-computed boolean OR — fallback from pure-Foolish)

**DESIGN CHANGE:** The pure-Foolish truth-table approach (FOOP-55 §2) was
tried and failed — the value search `T~A=A` inside system.foo can't resolve
when `A` is ECONSTANIC (no neighbours). The root never settles. Switched to
FOOP-73's fallback: `OrFir` as a dedicated FIR kind, same pattern as
`ComparisonFir`/`ModuloFir`. Records the decision in FOOP-55.md §2.

- [x] Read FOOP-55.md §Specification §2 and FOOP-73.md §Preferred design
      (2026-08-09 13:30)
- [x] Write the tests FIRST:
  - [x] Unit: all four `{A, B, 'or}$` rows via
        `compose_program_with_system`, asserting the result is the SAME
        `Rc` as `system.foo`'s `'True`/`'False` (identity, not just
        display); non-boolean argument (e.g. `{3, 'True, 'or}$`) → NK
        (2026-08-09 13:30)
  - [x] Einmo input `foolish-ubca/einmo_suite/input/foop/55/or_table.foo`
        (four rows + the non-boolean miss)
        (2026-08-09 13:30)
- [x] Implement: changed `'or` from pure-Foolish brane to `'or = ⬤` with
      `OrFir` FIR kind (referential identity check against 'True/'False)
      (2026-08-09 13:50)
- [x] SANITY CHECK — pure-Foolish approach failed (root never settles);
      FVM-computed fallback implemented per FOOP-73
      (2026-08-09 13:45)
- [x] Unit tests pass (or_all_four_rows, or_non_boolean_argument_settles_nk)
      (2026-08-09 13:50)
- [x] Run all tests — old and new — and make sure they all pass correctly.
      (2026-08-09 13:50)

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

## Phase 5 — The D1 decision (leading-underscore names)

Deferred to the end deliberately: the exercise runs on the `INTERN_`
workaround (E3), so this is a question about the *platform*, answered once
the exercise is green and the real cost is known.

- [ ] Confirm the workaround held: the exercise runs with `INTERN_` names
      and no leading-underscore identifier remains in it.
- [ ] Re-measure the defect on the merged tree — it may have shifted:
      ```
      {_x = 1; y = _x}
      ```
      Measured on `jia` 2026-08-09 (post-FOOP-75 merge):
      `ERR: expected primary expression, found LineComment at line 1,
      column 10` — the `_x` is silently swallowed and the error surfaces
      somewhere unrelated, which is what makes D1 inscrutable rather than
      merely restrictive.
- [ ] **DECIDE, and record the decision here**: fix the lexer so a leading
      `_` starts an identifier, or keep `INTERN_` as the standing
      convention?
      - Fixing it removes a papercut and an inscrutable error, but touches
        the lexer's identifier rule — check first whether `_` is load-
        bearing elsewhere (`is_id_sep` treats `_` as a *separator*, so a
        leading `_` may collide with that role; FOOP-55.md §D1's note that
        `_a` and `a` could become the *same* name is the specific hazard).
      - Keeping the workaround costs nothing now but leaves the trap armed
        for the next Foolisher.
- [ ] If FIX: it needs its own FOOP (it is a language-surface change with a
      naming-collision hazard, not part of "make the exercise run"). Create
      it via `foop_check.py gen_next` and record the number here.
- [ ] If KEEP: record `INTERN_` in AGENTS.md §Code Style as the convention,
      so it is a documented choice rather than a local hack.
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

## Last Updated

**Date**: 2026-08-09
**Updated By**: Claude Code / claude-opus-5
**Changes**: **Dropped the FOOP-65 dependency** — verified live on `jia`
after the FOOP-75 merge that the exercise's juxtaposition application form
evaluates correctly as written, so the backtick unblocks nothing and E4's
rewrite is optional polish. Recorded a **requirements survey** in Phase 0:
every feature the exercise needs was probed through `UbcaEvaluator` for
value as well as parse, leaving **`'mod` and `'or` as the only two features
to build**. The six `$=` lines become `=$` (FOOP-75 is merged). Added
**Phase 5 — the D1 decision**, deferred to the end deliberately: the
exercise runs on the `INTERN_` workaround, so whether to fix the
leading-underscore lexer defect is answered once the real cost is known,
and a fix would need its own FOOP.
