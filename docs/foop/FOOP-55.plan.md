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

~~**Dependency gate: FOOP-65 must be merged before this plan starts**~~ —
**DROPPED 2026-08-09.** Verified live on `jia` after the FOOP-75 merge that the
exercise's juxtaposition application form (`{p=1} f`) evaluates correctly as
written, so FOOP-65's backtick rewrite (E4) is optional polish, not a
precondition. This plan has **no FOOP dependencies**. (See Phase 0's checked
item for the verification.)

## Execution order — CORRECTNESS FIRST (Atlas, 2026-08-14)

This plan is **not** executed top to bottom any more. Two phases were promoted
ahead of the rest because they are correctness work, not features:

| # | Phase | Why it is first |
|---|-------|-----------------|
| **1** | **3A — the SFF strip budget** | The mechanism ships and behaves correctly, but its ONLY test asserts nothing (see 3A). Nothing defends the behaviour; a refactor could silently break it and every gate would stay green. |
| **2** | **3G — concatenation ergonomics** | Not a nice-to-have. The old code is hard to understand, and unreadable correct code decays into incorrect code. The jia merge proved the cost concretely: an OLDER four-kind classifier had been living there, contradicting §9.2 in two places (`BareConcat`, `SfBrane`), and nobody had noticed. |
| 3 | 3G.5 / D9 | Unblocks `'match`, fibonacci, and probably the three `extremum_*` failures — all one question: what survives a constanic clone. |
| 4 | everything else | 3B, 3C, 3C2, 3E, 3D, 3F in their existing order |

**Rationale for putting readability under "correctness".** `AGENTS.md`
§"Development Rules" ranks adherence-to-spec, then coherence, then elegance —
and says elegance is last "because it never outranks correctness — but it is on
the list because unreadable correct code decays into incorrect code." §9 is that
last clause: the concatenation compiler had already decayed into a state where
two implementations of the same rule coexisted on different branches. Making it
legible IS the correctness work.

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

## Phase 2B — Restore the tree to `'mod` + `'or` only

**Done 2026-08-10.** Phase 3 work-in-progress (`IteFir`, `CmodFir`, and their
`system.foo` declarations) was uncommitted in the worktree; §5 makes both
unnecessary, so it was discarded rather than finished. `'mod` and `'or` were
already committed in `f8d3ea77` and are untouched — no rollback or cherry-pick
was needed.

- [x] Discard the uncommitted `'ite`/`'cmod` WIP; keep committed `'mod`/`'or`
      (2026-08-10 00:20)
- [x] `system/system.foo` declares `'True 'False 'lt 'gt 'le 'ge 'eq 'mod 'or`
      and **no** `'ite`/`'cmod`
      (2026-08-10 00:20)
- [x] Working tree clean; baseline recorded below
      (2026-08-10 00:20)

**Baseline at the start of §5** (`cargo test -p foolish-ubca --lib
-- --test-threads=1`): **332 passed, 2 failed.** Both failures are
`einmo_gate_checked` and `einmo_gate_verified`, and both report only *missing*
baselines — FOOP-55's own new tests (`foop/55/{cmod,ite,euler_small,mod_basic,
mod_edge,or_table}`, `exercises/project_euler/1`) have never been promoted to
`checked/`. **No pre-existing baseline diverges.** Promotion happens in Phase
3C/4 through the Promotion Review Gate, not before.

> **Run the suite with `--test-threads=1` on this branch.** The three einmo
> gates share `einmo_suite/output/` and race under the default parallel runner,
> which shows up as a spurious `einmo_gate_output` failure. `jia` has the
> `GATE_LOCK` fix (commit `0a356f88`); this branch predates it and will inherit
> it at merge.

## Phase 2C — `congruent_modulo` in pure Foolish (the §5 proof case)

`'cmod` is not reimplemented as a Rust FIR kind. Once the SFF mark defers
correctly it can be **defined in Foolish**, which makes it the natural
end-to-end test that §5 works. Spelled out in full — `congruent_modulo`, not
`'cmod` — since it becomes part of `system.foo`.

- [ ] First as a **test case**, not in `system.foo`: define `congruent_modulo`
      in Foolish inside an einmo input under `foop/55/SFF/` and confirm
      `{a, b, c} congruent_modulo` computes `a % b == c`. This is the honest
      proof that the upgraded mark carries a real definition, not just a toy.
- [ ] Only if that works, promote it into `system/system.foo` as
      `congruent_modulo`
- [ ] Run all tests — old and new — and make sure they all pass correctly.

## Phase 3A — §5: the SFF strip budget  ← **PRIORITY 1 (correctness)**

**STATUS (audited 2026-08-14): the MECHANISM is implemented and behaves
correctly; the VERIFICATION does not exist.** Both halves matter, and the second
is why a reviewer reported this phase as never done.

What is genuinely in place:

- `StripBudget` — `foolish-ubca/src/fir_kinds.rs:139`. `Copy`, passed **by
  value**, which is exactly what makes the budget per-root-to-leaf-PATH rather
  than per-clone-tree: descending inherits the parent's remaining budget, but
  spending it in one child cannot affect that child's siblings.
- Spent at `fir_kinds.rs:264` via `StripBudget::spend()`, gating the strip.
- Threaded through `clone_children_budgeted` and `constanic_clone_at_budgeted`
  down every recursive path.

Measured behaviour (2026-08-14), which agrees with §5:

| Probe | Result | Reading |
|-------|--------|---------|
| `{a = 5; b = << <<a>> >>; c = b;}` | `c`'s clone carries ONE `<<>>` | nested marks on one path: outer stripped, inner kept — a deferral count |
| `{x = {1,2}; y = <<#-1>> + <<#-1>>;}` | both operands keep their marks | siblings are independent, each with its own budget |

**The defect: the only budget test in the codebase asserts nothing.**
`strip_budget_is_per_clone_tree_not_per_node` (`fir_kinds.rs:5773`) wraps its
sole assertion in two `if let Some(...)` guards, and `inner_stmt` resolves to
`None` — verified by replacing the guards with `expect` and watching the test
panic with "asserted NOTHING". **It has passed vacuously since it was written,
whether or not the budget worked.** Its name also still says `per_clone_tree`,
the design 12 tests later disproved and §5 corrected to per-path.

A test that cannot fail is worse than a missing test: a missing test is visibly
absent, while this one reported the feature verified on every run. It is the
reason "was the budget implemented?" could not be answered from the suite.

- [ ] Read `rust_instructions.md` in full (mandatory before any Rust)
- [ ] Read FOOP-55.md §5 and Appendix A
- [x] **DELETE `strip_budget_is_per_clone_tree_not_per_node`** (2026-08-14 18:20) — — do not repair
      it. Its name encodes the superseded design, and its navigation is what
      silently returned `None`. Replace with the tests below.
- [ ] **Tests FIRST**, each one asserting UNCONDITIONALLY — no `if let` around
      an assertion, ever. Where a navigation may fail, `expect()` with a message
      naming what was being navigated to.
  - [x] one clone strips exactly one mark — `strip_budget_spends_one_mark_per_path` (2026-08-14 18:20)
  - [x] budget is **per-PATH** — `strip_budget_is_per_path_not_per_tree_so_siblings_are_independent` (2026-08-14 18:20)
  - [ ] budget is **per-use-site**: `B = A` and `C = A` decrement independently
  - [ ] a retained mark is **shared** — assert `Rc::ptr_eq` against the
        original, not a deep copy
  - [ ] the retained path never reaches the "SF/SFF node has no children" ALARM
  - [x] **anti-vacuity guard** — the nesting test asserts the SOURCE carries 2
        marks before comparing the clone, so wrong navigation fails loudly;
        `peel_marks` `expect`s rather than returning `None` (2026-08-14 18:20)
- [x] Confirmed by MUTATION (2026-08-14 18:20) — two mutants, each caught by the
      test written to exclude it: (1) always-strip → the nesting test failed
      (0 marks survived, 1 expected); (2) shared per-TREE budget via a
      thread-local → the sibling test failed (sibling 1 kept its mark, the
      `'mod` breakage). Neither mutant was caught by the OLD test, which passed
      under both. A test that stays green with
      the feature removed is not testing the feature — this is the specific
      failure being corrected here.
- [x] Renamed — no `per_clone_tree` remains (2026-08-14 18:20)
- [ ] Add the style rule to `AGENTS.md` §Code Style: nested marks are written
      `<< <<A>> >>` or `<<(<<A>>)>>`, never `<<<<A>>>>`. (All three already lex
      identically — this is convention, not grammar.)
- [ ] Run all tests — old and new — and make sure they all pass correctly.

## Phase 3B — §5: the existing SFF corpus

17 einmo inputs use SFF marks. Exactly ONE nests. The single-mark cases are the
regression gate for this whole FOOP.

- [ ] Confirm the **15 single-mark inputs produce byte-identical OUTPUT**, step
      counts included. Any divergence is a regression in the strip budget — fix
      the code, do not promote. The 15 are every `<<`-bearing input except
      `misc/sff_nested.foo` and `misc/concat_sf_f_more.foo` (below).
- [ ] `misc/sff_nested.foo` — `{a=1,b=2; c=<<a+<<b>>>>; c; c;}` — is a **direct
      semantic conflict**: its inner `<<b>>` means "resolve on each use" today
      and "defer one coordination" after §5.
  - [ ] Record the old and new OUTPUT side by side in this plan
  - [ ] STOP! ASK HUMAN to approve deprecating this input in favour of the new
        `foop/55/SFF/` cases. Do NOT rewrite a baseline whose meaning changed
        without review.
- [ ] `misc/concat_sf_f_more.foo` — found nesting an SFF mark 2026-08-14 while
      bisecting the FOOP-65 regression (see plan 3G.6c, FOOP-55.md §D11's
      sibling finding). `oo` changes from `-54` to `-116`: the old
      all-strip-in-one-pass code resolved a nested `<<...>>` mark too early;
      §5's per-path budget correctly defers it one extra coordination. Same
      disposition as `misc/sff_nested.foo` — **not a bug**, but a meaning
      change requiring human approval before deprecating/re-baselining.
  - [ ] Record the old and new OUTPUT side by side in this plan
  - [ ] STOP! ASK HUMAN to approve deprecating this input (or re-baselining it
        with an explanatory comment) in favour of the new `foop/55/SFF/` cases.
        Do NOT rewrite a baseline whose meaning changed without review.
- [ ] Run all tests — old and new — and make sure they all pass correctly.

## Phase 3C — §5: the new `foop/55/SFF/` einmo suite

- [ ] Write the inputs under `foolish-ubca/einmo_suite/input/foop/55/SFF/`:
  - [ ] `single_mark_unchanged.foo`
  - [ ] `double_mark_defers.foo`
  - [ ] `budget_is_per_tree.foo`
  - [ ] `budget_is_per_use_site.foo`
  - [ ] `nested_in_expression.foo`
  - [ ] `deferred_avoids_premature_nk.foo` (the `A=<{...}>; B=A; C=B` case —
        resolves at `C` instead of dying NK at `B`)
  - [ ] `separator_forms_agree.foo` (`<< <<A>> >>` vs `<<(<<A>>)>>`)
  - [ ] `nest_case1_syntactic.foo` (marks on ONE term — depth is a count)
  - [ ] `nest_case2_search_chain.foo` (marks on SEPARATE terms — one hop per
        link; **must be byte-identical before and after §5**, it is the control)
  - [ ] `nest_case2_chain_lengths.foo` (lengths 1, 2, 3 all terminate)
  - [ ] `nest_case2_double_link.foo` (§5 point 2 — PREDICTION; if the
        implementation disagrees, amend FOOP-55.md §5 to match and record why)
  - [ ] `nest_case2_mixed.foo` (§5 point 3 — also a prediction)
  - [ ] `nest_chain_that_hits.foo` (the untested path: a chain that FINDS and
        carries a value back, rather than missing)
- [ ] **Capture the case-2 baseline BEFORE touching `constanic_clone_at`** —
      run `nest_case2_search_chain.foo` and `nest_case2_chain_lengths.foo` on
      the unmodified tree and record their OUTPUT in this plan. They are the
      control for the whole change; a pre-change record is what makes
      "byte-identical" checkable rather than assertable.
- [ ] Run all tests — old and new — and make sure they all pass correctly.
- [ ] Review and promote `output` → `checked` for the `foop/55/SFF/` cases
  - [ ] Confirm the rest of the suite is green — no foreign-FOOP baseline diverges
  - [ ] Confirm no case has a `verified/` twin (if one does: STOP, ask the human)
  - [ ] Re-read FOOP-55.md §5 before judging any OUTPUT
  - [ ] Review `foop/55/SFF/single_mark_unchanged` — every OUTPUT statement justified
  - [ ] Review `foop/55/SFF/double_mark_defers` — every OUTPUT statement justified
  - [ ] Review `foop/55/SFF/budget_is_per_tree` — every OUTPUT statement justified
  - [ ] Review `foop/55/SFF/budget_is_per_use_site` — every OUTPUT statement justified
  - [ ] Review `foop/55/SFF/nested_in_expression` — every OUTPUT statement justified
  - [ ] Review `foop/55/SFF/deferred_avoids_premature_nk` — every OUTPUT statement justified
  - [ ] Review `foop/55/SFF/separator_forms_agree` — every OUTPUT statement justified
  - [ ] Write the justification summary into this plan or the commit message
  - [ ] Report ALL accumulated doubts to the human in ONE statement — or record
        "no doubts"
  - [ ] Run `einmo promote output to checked foolish-ubca/einmo_suite`
  - [ ] Re-run `cargo test -p foolish-ubca --lib -- einmo_gate_checked` — must exit 0

## Phase 3C2 — §6: the brane view and unanchored forward search `~name`

Read FOOP-55.md §6 first. This is what unblocks `'ite`: with `~`, the branch
table is selected by an **anchored** search over a view, so there is no carried
position to survive a coordination — no `&#1`, and no mark-depth puzzle.

- [ ] Read FOOP-55.md §6 in full
- [ ] Read `foolish-parser/src/parser.rs` (the bare `Token::Question` arm, and
      the `Token::Tilde` arms) and `fir_kinds.rs`
      `ib_search_with_engine` / `ab_search_with_engine`
- [ ] **Tests FIRST** (unit, `fir_kinds.rs`) — note these must compose with
      `system.foo` via `compose_program_with_system`, or step through the real
      evaluator; the module's own `step_to_settled` helper caps at 50 steps and
      will not settle these:
  - [ ] `~a` finds the EARLIEST match: `{a=1; b=2; a=4; a=5; r = ~a;}` → 1
  - [ ] `?a` (control, must keep passing) finds the NEAREST PRECEDING: → 5
  - [ ] the window stops before the searching statement: `{a=1; r = ~a; a=99;}`
        → 1, with no self-match and `a=99` invisible
  - [ ] an unanchored forward MISS settles ECONSTANIC, never NK
  - [ ] a view is constanic while its source brane is not (FOOP-55.md §6
        property 3 — the load-bearing one)
- [ ] Implement the brane view: contiguous `[start, end]` over a source brane,
      **same parent**, read-only. Index `i` in the view IS index `i` in the
      source — no translation.
  - [ ] **Never enqueue a view** — add an explicit check. It has no evaluation
        of its own; its statements are stepped by the source brane's queue, and
        enqueueing would step them twice through two owners.
  - [ ] **`get_nyes()` is an ACTIVE SCAN** of direct children via the existing
        `_decide_nyes_due_to_children`, never stored state — so it cannot go
        stale as the window settles. Do NOT write a second classification rule.
        (`Nyes` is not `Ord`; "lowest" is that function's rule, not `.min()`.)
  - [ ] **No `set_nyes`** on a view — enforce in the type, not by convention.
- [ ] Parser: accept bare `Token::Tilde` in primary position, mirroring the bare
      `Token::Question` arm
- [ ] Update the `RegexpSearch` doc comment in `foolish-parser/src/ast.rs`,
      which currently states the unanchored forward form does not exist and
      that `RegexpForward` always carries an anchor
- [ ] Rewrite `ib_search_with_engine` / `ab_search_with_engine` to anchor on a
      view honouring `self.forward`, replacing the hardcoded
      `BraneNavigator::new(&brane, false)` + inline `set_range`
- [ ] Run all tests — old and new — and make sure they all pass correctly.
- [ ] REPORT baseline changes to the human: `~` is NEW syntax, so no existing
      input can use it; any existing baseline that moves is a regression in the
      shared search path, not an expected consequence.

## Phase 3E — §8: `@`, `#` over an expression, and the continuation checks

Read FOOP-55.md §8 in full first. §8 subsumes `'ite` — a two-row table keyed on
`'True`/`'False` IS an if-then-else — so this phase may make Phase 3D
unnecessary; decide at its end rather than assuming either way.

**Baseline facts, verified 2026-08-11 (do not re-derive):**

- `@` does **not** exist and is **silently ignored**: `tbl~key=(77)@` and
  `tbl~key=(77)` both evaluate to `77`. This is the dangerous failure mode — a
  program written to §8 today runs and gives a plausible wrong answer.
- `#` takes only a **literal**: `tbl#(1+1)`, `tbl#n`, `tbl# (1+1)` are parse
  errors; `tbl#1+1` parses as `(tbl#1)+1`.
- **Value already chases through a reference**: with `hello_world=10; b=?hello_world`,
  `tbl~=(b)` matches `q=10`. §8 must PRESERVE this, not build it.

### 3E.1 — the continuation requirement, enforced at construction

- [x] **Tests FIRST**, one per operator — a rejected non-search anchor.
      `continuation_on_a_non_search_anchor_is_nk` covers `&#`, `&?`, `&~`,
      `&?=`, `&~=`; `continuation_on_a_search_anchor_resolves` is the control.
      (2026-08-11 11:34)
- [x] **NO IMPLEMENTATION NEEDED — the behaviour is already correct.** All five
      malformed forms already settle NK on a non-search anchor, exactly as §8
      specifies. 3E.1 is therefore a VERIFICATION that pins the rule so a future
      change cannot silently make a malformed continuation resolve.

      Two syntax facts found while writing the tests:
      - `&^` / `&$` cannot be written malformed — FOOP-75's attached-search
        sugar claims them first (`r = X&^` parses as `r =^ X`).
      - There is **no bare `&=`** — the value continuations are `&?=` and `&~=`
        ("expected search operator after &").
      (2026-08-11 11:34)
- [x] **DECIDED (2026-08-11): a malformed continuation becomes a TRUE NK**, not
      a compile error. Checking at construction means the FIR is BUILT AS an NK,
      not that the build fails — an unanswerable question yields NK exactly as
      the rest of Foolish does. Recorded in FOOP-55.md §8.
      (2026-08-11 10:32)
- [ ] Run all tests — old and new — and make sure they all pass correctly.

### 3E.2 — `candidates_exhausted()` on every search

- [x] **Tests FIRST** — four, all passing:
      `candidates_exhausted_true_when_the_scan_ran_and_matched_nothing`,
      `..._false_when_the_anchor_was_nk`,
      `..._false_when_the_search_found_a_match`,
      `..._does_not_cascade`.
      (2026-08-12 03:58)
- [x] Implemented as an `exhausted: Cell<bool>` on `SearchFir`, set at the two
      `ScanOutcome::Miss` sites and read through the trait method. The
      information was already computed and thrown away: the value-search step
      mapped `ScanOutcome::NkStop` and an anchored `Miss` to the same bare
      `Nyes::Nk`, so the scan's own knowledge was lost exactly where it was
      known. `NkStop` deliberately does NOT set the flag — that is the
      distinction `@` reads back.
      (2026-08-12 03:58)
- [x] **Universal on every FIR**, not a hook for `@` (rule zero) — a `false`
      default on the `Fir` trait for kinds that never scan.
      (2026-08-12 03:58)
- [x] **DOES NOT CASCADE** — pinned by `candidates_exhausted_does_not_cascade`:
      `(tbl?q) &#1` found `r=3`, so the outer continuation's own status is
      "found" and it inherits nothing from the search it continues.
      (2026-08-12 03:58)
- [ ] Run all tests — old and new — and make sure they all pass correctly.

### 3E.3 — `@`

- [x] **Tests FIRST** — five, all passing: `at_yields_the_found_statements_index`
      (1, not 77 — the position, not the value), `at_and_no_at_now_differ`,
      `at_yields_minus_one_when_candidates_are_exhausted`,
      `at_propagates_nk_when_the_anchor_was_nk`,
      `at_on_a_non_search_anchor_is_nk`.
      (2026-08-12 04:12)
- [x] **`@` and no-`@` now DIFFER** — pinned. Before this, `@` fell into the
      lexer's unknown-character fallback and was SILENTLY IGNORED, so
      `tbl~key=(77)@` and `tbl~key=(77)` both gave 77 and a program written to
      §8 would have run with a plausible wrong answer.
      (2026-08-12 04:12)
- [x] Implemented: `Token::At` (lexer + Display), `Astn::SearchPosition`,
      `SearchPositionFir` with `FirKind::SearchPosition`, and its evaluator and
      constanic-clone arms. `@` reads the position from the anchor's
      `ubc_children[1]` FoolRefFir — the position FOOP-23's two-child invariant
      has carried all along, which nothing in the language read until now.
      Single dependency (the anchor); settles WOCONSTANIC.
      (2026-08-12 04:12)
- [ ] Run all tests — old and new — and make sure they all pass correctly.

### 3E.4 — `#` over an expression

- [x] **Tests FIRST** — four, all passing:
      `hash_accepts_a_parenthesized_expression` (`#(1+1)` → c=3),
      `hash_accepts_a_named_operand` (`#(n)`),
      `hash_accepts_a_search_expression_operand`
      (`tbl#(tbl~key=(77)@+1)` → the row beside the matched key),
      and `hash_literal_then_plus_keeps_its_old_meaning` — `tbl#1+1` still
      parses as `(tbl#1)+1`, which is existing behaviour and unchanged.
      (2026-08-12 04:31)
- [x] Implemented as `Astn::ComputedSeek` + an `index_expr: Option<FirRef>` on
      `IndexFir`. The operand is enqueued alongside the anchor — a genuine
      SECOND DEPENDENCY, not a new evaluation phase — and `effective_offset()`
      reads it at all three navigation sites, falling back to the literal
      `offset` when absent.
      (2026-08-12 04:31)
- [x] **The full idiom verified end to end** with an `else_value`-first table:
      `hit_true=10`, `hit_false=20`, `miss=999`. One expression, no branch —
      the same `@+1` that steps a hit to its adjacent `value=` row steps a miss
      from -1 to index 0.
      (2026-08-12 04:31)
- [ ] Run all tests — old and new — and make sure they all pass correctly.

### 3E.5 — fibonacci, then Euler 1

**Development loop, per feature** (3E.1–3E.4 each follow it):
1. unit tests first, in `fir_kinds.rs` / `system_foo.rs`
2. implement
3. verify **that one einmo case alone** — `cargo test -p foolish-ubca --lib
   -- --test-threads=1 einmo_gate_output`, then read the single file's OUTPUT.
   Do not run the whole suite to check one feature.
4. only then move to the next feature

**Fibonacci is the stepping stone; Euler 1 is the target.** `exercises/fibonacci/1.foo`
is the smaller integration test — recursion, pattern matching, and the doubled
mark in one program, with no `'mod`/`'or`/`'cmod` in the way. Euler 1 then gets
**rewritten to the same idiom** rather than kept in its old `'ite` form.

- [ ] `exercises/fibonacci/1.foo` settles — `fib_5` is a number, not a parse
      error. (Carried in the suite red today: `expected integer, found LParen at
      line 15, column 38`, which is exactly the `#`-over-expression 3E.4 adds.)
- [ ] Review its OUTPUT statement by statement, then promote through the
      Promotion Review Gate
- [ ] **REWRITE Euler 1 to the pattern-matching idiom** — `'match` + a keyed
      table, replacing the `'ite` formulation. `'ite` was never implemented in
      pure Foolish and §8 subsumes it, so the rewrite removes the dependency
      rather than working around it.
- [ ] `exercises/project_euler/1.foo` settles to **233168**
- [ ] `foop/55/euler_small.foo` settles to **23** (3+5+6+9 — hand-checkable)
- [ ] einmo `foop/55/pattern_match.foo`: the `else_value`-first table, a hit
      selecting its adjacent `value=` row and a miss falling through to index 0
      by the same `@+1`
- [ ] einmo `foop/55/continuation_value_vs_position.foo`: value chases through
      (`{b = ?hello_world; {a = b&=10}}` matches when hello_world is 10) while
      position does not (`b@+1` is **b's own** position plus one)
- [ ] Review every OUTPUT statement, then promote through the Promotion Review
      Gate
- [ ] Run all tests — old and new — and make sure they all pass correctly.
- [x] **DECIDE**: with §8 landed, are §7 (`ExtremumFir`) and Phase 3D (`'ite`)
      still needed? Both were routes to the same end. Delete what §8 made
      redundant rather than carrying it.
      (2026-08-24) §8 (`@`) is confirmed implemented and wired
      (`SearchPositionFir::combine`, `fir_kinds.rs:751-794`; dispatched from
      `evaluator.rs:373` and `fir_kinds.rs:376`). §7 (`ExtremumFir`) is
      superseded — remove it (Phase 3D2 below). Phase 3D (`'ite`) is a
      separate mechanism (branch selection via SFF-marked table lookup, not
      order-statistic selection) and is NOT superseded by §8; it proceeds
      below unchanged.

## Phase 3D — §5: rewrite `'ite` and remove the directive

- [ ] Rewrite `'ite` using §6's `~` over the branch table — an ANCHORED search
      over a view, with no `&#1` and therefore no carried position. Apply to
      `input/foop/55/euler_small.foo` and the exercise. If a doubled mark is
      still needed anywhere, record WHY in FOOP-55.md §5 rather than tuning
      depths by trial.
- [ ] `euler_small.foo` settles to **23** (hand-checkable: 3+5+6+9)
- [ ] Remove `@einmo set iteration depth to 40000` from
      `input/exercises/project_euler/1.foo`. If the exercise still needs it,
      §5 is not finished.
- [ ] **RECONSIDER `IteFir` and `OrFir`** (FOOP-55.md §5, Appendix A.A): both
      Rust kinds exist only because the pure-Foolish definitions could not
      resolve. If §5 frees them, DELETE the custom kinds — a custom
      short-circuiting kind obliges every collaborator (searches, sequencer,
      recoordination) to handle a FIR with un-stepped members, which the generic
      mechanism does not. Confirm live before deleting; if a kind is still
      needed, record why in FOOP-55.md §5.
- [ ] Run all tests — old and new — and make sure they all pass correctly.

## Phase 3D2 — Remove `ExtremumFir` (§7), superseded by §8

§7's order-statistic selection (`'min_int_val`/`'max_int_val`) was a route to
values Euler 1 needs; §8 (`@`/pattern matching) is confirmed implemented and
supersedes it (Phase 3E's DECIDE box, above, 2026-08-24). `exercises/
fibonacci/1.foo` and `exercises/project_euler/1.foo` are this FOOP's target
features going forward — remove code that is no longer on the path to
either.

- [x] Establish relevant tests for this phase. Use
      [these instructions](../../README.md#running-specific-tests) to run
      unit tests: `foolish-ubca::system_foo::extremum_selects_min_and_max`,
      `foolish-ubca::system_foo::extremum_result_is_independent`,
      `foolish-ubca::system_foo::extremum_skips_non_integer_members`,
      `foolish-ubca::system_foo::extremum_with_no_integer_candidates_settles_nk`
      (all four are deleted by this phase, not fixed — confirm each is gone
      after, not passing).
      (2026-08-24 07:55)
- [x] Remove `ExtremumFir` and its impl from `foolish-ubca/src/system_foo.rs`
      (struct, `impl ExtremumFir`, `impl Fir for ExtremumFir`, the
      `ALIASES` registration loop, and the four `extremum_*` unit tests).
      (2026-08-24 07:55)
- [x] Remove the `FirKind::Extremum` dispatch arm from
      `foolish-ubca/src/evaluator.rs:381-395`.
      (2026-08-24 07:55)
- [x] Remove `FirKind::Extremum` from `foolish-ubca/src/fir_trait.rs:68` and
      `as_extremum_config`/its default impl at `fir_trait.rs:384-386`.
      (2026-08-24 07:55)
- [x] Grep for any remaining `Extremum`/`extremum` reference (including
      `'min_int_val`/`'max_int_val` in einmo `.foo` inputs) and remove or
      update each; none should remain.
      (2026-08-24 07:55) Confirmed zero remaining references in
      `foolish-ubca/src/*.rs`; also removed the two declarations in
      `system/system.foo` (not originally listed above — found during the
      sweep). No einmo `.foo` input referenced either alias.
- [x] Update FOOP-55.md §7 to state plainly, at its top, that `ExtremumFir`
      was removed (2026-08-24) as superseded by §8, with a pointer to this
      phase — do not delete §7's text outright; it documents a design that
      was built, worked, and was later superseded, which is worth keeping
      as a record (matches how §10 documents the `BraneConcatOp` rename
      rather than silently rewriting history).
      (2026-08-24 07:55)
- [x] Run all tests — old and new — and make sure they all pass correctly.
      (Confirms 3 of the 5 currently-broken tests — `extremum_result_is_
      independent`, `extremum_selects_min_and_max`,
      `extremum_skips_non_integer_members` — are gone rather than fixed.)
      (2026-08-24 07:55) `cargo check -p foolish-ubca` and
      `cargo clippy -p foolish-ubca --all-targets -- -D warnings` both clean.
      `cargo test --workspace`: all four `extremum_*` tests confirmed gone
      (374 tests remain, filtered-count dropped from 378; explicit
      `-- extremum` filter now matches 0). Remaining failures are
      `einmo_gate_checked`/`einmo_gate_verified` only, both pre-existing and
      listing exactly the expected in-scope divergences (`exercises/
      fibonacci/1.foo`, `exercises/project_euler/1.foo`, plus the parallel
      session's own in-flight §10/§9.2 concat work on this shared branch) —
      not a regression introduced by this phase. `cargo fmt --check` shows
      pre-existing drift in `compiler.rs`/`fir_kinds.rs` only, from the
      parallel session's concurrent commits — no file touched by this phase
      has any fmt drift.

## Phase 3H — §11: two-phase child-readiness gates

**No longer deferred (human direction, 2026-08-24). Read FOOP-55.md §11 in
full — it was rewritten same day to add the two-phase (`foolish_children`
then `ubc_children`) queue model and the `Option<Nyes>`-returning handler
contract; two earlier drafts (four-handler/microstate, and a flat
one-phase `constanic_enough`) are superseded and moved to Appendix
B/B.2.** `Step 1` below is ALREADY LANDED and verified byte-identical; do
not redo it.

- [ ] Establish relevant tests for this phase — see
      [these instructions](../../README.md#running-specific-tests). Run the
      full SF/SFF corpus (§5) and the concatenation-ergonomics corpus (§9)
      frequently while implementing this; both depend on the drain path
      this section changes.
- [x] **Step 0 (prepended, human direction 2026-08-25) — migrate `IndexFir`,
      then `SearchFir`, onto the same `on_foolish_op_ready`/
      `is_foolish_child_constanic_enough` mechanism, AHEAD of the remaining
      Step 5 kinds below.** `IndexFir` (`#N`/`^`/`$`) and `SearchFir` are
      what `$` actually compiles to and have NOT been migrated at all yet —
      still the old inline `fir_op_step`, unlike `SearchPositionFir`/`@`
      (done) and `OperatorFir` (done). `IndexFir`'s anchored path
      (`fir_kinds.rs`, `impl Fir for IndexFir`) has the SAME bug class `@`
      had: `if !resolved.borrow().core().get_nyes().is_constanic() { return
      Ok(()); }` waits for the anchor's fully-resolved VALUE to be
      constanic, rather than checking whether the anchor has FOUND
      something — confirmed live 2026-08-25 (traced, not yet fixed).
      `IndexFir` is currently ~240 lines with four branches
      (anchored/unanchored × contexted/not); `SearchFir` ~200 lines with
      its own staged sequence (`ib_search_with_engine`/
      `ab_search_with_engine`/`contexted_search_from_anchor`). **Human
      assessment (2026-08-25): this size is a smell in itself — the new
      predicate/handler shape should likely SHRINK this code, not just
      wrap it as-is; do not treat 240/200 lines as a fixed target to
      preserve line-for-line.** Order: `IndexFir` first, `SearchFir`
      second, both before the remaining `ComparisonFir`/`ModuloFir`/`OrFir`
      work already queued below. Same `_deprecating_op_step` discipline —
      tests first, one piece moved at a time, full suite green after each.
      **DONE (2026-08-25):** Both migrated. `IndexFir`: renamed
      `fir_op_step` to `_deprecating_op_step`; the Braning arm's contexted-
      anchored and plain-anchored lookup logic both moved verbatim into
      `on_foolish_op_ready`, converting `set_nyes`+early-`return Ok(())`
      into `Some(nyes)` and the bare "still waiting" `return Ok(())` into
      `None`; the FOOP-75 §7 "X is not a brane" diagnostic and the
      computed-index (`#(expr)`) NK path preserved verbatim. `SearchFir`:
      same rename; its TWO Braning-arm flavors (plain name/anchored search,
      and the separately-dispatched value-search path) each got their own
      handler (`on_foolish_op_ready` for the plain path,
      `on_value_search_op_ready` for `value_search_step`'s Braning arm —
      `is_value_search` still dispatches at the top of `fir_op_step` before
      reaching the NYES match, unchanged). Both `SearchFir` handlers, and
      `IndexFir`'s, additionally received the `is_found()`-based fix (the
      same class as `@`'s fix below) on every branch that previously
      waited on `resolved.is_constanic()`/`anchor.is_constanic()` alone: a
      search anchor can be permanently WOCONSTANIC (found a statement whose
      value never resolves) while still holding a real, final resolved
      position/value, so `is_found()` is now checked first, falling back to
      `is_constanic()` only when not found. Note: the code was NOT
      meaningfully shrunk by this migration (contrary to the human's
      pre-migration guess above) — the Braning-arm logic was already a
      single flat sequence per branch with no cross-arm duplication to
      collapse; the win is confined to the intended one (readiness
      decision vs. commit now separated, matching every other migrated
      kind), not a line-count reduction. Full suite: 377 passed / 0 failed
      throughout, verified after each of the four handler edits
      individually. `einmo_gate_checked`'s failure set confirmed
      byte-identical before/after via `git stash` comparison (all
      pre-existing, already-tracked pending items — see Step 5's own
      note below for the list).
- [x] **Step 1 — per-child predicates, standalone.** Added
      `is_foolish_child_constanic_enough`/`is_ubc_child_constanic_enough` to
      the `Fir` trait (`fir_trait.rs`), alongside `is_constanic_branelike`/
      `is_search_kind` — same overridable-default pattern. Default body:
      `child.is_constanic()`. Changed `step_inner`'s dequeue check to call
      `this.is_foolish_child_constanic_enough(&front_rc)` instead of the
      bare `front_rc.get_nyes().is_constanic()`. Verified: full suite
      374 passed / 2 pre-existing failures (same 2 on clean HEAD via
      `git stash`), byte-identical. New regression test:
      `fir_kinds::tests::constanic_enough_default_matches_is_constanic`.
      (2026-08-24)
- [x] **Step 2 — whole-set readiness checks.** Added
      `are_foolish_children_ready_for_op`/`are_ubc_children_ready_for_op` to
      the `Fir` trait — default: `true` iff every member of the
      corresponding store passes its per-child predicate. New, currently-
      unused (no caller yet) trait methods. New tests:
      `are_foolish_children_ready_for_op_default_waits_for_all`,
      `are_ubc_children_ready_for_op_vacuously_true_when_empty`. Full suite:
      377 passed / 2 pre-existing failures, unchanged (dead code so far).
      (2026-08-24)
- [x] **Step 3 — the two handlers, unused.** Added
      `on_foolish_op_ready`/`on_ubc_op_ready` to the `Fir` trait, each
      `fn(&self, scope: &Scope) -> Option<Nyes>`, default `None`. Still not
      called from anywhere. New test: `on_op_ready_handlers_default_to_none`.
      (2026-08-24) Note: skipped the plan's original "trivial override
      returning Some(Nyes::Constant)" sub-case — the default-only test
      (verifying None) covers this step's actual surface; a real
      Some-returning override is exercised for real in Step 4.
- [x] **Step 4 — migrate `ConcatenationFir` onto the new mechanism, via
      `_deprecating_op_step`.** Followed FOOP-55.md §11's migration path;
      all sub-tasks below complete, full suite unchanged throughout.
      (2026-08-24)
  - [x] Rename `ConcatenationFir::fir_op_step`'s current body to
        `_deprecating_op_step`, called unconditionally from the real
        `fir_op_step`. Byte-identical; pure rename. Full suite:
        377 passed / 2 pre-existing failures, unchanged. (2026-08-24)
  - [x] Move the `foolish_children`-phase logic (the `all_brane_like`/
        `type_errors` scan) into `on_foolish_op_ready`, moved verbatim.
        **Note, corrected from the plan's original wording**: NOT wired
        through the generic `are_foolish_children_ready_for_op()`/per-child
        predicate — `_deprecating_op_step`'s `Braning` arm calls
        `on_foolish_op_ready` directly, unconditionally, exactly where the
        scan used to run (the scan's own pass-fail IS the readiness
        decision here; routing it through the separate generic gate first
        would be a second, entangled behavior change, deferred to Step 6
        where `ConcatenationFir` gets its real `is_foolish_child_constanic_enough`
        override). `Some(Nk)`/`Some(Woconstanic)` on failure, `None` on a
        clean pass (falls through to the still-unmoved ubc-phase logic).
        Full suite: 377 passed / 2 pre-existing failures, unchanged.
        (2026-08-24)
  - [x] Move the `ubc_children`-phase logic (only the settle-from-drained-
        helpers half — `populate_concat_helpers`/task-push stays, it is
        ubc-phase task-POPULATION, analogous to the PREMBRIONIC/EMBRYONIC
        arm, not an "on ready" decision) into `on_ubc_op_ready`. Full
        suite: 377 passed / 2 pre-existing failures, unchanged. (2026-08-24)
  - [x] `_deprecating_op_step` had nothing left but orchestration (the
        PREMBRIONIC/EMBRYONIC arm and the two handler-call sites) — no
        further business logic to move. Renamed back into `fir_op_step`
        directly (the indirection served no further purpose); the
        `_deprecating_op_step` inherent method is gone. Full suite:
        377 passed / 2 pre-existing failures, unchanged. (2026-08-24)
  - [x] Ran all tests after each sub-step above (this sub-step's own gate).
        (2026-08-24)
- [x] **Step 5 — `OperatorFir`-style override**, same `_deprecating_op_step`
      migration path. `OperatorFir` DONE; `ComparisonFir`/`ModuloFir`/`OrFir`
      still outstanding (tracked below as their own sub-item, since they
      live in `system_foo.rs` and were not reached this session).
      (2026-08-25)
  - [x] `OperatorFir` migrated via `_deprecating_op_step`: `combine`'s body
        moved into `on_foolish_op_ready` (every `set_nyes`+early-return
        converted to `Some(nyes)`), `_deprecating_op_step` collapsed back
        into `fir_op_step` directly once empty (same pattern as
        `ConcatenationFir`). The any-NK-poison short-circuit's ordering
        (checked first, separately, before the values/constantew wait) is
        preserved intact. New shared helper `push_nk_result` (was three
        near-identical inline NK-construction blocks). (2026-08-25)
  - [x] **Real behavior change, human-directed**: the unknown-operator arm
        (previously a hard `UbcError::Eval`, unrepresentable in
        `on_foolish_op_ready`'s `Option<Nyes>` signature) now settles NK
        with an explanation instead. Unreachable in practice (parser only
        constructs known operators); parser-side validation to make it
        TRULY unreachable is tracked in Phase 4B. New regression test:
        `unknown_operator_settles_nk_not_hard_error`. (2026-08-25)
  - [x] **`are_foolish_children_ready_for_op`'s DEFAULT corrected** (human,
        2026-08-25): checks `constantew` on every `foolish_children`
        member DIRECTLY (`c.get_nyes().is_constantew()`), NOT by
        aggregating `is_foolish_child_constanic_enough` (that predicate
        stays independently defaulted to `is_constanic()`, gating the
        QUEUE-DRAIN dequeue point in `step_inner` — a different question
        from whether the kind's OWN operation may be attempted). Wired
        into `OperatorFir::fir_op_step`'s `Braning` arm per the shape
        `if any_nk { ... } else if are_foolish_children_ready_for_op() {
        on_foolish_op_ready() } else { _decide_nyes_due_to_children(...) }`
        — the NK-poison check still runs first, unconditionally, ahead of
        the readiness gate (an NK poisons regardless of what else is
        waiting). New test:
        `are_foolish_children_ready_for_op_default_waits_for_all` (updated
        semantics), full suite re-verified green after the change.
        (2026-08-25)
  - [x] **Real bug found and fixed via this instrumentation, NOT
        `OperatorFir`-specific**: `key@+1` (`SearchPositionFir`/`@` as an
        `OperatorFir` operand) broke under the corrected default, exposing
        that `@`'s own top-level NYES was hardcoded `Woconstanic` even
        once it had computed a final, permanent integer — a pre-existing
        mismatch between what `@` computed and what state it reported,
        invisible before because the OLD arithmetic code reached through
        `.value()` rather than checking an operand's own NYES directly.
        Root-caused and fixed properly (human-directed), not patched
        around — see the `SearchFir::found_context`/`Fir::is_found` and
        `SearchPositionFir` sub-items below. New regression test:
        `hash_accepts_a_search_expression_operand` (pre-existing test,
        confirmed still green after the fix, not new — the fix restores
        it rather than adding new coverage for this specific case).
  - [x] **New: `SearchFir::found_context` / `Fir::is_found()` /
        `Fir::found_context_index()`** (human, 2026-08-25). A search
        captures its found statement's `(home brane, statement index)` the
        MOMENT `handle_found` discovers it — before `clone_stmt_result`
        constanic-clones the body and reparents the clone away from the
        original. `is_found()` (default `false`, `SearchFir` overrides via
        `found_context.is_some()`) answers "did this search find something"
        independent of whether the found statement's own VALUE ever
        resolves (a WOCONSTANIC/ECONSTANIC find is still a genuine find —
        D9/§5.5). This is a DIRECT, named alternative to the existing
        positional convention (`ubc_children.get(1)`'s `FoolRefFir`, still
        used unchanged by `contexted_search_from_anchor` — NOT migrated
        onto the new mechanism this session, left alone per human
        direction "using those features, updating just `@`"). TODO
        (Phase 4B): the stored `FirRef` to the home brane is a GC hazard —
        needs releasing once the whole statement is constanic; see the
        code comment at the field definition and Phase 4B's own entry.
  - [x] **`SearchPositionFir` (`@`) fully migrated** via the same
        `on_foolish_op_ready`/`is_foolish_child_constanic_enough` pattern:
        - `is_foolish_child_constanic_enough` overridden — the anchor is
          "done enough" the moment `anchor.is_found()`, OR (fallback) once
          it is plain `is_constanic()`. `is_constanic()` alone was already
          `true` for a WOCONSTANIC anchor, so this override's real purpose
          is not about the DEQUEUE gate at all — it is about letting
          `on_foolish_op_ready` distinguish "found, value pending" from
          "genuinely still searching" the moment the anchor is dequeued,
          without waiting for a fully-settled anchor when a found-but-
          unresolved one already carries a real answer.
        - `on_foolish_op_ready` replaces `combine`: reads `is_found()` /
          `found_context_index()` directly instead of reaching into
          `ubc_children[1]`'s `FoolRefFir`; `settle_nk`/`settle_index`
          converted from `set_nyes`-calling void methods to
          `Nyes`-returning ones.
        - **The actual fix for `key@+1`**: `settle_index` now reports
          `Nyes::Independent` (matching the genuinely final, permanent
          integer it just computed) instead of the old hardcoded
          `Nyes::Woconstanic` (which asserted an ongoing wait that did not
          exist once the index was known — the original `// one
          dependency, and it is constanic: WOCONSTANIC` comment's
          reasoning conflated "the anchor being constanic" with "`@` itself
          still depends on something further," which it does not once
          `settle_index` runs). This is what lets `OperatorFir`'s
          corrected `constantew` check see `@` as ready with no
          `@`-specific special-casing needed on the consumer side.
        Full suite: 377 passed, 0 unit-test failures (einmo gates hit a
        PRE-EXISTING catastrophe-crumb race under concurrent gate runs,
        unrelated to this work — confirmed via repeated isolated runs of
        `einmo_gate_checked` alone showing the SAME, already-known,
        already-documented divergence set: FOOP-65 INPUT changes plus
        `sff_nested`/`concat_sf_f_more` OUTPUT, all pending promotions from
        before this session, not caused by any change in this step).
        (2026-08-25)
  - [x] `ComparisonFir`/`ModuloFir`/`OrFir` (`system_foo.rs`) — same
        `_deprecating_op_step` migration + `constantew` override, not yet
        started. **DONE (2026-08-25), via sub-agent (all three kinds'
        `fir_op_step` bodies were already small — Prembrionic|Embryonic
        enqueue arm, Braning arm calling `combine`, catch-all — so no
        rename/redirect step was needed; `combine`'s body moved directly
        into `on_foolish_op_ready` in place).** `settle_nk` no longer calls
        `set_nyes` itself (only pushes the NK ubc_child + sets
        `alarm_reason`); every call site became
        `self.settle_nk(reason, scope); return Some(Nyes::Nk);`. The two
        "system.foo must define 'True/'False" invariant-violation paths in
        `ComparisonFir`/`OrFir`, previously
        `Err(UbcError::InternalConsistency(...))`, became `panic!(...)`
        with identical message text — forced by `on_foolish_op_ready`'s
        `Option<Nyes>` signature, which cannot propagate a typed `Err`;
        matches the existing `.expect(...)`-based precedent used elsewhere
        in this migration for the same "must be Some" invariant, and this
        path is unreachable in practice (system.foo always defines both).
        Full suite: 377 passed / 0 failed, verified after each of the
        three kinds individually.
  - [x] Run all tests — old and new — and make sure they all pass
        correctly (final gate for this whole step, once the three
        remaining kinds are done). **DONE (2026-08-25):** full suite
        377 passed / 0 failed with all of Step 0 and Step 5 landed
        together; `einmo_gate_checked` fails only on the same pre-existing,
        already-tracked set confirmed unchanged by this work:
        `exercises/fibonacci/1.foo`, `exercises/project_euler/1.foo`,
        `foop/55/cmod.foo`, `foop/55/euler_small.foo`, `foop/55/ite.foo`
        (all "missing entirely from checked/, present in output/" — not
        yet promoted from earlier session work), and
        `misc/concat_sf_f_more.foo`/`misc/concat_sf_f_more_strange.foo`
        (the already-documented slow/non-terminating investigation,
        deferred to revisit after Phase 3I). Confirmed via `git stash`
        that this exact failure list is identical on the pre-Step-0 commit
        — zero new divergences from Steps 0/5's logic changes, including
        the `is_found()`-based behavior changes.
- [ ] **Step 6 — `terminates_econstanic()` and `ConcatenationFir`'s D9
      override.** Add `terminates_econstanic()` to `SearchFir` per
      FOOP-55.md §11's algorithm. Override `ConcatenationFir`'s
      `is_foolish_child_constanic_enough` to
      `child.is_constanic() && !(child is a Search && child.terminates_econstanic())`.
      **Depends on Phase 3I (the `has_ancestral_sfm` leak fix) already
      being landed** — without it, D9's own example
      (`{a = {1,2}, b=<<#-2>>, c= a b}`) cannot reach the fixed result even
      with this override in place, because the recoordinated clone never
      gets the chance to run at all. Tests first:
      `d9_recoordinated_index_...` (renamed once fixed, see Phase 3I) must
      assert `c` settles CONSTANT to `{1,2,1,2}`.
- [ ] Run all tests — old and new — and make sure they all pass correctly.
      Pay particular attention to the SFF strip-budget corpus (§5), D10/§5.5/
      §5.6, and the concatenation-ergonomics corpus (§9) — this changes a
      drain condition every Braning-capable kind's stepping depends on.

## Phase 3I — Fix the `constanic_clone` SF/SFF-mark ambient-flag defect (D9's real cause)

**Independent of Phase 3H/§11's Steps 1-5 (both done) — land this before
Phase 3H's Step 6** (`terminates_econstanic()`/`ConcatenationFir`'s D9
override), since Step 6 cannot produce D9's expected result without this
fix in place first, per Phase 3H's Step 6 note. Read FOOP-55.md's D9
finding, item 3 ("Designed, human, live conversation 2026-08-26" —
supersedes the earlier `step_inner`-reset framing) in full before starting.

**Design summary** (full spec is FOOP-55.md's D9 item 3): rename
`constanic_clone_at`/`constanic_clone_at_budgeted` to an internal
`_inner_constanic_clone(node, stay_budget, disable_nyes_reset)`. Add a new
public `constanic_clone(node)` that every `Fir` kind calls instead of
today's direct `constanic_clone_at` calls, always starting with
`stay_budget=1, disable_nyes_reset=false`. When `_inner_constanic_clone`
meets an SF/SFF mark (one shared budget pool, either kind): `stay_budget≥1`
→ strip it, recurse on its content with `stay_budget-1` and
`disable_nyes_reset` unchanged; `stay_budget==0` → clone the mark itself as
a wrapper with `disable_nyes_reset=true` for that call only. Ordinary
structural recursion (operator operands, brane statements, etc.) passes
`disable_nyes_reset` through unchanged, but each distinct child gets its
own **fresh** `stay_budget`, never inherited/decremented from the parent.

- [ ] Establish relevant tests for this phase. Use
      [these instructions](../../README.md#running-specific-tests) to run
      unit test: `foolish-ubca::fir_kinds::tests::d9_recoordinated_index_currently_stuck_woconstanic`
      (currently pins the BROKEN state — update its assertions once fixed,
      per the doc comment on that test), `cloning_sf_strips_the_mark`, and
      `step_sets_foolish_scope_inside_sf`. Also run the full SF/SFF-related
      corpus (§5's existing tests) frequently — this touches shared clone
      machinery every SFF-marked term depends on.
- [ ] **Tests first.** Before touching `constanic_clone_at`, write new unit
      tests pinning the NEW mechanism's contract, at minimum:
  - [ ] A clone of a node that IS itself SF/SFF-marked strips exactly the
        outer mark and leaves a nested (mark-inside-a-mark) constituent
        still wrapped — i.e. `stay_budget=1` strips one layer only, per any
        single path to a leaf.
  - [ ] A clone of a plain (unmarked) node containing MULTIPLE independent
        SF/SFF-marked descendants (e.g. sibling statements in a brane, or
        an operator's several operands) strips the outer mark on EACH
        independently — confirms budget is per-child-call, not shared/
        threaded across siblings from one parent budget.
  - [ ] A `Fir` kind's `constanic_clone` call reaching a FRESH mark (one
        the ambient ancestor scope had nothing to do with — modeling D9's
        exact shape: a search's own found-body clone, itself SF/SFF-marked
        independently of whatever marked the search) strips that mark and
        resets its NYES, regardless of what any calling context "remembers"
        about a DIFFERENT, unrelated mark higher up. This is the test that
        should fail today and pass once fixed.
  - [ ] Two or three levels of NESTING (a mark directly wrapping another
        mark, wrapping another) to pin exactly one layer strips per
        `constanic_clone` call, confirming the second and deeper marks stay
        intact until their OWN fresh `constanic_clone` call reaches them
        (e.g. via a later search result, as D9's `b` does).
- [ ] Implement: rename to `_inner_constanic_clone`, add `stay_budget`/
      `disable_nyes_reset` params, add the public `constanic_clone` entry
      point, and switch every existing call site (`SearchFir`,
      `OperatorFir`, `ComparisonFir`, `ModuloFir`, `OrFir`,
      `SearchPositionFir`, `IndexFir`, and any other `Fir` kind currently
      calling `ProtoBrane::constanic_clone_at` directly) to call the new
      public `constanic_clone` instead. Confirm `scope.has_ancestral_sfm`
      is no longer read at any of these call sites once done; check whether
      `Scope::has_ancestral_sfm`/`with_ancestral_sfm` has any OTHER caller
      before considering removing the field itself (do not remove it as
      part of this step if anything else still depends on it — a separate,
      reviewed cleanup).
- [ ] Re-run `d9_recoordinated_index_currently_stuck_woconstanic` — it
      should now FAIL (the broken-state assertions no longer hold). Update
      its assertions to match D9's expected result (`c` settles CONSTANT to
      `{1,2,1,2}`) and rename it to drop "currently_stuck_woconstanic" once
      it asserts the fixed behavior.
- [ ] Run all tests — old and new — and make sure they all pass correctly.
      Pay particular attention to the SFF strip-budget corpus (§5) and
      anything D10/§5.5/§5.6 touch — this is shared clone machinery every
      SFF-marked term depends on, a regression here would be silent and
      wide-reaching.
- [ ] Run the full einmo suite (`einmo_gate_checked`) and diff EVERY
      divergence against `checked/` case by case — this change touches the
      shared clone path every SF/SFF-marked construction in the suite goes
      through, so expect multiple pre-existing `checked/` baselines to
      shift. **Do not promote any of them without the human's own review**
      (human, 2026-08-26: "I will have to reapprove a lot of tests... I
      will review to verify the tests" — this promotion pass is the
      human's, not the agent's, once the agent's own line-by-line
      Promotion Review Gate justification is written down for each case).
- [ ] Promote `foop/55/d9_recoordinated_index.foo.einmo` (added 2026-08-26,
      currently `@agent`-marked known-broken) once its OUTPUT genuinely
      shows `c` settling CONSTANT to `{1,2,1,2}` — remove the `@agent` note
      from the input first, per AGENTS.md's embedded-communication
      resolution discipline.

## Phase 3J — UFM, the Unstay Foolishness Mark `<@ … @>`  ← **EARLY: implement before 3G**

**Human direction, 2026-08-27.** Scoping study with full call-graph, git archaeology
and blast radius: `docs/foop/UFM-scoping-study.md` (read §F first — it supersedes
§C/§E wherever those assume the wrapper shape).

**Decisions already made — fixed, do not re-litigate:**

- **Name and syntax:** it is the **Unstay Foolishness Mark**, written `<@ … @>`.
  It remains a MARK to the Foolisher. (`<* … *>` was rejected: `<* 5 >` parses
  *today* as `StayFoolish{UnaryOp{"*"}}` and evaluates to NK.)
- **Implementation is an OPERATOR.** A mark to the Foolisher, an operator to the
  evaluator. It owns its content in `foolish_children`, waits for it to go
  constanic, constanic-clones it stripping **every** SF/SFF mark into
  `ubc_children`, and lets it step again.
- **Removes ALL layers.** `{a=x}` removes one layer of SFF detachment;
  `{a=<@ x @>}` removes all of them, on every path below.
- **UFM does not survive the clone** — it is consumed by producing its result,
  like any other operator.
- **It undoes SFF's COMPILE-TIME detachment**, which the wrapper shape could not.
  The mechanism is already in the code: `transform_for_clone(false)` maps
  `Econstanic → Embryonic`, and SFF's detachment lives entirely in "this search
  was born ECONSTANIC". Re-birthing it EMBRYONIC undoes it, so UFM needs no
  compile-time half and the governing principle stays intact:
  **SF and UFM affect STEPPING; SFF detaches during COMPILATION.**

- [ ] Establish relevant tests for this phase — see
      [these instructions](../../README.md#running-specific-tests). The SF/SFF
      corpus (§5) and `misc/sf_of_sff`, `foop/62/sf_sff_nested_combined`.
- [x] **`OpInstructions` enum replaces `inside_sf_mark: bool`.** *(2026-08-27,
      commit `55caa37d`)* Human direction: a bool can name only two of the three
      real conditions. `constanic_clone`'s last parameter is now
      `OpInstructions::{Normal, InsideSfm, InsideUfm}`, and each variant chooses
      the starting budget via `OpInstructions::starting_budget()` —
      `fresh()` / `fresh().spend().1` / `unlimited()` respectively. Behavior for
      the two pre-existing conditions is unchanged (einmo divergence set is
      byte-identical before and after). Four new tests pin all three variants,
      including that the unlimited budget is consumed by neither depth nor
      breadth. `StripBudget::unlimited()` lost its `#[expect(dead_code)]` — this
      is the caller it was waiting for, so the later "Wire `unlimited()`" item
      below is now only about the UFM *operator* naming the variant.
- [x] **Blocker: finish the budget refactor into `system_foo.rs`.**
      *(2026-08-27, commit `55caa37d`)* Fixed as described below: the three
      kind-arms now take and pass `stay_budget`, calling
      `clone_children_budgeted` like the `OperatorFir` arm; the
      `clone_children_for_constanic_clone` wrapper had zero callers left and is
      deleted. Worth recording precisely what these three are, since the naming
      misleads: **they are not clone entry points.** They are dispatch arms of
      `_inner_constanic_clone`'s `match kind`, living in `system_foo.rs` only
      because their types do — so yes, they genuinely do need to clone (`'lt` is
      cloned out of `system.foo` and recoordinated into the referencing brane,
      and its operands must cross as children so recoordination reaches them).
      The defect was never *that* they clone, only *which budget* they cloned
      with.
- [ ] ~~Blocker first: finish the budget refactor into `system_foo.rs`.~~
      This is an INCOMPLETE REFACTORING, not a design choice (human, 2026-08-27;
      confirmed from history). `779b63f5` "FOOP-55 §5 Phase 3A: implement the SFF
      strip budget" introduced `StripBudget` and threaded it through every clone
      arm — but its diffstat shows it touched only `evaluator.rs` and
      `fir_kinds.rs`. **`system_foo.rs` was never updated.**

      The result: `ComparisonFir`/`ModuloFir`/`OrFir::constanic_clone` take
      `nyes`, `disable_nyes_reset`, `skip_foolish_children` — but NO budget
      parameter — so they call the pre-budget
      `ProtoBrane::clone_children_for_constanic_clone`, which mints
      `StripBudget::fresh()` unconditionally (`fir_kinds.rs:210`). Its comment
      still reads "Children of one clone share that clone's budget", describing
      a budget it never receives. Compare `fir_kinds.rs`'s own `FirKind::Operator`
      arm (`:398`), structurally identical, which correctly passes `stay_budget`.
      The three kinds differ ONLY because they live across a module boundary, so
      the worker delegates to them and the threading stopped there.

      Consequence for UFM: `unlimited()` silently truncates to `Some(1)` the
      moment a path crosses a Comparison/Modulo/Or operand — defeating "remove
      ALL layers" exactly where nested marks are thickest, since `system.foo`'s
      operators are built from `<<#-2>>`/`<<#-1>>` operands.

      Fix: give the three `constanic_clone` methods a `stay_budget` parameter and
      have them call `clone_children_budgeted`, matching the `OperatorFir` arm;
      then `clone_children_for_constanic_clone` has no callers and is deleted.
      That deletion also settles part of Phase 4B's "contract/combine the
      constanic_clone family" item. **This is a pre-existing latent bug in its own
      right — sibling marks under those three operators get a strip they should
      not — so it is worth fixing and testing on its own, ahead of UFM.**
- [ ] **`skip_foolish_children` — do NOT turn it on by default; decide its fate.**
      Human asked (2026-08-27) that it default to `true`. **Tried, and it must
      not be** — recorded here with the evidence so the question is settled and
      not re-opened blind.

      What it does: the clone's `foolish_children` comes out EMPTY;
      `ubc_children` is still cloned. Read plainly, "copy the computed result,
      drop the written expression that produced it."

      Why default-on breaks the language: **a `BraneFir`'s statements ARE its
      `foolish_children`** (see the `FirKind::Brane` arm of
      `_inner_constanic_clone`). Cloning a brane with the flag on therefore
      yields an EMPTY brane, and every search into it finds nothing. Flipping
      all call sites to `true` fails **55 of 385 unit tests** — `&?`/`&~`
      contexted searches return `None` instead of a found index, operator
      operands arrive as 0 children instead of 2, and concat cross-element
      resolution collapses. The old test's own assertion says it outright:
      `"skip_foolish_children must drop brane children"`.

      Status today: **dead** — every production call site passes `false`; the
      only `true`s are two tests. Born in `69c6e281` (FOOP-13 A3 step 6) whose
      message states "All existing call sites pass false by default" and names
      two intended callers that were never wired: a settled ConcatBrane clone
      (obsolete — that is now `BraneConcatOpFir`, an operator, not a brane) and
      the settled SearchFir clone path (the `:486`-style arm that duplicates
      `clone_children_budgeted` solely to honor this flag).

      Recommendation: **delete the parameter** and collapse that duplicated arm,
      folding it into Phase 4B's "contract/combine the constanic_clone family".
      It is scaffolding for a design that has since changed, not a capability
      awaiting a caller. Needs the human's word before removal, since the ask
      was to enable it.
- [ ] Lexer: `<@` / `@>`. Check for collisions the way `<*` was checked — build a
      one-line program and see what it parses to TODAY before assuming it is free.
- [ ] Token, AST node (`Astn::UnstayFoolish`), parser arm.
- [ ] `FirKind::Ufm` + `UfmFir`, built to the FOOP-55 §11 event-driven idiom
      (`BraneConcatOpFir` is the worked example): `fir_op_step` is pure
      orchestration; `on_foolish_op_ready` gates readiness and does the
      strip-clone; `on_ubc_op_ready` gates on `are_ubc_children_ready_for_op()`
      and settles from the drained result. `push_ubc_child` auto-enqueues the
      stripped EMBRYONIC clone, so the re-step is free — no explicit task push.
- [ ] Have the UFM operator name `OpInstructions::InsideUfm` for its strip-clone
      (the budget wiring and its `#[expect(dead_code)]` removal are already done,
      2026-08-27) and drop `InsideUfm`'s `cfg_attr(not(test), expect(dead_code))`
      once the operator constructs it.
- [ ] `foolish-core::Fir` variant and its satellite sites (~15), sequencer arm.
- [ ] `classify_concat_element`: a UFM is operator-shaped, so §9.2 rule 5 would
      currently make it an NK concatenation element. Decide its rule.
- [ ] DECIDE, then record: what does a UFM settle to when its content settles NK,
      or ECONSTANIC-after-unfreezing?
- [ ] DECIDE, then record: `< <@ x @> >` — the outer SF defers the UFM; once the
      UFM runs it strips everything below it. Confirm against a written example.
- [ ] Run all tests — old and new — and make sure they all pass correctly.

## Phase 3G — §9: concatenation ergonomics  ← **PRIORITY 2 (correctness)**

Read FOOP-55.md §9 first. **It supersedes every earlier statement in this FOOP
about element marking**; the "five rules" cascade implemented in `6eb69647` is
one of the superseded formulations and is to be replaced, not extended.

### 3G.1 — a central, maintained record of how FIR trees branch

- [ ] **Create one document that owns the FIR-tree shape list** — brane;
      statement (concatenation / operator / search, the search forms); markers
      with their single child. FOOP-55 §9.1 currently restates it, which is
      exactly the duplication to remove.
- [ ] **DECIDE where it lives** and record the choice: a new `docs/how/`
      document, or a section of an existing one. It must be a place that is
      maintained as the language grows, not a FOOP — FOOPs are dated proposals,
      and this is a living inventory.
- [ ] Replace FOOP-55 §9.1's list with a citation of that document.
- [ ] Note in it that the list is expected to grow, and that anything adding a
      FIR kind updates it.

### 3G.2 — specify the ergonomics (done)

- [x] FOOP-55 §9.2 states the rule: markers compile as written; branes and
      concatenations are SFF-marked; searches of every form are SF-marked;
      operators are not allowed and the compiler emits NK. §9.3 works the
      example through.
      (2026-08-13 17:05)

### 3G.3 — one einmo case, simple to complex

- [x] **`foop/55/concat_ergonomics.foo`** (2026-08-14 18:30) — a single input covering every case
      in §9.2, ordered simple → complex, each line commented with the rule it
      exercises:
  - [x] brane constituent → SFF-marked — `brane_pair` = {1;2;3;4}
  - [x] search constituent, uncontexted **unanchored** — `search_unanchored` = {0;8;9}
  - [x] search constituent, uncontexted **anchored** — `search_anchored` = {0;10;11}
  - [ ] search constituent, **contexted chain** — BLOCKED by D9: a MARKED
        search constituent stays ECONSTANIC and the concatenation never
        settles. Same signature as `{a={1,2}, b=<<#-2>>, c= a b}`. Add after 3G.5.
  - [x] constituent already `<…>`-marked → `mark_sf`/`ctx_sf` agree with the unmarked form
  - [x] constituent already `<<…>>`-marked → `mark_sff`/`ctx_sff` agree; `ctx_*` uses a context-dependent body (q=7), the case that would expose a downgrade
  - [x] nested written concatenation → `nested_deep` = {1..6}
  - [x] operator constituent → **NK** — `operator_nk` = {NK 1}
  - [x] the §9.3 worked example — marking is CORRECT ({e f g} flattens, x/y resolve). Its `b` is WOCONSTANIC because `{…}$ + 1` adds a number to a BRANE, an operator question outside §9.2; noted in the file
  - [x] `(({1}{2}) ({3}{4}))` flattens to four statements (§9.4b) — `nested_concat`
  - [ ] `{0} <<{q=1;}>>`, `{0} <{q=1;}>` and `{0} {q=1;}` — pin that a marked
        constituent is compiled as written (§9.2). They agree today, including
        with a context-dependent body; the case exists so a future change that
        makes them diverge is caught.
  - [x] single-element `c = {1}` is a brane (§9.4e) — `single_is_brane`
- [ ] Review every OUTPUT statement, then promote through the Promotion Review
      Gate — **HELD 2026-08-14**: the file carries two `@agent` notes (the
      D9-blocked search-constituent line, and `worked`'s WOCONSTANIC `b`). Per
      AGENTS.md an `@agent`-annotated snapshot may remain non-conformant
      pending human review. Do NOT promote until Atlas has read them.
- [ ] Run all tests — old and new — and make sure they all pass correctly.

### 3G.3b — what §9.4 exposed

See `docs/foop/FOOP-55.md` §9.4 (around line 1790) for the measured evidence
behind each item.

- [x] **(a) Give the concatenation NK a reason.** DONE (2026-08-14 18:12) — the
      rule-5 NK now reads "cannot concatenate number" via
      `concat_element_kind_name` (`foolish-ubca/src/compiler.rs`), detected at
      CLASSIFY time (earlier than settle, which §9.4(a) prefers). Pinned by
      `operator_constituent_is_error_naming_the_cause`.
- [x] **(a2) DECIDED — the parser may barf.** `{1} 2+3` need not reach the
      concatenation. Today it silently splits into `c={1}` and a separate
      statement `5`, because `is_concatenation_continuation`
      (`foolish-parser/src/parser.rs:532-554`) does not accept a bare integer as
      a continuation. Atlas's decision (2026-08-13): **a parse error is an
      acceptable outcome for now** — it is not worth widening
      continuation-start, which would touch every concatenation parse. The
      silent split is still the worse failure mode (it turns a malformed program
      into a DIFFERENT VALID ONE), so if the parser is being touched anyway,
      prefer erroring over splitting. NOT a blocker for §9.2. (2026-08-13 19:16)
- [x] **(b) A nested written concatenation loses constituents.** FIXED (2026-08-14 18:12)
      — rule 4 now matches `Astn::Concatenation` and `Astn::TailConcatenation`
      alongside `Astn::Brane`. Measured: `(({1}{2}) ({3}{4}))` was `{NK 1; 2}`,
      now flattens to `{1; 2; 3; 4}`. Pinned by
      `concat_constituent_classifies_as_brane_like`, which asserts a nested
      concatenation and a brane classify IDENTICALLY, since §9.2 equates them.
      Was: the second inner concatenation is absent, not NK. The parser produces
      `Concat(Concat(brane(1),brane(2)), Concat(brane(3),brane(4)))`, and §9.2
      treats a concatenation constituent exactly as a brane, so a four-statement
      flatten is expected.
- [x] ~~(c) `<<{…}>>` resolves as if single-marked~~ — **RETRACTED, not a
      defect.** It parses as a concatenation and behaves correctly; no case was
      ever shown in which SFF and SF should differ here but do not. See §9.4(c).
      (2026-08-13 18:52)
- [ ] Run all tests — old and new — and make sure they all pass correctly.

### 3G.4 — implement §9.2

- [ ] Unit tests first, one per §9.2 row
- [ ] Rewrite the classifier and builder to §9.2. The current cascade tests
      "already marked?" first and then inspects contents; §9.2 is flatter —
      classify the constituent, then apply that constituent's marking. Prefer
      the shape that reads like the table.
- [ ] **Operators emit NK** with a reason naming the constituent, per §9.2
- [ ] Implement marking as the **inserted-marker-node** procedure of §9.2, or
      something demonstrably equivalent; if equivalent-but-different, say why in
      the code comment
- [ ] Run all tests — old and new — and make sure they all pass correctly.
- [ ] REPORT baseline changes to the human before promoting anything

### 3G.5 — the D9 fix

Read FOOP-55.md §5.5 first — D9 and D10 (3G.6) are the two named violations of
one rule, and §5.5 states the rule once so the two fixes are not designed
independently.

- [ ] **A search RESULT must not inherit the searcher's SFM context.**
      `handle_found` passes `scope.has_ancestral_sfm` into the clone, so a
      result fetched from outside an SF marker keeps ECONSTANIC instead of
      resetting to Embryonic (`transform_for_clone`, `fir.rs:186-189`), and
      `push_ubc_child` then declines to enqueue it. See FOOP-55.md §D9.
- [ ] Tests first: the `{a = {1,2}, b=<<#-2>>, c= a b}` chain resolves
- [ ] Run all tests — old and new — and make sure they all pass correctly.

### 3G.6 — the D10 fix ← **PROMOTED from follow-up to required, DONE** (bisection, this branch)

Read FOOP-55.md §5.5, §5.6, §9.2, and §D10 first. D10 was originally scoped
as off the critical path; bisecting the FOOP-65 einmo regression found it is
the live cause of `foop/65/tail_concat_chain.foo.einmo` and
`foop/65/comprehensive.foo.einmo` losing elements (`d, e, f` silently dropped
from a backtick chain whose last operand is a nested written concatenation).
Both are already-shipped FOOP-65 baselines — this was a regression FOOP-55
introduced, now fixed with no code change to FOOP-65's own files.

**The fix turned out to be TWO independent defects, not one — the second
found only because fixing the first exposed it.** The first draft of this
sub-section (written from the API survey alone, before implementation)
correctly identified defect 1 but did not yet know about defect 2:

1. `ConcatenationFir::stmt_count`/`stmt_at` called `populate_concat_helpers()`
   directly, bypassing the gate `fir_op_step`'s own `Braning` arm applies.
   Fixed by gating on `_helpers_populated` (see the checkbox below for why
   that, not `is_constanic()`, is the correct gate).
2. **§9.2's classifier itself was wrong**: "Concatenation → SFF-mark it,
   treated exactly as a brane" conflated a Foolish Brane literal (a deferred
   value) with a written concatenation (an active joining process meant to
   run immediately). SFF-marking a nested concatenation froze its own
   constituent searches at construction, so it could never complete its own
   join. This was invisible until defect 1 was fixed, because defect 1's
   premature populate was itself masking the fact that the concatenation's
   own searches never even got a chance to run at all.

- [x] **Tests first** — the bisection's temporary reproduction was
      superseded by cleaner, purpose-written regression tests instead of a
      literal promotion:
      `nested_written_concat_as_constituent_joins_immediately` (isolated
      minimal case, pins both the classifier fix and that the join actually
      completes) and `nested_concat_as_tail_concat_last_element_flattens_completely`
      (the original tail-concatenation reproduction, now asserting all six
      values present). The original temporary test's throwaway diagnostics
      were removed per the `foolish-debugging` skill's cleanup discipline.
      (2026-08-22)
- [x] `ConcatenationFir::stmt_count()` refuses to answer (returns `None`) —
      and, in particular, no longer calls `populate_concat_helpers()` at
      all from outside `fir_op_step` — unless `_helpers_populated` is
      already `true`. **Corrected during implementation**: the gate is
      `_helpers_populated`, not `self.core.get_nyes().is_constanic()` as
      first planned — `Nyes::Woconstanic` is a GENUINELY correct terminal
      answer when a concatenation gives up via the "not joinable yet"
      escape (§5.5's correction), so `is_constanic()` alone still let a
      premature populate through; only `_helpers_populated` actually
      distinguishes "never attempted to join" from "joined successfully".
      (2026-08-22)
- [x] **Fix the three call sites where an anchored search trusts a resolved
      anchor's `constanic_is_brane_like()` without first confirming the
      anchor itself is constanic** (`fir_kinds.rs` — the `ib_search_with_engine`
      value-search-anchored arm, the positional-search anchored arm, and the
      `#`-index-search anchored arm). Each now checks
      `resolved.borrow().core().get_nyes().is_constanic()` before trusting
      `constanic_is_brane_like()`/scanning, staying pre-constanic (or NOT
      settling a permanent NK) rather than misreading a pre-constanic
      `resolved` as a final "not brane-like". (2026-08-22)
- [x] **A second, independently-discovered defect, found only once the
      `stmt_count` fix above exposed it: §9.2's classifier itself was wrong.**
      "Concatenation → SFF-mark it, treated exactly as a brane" conflated a
      Foolish Brane literal (a deferred value, correctly SFF-marked) with a
      written concatenation (an active joining process that must run
      immediately) — SFF-marking a nested concatenation propagated
      `under_sff` into its OWN constituent searches, freezing them at
      construction so it could never complete its own join. Fixed:
      `classify_concat_element` gives a nested `Concatenation`/
      `TailConcatenation` its own `ConcatElemKind::BareConcatenation` with
      no mark; only a genuine `Astn::Brane` literal gets `BareBrane`/SFF.
      §9.2's table in FOOP-55.md corrected accordingly. Two now-superseded
      unit tests (`concat_constituent_classifies_as_brane_like`,
      `tail_concat_equivalence_brane_literal`) updated to match — see
      commit `99ce4741`. (2026-08-22)
- [x] **A forward-to-parent search mechanism on `ConcatenationFir` was
      designed and implemented, then found unnecessary and removed.**
      Investigated per Atlas's design intent that a not-yet-populated
      concatenation should forward an IB search to its own parent. Traced
      and confirmed: `ConcatenationFir::stmt_count() == None` already makes
      `find_stmt_index` return `None` via `?`, which `SearchFir`'s existing
      `Embryonic`→`Braning` (IB-then-AB) fallback already treats as "try my
      parent next" — the general search machinery already implements the
      intended behavior once `stmt_count` stopped lying, with no
      concatenation-specific code needed. Confirmed by disabling the added
      mechanism and re-running the full suite with an identical result
      before removing it. FOOP-55.md §5.6 documents the final mechanism.
      (2026-08-22)
- [x] Re-verified `foop/65/tail_concat_chain.foo.einmo` and
      `foop/65/comprehensive.foo.einmo` now match their `checked/` baselines
      exactly (`einmo compare`: 4 matching, 0 differing across all four
      affected FOOP-65 files), with NO code change to FOOP-65's own files.
      (2026-08-22)
- [x] Run all tests — old and new — and make sure they all pass correctly.
      373 passed, 5 failed — every failure is pre-existing/expected
      (`ExtremumFir` WIP unrelated to this work, new FOOP-55 einmo cases
      pending their own promotion, `misc/sff_nested.foo`/
      `misc/concat_sf_f_more.foo` pending human-approval deprecation per
      Phase 3B). Zero new regressions.
      (2026-08-22)

### 3G.6b — commit D11's fix (found during the same bisection, already verified)

- [ ] Commit the `.rev()` restoration in `compiler.rs` (gated on
      `is_tail_concat`) — see FOOP-55.md §D11. Already verified against the
      full `cargo test -p foolish-ubca --lib` suite (370 passed) and against
      `einmo compare` for `tail_concat_basic`/`tail_concat_system_ops`.
- [ ] Run all tests — old and new — and make sure they all pass correctly.

### 3G.6c — add `misc/concat_sf_f_more.foo` to the Phase 3B human-approval list

- [ ] `misc/concat_sf_f_more.foo.einmo`'s divergence (`oo=-54` → `oo=-116`) is
      NOT a bug — it is §5's strip-budget fix correctly deferring a nested SFF
      mark that the old all-strip-in-one-pass code resolved too early (same
      category as `misc/sff_nested.foo`). Add it to Phase 3B's STOP-and-ask-
      human deprecation list rather than treating it as something to silently
      re-baseline.
- [ ] Run all tests — old and new — and make sure they all pass correctly.

### 3G.7 — `BraneConcatOp`: concatenation is an operator, not a brane

Read FOOP-55.md §10 first — it is the specification for this sub-section, drafted
before implementation per Atlas's direction (spec first, then test-driven).

**Already done, ahead of this phase (commit `b7b4813d`):** the `settled_result()`
half of §10 — deleting `ConcatenationFir`'s hardcoded-`None` override so `.value()`
correctly unwraps to the `ConcatHelper` once settled, per the universal rule every
FIR follows. Confirmed by experiment: exactly two test-behavior changes (both
rewritten to match), zero einmo OUTPUT regressions across the full suite.

- [ ] **Tests first.** For each of the ~21 direct-call unit-test sites (§10's
      survey) that call `.stmt_count()`/`.stmt_at()`/`.is_constanic_branelike()`
      (post-rename)/`._search_brane()` directly on a concatenation `FirRef`:
      decide whether the test's real intent is the OPERATOR or the RESULT, and
      rewrite to insert `.value()` where the RESULT was intended. Run against
      TODAY's code first — record which already pass (a pre-constanic `.value()`
      is unaffected by this work) and which correctly fail (the settled case).
  - [ ] Read the dense ~9-test block first (`concat_equals_big_brane`,
        `concat_search_brane_translates_global_indices`,
        `concat_ib_search_crosses_segments`, `concat_ab_search_reaches_outward`,
        `concat_contexted_search_spans_segments`, `concat_index_spans_segments`,
        `concat_find_stmt_index_is_global`,
        `concat_statement_parents_point_at_concat_helper`,
        `concat_constanic_clone_rewires_and_recoordinates`; `fir_kinds.rs`
        ~8639-8924) — built around "concatenation behaves like an equivalent
        big brane." This is where a genuine behavior gap, if one exists, will
        surface; read each individually rather than batch-rewriting.
- [ ] Rename `constanic_is_brane_like` → `is_constanic_branelike` trait-wide
      (`fir_trait.rs`'s default impl, every kind's override, every caller in
      `fir_kinds.rs`/`evaluator.rs`) — its own mechanical pass, done FIRST so
      the removals below are written against the final name. State the
      constanic-only-call precondition explicitly in the doc comment on the
      default impl and each override, since the old name signaled it
      implicitly and the new name no longer does.
- [ ] Remove `stmt_count()`, `stmt_at()`, `_search_brane()`, and
      `is_constanic_branelike()` from `ConcatenationFir` **one at a time**,
      running the full suite after each removal (`rust_instructions.md`'s
      testing discipline — one change, one verification, not a batch).
  - [ ] Before removing `is_constanic_branelike()`: confirm the one caller §10
        flags as needing care (`fir_op_step`'s element-classify,
        `elem.value().borrow().is_constanic_branelike()`) already calls
        `.value()` first — it does, per the read done while drafting §10, but
        confirm again against the code at removal time, not from memory.
- [ ] Rename `ConcatenationFir` → `BraneConcatOp` (mechanical). Decide and
      record: does `FirKind::Concatenation` stay (it names the syntax, which is
      accurate) or become `FirKind::BraneConcatOp` (for naming consistency)?
      Either is acceptable per §10 — record the choice made.
- [ ] Re-verify the full einmo suite: zero OUTPUT regressions across all 163
      files is the acceptance bar, exactly as it was for the `settled_result()`
      deletion. Re-verify the 8 einmo files §10's survey flagged as
      concatenation-related specifically.
- [ ] Run all tests — old and new — and make sure they all pass correctly.

## Phase 3F — Integration: make the exercise run

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
  - [ ] RE-MEASURE the depth/budget question (FOOP-55 §4 risks 1-2) **after
        §5 lands** — these were framed as budget questions, but the exercise
        computes nothing at any budget, so §5 is the cause. Does the
        ~1000-deep recursion still hit `MAX_DEPTH` (`fir_trait.rs:467`) or the
        iteration cap (`evaluator.rs:168`) once searches stop over-waiting? If
        yes: consult Atlas if the remedy is semantic; implement with tests;
        record in FOOP-55.md §4. Also decide whether `step_inner`'s **silent**
        `NoProgress` at `MAX_DEPTH` should become a loud error — a silent
        depth cap is how this failure stayed invisible.
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

## Phase 4B — Follow-up FOOPs discovered by this work

Not prerequisites, and not to be done inside this FOOP. Recorded here so the
work is not lost when this plan closes.

- [ ] **`misc/concat_sf_f_more_strange.foo` (new, split off 2026-08-25) is
      MUCH slower/more step-hungry than `misc/concat_sf_f_more.foo`, and it
      is not yet known whether it terminates at all.** History, corrected
      from an earlier draft of this entry that wrongly claimed a confirmed
      infinite loop (see below):
      - The committed `concat_sf_f_more.foo` originally had only
        `oo = o$;` (tail-extraction). `oo = o;`/`ooo = o$;` (full-clone of
        `o`, then tail) were tried as an addition and INITIALLY appeared to
        hang under a small (3000-step) raw test harness — that was a
        **false alarm**: re-run with the real composed-with-`system.foo`
        path and the CLI's actual 20000-step default budget, it settles
        cleanly (`root=Woconstanic`, `ooo=Constant`, matching the CLI's
        `ooo=-116`). **`oo = o;`/`ooo = o$;` is now the content of
        `concat_sf_f_more.foo`** (human direction, 2026-08-25) — this is
        the "working" version, confirmed settling within budget.
      - Separately, adding `check_b = b;`/`check_a = b;`/`check_c = c;`
        trailing statements to `f1`/`f2`/`f3` (to inspect `b`/`c` directly)
        DOES still leave the program `Braning`/`Prembrionic` after the same
        20000-step budget — genuinely much slower or non-terminating,
        NOT yet distinguished (human direction: no need to investigate
        whether more iterations would finish it, for now). Split into its
        own file, `misc/concat_sf_f_more_strange.foo` (`oo = o;`/
        `ooo = o$;` PLUS the three `check_*` statements), rather than kept
        as a variant of the working file.
      **TODO**: investigate `concat_sf_f_more_strange.foo` further —
      determine whether it is genuinely unbounded (per D8's "honest
      non-termination is not a bug" precedent) or a budget/efficiency
      question (per D12's retracted framing earlier this session — more
      steps might resolve it, or the `has_ancestral_sfm` leak tracked as
      Phase 3I might be the actual cause, given `check_c = c` in `f3`
      changes what `$`'s tail-extraction resolves through). Not scoped or
      started; revisit after Phase 3I lands, since that fix may resolve or
      clarify this file for free.

- [x] **`OperatorFir` settles NK (not WOCONSTANIC / empty brane) when an
      operand is a brane, not a number — FIXED 2026-08-25.** Confirmed live
      against `foolish-ubca/einmo_suite/input/foop/55/concat_ergonomics.foo`:
      - `operator_nk = {1} (2 + 3);` (line 88) now correctly settles NK
        (`??? (concatenation constituent indexes where it's not a brane:
        1)`) — matching the input's own `§9.2` comment's documented intent.
      - `worked = {b = {c = {e f g}}$ + 1; ...}` (line 118) now correctly
        settles `b` to NK (`??? (operator operand indexes that are not
        integers: 0)`) instead of hanging WOCONSTANIC forever — resolving
        the pre-existing `@agent` comment's concern (lines 98-106). That
        `@agent` comment should be removed from the input file the next
        time it is touched, per `AGENTS.md`'s embedded-communication
        convention (not done in this pass — the input file itself was not
        edited, only the code).
      **Fix** (`fir_kinds.rs`, `OperatorFir::on_foolish_op_ready`): the old
      `values.len() != children.len()` → `Woconstanic` branch conflated
      "an operand is not ready yet" with "an operand is ready but
      permanently the wrong type." Since this method is only ever called
      once the caller has already established every child is `constantew`
      (either via the any-NK check, or via
      `are_foolish_children_ready_for_op`), a `constantew` child that still
      fails to produce an integer here is ALWAYS a type error, never
      "unready" — now settles NK, naming the offending operand indexes
      (mirroring `ConcatenationFir`'s own `all_brane_like`/`type_errors`
      split).
      **Second bug found and fixed along the way (`evaluator.rs`,
      `proto_to_core_fir_inner`'s `FirKind::Concatenation` arm)**: a
      concatenation whose constituent-type-error NK path pushes an `Nk`
      into `ubc_children` was rendering as an EMPTY BRANE `{}` rather than
      NK — `.value()` correctly resolved to the `Nk` node, but
      `stmt_count()`'s default (`None` → `0`) silently rendered it as a
      zero-statement brane instead of falling through to NK rendering.
      This bug PRE-DATES the `OperatorFir` fix above (confirmed via
      `git stash`) and is what made `operator_nk` render wrong even though
      its own NYES was already, correctly, `Nk`. Fixed by checking
      `result.borrow().kind() == FirKind::Nk` before attempting
      brane-shaped rendering.
      Full suite: 377 passed, 0 unit-test failures, confirmed unchanged.

- [ ] **`SearchFir::found_context`'s brane reference needs releasing once
      the whole statement is constanic (GC hazard).** Found 2026-08-25
      while implementing `found_context: Option<(FirRef, usize)>` (the home
      brane + statement index captured at discovery, backing
      `Fir::is_found`). The stored `FirRef` to the home brane keeps that
      brane alive for as long as the search node itself lives, even after
      the search (and the whole statement containing it) has settled
      constanic and no longer needs the reference — a live `Rc` an
      already-finished computation has no further use for. Human direction
      (2026-08-25): revisit this once needed, likely by clearing
      `found_context` (or just the brane half of it) once the search's own
      statement reaches a constanic NYES, facilitating garbage collection.
      An earlier, since-reverted design (appending the found brane onto
      `foolish_children` instead of a side field) was considered and set
      aside — not the direction to pick back up without re-discussing.

- [ ] **`foolish_children`/`ubc_children` positional-index smell.** Found
      2026-08-25 while tracing how a contexted search / `@` (`SearchPositionFir`)
      learns whether its anchor "has a context" (a found statement) versus
      "has resolved a value." Both questions are answered today by reading
      `anchor.core().ubc_children().get(1)` / `.get(0)` directly, at
      multiple call sites (`contexted_search_from_anchor`,
      `SearchPositionFir::combine`, others) — the FOOP-23 "two-child
      invariant" (`[0]` = value, `[1]` = `FoolRefFir` referent) is enforced
      only by convention/comment, not by any named accessor or the type
      system. Human direction (2026-08-25): make `foolish_children` and
      `ubc_children` PRIVATE on every `Fir` implementor that has them, and
      provide named accessor/mutator methods instead of positional-index
      reads scattered across the codebase — e.g. something like
      `search_result_value()`/`search_result_referent()` in place of
      `ubc_children().get(0)`/`.get(1)`. A real refactor (touches every
      kind that has these fields), not a quick fix — scope and design in
      its own FOOP/plan section when picked up, not folded into FOOP-55's
      §11 work.

- [ ] **Investigate: `t~c@` (no parens) absorbs `@` into the search
      pattern.** Found 2026-08-25 while live-tracing `OperatorFir`'s
      `constanic_enough` migration against `@`. `{t={a=1,b=2,c=d};
      r=t~c@;}` parses/settles as a single search with literal pattern
      `'c@'` (ANCHORED) — which finds nothing (no statement is named
      `c@`) and settles NK — rather than `(t~c)` followed by a
      `SearchPositionFir` (`@`) continuation, which is what `(t~c)@`
      (parens required) actually produces (verified live: `r=2`, the
      correct position of `c` in `t`). Shouldn't `@` be recognized as a
      continuation operator directly after an unparenthesized search too
      (matching `#`/`$`/`^`'s own postfix-continuation behavior elsewhere),
      or is requiring explicit parens the intended, documented syntax for
      `@`? FOOP-55.md §8 (`@` spec) doesn't appear to say either way —
      check there first, then the parser's `is_concatenation_continuation`/
      continuation-parsing logic (`parser.rs`) for how `#`/`$`/`^` are
      recognized post-search versus how `@` is (or isn't) wired in
      alongside them.

- [ ] **Parser: validate operator strings at parse time.** Found 2026-08-24
      while migrating `OperatorFir` onto FOOP-55 §11's handlers.
      `OperatorFir::combine`'s `match self.op.as_str() { ... op => Err(...) }`
      unknown-operator arm is unreachable in practice (the parser only ever
      constructs `OperatorFir` with a known operator string), but nothing
      structurally enforces that — an `OperatorFir` built with a bad `op`
      string reaches this fallback at evaluation time, not construction
      time. §11's migration changes this arm from a hard `Result::Err` to
      an `NK` with an explanation (since `on_foolish_op_ready` reports
      `Option<Nyes>`, not `Result`), which is the right FVM-level answer,
      but the human's direction (2026-08-24) is that the PARSER should
      reject an unrecognized operator at parse time, so a malformed
      `OperatorFir` can never be constructed in the first place — that is
      parser work, out of scope for this FOOP's FIR-kind migration.

- [ ] **Create the "Search Context Access" FOOP** (run `foop_check.py gen_next`
      at creation time — do NOT reserve a number now). See FOOP-55.md
      Appendix A.B for the investigation that produced it.

      Every search result already carries its found statement as
      `ubc_children[1]`, a `FoolRefFir` holding the original with its parent
      chain, line number, and home brane intact (FOOP-23's two-child
      invariant). Today nothing in the *language* reads that context except a
      following `&`-search, which consumes it implicitly. The FOOP would make
      the context addressable:

      - **`@` — the found statement's index** in its home brane (0-based, to
        match `#`). `A~cond='True@`. Chosen over a trailing `#` because `#` is
        absorbed into `?`-patterns today (`src?b#` searches for the *name*
        `b#`), so overloading it would break the pattern language and need a
        delimiter rule; `@` has no conflict and composes with arithmetic
        directly (`A~cond='True@+1`).
      - Misses **propagate rather than sentinel**: anchored → NK, unanchored →
        ECONSTANIC. Collapsing ECONSTANIC to NK would kill a definition inside
        `system.foo`, where operands legitimately have no neighbours yet — the
        exact failure that forced `OrFir` (see plan Phase 2).
      - **Other extractions worth specifying in the same FOOP**, since they
        come from the same referent: the found statement's **name** (what a
        regex pattern actually matched), its **home brane** as a searchable
        value, and its settled **NYES** (did this hit or miss, without
        collapsing to a value).

      **Known blocker to design around:** `#` currently accepts only a
      **literal** integer — `src#n` and `src#(0-1)` both fail to parse — so
      `r#(c~cond='True@)` does not work today. Either `#` gains a computed
      operand (which gives the index search an evaluation phase it lacks), or
      the FOOP finds another consumer for the extracted index. Negative
      indices DO work (`src#-1` → last member), which makes a parallel
      "result table with a default row in the tail" idiom attractive.

- [ ] **RE-TEST D7 with SF instead of SFF** before treating it as a platform
      defect. D8's retraction (FOOP-55.md §Findings) shows the same mistake:
      SFF marks operands that are consumed in the SAME brane, so the local read
      never resolves. D7's reproduction
      (`f = {out = ({<<#-2>>, <<#-1>>, 'mod})$;}`) has exactly that shape.
- [ ] **MEASURE how many constanic clones each source construct performs.** The
      required SFF mark depth is one per clone, and a single source-level step
      can perform more than one (concatenation strips two — measured
      2026-08-11), so depth is NOT countable by reading the source. Until it is,
      every doubled mark is a guess. Options: a rule stating clones-per-
      construct, a diagnostic reporting the count, or a mark form that means
      "defer until used" rather than "defer N times".

- [ ] **Write the SF/SFF-usability FOOP** (Atlas, 2026-08-11: "this flexibility
      is dangerous and hard to reason about"). Both marks parse, both look
      reasonable, one silently never terminates with no diagnostic. Options: a
      diagnostic for the common failure (an SFF-marked statement read by name
      within its own brane is almost certainly a usage error), or a
      re-examination of whether both marks need to be surface syntax.

- [ ] **Write the equality-primitive FOOP.** FOOP-55 §8 clarifies FOOP-33 §2's
      rule 4 in place (naming "not mutually identifiable" and seaming the choice
      as `NOT_MUTUALLY_IDENTIFIABLE_IS_NOT_EQUAL`), but the equality primitive
      is now described across FOOP-33 §2, FOOP-23, and FOOP-55 §8. A future FOOP
      should **restate it succinctly in one place** — the three-valued result,
      what identity means for each kind, the not-mutually-identifiable relation,
      and the configuration seam. Behaviour is unchanged by that work; it is a
      consolidation.

- [ ] **Consider migrating the existing computing postfix operators to the
      brane-wrapper form** (`'name = {NameFir}`), per FOOP-55.md §7. `'mod`,
      `'or` and the comparisons use fixed SFF offsets today. The wrapper makes
      `1 + 'name` a TYPE ERROR rather than a runtime check, and allows any
      arity — but it is **not a mechanical rewrap**: each operator becomes
      responsible for its own arity checking, for deciding whether a
      non-candidate member is skipped or fatal, and for reading through
      `stmt_count`/`stmt_at` rather than `foolish_children` (its container may
      be a ConcatBrane). Out of scope here — those operators work, Euler 1 does
      not need them converted, and each conversion is a behavioural change
      deserving its own tests.

- [ ] **Record D7** in FOOP-55.md §Findings, or in its own defect FOOP: a bare
      `B = A` does not resolve an SFF-bearing expression. `{X=42; A = 1 +
      <<#-1>>; B = A}` hangs at `B` (BRANING) with one mark *or* two, while
      the same body under juxtaposition (`({X=41} A)$`) resolves to 42. Whether
      a plain name reference is *supposed* to recoordinate is a semantic
      question this FOOP does not need answered — §5 routes through
      juxtaposition, which works — but the next person will hit it.

- [ ] **Switch `foolish-ubca/einmo_suite`'s separator back to einmo's own
      default, `①` (circled digit one, `DEFAULT_SEPARATOR` in
      `einmo/src/format.rs`), instead of the current `FOOLISH_SEPARATOR`
      (`"!!\n"`, chosen because `!!` is a Foolish line-comment token — see
      `EinmoConfig::foolish_separator` in `einmo/src/config.rs`).** Human
      request (2026-08-26), found while adding
      `foop/55/d9_recoordinated_index.foo`: a `.foo` input comment written
      as a BARE `!!` line (no trailing text, used as a visual spacer between
      paragraphs — a style already present in this suite, e.g.
      `concat_ergonomics.foo`) collides with the section separator and
      einmo refuses to write the file (`"section \"INPUT\" contains the
      configured separator"`). `①` cannot appear in ordinary Foolish
      comments, so it would not collide the same way. **Trade-off to weigh,
      not yet decided**: `!!` reads naturally as "this is a Foolish
      comment" when a human opens a raw `.einmo` file's INPUT section,
      which `①` does not — confirm with a human reviewer before switching,
      and check whether any existing `.foo.einmo` file's INPUT/OUTPUT/
      COMMENTS content contains a literal `①` character before flipping
      (that would newly collide the other way). Not scoped as part of this
      FOOP's own work; a standalone follow-up.

- [ ] **Contract/combine `constanic_clone` methods using macros; search all
      `fn constanic_*`/`fn *_constanic_clone*` methods and review for
      redundancies.** Human request (2026-08-26), raised while reviewing
      D9's fix: the clone machinery ended up with 4+ named methods
      (`ProtoBrane::constanic_clone`, `ProtoBrane::_inner_constanic_clone`,
      `ProtoBrane::clone_children_budgeted`,
      ~~`ProtoBrane::clone_children_for_constanic_clone`~~ *(deleted
      2026-08-27, `55caa37d`)*, plus one `constanic_clone` per
      `system_foo.rs` kind — `ComparisonFir`/`ModuloFir`/`OrFir` — needed
      only because those structs live outside `fir_kinds.rs` and can't be
      built by its central `match`) where the design called for 2
      (`constanic_clone` the public entry, `_inner_constanic_clone` the
      recursive worker). Two real, separate sources of the bulk, worth
      addressing together:

      **On "can't we just `friend` the module?" (human, 2026-08-27).** Rust
      has no `friend`. The nearest tools are `pub(in crate::path)` (a
      narrower `pub(crate)`, but it restricts by ancestor module — it cannot
      name one sibling as privileged) and simply making the two files one
      module. Neither is the right answer here, because **visibility is the
      symptom, not the disease**: the bulk is ~10 near-identical bodies (8
      `Rc::new_cyclic` arms in `fir_kinds.rs`, 7 of them calling
      `clone_children_budgeted` verbatim, plus these 3), and merging the
      modules would leave ~10 near-identical bodies in one file. The
      `clone_self_shallow` sub-item below dissolves the module boundary
      outright — a trait method needs no visibility grant at all, each kind
      implements it where the type already lives, and the 3 `system_foo`
      functions disappear rather than merely getting shorter. Prefer it over
      both the macro and the module move; keep macros in reserve for
      whatever boilerplate survives in the central `match`.
  - [ ] The `Rc::new_cyclic(|me| { let self_weak = me.clone(); RefCell::new(Struct {
        core, self_weak, ...}) })` boilerplate repeated once per FIR kind
        in `_inner_constanic_clone`'s `match` — some kinds don't even read
        `self_weak` and could use a plain `Rc::new(RefCell::new(...))`
        instead of `new_cyclic`.
  - [ ] The per-kind reconstruction logic itself (each `match` arm
        re-deriving a kind's fields via its own `as_*` accessors) might be
        better as a trait method each kind implements on itself (e.g.
        `fn clone_self_shallow(&self, cloned_children, new_parent) -> FirRef`),
        replacing the one large central `match` — analogous to how
        `on_foolish_op_ready` already moved per-kind Braning logic out of a
        shared function and onto the kinds themselves (FOOP-55 §11).
      Not scoped as part of this FOOP's active work — a structural
      refactor, not a behavior fix; do it as its own deliberate pass once
      D9's fix itself is correct and tested, not interleaved with it.

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

**Date**: 2026-08-27
**Updated By**: Claude Code / claude-opus-5
**Changes**: Phase 4B — refreshed the "contract/combine `constanic_clone`
methods" item: struck `clone_children_for_constanic_clone` from its list of
methods (deleted 2026-08-27 in `55caa37d`) and answered the human's
2026-08-27 question "is there no way to `friend` a module so all the clone
code can be together?" in place. Recorded that Rust has no `friend`; that
`pub(in crate::path)` restricts by ancestor module and cannot privilege one
sibling; and that visibility is the symptom rather than the disease, since
the bulk is ~10 near-identical bodies (8 `Rc::new_cyclic` arms in
`fir_kinds.rs`, 7 calling `clone_children_budgeted` verbatim, plus the 3 in
`system_foo.rs`) that a module merge would leave untouched. Names the
existing `clone_self_shallow` trait-method sub-item as the preferred fix
over both the macro and the module move — it needs no visibility grant and
removes the three `system_foo` functions entirely — with macros held in
reserve for boilerplate surviving in the central `match`.

Earlier the same day: Phase 3J — checked off two items, both in commit
`55caa37d`. (1) `constanic_clone`'s `inside_sf_mark: bool` is replaced by the
three-variant `OpInstructions` enum (`Normal` / `InsideSfm` / `InsideUfm`),
each variant choosing its own starting strip budget; behavior for the two
pre-existing conditions is unchanged (einmo divergence set byte-identical),
and four new tests pin all three variants. (2) The `system_foo.rs` budget
refactor left half done by `779b63f5` is finished — the Comparison/Modulo/Or
dispatch arms now thread `stay_budget`, and the
`clone_children_for_constanic_clone` wrapper is deleted with zero callers;
recorded that those three are dispatch arms of `_inner_constanic_clone`, not
clone entry points, so they do legitimately clone. Added a decision checkbox
recording that `skip_foolish_children` must NOT be defaulted to `true`: a
brane's statements ARE its `foolish_children`, so enabling it empties every
cloned brane and fails 55 of 385 unit tests; the flag is dead (all production
call sites pass `false`) and the recommendation is deletion, pending the
human's word since the ask was to enable it. Prior history of this section is
in `git log`/`git blame` on this file.
