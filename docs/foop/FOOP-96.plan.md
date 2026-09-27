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
| `cargo test --workspace` passed | **470** (UPDATED 2026-09-27 — see Phase 0; was 467 before `f8134ac5` landed) |
| failed | **0** |
| ignored | **0** (the old `foolish-ubca` doctest went with that crate, FOOP-86) |
| `cargo fmt --all --check` | clean |
| `cargo clippy -p foolish-ubca2 --all-targets` | no NEW warnings |

**470 must not move in EITHER direction.** A test that vanishes is as bad as one that fails —
a `#[cfg(test)]` block that stops being compiled reports no error, it just stops running.

*Known, NOT this FOOP's to fix:* 4 pre-existing `foolish-core` clippy errors in `sequencer.rs`
break a workspace-wide `-D warnings` gate. Scope clippy to `-p foolish-ubca2`. A **new** warning
inside `foolish-ubca2` IS this FOOP's.

### Stop conditions — every phase

**STOP and report to the human, do not improvise**, if any of these occurs:

- A compile error naming a **private** item (`Slot`, `.payload`, `self.slots`, `validate`,
  or any private field) — the move has found a real coupling. **Do NOT widen visibility.**
- The test count is **not 470 passed / 0 failed**, in either direction.
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

- [x] Read [`FOOP-96.md`](FOOP-96.md) in full — especially §0, §1 (the measured structure),
      §3 (target layout), §3.2 (the tests import hazard), §4 (the safety argument), §5.
      (2026-09-27 10:00)
- [x] Begin work: commit `FOOP-96.md` and `FOOP-96.plan.md` to `jia`, check `begun: [x]` in the
      `FOOP-96.md` frontmatter.
      (2026-09-27 10:00)
- [x] Create worktree at `/yolo/foolish/../foolish_worktrees/foop-96-split-fvm-storage` with
      branch `foop-96-split-fvm-storage`:
      `git worktree add -b "foop-96-split-fvm-storage" "/yolo/foolish/../foolish_worktrees/foop-96-split-fvm-storage"`
      **From here until merge, ALL work — including edits to `FOOP-96.md` and this plan — happens
      ONLY in the worktree.**
      (2026-09-27 10:00)
- [x] **Determine whether FOOP-86 has landed.** — **RESOLVED 2026-09-24, before execution.**
      FOOP-86 merged to `jia` on 2026-09-21. Verified:
      `grep -c "fn proto_to_core_fir" foolish-ubca2/src/fvm_storage.rs` → **0**.
      **Phase 5 is struck** and Phase 7's dependent `core_fir_bridge` rename with it. Re-run the
      grep to confirm for yourself, but do not re-litigate the branch — it is decided.
- [x] **Re-measure the block boundaries** (they shift if FOOP-86 landed).
      (2026-09-27 10:15)
      **RE-MEASURED again — the plan's `0df1865b` figures are ALSO now stale.** Commit `f8134ac5`
      ("Make the FVM debugger a public API...", 2026-09-26, landed on `jia` AFTER this plan's last
      prep pass) changed `fvm_storage.rs` before this FOOP's worktree was ever created: it grew
      the file from 7 737 → **7 987** lines (+250: a new lazy post-order iterator, its invariant
      test, and a new `foolish-ubca2/tests/debugger_api.rs` integration test — see below), and it
      changed `core_fir_conversion` from a *private* `mod` with `pub(crate)` functions to a
      **`pub mod` with `pub` functions** (the human's ruling, 2026-09-26: *"the debugging code
      should be accessible by users of the fvm"*), removing the three `expect(dead_code)`
      attributes in the process. Brace-matched boundaries as of `jia` HEAD (`a4fa3ea3`):

      | block | lines | size | vs. plan's stale figure |
      |---|---|---|---|
      | `search_engine` | 2237–2607 | 371 | unchanged |
      | `search_fir_dispatch` | 2612–3471 | 860 | unchanged |
      | `core_fir_conversion` | 3483–3570 | **88** | was 94 — shrank 6 (dead_code attrs removed, `pub` added; net negative) |
      | `arena_compiler` | 3574–4322 | 749 | unchanged |
      | `mod tests` | 4331–7987 | **3657** | was 3400 — grew 257 (new iterator + invariant test) |

      `#[test]` count is now **117**, not 115 (`grep -c "^\s*#\[test\]"
      foolish-ubca2/src/fvm_storage.rs` → 117). The two new tests are
      `stepping_leaves_a_constanic_prefix_in_traversal_order` and its supporting iterator test,
      added by `f8134ac5`. The `step_until*` hard-gate count Phase 4 relies on is UNCHANGED at
      exactly 3 — verified: `cargo test -p foolish-ubca2 --lib -- step_until --test-threads=1`
      still reports `3 passed`.

      **New discovery, not anticipated by any prior prep pass:**
      `foolish-ubca2/tests/debugger_api.rs` (added by `f8134ac5`) is an INTEGRATION test that
      imports `foolish_ubca2::fvm_storage::core_fir_conversion::{step_to_constanic, step_until,
      step_until_line_number, step_until_statement_name}` by its full public path. Phase 4 moves
      and renames this module to `fvm_storage::stepping`; **that import line must be updated in
      the same phase**, or the integration test fails to compile (not merely fails — the crate
      fails to build `--all-targets`, which Phase 4's own build-check step will catch, but the fix
      is now written down here so it isn't a surprise). Added an explicit checkbox to Phase 4
      below for this.
- [x] **Re-verify safety property (2): ZERO private-internal reaches.** From the first inner
      module's line to EOF:
      `sed -n '2237,$p' foolish-ubca2/src/fvm_storage.rs | grep -c "self\.slots\|\.payload\|validate("`
      → **0.** Property holds; safe to proceed mechanically.
      (2026-09-27 10:15)
- [x] **Record the test baseline** in this plan: ran `cargo test --workspace` in the worktree.
      **Result: 470 passed, 0 failed, 0 ignored — NOT 467.** The plan's 467 predates `f8134ac5`
      (2026-09-26), which added `debugger_entry_points_are_reachable_from_outside_the_crate` (the
      new integration test, +1) and the two stepping-invariant unit tests (+2) = +3, matching
      467 → 470 exactly. That commit's own message states "Workspace 470 / 0 / 0", corroborating.
      **470 / 0 / 0 is the invariant for every phase below, replacing 467 everywhere it appears.**
      Per-crate breakdown (workspace total 470): foolish-core 133, foolish-cli 4,
      foolish-parser 84, einmo 62, foolish-ubca2 lib 186 (includes all 4 einmo-related tests:
      `einmo_gate_checked`, `einmo_gate_output`, `einmo_gate_verified`,
      `einmo_corpus_wide_foolish_rendering_parses`), foolish-ubca2 integration
      (`debugger_api.rs`) 1. All four einmo tests confirmed green including `einmo_gate_verified`
      (run serially, `-- --test-threads=1`).
      (2026-09-27 10:20)
- [x] Establish relevant tests for this FOOP. Use
      [these instructions](../../README.md#running-specific-tests). **Every phase of this FOOP
      uses the SAME set — the whole workspace — because a move can break anything:**
      `cargo test --workspace -- --test-threads=1` (all 470), plus the einmo gates
      (2026-09-27 10:25)
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
- [x] Run all tests — old and new — and make sure they all pass correctly.
      `cargo test --workspace` → 470/0/0. `cargo test -p foolish-ubca2 --lib --
      --test-threads=1 einmo_gate` → all 4 einmo-related tests pass, including
      `einmo_gate_verified`. No code has been touched yet in this worktree (Phase 0 is pure
      re-measurement), so this run also serves as the pre-Phase-1 baseline confirmation.
      (2026-09-27 10:25)

---

## Phase 1 — Move `mod search_engine` → `fvm_storage/search_engine.rs`

*Execution phase — smaller model. Fixed target: 371 lines move, 470 still pass.*

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
- [ ] `cargo test --workspace` — **470 passed, 0 failed, 0 ignored.**
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
- [ ] `cargo test --workspace` — **470 / 0 / 0.**
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
- [ ] `cargo test --workspace` — **470 / 0 / 0.**
- [ ] Commit, alone: `Major: Split fvm_storage.rs, Phase: move arena_compiler--complete`
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 4 — Move the stepping driver → `fvm_storage/stepping.rs` (a RENAME, not a split)

*Execution phase — smaller model. 88 lines, moved whole.*

> **REFRAMED 2026-09-24 (prep for execution).** This phase was written as "the one phase that
> divides an existing module", because `core_fir_conversion` then bundled the stepping driver
> with the `proto_to_core_fir` bridge (FOOP-96.md §2). **FOOP-86 deleted the bridge**, so there
> is nothing left to divide: the module is stepping driver and nothing else.
> This phase is therefore an ordinary whole-module move, like Phases 1–3, that happens to
> rename the module on arrival.
>
> **`stepping.rs` IS created — this is decided, not open (the human, 2026-09-25).** The prep
> pass raised whether the small module earns its own file; the answer is yes, and emphatically:
>
> > *"those needs their own file. The ability is very important for debugging and have been used
> > a lot!!! It must be maintained separately and kept in working order."*
>
> So do **not** "simplify" by leaving these in the core.
>
> **Consequences for this phase:** their tests
> (`fvm_storage::tests::step_until_*`, `step_to_constanic_settles_a_simple_fir`) are a hard gate,
> not incidental coverage — if any of them fails or stops being compiled, STOP. And the `//!`
> module doc must name the debugging role explicitly.

> **UPDATED 2026-09-27 (Phase 0 re-measurement, post-`f8134ac5`).** Two facts below changed AGAIN
> after this section's 2026-09-25 pass, from a commit that landed on `jia` before this FOOP's
> worktree existed:
>
> 1. **Visibility is no longer private/`pub(crate)`.** The human separately ruled (2026-09-26)
>    that *"the debugging code should be accessible by users of the fvm"*: `core_fir_conversion`
>    is now `pub mod`, and all four `step_*` functions are `pub fn` (not `pub(crate) fn`). This
>    move must PRESERVE that visibility exactly — the new `stepping.rs` module and its four
>    functions must land as `pub mod stepping` / `pub fn`, not narrowed back to `pub(crate)`. A
>    visibility narrowing here is a regression this FOOP must not introduce (§0.2 forbids
>    WIDENING; this is the mirror case, narrowing, which is equally a behavior change).
> 2. **The `expect(dead_code)` attributes named below NO LONGER EXIST.** `f8134ac5` removed all
>    three, because a `pub` item in a `pub mod` is never dead code — there is nothing to
>    "preserve" on the four functions; the instruction below to keep them verbatim is VOID. Do
>    not re-add them.
> 3. **A new integration test now depends on this module's path.**
>    `foolish-ubca2/tests/debugger_api.rs` imports
>    `foolish_ubca2::fvm_storage::core_fir_conversion::{step_to_constanic, step_until,
>    step_until_line_number, step_until_statement_name}` by full public path. **This phase must
>    update that import to `foolish_ubca2::fvm_storage::stepping::{...}`** — added as its own
>    checkbox below. Without it, `cargo build -p foolish-ubca2 --all-targets` fails to compile
>    (the build-check step below will catch it, but now it's expected rather than a surprise).

**The stepping driver** — `fvm_storage.rs:3483–3570` as re-measured 2026-09-27 on `jia` HEAD
(`a4fa3ea3`), **88 lines** (was 94 at `0df1865b`, before `f8134ac5` removed the three
`expect(dead_code)` attributes and added `pub`). NOTE: this module is now the stepping driver
ONLY — FOOP-86 deleted the `proto_to_core_fir` bridge that used to share it, so the split this
phase was written to perform has already happened. What remains is a RENAME plus a doc-comment
fix. See FOOP-96.md §2.

| function | note |
|---|---|
| `const MAX_STEPS: usize = 10_000;` | **moves with the driver** — it is `step_to_constanic`'s budget |
| `pub fn step_to_constanic` | the production stepping loop |
| `pub fn step_until` | generic matcher breakpoint |
| `pub fn step_until_line_number` | breakpoint by line |
| `pub fn step_until_statement_name` | breakpoint by statement name |

~~**Stays behind with the bridge:** `fn display_stmt_name`.~~ **VOID 2026-09-24** — FOOP-86
deleted `display_stmt_name` along with the bridge it served (`grep -c display_stmt_name
foolish-ubca2/src/fvm_storage.rs` → **0**). Nothing stays behind: the module holds only
`MAX_STEPS` and the four `step_*` functions, so this phase empties it.

~~**Preserve the `#[cfg_attr(not(test), expect(dead_code, …))]` attributes** on `step_until_*`
verbatim.~~ **VOID 2026-09-27** — `f8134ac5` already removed these three attributes (see the
updated-facts note above). There is nothing to preserve; do not re-add them.

**The re-export line, current form:**
`pub(crate) use core_fir_conversion::step_to_constanic;` (a single line — the `proto_to_core_fir`
half of the earlier pair is long gone with the bridge, so there is no pair to split). Change it to
`pub(crate) use stepping::step_to_constanic;`.

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
- [ ] Move `MAX_STEPS` + the four `pub fn step_*` functions **as text** into
      `foolish-ubca2/src/fvm_storage/stepping.rs`, with their doc comments, **preserving `pub`
      visibility on the module and all four functions exactly as they are now** (do NOT narrow to
      `pub(crate)` — see the 2026-09-27 updated-facts note above). There are no `#[cfg_attr]`
      dead-code attributes left to move (removed by `f8134ac5`).
- [ ] Give the new file a `//!` module doc: the stepping loop and the `step_until*` breakpoints.
      **It must state that the `step_until*` functions are the project's Foolish debugger, that
      the `foolish-debugging` skill is built on them, and that they are `pub` because downstream
      code driving the FVM needs them** (the human, 2026-09-25: *"very important for debugging and
      have been used a lot!!! It must be maintained separately and kept in working order."*; the
      human, 2026-09-26: *"the debugging code should be accessible by users of the fvm"*). Do NOT
      describe them via `expect(dead_code)` framing — that framing was already corrected by
      `f8134ac5` and reintroducing it in the new doc would be a regression of that fix.
      (`rust_instructions.md` §2d.3.)
- [ ] Move `core_fir_conversion`'s `use super::{…}` list across as well. Since the module holds
      ONLY these functions now, the whole list comes with them — there is no subset to narrow.
      Let the compiler flag anything unused; **do not widen anything to make it resolve.**
- [ ] In `fvm_storage.rs`: replace `pub mod core_fir_conversion { … }` with `pub mod stepping;`
      (**preserve `pub`** — do not narrow to `mod` or `pub(crate) mod`), and change the re-export
      from `pub(crate) use core_fir_conversion::step_to_constanic;` to
      `pub(crate) use stepping::step_to_constanic;`. **That re-export is already the module's
      only one** — the earlier draft said "alongside the bridge's own re-export", which no
      longer exists.
- [ ] **Re-point `foolish-ubca2/tests/debugger_api.rs`'s import** from
      `foolish_ubca2::fvm_storage::core_fir_conversion::{step_to_constanic, step_until,
      step_until_line_number, step_until_statement_name}` to
      `foolish_ubca2::fvm_storage::stepping::{step_to_constanic, step_until,
      step_until_line_number, step_until_statement_name}`. This is a required re-pointing
      of an existing bare-path-style reference, not a new behavior — the test's assertions are
      unchanged. (Discovered in this FOOP's Phase 0 re-measurement, 2026-09-27; the integration
      test did not exist when this phase was originally drafted.)
- [ ] Confirm `core_fir_conversion` is now EMPTY and its `mod` block is gone entirely. *If
      anything is left in it → STOP and report: something was in that module that this plan did
      not account for.*
- [ ] `cargo build -p foolish-ubca2 --all-targets` — compiles (note `--all-targets`: this crate
      now has an integration test in `tests/`, which a plain `build` does not compile).
      *Private-item error → STOP and report.*
- [ ] `cargo fmt --all` + `--check`; `cargo clippy -p foolish-ubca2 --all-targets` — clean.
      *An unused-import or dead-code warning here is THIS phase's and must be fixed.*
- [ ] `cargo test --workspace` — **470 / 0 / 0.** Additionally confirm
      `debugger_entry_points_are_reachable_from_outside_the_crate`
      (`foolish-ubca2/tests/debugger_api.rs`) still passes — it is the strongest available check
      that this move preserved external reachability of the renamed module.
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
- [ ] `cargo test --workspace` — **470 / 0 / 0.**
- [ ] Commit, alone: `Major: Split fvm_storage.rs, Phase: move core_fir bridge--complete`
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 6 — Move `mod tests` → `fvm_storage/tests.rs`

*Execution phase — smaller model, but this is the **largest and riskiest** block. Read
FOOP-96.md §3.2 first.*

**The block**: `fvm_storage.rs:4331–7987` as of `jia` HEAD (`a4fa3ea3`), **3 657 lines, 117
`#[test]` functions** — 46% of the file. Moved as ONE file; **not** split by subject
(FOOP-96.md §Rejected Alternatives G).

**Why this is the riskiest phase, stated concretely.** The block opens with `use super::*`, and
then reaches its **sibling** modules by **bare path** — those paths resolve ONLY because the glob
pulled the sibling module names into scope:

| line (abs., at `a4fa3ea3`, 2026-09-27) | the import |
|---|---|
| 5478 | `use search_engine::{ … };` (multi-line — copy it verbatim from the file) |
| 6073 | `use core_fir_conversion::step_to_constanic;` — **NOTE: by the time this phase runs, Phase 4 has already renamed the module to `stepping` AND is required (its own checklist, updated 2026-09-27) to repoint this exact line to `use stepping::step_to_constanic;`. Verify it reads `stepping::…`, not `core_fir_conversion::…`, before moving — if it still says `core_fir_conversion`, Phase 4 was not completed correctly; STOP and go fix Phase 4, do not silently repoint it here.** |
| 6084 | `use core_fir_conversion::{step_until, step_until_line_number, step_until_statement_name};` — **same note: expect `use stepping::{…}` here by the time this phase runs.** |
| 6166 | `use arena_compiler::compile;` |

**RE-MEASURED 2026-09-27** (superseding the 2026-09-24 pass's 5484/6079/6090/6172, which are all
off by a handful of lines due to `f8134ac5` landing in between). Re-derive these yourself before
moving anything — do not trust any line number printed above without re-running this:
`awk 'NR>=4331 && /use / && /(search_engine|core_fir_conversion|stepping|arena_compiler)/ {print NR": "$0}' foolish-ubca2/src/fvm_storage.rs`

**Two consequences, both already handled:**
- **`use super::*` keeps working.** A `#[cfg(test)] mod tests;` in a sibling file has the same
  `super` — `fvm_storage` — so every `FirSpec` / `FVMStorage` / `FirCursor` / `revive_constanic` /
  … reference resolves exactly as before. The move alone is safe. (The exact per-symbol
  occurrence counts in earlier drafts — 229/115/99/14, before that 235/114/76/21 — are not
  re-verified here; they are illustrative of the pattern, not a gate this phase checks.)
- **Phase 4 already moved and renamed `step_until*` and `step_to_constanic` to `stepping::`.**
  The two `core_fir_conversion::` import lines above must already read `stepping::` by the time
  this phase executes — confirmed by the checkbox above, not assumed.

- [ ] Establish relevant tests for this sub-section: the whole workspace (Phase 0's set). Use
      [these instructions](../../README.md#running-specific-tests). **This phase's specific
      risk is the test COUNT**, not just pass/fail — see the dedicated checkbox below.
- [ ] **Record the pre-move `#[test]` count**:
      `grep -c "^\s*#\[test\]" foolish-ubca2/src/fvm_storage.rs` → *expected 117 (UPDATED
      2026-09-27; was 115 before `f8134ac5` added two stepping-invariant tests).*
- [ ] Move `mod tests`'s body **as text** into `foolish-ubca2/src/fvm_storage/tests.rs`; strip
      one indent level; drop the `#[cfg(test)] mod tests {` wrapper and its closing `}`.
      **Keep `use super::*;` as the first line** and keep every bare-path sibling import exactly
      as it is (they should already say `stepping::…`, not `core_fir_conversion::…` — see above).
- [ ] In `fvm_storage.rs`, replace the removed block with:
      ```rust
      #[cfg(test)]
      mod tests;
      ```
- [ ] **Verify the test count moved intact**:
      `grep -c "^\s*#\[test\]" foolish-ubca2/src/fvm_storage/tests.rs` → **must be 117**, and
      `grep -c "^\s*#\[test\]" foolish-ubca2/src/fvm_storage.rs` → **must be 0**.
      *Any other numbers → STOP and report.*
- [ ] `cargo build -p foolish-ubca2 --all-targets` — compiles (note `--all-targets`: a plain
      `build` does not compile `#[cfg(test)]` code, so it would not catch a broken test import).
- [ ] `cargo fmt --all` + `--check`; `cargo clippy -p foolish-ubca2 --all-targets` — clean.
- [ ] `cargo test --workspace` — **470 / 0 / 0.**
      **This is the phase where a silent test-count drop is most likely.** Read the number; do
      not glance at "ok".
- [ ] Additionally confirm the `foolish-ubca2` lib total is unchanged: `cargo test -p foolish-ubca2
      --lib` → **186 passed** (UPDATED 2026-09-27 — was 184 before `f8134ac5` added
      `stepping_leaves_a_constanic_prefix_in_traversal_order` and its supporting iterator test;
      its share of the 470 workspace total, including all four einmo-related lib tests). Note the
      `foolish-ubca2` INTEGRATION test (`debugger_api.rs`, 1 test) is separate from this `--lib`
      count — it is exercised by the plain `cargo test --workspace` line above instead.
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
  - [ ] `cargo build -p foolish-ubca2 --all-targets`; `cargo test --workspace` — **470 / 0 / 0.**
  - [ ] Commit, alone: `Major: Split fvm_storage.rs, Phase: rename search_fir_dispatch to search_dispatch--complete`
- [ ] **Rename `arena_compiler` → `compiler`.**
      `git mv foolish-ubca2/src/fvm_storage/arena_compiler.rs foolish-ubca2/src/fvm_storage/compiler.rs`
      Update the `mod` declaration, the
      `pub(crate) use arena_compiler::{compose_program_with_system, program_result};` re-export,
      and `tests.rs:use arena_compiler::compile;`.
  - [ ] `cargo build -p foolish-ubca2 --all-targets`; `cargo test --workspace` — **470 / 0 / 0.**
  - [ ] Commit, alone: `Major: Split fvm_storage.rs, Phase: rename arena_compiler to compiler--complete`
- [-] ~~**Rename `core_fir_conversion` → `core_fir_bridge`**~~ — **SKIP: Phase 5 was struck
      (2026-09-24), so this file never exists.** The `core_fir_conversion` NAME is retired by
      Phase 4 instead, which moves the stepping driver to `stepping.rs`.
      `git mv foolish-ubca2/src/fvm_storage/core_fir_conversion.rs foolish-ubca2/src/fvm_storage/core_fir_bridge.rs`
      Update the `mod` declaration, the `pub(crate) use` re-export, and `tests.rs`'s bare-path
      import of it.
  - [ ] `cargo build -p foolish-ubca2 --all-targets`; `cargo test --workspace` — **470 / 0 / 0.**
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
  - [ ] Repair ALL tests in `jia` at `/yolo/foolish` after the merge — **470 / 0 / 0.**
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
