# FOOP-75.plan — assignment-attached-searches

Implementation plan for [FOOP-75](FOOP-75.md) — *Assignment Attached
Searches*. **Read `FOOP-75.md` in full before executing any checkbox**, and
`FOOP-75.tests.md` (the tests written during design) before Phase 2.

Worktree variables, expanded:

```
WORKTREE_ORIGIN_BRANCH = jia
WORKTREE_ORIGIN_PATH   = /storage1/human/hcbusy/foolish
WORKTREE_BRANCH_NAME   = foop-75-assignment-attached-searches
WORKTREE_FULL_FS_PATH  = /storage1/human/hcbusy/foolish/../foolish_worktrees/foop-75-assignment-attached-searches
```

---

## Phase 0 — Gates and setup

- [x] Confirm `jia` is green before starting: `cargo test --workspace` passes with zero failures. **Do not proceed if anything is broken** (AGENTS.md §Development Rules).
      (2026-08-07 15:05) — **GATE FAILS. Breakage is PRE-EXISTING, not FOOP-75's.**
      Verified by stashing all FOOP-75 work and re-running on clean `jia@dc6db093`:
      the same three tests fail. Details:

      ```
      test ubca_snapshot_tester::einmo_tests::einmo_gate_output   ... FAILED
      test ubca_snapshot_tester::einmo_tests::einmo_gate_checked  ... FAILED
      test ubca_snapshot_tester::einmo_tests::einmo_gate_verified ... FAILED
      test result: FAILED. 310 passed; 3 failed
      ```

      Root cause, from `einmo_gate_checked --nocapture`:
      ```
      ALARM: ubca evaluation error: Iteration exceeded 9999
      exercises/project_euler/1.foolish.einmo was not written+verified:
        write/serialize error: section "INPUT" contains the configured
        separator; configure a different one
      ```

      Two distinct problems, **both belonging to FOOP-55** (the Project Euler
      exercise), introduced by commit `dc6db093` "Add an exercise. Many issues
      arose, two new foops to address them.":
      1. `exercises/project_euler/1.foolish` does not terminate — it hits the
         9999-iteration cap. This is the exercise FOOP-55 exists to make run.
      2. The einmo envelope cannot serialize that input because the file
         contains the configured separator (`!!` + LF — the Foolish line
         comment set by `TestConfig::foolish_separator()`).

      All other 310 tests pass, including every `foolish-parser`,
      `foolish-core`, and non-exercise `foolish-ubca` test. **Every crate
      FOOP-75 touches is green.**

      → **AGGREGATED AS QUESTION 1 FOR THE HUMAN.** AGENTS.md §Development
      Rules says never to start Phase-or-larger work while tests are broken,
      and that broken tests must be manually disabled by a human OR repaired
      and committed. This breakage is in a different FOOP's area, is not
      caused by and does not interact with FOOP-75, and cannot be repaired
      by FOOP-75 without taking over FOOP-55's work. Proceeding with
      implementation is therefore **deferred to the human's decision**;
      planning and committing plans (which the human explicitly requested)
      proceeds regardless, as it changes no code.
- [ ] Read `FOOP-75.md` §Terminology through §8 in full.
- [ ] Read `FOOP-75.tests.md` in full.
- [ ] Read FOOP-54 §D.5 (`docs/foop/FOOP-54.md:862-887`) — the in-force authority for what `=$` means.
- [ ] Read AGENTS.md §Searches (operator groups, NK vs ECONSTANIC miss outcomes) and `rust_instructions.md` §"Phase-by-phase testing discipline".
- [ ] Begin work: commit `FOOP-75.md`, `FOOP-75.plan.md`, `FOOP-75.tests.md` to `jia`, check `begun: [x]` in frontmatter
- [ ] Create worktree at /storage1/human/hcbusy/foolish/../foolish_worktrees/foop-75-assignment-attached-searches with branch `foop-75-assignment-attached-searches`
- [ ] **From here until merge, ALL work — including edits to `docs/foop/` — happens ONLY in the worktree.**

---

## Phase 1 — Surveys that gate the design

These determine whether §6.2 stays in this FOOP or splits out. **Do them before writing code.**

- [ ] Survey the repo for existing parenthesized patterns: `grep -rn '[~?](' --include=*.foo` across `foolish-ubca/einmo_suite/`, `test-resources/`, `samples/`, `docs/`.
  - [ ] Classify each hit: is the paren the WHOLE pattern (semantically inert under §6.2), or a sub-group like `~(a|b)c` (a **meaning change**)?
  - [ ] Record the classification in this plan as a checked sub-item with counts.
- [ ] Survey for attached-search forms already in use: `grep -rn '=[$^~?#.]' --include=*.foo`.
  - [ ] Verify `input/foop/13/comprehensive.foo:40`'s `#-1=$=s` is inside a **comment** and therefore unaffected (FOOP-75 §Test Plan).
- [ ] Identify every einmo baseline whose OUTPUT contains a `$`/`^` statement rendering, and list them here. These are the baselines Phase 5 must re-justify line by line.
- [ ] **Decision checkbox**: based on the surveys, does §6.2 (parenthetical pattern terminator) stay in FOOP-75, or split to its own FOOP?
  - [ ] If it splits: update `FOOP-75.md` §6 to state the split, adjust `FOOP-75.tests.md` §F to the §6.4 variant, and create the new FOOP via `foop_check.py gen_next`.
- [ ] **STOP — ask the human to confirm the §6.2 decision before implementing.** Present the survey counts and the recommendation.
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 2 — Lexer: adjacency information (§5.3)

**Prerequisite for everything else.** Verified on `jia@dc6db093`: `=$`, `= $`
and `=   $` lex to byte-identical streams, so §5 is unimplementable until
this lands.

- [ ] Write the §C tests from `FOOP-75.tests.md` into `foolish-parser/src/lexer.rs` tests module. Confirm they **fail** (red) before implementing.
- [ ] Add `preceded_by_space: bool` to `TokenAndLocation` (`foolish-parser/src/token.rs:59-63`) with the doc comment from §5.3.
- [ ] Make `skip_whitespace` (`lexer.rs:38-58`) report whether it consumed anything; carry that onto the next token. Note `next_token` already returns a `(TokenAndLocation, bool)` pair whose second element is discarded at `lexer.rs:32` — wire it rather than adding a parallel mechanism.
- [ ] Update `TokenAndLocation::new` call sites for the new field.
- [ ] Confirm the §C tests now pass (green).
- [ ] Confirm **no existing parse changes**: the field is additive and nothing reads it yet.
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 3 — Parser: attached searches (§1, §2, §3, §5)

- [ ] Write the §A and §B tests from `FOOP-75.tests.md` into `foolish-parser/src/parser.rs` tests module. Confirm they fail (red).
- [ ] Extract the postfix suffix loop (`parser.rs:640-760`) into a reusable routine that applies a recorded suffix run to a given anchor expression. **Reusing this exact code path is what guarantees §2's tree identity** — do not reimplement it.
- [ ] In `parse_assignment` (`parser.rs:296-368`), after consuming `Token::Assign`: if the next token begins a search operator (`^ $ ~ ? # .`) **and** `!preceded_by_space`, record the suffix run, parse the RHS, then replay the suffix against it.
- [ ] Enforce §5.1(3): an attached search **must** be terminated by a space. Emit a parse error naming the rule ("attached search must be terminated by a space") with line and column. Not an empty RHS.
- [ ] Exclude `&` from the trigger set (§5.4).
- [ ] **Delete** the two bespoke branches at `parser.rs:326-354` (the `=$`/`=^` special cases) and the synthetic `UnanchoredSeek { offset: -1 }` construction (§7).
- [ ] Confirm §A tree-identity tests pass for every operator and chain.
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 4 — Dead-path removal and value correctness (§7)

- [ ] Write the §D tests from `FOOP-75.tests.md` into `foolish-ubca`. Confirm they fail (red) — these are the three measured defects.
- [ ] Confirm the `"$"` arm of `OperatorFir` (`fir_kinds.rs:713`) is now **unreachable** (no parser path constructs it), then delete it.
- [ ] Verify no `"^"` arm needs adding — `=^` now routes through `IndexFir` via `Astn::HeadTail`, so the missing arm is moot rather than a gap to fill.
- [ ] Confirm `{b = {1,2,3}; y =$ b}` settles to `3` and `{b = {1,2,3}; y =^ b}` settles to `1` (FOOP-54 §D.5).
- [ ] Confirm `{d =$ 4}` settles NK per the anchored-miss rule.
- [ ] Extend the existing `IndexFir` `*_nyes_transitions` test's documentation to note it now covers the attached forms too (AGENTS.md §"NYES transition tests"). No new FIR kind is added, so no new transition test is required.
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 5 — Sequencer: the reverse direction (§4)

- [ ] Write the §E tests from `FOOP-75.tests.md` into `foolish-core/src/sequencer_tests.rs`. Confirm they fail (red).
- [ ] Generalize the `=$` sugar branch (`sequencer.rs:650-700`) into an **anchor-spine walk**: while the node is a search, follow its anchor; stop at the first non-search. Lift the whole run to the attached position.
- [ ] Include `IndexFir` in the spine walk — this is the gap that made `A = B$` render as `z=3` with the `$` lost. Consider adding `is_head()`/`is_tail()` predicates over `hs_index()` rather than repeating the `offset == -1 && anchored` magic-number pair at four call sites (readability; AGENTS.md §"When in Doubt").
- [ ] Change the emitted spelling per §4 normalization: `A = B$` renders as `A =$ B`.
- [ ] Confirm the fallback is total: non-search statements render byte-identically to before.
- [ ] Confirm round-trip idempotence (§E).
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 6 — einmo tests

- [ ] Create `foolish-ubca/einmo_suite/input/foop/75/` with per-operator attached-form cases, §3 chains, and §5 adjacency/termination/`&` cases.
- [ ] Write `foolish-ubca/einmo_suite/input/foop/75/search_operator_inside_patterns_howto.foo` from `FOOP-75.tests.md` §F — the documentary howto. **Its comments are its purpose**; a reader must be able to answer "can I put a `$` in a pattern?" from the file alone.
  - [ ] If §6.2 was deferred in Phase 1, write the §6.4 variant instead and state plainly in the comments that the parenthetical terminator is not yet available, naming the FOOP that would add it.
- [ ] Add the §G pin tests (current paren-absorption behavior), each with the "do not fix this test" comment.
- [ ] Evaluate: `cargo test -p foolish-ubca --lib -- run_einmo_tests`.
- [ ] **Justify every OUTPUT line before promoting** (AGENTS.md §"The einmo review workflow" step 4). For each changed or new line, state in your own words why that value is correct, citing FOOP-54 §D.5, AGENTS.md §Searches, or FOOP-75 §2/§4. "The evaluator emitted this" is not a justification.
  - [ ] Be skeptical of any `NK`: name which legitimate case applies, or trace it (`foolish-debugging` skill) before accepting.
- [ ] Promote only this FOOP's own new tests: `einmo promote output to checked foolish-ubca/einmo_suite`.
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 7 — The frozen verified baseline (§8)

**Handle separately and deliberately. Never promote over this.**

- [ ] Evaluate `regression/disappearing_brane_statements.foo` (input contains `d =$ 4`) and capture the exact new OUTPUT text.
- [ ] Compare against the frozen `verified/regression/disappearing_brane_statements.foo.einmo`, which pins `d =$ ??? (4 is not a brane);`.
- [ ] If the text is **unchanged**: record that here and move on.
- [ ] If the text **changed**: determine whether it can be made identical by giving the `IndexFir` miss path the same alarm reason (§8). Prefer preserving the baseline over changing it.
- [ ] **STOP — ASK HUMAN.** Present the exact diff and the recommendation. A `verified/` baseline is frozen and requires a human reviewer's key. **UNDER NO CIRCUMSTANCES may an agent promote or re-sign this file.**
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 7.5 — FOOP-65 coordination (§9)

Read `FOOP-75.md` §9 before this phase. Neither FOOP blocks the other; this
phase keeps them from colliding.

- [ ] Determine whether FOOP-65 has landed on `jia` at this point (`git log --oneline jia -- docs/foop/FOOP-65.plan.md` and check its `begun`/checkbox state).
- [ ] **If FOOP-65 landed first**: add the §F2 tests from `FOOP-75.tests.md`, confirming `A =$ fn`{a,b}` and `A = (fn`{a,b})$` build identical trees (§9.3 reading (i)).
  - [ ] Confirm FOOP-65's `parse_expr` refactor (its §4 — the new weakest precedence level) did not disturb the attached-search replay. The replay operates on `parse_expr`'s returned tree, so it should be transparent; verify rather than assume.
- [ ] **If FOOP-75 lands first**: copy §F2 verbatim into FOOP-65's test plan and add a checkbox to FOOP-65's plan requiring `Token::Backtick` to populate the `preceded_by_space` field added by §5.3 (§9.4).
- [ ] Either way: note in FOOP-65's Open Questions that its "`$`-after-backtick ergonomics" question is **answered** by FOOP-75 §9.3 — `=$ fn`X` is the parenthesis-free form of `(fn`X)$`.
- [ ] Consider siting `is_head()`/`is_tail()` (Phase 5) as a named `FirQueryable` classifier family so FOOP-65's `is_tail_concatenation` joins it rather than adding a fourth bespoke sequencer branch (§9.2).
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 8 — Documentation corrections

- [ ] Correct FOOP-23 §942/946: the claim that `a$=b` / `a^=b` are "already implemented" is a **transposition error** — the implemented forms were `a=$b` / `a=^b` (FOOP-75 §7). Fix the prose; do not rewrite the surrounding decision.
- [ ] Update FOOP-55 §D6/§E5 to note that FOOP-75 resolves the `$=` question (`$=` is not adopted; `=SEARCH_SPEC` is the general form). Leave the §E5 `(X)$` rewrite as-is — it remains valid.
- [ ] Add **"Attached search"** and **"Attached search sequence"** to AGENTS.md §Foolish Terminology, per FOOP-75 §Terminology.
- [ ] Add FOOP-75 to `docs/foop/INDEX.md` in the correct little-endian position, with a one-line summary.
- [ ] Update the "## Last Updated" section of every `.md` touched (AGENTS.md §Markdown File Update Protocol — **replace** the entry, do not append).
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 9 — Comprehensive test, merge, cleanup

- [ ] Write and verify `foolish-ubca/einmo_suite/input/foop/75/foop_75_comprehensive.foo` — attached searches mixed with existing features: nested branes, FOOP-54 §D.5 concatenation-based function application, creations, and searches inside attached RHS expressions. At least one path through every operator in the §1 trigger set.
- [ ] Run all tests — old and new — and make sure they all pass correctly.
- [ ] Verify all work is complete in /storage1/human/hcbusy/foolish/../foolish_worktrees/foop-75-assignment-attached-searches and committed to `foop-75-assignment-attached-searches`
- [ ] Merge `foop-75-assignment-attached-searches` to `jia`
  - [ ] Run all tests — old and new — and make sure they all pass correctly.
  - [ ] STOP! STOP!! STOP!!! ASK HUMAN to check this box before continuing. UNDER NO CIRCUMSTANCES will Agent continue past this point automatically!!
    - [ ] Present human with `cd /storage1/human/hcbusy/foolish/../foolish_worktrees/foop-75-assignment-attached-searches` and ask them to review snapshots BEFORE checking the parent checkbox.
  - [ ] Repair ALL tests in `jia` if the merge breaks anything.
- [ ] Cleanup /storage1/human/hcbusy/foolish/../foolish_worktrees/foop-75-assignment-attached-searches
  - [ ] Check that this plan has all but Cleanup checkboxes completed
  - [ ] Remove /storage1/human/hcbusy/foolish/../foolish_worktrees/foop-75-assignment-attached-searches
  - [ ] This is the last sub-task checkbox to be checked in this block

---

## Last Updated

**Date**: 2026-08-07
**Updated By**: Claude Code / claude-opus-5
**Changes**: Initial plan. Phase 1 gates the §6.2 parenthetical-terminator
decision on a repo survey (it may split to its own FOOP). Phase 2 is the
lexer adjacency prerequisite without which §5 is unimplementable. Phase 7
isolates the frozen verified baseline behind a human STOP.
