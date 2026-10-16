# FOOP-27.plan — equivalences

Read `FOOP-27.md` in full before executing any box. Phases 1–3 are planning: they run in the main
checkout (`/storage1/human/hcbusy/foolish`, branch `jia`) as documentation and test-writing commits,
and `begun:` stays `[ ]`. Phase 4 (code) is written by Phase 3 into this file. It starts with the
worktree checkboxes and runs **only after FOOP-26 has merged to `jia`** (FOOP-27 §D.3).

Prepared against `jia` at `eec1dfc4` (2026-10-16).

## Standing rule — concurrent efforts (FOOP-27 §D.4)

- [ ] At the start of EVERY phase below, and before every commit: run `git log --oneline eec1dfc4..jia`
      and `git branch -a`; re-read the spec and plan of each FOOP in §D.4's table whose state moved;
      record here the `jia` commit the phase was prepared against. Findings for another FOOP go to
      that FOOP's planner or the human, never into its files.

## Phase 1 — Semantic design (larger model)

- [ ] Establish relevant tests for this phase. Use [these instructions](../../README.md#running-specific-tests) to run unit tests: foolish_ubca2::sequencer::tests::foolish_rendering_round_trips_constanic_programs, foolish_ubca2::einmo_gates::einmo_corpus_wide_foolish_rendering_parses. Nothing in this phase changes code; the subset is to confirm the baseline (470 / 0 / 0 at `eec1dfc4`).
- [ ] Read FOOP-27 Parts A–D, `docs/vintage_legacy/EQUIVALENCE.md`, FOOP-36 §2 and §N1–§N4, and
      FOOP-07 §2 and §4 (on branch `worktree-agent-a5e87dc31591ca72a`).
- [ ] Re-run the E1 measurement (FOOP-27 §Motivation method) on the current `jia`; update the table
      if anything moved.
- [ ] Present Q-A1–Q-A5, Q-E3a–Q-E3d and Q-C1, Q-C2 (including Appendix 1 §X1.3's brane-vs-brane
      question) to the human in ONE statement, each with a recommendation.
      STOP until answered.
- [ ] Record the answers in the spec (§A.2, §A.3, §A.5, Appendix 1) and empty those Open Questions.
- [ ] Complete Part A's catalogue: for each of E4–E13, state what it compares, what it ignores, and
      its status (catalogued / sketch), and whether it is Foolish-accessible (today: E0 only). Add
      any equivalence the reading turns up.
- [ ] Specify E3 (Peerage) from §A.3's answers, including the shape of its later name-and-type
      (type-checking) extension.
- [ ] Settle E0's relation to E2/E3 (Appendix 1 §X1.3): the brane-vs-brane outcome, and the mapping
      between E0's three outcomes and E2's.
- [ ] Work Part B item by item. For each of B1–B13, record the representation or rendering rule that
      makes it impossible, or the owning FOOP plus the gate that would catch a regression, or an
      explicit exemption with its reason.
- [ ] For every FIR or FVM change Part B calls for, write §D.2's three statements: what it adds, that
      every program keeps its meaning, and which einmo cases would move. Any change that narrows a
      capability: STOP and ask.
- [ ] Raise Q-B1 (FOOP-07 §4.3) and Q-B2 (`&^`/`&$`) with the human and FOOP-07's planner.
- [ ] Draft the new `EQUIVALENCE.md` content inside FOOP-27 as an appendix. It is moved into the live
      docs tree in Phase 4, per §C.5.
- [ ] Report all accumulated doubts to the human in one statement, or record "no doubts".

## Phase 2 — Illustrative tests, written into the FOOP (larger model)

- [ ] Establish relevant tests for this phase. Use [these instructions](../../README.md#running-specific-tests) to run the same subset as Phase 1. No code changes.
- [ ] Add a §Illustrations section to FOOP-27.
- [ ] E1: one minimal program per resolved Part B item, with its expected `R1` written by hand
      (minimum: `recipe.steps~step_1&#5`, `recipe.steps~step_1&?outside`, `ba~=(bb.v)`,
      `(r.s?fry).oil`, `{a=1;b=2}&#0`, the two unsteppable routes).
- [ ] E2, per parameter: one pair that the parameter decides, with the answer under each setting.
      For example, `{a=1;b=2}` vs `{b=1;a=2}` is `NO` under `names_compared` and `YES` under
      `names_ignored`; `{x=1/0}` vs `{x=1/0}` is `YES` under `nk_equal` and `NO` under `nk_strict`.
      Cover every interaction in §A.2's "must settle" list.
- [ ] E3: peer and non-peer pairs, at least the four in §A.3's example, plus one per Q-E3a–Q-E3c answer.
- [ ] E0: one value-search and one null-const-rule example per Appendix 1 §X1.1 rule, plus any
      change §X1.3 decides.
- [ ] E2, carried: write out T-BIJ, T-SEED, T-SIG, T-SHAPE, T-SHARE and T-PRE as concrete inputs with
      expected answers, restated against §A.2's parameters.
- [ ] Each illustration names the concurrent-FOOP assumption it depends on, if any (§D.4 rule 2).
- [ ] Human review of §Illustrations. STOP until approved.

## Phase 3 — Coding plan and all test cases (3a larger, 3b smaller)

- [ ] STOP unless FOOP-07, FOOP-46 and FOOP-26 have all merged to `jia`. Record the `jia` commit.
- [ ] Re-measure E1 over the corpus; reconcile with Part B; count `verified/` twins that will move (Q-B3).
- [ ] Enumerate the parser's precedence pairs and add one E1 illustration per pair.
- [ ] 3a: write Phase 4 into this file:
      - worktree `foop-27-equivalences` at `/storage1/human/hcbusy/foolish_worktrees/foop-27-equivalences`;
      - per-sub-section "Establish relevant tests" boxes;
      - Promotion Review Gate blocks naming every moving case;
      - the comprehensive test `foop/27/comprehensive`;
      - the move of `EQUIVALENCE.md` out of `vintage_legacy/`;
      - the merge STOP and cleanup.
- [ ] 3b: transcribe every §Illustrations entry into a real einmo input or Rust unit test, with its
      expectation, as files in the Phase 4 worktree's first commit. They are expected to FAIL until
      Phase 4's code lands.
- [ ] Human review of the Phase 4 plan. STOP until approved; then set `status: Final`.

## Phase 4 — Code

- [ ] (written by Phase 3)
