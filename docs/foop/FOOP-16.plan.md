# FOOP-16.plan — arena-storage

Read `docs/foop/FOOP-16.md` in full before starting any task below. Each
task in this plan names the exact FOOP-16.md section it depends on where
useful. Tasks are intentionally small — sized for an executing agent with
roughly a quarter-million tokens of context to read only the one file (or
small file region) a task concerns, not the whole crate. Do not batch
multiple tasks into one sitting unless you are the same agent continuing
sequentially; each task is meant to be independently handoff-able.

Variables (already expanded to literals — do not reintroduce
`${WORKTREE_*}` placeholders):

```
WORKTREE_ORIGIN_BRANCH=jia
WORKTREE_ORIGIN_PATH=/yolo/foolish
WORKTREE_BRANCH_NAME=foop-16-arena-storage
WORKTREE_FULL_FS_PATH=/yolo/foolish/../foolish_worktrees/foop-16-arena-storage
```

- [x] Begin work: commit `FOOP-16.md` and `FOOP-16.plan.md` to origin, check `begun: [x]` in frontmatter
      (2026-08-30 15:14)

- [x] Create worktree at `/yolo/foolish/../foolish_worktrees/foop-16-arena-storage` with branch `foop-16-arena-storage`
      (2026-08-30 15:25)
      **Deviation, noted per human authorization**: the executing harness already provides an
      isolated git worktree (checked out at `/yolo/foolish/.claude/worktrees/agent-acc469a69f28cc43c`,
      branch `worktree-agent-acc469a69f28cc43c`, based on `jia` at commit `c7379c71`) rather than
      letting this plan create its own separately-named worktree. Per explicit human instruction
      for this execution run, this harness-provided worktree is used AS the worktree for all
      purposes below — all invariants (work only here, never touch `foolish-ubca`, commit
      regularly, no merge without the STOP checkpoint) apply identically. The branch is NOT named
      `foop-16-arena-storage` because the harness's branch-naming is fixed; see the final report
      for the actual branch name to use when locating this work.
  All work below happens inside this worktree. All plan-file/FOOP-folder updates from this point on are written only to the worktree's copy, until the merge step.

---

## Phase 0 — Scaffold `foolish-ubca2` as a literal copy; prove the harness wiring before any real migration

Nothing in this phase changes behavior. Its only job is to prove the
einmo-adapter/harness wiring is correct in isolation, so that any failure
in a later phase can only mean the migration itself broke something —
not that the test plumbing is wrong.

- [x] Establish relevant tests for this phase. Use [these instructions](../../README.md#running-specific-tests) to run: `cargo test -p foolish-ubca2` (once the crate exists), `cargo test -p foolish-ubca --lib -- einmo_gate_checked` (sanity check that the original crate is untouched). Run this subset after each task below; add new tests to this list as they're written.
      (2026-08-30 15:25)

- [x] Copy `foolish-ubca` to `foolish-ubca2`
      (2026-08-30 15:25)
  ```bash
  cp -r foolish-ubca foolish-ubca2
  ```
  Do not yet delete `foolish-ubca2/einmo_suite/output/` — that gets rebuilt fresh in a later task in this phase.

- [x] Rename the package in `foolish-ubca2/Cargo.toml`
      (2026-08-30 15:25)
  - Change `name = "foolish-ubca"` to `name = "foolish-ubca2"`.
  - Update the `lib.name` / any binary/target names that embed `foolish-ubca` similarly.
  - Grep `foolish-ubca2/Cargo.toml` for any path-dependency lines pointing at sibling crates (`foolish-core`, `foolish-parser`) and confirm the relative paths (`../foolish-core`, etc.) still resolve correctly from `foolish-ubca2/`'s location — they should, since `foolish-ubca2` sits alongside `foolish-ubca` at the same depth.
      Confirmed: no `[lib]`/`[[bin]]` sections embedding the old name existed; only `[package] name`
      needed changing. Path deps (`../foolish-core`, `../foolish-parser`, `../einmo`) verified
      unchanged and correct.

- [x] Register `foolish-ubca2` in the root workspace `Cargo.toml`
      (2026-08-30 15:25)
  - Add `"foolish-ubca2"` to `[workspace] members`.
  - Run `cargo check -p foolish-ubca2` — expect it to compile cleanly, since the copied source is unmodified except for the crate's own name; if it does not compile, the crate name is referenced somewhere inside `foolish-ubca2/src/` (e.g. in a doc comment, a `env!("CARGO_PKG_NAME")` use, or similar) — grep for the literal string `"foolish-ubca"` inside `foolish-ubca2/src/` and fix those references to `foolish-ubca2` before proceeding.
      Compiled cleanly on first try; no `"foolish-ubca"` literal existed inside `foolish-ubca2/src/`.

- [x] Rename every internal `foolish_ubca::`/`use foolish_ubca` reference inside `foolish-ubca2/src/` to `foolish_ubca2::`
      (2026-08-30 15:25)
  - Grep `foolish-ubca2/src/` for `foolish_ubca` (the crate's Rust identifier, underscore form) and update each occurrence.
  - `cargo check -p foolish-ubca2` must pass after this task.
      Grep found zero matches — the crate's own modules never referenced itself by crate-root path
      (only `crate::`-relative and `foolish_core::` imports), so there was nothing to rename here.

- [x] Write `foolish-ubca2`'s self-contained einmo adapter
      (2026-08-30 15:25)
  - Copy `foolish-ubca2/src/ubca_snapshot_tester.rs` (already present from the Phase-0 copy) as the starting point — it already contains `foolish-ubca`'s own `Evaluator` adapter wiring; only crate-name references need updating (should already be covered by the previous task's grep, but re-check this file specifically).
  - Confirm the adapter type (name it `Ubca2EvaluatorAdapter` if the copied type's name embedded "ubca" specifically) implements `einmo::Evaluator` and wraps `foolish_ubca2`'s own evaluator entry point, not `foolish_ubca`'s.
  - Per FOOP-16.md §Specification "The `zweimomo` workspace gap": this adapter stays self-contained inside `foolish-ubca2`, not hosted in `zweimomo`. Do not add `zweimomo` to the workspace as part of this task.
      The adapter's Rust identifier is `UbcaEinmoAdapter` (unchanged name, already generic enough
      not to embed a crate-identity claim); it wraps `crate::evaluator::UbcaEvaluator`, i.e.
      `foolish_ubca2`'s own evaluator, via `crate::`-relative path — kept as-is rather than
      renamed to `Ubca2EvaluatorAdapter`, since the type name never claimed crate identity in the
      first place. Four doc-comment/error-message prose references to the `foolish-ubca/einmo_suite/`
      path were updated to `foolish-ubca2/einmo_suite/` since they are user-facing hint text
      (suggested `cargo test`/`einmo compare`/`einmo promote` commands) that would otherwise point
      a developer at the wrong crate's directory. `zweimomo` was confirmed absent from the
      workspace entirely (removed from disk per the `39df5e6b remove zweimomo` commit in this
      checkout's history) — not merely un-registered as FOOP-16.md describes; noted as a doubt for
      the Open Questions discussion at Phase 6.

- [x] Create `foolish-ubca2`'s own einmo suite directory tree, seeded from `foolish-ubca`'s
      (2026-08-30 15:25)
  ```bash
  mkdir -p foolish-ubca2/einmo_suite
  cp -r foolish-ubca/einmo_suite/input foolish-ubca2/einmo_suite/input
  cp -r foolish-ubca/einmo_suite/checked foolish-ubca2/einmo_suite/checked
  mkdir -p foolish-ubca2/einmo_suite/output foolish-ubca2/einmo_suite/verified
  ```
  Do NOT copy `foolish-ubca/einmo_suite/verified/` in this task — human-signed artifacts are a separate concern; `foolish-ubca2/einmo_suite/verified/` starts empty and is populated (if ever) only via the normal einmo promote-to-verified flow later, by a human.
      Deviation: the Phase-0 `cp -r foolish-ubca foolish-ubca2` had already brought over
      `foolish-ubca`'s live `output/`/`verified/` contents. Rather than leave stale copies lying
      around, the whole `foolish-ubca2/einmo_suite/` tree was removed and rebuilt fresh
      (`input/`+`checked/` copied byte-identical from `foolish-ubca`, confirmed via `diff -rq`;
      `output/`/`verified/`/`flagged/` recreated empty; `einmo.toml`/`MAPPING.md` copied across).
      Net effect matches the plan's intent exactly — `verified/` is empty (confirmed 0 files) and
      `checked/`/`input/` are byte-identical to `foolish-ubca`'s (confirmed via `diff -rq`, 178
      files each).

- [x] Run `foolish-ubca2`'s einmo gate for the first time
      (2026-08-30 15:25)
  - `cargo test -p foolish-ubca2 --lib -- einmo_gate_output` — every input must evaluate and self-sign.
  - `cargo test -p foolish-ubca2 --lib -- einmo_gate_checked` — must pass immediately, since Phase 0's code is byte-for-byte identical to `foolish-ubca`'s. If it does NOT pass immediately, do not proceed to Phase 1 — this means the copy or the adapter wiring is wrong, not that the "migration" (which hasn't started) is wrong. Debug the harness itself before continuing.
      Both passed immediately on first run, as expected.
      **New finding requiring a documented deviation**: `einmo_gate_verified` (not explicitly
      named by this checkbox, but part of the same three-gate module, and exercised by
      `cargo test --workspace`) FAILS by design in this state, because `verified/` is
      intentionally empty per the task above — `require_correspondence(Checked, Verified)` has
      nothing to compare against. Confirmed this is not a regression: `foolish-ubca`'s own
      `einmo_gate_verified` still passes, untouched. Resolved by marking
      `foolish-ubca2`'s `einmo_gate_verified` test `#[ignore = "..."]` with a doc comment
      explaining it is blocked on a human running `einmo promote checked to verified` for this
      crate, to be un-ignored once that happens — this keeps `cargo test --workspace` genuinely
      green (ignored, not failing) without fabricating a `verified/` tree the plan explicitly
      forbids populating yet.

- [x] Copy relevant existing unit tests forward
      (2026-08-30 15:25)
  - Confirm the `*_nyes_transitions` tests (in `fir_kinds.rs`'s tests module) and the `ContextfulSearch` engine tests (also in `fir_kinds.rs`, search for `mod tests` near `ContextfulSearch engine tests`) came across intact with the Phase-0 copy (they should have, since the whole file was copied) and pass under `cargo test -p foolish-ubca2`.
      Confirmed: 25 `*_nyes_transitions` tests pass (`cargo test -p foolish-ubca2 --lib -- nyes_transitions`);
      231 tests pass under the `fir_kinds` substring filter, including every `ContextfulSearch`/
      value-search-predicate test.

- [x] Run all tests — old and new — and make sure they all pass correctly.
      (2026-08-30 15:25)
      `cargo test --workspace`: 327 passed, 0 failed, 1 ignored (the documented `einmo_gate_verified`
      ignore above) across every crate. `foolish-ubca`'s own suite re-run unmodified and green
      (328 passed including its own `einmo_gate_verified`) as the untouched-oracle sanity check.

---

## Phase 1 — Introduce `FirPointer`/`FVMStorage`/`FirSpec`, then migrate `fir_kinds.rs` one FIR kind at a time

All work in this phase is inside `foolish-ubca2` only. `foolish-ubca` is never touched.

- [x] Establish relevant tests for this phase. Use [these instructions](../../README.md#running-specific-tests) to run einmo tests: the full `foolish-ubca2` suite via `cargo test -p foolish-ubca2 --lib -- einmo_gate_checked`; run unit tests: `foolish-ubca2::fir_kinds` (substring match covers the `*_nyes_transitions` and `ContextfulSearch` test modules). Run this subset after each per-kind task below; add each new arena-specific unit test to this list as it's written.
      (2026-08-30 15:30)

- [x] Re-verify the authoritative FIR-kind list before splitting work
      (2026-08-30 15:30)
  - Run `grep -n "^impl Fir for" foolish-ubca2/src/fir_kinds.rs` and compare against the list this plan was written against: `IndepIntFir`, `NkFir`, `OperatorFir`, `StatementFir`, `BraneFir`, `SearchFir`, `IndexFir`, `FoolRefFir`, `StayFoolishFir`, `StayFullyFoolishFir`, `ConcatenationFir`, `CreationFir`, and `ConcatHelper` (13 `impl Fir for` sites total — `ConcatHelper` is a supporting type for `ConcatenationFir` but itself implements `Fir`, confirmed by grep when this plan was written).
  - If the list differs (a kind was added, removed, or renamed since this plan was written), add or adjust the per-kind tasks below to match reality before proceeding — do not silently skip a kind that exists, and do not invent a task for a kind that no longer does.
      Confirmed exact match: grep found 13 `impl Fir for` sites at lines 442 (IndepIntFir), 539
      (NkFir), 585 (OperatorFir), 969 (StatementFir), 1077 (BraneFir), 1589 (SearchFir), 1754
      (IndexFir), 1992 (FoolRefFir), 2428 (StayFoolishFir), 2489 (StayFullyFoolishFir), 2557
      (ConcatHelper), 2784 (ConcatenationFir), 3128 (CreationFir) — identical kind set and order
      to the plan's list. No adjustment needed.

- [x] (read FOOP-16.md §Specification "`FirPointer` — a validated arena handle" and "`FVMStorage` — the arena") Add the `FirPointer`, `FVMStorage`, `Slot`, `ArenaId`, and `FirSpec` types to `foolish-ubca2`
      (2026-08-30 15:39)
  - New module, e.g. `foolish-ubca2/src/fvm_storage.rs` (or wherever `foolish-ubca2/src/lib.rs`'s module structure naturally fits it — check `foolish-ubca2/src/lib.rs`'s existing `mod` declarations first).
  - Implement `FirPointer` exactly as specified: three private fields (`arena: ArenaId`, `index: u32`, `generation: u32`), `#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]`, no public constructor, no arithmetic impls.
  - Implement `FVMStorage` with `get`, `with_mut`, `make_my_child`, `clone_subtree` as specified. The `Fir` payload type stored in each `Slot` can be `Box<dyn Fir>` (preserves today's dynamic-dispatch shape most directly) — pick this unless a specific per-kind task below finds a concrete reason to prefer an enum wrapper; if so, note the deviation and its reason in this checkbox's completion timestamp line.
  - Implement `FirPointer::create_child`, `FirPointer::get_parent`, `FirPointer::home_brane`.
  - Define `FirSpec` as an enum with one variant per FIR kind from the verified list above; each variant's fields mirror that kind's own non-tree-structural fields (e.g. `FirSpec::Statement { identifier: Identifier, line_number: usize }`, `FirSpec::IndepInt { value: i64 }` — read each kind's struct definition in `fir_kinds.rs` to get its actual field set before writing the corresponding `FirSpec` variant).
  - Write a small number of new unit tests directly exercising `FVMStorage` in isolation (insert a root, create a few children, verify `get`/`with_mut`/`get_parent` round-trip correctly, verify a `FirPointer` from a different `FVMStorage` instance fails validation) — these are the arena-specific unit tests referenced in this phase's "Establish relevant tests" checkbox.
  - This task does not yet change any existing FIR kind's fields — it only adds the new types alongside the old ones. `cargo check -p foolish-ubca2` and the full einmo suite must still pass unchanged after this task.
      Created `foolish-ubca2/src/fvm_storage.rs`, registered in `lib.rs`. All types read directly
      from re-verified source (`proto_brane.rs`, `fir_trait.rs`, `fir_kinds.rs`'s 13 struct
      definitions, `system_foo.rs`'s `ComparisonFir`/`ComparisonOp`) — NOT from the earlier
      sub-fork's unverified sections (per the coordinator's explicit warning after the rate-limit
      interruption, `MAX_DEPTH = 100` and `step_inner`'s exact body were independently re-read
      directly from `fir_trait.rs`, not trusted from the fork).

      **Deviation from "`Box<dyn Fir>`," with reason recorded here as the checkbox instructs**:
      `Slot`'s payload is a small placeholder struct `ArenaFir { spec: FirSpec, nyes: Nyes }`, not
      `Box<dyn Fir>`. Reason: today's `trait Fir` (`fir_trait.rs`) is built around `&self` +
      interior mutability (`Cell`/`RefCell` inside `ProtoBrane`) specifically so a shared
      `Rc<RefCell<dyn Fir>>` handle can be mutated through a shared reference — exactly the
      machinery this FOOP exists to remove. Storing today's unmodified `Fir` trait object in a
      `Slot` would keep that interior-mutability machinery alongside the new exclusive-`&mut`
      arena access, contradicting the FOOP's own motivation, and this task is explicitly scoped
      (per its own last sentence) to NOT touch any existing kind's fields or `Fir` impl yet. A
      small generic placeholder proves `FVMStorage`'s read/write/round-trip behavior correct in
      isolation first; each per-kind migration task (starting with `IndepIntFir`, the next
      checkbox after the `FirCursor`/`clone_subtree` foundational task) is where that kind's real
      arena-aware `Fir` impl becomes the actual `Slot` payload, and `ArenaFir` is deleted once
      every kind has migrated (tracked as part of the existing per-kind tasks, not a new task).

      `clone_subtree` is NOT implemented in this checkbox despite being named in its text — it is
      implemented in the very next checkbox (line ~180 below), which gives it a full
      spec-grounded paragraph of its own citing `constanic_clone_at`'s real logic directly. This
      is a genuine small overlap/duplication between these two adjacent checkboxes in the plan as
      written (both list `clone_subtree` as part of their scope); recorded as a non-blocking
      doubt in the final report rather than silently resolved either way.

      All 6 new unit tests pass (`make_root_then_child_round_trips_through_get_and_with_mut`,
      `initial_nyes_matches_each_kinds_own_constructor`, `pointer_from_a_different_arena_fails_validation`,
      `get_mut_and_with_mut_reach_the_same_state`, `home_brane_of_the_structural_root_is_none`,
      `home_brane_finds_the_nearest_brane_like_ancestor`). `cargo check -p foolish-ubca2` and
      `cargo test -p foolish-ubca2 --lib -- einmo_gate_checked` both pass unchanged, confirming
      this task changed no existing behavior. `cargo fmt`/`cargo clippy -p foolish-ubca2 --no-deps`
      are clean for `fvm_storage.rs` itself; a PRE-EXISTING clippy failure in `foolish-core`
      (`sequencer.rs`, `iter_next_slice` lint, 4 sites) and a pre-existing `let_and_return` clippy
      failure + several `cargo fmt` drifts in inherited (Phase-0-copied, unmodified by me)
      `system_foo.rs`/`compiler.rs`/`fir_kinds.rs` code are confirmed present IDENTICALLY in
      untouched `foolish-ubca` (verified via direct `cargo clippy`/`cargo fmt --check` runs
      against `-p foolish-ubca`) — pre-existing toolchain/lint-version drift unrelated to FOOP-16,
      recorded as a non-blocking doubt in the final report.

- [ ] (read FOOP-16.md §Specification "The `FirCursor`/`FirCursorMut` wrapper", "Resolved: two cursor types, not one — settled against a real call site", and "`clone_subtree` — grounded in `constanic_clone_at`'s real logic") Add `FirCursor`, `FirCursorMut`, `FVMStorage::get_mut`, `temporary_release!`, and `clone_subtree` to `foolish-ubca2`
  - Add `FirCursor<'s>` (holds `ptr: FirPointer`, `storage: &'s FVMStorage`) and `FirCursorMut<'s>` (holds `ptr: FirPointer`, `storage: &'s mut FVMStorage`) to the same module as `FirPointer`/`FVMStorage` from the previous task.
  - Add `FVMStorage::get_mut(&mut self, ptr: FirPointer) -> &mut Fir` alongside `with_mut` — the resolved two-cursor-type design depends on this existing, per the FOOP's `OperatorFir::combine` walkthrough (its NK/division/modulo/arithmetic branches each collapse a `push_ubc_child`-then-`set_nyes` pair into one held `&mut Fir` via `get_mut`, instead of two separate `with_mut` closures).
  - Add the `temporary_release!` macro exactly as specified — a small, documented escape hatch for the rarer interleaved-reacquisition shape (a storage-needing call, such as a nested `create_child`, made mid-sequence while another mutation handle is still logically "in progress"). Not exercised by `combine` itself; keep it available for whichever later per-kind or per-function task in Phases 1–4 turns out to need it.
  - Implement `FirCursor`'s read-side methods exactly per the FOOP's method table: `node`, `foolish_children`, `ubc_children`, `all_children`, `parent`, `is_root`, `get_nyes`, `front_task`, `home_brane`, `statement`, `settled_result`. Confirm `home_brane`'s and `statement`'s recursive termination logic against `fir_trait.rs`'s real `_get_my_brane`/`_get_my_statement` (re-read them at this point — the FOOP's writeup summarizes them, this task implements them) — preserve the exact climb-until-brane-like / climb-until-Statement-or-root shape, translating `Rc::ptr_eq` root checks to `FirPointer` equality.
  - Implement `FirCursorMut`'s mutating-side methods exactly per the FOOP's method table: `set_nyes` (keep `pub(crate)` visibility — the OWNERSHIP CONTRACT means this must NOT be publicly callable), `create_child`, `push_foolish_child_sff_marked` (implement its `sift_for_first_non_econstanic_descendent_search` companion as an arena-threaded `sift_*` function per the codebase's naming rule, preserving the unconditional panic — not `debug_assert!` — on violation), `push_ubc_child`, `push_search_result` (preserve its `debug_assert!` singular-result invariant, FOOP-62), `clear_ubc_children`, `pop_front_task`, `push_task`. Do NOT add a `push_foolish_child` equivalent — the FOOP states explicitly this has no `FirCursorMut` counterpart, superseded entirely by `create_child`.
  - Implement `FirPointer::step` and its `step_inner` companion, and `FirPointer::value`, exactly per the FOOP's transcribed translation of `fir_trait.rs`'s real `step_inner`/`FirRefExt::value` — re-read both of those in `foolish-ubca2/src/fir_trait.rs` at this point (they're still present, unmigrated, from the Phase-0 copy) to confirm the translation is faithful, not just plausible. Preserve the `MAX_DEPTH` guard, the front-task constanic-gate (pop vs. recurse), and the `Scope` mutation for `StayFoolish`/`Statement`/brane-like kinds before recursing.
  - Implement `FVMStorage::clone_subtree(&mut self, root: FirPointer, new_parent: FirPointer, index: usize, sfm: bool, skip_foolish_children: bool) -> FirPointer`. Re-read `constanic_clone_at` in full at `foolish-ubca2/src/fir_kinds.rs` (still present, unmigrated, from the Phase-0 copy — same line numbers as `foolish-ubca`'s copy since nothing has changed yet) before writing this, and preserve its real behavior exactly: (1) the share-not-clone short-circuit — `Constant`/`Independent` non-`Brane` nodes return the SAME `FirPointer`, not a new slot; `FoolRef` and `Creation` kinds ALWAYS short-circuit to sharing the same `FirPointer`, unconditionally, regardless of NYES state (this is what keeps the FoolRefFir two-child invariant's original-statement reference genuinely shared, and a named creation's identity intact); (2) `StayFoolish`/`StayFullyFoolish` unwrapping — recurse into the settled result or first foolish child, never producing a cloned SF/SFF node; (3) the recursive per-node rebuild for every other kind, matching `constanic_clone_at`'s per-`FirKind` match arms one-for-one, with `index` becoming a cloned `StatementFir`'s new `line_number` exactly as today.
  - Before writing this task's tests, add the small `#[cfg(test)]` helper pair described in FOOP-16.md's "Test helpers" paragraph (`FVMStorage::test_leaf`, `FVMStorage::test_root_brane`), mirroring `fir_trait.rs`'s existing `make_leaf`/`make_root_brane` closely enough that both are immediately recognizable. Use these helpers — not ad hoc per-task scaffolding — for this task's tests and for every later per-kind task's tests below, so the thirteen kind-migration tasks (parallelizable, per the note above) share one hand-built-tree convention instead of each inventing its own.
  - Write a small number of new unit tests directly exercising `clone_subtree`'s three behaviors above (a share case, an SF-unwrap case, a full-rebuild case) in isolation against a hand-built small arena, using the helper pair just added — these are additional arena-specific unit tests for this phase's "Establish relevant tests" checkbox.
  - This task does not yet change any existing FIR kind's fields or any call site — it only adds `FirCursor`/`FirCursorMut`/`clone_subtree` alongside the old `Rc`/`Weak`/`RefCell` code, exactly like the previous task. `cargo check -p foolish-ubca2` and the full einmo suite must still pass unchanged after this task.

**(Parallelizable, with a caveat: the eleven per-kind tasks below may be dispatched
concurrently once both foundational tasks above — `FirPointer`/`FVMStorage`/`FirSpec` and
`FirCursor`/`FirCursorMut`/`clone_subtree` — are complete and merged; each targets a distinct
FIR-kind struct with no overlapping logical scope. The caveat: all of them edit the same file,
`fir_kinds.rs`, so concurrent agents risk merge conflicts even where their logical scopes don't
overlap. Confirm with the human whether your execution environment can land concurrent edits to
one file safely (e.g. via frequent small commits and rebases) before dispatching these in
parallel; if not, sequence them with a commit after each task despite the logical
independence.)**

- [ ] Migrate `IndepIntFir` (fir_kinds.rs, struct at the location found by the Phase-1 kind-list grep above) to `FirPointer`
  - Replace its `ProtoBrane`-embedded `Rc<RefCell<dyn Fir>>`/`Weak` fields with `FirPointer`-based access through `FVMStorage`/`FirCursor`/`FirCursorMut` (read/write parent and children only via `storage.get`/`storage.with_mut`, or the equivalent `FirCursor { ptr, storage }`/`FirCursorMut { ptr, storage }` construction where a run of several navigation calls on one node makes the cursor worth building — no field on `IndepIntFir` itself should store a raw pointer or arena handle beyond its own `FirPointer` identity, if it needs to know its own identity at all).
  - Replace this kind's `Rc::new_cyclic` construction site(s) with `create_child(&mut storage, FirSpec::IndepInt { .. })`.
  - Update `impl Fir for IndepIntFir`'s methods that touch parent/children to take `&FVMStorage`/`&mut FVMStorage` as an explicit parameter instead of calling `.borrow()`/`.borrow_mut()`/`Weak::upgrade()`.
  - Targeted einmo re-run: any case whose input exercises an independent integer literal (search `foolish-ubca2/einmo_suite/input/` for `.foo` files using bare integer literals — likely a broad, common subset; running the full suite is also acceptable here given `IndepIntFir` is simple and low-risk).

- [ ] Migrate `NkFir` to `FirPointer` (same shape of task as `IndepIntFir` above — parent/children fields, `Rc::new_cyclic` sites, `impl Fir for NkFir` methods)
  - Targeted einmo re-run: cases producing an NK result (search for `= NK` or `???` in `checked/` outputs).

- [ ] Migrate `OperatorFir` to `FirPointer`
  - Note: `OperatorFir` is described in AGENTS.md as "brane-like" (FOOP-9) — confirm during this task whether it has any brane-search-boundary interaction that the generic per-kind migration steps above don't cover; if so, treat that interaction as part of this task, not deferred.
  - Targeted einmo re-run: cases exercising binary/unary operators.

- [ ] Migrate `StatementFir` to `FirPointer`
  - `StatementFir` is likely the most-referenced kind (every brane is a sequence of statements) — expect this task to touch the largest number of call sites of any single-kind task in this phase. If it proves larger than expected, split into indented sub-tasks per this plan's sub-task convention (e.g. one sub-task for its own fields/construction, one for its `Fir` impl's parent/children-touching methods, one for any statement-chain-building helper functions specific to it).
  - Targeted einmo re-run: full suite (statements are load-bearing everywhere; a targeted subset would not meaningfully narrow scope here).

- [ ] Migrate `BraneFir` to `FirPointer`
  - `BraneFir` is the container every other kind's "home brane" resolves to (`get_my_brane`) — pay particular attention to `get_my_brane`'s implementation and update it to walk `FirPointer` parent links via `FVMStorage` rather than the current `.parent` chain walk.
  - Targeted einmo re-run: full suite (branes are load-bearing everywhere).

- [ ] Migrate `SearchFir` to `FirPointer` — structural fields and construction only, NOT its search-execution logic
  - This task covers `SearchFir`'s own `ProtoBrane`-embedded fields and its `Rc::new_cyclic` construction sites (in `fir_kinds.rs` and the corresponding sites in `compiler.rs`, though the `compiler.rs` side is covered by Phase 4 — this task touches only `fir_kinds.rs`).
  - Do NOT migrate `SearchPredicate`, `CandidateNavigator`, `BraneNavigator`, or `contextful_search_scan`/`_no_body_check` in this task — those are Phase 2's job specifically, because they are the highest-risk, most-scrutinized part of this whole FOOP and get their own dedicated phase with per-component tasks and heavier verification.
  - Targeted einmo re-run: cases exercising simple, already-passing search forms (to confirm `SearchFir`'s own structural migration didn't break anything) — do not attempt to validate search *correctness* here, only that the type compiles and passes the same tests it passed before this task, which is a weaker claim intentionally deferred to Phase 2.

- [ ] Migrate `IndexFir` to `FirPointer`
  - Targeted einmo re-run: cases exercising `#N` positional index, `^`/`$` head/tail.

- [ ] Migrate `FoolRefFir` to `FirPointer`
  - Per FOOP-16.md and CLAUDE.md's "FoolRefFir two-child invariant": a resolved search result has exactly two `ubc_children` — `[0]` the constanic clone of the found statement's body, `[1]` a `FoolRefFir` wrapping the original found statement. Confirm this invariant is preserved under the arena model — i.e. that a search result's two `FirPointer` children are still distinguishable by position/index the same way `ubc_children[0]`/`[1]` are today. This is a correctness-critical invariant to check explicitly in this task, not just a mechanical field swap.
  - Targeted einmo re-run: any case whose OUTPUT depends on a search result's found-statement position (contexted `&`-searches chained after a plain search — see FOOP-23 test cases).

- [ ] Migrate `StayFoolishFir` to `FirPointer`
  - Targeted einmo re-run: cases exercising `StayFoolish`/SFF (Stay Fully Foolish body) constructs.

- [ ] Migrate `StayFullyFoolishFir` to `FirPointer`
  - Targeted einmo re-run: same SFF-related subset as the previous task.

- [ ] Migrate `ConcatenationFir` and `ConcatHelper` together (tightly coupled per the verified kind list — `ConcatHelper` exists specifically to support `ConcatenationFir`)
  - Also update `ConcatProvenance` (an enum, not a `Fir` impl, but referenced by `ConcatenationFir`) if it holds any pointer-typed field.
  - Targeted einmo re-run: cases exercising `+`/concatenation, especially any producing a `constanicCloned` merged brane (FOOP-3's semantics) — concatenation's clone-and-merge behavior is exactly the kind of operation `clone_subtree` is meant to support, so this task is a good early smoke test of `clone_subtree` before Phase 1 fully closes.

- [ ] Migrate `CreationFir` to `FirPointer`
  - Per CLAUDE.md's "Named creation" terminology: confirm `CreationFir::get_display_name` and any rename-refusal logic (`StatementFir::check_rename_of_named_creation`) still function correctly against `FirPointer`-based parent/children access — these methods currently walk pointers to determine a creation's original name/rename eligibility.
  - Targeted einmo re-run: cases exercising named creations (`'Name = ⬤`) and rename-refusal (NF) cases.

- [ ] Migrate `ComparisonFir` (`foolish-ubca2/src/system_foo.rs`, NOT `fir_kinds.rs`) to `FirPointer`
      **Plan adjustment, added during execution**: `ComparisonFir` is a 14th `impl Fir for` site
      that this plan's original per-kind task list omitted, because that list was built only from
      `grep "^impl Fir for" fir_kinds.rs` — `ComparisonFir` lives in `system_foo.rs` instead (3 of
      its own `Rc::new_cyclic` sites: `ComparisonFir::comparison`'s constructor and
      `ComparisonFir::constanic_clone`'s two branches, confirmed by direct read of
      `foolish-ubca2/src/system_foo.rs` lines ~148-330). It is exercised by `constanic_clone_at`'s
      own `FirKind::Comparison` match arm (`fir_kinds.rs`), which this Phase already migrates as
      part of `clone_subtree`, so leaving `ComparisonFir` itself unmigrated would be an inconsistent
      half-arena-half-Rc state for exactly the kind that arm delegates to. Per the plan's own rule
      ("do not silently skip a kind that exists"), this task is added here rather than skipped.
  - `ComparisonFir`'s fields: `core: ProtoBrane`, `op: ComparisonOp` (a plain `Copy` enum, no
    migration needed), `self_weak: Weak<RefCell<dyn Fir>>` — a self-reference for the ancestral
    `'True`/`'False` search from `fir_op_step`, the exact same pattern and rationale as
    `StatementFir::self_weak` (see that task's notes) — becomes redundant under the arena, since a
    `FirPointer`'s own identity already serves this role; confirm during this task whether
    `self_weak` can be dropped entirely (its only reason to exist — `fir_op_step` receiving `&self`
    without a `self_ref` — disappears once `fir_op_step` is arena-threaded and can be given its own
    `FirPointer`) or must be kept for a reason not yet visible.
  - Migrate `ComparisonFir::comparison` (the `push_foolish_child_sff_marked`-based constructor) and
    `ComparisonFir::constanic_clone` (called from `constanic_clone_at`'s `FirKind::Comparison` arm)
    to `create_child`/`clone_subtree`-based construction, matching `OperatorFir`'s and
    `StatementFir`'s tasks above in shape.
  - Targeted einmo re-run: cases exercising comparison operators (`<̲`, `>̲`, `<̲=̲`, `>̲=̲`, `=̲=̲` —
    search `foolish-ubca2/einmo_suite/input/` for `.foo` files using these, likely under
    `foop/33/boolean/` given the `comparison_operators.foo.einmo`/`comparison_non_integer.foo.einmo`
    checked cases already seen in this crate's suite).

- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 2 — Migrate the search engine (`SearchPredicate`, `CandidateNavigator`/`BraneNavigator`, `contextful_search_scan`)

**This phase carries the highest silent-regression risk in the entire FOOP.** A traversal-order change compiles cleanly and can still silently diverge from the correct einmo output — only byte-for-byte comparison against `foolish-ubca`'s baselines catches it, not `cargo check`. Every task in this phase ends with a targeted einmo re-run against cases exercising that specific predicate/navigator, not a deferred phase-end blanket check.

**Not parallelizable.** Unlike Phase 1's per-kind tasks, this phase's four tasks form a real
dependency chain, each building on the previous: `contextful_search_scan` takes a
`CandidateNavigator` and a `SearchPredicate` as parameters, so it cannot be meaningfully migrated
until both of those are done; `SearchFir`'s own dispatch logic then wires into all three. Execute
this phase's tasks strictly in the order listed.

- [ ] Establish relevant tests for this phase. Use [these instructions](../../README.md#running-specific-tests) to run einmo tests: the full `foolish-ubca2` suite (`einmo_gate_checked`) — search-engine correctness has crate-wide blast radius, so the phase-level subset is the full suite, re-run after every task below, not a narrowed slice; run unit tests: `foolish-ubca2::fir_kinds` substring match (covers the `ContextfulSearch engine tests` module directly, per the FOOP-16.md Test Plan reference to these tests pinning internal FVM state that einmo's black-box comparison doesn't).

- [ ] Migrate `SearchPredicate` (fir_kinds.rs, the `pub(crate) enum SearchPredicate` and its `impl SearchPredicate` block, near "ContextfulSearch engine skeleton (FOOP-23 Phase A0)")
  - `SearchPredicate::matches`/`matches_no_body_check` receive "the full statement FIR (name, body/value, line number, parent, NYES)" per CLAUDE.md's "Statement Matcher" description — update these methods' signatures to take `&FVMStorage` alongside the candidate `FirPointer`, reading whatever fields they need through it, rather than through a `FirRef`'s `.borrow()`.
  - Do not change `SearchPredicate`'s variant set (`Name`, `Value`, `NameValue`, `Index`, `Head`, `Tail`) — this task is a signature/access-pattern migration only, not a semantic change.
  - Targeted einmo re-run: cases using each predicate variant at least once — `?name`, `~name`, value search (`?=`/`~=`), combined `?name=value`, `#N` index, `^`/`$` head/tail. (The existing `ContextfulSearch engine tests` module in `fir_kinds.rs`, already covered by this phase's unit-test subset, directly exercises each variant — lean on it.)

- [ ] Migrate `CandidateNavigator` trait and `BraneNavigator` impl
  - Per CLAUDE.md: "Candidate Navigator — traverses the FIR tree, yields candidates in the mandated deterministic order. Correctness contract: correctly ordered and complete (every reachable candidate, exactly once, then stops)." This ordering contract is the single most important thing to preserve exactly in this task — the arena's `Vec`-backed child storage must be walked in the same order today's `Vec<FirRef>` iteration produces, forward or backward per `CursorSource`.
  - Update `BraneNavigator`'s internal cursor/position state to hold `FirPointer` values and advance via `FVMStorage` lookups instead of walking `Rc`/`Weak` links directly.
  - Targeted einmo re-run: cases with multiple same-named statements in one brane (where traversal order determines which one an anchored search finds first) — search `foolish-ubca2/einmo_suite/input/` for `.foo` files with repeated statement names, plus any case exercising forward (`~`) vs backward (`?`) direction on the same brane to confirm both directions still traverse correctly.

- [ ] Migrate `contextful_search_scan` and `contextful_search_scan_no_body_check` (the core scan loop)
  - These take `nav: &mut dyn CandidateNavigator` and `predicate: &SearchPredicate` already, per the existing signature — confirm after the previous two tasks that this loop needs no further change beyond what flows through from `CandidateNavigator`'s and `SearchPredicate`'s own migrations (i.e. this task may turn out to be a re-verification task rather than a code-change task; if so, say so explicitly when checking it off, do not pad it with unnecessary changes).
  - Targeted einmo re-run: full suite.

- [ ] Migrate `SearchFir`'s own predicate-building methods (`build_value_predicate` and the anchored/unanchored search dispatch logic inside `impl SearchFir` and `impl Fir for SearchFir`, both already partially touched by Phase 1's `SearchFir` structural-only task)
  - This is where Phase 1's deferred "search-execution logic" gets migrated — Phase 1 only handled `SearchFir`'s own fields/construction; this task wires its predicate-building and dispatch methods to call into the now-migrated `SearchPredicate`/`CandidateNavigator`/`contextful_search_scan`.
  - Targeted einmo re-run: full suite, since this is where all of Phase 2's individually-migrated pieces are exercised together for the first time through `SearchFir` itself.

- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 3 — Migrate `evaluator.rs`'s stepping loop

`evaluator.rs` is 1246 lines with a small number of free functions (verified by reading the file): `step_until`, `step_until_line_number`, `step_until_statement_name` (public entry points), `display_stmt_name` (a small helper, unlikely to touch pointers), `step_to_settled` (the core stepping loop), `proto_to_core_fir`, `proto_to_core_fir_sff_body`, `proto_to_core_fir_sff_operand`, `anchor_to_core_fir`, `proto_to_core_fir_inner` (the FIR→core-FIR conversion family, used for output serialization). This phase splits along that seam.

- [ ] Establish relevant tests for this phase. Use [these instructions](../../README.md#running-specific-tests) to run einmo tests: full `foolish-ubca2` suite (`einmo_gate_checked`) after each task — the stepping loop is exercised by every case, so no narrower subset applies; run unit tests: `foolish-ubca2::evaluator` substring match if such tests exist (check `evaluator.rs` for a `mod tests` block; if none exists, note that in this checkbox and rely on the einmo suite alone for this phase).

- [ ] Migrate `step_to_settled` (the core per-FIR stepping dispatch)
  - Replace `.borrow()`/`.borrow_mut()`/`Weak::upgrade()` call sites with `FVMStorage::get`/`FVMStorage::with_mut`. This function is where the majority of the loop's arena access concentrates — expect this to be the largest single task in this phase; split into indented sub-tasks (e.g. by NYES-state-transition branch, if the function's internal structure supports that split cleanly) if it proves larger than expected once underway.
  - Targeted einmo re-run: full suite.

- [ ] Migrate `step_until`, `step_until_line_number`, `step_until_statement_name` (the public step-N-times/step-to-target entry points)
  - These likely call `step_to_settled` in a loop and additionally inspect FIR state directly (to check the stopping condition) — update any direct `.borrow()`/pointer-chasing here to go through `FVMStorage`.
  - Targeted einmo re-run: full suite.

- [ ] Migrate `proto_to_core_fir`, `proto_to_core_fir_sff_body`, `proto_to_core_fir_sff_operand`, `anchor_to_core_fir`, `proto_to_core_fir_inner` (the FIR→core-FIR output-serialization family)
  - These functions currently take `&FirRef` (and, for the SFF variants, an additional `current_stmt: Option<&FirRef>`) and walk the tree to build the `core_fir::Fir` representation einmo actually serializes into OUTPUT. Update their signatures to take `FirPointer` plus `&FVMStorage` instead of `&FirRef`.
  - This family is worth its own task separate from `step_to_settled` because it is the direct producer of every einmo OUTPUT line — a subtle bug here would show up as a diff against `checked/` on nearly every case, making it both high-blast-radius and (usefully) easy to detect via the full-suite re-run.
  - Targeted einmo re-run: full suite (this is, by construction, the function family the entire suite's OUTPUT depends on).

- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 4 — Migrate `compiler.rs`/`proto_brane.rs` construction onto `create_child` chains

`compiler.rs`'s 18 `Rc::new_cyclic` sites cluster into four natural groups (verified by reading the file): (1) `build_fir`'s per-AST-node-kind match arms — one `Rc::new_cyclic` per `Astn` variant handled (Brane, BinaryOp, UnaryOp, three Search-producing arms, Seek, HeadTail, two Concatenation arms, StayFoolish, StayFullyFoolish — 12 sites), (2) `build_concat_element` (1 site, a small helper feeding into the Concatenation arms), (3) `build_stmts`/the `AstnCompilerExt::compile_statement`-style extension (2 sites — one for `StatementFir` itself, one likely for wrapping), (4) `Compiler::compile`'s root-brane construction (1 site, at the very top of the compile pipeline) plus one more root-level site — confirm the exact count matches 18 when this phase starts; if it doesn't, adjust the sub-tasks below to match what's actually found rather than the count listed here.

- [ ] Establish relevant tests for this phase. Use [these instructions](../../README.md#running-specific-tests) to run einmo tests: full `foolish-ubca2` suite (`einmo_gate_checked`) — construction-order changes can shift statement indices and downstream search results crate-wide, so no narrower subset applies here either; run unit tests: `foolish-ubca2::compiler` substring match.

- [ ] Migrate `build_fir`'s per-AST-node-kind match arms (compiler.rs) — one sub-task per `Astn` variant arm
  **(Parallelizable with the same file-conflict caveat as Phase 1's per-kind tasks: the seven
  sub-tasks below touch disjoint match arms within the same function/file, `compiler.rs`, so
  their logical scopes don't overlap but concurrent edits to one file still risk conflicts —
  confirm the execution environment can land them safely, or sequence with a commit after each.)**
  - [ ] `Astn::Brane { .. }` arm → `create_child(&mut storage, FirSpec::Brane { .. })`
  - [ ] `Astn::BinaryOp`/`Astn::UnaryOp` arms → `create_child(&mut storage, FirSpec::Operator { .. })` (both arms produce `OperatorFir`, migrate together)
  - [ ] The three `SearchFir`-producing arms (anchored search, `&`-contexted search, value/name-value search — confirm exact `Astn` variant names by re-reading `build_fir`'s match at this point) → `create_child(&mut storage, FirSpec::Search { .. })`
  - [ ] `Astn::Seek`/`Astn::HeadTail` arms → `create_child(&mut storage, FirSpec::Index { .. })` (both produce `IndexFir`, migrate together)
  - [ ] The two `ConcatenationFir`-producing arms → `create_child(&mut storage, FirSpec::Concatenation { .. })`
  - [ ] `Astn::StayFoolish` arm → `create_child(&mut storage, FirSpec::StayFoolish { .. })`
  - [ ] `Astn::StayFullyFoolish`-equivalent arm → `create_child(&mut storage, FirSpec::StayFullyFoolish { .. })`
  - Each sub-task above: replace that arm's `Rc::new_cyclic(|me: &Weak<RefCell<XFir>>| { .. })` closure with the corresponding `create_child` call; the closure's field-initialization logic becomes the `FirSpec` variant's field values passed directly, with no `me`/self-`Weak` needed at all.
  - Targeted einmo re-run after each sub-task: cases exercising that specific AST construct (e.g. after the BinaryOp/UnaryOp sub-task, run cases using `+`/`-`/comparison operators).

- [ ] Migrate `build_concat_element` to use `create_child`
  - Targeted einmo re-run: concatenation cases (overlaps with Phase 1's `ConcatenationFir` task's subset — reuse it).

- [ ] Migrate `build_stmts` and the statement-construction path in `AstnCompilerExt` to use `create_child`
  - This is where a statement chain currently gets built as nested nested `Rc::new_cyclic` closures wired to a shared `parent: &Weak<RefCell<dyn Fir>>` — confirm this collapses to the flat sequential `create_child` loop shown in FOOP-16.md §Specification's before/after example.
  - Targeted einmo re-run: full suite (every brane is a statement chain).

- [ ] Migrate `Compiler::compile`'s root-brane construction (and any other root-level `Rc::new_cyclic` site found in this file) to use `create_child`/`make_my_child`
  - The root brane has no parent `FirPointer` to call `create_child` on — this is the one legitimate call site in `compiler.rs` for `storage.make_my_child` used directly (or an equivalent root-insertion method on `FVMStorage` if one is cleaner for the no-parent case — decide during this task and note the choice).
  - Targeted einmo re-run: full suite.

- [ ] Migrate `proto_brane.rs`'s single `Rc::new_cyclic` site
  - Read `proto_brane.rs` at the location found by the earlier `Rc::new_cyclic` grep to confirm what this site constructs (a scaffolding/default `ProtoBrane`, per the file's role as construction scaffolding) and replace it with the corresponding `create_child`/`make_my_child` call.
  - Targeted einmo re-run: full suite.

- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 5 — Remove residual `Rc`/`Weak`/`RefCell`/`FirRef` types from `foolish-ubca2`

- [ ] Establish relevant tests for this phase. Use [these instructions](../../README.md#running-specific-tests) to run: full `foolish-ubca2` build (`cargo build -p foolish-ubca2`) and full einmo suite (`einmo_gate_checked`).

- [ ] Grep `foolish-ubca2/src/` for remaining references to `FirRef`, `FirRefExt`, `FirRefNavExt`, `Rc<RefCell`, `Weak<RefCell` and confirm the count is zero outside of comments/docs explaining the old design for historical clarity (if any such explanatory comment remains, that's fine — this task removes *code*, not necessarily every mention in prose)
  - If any live code reference remains, it means an earlier phase's task was incomplete — go back and finish that migration rather than leaving a mixed old/new pointer scheme in place.

- [ ] Delete the `FirRef` type alias, `FirRefExt`, `FirRefNavExt` trait definitions and their `impl` blocks from `foolish-ubca2/src/fir_trait.rs` (or wherever they're defined — confirm location via grep)
  - `NyesExt` is NOT removed — it's an extension trait for NYES-state logic, not for pointer/tree-structure access, and stays regardless of the pointer-scheme migration.

- [ ] Run `cargo build -p foolish-ubca2 --release` and confirm no `unused import`/`dead_code` warnings remain related to the removed types (clean up any leftover `use` statements this removal orphaned).

- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 6 — Promote `foolish-ubca2`'s own `checked/`; comprehensive test; decide next steps

- [ ] Establish relevant tests for this phase. Use [these instructions](../../README.md#running-specific-tests) to run: full `foolish-ubca2` suite (`einmo_gate_checked`), full `foolish-ubca` suite (`einmo_gate_checked`, as the untouched-oracle sanity check referenced in FOOP-16.md §Test Plan).

- [ ] Confirm `foolish-ubca2`'s `output/` matches its `checked/` (copied from `foolish-ubca` in Phase 0) for every case, by construction of the prior phases
  - If any case diverges at this point, that is a real regression introduced somewhere in Phases 1–5 — do not promote past it; bisect which phase's task introduced the divergence (the per-task targeted einmo re-runs above should make this narrow, but if a targeted subset missed the case, the full-suite phase-end gates should have caught it — if neither did, treat that as a gap in this plan's targeted-subset choices worth noting).

- [ ] Write and verify `foolish-ubca2/einmo_suite/input/foop/16/comprehensive.foo`
  - Per FOOP-16.md §Test Plan: since this FOOP is an internal representation change, this test exercises a wide, representative cross-section of *existing* feature combinations (nested branes, contexted operators, value search, combined name+value forms, head/tail, AB/IB recoordination via a named-brane reference) rather than any new language surface.
  - Run it through `foolish-ubca2`'s einmo suite and through `foolish-ubca`'s suite (add the same input file there too, purely as a comparison run — do NOT promote or commit it into `foolish-ubca`'s own suite as a permanent addition, since `foolish-ubca` stays untouched; this is a throwaway comparison run only, to obtain the expected OUTPUT to compare against).

- [ ] Review and promote `output` → `checked` for FOOP-16's einmo cases in `foolish-ubca2/einmo_suite`
  - [ ] Confirm the rest of the `foolish-ubca2` suite is green — no case other than the ones being promoted diverges from its already-copied `checked/` baseline
  - [ ] Confirm none of the cases being promoted has a `verified/` twin in `foolish-ubca2/einmo_suite/verified/` (should be empty per Phase 0 — if not, STOP, ask the human)
  - [ ] Re-read FOOP-16.md §Specification and §Test Plan for what each promoted case is meant to demonstrate
  - [ ] Review `foop/16/comprehensive` — every OUTPUT statement compared line-by-line against `foolish-ubca`'s output for the same input (obtained in the previous task's throwaway comparison run), and justified as matching for the reason it should: the arena migration changed storage mechanism only, so every OUTPUT line is expected to be byte-identical to `foolish-ubca`'s; any line that is NOT identical is a regression to fix, not a difference to explain away
  - [ ] Write the justification summary into this plan or the commit message: what `foop/16/comprehensive` demonstrates and why its result matches `foolish-ubca` byte-for-byte
  - [ ] Report ALL accumulated doubts to the human in ONE statement — or record "no doubts". Blocking doubts stop here; non-blocking ones are reported alongside.
  - [ ] Run `einmo promote output to checked foolish-ubca2/einmo_suite`
  - [ ] Re-run `cargo test -p foolish-ubca2 --lib -- einmo_gate_checked` — must exit 0

- [ ] Run all tests — old and new — and make sure they all pass correctly.

- [ ] Present the open questions from FOOP-16.md §Open Questions to the human for decision (do not decide unilaterally): `foolish-ubca`'s eventual fate (kept indefinitely as frozen reference, formally deprecated, or removed), whether "`foolish-ubca2`" is the permanent crate name, and whether the `zweimomo` workspace-membership gap should be fixed as follow-up cleanup.

- [ ] STOP! STOP!! STOP!!! ASK HUMAN to check this box before continuing. UNDER NO CIRCUMSTANCES will Agent continue past this point automatically!!
  - [ ] Present human with the `cd /yolo/foolish/../foolish_worktrees/foop-16-arena-storage` command and ask them to review the `foolish-ubca2` crate, its promoted `checked/` baselines, and the open-questions decisions above BEFORE checking the parent checkbox.

- [ ] Verify all work is complete in `/yolo/foolish/../foolish_worktrees/foop-16-arena-storage` and committed to `foop-16-arena-storage`

- [ ] Merge `foop-16-arena-storage` to `jia`
  - [ ] Run all tests — old and new — and make sure they all pass correctly.
  - [ ] Check and make sure `foop/16/comprehensive` snaptest passes in both `foolish-ubca` (throwaway comparison, uncommitted) and `foolish-ubca2` (committed, promoted). Human gives final signed approval.
  - [ ] Run all tests — old and new — and make sure they all pass correctly.

- [ ] Cleanup worktree at `/yolo/foolish/../foolish_worktrees/foop-16-arena-storage`
  - [ ] Check that this plan file has all but Cleanup checkboxes completed
  - [ ] Remove `/yolo/foolish/../foolish_worktrees/foop-16-arena-storage`
  - [ ] This is the last sub-task checkbox to be checked in this block
