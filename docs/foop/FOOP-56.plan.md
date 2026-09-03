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

- [x] Begin work: commit `FOOP-56.md` and `FOOP-56.plan.md` to origin, check `begun: [x]` in
      the frontmatter
      (2026-09-02 18:21)
- [x] Record the BEFORE state, and write the numbers into this plan:
      (2026-09-02 18:21)
  - [x] `cargo test -p foolish-ubca2 --lib` — expect 134/134. **Actual: 134 passed; 0 failed.**
        (2026-09-02 18:21)
  - [x] `cargo test -p foolish-ubca --lib -- einmo_gate_checked` — expect pass. **Actual: 1
        passed; 0 failed.**
        (2026-09-02 18:21)
- [x] Confirm FOOP-36 and FOOP-26 have **not** begun in a worktree. If either has, stop and ask
      the human — this FOOP is scheduled before both precisely to avoid colliding with them.
      **Confirmed: both `begun: [ ]`, no worktrees for either exist (only a pre-existing,
      unrelated foop-55 worktree).**
      (2026-09-02 18:21)
- [x] Create worktree at `/yolo/foolish/../foolish_worktrees/foop-56-nyes-groups` with branch
      `foop-56-nyes-groups`:
      `git worktree add -b foop-56-nyes-groups /yolo/foolish/../foolish_worktrees/foop-56-nyes-groups`
      (2026-09-02 18:21)
- [x] **All work from here happens in the worktree**, including edits to `docs/foop/`.
      (2026-09-02 18:21)

---

## Phase 1 — The four predicates

- [x] (read §1 of `FOOP-56.md`)
      (2026-09-02 18:21)
- [x] Establish relevant tests for this phase. Use [these instructions](../../README.md#running-specific-tests)
      to run unit tests: `foolish-ubca2::nyes_ext`. Run after each edit.
      (2026-09-02 18:21)
- [x] Add to `NyesExt` in `foolish-ubca2/src/nyes_ext.rs`:
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
      **Deviation from the literal spec snippet:** `is_preconstanic` and `is_nye` are default
      trait methods exactly as shown (they only call `self.is_constanic()`, generic over
      `Self`). `is_conclusive`'s body pattern-matches concrete `Nyes` variants
      (`Nyes::Constant | Nyes::Independent`), which does not type-check as a default method on
      a generic `Self` (E0308: expected type parameter `Self`, found `Nyes`) — moved its body
      into the `impl NyesExt for Nyes` block instead, alongside `is_constanic`/`is_constantew`,
      which already do the same for the same reason. The trait declares only its signature.
      Behaviour is identical to the spec snippet; only where the body lives differs.
      (2026-09-02 18:21)
- [x] **T1 — predicate unit tests**, in the shape of the existing `constantew_states()`,
      asserting over `ALL_NYES`:
      (2026-09-02 18:21)
  - [x] `conclusive_states()`
        (2026-09-02 18:21)
  - [x] `preconstanic_states()`
        (2026-09-02 18:21)
  - [x] `is_nye_is_alias_for_preconstanic()` — agree on every state
        (2026-09-02 18:21)
  - [x] `conclusive_is_subset_of_constantew()`
        (2026-09-02 18:21)
  - [x] `conclusive_and_constantew_differ_exactly_on_nk()` — **the load-bearing one**: the two
        cuts agree everywhere except NK, which is constantew but not conclusive
        (2026-09-02 18:21)
  - [x] `preconstanic_is_complement_of_constanic()` — every state is in exactly one
        (2026-09-02 18:21)
- [x] Update the module doc at the top of `nyes_ext.rs`: it says "Three categories" and omits
      conclusive. State all four groups and note `is_nye` as the alias.
      (2026-09-02 18:21)
- [x] Run all tests — old and new — and make sure they all pass correctly. Still 134 plus the
      new unit tests. **Actual: 140 passed; 0 failed** (`cargo test -p foolish-ubca2 --lib`).
      One run mid-phase hit a stale "catastrophe crumb" (`output-error` status) left in
      `einmo_suite/output/` by an earlier interrupted tool call — 6 files, all under
      `output/` (scratch, regenerated data; nothing under `checked/`/`verified/` was touched).
      Restored with `git checkout -- foolish-ubca2/einmo_suite/output/` and reran clean.
      (2026-09-02 18:21)

---

## Phase 2 — Replace the five hand-rolled conclusive tests

- [x] (read §3 of `FOOP-56.md`)
      (2026-09-02 18:21)
- [x] Establish relevant tests for this phase. Use [these instructions](../../README.md#running-specific-tests)
      to run: `cargo test -p foolish-ubca2 --lib` (whole crate — a wrong replacement here
      changes behaviour, and the suite is what catches it).
      (2026-09-02 18:21)
- [x] Replace with `.is_conclusive()`, **reading each one first** to confirm it is a conclusive
      *test* and not a coincidence of the same two states:
      (2026-09-02 18:21)
  - [x] `fvm_storage.rs:818` (the `all_settled` binding — see Phase 3 for its rename)
        (2026-09-02 18:21)
  - [x] `fvm_storage.rs:2007` (`is_constanic_non_brane` — note the existing name says
        "constanic" but the match is conclusive; rename it `is_conclusive_non_brane` while here)
        (2026-09-02 18:21)
  - [x] `fvm_storage.rs:3739`
        (2026-09-02 18:21)
  - [x] `fvm_storage.rs:3810`
        (2026-09-02 18:21)
  - [x] `fvm_storage.rs:3950` (`empty_done`)
        (2026-09-02 18:21)
- [x] **Leave `fvm_storage.rs:1375` alone** — it is a match arm in `nyes_from_found` mapping
      states to states, not a test. Confirm by reading it.
      **Confirmed**: it remaps a found node's Nyes to a caller settlement value
      (Econstanic/Woconstanic → Woconstanic; Constant/Independent → Constant; Nk → Nk), not a
      conditional test — left unchanged.
      (2026-09-02 18:21)
- [x] **T2 — the untested distinction.** `operator_pushes_tasks_for_unsettled_operands` uses
      PREMBRYONIC operands only, so it never distinguishes conclusive from constanic. Add a
      case with an **ECONSTANIC** operand and assert it is still queued as a task — constanic,
      but not conclusive, so line 818's rule must still push it.
      Added `operator_pushes_tasks_for_econstanic_operand` in `fvm_storage.rs`'s tests module,
      immediately after the PREMBRYONIC-only test it complements.
      (2026-09-02 18:21)
- [x] Run all tests — old and new — and make sure they all pass correctly. **134/134 plus the
      new tests**; if any pre-existing test moved, a replacement was wrong.
      **Actual: 141 passed; 0 failed** (140 from Phase 1 + this phase's 1 new T2 test). Also
      required adding `NyesExt` to `mod core_fir_conversion`'s `use super::{...}` list (that
      submodule imports by explicit name, not glob, so the file-level `use` at line 15 did not
      reach lines 3739/3810/3950 inside it).
      (2026-09-02 18:21)

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

## Phase 4 — Bring ALL ubca2 documentation onto the vocabulary

*The predicates and renames of Phases 1–3 are only half the job. The other half is that every
document describing `foolish-ubca2` should say which NYES group it means, so the next agent
reads one vocabulary rather than inferring from context. **This phase is uniform**: the same
pass over each ubca2-related FOOP, not a special case per document.*

**The pass, applied to each document below:**

1. **Every bare "settled"** gains its group adjective — "settled constanic", "conclusive",
   "constantew" — or is left alone where context already makes it unambiguous. Do not sweep for
   its own sake.
2. **Every "constanic" that actually means *conclusive*** — where the point is that a value was
   reached, not merely that a terminal state was — is corrected. **This is a semantic
   distinction; read each occurrence.** Getting it wrong changes what the document specifies.
3. **Renamed identifiers** (`settled_result` → `settled_constanic_result`, `all_settled` →
   `all_foolish_children_conclusive`, `step_to_settled` → `step_to_constanic`, and the rest of
   §2) are updated wherever the document names them.
4. **Where a document hand-rolls a state list** (`CONSTANT or INDEPENDENT`, `ECONSTANIC,
   WOCONSTANIC, NK`) in prose, name the group instead and keep the list only where the
   enumeration is the point.
5. **Anything genuinely ambiguous is listed for the human**, not guessed. One consolidated list
   at the end of the phase (AGENTS.md §"Accumulate doubts; report them once").

- [ ] (read §2 and §4 of `FOOP-56.md`)
- [ ] Establish relevant tests for this phase: none — this phase edits documentation only. Run
      `cargo test -p foolish-ubca2 --lib` once at the end to confirm nothing was touched by
      accident (still 134/134 plus Phases 1–2's new tests).

### 4a — `AGENTS.md`, the authority

- [ ] §Foolish Terminology already defines all four groups. Add the **predicate name** to each
      entry (`is_preconstanic()` / `is_nye()`, `is_constanic()`, `is_constantew()`,
      `is_conclusive()`) so a reader moves from concept to call.
- [ ] Note that `is_settled()` does **not** exist, so no one goes looking for it.
- [ ] Markdown File Update Protocol: **replace** the "## Last Updated" entry, do not append.

### 4b — The three ubca2 FOOPs (uniform pass)

*All three describe `foolish-ubca2`. Apply the five-step pass above to each. Counts are
starting points measured 2026-09-02, not targets — some occurrences will rightly stay.*

- [ ] **`FOOP-16.md`** (8 "settled", 28 "constanic") and **`FOOP-16.plan.md`** (30 "settled") —
      the FOOP that built `foolish-ubca2`. Its plan is largely complete, so **prefer leaving
      completed checkboxes as the historical record they are** (`foop.md`: completed plan files
      are not rewritten). Update the spec; touch the plan only where a name it references no
      longer exists in the code.
- [ ] **`FOOP-26.md`** (21 "settled", 87 "constanic") — marks, concatenation-as-operator, the
      three-beat step. **The heaviest of the three and the one where step 2 matters most**: it
      reasons throughout about which children must be constanic before a step proceeds, and
      several of those are conclusive tests. Its "wait for `foolish_children` to become
      constanic" beat is exactly the kind of sentence to read closely — `fvm_storage.rs:818`
      gates on conclusive, so if the spec means that rule, it should say conclusive.
      **This FOOP has an active author**: check with the human before editing if it is being
      worked on concurrently.
- [ ] **`FOOP-46.md`** (9 "settled", 29 "constanic") — BraneConcatOp. Its §"A constituent that
      is not constanic is not ready" reasoning is the passage to read against step 2.
      Spec only; it has no plan yet.
- [ ] **`FOOP-36.md`** and **`FOOP-36.plan.md`** — the sequencer FOOP this one was extracted
      from:
  - [ ] §0.1.2 already points here; confirm it matches what Phases 1–3 actually built.
  - [ ] §0.1/§0.1.1's survey of "settled" describes the OLD names. Update to the qualified
        ones, **keeping the survey** — it is still the explanation of *why* the groups differ.
  - [ ] Phase 0.5 in its plan is already reduced to "confirm FOOP-56 landed"; verify.
  - [ ] Check every §3 sentence reads correctly against the now-real predicate names.

### 4c — Sweep for stragglers

- [ ] `grep -rn "settled" docs/foop/FOOP-16* docs/foop/FOOP-26.md docs/foop/FOOP-36* docs/foop/FOOP-46.md`
      and confirm each remaining occurrence is deliberate — qualified, or unambiguous in
      context. Record the count you are leaving and why.
- [ ] Check `foolish-ubca2/einmo_suite/MAPPING.md` and the crate's own `lib.rs` module docs for
      the same issue.
- [ ] **Deliberately NOT in scope:** FOOP-62, FOOP-23, FOOP-33, FOOP-55 and the other
      `foolish-ubca` (not ubca2) FOOPs. They are shipped or describe the sibling crate;
      rewriting them is a larger act than this FOOP claims. **FOOP-62 §Terminology is the
      origin of `constanic`/`constantew`** and lists `is_settled()`, which was never
      implemented — note the discrepancy (§4), do not amend a shipped FOOP.
- [ ] `docs/foop/INDEX.md`: confirm the FOOP-26/36/46/56 rows and Track 6's
      **56 → 36 → 26** ordering are present and accurate. (Added on `jia` 2026-09-02.)

### 4d — Report

- [ ] **One consolidated report to the human**: every occurrence that was genuinely ambiguous
      between constanic and conclusive, with the document, the line, and what each reading
      would mean. A "constanic" that should be "conclusive" in FOOP-26 or FOOP-46 is a **spec
      correction**, not a wording tweak — the human should see the list before it is treated as
      settled. If there were none, say so plainly.
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
**Changes**: Rewrote Phase 4 as a **uniform documentation pass** over all four ubca2 FOOPs
(16, 26, 36, 46) plus AGENTS.md and the crate's module docs, with the five-step pass stated
once and applied to each, a straggler sweep, an explicit out-of-scope list, and a consolidated
ambiguity report to the human. Prior entry: created the FOOP-56 plan — five phases: add the four predicates and their unit
tests; replace the five hand-rolled conclusive `matches!` (and add the ECONSTANIC-operand test
the distinction currently lacks); qualify every bare "settled" with its group; update AGENTS.md,
FOOP-36, FOOP-26 and INDEX.md; merge. The governing invariant throughout is 134/134 before and
after — this FOOP changes no behaviour.
