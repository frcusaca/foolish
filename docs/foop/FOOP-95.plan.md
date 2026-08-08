# FOOP-95.plan — embryonic-and-resequencing-einmo-sections

Read `FOOP-95.md` FIRST — this plan assumes the specification. Execute top
to bottom. Variables are already expanded to literals (per `foop.md`):

```
WORKTREE_ORIGIN_BRANCH=jia
WORKTREE_ORIGIN_PATH=/storage1/human/hcbusy/foolish
WORKTREE_BRANCH_NAME=foop-95-embryonic-resequencing-sections
WORKTREE_FULL_FS_PATH=/storage1/human/hcbusy/foolish/../foolish_worktrees/foop-95-embryonic-resequencing-sections
```

Note: this shell needs `RUSTUP_TOOLCHAIN=stable` exported (no default
toolchain configured on this machine).

**[FOOP-65](FOOP-65.md) depends on this FOOP** (for the einmo visibility of
its §5.3.1 backtick rendering). Nothing here depends on FOOP-65. Landing
this one first is preferable — confirm the order with Atlas.

**Scope warning.** This FOOP rewrites EVERY baseline in the einmo suite
(new EMBRYONIC section + section reorder). Phases 1–3 are ordinary code
work; Phase 4 is a corpus-wide re-promotion behind two human-gated
inspections. Do not let Phase 4 begin until Phases 1–3 are green.
**Phase 5 (the Foolish Resequencer) lands LAST** and is separable — if the
FOOP feels sprawling by then, ask Atlas whether it should become its own
FOOP rather than pressing on.

## Phase 0 — Begin, baseline, and orientation

- [ ] Begin work: commit FOOP-95.md and FOOP-95.plan.md to origin (`jia`),
      check `begun: [x]` in FOOP-95.md frontmatter
- [ ] Create worktree at /storage1/human/hcbusy/foolish/../foolish_worktrees/foop-95-embryonic-resequencing-sections with branch `foop-95-embryonic-resequencing-sections`
      (`git worktree add -b foop-95-embryonic-resequencing-sections /storage1/human/hcbusy/foolish/../foolish_worktrees/foop-95-embryonic-resequencing-sections`
      from `jia` at /storage1/human/hcbusy/foolish; ALL subsequent work —
      including edits to FOOP-95.md and this plan — happens ONLY in the
      worktree until merge)
- [ ] Read `rust_instructions.md` in full (mandatory before any Rust;
      especially §"Phase-by-phase testing discipline")
- [ ] Read FOOP-95.md in full — especially §1.1 (section order), §3 (the
      `stmt_count` split), §4 (the inspection step), §5/§5.1 (corpus
      impact and the two-gate re-promotion)
- [ ] Read the touched sites: `foolish-ubca/src/fir_kinds.rs`
      (`ConcatenationFir::stmt_count` 2840-2855 — THE DEFECT, `stmt_at`
      2857-2869, `_search_brane` 2887-2893, `populate_concat_helpers`
      2632-2674); `foolish-ubca/src/fir_trait.rs` (trait decl 346,
      `is_brane_like` 366); `foolish-ubca/src/evaluator.rs`
      (`evaluate` 118-149, Concatenation render arm 706-764);
      `foolish-ubca/src/system_foo.rs` (`program_result` 482-504);
      `foolish-ubca/src/ubca_snapshot_tester.rs` (adapter 36-48);
      `foolish-core/src/sequencer.rs` (concatenation 496-545);
      `einmo/src/compare.rs` (always-compared set 69-71),
      `einmo/src/verify.rs` 257-265, `einmo/src/transitions.rs` 461-476,
      `einmo/src/cli.rs` 1151
- [ ] Baseline in the worktree: `RUSTUP_TOOLCHAIN=stable cargo test --workspace`
      and `cargo test -p foolish-ubca --lib -- run_einmo_tests` — both
      green before any change
- [ ] **Enumerate every `verified/` baseline** and present the list to
      Atlas — these are frozen, need the human key, and cannot be
      re-promoted by the agent (FOOP-95 §5)
- [ ] **Snapshot the pre-change corpus** (copy `checked/` aside) so Phase 4
      Gate A can compare section bodies old-vs-new mechanically
- [ ] Run all tests — old and new — and make sure they all pass correctly.

## Phase 1 — The `stmt_count` purity split (tests first)

- [ ] Write the failing tests FIRST (FOOP-95 §3, Test Plan):
      `stmt_count()` on an un-stepped concatenation leaves
      `_helpers_populated == false` and `ubc_children()` empty (**fails
      against today's code — this is the defect's regression test**);
      `stmt_count()` and `stmt_at(..)` agree about emptiness on an
      un-populated concatenation; `ensure_joined_stmt_count()` populates
      as today's `stmt_count` did and is idempotent
- [ ] Implement the split: pure `stmt_count` + explicitly-named
      `ensure_joined_stmt_count` on the `Fir` trait
      (`fir_trait.rs:346`) and on `ConcatenationFir`
- [ ] **Classify EVERY existing `stmt_count()` call site individually**
      (~20 non-test sites in `fir_kinds.rs`, `evaluator.rs`,
      `system_foo.rs`) — join-needing sites (settled render
      `evaluator.rs:713-716`, `program_result` `system_foo.rs:486`,
      search/navigation paths) → `ensure_joined_stmt_count()`; genuine
      queries → `stmt_count()`. **Do NOT blanket-rename** (FOOP-95
      Rejected Alternative B)
- [ ] Migration guard: the whole einmo suite is still byte-identical —
      `cargo test -p foolish-ubca --lib -- run_einmo_tests` green with NO
      baseline touched. Any divergence = a misclassified call site; fix
      the classification, never promote
- [ ] Run all tests — old and new — and make sure they all pass correctly.

## Phase 2 — The pre-step rendering path (tests first)

- [ ] Write the failing tests FIRST: producing the EMBRYONIC rendering
      leaves the settled OUTPUT byte-identical to the same input evaluated
      without it; after producing it the composed root is still wholly
      un-stepped (no NYES advanced, no helper populated)
- [ ] Implement per FOOP-95 §2: `UbcaEvaluator::evaluate`
      (`evaluator.rs:118-149`) gains a pre-step rendering taken after
      `compose_program_with_system` and BEFORE `step_to_settled`; the
      conversion must be **purely read-only**
- [ ] Decide and record: do the new sections render the composed root or
      only the `program` member? (FOOP-95 Open Questions — presumed
      `program` only; confirm with Atlas)
- [ ] Run all tests — old and new — and make sure they all pass correctly.

## Phase 3 — einmo section reorder + EMBRYONIC wiring (tests first)

- [ ] Write the failing einmo tooling tests FIRST (FOOP-95 §1.1, Test
      Plan): `einmo compare` reports a difference when ONLY the EMBRYONIC
      body differs (**the critical one — proves the section is actually
      gated, not merely written**); an envelope round-trips under the new
      order with every body byte-identical; `sections:` declares exactly
      the emitted order
- [ ] Implement the order `METADATA, OUTPUT, EMBRYONIC, INPUT, COMMENTS,
      STAMPS` (`STAMPS` MUST remain last — it signs what precedes it);
      `INPUT` keeps its wire name
- [ ] **Add `EMBRYONIC` to `einmo/src/compare.rs`'s always-compared set
      (69-71).** Without this the section is pinned in name only
- [ ] Update section-name constants/fixtures: `einmo/src/verify.rs`
      257-265, `transitions.rs` 461-476, `cli.rs` 1151; check the
      parser/writer for any "INPUT is first" assumption
- [ ] Emit EMBRYONIC from the adapter (`ubca_snapshot_tester.rs:36-48`)
- [ ] Run all tests — old and new — and make sure they all pass correctly.

## Phase 4 — Inspection, repair, and corpus re-promotion (HUMAN-GATED)

- [ ] **Significant step — inspection of embryonic Foolish for reasonably
      informative rendering, BY AGENT (FOOP-95 §4).** Generate the
      pre-step rendering across a broad slice of the corpus (nested
      branes, concatenations, searches, SF/SFF markers, operators,
      creations, comparisons, if-expressions). For each construct answer:
      does it show what was built? can a PROGRAM AUTHOR read operand
      grouping and precedence off it? does it read as Foolish? any bare
      placeholders, internal names, `items=` debug forms, empty shapes,
      or missing structure? Record every finding as a defect against the
      construct that provoked it.
- [ ] **Inspection BY HUMAN.** Present samples to Atlas against the
      governing criterion: *is this reasonably informative for the
      purposes of future development, writing and maintaining Foolish
      programs?* The human's judgement governs.
- [ ] Repair sequencer defects found above; agree with Atlas which are
      fixed here vs deferred to their own FOOPs (FOOP-95 Open Questions)
- [ ] Re-inspect after repair; loop until Atlas is satisfied. **Only then**
      does the rendering format freeze.
- [ ] Write per-construct `foolish-core` sequencer tests recording what
      "informative" was decided to mean (FOOP-95 Test Plan)
- [ ] Write and verify `foolish-ubca/einmo_suite/input/foop/95/comprehensive.foo`
      (reserved name) exercising every construct §4 examined
- [ ] Run all tests — old and new — and make sure they all pass correctly.
- [ ] **GATE A (mechanical, whole corpus) — nothing that existed changed.**
      Script the comparison against the Phase-0 snapshot: extract
      OUTPUT/INPUT/COMMENTS bodies from every old and new envelope and
      assert byte equality. Report QUANTITATIVELY ("N baselines checked, N
      byte-identical"). Any OUTPUT difference = a misclassified Phase-1
      call site; any INPUT/COMMENTS difference = the writer mangling
      content during reorder. **Fix the code — never promote past it.**
- [ ] **GATE B (human) — the new sections are worth having.** Atlas reviews
      EMBRYONIC bodies (and RESEQUENCED, once Phase 5 has landed) across a
      representative spread of the corpus.
      Justify every EMBRYONIC line in your own words first (AGENTS.md
      §"The einmo review workflow" step 4); "the evaluator emitted this"
      is NOT a justification.
- [ ] STOP! ASK ATLAS to sign off on BOTH Gate A and Gate B before any
      `checked/` promotion. UNDER NO CIRCUMSTANCES promote the corpus
      automatically.
- [ ] Promote `output` → `checked` corpus-wide only after sign-off
- [ ] Present the `verified/` twin list again — those need the human key
      and cannot be re-verified by the agent
- [ ] Run all tests — old and new — and make sure they all pass correctly.

## Phase 5 — The Foolish Resequencer + normalization (LAST; tests first)

Per FOOP-95 §6 — this lands LAST.
**Do not begin until Phases 1-4 are green and promoted.** If this FOOP is
sprawling by the time you reach here, STOP and ask Atlas whether §6 should
become its own FOOP (FOOP-95 Open Questions flags it as the clean cut).

- [ ] Confirm with Atlas: does RESEQUENCED reserve its section slot from
      Phase 3, or get inserted now (rewriting every baseline a SECOND
      time)? (FOOP-95 §1.1, Open Questions)
- [ ] Confirm with Atlas: normalization textual-on-lexer-tokens vs
      canonical-resequencing-derived (FOOP-95 §6.2, Open Questions —
      recommendation is textual, built on the REAL lexer's token stream so
      it cannot drift from the lexer)
- [ ] Write the failing normalization tests FIRST (FOOP-95 Test Plan): one
      per rule — comments removed; whitespace runs collapse **but NOT
      inside strings**; empty lines removed; `,`/`;` → `;`; Unicode
      operator forms; brane on a new line; 2-space indent. Plus
      `normalize(normalize(x)) == normalize(x)` and adversarial strings
      containing `!!`, `;`, and space runs
- [ ] Implement normalization as a documented module with its own tests —
      NOT a test helper (FOOP-95 §6.2)
- [ ] Write the failing resequencer tests FIRST, **check 2 before check 1**
      (idempotence is cheaper and needs no normalizer):
      `resequence(parse(r)) == r`; then fidelity
      `normalize(resequence(parse(src))) == normalize(src)` across nested
      branes, concatenations, searches, SF/SFF markers, operators,
      comparisons, if-expressions, creations
- [ ] Implement the **Foolish Resequencer** as a SEPARATE component beside
      `FirSequencer` (`foolish-core/src/sequencer.rs:32-34`), not a mode of
      it (FOOP-95 Rejected Alternative E)
- [ ] Creation limit tests (FOOP-95 §6.4): `⬤` resequences to valid syntax
      and re-parses to a NEW, DISTINCT creation — assert identity is NOT
      preserved; a named creation resequences under its original name and
      does NOT re-parse as a FOOP-33 rename
- [ ] Emit the RESEQUENCED section; add it to `einmo/src/compare.rs`'s
      always-compared set (same hazard as EMBRYONIC — a section that is
      written but never diffed is pinned in name only)
- [ ] **Corpus-wide round-trip:** run both equality checks over EVERY input
      in the suite. **Expect genuine parser/FIR-gen bugs to surface — each
      is a real find to REPORT TO ATLAS, never to normalize away**
      (FOOP-95 §6.5). A fidelity failure must NOT be "fixed" by weakening
      the normalizer until it passes.
- [ ] **Inspection step (FOOP-95 §6.5), by agent AND human:** is the
      normalized form one a Foolisher would accept as a faithful rendering
      of their program? are the normalization rules right and complete?
      For each fidelity failure, classify: resequencer bug, normalizer
      bug, or genuine parser/FIR-gen information loss. PRESENT TO ATLAS.
- [ ] Promote RESEQUENCED baselines only after Atlas sign-off
- [ ] Run all tests — old and new — and make sure they all pass correctly.

## Merge

- [ ] Verify all work is complete in /storage1/human/hcbusy/foolish/../foolish_worktrees/foop-95-embryonic-resequencing-sections and committed to `foop-95-embryonic-resequencing-sections`
- [ ] Merge `foop-95-embryonic-resequencing-sections` to `jia`
  - [ ] Run all tests — old and new — and make sure they all pass correctly.
  - [ ] Check and make sure current foop has, and passes, its comprehensive
        snaptest (`einmo_suite/input/foop/95/comprehensive.foo`)
  - [ ] Run all tests — old and new — and make sure they all pass correctly.
  - [ ] (If complex merge situation: repair work sub-tasks land here, each
        timestamped)
  - [ ] STOP! STOP!! STOP!!! ASK HUMAN to check this box before continuing.
        UNDER NO CIRCUMSTANCES will Agent continue past this point
        automatically!!
    - [ ] Present human with the
          `cd /storage1/human/hcbusy/foolish/../foolish_worktrees/foop-95-embryonic-resequencing-sections`
          command and ask them to review snapshots BEFORE checking the
          parent checkbox.
  - [ ] Cleanup /storage1/human/hcbusy/foolish/../foolish_worktrees/foop-95-embryonic-resequencing-sections
    - [ ] Check that FOOP-95.plan.md has all but Cleanup checkboxes completed
    - [ ] Remove /storage1/human/hcbusy/foolish/../foolish_worktrees/foop-95-embryonic-resequencing-sections
          (`git worktree remove` + branch deletion after merge)
    - [ ] This is the last sub-task checkbox to be checked in this block of
          subtasks
- [ ] After merge: notify the [FOOP-65](FOOP-65.md) plan that its Phase-0
      dependency check can now resolve YES — the §5.3.1 backtick rendering
      can be confirmed at the einmo level

## Last Updated

**Date**: 2026-08-08
**Updated By**: Claude Code / claude-opus-5
**Changes**: Created alongside FOOP-95.md. Five phases: the
`stmt_count` purity split with individual call-site classification (1),
the read-only pre-step rendering path (2), the einmo section reorder to
`METADATA, OUTPUT, EMBRYONIC, INPUT, COMMENTS, STAMPS` plus adding
EMBRYONIC to `compare.rs`'s always-compared set (3), and the human-gated
inspection + corpus re-promotion behind Gate A (mechanical: OUTPUT/INPUT/
COMMENTS byte-identical) and Gate B (human: embryonic Foolish is
reasonable and useful) (4). Phase 0 enumerates `verified/` twins and
snapshots the corpus so Gate A can compare old-vs-new. Phase 5 adds the
**Foolish Resequencer** and **normalization** last (FOOP-95 §6): a separate
sequencer emitting parsable Foolish, checked by idempotence first then
fidelity against the normalized input, with the creation-identity limit
pinned by test and every fidelity failure triaged (resequencer bug /
normalizer bug / genuine parser information loss) rather than normalized
away.
