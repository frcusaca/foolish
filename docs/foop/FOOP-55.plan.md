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
- [ ] **DECIDE**: with §8 landed, are §7 (`ExtremumFir`) and Phase 3D (`'ite`)
      still needed? Both were routes to the same end. Delete what §8 made
      redundant rather than carrying it.

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

### 3G.6 — the D10 fix ← **PROMOTED from follow-up to required** (bisection, this branch)

Read FOOP-55.md §5.5 and §D10 first. D10 was originally scoped as off the
critical path; bisecting the FOOP-65 einmo regression found it is the live
cause of `foop/65/tail_concat_chain.foo.einmo` and
`foop/65/comprehensive.foo.einmo` losing elements (`d, e, f` silently dropped
from a backtick chain whose last operand is a nested written concatenation).
Both are already-shipped FOOP-65 baselines — this is a regression FOOP-55
introduced, not a stale baseline, and must be fixed before either file's
`einmo_gate_checked` can pass again.

**Corrected 2026-08-22, after reading the exact call sites (superseding the
first draft of this sub-section, written from the survey alone).**
`constanic_is_brane_like()` itself is NOT to be split or NYES-gated — its
existing callers at `fir_trait.rs:307,522` (`_get_my_brane`'s parent walk,
`step_inner`'s brane-cache) and `fir_kinds.rs:3180` (`fir_op_step`'s own
per-element `all_brane_like` classify) are genuine shape-only questions and
correctly work at any NYES; gating the method itself would break them. The
actual defect is narrower: `ConcatenationFir::stmt_count`/`stmt_at`
(`fir_kinds.rs:3250-3280`) call `populate_concat_helpers()` **directly**,
bypassing the gate `fir_op_step`'s own `Braning` arm already applies
correctly (it only populates helpers after confirming every element's
`.value()` is `all_brane_like`, `fir_kinds.rs:3177-3213`). That direct call
is the side door D10's mechanism goes through: asking `stmt_count()` on a
still-PREMBRIONIC nested `ConcatenationFir` triggers *that* nested
concatenation's own `populate_concat_helpers()` before its own `fir_op_step`
ever validated its elements.

- [ ] **Tests first** — promote the bisection's temporary
      `temporary_reproduce_to_debug_nested_concat_inside_tail_concat_drops_elements`
      (`foolish-ubca/src/fir_kinds.rs`, `mod tests`) to a named regression test
      per the `foolish-debugging` skill's cleanup discipline, rather than
      leaving it as a temporary test or deleting it
- [ ] `ConcatenationFir::stmt_count()` and `stmt_at()` must refuse to answer
      (return `None`) — and, in particular, must NOT call
      `populate_concat_helpers()` — unless `self.core.get_nyes().is_constanic()`.
      This is the actual D10 fix: per §5.5, "not yet constanic" is not a
      license to answer via a side door around `fir_op_step`'s own gate.
- [ ] **Fix the three call sites where an anchored search trusts a resolved
      anchor's `constanic_is_brane_like()` without first confirming the
      anchor itself is constanic** (`fir_kinds.rs:1847`, `:1965`, `:2238` —
      all three currently check only for `Nyes::Nk`, or nothing, before
      calling `constanic_is_brane_like()`/scanning; none currently confirms
      `resolved.borrow().core().get_nyes().is_constanic()` first). A
      pre-constanic `resolved` that happens to be a `ConcatenationFir`
      reaches the exact same premature-`populate_concat_helpers` path as
      the direct D10 case above, just via a search instead of a bare
      `stmt_count()` call. Add the `is_constanic()` check at all three,
      matching the pattern the sequencer already uses at `evaluator.rs:789`.
- [ ] Re-verify `foop/65/tail_concat_chain.foo.einmo` and
      `foop/65/comprehensive.foo.einmo` now match their `checked/` baselines
      with NO code change to FOOP-65's own files — the fix is entirely in
      `ConcatenationFir`'s content-readiness gating
- [ ] Run all tests — old and new — and make sure they all pass correctly.

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

**Date**: 2026-08-22
**Updated By**: Claude Code / claude-sonnet-5
**Changes**: Root-caused a live `einmo_gate_checked` failure: several
FOOP-65-owned baselines had regressed on this branch, which per the
non-regression invariant meant something in FOOP-55's own work had broken
them. Bisection found two distinct causes and this update records the plan
consequences (full narrative in FOOP-55.md's own Last Updated and its new
§5.5/§D10/§D11). **3G.5** (D9) gained a pointer to new **§5.5 — search-context
constancy**, the general rule D9 and D10 are both violations of: a
content/search query needs its search context constanic, produced by
ordinary stepping — never assumed from a pre-clone NYES, never approximated
by a partial answer, since UBCa's SFF mark already carries whatever
deferral would otherwise require one. **New 3G.6** promotes D10 from an
off-critical-path follow-up to a required fix: it is the live cause of
`foop/65/tail_concat_chain.foo.einmo` and `foop/65/comprehensive.foo.einmo`
silently dropping elements, gates `ConcatenationFir::stmt_count`/`stmt_at`
on `is_constanic()`, splits `constanic_is_brane_like()`'s conflated
shape/content questions, and fixes two ungated anchored-search sites the
same survey found. **New 3G.6b** commits an already-fixed, unrelated
defect (D11) the same bisection found: a compiler refactor merging the
`Concatenation`/`TailConcatenation` build arms dropped the `.rev()`
FOOP-65's backtick equivalence depends on. **New 3G.6c** and a **Phase 3B**
addition flag `misc/concat_sf_f_more.foo` for the same human-approval
deprecation path already used for `misc/sff_nested.foo` — its divergence is
§5's strip-budget correctly deferring a nested mark, not a bug. Prior
history of this section is in `git log`/`git blame` on this file, not
accreted here.
