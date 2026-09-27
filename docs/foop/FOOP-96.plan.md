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

**AMENDED 2026-09-27.** `foolish-ubca2` is **not** clippy-clean either: `sequencer.rs:525`
carries a pre-existing `clippy::collapsible_if`. Verified pre-existing by running clippy on
**unmodified `jia`** — same single warning, same location. It is outside `fvm_storage.rs` and
untouched by this FOOP, so the operative test remains *"no NEW warnings"*, which the move must
satisfy. Recorded so a later phase does not mistake it for damage it caused.

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

- [x] Read [`FOOP-96.md`](FOOP-96.md) in full — especially §0, §1 (the measured structure),
      §3 (target layout), §3.2 (the tests import hazard), §4 (the safety argument), §5.
      (2026-09-27 12:55)
- [x] Begin work: commit `FOOP-96.md` and `FOOP-96.plan.md` to `jia`, check `begun: [x]` in the
      `FOOP-96.md` frontmatter.
      (2026-09-27 12:55)
- [x] Create worktree at `/yolo/foolish/../foolish_worktrees/foop-96-split-fvm-storage` with
      branch `foop-96-split-fvm-storage`:
      `git worktree add -b "foop-96-split-fvm-storage" "/yolo/foolish/../foolish_worktrees/foop-96-split-fvm-storage"`
      **From here until merge, ALL work — including edits to `FOOP-96.md` and this plan — happens
      ONLY in the worktree.**
      (2026-09-27 12:55)
- [x] **Determine whether FOOP-86 has landed.** — **RESOLVED 2026-09-24, before execution.**
      FOOP-86 merged to `jia` on 2026-09-21. Verified:
      `grep -c "fn proto_to_core_fir" foolish-ubca2/src/fvm_storage.rs` → **0**.
      **Phase 5 is struck** and Phase 7's dependent `core_fir_bridge` rename with it. Re-run the
      grep to confirm for yourself, but do not re-litigate the branch — it is decided.
- [x] **Re-measure the block boundaries** (they shift if FOOP-86 landed).
      *Verify, don't re-derive* — FOOP-96.md §1 gives the values RE-MEASURED at `0df1865b`
      (2026-09-24, post-FOOP-86). The 2026-09-16 `8c9043d8` figures are stale in EVERY row:
      `grep -n "^mod search_engine\|^pub(crate) mod search_engine\|^mod search_fir_dispatch\|^mod core_fir_conversion\|^mod arena_compiler\|^mod tests\|^pub(crate) use" foolish-ubca2/src/fvm_storage.rs`
      Record the CURRENT line numbers in this plan before moving anything.
      (2026-09-27 12:55)

      **RE-MEASURED 2026-09-27 at `f8134ac5`** (brace-matching walk, 1-based, inclusive). The
      file is now **7 987** lines, not 7 737: commit `f8134ac5` (2026-09-26, "Make the FVM
      debugger a public API") landed AFTER this plan's 2026-09-25 prep pass and changed three
      things that this FOOP must respect. Recorded below as **§"Phase 0 measurements"**.

- [x] **Re-verify safety property (2): ZERO private-internal reaches.** From the first inner
      module's line to EOF:
      `sed -n '<first_mod_line>,$p' foolish-ubca2/src/fvm_storage.rs | grep -c "self\.slots\|\.payload\|validate("`
      **Expected: `0`.** Any non-zero result means FOOP-96.md §4(2) no longer holds — **STOP and
      report** before moving any block.
      (2026-09-27 12:55)

      **Verified: `0`.** Run over `sed -n '2237,$p'` (first inner module's `mod` line through
      EOF). FOOP-96.md §4(2) still holds; no field needs widening and no accessor needs inventing.

- [x] **Record the test baseline** in this plan: run `cargo test --workspace` and write down
      passed / failed / ignored. *Expected 467 / 0 / 0.* If it differs, the baseline has moved —
      record the NEW number and use it as the invariant for every phase below.
      (2026-09-27 12:55)

      **BASELINE IS NOW 470 / 0 / 0** (not 467 / 0 / 0). Measured on `jia` at `95e7137d` via
      `cargo test --workspace -- --test-threads=1`. The `+3` is `f8134ac5`: two new `foolish-ubca2`
      unit tests and one new integration test. Per the instruction above, **470 is the invariant
      for every phase below**, in BOTH directions. Breakdown, `test result: ok.` throughout:

      | target | passed |
      |---|---|
      | `einmo` lib | 133 |
      | `foolish-cli` bin | 4 |
      | `foolish-core` lib | 84 |
      | `foolish-parser` lib | 62 |
      | **`foolish-ubca2` lib** | **186** (was 184; plan's "184" is stale) |
      | `foolish-ubca2/tests/debugger_api.rs` | **1** (NEW — see below) |
      | **workspace total** | **470 / 0 / 0** |

- [x] Establish relevant tests for this FOOP. Use
      [these instructions](../../README.md#running-specific-tests). **Every phase of this FOOP
      uses the SAME set — the whole workspace — because a move can break anything:**
      `cargo test --workspace -- --test-threads=1` (all 467), plus the einmo gates
      `einmo_gates::einmo_tests::einmo_gate_checked` and `einmo_gate_verified` as the
      byte-identity oracle. There is no smaller meaningful subset for a file split, and the plan
      says so rather than inventing one.
      (2026-09-27 12:55)

      *(467 in the line above is stale text from the prep pass; the set is unchanged — the whole
      workspace — and its size is now 470, recorded above.)*

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
      (2026-09-27 12:55) — **470 / 0 / 0**, fmt clean.

---

## Phase 0 measurements (recorded before moving anything)

*Every figure below was measured in the worktree at `f8134ac5` on 2026-09-27 by a
brace-matching walk (`mod X {` through its closing `}`), which is what actually moves. 1-based,
inclusive. The doc comment immediately above each `mod` line moves with it.*

| block | lines | size | `mod` form | note |
|---|---|---|---|---|
| top-level (the core) | 1–2236 | 2 236 | — | unchanged from FOOP-96.md §1 |
| `mod search_engine` | **2237–2607** | 371 | `pub(crate) mod search_engine {` | doc comment at 2235–2236 |
| *(gap)* | 2608–2611 | 4 | | |
| `mod search_fir_dispatch` | **2612–3471** | 860 | `mod search_fir_dispatch {` | doc comment at 2608–2610 |
| *(gap)* | 3472 | 1 | | |
| `mod core_fir_conversion` | **3483–3570** | **88** | **`pub mod core_fir_conversion {`** | doc comment 3473–3482; **was 94, and was private** |
| *(gap)* | 3571–3573 | 3 | | |
| `mod arena_compiler` | **3574–4322** | 749 | `mod arena_compiler {` | doc comment at 3571–3573 |
| *(gap)* | 4323–4329 | 7 | 4323–4325 re-export doc, 4326 blank | |
| `pub(crate) use` re-exports | 4327–4328 | 2 | | stay in `fvm_storage.rs` |
| *(blank)* | 4329–4330 | 2 | | |
| `mod tests` | **4331–7987** | **3 657** | `#[cfg(test)] mod tests {` (4330–4331) | was 3 400; **117** `#[test]` fns, was 115 |

`#[test]` count in `fvm_storage.rs`: **117** (`grep -c "^\s*#\[test\]"`) — the plan's "115" is
stale; `f8134ac5` added `stepping_leaves_a_constanic_prefix_in_traversal_order` and one more.

### What `f8134ac5` (2026-09-26) changed that this FOOP must respect

That commit landed the day after this plan's last prep pass and is **not** reflected in the plan's
prose. Three consequences, each handled below:

1. **`mod core_fir_conversion` is now `pub mod`, and its four `step_*` functions are `pub`.** The
   `expect(dead_code)` attributes are **gone**. Phase 4's instruction to "preserve the
   `#[cfg_attr(not(test), expect(dead_code, …))]` attributes verbatim" is therefore **inert** —
   there are none to preserve, and *adding them back would be a regression* the commit explicitly
   removed. Phase 4's `//!` doc requirement is also partly falsified: it must NOT claim the
   functions carry `expect(dead_code)`, because they no longer do. What the doc must still say —
   and this part is unchanged — is that these are the project's Foolish debugger, that the
   `foolish-debugging` skill is built on them, and that they must be kept in working order.
   **Adjusted wording is given in Phase 4.**
2. **`foolish-ubca2/tests/debugger_api.rs` is a new integration test** that imports
   `foolish_ubca2::fvm_storage::core_fir_conversion::{step_to_constanic, step_until,
   step_until_line_number, step_until_statement_name}`. It compiles as a SEPARATE crate and is
   the visibility regression test. **The Phase 4 rename moves this public path** to
   `...::fvm_storage::stepping::…`, so `tests/debugger_api.rs` MUST be repointed in the rename
   commit or the crate stops compiling. Its `//!` doc also names `mod core_fir_conversion` and
   must be updated. The commit message of `f8134ac5` anticipated exactly this: *"relocating them
   to stepping.rs is FOOP-96 Phase 4, still forthcoming."* **This is a public-path change** —
   recorded as a doubt for the human in Phase 8, since FOOP-96.md §Abstract predates the module
   becoming public and asserts "no public API changes".
3. **The tests call their siblings by bare path INLINE IN BODIES**, not only in `use` lines:
   `core_fir_conversion::step_to_constanic(...)` (~30 call sites), `arena_compiler::compile` /
   `compose_program_with_system` / `program_result` (~15), `search_fir_dispatch::ib_search_by_pattern`
   (1). All resolve today only because `use super::*` pulls the sibling module names into scope.
   They survive the **moves** unchanged (the names do not change) and must all be repointed at the
   **renames** — Phase 4 and Phase 7. The compiler enforces every site, which is why the renames
   are safe to do mechanically.

### The four bare-path sibling imports inside `mod tests` (re-measured)

| line | import |
|---|---|
| 5478 | `use search_engine::{BraneNavigator, CandidateNavigator, MatchOutcome, ScanCtx, ScanOutcome, SearchPredicate, contextful_search_scan, contextful_search_scan_no_body_check};` |
| 6073 | `use core_fir_conversion::step_to_constanic;` |
| 6084 | `use core_fir_conversion::{step_until, step_until_line_number, step_until_statement_name};` |
| 6166 | `use arena_compiler::compile;` |

### Each module's current `use super::{…}` list (verbatim — copy these)

| module | lines | its imports |
|---|---|---|
| `search_engine` | 2238–2241 | `use super::{Equality, FVMStorage, FirCursor, FirPointer, default_equal};` / `use foolish_core::fir::Nyes;` / `use regex::Regex;` |
| `search_fir_dispatch` | 2613–2620 | `use super::search_engine::{BraneNavigator, ScanOutcome, SearchPredicate, contextful_search_scan, contextful_search_scan_no_body_check};` / `use super::{FVMStorage, FirCursor, FirPointer, FirSpec};` / `use foolish_core::fir::Nyes;` |
| `core_fir_conversion` | 3484 | `use super::{FVMStorage, FirCursor, FirPointer};` — **only these three**, narrower than FOOP-96.md §4(1)'s table |
| `arena_compiler` | 3575–3583 | `use super::{ANON_STMT_NAME, ConcatProvenance, ConcatRenderingAid, FVMStorage, FirCursor, FirCursorMut, FirPointer, FirSpec, StayMarker};` / `use foolish_core::fir::Nyes;` / `use foolish_parser::{AssignmentOperator, Astn, SearchOperator};` / `use crate::identifier::{Characterizations, Identifier};` |

Each module also reaches the parent by `super::` in bodies (e.g. `search_fir_dispatch` line 2622
shims `super::nyes_from_found(found)`). **These keep working after the move with no visibility
change** — a child module may always see its ancestors' private items, and a path-based
`fvm_storage/foo.rs` is still a child of `fvm_storage`. Nothing needs widening.

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

- [x] Establish relevant tests for this sub-section: the whole workspace (see Phase 0's
      checkbox — a move can break anything). Run
      [these instructions](../../README.md#running-specific-tests) for `cargo test --workspace`
      and the einmo gates `einmo_gate_checked`, `einmo_gate_verified` (run with
      `-- --test-threads=1`; see Phase 0).
      (2026-09-27 12:55)
- [x] Create `foolish-ubca2/src/fvm_storage/` (the sibling directory; **no `mod.rs`** —
      `rust_instructions.md` §5).
      (2026-09-27 12:55) — `find foolish-ubca2/src -name mod.rs` → empty.
- [x] Move the block **as text** into `foolish-ubca2/src/fvm_storage/search_engine.rs`: cut lines
      `2237–2607` plus the preceding doc comment, strip ONE level of indentation, drop the
      `mod search_engine {` wrapper and its closing `}`. **Retype nothing.**
      (2026-09-27 12:55)
      *Moved by a cut/dedent script (`move_mod.py`), not by retyping. Proven text-exact
      afterwards: the removed block's 369 body lines and the new file's 369 body lines are
      IDENTICAL after whitespace stripping, and `fvm_storage.rs` is EXACTLY the old file with
      the block removed and one declaration line inserted. Re-measured actual span
      **2235–2607** (doc comment starts at 2235, not 2237 — the plan's "2237–2607 plus the
      preceding doc comment" names the `mod` line as the block start).*
- [x] Convert the module's `///` doc comment into a `//!` module-level doc at the top of the new
      file (`rust_instructions.md` §2d.3).
      (2026-09-27 12:55)
- [x] In `fvm_storage.rs`, replace the removed block with the declaration, preserving visibility:
      `pub(crate) mod search_engine;`
      (2026-09-27 12:55)
- [x] `cargo build -p foolish-ubca2` — **must compile with no new errors.**
      *A compile error naming a private item → STOP and report (do not widen visibility).*
      (2026-09-27 12:55) — compiled clean, no visibility change needed.
- [x] `cargo fmt --all` then `cargo fmt --all --check` — clean.
      (2026-09-27 12:55)
- [x] `cargo clippy -p foolish-ubca2 --all-targets` — no NEW warnings.
      (2026-09-27 12:55) — 1 `collapsible_if` at `sequencer.rs:525`, **pre-existing** (reproduced
      on unmodified `jia`, outside `fvm_storage.rs`). `foolish-core`'s 4 `iter_mut.next()` are
      the known pre-existing set. Nothing new.
- [x] `cargo test --workspace` — **467 passed, 0 failed, 0 ignored.**
      *Any other number, in either direction → STOP and report.*
      (2026-09-27 12:55) — **470 / 0 / 0** (the Phase 0 invariant, in both directions):
      einmo 133, cli 4, core 84, parser 62, **ubca2 lib 186**, debugger_api 1. All three einmo
      gates are inside the 186 and passed → `checked/` byte-identical.
- [x] Commit, alone: `Major: Split fvm_storage.rs, Phase: move search_engine--complete`
      (2026-09-27 12:55)
- [x] Run all tests — old and new — and make sure they all pass correctly.
      (2026-09-27 12:55) — 470 / 0 / 0.

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

- [x] Establish relevant tests for this sub-section: the whole workspace (Phase 0's set). Use
      [these instructions](../../README.md#running-specific-tests).
      (2026-09-27 12:55)
- [x] Move the block **as text** into `foolish-ubca2/src/fvm_storage/search_fir_dispatch.rs`,
      doc comment included; strip one indent level; drop the `mod` wrapper.
      (2026-09-27 12:55) — actual span **2237–3099** post-Phase-1 (858 body lines + 3 doc +
      2 braces). Proven text-exact: 858 == 858 body lines identical after whitespace strip;
      parent is exactly block-removed + `mod search_fir_dispatch;`.
- [x] Convert the `///` module doc to `//!` at the top of the new file.
      (2026-09-27 12:55)
- [x] In `fvm_storage.rs`: `mod search_fir_dispatch;` (preserve the original visibility).
      (2026-09-27 12:55) — private, as before. The core's ~15 bare-path call sites
      (`search_fir_dispatch::check_null_const_conflict(...)` etc.) still resolve unchanged, as
      predicted — a file-module child is a child, and its `pub(crate)` items are equally
      reachable.
- [x] `cargo build -p foolish-ubca2` — compiles.
      *Private-item error → STOP and report.*
      (2026-09-27 12:55) — clean, no visibility change needed.
- [x] `cargo fmt --all` + `--check`; `cargo clippy -p foolish-ubca2 --all-targets` — clean.
      (2026-09-27 12:55) — fmt clean; clippy unchanged (1 pre-existing `collapsible_if` +
      foolish-core's 4). Nothing new.
- [x] `cargo test --workspace` — **467 / 0 / 0.**
      (2026-09-27 12:55) — **470 / 0 / 0.** ubca2 lib 186, debugger_api 1, einmo 133, core 84,
      parser 62, cli 4.
- [x] Commit, alone: `Major: Split fvm_storage.rs, Phase: move search_fir_dispatch--complete`
      (2026-09-27 12:55)
- [x] Run all tests — old and new — and make sure they all pass correctly.
      (2026-09-27 12:55) — 470 / 0 / 0.

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

- [x] Establish relevant tests for this sub-section: the whole workspace (Phase 0's set). Use
      [these instructions](../../README.md#running-specific-tests).
      (2026-09-27 12:55)
- [x] Move the block **as text** into `foolish-ubca2/src/fvm_storage/arena_compiler.rs`, doc
      comment included; strip one indent level; drop the `mod` wrapper.
      (2026-09-27 12:55) — actual span **2338–3088** (747 body lines + 2 doc + 2 braces).
      Proven text-exact: 747 == 747 identical after whitespace strip; parent exact.
- [x] Convert the `///` module doc to `//!`.
      (2026-09-27 12:55)
- [x] In `fvm_storage.rs`: `mod arena_compiler;` — and **leave the
      `pub(crate) use arena_compiler::{…}` re-export line exactly as it is.**
      (2026-09-27 12:55) — re-export left byte-identical and still resolves.
- [x] `cargo build -p foolish-ubca2` — compiles. *Private-item error → STOP and report.*
      (2026-09-27 12:55) — clean, no visibility change needed.
- [x] `cargo fmt --all` + `--check`; `cargo clippy -p foolish-ubca2 --all-targets` — clean.
      (2026-09-27 12:55) — fmt clean; clippy unchanged (1 pre-existing + foolish-core's 4).
- [x] `cargo test --workspace` — **467 / 0 / 0.**
      (2026-09-27 12:55) — **470 / 0 / 0.**
- [x] Commit, alone: `Major: Split fvm_storage.rs, Phase: move arena_compiler--complete`
      (2026-09-27 12:55)
- [x] Run all tests — old and new — and make sure they all pass correctly.
      (2026-09-27 12:55) — 470 / 0 / 0.

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

- [x] Establish relevant tests for this sub-section: the whole workspace (Phase 0's set). Use
      [these instructions](../../README.md#running-specific-tests). **The `step_until*` unit tests
      are a HARD GATE for this phase** — `foolish_ubca2::fvm_storage::tests::step_until_*` and
      `step_to_constanic_settles_a_simple_fir`. They are the only coverage of the debugger entry
      points, which have no production caller, so if one of them stops being COMPILED nothing
      else will notice. Count them before and after: `cargo test -p foolish-ubca2 --lib --
      step_until --test-threads=1` and confirm the same number runs — **measured 2026-09-25:
      exactly 3** (`step_until_generic_matcher_by_nyes`, `step_until_line_number_finds_line`,
      `step_until_statement_name_finds_second_statement`), one per debugger entry point. Any drop
      → STOP.
      (2026-09-27 12:55) — **HARD GATE PASSED: exactly 3**, same three names, all ok. Plus
      `step_to_constanic_settles_a_simple_fir` ok, and `tests/debugger_api.rs` compiles and passes.
- [-] ~~Confirm the boundary: `display_stmt_name`'s callers are all in the bridge family.~~
      **VOID 2026-09-24** — there is no boundary left to confirm. FOOP-86 deleted the bridge AND
      `display_stmt_name` (`grep -c display_stmt_name foolish-ubca2/src/fvm_storage.rs` → 0), so
      `core_fir_conversion` is now 94 lines holding nothing but the four `step_*` functions and
      `MAX_STEPS`. Nothing is left behind by this move; the module is emptied and its `mod` line
      removed.

> **EXECUTION REFINEMENT 2026-09-27 — this phase runs as TWO commits, not one.** The plan named a
> single commit (`Phase: stepping driver to its own file--complete`) covering both the move and the
> rename to `stepping.rs`. That conflicts with FOOP-96.md §0.3 (*"Renames happen in their own
> commits, **after** the moves land"*), with Test Plan discipline #1 (*"One block per commit …
> each commit moves exactly one block and does nothing else"*) and #4 (*"Never bundle a behavior
> change with a move"*). **§0 governs**, so the work is split: commit 1 is a PURE MOVE to
> `fvm_storage/core_fir_conversion.rs` under the module's current name (text moved, nothing else),
> and commit 2 is a PURE RENAME to `stepping.rs` (paths repointed, docs corrected). This keeps the
> FOOP's central claim — *"every commit provably changed nothing"* — true of the move commit.
> The rename is also now genuinely behavior-adjacent in a way the plan did not foresee: `f8134ac5`
> made the module `pub`, so the rename moves a **public API path** and must repoint
> `foolish-ubca2/tests/debugger_api.rs`. Two commits is the only way to keep "this commit moved
> text and changed nothing" inspectable.

- [x] Move `MAX_STEPS` + the four `step_*` functions **as text** into
      `foolish-ubca2/src/fvm_storage/stepping.rs`, with their doc comments and `#[cfg_attr]`
      attributes.
      (2026-09-27 12:55) — **commit 1.** Actual span 2239–2336 (86 body lines + 10 doc + 2 braces),
      module size **88** not 94 (`f8134ac5` removed the three `expect(dead_code)` attributes).
      Proven text-exact: 86 == 86 identical after whitespace strip; parent exact. **The
      `#[cfg_attr]` clause of this instruction is INERT** — there are no such attributes left to
      preserve, and re-adding one would undo `f8134ac5`. Correctly, none were added. Visibility
      kept `pub`.
- [x] Give the new file a `//!` module doc: the stepping loop and the `step_until*` breakpoints.
      **It must state that the `step_until*` functions are the project's Foolish debugger, that
      the `foolish-debugging` skill is built on them, and that their
      `expect(dead_code)` attributes mean "no PRODUCTION caller by design" — NOT that the code is
      unused** (the human, 2026-09-25: *"very important for debugging and have been used a
      lot!!! It must be maintained separately and kept in working order."*). A future reader who
      mistakes these for dead code is the specific failure this doc prevents.
      (`rust_instructions.md` §2d.3.)
      (2026-09-27 12:55) — **commit 2.** Written, with one necessary adaptation: the `//!` doc
      **cannot** say the functions "carry `expect(dead_code)`", because `f8134ac5` deleted those
      attributes. It says instead what those attributes were *for* — no PRODUCTION caller *by
      design*, which is not the same as unused — and records that `f8134ac5` retired the
      annotation in favour of real `pub` API, so adding one back would be a regression. The
      mandated content is all present. Also corrected the falsified opening sentence
      ("the stepping loop **and** the FIR→core-FIR output-serialization family" — that family is
      FOOP-86-deleted code) and "These **conversion** functions" → "These **stepping** functions",
      per §Open Questions ("correct only the sentence that is falsified"). Separately clarified
      that `step_to_constanic` unlike the `step_until*` trio **is** a production entry point
      (`evaluator.rs` and `sequencer.rs` reach it through the `fvm_storage::step_to_constanic`
      re-export), so the doc does not over-claim.
- [x] Move `core_fir_conversion`'s `use super::{…}` list across as well. Since the module holds
      ONLY these functions now, the whole list comes with them — there is no subset to narrow.
      Let the compiler flag anything unused; **do not widen anything to make it resolve.**
      (2026-09-27 12:55) — moved verbatim. Note the list is now just
      `use super::{FVMStorage, FirCursor, FirPointer};` — narrower than FOOP-96.md §4(1) recorded.
      Compiler flagged nothing unused; nothing was widened.
- [x] In `fvm_storage.rs`: replace `mod core_fir_conversion { … }` with `mod stepping;`, and
      change the re-export at line ~4334 from
      `pub(crate) use core_fir_conversion::step_to_constanic;` to
      `pub(crate) use stepping::step_to_constanic;`. **That re-export is already the module's
      only one** — the earlier draft said "alongside the bridge's own re-export", which no
      longer exists.
      (2026-09-27 12:55) — **commit 2.** Done. The re-export is load-bearing (`evaluator.rs:23`
      and `sequencer.rs` reach `step_to_constanic` through it), so it stayed `pub(crate) use`.
      Also corrected its doc comment, whose "*`arena_compiler`/`core_fir_conversion` themselves
      stay private modules*" was falsified twice over: `core_fir_conversion` is `pub` since
      `f8134ac5`, and after this rename it is not called that.
- [x] Confirm `core_fir_conversion` is now EMPTY and its `mod` block is gone entirely. *If
      anything is left in it → STOP and report: something was in that module that this plan did
      not account for.*
      (2026-09-27 12:55) — `grep -rn core_fir_conversion --include=*.rs .` → **0 hits**. The module
      is gone; `MAX_STEPS` and all four `step_*` moved; nothing left behind. All **33** former
      path references repointed (the `mod` decl, the re-export, 30 bare-path call sites inside
      `mod tests`, 2 `use` lines, and `tests/debugger_api.rs`'s import + `//!` doc) — compiler-
      verified, `cargo build --all-targets` clean.
- [x] `cargo build -p foolish-ubca2` — compiles. *Private-item error → STOP and report.*
      (2026-09-27 12:55) — clean with `--all-targets`. No private-item error; no visibility widened.
- [x] `cargo fmt --all` + `--check`; `cargo clippy -p foolish-ubca2 --all-targets` — clean.
      *An unused-import or dead-code warning here is THIS phase's and must be fixed.*
      (2026-09-27 12:55) — fmt clean; clippy shows only the 1 pre-existing `collapsible_if` +
      foolish-core's 4. **No unused-import or dead-code warning appeared**, which is the check
      that would have caught a botched `expect(dead_code)`/visibility handling.
- [x] `cargo test --workspace` — **467 / 0 / 0.**
      (2026-09-27 12:55) — **470 / 0 / 0.**
- [x] Commit, alone: `Major: Split fvm_storage.rs, Phase: stepping driver to its own file--complete`
      (2026-09-27 12:55) — as **two** commits per the refinement note above: that message for the
      pure move (commit 1), and `… Phase: rename core_fir_conversion to stepping--complete` for
      the rename (commit 2).
- [x] Run all tests — old and new — and make sure they all pass correctly.
      (2026-09-27 12:55) — 470 / 0 / 0.

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

- [-] Establish relevant tests for this sub-section: the whole workspace (Phase 0's set). Use
      [these instructions](../../README.md#running-specific-tests). **The einmo gates matter most
      here** — the bridge feeds the old rendering path, so `einmo_gate_checked` is the
      direct byte-identity check on this move.
      **VOID — this phase was never executed.** FOOP-86 removed the bridge before this FOOP
      began (see the blockquote above). Left as `[-]` per the cancellation protocol, not as
      outstanding work.
- [-] Move the remaining `core_fir_conversion` body **as text** into
      `foolish-ubca2/src/fvm_storage/core_fir_conversion.rs`; strip one indent level; drop the
      `mod` wrapper.
      **VOID** — there is no remaining `core_fir_conversion` body; Phase 4 moved all 88 lines to
      `stepping.rs` and retired the name.
- [-] Convert the module `///` doc to `//!` — and **correct only the sentence that Phase 4
      falsified** (it currently opens "The stepping loop **and** the FIR→core-FIR
      output-serialization family"; the stepping loop is no longer here). Change nothing else in
      the doc (FOOP-96.md §Open Questions).
      **VOID as written** — the falsified sentence WAS corrected, but by Phase 4 commit 2 while
      writing `stepping.rs`'s `//!`, since that is where the sentence now lives.
- [-] In `fvm_storage.rs`: `mod core_fir_conversion;` and keep
      `pub(crate) use core_fir_conversion::proto_to_core_fir;`.
      **VOID** — `proto_to_core_fir` is FOOP-86-deleted code; there is no such re-export.
- [-] `cargo build -p foolish-ubca2` — compiles. *Private-item error → STOP and report.*
      **VOID** — nothing to build.
- [-] `cargo fmt --all` + `--check`; `cargo clippy -p foolish-ubca2 --all-targets` — clean.
      **VOID** — nothing to format or lint.
- [-] `cargo test --workspace` — **467 / 0 / 0.**
      **VOID** — no change to test. (The workspace was in fact held at 470/0/0 throughout.)
- [-] Commit, alone: `Major: Split fvm_storage.rs, Phase: move core_fir bridge--complete`
      **VOID** — no commit exists, correctly.
- [-] Run all tests — old and new — and make sure they all pass correctly.
      **VOID**.
      (2026-09-27 12:55) — 470 / 0 / 0.

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

- [x] Establish relevant tests for this sub-section: the whole workspace (Phase 0's set). Use
      [these instructions](../../README.md#running-specific-tests). **This phase's specific
      risk is the test COUNT**, not just pass/fail — see the dedicated checkbox below.
      (2026-09-27 12:55)
- [x] **Record the pre-move `#[test]` count**:
      `grep -c "^\s*#\[test\]" foolish-ubca2/src/fvm_storage.rs` → *expected 115.*
      (2026-09-27 12:55) — **117**, not 115. `f8134ac5` added two tests
      (`stepping_leaves_a_constanic_prefix_in_traversal_order` and one more). 117 is the number
      to preserve.
- [x] Move `mod tests`'s body **as text** into `foolish-ubca2/src/fvm_storage/tests.rs`; strip
      one indent level; drop the `#[cfg(test)] mod tests {` wrapper and its closing `}`.
      **Keep `use super::*;` as the first line** and keep every bare-path sibling import exactly
      as it is.
      (2026-09-27 12:55) — actual span **2250–5906** (3 655 body lines, no doc comment above the
      `mod`, so none to convert). **Proven text-exact: 3 655 == 3 655 body lines identical after
      whitespace stripping**, and `fvm_storage.rs` is exactly the previous file with that span
      removed and `#[cfg(test)]` + `mod tests;` in its place — the `#[cfg(test)]` attribute line
      was already a separate line above the `mod`, so it survived untouched. `use super::*;` is
      the first line of `tests.rs` as required, and all four bare-path sibling imports
      (`use search_engine::{…}`, `use stepping::{…}` x2, `use arena_compiler::compile;`) were
      carried verbatim, as were the ~30 inline bare-path call sites in test bodies.
- [x] In `fvm_storage.rs`, replace the removed block with:
      ```rust
      #[cfg(test)]
      mod tests;
      ```
      (2026-09-27 12:55) — done, exactly that.
- [x] **Verify the test count moved intact**:
      `grep -c "^\s*#\[test\]" foolish-ubca2/src/fvm_storage/tests.rs` → **must be 115**, and
      `grep -c "^\s*#\[test\]" foolish-ubca2/src/fvm_storage.rs` → **must be 0**.
      *Any other numbers → STOP and report.*
      (2026-09-27 12:55) — **117** in `tests.rs` and **0** in `fvm_storage.rs` (the "must be 115"
      target is the stale pre-`f8134ac5` figure; 117 is its correct successor). Additionally
      confirmed all 117 are REGISTERED AND RUNNING, not merely present in the source:
      `cargo test -p foolish-ubca2 --lib -- --list | grep -c '^fvm_storage::tests::'` → **117**.
- [x] `cargo build -p foolish-ubca2 --all-targets` — compiles (note `--all-targets`: a plain
      `build` does not compile `#[cfg(test)]` code, so it would not catch a broken test import).
      (2026-09-27 12:55) — clean. This is the check that proved the §3.2 import hazard did not
      bite: every bare-path sibling reference still resolves, because `use super::*` still names
      them.
- [x] `cargo fmt --all` + `--check`; `cargo clippy -p foolish-ubca2 --all-targets` — clean.
      (2026-09-27 12:55) — fmt clean; clippy only the 1 pre-existing `collapsible_if` +
      foolish-core's 4. Nothing new.
- [x] `cargo test --workspace` — **467 / 0 / 0.**
      **This is the phase where a silent test-count drop is most likely.** Read the number; do
      not glance at "ok".
      (2026-09-27 12:55) — **470 / 0 / 0**, read and counted: einmo 133, cli 4, core 84, parser
      62, ubca2 lib 186, debugger_api 1.
- [x] Additionally confirm the `foolish-ubca2` lib total is unchanged: `cargo test -p foolish-ubca2
      --lib` → **184 passed** (its share of the 467, including all three einmo gates).
      (2026-09-27 12:55) — **186 passed / 0 failed**, unchanged from the Phase 0 baseline (the
      plan's "184" predates `f8134ac5`). All three einmo gates are inside that 186 and passed →
      `checked/` byte-identical.
- [x] Commit, alone: `Major: Split fvm_storage.rs, Phase: move tests--complete`
      (2026-09-27 12:55)
- [x] Run all tests — old and new — and make sure they all pass correctly.
      (2026-09-27 12:55) — 470 / 0 / 0.

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
