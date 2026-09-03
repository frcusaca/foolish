# FOOP-56.plan — nyes-groups

**Read `docs/foop/FOOP-56.md` before executing this plan.**

**Worktree variables, expanded:**

```
WORKTREE_ORIGIN_BRANCH  = jia
WORKTREE_ORIGIN_PATH    = /yolo/foolish
WORKTREE_BRANCH_NAME    = foop-56-nyes-groups
WORKTREE_FULL_FS_PATH   = /yolo/foolish/../foolish_worktrees/foop-56-nyes-groups
```

## What this FOOP is, in one line

Give each of the four NYES groups a predicate, and qualify every bare "settled" with the group
it means. **Behaviour does not change** — every edit is a rename, a doc comment, or a
`matches!` replaced by an equivalent predicate call.

**The invariant that governs the whole plan:** `cargo test -p foolish-ubca2 --lib` is
**134/134 before and after**. Any test movement means something other than a rename happened;
find it and revert it. Do not "fix" a moved test.

## Scope guard

Changes are confined to:

- `foolish-ubca2/src/nyes_ext.rs` (the predicates)
- `foolish-ubca2/src/fvm_storage.rs` (renames, five call sites, doc comments)
- `foolish-ubca2/src/lib.rs`, `evaluator.rs`, `system_foo.rs` (call sites and one stale comment)
- `AGENTS.md`, and the two FOOP documents named in Phase 4

**Do not touch** `foolish-core/**` or `foolish-ubca/**`. `foolish_core::Nyes::is_nye()` stays
exactly as it is (§1: it has zero callers, so `NyesExt`'s method shadows nothing in practice).
If a change seems to require editing `foolish-core`, stop and report — it does not.

## Orientation — facts established while writing the spec

*Verified 2026-09-02. Confirm before relying on any of them, but do not re-derive from scratch.*

- `nyes_ext.rs` already has `is_constanic()`, `is_constantew()` and `is_nnk_constanic()`; its
  module doc says "Three categories" and omits conclusive.
- `foolish_core::Nyes::is_nye()` is at `fir.rs:143` and has **zero callers workspace-wide**.
- `is_settled()` **does not exist** anywhere, despite `lib.rs:24` and FOOP-62 §Terminology.
- "settled" appears **134 times** in `foolish-ubca2`, 131 of them in `fvm_storage.rs`.
- The five hand-rolled `matches!(…, Nyes::Constant | Nyes::Independent)` conclusive tests are at
  `fvm_storage.rs` **818, 2007, 3739, 3810, 3950**. Line **1375** matches the same two states
  but is a mapping arm, not a test — **leave it**.
- Commands:
  ```bash
  cargo test -p foolish-ubca2 --lib                        # must stay 134/134
  cargo test -p foolish-ubca  --lib -- einmo_gate_checked  # must not move
  cargo fmt --all
  cargo clippy -p foolish-ubca2 --all-targets -- -D warnings
  ```

---

## Phase 0 — Begin

- [ ] Begin work: commit `FOOP-56.md` and `FOOP-56.plan.md` to origin, check `begun: [x]` in
      the frontmatter
- [ ] Record the BEFORE state, and write the numbers into this plan:
  - [ ] `cargo test -p foolish-ubca2 --lib` — expect 134/134
  - [ ] `cargo test -p foolish-ubca --lib -- einmo_gate_checked` — expect pass
- [ ] Confirm FOOP-36 and FOOP-26 have **not** begun in a worktree. If either has, stop and ask
      the human — this FOOP is scheduled before both precisely to avoid colliding with them.
- [ ] Create worktree at `/yolo/foolish/../foolish_worktrees/foop-56-nyes-groups` with branch
      `foop-56-nyes-groups`:
      `git worktree add -b foop-56-nyes-groups /yolo/foolish/../foolish_worktrees/foop-56-nyes-groups`
- [ ] **All work from here happens in the worktree**, including edits to `docs/foop/`.

---

## Phase 1 — The four predicates

- [ ] (read §1 of `FOOP-56.md`)
- [ ] Establish relevant tests for this phase. Use [these instructions](../../README.md#running-specific-tests)
      to run unit tests: `foolish-ubca2::nyes_ext`. Run after each edit.
- [ ] Add to `NyesExt` in `foolish-ubca2/src/nyes_ext.rs`:
      ```rust
      /// Pre-constanic (nigh): PREMBRYONIC, EMBRYONIC, BRANING — still stepping.
      fn is_preconstanic(&self) -> bool {
          !self.is_constanic()
      }

      /// Not Yet Evaluated — the older name for the same group. An alias, kept so
      /// the traditional Foolish vocabulary still reads.
      fn is_nye(&self) -> bool {
          self.is_preconstanic()
      }

      /// Conclusive: the FIR reached a value — CONSTANT or INDEPENDENT.
      /// Distinct from `is_constantew()`, which also admits NK: NK is constant
      /// everywhere yet never produced a value.
      fn is_conclusive(&self) -> bool {
          matches!(self, Nyes::Constant | Nyes::Independent)
      }
      ```
      Declare each in the trait AND implement it in the `impl NyesExt for Nyes` block, matching
      the file's existing style.
- [ ] **T1 — predicate unit tests**, in the shape of the existing `constantew_states()`,
      asserting over `ALL_NYES`:
  - [ ] `conclusive_states()`
  - [ ] `preconstanic_states()`
  - [ ] `is_nye_is_alias_for_preconstanic()` — agree on every state
  - [ ] `conclusive_is_subset_of_constantew()`
  - [ ] `conclusive_and_constantew_differ_exactly_on_nk()` — **the load-bearing one**: the two
        cuts agree everywhere except NK, which is constantew but not conclusive
  - [ ] `preconstanic_is_complement_of_constanic()` — every state is in exactly one
- [ ] Update the module doc at the top of `nyes_ext.rs`: it says "Three categories" and omits
      conclusive. State all four groups and note `is_nye` as the alias.
- [ ] Run all tests — old and new — and make sure they all pass correctly. Still 134 plus the
      new unit tests.

---

## Phase 2 — Replace the five hand-rolled conclusive tests

- [ ] (read §3 of `FOOP-56.md`)
- [ ] Establish relevant tests for this phase. Use [these instructions](../../README.md#running-specific-tests)
      to run: `cargo test -p foolish-ubca2 --lib` (whole crate — a wrong replacement here
      changes behaviour, and the suite is what catches it).
- [ ] Replace with `.is_conclusive()`, **reading each one first** to confirm it is a conclusive
      *test* and not a coincidence of the same two states:
  - [ ] `fvm_storage.rs:818` (the `all_settled` binding — see Phase 3 for its rename)
  - [ ] `fvm_storage.rs:2007` (`is_constanic_non_brane` — note the existing name says
        "constanic" but the match is conclusive; rename it `is_conclusive_non_brane` while here)
  - [ ] `fvm_storage.rs:3739`
  - [ ] `fvm_storage.rs:3810`
  - [ ] `fvm_storage.rs:3950` (`empty_done`)
- [ ] **Leave `fvm_storage.rs:1375` alone** — it is a match arm in `nyes_from_found` mapping
      states to states, not a test. Confirm by reading it.
- [ ] **T2 — the untested distinction.** `operator_pushes_tasks_for_unsettled_operands` uses
      PREMBRYONIC operands only, so it never distinguishes conclusive from constanic. Add a
      case with an **ECONSTANIC** operand and assert it is still queued as a task — constanic,
      but not conclusive, so line 818's rule must still push it.
- [ ] Run all tests — old and new — and make sure they all pass correctly. **134/134 plus the
      new tests**; if any pre-existing test moved, a replacement was wrong.

---

## Phase 3 — Qualify every "settled"

- [ ] (read §2 and §2.1 of `FOOP-56.md`)
- [ ] Establish relevant tests for this phase. Use [these instructions](../../README.md#running-specific-tests)
      to run: `cargo test -p foolish-ubca2 --lib`. A rename that misses a call site fails to
      compile, which is the fast signal here.
- [ ] Renames in `foolish-ubca2/src/fvm_storage.rs` (plus their call sites):
  - [ ] `FirPointer::settled_result` → `settled_constanic_result` (~639). **Not
        `constantew`** — §2.1: the StayFoolish path admits ECONSTANIC/WOCONSTANIC.
  - [ ] `FirCursor::settled_result` → `settled_constanic_result` (~1602)
  - [ ] `step_to_settled` → `step_to_constanic` (~3272, its re-export ~4825, and its caller in
        `evaluator.rs`)
  - [ ] `all_settled` → `all_foolish_children_conclusive` (~816) — it iterates
        `foolish_children`, so name that rather than "operands"
  - [ ] `let settled = …decide_nyes_due_to_children(…)` → `let decided_nyes = …` (~1070). It
        can hold **BRANING**, so "settled" is wrong rather than merely vague.
  - [ ] `settled_nyes = nyes_from_found(…)` → `constanic_nyes` (~968)
  - [ ] `anchor_settled` → `anchor_constanic` (~3178)
- [ ] Test renames:
  - [ ] `indep_int_stepping_already_settled_is_noop` → `…_already_conclusive_…`
  - [ ] `operator_pushes_tasks_for_unsettled_operands` → `…_for_inconclusive_operands`
  - [ ] `revive_constanic_unwraps_stay_foolish_to_its_settled_result` →
        `…_settled_constanic_result`
- [ ] Doc comments: qualify each bare "settled" with its group. **Leave alone** any comment
      already unambiguous from context — this is clarification, not a sweep.
- [ ] Fix `lib.rs` line 24: it claims `NyesExt` "adds `is_settled()` to `Nyes`". No such method
      exists; describe what the trait actually provides.
- [ ] Run all tests — old and new — and make sure they all pass correctly. **Still 134/134 plus
      Phase 1–2's new tests.**

---

## Phase 4 — Documentation, and the two waiting FOOPs

- [ ] (read §4 of `FOOP-56.md`)
- [ ] `AGENTS.md` §Foolish Terminology: the four group entries already define the concepts.
      Add the **predicate name** to each (`is_preconstanic()` / `is_nye()`, `is_constanic()`,
      `is_constantew()`, `is_conclusive()`), so a reader moves from concept to call. Follow the
      Markdown File Update Protocol — **replace** the "## Last Updated" entry, do not append.
- [ ] **Update `FOOP-36.md` and `FOOP-36.plan.md`:**
  - [ ] Its §0.1.2 proposes exactly these predicates; replace that with a pointer to FOOP-56
        and state that the predicates exist.
  - [ ] Its §0.1/§0.1.1 survey of "settled" describes the OLD names. Update to the qualified
        names, keeping the survey (it is still the explanation of *why* they differ).
  - [ ] **Delete Phase 0.5 from `FOOP-36.plan.md`** — this FOOP is that phase, done properly.
        Note in its place that FOOP-56 landed first and the vocabulary is already in the code.
  - [ ] Check every §3 sentence still reads correctly against the now-real predicate names.
- [ ] **Update `FOOP-26.md`:** it reasons throughout about which children must be constanic
      before a step proceeds. Where it says "constanic" but means *conclusive* — a value was
      actually reached — say conclusive; where it means the terminal-state group, leave it.
      **Read each occurrence rather than sweeping**: this is a semantic distinction, and
      getting it wrong changes what FOOP-26 specifies. If any occurrence is genuinely
      ambiguous, list it for the human rather than guessing.
- [ ] `docs/foop/INDEX.md`: add FOOP-56, FOOP-36, FOOP-46 and FOOP-26 rows, and record the
      ordering **FOOP-56 → FOOP-36 → FOOP-26** in the Implementation Roadmap.
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 5 — Merge

- [ ] Verify all work is complete in `/yolo/foolish/../foolish_worktrees/foop-56-nyes-groups`
      and committed to `foop-56-nyes-groups`
- [ ] **T3 — behaviour unchanged:** `cargo test -p foolish-ubca2 --lib` is **134/134**, exactly
      matching Phase 0's before-reading, plus the new unit tests from Phases 1–2.
- [ ] **T4 — neighbours untouched:** `cargo test -p foolish-ubca --lib -- einmo_gate_checked`
      passes unchanged, and `git diff jia --stat` shows **no** changes under `foolish-core/` or
      `foolish-ubca/`.
- [ ] **No einmo baseline moved** in either crate. This FOOP renames; a moved baseline means
      behaviour changed — stop and find out why.
- [ ] `cargo fmt --all`; `cargo clippy -p foolish-ubca2 --all-targets -- -D warnings` clean.
      (`foolish-core/src/sequencer.rs` has 4 pre-existing clippy warnings — not this FOOP's,
      and the scope guard forbids touching that file. Scope the run to `-p foolish-ubca2`.)
- [ ] Merge `foop-56-nyes-groups` to `jia`
  - [ ] STOP! STOP!! STOP!!! ASK HUMAN to check this box before continuing. UNDER NO
        CIRCUMSTANCES will Agent continue past this point automatically!!
    - [ ] Present the human with
          `cd /yolo/foolish/../foolish_worktrees/foop-56-nyes-groups` and ask them to review
          BEFORE checking the parent checkbox. Say plainly that this is a rename-and-document
          change with **no behaviour delta**, that the suite is 134/134 either side, and that
          FOOP-36 and FOOP-26 were updated to use the new names.
  - [ ] Repair ALL tests in `jia` at `/yolo/foolish` if the merge broke any
- [ ] Cleanup `/yolo/foolish/../foolish_worktrees/foop-56-nyes-groups`
  - [ ] Check that this `.plan.md` has all but Cleanup checkboxes completed
  - [ ] Remove `/yolo/foolish/../foolish_worktrees/foop-56-nyes-groups`
  - [ ] This is the last sub-task checkbox to be checked in this block

---

## Last Updated

**Date**: 2026-09-02
**Updated By**: Claude Code / claude-opus-5
**Changes**: Created the FOOP-56 plan — five phases: add the four predicates and their unit
tests; replace the five hand-rolled conclusive `matches!` (and add the ECONSTANIC-operand test
the distinction currently lacks); qualify every bare "settled" with its group; update AGENTS.md,
FOOP-36, FOOP-26 and INDEX.md; merge. The governing invariant throughout is 134/134 before and
after — this FOOP changes no behaviour.
