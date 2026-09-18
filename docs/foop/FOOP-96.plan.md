# FOOP-96.plan — split-fvm-storage

**Read [`FOOP-96.md`](FOOP-96.md) in full before executing a single checkbox.** The plan assumes
the specification's context — in particular §0 ("what mechanical means"), §4 (the safety
argument), and the stop conditions.

---

## What this plan is, and the one rule that governs it

Every phase below moves **one block of text** out of `foolish-ubca2/src/fvm_storage.rs` into a
new file, and changes nothing else. The governing rule, from FOOP-96.md §0:

> **Move, never rewrite. No visibility widening. No behavior change bundled in.**

### The invariant, checked after every commit

| metric | required value |
|---|---|
| `cargo test --workspace` passed | **791** |
| failed | **0** |
| ignored | **1** (pre-existing `foolish-ubca` doctest) |
| `cargo fmt --all --check` | clean |
| `cargo clippy -p foolish-ubca2 --all-targets` | no NEW warnings |

**791 must not move in EITHER direction.** A test that vanishes is as bad as one that fails —
a `#[cfg(test)]` block that stops being compiled reports no error, it just stops running.

*Known, NOT this FOOP's to fix:* 4 pre-existing `foolish-core` clippy errors in `sequencer.rs`
break a workspace-wide `-D warnings` gate. Scope clippy to `-p foolish-ubca2`. A **new** warning
inside `foolish-ubca2` IS this FOOP's.

### Stop conditions — every phase

**STOP and report to the human, do not improvise**, if any of these occurs:

- A compile error naming a **private** item (`Slot`, `.payload`, `self.slots`, `validate`,
  or any private field) — the move has found a real coupling. **Do NOT widen visibility.**
- The test count is **not 791 passed / 0 failed**, in either direction.
- A **new** clippy warning appears inside `foolish-ubca2`.
- **Any einmo `checked/` baseline diverges.** That is a regression this FOOP introduced —
  fix the move. **NEVER `einmo promote`** (see "No Promotion Review Gate" below).

### No Promotion Review Gate — deliberately, not by omission

This FOOP **generates no einmo output and promotes nothing**. Every baseline must stay
**byte-identical**, so a promotion would be the precise signal that something broke. There is
likewise **no `comprehensive.foo`** — that test demonstrates a new feature interacting with old
ones, and this FOOP has no feature. See FOOP-96.md §Test Plan.

**An executing agent that finds itself wanting to run `einmo promote` has found a bug. Stop.**

### Dependency on FOOP-86 — and what changes if it slips

FOOP-86 (retire UBCa) is scheduled **before** this FOOP and deletes the ~650-line
`proto_to_core_fir` bridge. FOOP-96.md §3.1 is written so **either answer works**:

| if FOOP-86 has landed | if it has not |
|---|---|
| The bridge is gone. **Strike Phase 5** as `[-] not needed — FOOP-86 removed the bridge`. | **Execute Phase 5** normally. The bridge moves to its own file and FOOP-86 later deletes that file whole. |

Phase 0 determines which world this is. Nothing else in the plan changes either way.

### Worktree

```
WORKTREE_ORIGIN_BRANCH = jia
WORKTREE_ORIGIN_PATH   = /yolo/foolish
WORKTREE_BRANCH_NAME   = foop-96-split-fvm-storage
WORKTREE_FULL_FS_PATH  = /yolo/foolish/../foolish_worktrees/foop-96-split-fvm-storage
```

---

## Phase 0 — Begin, verify the premise, record the baseline

*Judgment phase — larger model (Opus/Sonnet). It decides whether the FOOP's premise still holds.*

- [ ] Read [`FOOP-96.md`](FOOP-96.md) in full — especially §0, §1 (the measured structure),
      §3 (target layout), §3.2 (the tests import hazard), §4 (the safety argument), §5.
- [ ] Begin work: commit `FOOP-96.md` and `FOOP-96.plan.md` to `jia`, check `begun: [x]` in the
      `FOOP-96.md` frontmatter.
- [ ] Create worktree at `/yolo/foolish/../foolish_worktrees/foop-96-split-fvm-storage` with
      branch `foop-96-split-fvm-storage`:
      `git worktree add -b "foop-96-split-fvm-storage" "/yolo/foolish/../foolish_worktrees/foop-96-split-fvm-storage"`
      **From here until merge, ALL work — including edits to `FOOP-96.md` and this plan — happens
      ONLY in the worktree.**
- [ ] **Determine whether FOOP-86 has landed.** Check whether `proto_to_core_fir` still exists:
      `grep -n "fn proto_to_core_fir" foolish-ubca2/src/fvm_storage.rs`
  - [ ] If **absent** → FOOP-86 landed. Strike Phase 5 below as
        `[-] not needed — FOOP-86 removed the bridge`, and note the date.
  - [ ] If **present** → execute Phase 5 normally.
- [ ] **Re-measure the block boundaries** (they shift if FOOP-86 landed).
      *Verify, don't re-derive* — FOOP-96.md §1 gives the values measured at `8c9043d8`:
      `grep -n "^mod search_engine\|^pub(crate) mod search_engine\|^mod search_fir_dispatch\|^mod core_fir_conversion\|^mod arena_compiler\|^mod tests\|^pub(crate) use" foolish-ubca2/src/fvm_storage.rs`
      Record the CURRENT line numbers in this plan before moving anything.
- [ ] **Re-verify safety property (2): ZERO private-internal reaches.** From the first inner
      module's line to EOF:
      `sed -n '<first_mod_line>,$p' foolish-ubca2/src/fvm_storage.rs | grep -c "self\.slots\|\.payload\|validate("`
      **Expected: `0`.** Any non-zero result means FOOP-96.md §4(2) no longer holds — **STOP and
      report** before moving any block.
- [ ] **Record the test baseline** in this plan: run `cargo test --workspace` and write down
      passed / failed / ignored. *Expected 791 / 0 / 1.* If it differs, the baseline has moved —
      record the NEW number and use it as the invariant for every phase below.
- [ ] Establish relevant tests for this FOOP. Use
      [these instructions](../../README.md#running-specific-tests). **Every phase of this FOOP
      uses the SAME set — the whole workspace — because a move can break anything:**
      `cargo test --workspace` (all 791), plus the einmo gates
      `foolish_ubca2::ubca_snapshot_tester2::einmo_tests::einmo_suite2_gate_checked` and
      `einmo_suite2_gate_verified` as the byte-identity oracle. There is no smaller meaningful
      subset for a file split, and the plan says so rather than inventing one.
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 1 — Move `mod search_engine` → `fvm_storage/search_engine.rs`

*Execution phase — smaller model. Fixed target: 377 lines move, 791 still pass.*

**The block** (verify against Phase 0's re-measurement): `fvm_storage.rs:2236–2612`, ~377 lines,
plus the doc comment immediately above the `mod` line, which moves WITH it.

**Its imports, verbatim** (*verify, don't re-derive*):
```rust
use super::{Equality, FVMStorage, FirCursor, FirPointer, default_equal};
use foolish_core::fir::Nyes;
use regex::Regex;
```
`use super::{…}` becomes `use super::{…}` unchanged — a child module's `super` is still
`fvm_storage`.

- [ ] Establish relevant tests for this sub-section: the whole workspace (see Phase 0's
      checkbox — a move can break anything). Run
      [these instructions](../../README.md#running-specific-tests) for `cargo test --workspace`
      and the einmo gates `einmo_suite2_gate_checked`, `einmo_suite2_gate_verified`.
- [ ] Create `foolish-ubca2/src/fvm_storage/` (the sibling directory; **no `mod.rs`** —
      `rust_instructions.md` §5).
- [ ] Move the block **as text** into `foolish-ubca2/src/fvm_storage/search_engine.rs`: cut lines
      `2236–2612` plus the preceding doc comment, strip ONE level of indentation, drop the
      `mod search_engine {` wrapper and its closing `}`. **Retype nothing.**
- [ ] Convert the module's `///` doc comment into a `//!` module-level doc at the top of the new
      file (`rust_instructions.md` §2d.3).
- [ ] In `fvm_storage.rs`, replace the removed block with the declaration, preserving visibility:
      `pub(crate) mod search_engine;`
- [ ] `cargo build -p foolish-ubca2` — **must compile with no new errors.**
      *A compile error naming a private item → STOP and report (do not widen visibility).*
- [ ] `cargo fmt --all` then `cargo fmt --all --check` — clean.
- [ ] `cargo clippy -p foolish-ubca2 --all-targets` — no NEW warnings.
- [ ] `cargo test --workspace` — **791 passed, 0 failed, 1 ignored.**
      *Any other number, in either direction → STOP and report.*
- [ ] Commit, alone: `Major: Split fvm_storage.rs, Phase: move search_engine--complete`
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 2 — Move `mod search_fir_dispatch` → `fvm_storage/search_fir_dispatch.rs`

*Execution phase — smaller model. ~744 lines.*

**Moved under its CURRENT name.** The rename to `search_dispatch.rs` happens in Phase 7 —
renames are behavior-adjacent and never bundled with a move (FOOP-96.md §0.3).

**Its imports, verbatim** (*verify, don't re-derive*):
```rust
use super::search_engine::{ /* … */ };
use super::{FVMStorage, FirCursor, FirPointer, FirSpec};
```

- [ ] Establish relevant tests for this sub-section: the whole workspace (Phase 0's set). Use
      [these instructions](../../README.md#running-specific-tests).
- [ ] Move the block **as text** into `foolish-ubca2/src/fvm_storage/search_fir_dispatch.rs`,
      doc comment included; strip one indent level; drop the `mod` wrapper.
- [ ] Convert the `///` module doc to `//!` at the top of the new file.
- [ ] In `fvm_storage.rs`: `mod search_fir_dispatch;` (preserve the original visibility).
- [ ] `cargo build -p foolish-ubca2` — compiles.
      *Private-item error → STOP and report.*
- [ ] `cargo fmt --all` + `--check`; `cargo clippy -p foolish-ubca2 --all-targets` — clean.
- [ ] `cargo test --workspace` — **791 / 0 / 1.**
- [ ] Commit, alone: `Major: Split fvm_storage.rs, Phase: move search_fir_dispatch--complete`
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 3 — Move `mod arena_compiler` → `fvm_storage/arena_compiler.rs`

*Execution phase — smaller model. ~801 lines.*

**Moved under its CURRENT name** (renamed to `compiler.rs` in Phase 7).

**Note the re-export**: `fvm_storage.rs` carries
`pub(crate) use arena_compiler::{compose_program_with_system, program_result};` — that line
**stays in `fvm_storage.rs` unchanged**; it re-exports from the now-external module and still
resolves.

**Its imports, verbatim** (*verify, don't re-derive*):
```rust
use super::{ANON_STMT_NAME, ConcatProvenance, ConcatRenderingAid, FVMStorage, FirCursor,
            FirCursorMut, FirPointer, FirSpec, StayMarker};
use foolish_core::fir::Nyes;
use foolish_parser::{AssignmentOperator, Astn, SearchOperator};
use crate::identifier::{Characterizations, Identifier};
```
**`use crate::…` lines move verbatim** — `crate` means the same thing in a child module.

- [ ] Establish relevant tests for this sub-section: the whole workspace (Phase 0's set). Use
      [these instructions](../../README.md#running-specific-tests).
- [ ] Move the block **as text** into `foolish-ubca2/src/fvm_storage/arena_compiler.rs`, doc
      comment included; strip one indent level; drop the `mod` wrapper.
- [ ] Convert the `///` module doc to `//!`.
- [ ] In `fvm_storage.rs`: `mod arena_compiler;` — and **leave the
      `pub(crate) use arena_compiler::{…}` re-export line exactly as it is.**
- [ ] `cargo build -p foolish-ubca2` — compiles. *Private-item error → STOP and report.*
- [ ] `cargo fmt --all` + `--check`; `cargo clippy -p foolish-ubca2 --all-targets` — clean.
- [ ] `cargo test --workspace` — **791 / 0 / 1.**
- [ ] Commit, alone: `Major: Split fvm_storage.rs, Phase: move arena_compiler--complete`
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 4 — Split the stepping driver out → `fvm_storage/stepping.rs`

*Execution phase — smaller model, but read the note below: this phase SPLITS a module rather than
moving one whole, so the boundary matters.*

**This is the one phase that divides an existing module.** `mod core_fir_conversion` bundles two
unrelated concerns (FOOP-96.md §2). The stepping driver comes out here; the bridge (if it still
exists) goes to its own file in Phase 5.

**The stepping driver** — `fvm_storage.rs:3395–3492` as measured at `8c9043d8`, ~100 lines:

| function | note |
|---|---|
| `const MAX_STEPS: usize = 10_000;` | **moves with the driver** — it is `step_to_constanic`'s budget |
| `step_to_constanic` | the production stepping loop |
| `step_until` | generic matcher breakpoint |
| `step_until_line_number` | breakpoint by line |
| `step_until_statement_name` | breakpoint by statement name |

**Stays behind with the bridge:** `fn display_stmt_name` — it is a *rendering* helper used by
`proto_to_core_fir_*`, **not** by the stepping driver. *Verify this before moving:*
`grep -n "display_stmt_name" foolish-ubca2/src/fvm_storage.rs` — every call site should be inside
the bridge family.

**Preserve the `#[cfg_attr(not(test), expect(dead_code, …))]` attributes** on `step_until_*`
verbatim — they have no production caller by design (they are the `foolish-debugging` skill's
entry points), and dropping them produces exactly the new clippy/rustc warning the stop
conditions name.

**The re-export line**
`pub(crate) use core_fir_conversion::{proto_to_core_fir, step_to_constanic};` must be updated:
`step_to_constanic` now comes from `stepping`. Split it into two lines (one per source module).

- [ ] Establish relevant tests for this sub-section: the whole workspace (Phase 0's set). Use
      [these instructions](../../README.md#running-specific-tests). **Pay particular attention to
      the `step_until*` unit tests** — `foolish_ubca2::fvm_storage::tests::step_until_*` and
      `step_to_constanic_settles_a_simple_fir`; they are the direct coverage of this block.
- [ ] Confirm the boundary: `display_stmt_name`'s callers are all in the bridge family
      (command above). *If a `step_until*` function calls it → STOP and report; the split
      boundary in FOOP-96.md §2 is wrong and needs the human.*
- [ ] Move `MAX_STEPS` + the four `step_*` functions **as text** into
      `foolish-ubca2/src/fvm_storage/stepping.rs`, with their doc comments and `#[cfg_attr]`
      attributes.
- [ ] Give the new file a `//!` module doc: the stepping loop and the `step_until*` breakpoints,
      naming the `foolish-debugging` skill as the consumer of the latter
      (`rust_instructions.md` §2d.3).
- [ ] Add `use super::{…}` to `stepping.rs` with **only** what the four functions actually need —
      a narrowed subset of `core_fir_conversion`'s list. Let the compiler tell you what is
      missing; **do not widen anything to make it resolve.**
- [ ] In `fvm_storage.rs`: add `mod stepping;` and update the re-export to
      `pub(crate) use stepping::step_to_constanic;` alongside the bridge's own re-export.
- [ ] Remove the now-moved functions from `core_fir_conversion`, and trim any import in its
      `use super::{…}` that only the stepping driver used. *The compiler flags unused imports —
      let it.*
- [ ] `cargo build -p foolish-ubca2` — compiles. *Private-item error → STOP and report.*
- [ ] `cargo fmt --all` + `--check`; `cargo clippy -p foolish-ubca2 --all-targets` — clean.
      *An unused-import or dead-code warning here is THIS phase's and must be fixed.*
- [ ] `cargo test --workspace` — **791 / 0 / 1.**
- [ ] Commit, alone: `Major: Split fvm_storage.rs, Phase: stepping driver to its own file--complete`
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 5 — Move the `proto_to_core_fir` bridge → `fvm_storage/core_fir_conversion.rs`

*Execution phase — smaller model. ~660 lines.*

> **CONDITIONAL — Phase 0 decides.** If FOOP-86 has already landed, the bridge does not exist:
> strike this whole phase as `[-] not needed — FOOP-86 removed the bridge`, note the date, and
> go to Phase 6. If the bridge is still present, execute normally.

**Rationale for giving it its own file even though FOOP-86 deletes it** (FOOP-96.md §3.1): the
cost is one text-move commit; the benefit lands either way. If 86 lands, deleting one file and
one `mod` line is strictly cheaper than excising 650 lines from a shared module. If 86 slips,
the core is ~660 lines smaller.

**The block**: `proto_to_core_fir`, `proto_to_core_fir_inner`, `proto_to_core_fir_sff_body`,
`proto_to_core_fir_sff_operand`, `anchor_to_core_fir`, `display_stmt_name` — everything left in
`core_fir_conversion` after Phase 4.

**Moved under its CURRENT name** (renamed to `core_fir_bridge.rs` in Phase 7).

- [ ] Establish relevant tests for this sub-section: the whole workspace (Phase 0's set). Use
      [these instructions](../../README.md#running-specific-tests). **The einmo gates matter most
      here** — the bridge feeds the old rendering path, so `einmo_suite2_gate_checked` is the
      direct byte-identity check on this move.
- [ ] Move the remaining `core_fir_conversion` body **as text** into
      `foolish-ubca2/src/fvm_storage/core_fir_conversion.rs`; strip one indent level; drop the
      `mod` wrapper.
- [ ] Convert the module `///` doc to `//!` — and **correct only the sentence that Phase 4
      falsified** (it currently opens "The stepping loop **and** the FIR→core-FIR
      output-serialization family"; the stepping loop is no longer here). Change nothing else in
      the doc (FOOP-96.md §Open Questions).
- [ ] In `fvm_storage.rs`: `mod core_fir_conversion;` and keep
      `pub(crate) use core_fir_conversion::proto_to_core_fir;`.
- [ ] `cargo build -p foolish-ubca2` — compiles. *Private-item error → STOP and report.*
- [ ] `cargo fmt --all` + `--check`; `cargo clippy -p foolish-ubca2 --all-targets` — clean.
- [ ] `cargo test --workspace` — **791 / 0 / 1.**
- [ ] Commit, alone: `Major: Split fvm_storage.rs, Phase: move core_fir bridge--complete`
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 6 — Move `mod tests` → `fvm_storage/tests.rs`

*Execution phase — smaller model, but this is the **largest and riskiest** block. Read
FOOP-96.md §3.2 first.*

**The block**: `fvm_storage.rs:4993–8282`, **3 290 lines, 110 `#[test]` functions** — 40% of the
file. Moved as ONE file; **not** split by subject (FOOP-96.md §Rejected Alternatives G).

**Why this is the riskiest phase, stated concretely.** The block opens with `use super::*`, and
then reaches its **sibling** modules by **bare path** — those paths resolve ONLY because the glob
pulled the sibling module names into scope:

| line (abs., at `8c9043d8`) | the import |
|---|---|
| 6242 | `use search_engine::{BraneNavigator, CandidateNavigator, MatchOutcome, ScanCtx, ScanOutcome, SearchPredicate, contextful_search_scan, contextful_search_scan_no_body_check};` |
| 6861 | `use core_fir_conversion::{proto_to_core_fir, step_to_constanic};` |
| 6981 | `use core_fir_conversion::{step_until, step_until_line_number, step_until_statement_name};` |
| 7068 | `use arena_compiler::compile;` |

**Two consequences, both already handled:**
- **`use super::*` keeps working.** A `#[cfg(test)] mod tests;` in a sibling file has the same
  `super` — `fvm_storage` — so all 235 `FirSpec` / 114 `FVMStorage` / 76 `FirCursor` / 21
  `revive_constanic` / … references resolve exactly as before. The move alone is safe.
- **Phase 4 already moved `step_until*` and `step_to_constanic`.** Line 6861's and 6981's
  imports were repointed in that phase. Confirm they still name the right modules here.

- [ ] Establish relevant tests for this sub-section: the whole workspace (Phase 0's set). Use
      [these instructions](../../README.md#running-specific-tests). **This phase's specific
      risk is the test COUNT**, not just pass/fail — see the dedicated checkbox below.
- [ ] **Record the pre-move `#[test]` count**:
      `grep -c "^\s*#\[test\]" foolish-ubca2/src/fvm_storage.rs` → *expected 110.*
- [ ] Move `mod tests`'s body **as text** into `foolish-ubca2/src/fvm_storage/tests.rs`; strip
      one indent level; drop the `#[cfg(test)] mod tests {` wrapper and its closing `}`.
      **Keep `use super::*;` as the first line** and keep every bare-path sibling import exactly
      as it is.
- [ ] In `fvm_storage.rs`, replace the removed block with:
      ```rust
      #[cfg(test)]
      mod tests;
      ```
- [ ] **Verify the test count moved intact**:
      `grep -c "^\s*#\[test\]" foolish-ubca2/src/fvm_storage/tests.rs` → **must be 110**, and
      `grep -c "^\s*#\[test\]" foolish-ubca2/src/fvm_storage.rs` → **must be 0**.
      *Any other numbers → STOP and report.*
- [ ] `cargo build -p foolish-ubca2 --all-targets` — compiles (note `--all-targets`: a plain
      `build` does not compile `#[cfg(test)]` code, so it would not catch a broken test import).
- [ ] `cargo fmt --all` + `--check`; `cargo clippy -p foolish-ubca2 --all-targets` — clean.
- [ ] `cargo test --workspace` — **791 / 0 / 1.**
      **This is the phase where a silent test-count drop is most likely.** Read the number; do
      not glance at "ok".
- [ ] Additionally confirm the `foolish-ubca2` lib total is unchanged: `cargo test -p foolish-ubca2
      --lib` → **184 passed** (its share of the 791, including all three einmo gates).
- [ ] Commit, alone: `Major: Split fvm_storage.rs, Phase: move tests--complete`
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 7 — The renames (behavior-adjacent; their own commits, after all moves)

*Execution phase — smaller model. Mechanical and compiler-verified: a missed site fails to build.*

All moves have landed. **Only now** do the modules get their final names (FOOP-96.md §3). Each
rename is its own commit — a rename touches every `use` site, and bundled with a move it destroys
the ability to say "this commit moved text and changed nothing."

- [ ] Establish relevant tests for this sub-section: the whole workspace (Phase 0's set). Use
      [these instructions](../../README.md#running-specific-tests).
- [ ] **Rename `search_fir_dispatch` → `search_dispatch`.**
      `git mv foolish-ubca2/src/fvm_storage/search_fir_dispatch.rs foolish-ubca2/src/fvm_storage/search_dispatch.rs`
      Update `mod` declaration and every `use` / path reference — **including the bare-path
      import inside `tests.rs`** if it names this module.
  - [ ] `cargo build -p foolish-ubca2 --all-targets`; `cargo test --workspace` — **791 / 0 / 1.**
  - [ ] Commit, alone: `Major: Split fvm_storage.rs, Phase: rename search_fir_dispatch to search_dispatch--complete`
- [ ] **Rename `arena_compiler` → `compiler`.**
      `git mv foolish-ubca2/src/fvm_storage/arena_compiler.rs foolish-ubca2/src/fvm_storage/compiler.rs`
      Update the `mod` declaration, the
      `pub(crate) use arena_compiler::{compose_program_with_system, program_result};` re-export,
      and `tests.rs:use arena_compiler::compile;`.
  - [ ] `cargo build -p foolish-ubca2 --all-targets`; `cargo test --workspace` — **791 / 0 / 1.**
  - [ ] Commit, alone: `Major: Split fvm_storage.rs, Phase: rename arena_compiler to compiler--complete`
- [ ] **Rename `core_fir_conversion` → `core_fir_bridge`** — *skip if Phase 5 was struck.*
      `git mv foolish-ubca2/src/fvm_storage/core_fir_conversion.rs foolish-ubca2/src/fvm_storage/core_fir_bridge.rs`
      Update the `mod` declaration, the `pub(crate) use` re-export, and `tests.rs`'s bare-path
      import of it.
  - [ ] `cargo build -p foolish-ubca2 --all-targets`; `cargo test --workspace` — **791 / 0 / 1.**
  - [ ] Commit, alone: `Major: Split fvm_storage.rs, Phase: rename core_fir_conversion to core_fir_bridge--complete`
- [ ] Update `foolish-ubca2/src/lib.rs`'s crate-level `//!` doc if it names any renamed module
      (`grep -n "core_fir_conversion\|arena_compiler\|search_fir_dispatch" foolish-ubca2/src/lib.rs`).
- [ ] **Sweep the repository for stale references to the old names** — docs included:
      `grep -rn "core_fir_conversion\|arena_compiler\|search_fir_dispatch" --include=*.rs --include=*.md .`
      Update the ones that are now wrong. **Do NOT edit completed FOOP plan files** — they are a
      historical record (`foop.md`). Do NOT edit `docs/foop/FOOP-26.md` or FOOP-86's files;
      they belong to other, concurrent work.
- [ ] `cargo fmt --all` + `--check`; `cargo clippy -p foolish-ubca2 --all-targets` — clean.
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 8 — Final review: assert it is a move

*Judgment phase — larger model. The deliverable is an assertion about the whole diff.*

- [ ] Establish relevant tests for this sub-section: the whole workspace (Phase 0's set). Use
      [these instructions](../../README.md#running-specific-tests).
- [ ] **Read the cumulative diff and assert it is a move.**
      `git diff jia...foop-96-split-fvm-storage --stat` then `-M --find-copies-harder` to let git
      detect the moves. **State, in the merge commit message, that no line of logic was retyped.**
      *If any hunk shows a logic change → it must be reverted or split into its own FOOP.*
- [ ] Confirm the final file sizes are roughly as FOOP-96.md §3 predicts; record the actuals in
      this plan. *A large discrepancy means a block did not move as expected — investigate.*
- [ ] `wc -l foolish-ubca2/src/fvm_storage.rs foolish-ubca2/src/fvm_storage/*.rs`
- [ ] Confirm **no `mod.rs` was created** (`rust_instructions.md` §5):
      `find foolish-ubca2/src -name mod.rs` → must be empty.
- [ ] Confirm **no visibility was widened**: review the diff for any `pub`/`pub(crate)` added to a
      previously-private item. *There should be NONE. Any one of them → report it to the human
      explicitly, even if the tests pass.*
- [ ] Confirm **no einmo baseline changed**: `git diff jia...foop-96-split-fvm-storage --stat --
      foolish-ubca2/einmo_suite*` → **must be empty.** A changed baseline is a regression
      (FOOP-96.md §Test Plan).
- [ ] Update `FOOP-96.md` frontmatter `status:` as appropriate and refresh its `## Last Updated`
      section (REPLACE the entry, do not append — AGENTS.md §Markdown File Update Protocol).
- [ ] **Accumulate and report ALL doubts in ONE statement** to the human — or record "no doubts"
      (AGENTS.md §"Accumulate doubts; report them once, at the end").
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 9 — Merge and cleanup

- [ ] Verify all work is complete in
      `/yolo/foolish/../foolish_worktrees/foop-96-split-fvm-storage` and committed to
      `foop-96-split-fvm-storage`.
- [ ] Merge `foop-96-split-fvm-storage` to `jia`
  - [ ] Run all tests — old and new — and make sure they all pass correctly.
  - [ ] **No comprehensive snapshot test is required for this FOOP.** It adds no feature, so
        there is nothing for `input/foop/96/comprehensive.foo` to demonstrate, and adding a new
        baseline would contradict this FOOP's contract that no baseline changes
        (FOOP-96.md §Test Plan). *This deviates from the usual merge checklist deliberately and
        is recorded here so the omission is visible rather than looking like an oversight.*
  - [ ] **Check for merge conflicts against concurrent work.** FOOP-26 and FOOP-46 edit this same
        file. If either landed on `jia` while this FOOP was in flight, the merge will conflict —
        resolve by **re-applying their changes into the new file layout**, never by discarding
        either side. *If the conflict is large, STOP and ask the human.*
  - [ ] Repair ALL tests in `jia` at `/yolo/foolish` after the merge — **791 / 0 / 1.**
  - [ ] STOP! STOP!! STOP!!! ASK HUMAN to check this box before continuing. UNDER NO
        CIRCUMSTANCES will Agent continue past this point automatically!!
    - [ ] Present the human with
          `cd /yolo/foolish/../foolish_worktrees/foop-96-split-fvm-storage` and ask them to
          review the diff BEFORE checking the parent checkbox. Remind them: *"Above message comes
          from FOOP-96, splitting `fvm_storage.rs` into one file per concern; the worktree is at
          /yolo/foolish/../foolish_worktrees/foop-96-split-fvm-storage. PTAL"*
  - [ ] Cleanup `/yolo/foolish/../foolish_worktrees/foop-96-split-fvm-storage`
    - [ ] Check that this `.plan.md` has all but Cleanup checkboxes completed
    - [ ] Remove `/yolo/foolish/../foolish_worktrees/foop-96-split-fvm-storage`
    - [ ] This is the last sub-task checkbox to be checked in this block

---

## Last Updated

**Date**: 2026-09-16
**Updated By**: Claude Code / claude-opus-5
**Changes**: Created the FOOP-96 plan — ten phases, one moved block per phase, each with its own
build/fmt/clippy/test gate asserting the **791 / 0 / 1** invariant in both directions. Phase 0 is
a judgment phase that re-measures boundaries, re-verifies the zero-private-reaches property, and
decides whether FOOP-86 has already removed the bridge (striking Phase 5 if so). Phases 1–6 move
blocks under their CURRENT names; Phase 7 does all renames separately, after the moves, because a
rename is behavior-adjacent. Phase 6 carries the `use super::*` / bare-path sibling-import hazard
and an explicit `#[test]`-count check (110). Records that there is **no Promotion Review Gate and
no `comprehensive.foo`** — deliberately, with the reason — rather than omitting them silently.
