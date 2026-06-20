# FOOP-62 Implementation Plan — UBCa Two-Store ProtoBrane

> **WORKTREE.**
> All FOOP-62 work happens in a dedicated worktree:
>
> ```
> WORKTREE_ORIGIN_BRANCH=alpha
> WORKTREE_ORIGIN_PATH=$(pwd)
> WORKTREE_BRANCH_NAME=foop-62-ubca-mimo
> WORKTREE_FULL_FS_PATH=/home/hcbusy/tmp/foolish-worktrees/foop-62-ubca-mimo
> ```
>
> Created from the starting branch/path:
> ```
> cd $WORKTREE_ORIGIN_PATH
> git checkout $WORKTREE_ORIGIN_BRANCH
> git worktree add -b "$WORKTREE_BRANCH_NAME" "$WORKTREE_FULL_FS_PATH"
> ```
>
> Per AGENTS.md the worktree lifecycle is tracked as explicit checkbox tasks below.
>
> Spec: `docs/foop/FOOP-62.md`. Memory: [[foop62-ubca-two-store-protobrane]].

## Phase −1 — HIGH PRIORITY: implement & verify foolish-ignorance clone model (BLOCKING)

> Raised 2026-06-19 (Atlas + previous coding agent's open concern). Spec rev 14 defines the
> **ignorance** model: a `Scope.has_ancestral_sfm: bool` carried by `step()`, seeding each
> `constanic_clone`'s `descendent_of_sfm_and_foolishly_ignorant: bool`. Ground truth as of SHA
> `cc3fe590` on branch `foop-62-ubca-mimo` in directory
> `/home/hcbusy/tmp/foolish-worktrees/foop-62-ubca-mimo` (plus this session's uncommitted
> doc-comment edits), in `foolish/foolish-ubca/src/`:
>   - `Scope` is a 2-field STUB in `fir_trait.rs` (`current_brane`, `current_stmt_idx`) with
>     `Scope::empty()`. `step_fir_ref`/`fir_op_step` already take `&Scope` but ignore it; no
>     `has_ancestral_sfm`. `bon` is NOT used in this crate.
>   - `constanic_clone_normal_at(fir_ref, new_parent, index)` has NO flag param; only normal
>     mode exists.
> Each item corrects code to match spec. These BLOCK further FOOP-62 work (they change
> evaluation semantics, hence snapshots).

- [x] **Add `has_ancestral_sfm: bool` to `Scope`** (`fir_trait.rs`), default false; thread it
      through `step_fir_ref`/`fir_op_step` (already take `&Scope`). Set true when entering an
      SF-mark's RHS; carried down the step recursion.
      (2026-06-19 — Scope.has_ancestral_sfm + with_ancestral_sfm(); step_fir_ref_inner switches
      to foolish child_scope when recursing into a StayFoolish node. commit 0e57636c)
- [x] **Add `descendent_of_sfm_and_foolishly_ignorant: bool` param to the clone** (rename/
      generalize `constanic_clone_normal_at`). Call sites in `step()` pass
      `scope.has_ancestral_sfm`; the clone's OWN recursion passes the CALLER's flag (the two
      recursions are independent — do NOT re-read scope inside clone recursion).
      (2026-06-19 — `constanic_clone_at(.., descendent_of_sfm_and_foolishly_ignorant)`; long
      name throughout per Atlas. commit 0bce9dd0)
- [x] **NYES-transfer rule by mode (spec Terminology + §6b).**
      - flag = false (normal): constanic NYES transfer UNCHANGED; pre-constanic → PREMBRYONIC.
      - flag = true (foolish): ALL NYES copied unchanged (constanic AND pre-constanic).
      (2026-06-19 — new `clone_nyes(source, flag)` replaces all hard-coded `Nyes::Prembrionic`
      in compound arms. Snapshots unchanged (85/86). commit 0bce9dd0)
- [x] **ConstantInt / Nk leaves transfer NYES unchanged** — verify they stay unchanged (both
      modes) after the fix above.
      (2026-06-19 — leaf arms keep `borrowed.core().get_nyes()`; unaffected by clone_nyes.)
- [x] **THE BIG BUT — later search of an SF-mark strips the mark (spec §9.x + §10.1).** When an
      SF-mark is constanic-cloned, STRIP the mark — clone the inner expression directly (no
      StayFoolish wrapper). Per Atlas: PASS ON the incoming
      `descendent_of_sfm_and_foolishly_ignorant` flag to the inner clone (do NOT force false),
      so a nested SF inside an outer SF's RHS stays foolish.
      (2026-06-19 — StayFoolish arm strips + recurses with incoming flag. commit pending)
- [x] **SFF / fully-foolish construction (spec Terminology + §10.1).** SFF realized at
      CONSTRUCTION (descendants ECONSTANIC); the `FirKind::StayFullyFoolish` clone arm now
      mirrors SF — strips the mark and clones the inner with the incoming flag.
      (2026-06-19 — commit pending; UBC-oracle confirmation folded into the oracle re-verify box)
- [ ] **Add/strengthen unit tests:** (a) normal clone — constanic (ECONSTANIC) compound stays
      ECONSTANIC, pre-constanic compound → PREMBRYONIC; (b) foolish clone (flag true) — ALL NYES
      copied verbatim; (c) `has_ancestral_sfm` propagation through step + clone recursion;
      (d) later search of an SF yields inner result with NO `StayFoolish` kind present and the
      inner re-resolves; (e) leaves unchanged in both modes.
- [ ] **Re-verify against the UBC oracle** (cross-check harness, Phase 1) after the fixes —
      UBCa sequencer output must still match UBC byte-for-byte. Any snapshot delta is
      PRESENTED to human; AI MUST NOT auto-accept.

## Phase 0 — Gate & baseline (BLOCKING)

- [x] Create worktree at `/home/hcbusy/tmp/foolish-worktrees/foop-62-ubca-mimo` with branch `foop-62-ubca-mimo`
      (worktree exists; `foolish-ubca` crate present and compiling)
      (2026-06-19)
- [ ] Confirm spec FOOP-62.md is reviewed/approved by human (status Draft → Brewing/Final)
- [x] DECIDED: UBCa is a new sibling CRATE `foolish-ubca` (like `foolish-ubcb`), NOT an
      in-crate module; original UBC stays in `foolish-core`. Task queue = `std::VecDeque`.
      (decided 2026-06-09 14:45; crate `foolish/foolish-ubca/` created and building 2026-06-19)
      (2026-06-19)
- [x] Human approval to add `bon` (`3.x`) to `[workspace.dependencies]` (new third-party dep)
      — APPROVED by Atlas: add latest stable bon
      (2026-06-09 14:30)
- [x] DECIDED (post deepseek+mimo review): composition = kinds CONTAIN a `ProtoBraneImpl`;
      `trait ProtoBrane` (shared code as defaults over `core()`), `trait Fir: ProtoBrane`
      (per-kind); `FirRef = Rc<RefCell<dyn Fir>>` (enum + clone_into_fir retired). Parent
      wiring = nested `Rc::new_cyclic`, parent immutable at construction.
      (2026-06-09 15:30)
- [x] DECIDED: step counts are NOT an acceptance constraint (snapshots carry no step count —
      verified snapshot_suite.rs:135). Acceptance = byte-exact sequencer output only.
      (2026-06-09 15:30)
- [x] DECIDED (spec rev 4, borrow-discipline experiment in /tmp/foop62-lang-experiment):
      shared topology = INHERENT methods on `struct ProtoBrane` (renamed from
      ProtoBraneImpl; the old `trait ProtoBrane` is eliminated); stepping =
      `step_fir_ref(&FirRef, &Scope)` FREE FUNCTION with transient borrows (nested
      `borrow_mut` recursion panics on ancestral search — confirmed experimentally);
      `step_fir_ref` returns `StepReport { NoProgress, Progress(Nyes) }` (Progress even
      when nyes unchanged); task-queue pop predicate is `is_settled()` =
      `is_constanic() || == Nk` (Nk is settled or it blocks the queue forever).
      (2026-06-10 09:00)
- [x] DECIDED (spec rev 5, Atlas direction): Scope is reworked into the search-capability
      surface (spec §10): `entries` flat name list REMOVED (parent chain IS the name
      table); public surface = `search_ib`/`search_ab`/`index(offset)`/`get_ignorance()`/
      `emit()`; positional fields private. `EvalContext{Normal,Sf,Sff}` RENAMED
      `Ignorance{Normally,Foolishly,Fully}`. Anchored searches are inherent methods on
      BraneFir (`search(pattern,from,to)`, `index(n)`, `head()`, `tail()`), NOT on Scope.
      Scope is built with `bon` fluent builders like every FIR node.
      (2026-06-10 09:00)
- [x] RULED by Atlas (2026-06-10): unanchored index (`a = #-1 + #-2`) permits ONLY negative
      offsets, [-k, -1]; out-of-range (incl. 0/positive) ⇒ NK. Anchored index (`b#1`/`b#-1`)
      is a DISTINCT operation, both signs valid. UBC's positive-offset acceptance in
      step_unanchored (fir.rs:1874) is a latent bug, not language behavior.
      (2026-06-10 10:30)
- [x] Residual corpus check: verify no approved snapshot exercises a positive UNANCHORED
      index offset — VERIFIED NONE: regex over snapshot_tests/input/ finds 19 files with
      unanchored NEGATIVE forms (`=#-2`, `#-1 + #-2`, …) and ZERO unanchored positive;
      every positive `#N` in the corpus is anchored (`b#1`, `data#0`, `index_brane#99`).
      No collision between the ruling and byte-exactness. Bonus: seek_negative_clamping.foo
      (`c=#-99`) already snapshot-covers unanchored out-of-range.
      (2026-06-10 10:45)
- [ ] DECIDED: Temporarily abandon UBC/UBCb development — UBCa implementation proceeds on
      alpha; UBC's pre-existing snapshot failure (anchored_search_foward.foo: `youˍis=5→2`)
      is a known UBC issue, not a blocker for UBCa.
      (2026-06-11 12:00)
- [ ] See `FOOP-62.feedback-synthesis.md` for the full 13-item action list behind these.

## Phase 1 — Clone the UBC interface + tests into UBCa (no new behavior)

- [ ] Create new crate `foolish-ubca/` (mirror `foolish-ubcb/` Cargo.toml shape; add to
      workspace `members`)
- [ ] **"Clone the UBC interface" means: same input (.foo files) and same output format
      (Humanizing sequence for snapshot testing).** NOT cloning the internal implementation.
      UBCa gets its own compiler, evaluator, and sequencer that produce byte-identical
      snapshot output.
- [ ] Copy UBC's snap tests to UBCa **as-is without change**: both `snapshot_tests/input/`
      (the `.foo` test programs) and `snapshot_tests/approved/` (the finalized, signed
      `.snap` files) are copied byte-for-byte.
- [ ] Cross-check harness: runs each `.foo` through UBCa, compares Humanizing sequence
      output against approved snapshots. Initially UBCa produces `.snap.new` files for
      human review — **never auto-accept** (AGENTS.md).
- [ ] Genericize the `Evaluator` trait over the FIR ref type (or thin adapter) so the harness
      can drive UBCa's `Rc<RefCell<dyn Fir>>` (currently pinned to `dyn Steppable`) — mimo #5
- [ ] Snapshot testing FULLY implemented in foolish-ubca — the complete SnapshotSuite
      machinery (runner, .snap.new generation, signature verification), running ALL of
      UBC's snapshot tests (every input `.foo`, every approved snap), not a subset or stub.
      Gate: UBCa's suite enumerates the same test count as UBC's.
- [ ] Verify UBC remains untouched and still green (it is the oracle)

## Phase 2 — Tests first for the new structure (write before impl)

(AGENTS.md dev process: tests first — they document the structure and pin behavior.)

- [ ] Unit: Quiescent-Representation Invariant (§9.0, THE core mandate) — between step() calls
      FIR structure agrees with its nyes; at Constant/Independent it denotes its genuine value.
      Assert at every quiescent point in a stepped tree.
- [ ] Unit: `foolish_children` has no public mutator; length stable across stepping
- [ ] Unit: `ubc_children` push (produce result) AND clear/shrink (re-step), then re-derive
- [ ] Unit: `ubc_children` push ORDER is render order — sequencer emits results as `result=`
      before foolish_children, byte-exact (order is snapshot-visible, §1/§8)
- [ ] Unit: task-list drain — `step_fir_ref` works front task, pops when `is_settled()`,
      returns `StepReport::Progress(nyes)`; `fir_op_step` runs only when child tasks
      drained; fir_op_step-pushed task drains before node settles; empty task list ⇒ node
      settled (terminal debug_assert)
- [ ] Unit: one action per `step_fir_ref` call; child climbs
      Prembrionic→Embryonic→Braning→settled across calls (mirrors UBC)
- [ ] Unit: DEEP NESTED SEARCH through the drain — a statement body several branes deep
      resolves an outer name via `scope.search_ab` WHILE being stepped by `step_fir_ref`
      (guards the transient-borrow discipline; the nested-borrow_mut shape panics here —
      write this test FIRST, it catches the §3 borrow hazard immediately)
- [ ] Unit: Nk in the task queue — an Nk child is popped (`is_settled`), parent classifies
      Nk, no stall and no max_steps exhaustion
- [ ] Unit: unresolvable forward ref terminates via `NoProgress`, NOT via max_steps
- [ ] Unit: Scope capability surface (spec §10.1) — `search_ib` sees only names BEFORE
      `current_stmt_idx`; `search_ab` widens via `get_parent_brane()` bounded at each level
      by `get_parent_statement().get_line_number()`, stops at `is_root()`; NO public
      positional getters compile
- [ ] Unit: upward navigation trio — `get_parent()` / `get_parent_statement()` /
      `get_parent_brane()` from arbitrary depths (expr in operator in statement in brane);
      NO downward containment scan anywhere in UBCa
- [ ] Unit: RANGE CHECKING per the 2026-06-10 ruling — unanchored `scope.index(offset)`:
      boundary `-k` ok, `-(k+1)` ⇒ NK, `0` ⇒ NK, positive ⇒ NK; anchored `brane.index(n)`:
      n≥0 front, n<0 back, both-signs out-of-bounds ⇒ NK; `head()`/`tail()` on empty brane
- [ ] Snapshot: `.foo` inputs demonstrating out-of-range unanchored AND anchored indexes
      render as NK in sequencer output (range rules are snapshot-visible, not unit-only)
- [ ] Unit: BraneFir anchored surface (spec §10.2) — `search(pattern, from, to)` with
      from > to searching backward; `index(n)` n≥0 front / n<0 back; `head()`/`tail()` ==
      `index(0)`/`index(-1)`; byte-parity with today's search.rs free functions
- [ ] Unit: `Ignorance` gates (spec §10.1, asked via `how_ignorant()`) — `Fully` ⇒ search
      goes Econstanic without running; `Foolishly` ⇒ search runs but a found BRANE is not
      consumed (Econstanic); `Normally` ⇒ unrestricted
- [ ] Unit/compile-fail: `nyes` is read-only from outside (only `get_nyes()` public; no
      public setter); only init/step/constanic-clone change it
- [ ] Unit: `is_root()` — root parent is self-Weak; non-root climbs; build via `Rc::new_cyclic`
- [ ] Unit: parent is IMMUTABLE — no `set_parent`; `clone_with_parent` produces a NEW node
      with the new parent, leaving the original untouched (detach-recoordinate)
- [ ] Unit: a computed `ubc_children` value is CONSTRUCTED with its parent (never mutated in);
      ancestral search resolves THROUGH a computed result (guards the §5 parent-wiring risk)
- [ ] Unit: concatenation — k inputs in foolish_children, k+1th result in ubc_children,
      appears only when all inputs constanic; re-step clears & rebuilds it
- [ ] Unit: single-child = len-1 ProtoBrane (SF/SFF, statement body); leaf = len-0
- [ ] Unit: brane statements iterable from foolish_children with name/line metadata;
      `search_ib`/`search_ab` over foolish_children
- [ ] Unit/compile-fail: builder is the ONLY construction path — a struct-literal or
      `Fir::Variant(..)` outside the module does not compile (trybuild/compile-fail test);
      `bon` rejects missing required `.parent(..)` at compile time
- [ ] Unit: constanic clone seeds builder from an existing Fir, overrides parent+state,
      builds a NEW node; SOURCE is unchanged (foolish_children + parent immutable)

## Phase 3 — Implement ProtoBrane, the Fir kinds, and the reworked Scope in UBCa

Every FIR node is one of a small set of **kinds** — Brane, Statement, Operator, Search,
Index, HeadTail, StayFoolish, StayFullyFoolish, Concatenation, ConstantInt, Nk. A kind is
a concrete struct (`BraneFir`, `SearchFir`, …) that CONTAINS the shared `struct ProtoBrane`
field-holder as its `core` and adds its own leaf data. All kinds share the same topology
code (inherent on `ProtoBrane`) and the same stepping function (`step_fir_ref`); kinds
differ ONLY in their `fir_op_step` (their own combining work) and leaf accessors, reached
by dynamic dispatch through `FirRef = Rc<RefCell<dyn Fir>>`. The work of this phase is:
the shared core once, then each kind's `fir_op_step`, then the reworked Scope they search
through.

- [ ] `struct ProtoBrane` (field-holder): `foolish_children` (fixed), `ubc_children`
      (mutable, ORDER significant), `nyes`, `tasks: VecDeque<FirRef>`,
      `parent: Weak<RefCell<dyn Fir>>` (immutable at construction). Shared topology as
      INHERENT methods (spec §1): children slice accessors, `all_children`,
      `push_ubc_child`/`clear_ubc_children`, `front_task`/`pop_front_task`/`push_task`,
      `parent()`, `is_root(self_rc)`, `get_nyes()`. NO mutable child iterator (rejected
      alt C). NO `trait ProtoBrane` — the trait is gone, the struct carries the code.
- [ ] `trait Fir` (dyn-dispatch surface): `core()`/`core_mut()`, `fir_op_step` (default =
      compute_brane_state classify), `kind`, leaf accessors. `FirRef = Rc<RefCell<dyn Fir>>`
      (enum + clone_into_fir RETIRED). Must stay dyn-compatible: no RPITIT, no `-> Self`.
- [ ] `Nyes::is_settled()` = `is_constanic() || == Nk` — the task-queue pop predicate
      (`is_constanic()` stays as the outer acceptance predicate)
- [ ] NYES: public READ-ONLY `get_nyes()`; NO public setter; `set_nyes` PRIVATE to
      ProtoBrane, callable only from the node's own `fir_op_step` via `core_mut()`.
      Written only by init/step/clone.
- [ ] Each kind = `struct XFir { core: ProtoBrane, <leaf data> }` impl Fir (returns &core,
      its own fir_op_step). `step_fir_ref` is the SAME free function for every kind;
      differences via construction-time state + fir_op_step ONLY. SF/SFF do NOT get a
      custom step path (their semantics are set at construction + via constanic_clone).
- [ ] `bon` builders per kind (`#[derive(Builder)]`); `parent` REQUIRED input; `foolish_children`
      complete-vector input; `ubc_children`/`tasks` NOT inputs (start empty)
- [ ] Builder-only construction ENFORCED by language (6a): all fields private, structs in a
      private module, `#[non_exhaustive]`, `build()` returns the `Rc<RefCell<dyn Fir>>` form
      (no public `fir_to_ref`/struct-literal seam)
- [ ] Constanic clone via builder-updater (6b): `#[builder(on(_, overwritable), on(String,
      into))]` + hand-written `updater(self)->XFirBuilder` per payload; `source.updater()
      .parent(..).nyes(..).build()`; `clone_with_parent` is the parent-only case
- [ ] `step_fir_ref(&FirRef, &Scope) -> Result<StepReport, UbcError>` written ONCE as a
      FREE FUNCTION (spec §3.2), CHECK-THEN-ACT, one action/call. TRANSIENT BORROWS ONLY:
      peek+clone the front handle under a short borrow, DROP it, then either pop
      (`is_settled`), recurse on the handle, or `this.borrow_mut().fir_op_step(scope)`.
      Never hold a borrow on `this` across the recursive call (nested borrow_mut panics on
      ancestral search — proven in /tmp/foop62-lang-experiment). Returns
      `StepReport { NoProgress, Progress(Nyes) }`; Progress even when nyes is unchanged.
- [ ] Per-kind `fir_op_step`: leaves (no-op) → Operator → Search/Index/HeadTail (resolve via
      `scope.search_ib/search_ab/index` for unanchored, via the anchor BraneFir's inherent
      `search/index/head/tail` for anchored; Woconstanic short-circuit collapses chain into
      ubc_children) → Concatenation (pushes result brane task) → SF/SFF → Brane (see §9).
      NYES transitions MIRROR UBC (step_one + compute_brane_state)
- [ ] §9 NormalBrane: statement = len-1 ProtoBrane w/ name+line leaf data, fed parent+line+body
      via BUILDER at construction (no parallel metadata vec); in-place stepping is CORRECT
      (§9.0 invariant, independence via constanic_clone — do NOT clone body in drain);
      re-step REBUILDS tasks from foolish_children (Econstanic-pop trap); incremental scope
      via the §10.4 PROPOSED StatementFir-boundary augmentation (statement builds its body's
      scope from own line_number + parent — validate; brane wrapper is the fallback);
      SF/SFF differ only at CONSTRUCTION (no custom step path)
- [ ] Re-step = clear ubc_children + REBUILD tasks from foolish_children + re-derive
- [ ] Outer loop: terminate on root settled OR `StepReport::NoProgress`. NOT the old root
      `prev==new`. Keep Woconstanic&&!forward_refs + max_steps (guard only — clean
      termination must come from NoProgress; a brane with unresolvable forward refs must
      report NoProgress instead of rebuilding forever). Re-express
      `has_unresolved_forward_refs` over the two stores.

### Phase 3a — Scope rework (spec §10)

- [ ] `enum Ignorance { Normally, Foolishly, Fully }` — RENAMES `EvalContext{Normal,Sf,Sff}`;
      Scope accessor `how_ignorant()` (adverb question, adverb answers); drop the separate
      `block_brane_searches` bool — VERIFIED dead (never set true anywhere; the SF blocking
      lives in step_except_brane_searches' hardcoded brane check, which `Foolishly` must
      reproduce: found BRANE not consumed ⇒ Econstanic)
- [ ] Scope struct: PRIVATE `current_brane`, `current_stmt_idx`, `ignorance`, `alarms`;
      NO `entries` field, NO public positional getters
- [ ] Upward navigation trio on ProtoBrane (inherent): `get_parent()` (one hop),
      `get_parent_statement()` (nearest enclosing StatementFir), `get_parent_brane()`
      (nearest enclosing brane); all walk the parent Weak chain, stop at is_root().
      RETIRES line_of_child/contains_fir downward containment scans entirely.
- [ ] Scope capability methods: `search_ib(pattern)` (backward in immediate brane from
      current position), `search_ab(pattern)` (delegates `current_brane.search_ab(pattern,
      current_stmt_idx)`; BraneFir::search_ab recurses upward via `get_parent_brane()` +
      `get_parent_statement().get_line_number()`), `index(offset)` (UNANCHORED form
      `#-1 + #-2`: only negatives, valid [-k,-1], out-of-range ⇒ NK — RULED 2026-06-10),
      `emit(alarm)`
- [ ] BraneFir inherent anchored-search surface: `search(pattern, from_idx, to_idx)`
      (from > to ⇒ backward), `index(n)` (n≥0 front, n<0 back), `head()` = index(0) = '#0',
      `tail()` = index(-1) = '#-1'. Replaces search.rs free functions (search_in_brane,
      index_in_brane, head_of_brane, tail_of_brane) — byte-parity required
- [ ] Scope built via `bon` fluent builder (`Scope::builder().current_brane(b)
      .current_stmt_idx(3).ignorance(…).build()`); ignorance/alarms optional with defaults;
      replaces with_brane/with_sf_context/with_sff_context chains
- [ ] Sequencer: DEFAULT = thin `FirQueryable` ADAPTER over ProtoBrane (reads two stores + leaf
      accessors, returns existing tuples; ~100 lines). Prove corpus green through adapter. HARD
      CONSTRAINT (§8): byte-exact; `result=` from ubc_children (order significant). Retiring the
      trait is a LATER optional cleanup OFF the acceptance path.

### Phase 3b — New compiler in foolish-ubca (snapshot-driven development)

- [ ] **DECIDED (2026-06-11): Write a NEW compiler in `foolish-ubca/src/compiler.rs`** that
      converts AST → ProtoBrane tree directly, instead of refactoring UBC's compiler or
      converting UBC's Fir enum. The parser (`foolish-parser`) is shared; the compiler is
      new. This avoids a conversion layer and lets snapshot tests drive FIR kind
      implementation incrementally.
- [ ] `compile_astn` gains `parent: Weak<RefCell<dyn Fir>>` threaded downward; each brane
      uses `Rc::new_cyclic` to mint its own `Weak` before compiling its children (nested
      cyclic construction). Children only STORE the Weak, never upgrade during construction.
      Root's Weak is self-ref.
- [ ] Wire up snapshot harness EARLY: copy UBC's input/approved tests, create
      `UbcaEvaluator` that uses the new compiler + step loop, run snapshot tests.
      Each failing test reveals which FIR kind needs work. Let the tests drive
      implementation order (most failures = highest priority kind).
- [ ] Implement remaining FIR kinds as snapshot tests demand them:
      SearchFir, IndexFir, HeadTailFir, StayFoolishFir, StayFullyFoolishFir,
      ConcatenationFir. Each kind gets unit tests before moving to the next.

## Phase 4 — Switch UBCa off the UBC delegation; cross-check is the oracle

- [ ] Point UBCa at its own ProtoBrane impl (stop delegating to UBC)
- [ ] Run the Phase-1 cross-check harness; iterate UBCa until **sequencer output** matches
      UBC byte-for-byte on every `.foo` input (step counts are NOT compared)
- [ ] Reproduce all currently-approved `*.foo.snap` byte-for-byte via UBCa.
      Generate `.snap.new` only; **present to human — NEVER auto-accept** (AGENTS.md)
- [ ] All new unit tests (Phase 2) pass

## Phase 5 — Review, decide UBC's fate, integrate

- [ ] `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`,
      full `cargo test --workspace` green (per rust_instructions.md hard gates)
- [ ] Human review of UBCa diff + any `.snap.new`
- [ ] STOP! ASK HUMAN: keep UBC permanently as oracle, or retire UBC and promote UBCa?
      Under no circumstances retire UBC automatically.
- [ ] Per human decision: either leave UBC in place, or migrate callers UBC→UBCa and
      remove UBC (separate, human-gated step)
- [ ] Verify all work is complete in `/home/hcbusy/tmp/foolish-worktrees/foop-62-ubca-mimo` and committed to `foop-62-ubca-mimo`
- [ ] Merge `foop-62-ubca-mimo` to alpha
- [ ] Update FOOP-62.md status; clear Open Questions; update INDEX.md
- [ ] Cleanup `/home/hcbusy/tmp/foolish-worktrees/foop-62-ubca-mimo`
  - [ ] Check that FOOP-62.plan.md has all but Cleanup checkboxes completed
  - [ ] Remove `/home/hcbusy/tmp/foolish-worktrees/foop-62-ubca-mimo`
  - [ ] This is the last checkbox to be checked in FOOP-62.plan.md

## Notes / discoveries

- (log split sub-tasks and timestamps here as work proceeds)

## Last Updated

**Date**: 2026-06-19 (Phase −1 added — constanic-clone semantics verification)
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes**: Added a HIGH-PRIORITY blocking **Phase −1** at the top of the plan to verify the
spec-rev-14 ignorance semantics against the code and correct each divergence:
(1) normally-ignorant NYES-transfer rule (constanic transfers unchanged; pre-constanic →
PREMBRYONIC) — KNOWN GAP: compound kinds hard-coded to PREMBRYONIC; (2) SF wrapper must never
be cloned (constanic-clone of an SF FIR = clone of its frozen inner result) — KNOWN GAP:
`FirKind::StayFoolish` clone arm re-wraps; (3) SFF fully-foolish construction check; plus unit
tests and UBC-oracle re-verification. Marked Phase 0 worktree-creation and "new sibling crate"
boxes done (`foolish-ubca` exists and compiles; 85/86 ubca snapshots pass, the 1 failure is a
parser edge case in `chained_undeclared.foo`). Merged FOOP-62.md (spec rev 14) is in place.

**Date**: 2026-06-10 (third update — worktree declaration)
**Updated By**: Claude Code 2.1.119 (Claude Code); Sonnet 4.6
**Changes**: Replaced "WORKED IN PLACE" on FOOP-52 worktree with a dedicated worktree
declaration: `foop-62-ubca-mimo` at
`/home/hcbusy/tmp/foolish-worktrees/foop-62-ubca-mimo`. Added worktree
lifecycle tasks: create worktree (Phase 0), verify+merge+cleanup (Phase 5) per AGENTS.md.

**Date**: 2026-06-10 (second update — spec rev 6 sync)
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5
**Changes**: Index-range task RULED (unanchored negative-only [-k,-1] ⇒ NK out-of-range;
anchored both signs; UBC's positive acceptance = latent bug) with residual corpus-check
task. Phase 1: new explicit task — snapshot testing FULLY implemented with ALL of UBC's
tests (same test count gate). Phase 2: new range-checking unit tests + NK snapshot
coverage, upward-navigation-trio test; `how_ignorant()` rename. Phase 3a: `get_ignorance`
→ `how_ignorant`; new upward-navigation trio task (get_parent / get_parent_statement /
get_parent_brane, retiring line_of_child/contains_fir); search_ab recursion spelled out;
block_brane_searches confirmed dead (verified never set).

**Date**: 2026-06-10
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5
**Changes**: Synced plan to spec revisions 4+5. Phase 0: two new DECIDED entries (rev 4:
struct-ProtoBrane inherent topology + `step_fir_ref` free fn with transient borrows +
`StepReport` + `is_settled`; rev 5: Scope-as-capability-surface + `Ignorance` rename +
BraneFir anchored surface + bon Scope builder + entries removed) and one new blocking
Phase 0 task (unanchored-index positive-offset discrepancy vs UBC). Phase 2: added tests
for deep-nested-search-through-drain (write FIRST — catches the borrow hazard),
Nk-in-queue, NoProgress termination, Scope capabilities, BraneFir anchored surface,
Ignorance gates; renamed step()/NYES-return wording to step_fir_ref/StepReport. Phase 3:
added prose introduction of FIR "kinds" before the checkboxes; rewrote core checkboxes for
the rev-4 composition (no trait ProtoBrane, dyn-compatible trait Fir, is_settled,
NoProgress-based outer loop); new Phase 3a — Scope rework task group (Ignorance enum,
private positional fields, capability methods, BraneFir search/index/head/tail replacing
search.rs free functions, bon-built Scope).

**Date**: 2026-06-09
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes**: Initial plan. Phase 0 baseline gate, clone-UBC-interface→UBCa, tests-first,
ProtoBrane two-store impl, oracle cross-check, human-gated UBC retirement.
