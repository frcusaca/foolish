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
| `cargo test --workspace` passed | **467** |
| failed | **0** |
| ignored | **0** (the old `foolish-ubca` doctest went with that crate, FOOP-86) |
| `cargo fmt --all --check` | clean |
| `cargo clippy -p foolish-ubca2 --all-targets` | no NEW warnings |

**467 must not move in EITHER direction.** A test that vanishes is as bad as one that fails —
a `#[cfg(test)]` block that stops being compiled reports no error, it just stops running.

*Known, NOT this FOOP's to fix:* 4 pre-existing `foolish-core` clippy errors in `sequencer.rs`
break a workspace-wide `-D warnings` gate. Scope clippy to `-p foolish-ubca2`. A **new** warning
inside `foolish-ubca2` IS this FOOP's.

### Stop conditions — every phase

**STOP and report to the human, do not improvise**, if any of these occurs:

- A compile error naming a **private** item (`Slot`, `.payload`, `self.slots`, `validate`,
  or any private field) — the move has found a real coupling. **Do NOT widen visibility.**
- The test count is **not 467 passed / 0 failed**, in either direction.
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

**RESOLVED 2026-09-24: FOOP-86 landed (merged 2026-09-21).** The left column is the world you
are in. Phase 5 is already struck in this plan; nothing else changes.

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
- [x] **Determine whether FOOP-86 has landed.** — **RESOLVED 2026-09-24, before execution.**
      FOOP-86 merged to `jia` on 2026-09-21. Verified:
      `grep -c "fn proto_to_core_fir" foolish-ubca2/src/fvm_storage.rs` → **0**.
      **Phase 5 is struck** and Phase 7's dependent `core_fir_bridge` rename with it. Re-run the
      grep to confirm for yourself, but do not re-litigate the branch — it is decided.
- [ ] **Re-measure the block boundaries** (they shift if FOOP-86 landed).
      *Verify, don't re-derive* — FOOP-96.md §1 gives the values RE-MEASURED at `0df1865b`
      (2026-09-24, post-FOOP-86). The 2026-09-16 `8c9043d8` figures are stale in EVERY row:
      `grep -n "^mod search_engine\|^pub(crate) mod search_engine\|^mod search_fir_dispatch\|^mod core_fir_conversion\|^mod arena_compiler\|^mod tests\|^pub(crate) use" foolish-ubca2/src/fvm_storage.rs`
      Record the CURRENT line numbers in this plan before moving anything.
- [ ] **Re-verify safety property (2): ZERO private-internal reaches.** From the first inner
      module's line to EOF:
      `sed -n '<first_mod_line>,$p' foolish-ubca2/src/fvm_storage.rs | grep -c "self\.slots\|\.payload\|validate("`
      **Expected: `0`.** Any non-zero result means FOOP-96.md §4(2) no longer holds — **STOP and
      report** before moving any block.
- [ ] **Record the test baseline** in this plan: run `cargo test --workspace` and write down
      passed / failed / ignored. *Expected 467 / 0 / 0.* If it differs, the baseline has moved —
      record the NEW number and use it as the invariant for every phase below.
- [ ] Establish relevant tests for this FOOP. Use
      [these instructions](../../README.md#running-specific-tests). **Every phase of this FOOP
      uses the SAME set — the whole workspace — because a move can break anything:**
      `cargo test --workspace -- --test-threads=1` (all 467), plus the einmo gates
      `einmo_gates::einmo_tests::einmo_gate_checked` and `einmo_gate_verified` as the
      byte-identity oracle. There is no smaller meaningful subset for a file split, and the plan
      says so rather than inventing one.

      **CORRECTED 2026-09-24.** This previously named
      `ubca_snapshot_tester2::einmo_tests::einmo_suite2_gate_checked` — a module and test name
      that FOOP-86 RETIRED along with `einmo_suite2`. Neither exists; running them matches
      nothing and reports success, so an executor would believe the byte-identity oracle passed
      when it never ran. Verified: `grep -rn "einmo_suite2_gate_checked" foolish-ubca2/src/`
      returns nothing.

      **Always pass `-- --test-threads=1`.** The three einmo gates share
      `foolish-ubca2/einmo_suite/output/` and corrupt each other when run in parallel, producing
      spurious `status: output-error` "catastrophe crumb" failures that look exactly like a
      regression you caused. If you see one: `git checkout -- foolish-ubca2/einmo_suite/output/`
      and re-run serially before concluding anything.
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 1 — Move `mod search_engine` → `fvm_storage/search_engine.rs`

*Execution phase — smaller model. Fixed target: 371 lines move, 467 still pass.*

**The block** (verify against Phase 0's re-measurement): `fvm_storage.rs:2237–2607`, ~371 lines,
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
      and the einmo gates `einmo_gate_checked`, `einmo_gate_verified` (run with
      `-- --test-threads=1`; see Phase 0).
- [ ] Create `foolish-ubca2/src/fvm_storage/` (the sibling directory; **no `mod.rs`** —
      `rust_instructions.md` §5).
- [ ] Move the block **as text** into `foolish-ubca2/src/fvm_storage/search_engine.rs`: cut lines
      `2237–2607` plus the preceding doc comment, strip ONE level of indentation, drop the
      `mod search_engine {` wrapper and its closing `}`. **Retype nothing.**
- [ ] Convert the module's `///` doc comment into a `//!` module-level doc at the top of the new
      file (`rust_instructions.md` §2d.3).
- [ ] In `fvm_storage.rs`, replace the removed block with the declaration, preserving visibility:
      `pub(crate) mod search_engine;`
- [ ] `cargo build -p foolish-ubca2` — **must compile with no new errors.**
      *A compile error naming a private item → STOP and report (do not widen visibility).*
- [ ] `cargo fmt --all` then `cargo fmt --all --check` — clean.
- [ ] `cargo clippy -p foolish-ubca2 --all-targets` — no NEW warnings.
- [ ] `cargo test --workspace` — **467 passed, 0 failed, 0 ignored.**
      *Any other number, in either direction → STOP and report.*
- [ ] Commit, alone: `Major: Split fvm_storage.rs, Phase: move search_engine--complete`
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 2 — Move `mod search_fir_dispatch` → `fvm_storage/search_fir_dispatch.rs`

*Execution phase — smaller model. ~860 lines.*

**Moved under its CURRENT name.** The rename to `search_dispatch.rs` happens in Phase 7 —
renames are behavior-adjacent and never bundled with a move (FOOP-96.md §0.3).

**Its imports, verbatim** (*verify, don't re-derive*):
```rust
use super::search_engine::{
    BraneNavigator, ScanOutcome, SearchPredicate, contextful_search_scan,
    contextful_search_scan_no_body_check,
};
use super::{FVMStorage, FirCursor, FirPointer, FirSpec};

use foolish_core::fir::Nyes;
```
**Verbatim as of `ef7fc880`.** The earlier draft elided the `search_engine::{…}` list as
`/* … */` and **omitted `use foolish_core::fir::Nyes;` entirely** — copying that block as
written produces an unresolved-`Nyes` compile error. Still verify against the file rather than
trusting this listing; that is the standing *verify, don't re-derive* rule.

- [ ] Establish relevant tests for this sub-section: the whole workspace (Phase 0's set). Use
      [these instructions](../../README.md#running-specific-tests).
- [ ] Move the block **as text** into `foolish-ubca2/src/fvm_storage/search_fir_dispatch.rs`,
      doc comment included; strip one indent level; drop the `mod` wrapper.
- [ ] Convert the `///` module doc to `//!` at the top of the new file.
- [ ] In `fvm_storage.rs`: `mod search_fir_dispatch;` (preserve the original visibility).
- [ ] `cargo build -p foolish-ubca2` — compiles.
      *Private-item error → STOP and report.*
- [ ] `cargo fmt --all` + `--check`; `cargo clippy -p foolish-ubca2 --all-targets` — clean.
- [ ] `cargo test --workspace` — **467 / 0 / 0.**
- [ ] Commit, alone: `Major: Split fvm_storage.rs, Phase: move search_fir_dispatch--complete`
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 3 — Move `mod arena_compiler` → `fvm_storage/arena_compiler.rs`

*Execution phase — smaller model. ~749 lines.*

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
- [ ] `cargo test --workspace` — **467 / 0 / 0.**
- [ ] Commit, alone: `Major: Split fvm_storage.rs, Phase: move arena_compiler--complete`
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 4 — Move the stepping driver → `fvm_storage/stepping.rs` (a RENAME, not a split)

*Execution phase — smaller model. 94 lines, moved whole.*

> **REFRAMED 2026-09-24 (prep for execution).** This phase was written as "the one phase that
> divides an existing module", because `core_fir_conversion` then bundled the stepping driver
> with the `proto_to_core_fir` bridge (FOOP-96.md §2). **FOOP-86 deleted the bridge**, so there
> is nothing left to divide: the module is now 94 lines of stepping driver and nothing else.
> This phase is therefore an ordinary whole-module move, like Phases 1–3, that happens to
> rename the module on arrival.
>
> **`stepping.rs` IS created — this is decided, not open (the human, 2026-09-25).** The prep
> pass raised whether 94 lines earns its own file; the answer is yes, and emphatically:
>
> > *"those needs their own file. The ability is very important for debugging and have been used
> > a lot!!! It must be maintained separately and kept in working order."*
>
> So do **not** "simplify" by leaving these in the core, and do not treat the `expect(dead_code)`
> attributes as evidence the code is unused — they mean *no PRODUCTION caller*, which is the
> design, not neglect. `step_until`, `step_until_line_number` and `step_until_statement_name` are
> the Foolish debugger: they are the entry points the `foolish-debugging` skill is built on and
> the primary way FVM behaviour gets diagnosed in this project. A separate file is what keeps
> them visible and maintained rather than quietly rotting inside the arena core.
>
> **Consequences for this phase:** their tests
> (`fvm_storage::tests::step_until_*`, `step_to_constanic_settles_a_simple_fir`) are a hard gate,
> not incidental coverage — if any of them fails or stops being compiled, STOP. And the `//!`
> module doc must name the debugging role explicitly, so the next reader does not mistake
> dead-code-expecting functions for dead code.

**The stepping driver** — `fvm_storage.rs:3483–3576` as re-measured at `0df1865b`, **94 lines**
(was ~100 at `8c9043d8`). NOTE: this module is now the stepping driver ONLY — FOOP-86 deleted the
`proto_to_core_fir` bridge that used to share it, so the split this phase was written to perform
has already happened. What remains is a RENAME plus a doc-comment fix. See FOOP-96.md §2.

| function | note |
|---|---|
| `const MAX_STEPS: usize = 10_000;` | **moves with the driver** — it is `step_to_constanic`'s budget |
| `step_to_constanic` | the production stepping loop |
| `step_until` | generic matcher breakpoint |
| `step_until_line_number` | breakpoint by line |
| `step_until_statement_name` | breakpoint by statement name |

~~**Stays behind with the bridge:** `fn display_stmt_name`.~~ **VOID 2026-09-24** — FOOP-86
deleted `display_stmt_name` along with the bridge it served (`grep -c display_stmt_name
foolish-ubca2/src/fvm_storage.rs` → **0**). Nothing stays behind: the module holds only
`MAX_STEPS` and the four `step_*` functions, so this phase empties it.

**Preserve the `#[cfg_attr(not(test), expect(dead_code, …))]` attributes** on `step_until_*`
verbatim — they have no production caller by design (they are the `foolish-debugging` skill's
entry points), and dropping them produces exactly the new clippy/rustc warning the stop
conditions name.

**The re-export line**
`pub(crate) use core_fir_conversion::{proto_to_core_fir, step_to_constanic};` must be updated:
`step_to_constanic` now comes from `stepping`. Split it into two lines (one per source module).

- [ ] Establish relevant tests for this sub-section: the whole workspace (Phase 0's set). Use
      [these instructions](../../README.md#running-specific-tests). **The `step_until*` unit tests
      are a HARD GATE for this phase** — `foolish_ubca2::fvm_storage::tests::step_until_*` and
      `step_to_constanic_settles_a_simple_fir`. They are the only coverage of the debugger entry
      points, which have no production caller, so if one of them stops being COMPILED nothing
      else will notice. Count them before and after: `cargo test -p foolish-ubca2 --lib --
      step_until --test-threads=1` and confirm the same number runs — **measured 2026-09-25:
      exactly 3** (`step_until_generic_matcher_by_nyes`, `step_until_line_number_finds_line`,
      `step_until_statement_name_finds_second_statement`), one per debugger entry point. Any drop
      → STOP.
- [-] ~~Confirm the boundary: `display_stmt_name`'s callers are all in the bridge family.~~
      **VOID 2026-09-24** — there is no boundary left to confirm. FOOP-86 deleted the bridge AND
      `display_stmt_name` (`grep -c display_stmt_name foolish-ubca2/src/fvm_storage.rs` → 0), so
      `core_fir_conversion` is now 94 lines holding nothing but the four `step_*` functions and
      `MAX_STEPS`. Nothing is left behind by this move; the module is emptied and its `mod` line
      removed.
- [ ] Move `MAX_STEPS` + the four `step_*` functions **as text** into
      `foolish-ubca2/src/fvm_storage/stepping.rs`, with their doc comments and `#[cfg_attr]`
      attributes.
- [ ] Give the new file a `//!` module doc: the stepping loop and the `step_until*` breakpoints.
      **It must state that the `step_until*` functions are the project's Foolish debugger, that
      the `foolish-debugging` skill is built on them, and that their
      `expect(dead_code)` attributes mean "no PRODUCTION caller by design" — NOT that the code is
      unused** (the human, 2026-09-25: *"very important for debugging and have been used a
      lot!!! It must be maintained separately and kept in working order."*). A future reader who
      mistakes these for dead code is the specific failure this doc prevents.
      (`rust_instructions.md` §2d.3.)
- [ ] Move `core_fir_conversion`'s `use super::{…}` list across as well. Since the module holds
      ONLY these functions now, the whole list comes with them — there is no subset to narrow.
      Let the compiler flag anything unused; **do not widen anything to make it resolve.**
- [ ] In `fvm_storage.rs`: replace `mod core_fir_conversion { … }` with `mod stepping;`, and
      change the re-export at line ~4334 from
      `pub(crate) use core_fir_conversion::step_to_constanic;` to
      `pub(crate) use stepping::step_to_constanic;`. **That re-export is already the module's
      only one** — the earlier draft said "alongside the bridge's own re-export", which no
      longer exists.
- [ ] Confirm `core_fir_conversion` is now EMPTY and its `mod` block is gone entirely. *If
      anything is left in it → STOP and report: something was in that module that this plan did
      not account for.*
- [ ] `cargo build -p foolish-ubca2` — compiles. *Private-item error → STOP and report.*
- [ ] `cargo fmt --all` + `--check`; `cargo clippy -p foolish-ubca2 --all-targets` — clean.
      *An unused-import or dead-code warning here is THIS phase's and must be fixed.*
- [ ] `cargo test --workspace` — **467 / 0 / 0.**
- [ ] Commit, alone: `Major: Split fvm_storage.rs, Phase: stepping driver to its own file--complete`
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 5 — ~~Move the `proto_to_core_fir` bridge~~ — CANCELLED

> **[-] not needed — FOOP-86 removed the bridge (resolved 2026-09-24, prep for execution).**
>
> The conditional below resolved in the "already landed" direction: FOOP-86 merged to `jia` on
> 2026-09-21 and deleted the `proto_to_core_fir` family entirely. Verified —
> `grep -c proto_to_core_fir foolish-ubca2/src/fvm_storage.rs` returns **0**.
>
> **Skip this phase and go to Phase 6.** Every checkbox below is void; they are left in place
> as the record of a conditional that resolved, not as instruction. `core_fir_conversion` is
> now 94 lines holding only the stepping driver (Phase 4's block), so there is no second
> concern to separate — see FOOP-96.md §2.

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
      here** — the bridge feeds the old rendering path, so `einmo_gate_checked` is the
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
- [ ] `cargo test --workspace` — **467 / 0 / 0.**
- [ ] Commit, alone: `Major: Split fvm_storage.rs, Phase: move core_fir bridge--complete`
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 6 — Move `mod tests` → `fvm_storage/tests.rs`

*Execution phase — smaller model, but this is the **largest and riskiest** block. Read
FOOP-96.md §3.2 first.*

**The block**: `fvm_storage.rs:4337–7736`, **3 400 lines, 115 `#[test]` functions** — 44% of the
file. Moved as ONE file; **not** split by subject (FOOP-96.md §Rejected Alternatives G).

**Why this is the riskiest phase, stated concretely.** The block opens with `use super::*`, and
then reaches its **sibling** modules by **bare path** — those paths resolve ONLY because the glob
pulled the sibling module names into scope:

| line (abs., at `ef7fc880`) | the import |
|---|---|
| 5484 | `use search_engine::{ … };` (multi-line — copy it verbatim from the file) |
| 6079 | `use core_fir_conversion::step_to_constanic;` |
| 6090 | `use core_fir_conversion::{step_until, step_until_line_number, step_until_statement_name};` |
| 6172 | `use arena_compiler::compile;` |

**RE-MEASURED 2026-09-24. All four line numbers in the earlier draft (6242 / 6861 / 6981 / 7068)
were wrong**, and the 6861 entry named `proto_to_core_fir`, which FOOP-86 deleted — the import is
now `step_to_constanic` alone. Re-derive these yourself before moving anything:
`awk 'NR>=4337 && /use / && /(search_engine|core_fir_conversion|arena_compiler)/ {print NR": "$0}' foolish-ubca2/src/fvm_storage.rs`

**Two consequences, both already handled:**
- **`use super::*` keeps working.** A `#[cfg(test)] mod tests;` in a sibling file has the same
  `super` — `fvm_storage` — so all 229 `FirSpec` / 115 `FVMStorage` / 99 `FirCursor` / 14
  `revive_constanic` / … references resolve exactly as before (re-counted 2026-09-24; the
  earlier 235 / 114 / 76 / 21 predate FOOP-86 and FOOP-94). The move alone is safe.
- **Phase 4 already moved `step_until*` and `step_to_constanic`.** Line 6861's and 6981's
  imports were repointed in that phase. Confirm they still name the right modules here.

- [ ] Establish relevant tests for this sub-section: the whole workspace (Phase 0's set). Use
      [these instructions](../../README.md#running-specific-tests). **This phase's specific
      risk is the test COUNT**, not just pass/fail — see the dedicated checkbox below.
- [ ] **Record the pre-move `#[test]` count**:
      `grep -c "^\s*#\[test\]" foolish-ubca2/src/fvm_storage.rs` → *expected 115.*
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
      `grep -c "^\s*#\[test\]" foolish-ubca2/src/fvm_storage/tests.rs` → **must be 115**, and
      `grep -c "^\s*#\[test\]" foolish-ubca2/src/fvm_storage.rs` → **must be 0**.
      *Any other numbers → STOP and report.*
- [ ] `cargo build -p foolish-ubca2 --all-targets` — compiles (note `--all-targets`: a plain
      `build` does not compile `#[cfg(test)]` code, so it would not catch a broken test import).
- [ ] `cargo fmt --all` + `--check`; `cargo clippy -p foolish-ubca2 --all-targets` — clean.
- [ ] `cargo test --workspace` — **467 / 0 / 0.**
      **This is the phase where a silent test-count drop is most likely.** Read the number; do
      not glance at "ok".
- [ ] Additionally confirm the `foolish-ubca2` lib total is unchanged: `cargo test -p foolish-ubca2
      --lib` → **184 passed** (its share of the 467, including all three einmo gates).
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
  - [ ] `cargo build -p foolish-ubca2 --all-targets`; `cargo test --workspace` — **467 / 0 / 0.**
  - [ ] Commit, alone: `Major: Split fvm_storage.rs, Phase: rename search_fir_dispatch to search_dispatch--complete`
- [ ] **Rename `arena_compiler` → `compiler`.**
      `git mv foolish-ubca2/src/fvm_storage/arena_compiler.rs foolish-ubca2/src/fvm_storage/compiler.rs`
      Update the `mod` declaration, the
      `pub(crate) use arena_compiler::{compose_program_with_system, program_result};` re-export,
      and `tests.rs:use arena_compiler::compile;`.
  - [ ] `cargo build -p foolish-ubca2 --all-targets`; `cargo test --workspace` — **467 / 0 / 0.**
  - [ ] Commit, alone: `Major: Split fvm_storage.rs, Phase: rename arena_compiler to compiler--complete`
- [-] ~~**Rename `core_fir_conversion` → `core_fir_bridge`**~~ — **SKIP: Phase 5 was struck
      (2026-09-24), so this file never exists.** The `core_fir_conversion` NAME is retired by
      Phase 4 instead, which moves the stepping driver to `stepping.rs`.
      `git mv foolish-ubca2/src/fvm_storage/core_fir_conversion.rs foolish-ubca2/src/fvm_storage/core_fir_bridge.rs`
      Update the `mod` declaration, the `pub(crate) use` re-export, and `tests.rs`'s bare-path
      import of it.
  - [ ] `cargo build -p foolish-ubca2 --all-targets`; `cargo test --workspace` — **467 / 0 / 0.**
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
  - [ ] Repair ALL tests in `jia` at `/yolo/foolish` after the merge — **467 / 0 / 0.**
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

**Date**: 2026-09-25
**Updated By**: Claude Code / claude-opus-5
**Changes**: THIRD PREP PASS — two human rulings recorded, plus a rename.
**(1) `stepping.rs` is DECIDED, not open.** The previous pass left "does a 94-line module earn its
own file?" for the executor to raise. The human settled it emphatically — *"those needs their own
file. The ability is very important for debugging and have been used a lot!!! It must be
maintained separately and kept in working order."* Phase 4 now says so, warns against
"simplifying" by leaving them in the core, and explains that the `expect(dead_code)` attributes
mean *no PRODUCTION caller by design* rather than unused code — the specific misreading that could
get the Foolish debugger deleted. The `//!` doc requirement now mandates stating that role, and
the three `step_until*` tests are named as a HARD GATE with their measured count (exactly 3, one
per entry point): they are the only coverage of functions with no production caller, so if one
stops being COMPILED nothing else notices.
**(2) `ubca_snapshot_tester` renamed to `einmo_gates`.** The human flagged the name as suspect —
the project uses einmo, not generic snapshots. The file's own first line already read "Einmo gates
for FOOP-36's hand-authored Foolish rendering contract", so the filename contradicted its own doc;
it is `#[cfg(test)]`-only and contains three einmo gates and nothing else. Note the `2` suffix was
already gone (FOOP-86 renamed the file); what this pass fixed was the NAME, and separately the
plan's stale references to the retired `ubca_snapshot_tester2` module. Live references updated in
`lib.rs`, `foolish-cli/src/main.rs`, and this plan; completed FOOP plans left as written, per the
historical-record rule.
