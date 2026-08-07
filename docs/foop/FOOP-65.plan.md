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

- [ ] Begin work: commit FOOP-65.md and FOOP-65.plan.md to origin (`jia`),
      check `begun: [x]` in FOOP-65.md frontmatter
- [ ] Create worktree at /storage1/human/hcbusy/foolish/../foolish_worktrees/foop-65-tail-concatenator with branch `foop-65-tail-concatenator`
      (`git worktree add -b foop-65-tail-concatenator /storage1/human/hcbusy/foolish/../foolish_worktrees/foop-65-tail-concatenator`
      from `jia` at /storage1/human/hcbusy/foolish; ALL subsequent work —
      including edits to FOOP-65.md and this plan — happens ONLY in the
      worktree until merge)
- [ ] Read `rust_instructions.md` in full (mandatory before any Rust;
      especially §"Phase-by-phase testing discipline")
- [ ] Read FOOP-65.md in full — especially §2 (precedence/associativity,
      the authoritative Atlas statements) and §5 (the separate-FIR
      directive: executes AS a concatenation, `value()` returns the inner)
- [ ] Read the touched sites: `foolish-parser/src/lexer.rs` (single-char
      arms, unknown-char fallback 297-299), `foolish-parser/src/parser.rs`
      (`parse_expr` 371-388, `is_concatenation_continuation` 390-411),
      `foolish-parser/src/ast.rs` + `token.rs`, `foolish-ubca/src/compiler.rs`
      (`build_fir` Concatenation arm), `foolish-ubca/src/fir_kinds.rs`
      (ConcatenationFir, `constanic_clone_at`)
- [ ] Baseline in the worktree: `RUSTUP_TOOLCHAIN=stable cargo test --workspace`
      and `cargo test -p foolish-ubca --lib -- run_einmo_tests` — both
      green before any change; re-confirm no backtick sits in CODE position
      in any einmo input (verified on `jia` @ `62706518`: comments only —
      `foop/13/comprehensive.foo:17-18,65`, `exercises/project_euler/1.foolish:15`)
- [ ] Run all tests — old and new — and make sure they all pass correctly.

## Phase 1 — Lexer, token, AST, parser (tests first)

- [ ] Write the failing parser/lexer tests FIRST (FOOP-65 Test Plan §unit):
      backtick token; `f`X` → `TailConcatenation [f, X]`; chain
      `f`g`h`X` → ONE flat node `[f, g, h, X]`; precedence pins
      (`fn`{a}{b}` groups the juxtaposition first; `fn`{a}$` keeps `$`
      inside the operand; search suffix on `(fn`X)`); backtick inside
      brane statements / parens / `<...>` / `<<...>>`; trailing backtick
      errors
- [ ] Implement: `Token::Backtick` (+ Display "`"), the lexer single-char
      arm, `Astn::TailConcatenation { elements }` (flat, source order,
      ≥ 2), the new weakest parse level per FOOP-65 §4 (current
      `parse_expr` body becomes the operand level; `Backtick` is NOT added
      to `is_concatenation_continuation`)
- [ ] Parser/lexer tests pass (`cargo test -p foolish-parser`)
- [ ] Confirm snapshot-invisible so far: no einmo input changes meaning —
      `cargo test -p foolish-ubca --lib -- run_einmo_tests` still green
- [ ] Run all tests — old and new — and make sure they all pass correctly.

## Phase 2 — `TailConcatenationFir` and the compiler arm (tests first)

- [ ] Write the failing FVM tests FIRST:
      `tail_concatenation_nyes_transitions` (REQUIRED by AGENTS.md —
      wrapper mirrors the inner concatenation's progression); equivalence
      `fn`X` ≡ `X fn` for several X (brane literal, search, concatenation —
      settled branes statement-for-statement identical); chain reversal
      `f`g`h`X` ≡ `X h g f`; system-operator application through the
      wrapper: `('lt`{1, 2})$` → `system.foo`'s `'True` BY IDENTITY (the
      FOOP-55 usage shape — proves recoordination through the wrapper)
- [ ] Implement per FOOP-65 §5: `TailConcatenationFir` (core with the
      inner ConcatenationFir as foolish_children[0]); `build_fir` arm for
      `Astn::TailConcatenation` (build operands, REVERSE, reuse the
      existing Concatenation machinery, wrap); `FirKind::TailConcatenation`;
      delegating `fir_op_step`; `value()` = inner's value;
      `constanic_clone_at` arm (clone inner through the existing
      ConcatenationFir recoordination, re-wrap); sequencer renders through
      the inner (add a foolish-core arm only if structurally required —
      verify, don't speculate)
- [ ] FVM tests pass (`cargo test -p foolish-ubca --lib`)
- [ ] Einmo inputs `foolish-ubca/einmo_suite/input/foop/65/tail_concat_basic.foo`
      (equivalence pairs side by side), `tail_concat_chain.foo` (flat
      chain), `tail_concat_system_ops.foo` (`'lt`/`'eq` via backtick);
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
- [ ] Present the foop/65 baselines + comprehensive OUTPUT to Atlas for
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
