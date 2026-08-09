# FOOP-65.plan — tail-concatenator

Read `FOOP-65.md` FIRST — this plan assumes the specification. Execute top
to bottom. Variables are already expanded to literals (per `foop.md`):

```
WORKTREE_ORIGIN_BRANCH=jia
WORKTREE_ORIGIN_PATH=/storage1/human/hcbusy/foolish
WORKTREE_BRANCH_NAME=foop-65-tail-concatenator
WORKTREE_FULL_FS_PATH=/storage1/human/hcbusy/foolish/../foolish_worktrees/foop-65-tail-concatenator
```

Note: this shell needs `RUSTUP_TOOLCHAIN=stable` exported (no default
toolchain configured on this machine). This FOOP is a prerequisite of
FOOP-55 (the Euler exercise rewrite) — it lands first and stands alone.

## Phase 0 — Begin, baseline, and orientation

- [x] **Dependency check — FOOP-65 `depends_on: [FOOP-95]`.** Determine
      whether [FOOP-95](FOOP-95.md) (the pre-step EMBRYONIC section +
      the `stmt_count` purity split) has merged.
      (2026-08-09 12:55)
      - FOOP-95 is still `Draft`, `begun: [ ]` — has NOT merged.
      - FOOP-65 proceeds: §1–§5 are fully testable without it (FOOP-65 §6).
        The §5.3.1 rendering will be unit-tested at the `foolish-core` level
        against a directly-constructed all-embryonic node. The einmo-level
        confirmation is deferred to when FOOP-95 lands.
- [x] Begin work: commit FOOP-65.md and FOOP-65.plan.md to origin (`jia`),
      check `begun: [x]` in FOOP-65.md frontmatter
      (2026-08-09 12:55)
      — `begun: [x]` already set from prior session; files already on `jia`.
- [x] Create worktree at /storage1/human/hcbusy/foolish/../foolish_worktrees/foop-65-tail-concatenator with branch `foop-65-tail-concatenator`
      (2026-08-09 12:55)
      — Rebranched from `jia` (`d95ef48e`): deleted stale branch, created
        fresh worktree. ALL subsequent work happens ONLY in the worktree.
- [x] Read `rust_instructions.md` in full (mandatory before any Rust;
      especially §"Phase-by-phase testing discipline")
      (2026-08-09 12:55)
- [x] Read FOOP-65.md in full — especially §2 (precedence/associativity,
      the authoritative statements), §3.1 (the worked example
      `` a`b`c`d e f `` → exactly TWO ConcatenationFirs), and §5 (the
      **flag-on-ConcatenationFir** design — NO separate FIR kind; the flag
      affects sequencing ONLY; precedence + reversal happen in `build_fir`)
      (2026-08-09 12:55)
- [x] Read the touched sites: `foolish-parser/src/lexer.rs` (single-char
      arms, unknown-char fallback 297-299), `foolish-parser/src/parser.rs`
      (`parse_expr` 513-530, `is_concatenation_continuation` 532-554),
      `foolish-parser/src/ast.rs` + `token.rs`, `foolish-ubca/src/compiler.rs`
      (`build_fir` Concatenation arm 371-383, `validate_astn` 167-172),
      `foolish-ubca/src/fir_kinds.rs` (`ConcatenationFir` 2634-2637,
      `fir_op_step` 2749-2835, `constanic_clone_at` Concatenation arm
      339-356), `foolish-ubca/src/evaluator.rs` (718-776),
      `foolish-core/src/fir.rs` (`ConcatenationFir` 356-360,
      `ConcatenationQuery` 563, builder 2096-2139),
      `foolish-core/src/sequencer.rs` (§9 concatenation, 496-545)
      (2026-08-09 12:55)
- [x] Baseline in the worktree: `RUSTUP_TOOLCHAIN=stable cargo test --workspace`
      and `cargo test -p foolish-ubca --lib -- run_einmo_tests` — both
      green before any change; re-confirm no backtick sits in CODE position
      in any einmo input (verified on `jia` @ `62706518`: comments only —
      `foop/13/comprehensive.foo:17-18,65`, `exercises/project_euler/1.foolish:15`)
      (2026-08-09 12:55)
      — 583 tests pass. einmo gates pass (single-threaded to avoid race).
- [x] Run all tests — old and new — and make sure they all pass correctly.
      (2026-08-09 12:55)

## Phase 1 — Lexer, token, AST, parser (tests first)

- [x] Write the failing parser/lexer tests FIRST (FOOP-65 Test Plan §unit):
      backtick token; `f`X` → `TailConcatenation [f, X]`; chain
      `f`g`h`X` → ONE flat node `[f, g, h, X]`; precedence pins
      (`fn`{a}{b}` groups the juxtaposition first; `fn`{a}$` keeps `$`
      inside the operand; search suffix on `(fn`X)`); **the §3.1 operand
      split: `` a`b`c`d e f `` parses to `TailConcatenation [a, b, c,
      Concatenation[d,e,f]]` — the trailing `d e f` is ONE operand**;
      backtick inside brane statements / parens / `<...>` / `<<...>>`;
      trailing backtick errors
      (2026-08-09 15:42)
- [x] Implement: `Token::Backtick` (+ Display "`"), the lexer single-char
      arm, `Astn::TailConcatenation { elements }` (flat, source order,
      ≥ 2), the new weakest parse level per FOOP-65 §4 (current
      `parse_expr` body becomes the operand level; `Backtick` is NOT added
      to `is_concatenation_continuation`)
      (2026-08-09 15:42)
- [x] Parser/lexer tests pass (`cargo test -p foolish-parser`)
      (2026-08-09 15:42)
      — 62 tests pass (12 new for backtick).
- [x] Confirm snapshot-invisible so far: no einmo input changes meaning —
      `cargo test -p foolish-ubca --lib -- run_einmo_tests` still green
      (2026-08-09 15:42)
      — einmo_gate_checked passes; no baseline changed.
- [x] Run all tests — old and new — and make sure they all pass correctly.
      (2026-08-09 15:42)

## Phase 2 — The provenance flag and the compiler arm (tests first)

NOTE (2026-08-08 revision): there is **NO new FIR kind**. Phase 2 adds one
field to the existing `ConcatenationFir`, one `build_fir` arm, and threads
the field to the sequencer. See FOOP-65 §5 — and do NOT add a `FirKind`
variant, a `constanic_clone_at` arm, or a `fir_op_step`.

- [ ] Write the failing FVM/compiler tests FIRST:
      **compiler shape (the §3.1 worked example)** — `` a`b`c`d e f ``
      builds EXACTLY two `ConcatenationFir`s: outer flagged
      `TailConcatenation` with elements `[Concat[Juxtaposition](d,e,f), c,
      b, a]`; **equivalence** `fn`X` ≡ `X fn` for several X (brane literal,
      search, concatenation — settled branes statement-for-statement
      identical); **chain reversal** `f`g`h`X` ≡ `X h g f`; **flag survives
      recoordination** (constanic-clone a tail-flagged concatenation, assert
      the clone's `provenance` is still `TailConcatenation`); **flag is
      evaluation-inert** (two `ConcatenationFir`s over the same elements
      differing only in `provenance` → identical settled NYES and identical
      joined statements); **system-operator application**
      `('lt`{1, 2})$` → `system.foo`'s `'True` BY IDENTITY (the FOOP-55
      usage shape — proves recoordination through the flagged node)
- [ ] **Extend** the existing ConcatenationFir NYES-transition test with a
      tail-flagged case asserting the flag changes NOTHING about the
      progression (AGENTS.md's `*_nyes_transitions` mandate applies as an
      extension, not a new test — no new FIR kind exists)
- [ ] Implement per FOOP-65 §5.1/§5.2: `ConcatProvenance` enum;
      `provenance` field on `ConcatenationFir` (`Juxtaposition` at every
      existing construction site); `build_fir` arm for
      `Astn::TailConcatenation` (build operands with the existing
      `build_concat_element`, **REVERSE**, set the flag) plus the
      `validate_astn` arm; copy `provenance` in the EXISTING
      `FirKind::Concatenation` arm of `constanic_clone_at` alongside
      `_helpers_populated`. **Do NOT touch `fir_op_step`,
      `populate_concat_helpers`, `stmt_count`, or `stmt_at`.**
- [ ] Thread the flag to the sequencer per §5.4: `ConcatenationQuery`
      (`foolish-core/src/fir.rs:563`), `ConcatenationFir` (356-360) and
      `ConcatenationFirBuilder` (2096-2136) gain the provenance with a
      `Juxtaposition` default; `foolish-ubca/src/evaluator.rs:706-764`
      passes it through
- [ ] Implement the sequencer branch (`foolish-core/src/sequencer.rs`
      §9, un-settled concatenation, 496-531) per FOOP-65 §5.3.1: a
      tail-flagged node renders in backtick form **ONLY while ALL its
      constituents are still embryonic**; once any has progressed, the
      ordinary rendering resumes. Pin BOTH sides with tests (all-embryonic
      → backtick; step one constituent → rendering flips). The SETTLED path
      renders the joined brane and must stay byte-identical to the
      juxtaposition equivalent.
- [ ] **Significant step — inspection of embryonic Foolish (FOOP-65 §6.1,
      FOOP-95 §4), by agent AND by human.** Render the all-embryonic
      `` a`b`c`d e f `` and judge it "reasonably informative for the
      purposes of future development, writing and maintaining Foolish
      programs": does it read as Foolish? is the operand grouping legible
      (`d e f` as ONE operand; `c,b,a` the reversed chain)? is backtick
      form genuinely more useful here than juxtaposition form? Record
      findings; PRESENT TO ATLAS and get sign-off before freezing the
      rendering into any baseline. The agent may NOT settle this alone.
- [ ] Sequencer tests pass (`cargo test -p foolish-core`) and FVM tests
      pass (`cargo test -p foolish-ubca --lib`)
- [ ] Einmo inputs `foolish-ubca/einmo_suite/input/foop/65/tail_concat_basic.foo`
      (equivalence pairs side by side), `tail_concat_chain.foo` (flat
      chain — MUST include the §3.1 example `` a`b`c`d e f `` beside its
      juxtaposition twin `d e f c b a`, settling identically),
      `tail_concat_system_ops.foo` (`'lt`/`'eq` via backtick);
      run `run_einmo_tests`, review OUTPUT, justify EVERY line (AGENTS.md
      step 4), promote ONLY these foop/65 baselines
- [ ] Run all tests — old and new — and make sure they all pass correctly.

## Phase 3 — Comprehensive test and final gate

- [ ] Write and verify `foolish-ubca/einmo_suite/input/foop/65/comprehensive.foo`
      (reserved name; mix the backtick with searches, `$`, SF/SFF markers,
      nested branes, system operators; at least one path through every
      behavior this FOOP adds; slight repetition acceptable for coverage)
- [ ] Run all tests — old and new — and make sure they all pass correctly.
- [ ] Verify ZERO foreign baseline divergence across the whole einmo suite
      (`einmo compare output checked foolish-ubca/einmo_suite/`) — any
      foreign divergence is a regression to fix, never to promote
      (`rust_instructions.md` §"Phase-by-phase testing discipline")
- [ ] Present the foop/65 baselines + comprehensive OUTPUT to the human for
      human review; checked-stage promotion only after approval; the
      verified stage requires the human key (einmo.toml leaves `verified`
      unconfigured on purpose)

## Merge

- [ ] Verify all work is complete in /storage1/human/hcbusy/foolish/../foolish_worktrees/foop-65-tail-concatenator and committed to `foop-65-tail-concatenator`
- [ ] Merge `foop-65-tail-concatenator` to `jia`
  - [ ] Run all tests — old and new — and make sure they all pass correctly.
  - [ ] Check and make sure current foop has, and passes, its comprehensive
        snaptest (`einmo_suite/input/foop/65/comprehensive.foo`)
  - [ ] Run all tests — old and new — and make sure they all pass correctly.
  - [ ] (If complex merge situation: repair work sub-tasks land here, each
        timestamped)
  - [ ] STOP! STOP!! STOP!!! ASK HUMAN to check this box before continuing.
        UNDER NO CIRCUMSTANCES will Agent continue past this point
        automatically!!
    - [ ] Present human with the
          `cd /storage1/human/hcbusy/foolish/../foolish_worktrees/foop-65-tail-concatenator`
          command and ask them to review snapshots BEFORE checking the
          parent checkbox.
  - [ ] Cleanup /storage1/human/hcbusy/foolish/../foolish_worktrees/foop-65-tail-concatenator
    - [ ] Check that FOOP-65.plan.md has all but Cleanup checkboxes completed
    - [ ] Remove /storage1/human/hcbusy/foolish/../foolish_worktrees/foop-65-tail-concatenator
          (`git worktree remove` + branch deletion after merge)
    - [ ] This is the last sub-task checkbox to be checked in this block of
          subtasks
- [ ] After merge: notify the FOOP-55 plan that its Phase-0 FOOP-65 gate
      can now be checked (FOOP-55 Phase 0 verifies FOOP-65 is merged and
      the exercise is rewritten in backtick form)
- [ ] After merge: correct FOOP-75 §9's comparison table — it records
      FOOP-65 as "New FIR? yes — `TailConcatenationFir`" and "render
      through the inner concatenation", both stale under the flag design
      (FOOP-65 §5, Open Questions). Nothing in FOOP-75's own design changes.

## Last Updated

**Date**: 2026-08-08
**Updated By**: Claude Code / claude-opus-5
**Changes**: Realigned the plan with FOOP-65's 2026-08-08 architecture
revision (separate `TailConcatenationFir` → provenance flag on the existing
`ConcatenationFir`). Phase 2 retitled and rewritten: no new FIR kind, no
`FirKind` variant, no `constanic_clone_at` arm, no `fir_op_step` — instead
one `ConcatProvenance` field, one `build_fir` arm (operands reversed at
compile time), the flag copied in the existing Concatenation clone arm,
threaded through `ConcatenationQuery` to a single sequencer branch, plus
new tests for compiler shape (§3.1), flag-survives-recoordination, and
flag-is-evaluation-inert; the NYES-transition item became an EXTENSION of
the existing concatenation test. Phase 0 orientation and Phase 1 parser
tests gained the §3.1 worked example and the widened code-anchor list; a
post-merge item was added to correct FOOP-75 §9's now-stale table.
Later the same day: the sequencer item was tightened to FOOP-65 §5.3.1
(backtick form ONLY while all constituents are embryonic, with both sides
pinned by tests); a **significant inspection step** was added — embryonic
Foolish reviewed by agent AND human for informativeness "for the purposes
of future development, writing and maintaining Foolish programs", with
human sign-off required before freezing the rendering; and Phase 0 gained
a **dependency check on [FOOP-95](FOOP-95.md)**, which now owns the
pre-step EMBRYONIC rendering and the `stmt_count` purity split (too large
to ride along in this FOOP).
