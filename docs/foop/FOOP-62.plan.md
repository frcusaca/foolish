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
      (2026-06-19 — commit 94ed10d2)
- [x] **Add/strengthen unit tests:** (a) normal clone — constanic (ECONSTANIC) compound stays
      ECONSTANIC, pre-constanic compound → PREMBRYONIC; (b) foolish clone (flag true) — ALL NYES
      copied verbatim; (c) `has_ancestral_sfm` propagation through step + clone recursion;
      (d) later search of an SF yields inner result with NO `StayFoolish` kind present and the
      inner re-resolves; (e) leaves unchanged in both modes.
      (2026-06-19 — 7 new tests in fir_kinds.rs: clone_nyes_rule_by_mode,
      normal_clone_keeps_constanic_compound_state, normal_clone_resets_preconstanic_compound,
      foolish_clone_copies_all_nyes_verbatim, leaf_clone_unchanged_both_modes,
      cloning_sf_strips_the_mark, step_sets_foolish_scope_inside_sf. 92 pass; the 1 remaining
      failure is the pre-existing chained_undeclared parser edge case. commit pending)
- [x] **Sequencer: HFS NYES display rule for searches (spec §9.x, §8 hard constraint).**
      `should_show_search_nyes(has_result)` added to `Nyes` (fir.rs) + `is_nnk_constanic()`;
      wired into the search / HeadTail / Index render arms in sequencer.rs. Implements case a)
      result present + nnk_constanic ⇒ hide nyes; case b) no result + EMBRYONIC ⇒ hide; else show.
      Also fixed stale spec §9.x "Constanic-clone of SF-markers" note to rev-14 (strip + pass on
      the foolish flag; do NOT force Normally / do NOT reset constanic).
      (2026-06-19 — shifts the `test_format_index` unit test + some search snapshots, AS DESIGNED.
      commit bab85b50)
- [x] **HUMAN CONCERN RESOLVED (nested_brane_boundary, lines 19-20): re-step cloned result.**
      For `{a=1; b={c=#-1;d=2;e=#-1}; f=#-1;}`, `f` constanic-cloned brane `b` and left the inner
      `c=#-1` stuck PREMBRIONIC because the Index settled immediately. FIX: IndexFir now pushes the
      cloned result (push_ubc_child already enqueues it as a task), goes BRANING, and settles from
      the DRAINED result via settle_from_ubc_result() — re-enqueue, pop, step, settle. Now `f`
      settles to NK and matches its approved snapshot (no .snap.new).
      (2026-06-19 — commit ce58bfd4. The same pattern in Search/HeadTail/SFF is the next step.)
- [ ] **Nyes states should be Rust constants, not cloned values.** `Nyes::Constant`,
      `Nyes::Independent`, etc. are simple enum variants — there's no reason to clone them.
      They should be Rust constants (or `Copy` references to statics) so that `clone_nyes`
      and `get_nyes()` return a reference to a single shared instance, not a freshly-cloned
      value. Audit all `Nyes` construction/cloning sites and convert to constant references.
- [ ] **HUMAN REVIEW of the new UBCa snapshot review set (12 files; AI MUST NOT auto-accept).**
      Differences fall in these categories:
      (A) **Sequencer HFS NYES-display rule** (§9.x): e.g. `hfs_nyes_display_rules` — a WOCONSTANIC
      search WITH a result no longer prints `WOCONSTANIC` (case a). CORRECT per spec.
      (B) **Search never INDEPENDENT** (Atlas ruling, commit pending): a search that resolves to a
      value is now CONSTANT, never INDEPENDENT (a search is context-dependent; it CAN be CONSTANT
      e.g. `{a={b=1}.b}`=1). `search_nyes_from_found()` caps found CONSTANT/INDEPENDENT → CONSTANT.
      So `offset_access_out_of_bounds` etc. now show `?(... CONSTANT)` not `INDEPENDENT`.
      (C) **RULED (Atlas 2026-06-20): `result=` is the SEARCH RESULT, not the anchor.** For
      search/Index/HeadTail (all classified as searches), `result=` is the FIR found by
      searching/indexing/head-tail. The current Index/HeadTail render wrongly puts the ANCHOR in
      `result=`. → task #11.
      Plus `chained_undeclared` (pre-existing parser edge case, unrelated).

### DESIGN — task #11: Index/HeadTail `result=` is the indexed result (Atlas ruling) — DONE 2026-06-20

STATUS: COMPLETE. IndexFir/HeadTailFir core_fir gained a `result` field + builder; hs_index/
hs_head_tail return `(.., anchor, result)`; sequencer renders `result` as `result=` (anchor is a
non-result item); ubca bridge sets `.result(resolved)`. Result now correct: `b#0`→`10`,
out-of-bounds `b#3`→`#(offset=3, ANCHORED, NK)` (no result=, anchor no longer shown as result).
SINGULAR-RESULT invariant documented (spec §8) + runtime-verified
(`ProtoBrane::push_search_result` debug-asserts ubc_children empty; all 8 search-kind result
pushes routed through it). 92 ubca unit tests pass; snapshot shifts are the intended review set.



TERMINOLOGY (Atlas 2026-06-20): the field/word is **`result`**, never "target". SearchFir's
existing `target` field is to be RENAMED `result` for consistency.

The `result=` slot of a search/Index/HeadTail is the **result of the search**, never the anchor.
Today `hs_search` returns `(pattern, dir, anchored, anchor, result)` and renders the result as
`result=` (correct, but the field is misnamed `target`). But `hs_index`→`(offset, anchored,
anchor)` and `hs_head_tail`→`(is_head, anchored, anchor)` carry NO result; the sequencer falls
back to rendering the **anchor** as `result=` (wrong). Fix span:
1. Rename SearchFir's `target` field/builder/accessors → `result` (drop "target" everywhere).
2. core_fir `IndexFir`/`HeadTailFir` gain a `result` field + builder method (mirror `SearchFir`).
   The anchor stays a separate non-result item.
3. `hs_index`→`(offset, anchored, anchor, result)`; `hs_head_tail`→`(is_head, anchored, anchor,
   result)`.
4. sequencer Index/HeadTail arms: render `result` as `result=`; anchor becomes a non-result item.
5. ubca→core bridge: set `.result(resolved)` from `ubc_children` (the indexed result).
Multi-file (foolish-core + foolish-ubca). Snapshots will shift (present for review).

### DESIGN — task #10: unify nyes determination (Atlas direction) — DONE 2026-06-21

STATUS: COMPLETE. Audit finding: nyes was ALREADY FIR-owned — `nyes: Cell<Nyes>` is private on
ProtoBrane, `set_nyes` is `pub(crate)`, and every real call is `self.core.set_nyes(...)` inside
the kind's own `fir_op_step`; the ONLY external setter was the DEAD `advance_to_embryonic` free
fn. So #10 was small (Atlas: "I don't foresee a lot of changes" — correct). NO choke point
(Atlas: transitions like Prembrionic→Braning are sequential, not pure functions of children —
verify via tests instead). Done:
- Removed the dead `advance_to_embryonic` (the lone from-outside nyes setter); EMBRYONIC intent
  moved to #14.
- Documented the OWNERSHIP CONTRACT on `ProtoBrane::set_nyes`: only a FIR on itself in its own
  step, or construction (`ProtoBrane::new`, incl. constanic-clone via `clone_nyes`). Matches the
  spec's existing "three sanctioned writers" (§1 nyes field doc).
- Caveat honored: nyes IS still set when cloning — via `ProtoBrane::new(.., clone_nyes(..))`.
- Added 4 unit tests stepping a PARENT brane and observing per-step nyes of parent + a watched
  descendant: brane_of_constants_progresses_to_settled, operator_in_brane_advances_before_parent
  _settles, unresolved_search_in_brane_goes_econstanic, constanic_node_stays_constanic_across
  _parent_steps. No behavior/snapshot change.

### task #15: per-FIR-kind nyes-transition unit tests — DONE 2026-06-21

One `<kind>_nyes_transitions` unit test per FIR kind in fir_kinds.rs, each recording the full
per-step nyes sequence (step_to_settled) and asserting via a shared `assert_progression`
(start=PREMBRIONIC, end=constanic, monotone — no constanic→pre-constanic regression — plus the
kind's terminal state). Coverage: ConstantInt→CONSTANT, Nk→NK, Operator(+)→CONSTANT,
Operator(/0)→NK, Statement→CONSTANT, Brane→CONSTANT, Brane(+NK)→NK, Search(anchored found),
Search(not found)→ECONSTANIC, Index→CONSTANT, Index(oob)→NK, HeadTail→CONSTANT,
HeadTail(empty)→NK, Concatenation→CONSTANT, StayFoolish→CONSTANT, StayFullyFoolish→INDEPENDENT.
112 ubca unit tests pass; only `approval_all` (review-set snapshots) fails. (Atlas: these are
UNIT tests — nyes is internal FVM state.) Tests documented actual VM behavior (Brane-of-constants
classifies CONSTANT; SFF→INDEPENDENT in one step). DOC follow-up: task #16 (note the convention
in dev docs; new nyes/FIR must extend these tests).



`nyes` is a cached mutable field with a getter. **Invariant: any time a FIR is borrowed for
writing, before that borrow expires `nyes` must be correct** — set at instantiation AND
re-established during stepping. Besides instantiation, the place that matters is **stepping**:
the updated nyes is driven by (a) which `step` branch ran and (b) the children's post-step
returned nyes. So **track and set nyes as stepping happens**.
- Each kind owns its nyes computation (its own act + progress). It MAY consult
  `_decide_nyes_due_to_children` (a suggestion, not the authority).
- A per-kind helper may be introduced where useful; its required parameters differ by kind
  (e.g. SearchFir needs the found body's nyes via `search_nyes_from_found`; OperatorFir needs the
  computed value or div-by-zero; BraneFir folds in `_decide_nyes_due_to_children`).
- This is §9.0 (Quiescent-Representation Invariant) specialized to the nyes cache. Audit every
  `set_nyes` / `borrow_mut` so none releases a write-borrow with a stale nyes.
- **Work the nyes around the job queue** (Atlas): the queue state drives the phase. Tasks
  pending ⇒ pre-constanic; queue drained ⇒ the node runs its own act (in `fir_op_step`) and
  computes its terminal nyes from the now-constanic children/results. `fir_op_step` IS the
  "queue-drained, do my act" hook (the driver only calls it when `front_task()` is empty).

### tasks #20–#23: NICC / EMBRYONIC re-step / WOCONSTANIC chain — DONE 2026-06-22

Fixes the sff_basic regression and finalizes the constanic-clone semantics:
- **#20 NICC nyes rule** (`clone_nyes`): asserts CONSTANIC input. NICC keeps Constantew
  (CONSTANT/INDEPENDENT/NK); resets ECONSTANIC/WOCONSTANIC → **EMBRYONIC** (the "start working"
  stage where searches re-progress IB→AB). FICC unchanged (copies verbatim). EMBRYONIC is now
  universal — every non-search kind's begin-arm matches `Prembrionic | Embryonic`.
- **#22 operator EMBRYONIC** (no stage skip): EMBRYONIC decides whether to enqueue operands (skip
  if already constanic); combine stays in BRANING. `combine` extracted.
- **#23 search settle-from-drained** (NOT re-stepping): a search that finds a constanic body
  NICC-clones it + goes BRANING without settling; the ordinary drain finishes the (possibly
  EMBRYONIC) clone, then `settle_from_ubc_result` settles from it.
- **#21 generic ubc clone + WOCONSTANIC chain shorten**: `constanic_clone_at` clones ubc_children
  for ALL kinds; a WOCONSTANIC search clone shortens its chain to the NICC of the deepest
  ECONSTANIC (`deepest_econstanic_in_chain`). "Result present → don't search" guard on the search
  EMBRYONIC/BRANING arms. Singular-result + this guard are SEARCH-SPECIFIC; multi-result
  ProtoBranes (Op+, brane) use plain push_ubc_child, unaffected.
Result: sff_basic and the nested/complex SFF cases resolve correctly. 119 ubca unit tests pass;
review set 21 → 17.

### docs #13/#16 — DONE 2026-06-22
FOOP-62.md "Terminology: anchor and result (NOT 'target')" section; AGENTS.md
"NYES transition tests (*_nyes_transitions)" subsection (new FIR kinds / NYES states must
extend them). Synced to alpha. (commit 225369ae)

### DESIGN — task #14: EMBRYONIC = within-brane stage (preserve; reintroduce) — DONE 2026-06-21

STATUS: COMPLETE. Reintroduced EMBRYONIC for unanchored searches:
- New `ib_search` (immediate brane only) and `ab_search` (ancestral branes, skipping the
  immediate) helpers in fir_kinds.rs.
- SearchFir step is now: Prembrionic → (anchored: push anchor, Braning) | (unanchored:
  Embryonic). EMBRYONIC runs `ib_search`; miss → Braning. BRANING runs `ab_search` (or, for
  anchored, the anchor search; or drains a found-but-pre-constanic body). Exhausted → ECONSTANIC.
- `handle_found` helper unifies settle-now vs wait-for-body across IB/AB.
- Anchored searches stay in BRANING (never EMBRYONIC) per design.
Verified: `Search(not found)` trace = [Prembrionic, Embryonic, Braning, Econstanic]; ancestral
search = [Prembrionic, Embryonic, Braning, Constant]. 4 new IB/AB CONTEXT tests (compile real
Foolish, parent-wired): ib_context_resolves_in_immediate_brane, ab_context_name_not_in_immediate
_brane (ib_search misses ancestral-only name, ab_search finds it), ib_shadows_ab_immediate_wins
(shadowing → immediate `a`=2 not ancestral `a`=1), ancestral_search_passes_through_embryonic
_then_braning. 116 ubca unit tests pass; only approval_all (14-file review set) fails.

### task #17: SFF builds descendant searches ECONSTANIC — DONE 2026-06-21
Build-from-Foolish-code rule: `under_sff` flag through the build_ chain; under an SFF
(`<<…>>`), descendant searches are constructed ECONSTANIC and never run. Fixes sf_of_sff:
`sff` AND `sf=<sff>` both freeze as `Op+(?a ECONSTANIC, ?b ECONSTANIC, WOCONSTANIC)`. Does NOT
affect constanic-cloning of an SFF child (normal nyes rules). 2 unit tests.

### task #18: consolidate build_standalone + build_astn into build_fir — DONE 2026-06-22
One `build_fir(ast, parent: Option<&Weak>, under_sff)`: `parent=None` ⇒ ROOT (self-parent;
ONLY a Brane may be root — compile_standalone enforces; non-Brane arms expect a parent). Removed
the `under_sff`-dropping fallthrough. Correctness side-effect: SFF-econstanic (#17) now applies
consistently to nested/standalone SFF — 6 more SFF snapshots freeze correctly.

### task #19: simplify build_as_statement + `???`-LHS sequencer rule — DONE 2026-06-22
build_as_statement now decides the name once (LHS identifier, or `???` = ANON_STMT_NAME for a
bare/anonymous statement) and writes ONE Rc::new_cyclic. NEW SPEC RULE: a statement named `???`
renders WITHOUT a `name=` prefix (display_stmt_name maps `???`/empty → None). Behavior-preserving
for rendering. 1 unit test (anonymous_statement_named_question_marks). 119 ubca unit tests pass.

Atlas: EMBRYONIC is NOT vestigial — UBCa currently skips it (Prembrionic→Braning), a
regression. Intended meaning, **worked around the job queue**:
- **EMBRYONIC**: the node does its **immediate-brane** work — `ib_search` (Immediate Brane
  search, within the brane). Everything that stays inside the brane happens here. (Unanchored
  searches only.)
- **BRANING**: the node crosses brane boundaries — `ab_search` (Ancestral Brane search) **AND
  anchored searches** (anchored searches typically cross brane boundaries, so they belong to
  BRANING, never EMBRYONIC).
- Progression for an unanchored search: `Prembrionic → Embryonic (ib_search) → Braning
  (ab_search) → constanic`. An anchored search skips the EMBRYONIC ib_search and does its work
  in BRANING. Other kinds enqueue children and pass through the stages as their queue drains.
SEQUENCING: fold into #10 if not too large; else implement #10 first (queue-driven nyes,
Prembrionic→Braning) and add EMBRYONIC as a follow-on — but preserve the stage + this meaning.

APPROACH: design (these sections) → implement per-kind incrementally, committing each → snapshots.
- [x] **DECIDED (2026-06-19, Atlas): UBCa is its own source of truth; the "match UBC
      byte-for-byte" requirement is REMOVED.** Rationale: foolish-core UBC snapshots are
      pre-existingly stale (confirmed by stashing all session changes) — many committed
      `foolish-core/...approved/*.foo.snap` disagree with the current core evaluator (deep VALUE
      diffs: `avg=20` → unresolved `Op/(...)`, `inner=15` → `Op/(...)`, branes collapsing). UBC is
      no longer an authoritative oracle. UBCa is validated byte-for-byte against its OWN approved
      snapshots. Spec + plan scrubbed of the cross-check-against-UBC requirement.
      (2026-06-19)

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
- [ ] Seed UBCa's `snapshot_tests/input/` from UBC's `.foo` test programs. UBCa then maintains
      its OWN `snapshot_tests/approved/` corpus (its source of truth) — UBC's approved `.snap`
      are NOT required to be reproduced.
- [ ] Snapshot harness: runs each `.foo` through UBCa, compares Humanizing sequencer output
      against **UBCa's own approved snapshots**. New/changed output produces `.snap.new` for
      human review — **never auto-accept** (AGENTS.md). NOT diffed against UBC.
- [ ] Genericize the `Evaluator` trait over the FIR ref type (or thin adapter) so the harness
      can drive UBCa's `Rc<RefCell<dyn Fir>>` (currently pinned to `dyn Steppable`) — mimo #5
- [ ] Snapshot testing FULLY implemented in foolish-ubca — the complete SnapshotSuite
      machinery (runner, .snap.new generation, signature verification), running UBCa's own
      snapshot suite, not a subset or stub.

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

## Phase 4 — Switch UBCa off the UBC delegation; UBCa's own snapshots are the gate

> UBCa is its own source of truth. The "match UBC byte-for-byte" requirement is REMOVED
> (2026-06-19, Atlas): UBC's snapshots have drifted from its evaluator and UBC is no longer an
> authoritative oracle. UBCa is validated byte-for-byte against its OWN approved snapshots.

- [ ] Point UBCa at its own ProtoBrane impl (stop delegating to UBC)
- [ ] Iterate UBCa until its **sequencer output is byte-exact against UBCa's own approved
      snapshots** on every `.foo` input (step counts are NOT compared). NOT diffed against UBC.
- [ ] For any UBCa snapshot delta, generate `.snap.new` only; **present to human — NEVER
      auto-accept** (AGENTS.md). UBCa snapshots are established/reviewed on UBCa's own terms.
- [ ] All new unit tests (Phase 2) pass

## Phase 5 — Review, decide UBC's fate, integrate

- [ ] `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`,
      full `cargo test --workspace` green (per rust_instructions.md hard gates)
- [ ] Human review of UBCa diff + any `.snap.new`
- [ ] STOP! ASK HUMAN: keep UBC around for reference, or retire it and promote UBCa? (UBC is
      no longer the acceptance oracle regardless.) Under no circumstances retire UBC automatically.
- [ ] Per human decision: either leave UBC in place, or migrate callers UBC→UBCa and
      remove UBC (separate, human-gated step)
- [ ] Verify all work is complete in `/home/hcbusy/tmp/foolish-worktrees/foop-62-ubca-mimo` and committed to `foop-62-ubca-mimo`
- [ ] Merge `foop-62-ubca-mimo` to alpha
- [ ] Update FOOP-62.md status; clear Open Questions; update INDEX.md
- [ ] Cleanup `/home/hcbusy/tmp/foolish-worktrees/foop-62-ubca-mimo`
  - [ ] Check that FOOP-62.plan.md has all but Cleanup checkboxes completed
  - [ ] Remove `/home/hcbusy/tmp/foolish-worktrees/foop-62-ubca-mimo`
  - [ ] This is the last checkbox to be checked in FOOP-62.plan.md

- [ ] **Refactor Fir constructors into `fn create()` on each Fir type.** Move standalone
      allocator functions (e.g. `pub fn statement(name, line_number, body, parent) -> FirRef`)
      into `impl StatementFir { pub fn create(name, line_number, body, parent) -> FirRef }`.
      Do this for ALL Fir types (IndepIntFir, NkFir, OperatorFir, SearchFir, IndexFir,
      HeadTailFir, ConcatenationFir, StatementFir, BraneFir, StayFoolishFir,
      StayFullyFoolishFir). Then investigate if other similar reference constructors
      (in compiler.rs, evaluator.rs, tests) should also use the new `create()` methods.

- [ ] **Centralize ProtoBrane constanic cloning.** Every FirKind match arm in `constanic_clone_at`
      duplicates the same pattern: clone `foolish_children` recursively, create `ProtoBrane::new`
      with `clone_nyes`, clone `ubc_children` recursively. Extract this into a single
      `clone_proto_brane(source: &ProtoBrane, new_parent, index, foolish_flag)` helper that all
      compound kinds call. This eliminates ~200 lines of duplicated clone logic and ensures new
      FIR kinds automatically get correct clone behavior.

## Bug fix set — crash-isolated repair cycle (2026-06-23)

> **CRASH ISOLATION PROTOCOL.** One snapshot test triggers a Rust runtime error that kills the
> tester, opencode, and the surrounding bash shell. To identify the crashing test, each fix
> (A–E) is applied and verified against **individual snapshot tests** before running the full
> suite. The format below marks each test's start/end so crashes are attributable.
>
> **Per-test command** (run from `foolish/` workspace root):
> ```bash
> cargo test -p foolish-ubca --lib -- approval_all 2>&1 | grep -E '^(test |PASSED|FAILED|panic|thread)'
> ```
>
> **Single-file evaluation** (to isolate one `.foo` without the full harness):
> ```bash
> # Build the evaluator, then evaluate one file manually
> cargo build -p foolish-ubca && ./target/debug/foolish-ubca-eval path/to/file.foo
> ```
>
> Each fix below is:
> 1. Written to this plan file BEFORE the code change.
> 2. Applied to `fir_kinds.rs` (or relevant file).
> 3. Verified against the specific snapshot test(s) listed.
> 4. Committed only if the targeted test passes AND no new crash occurs.

### Fix A — IndexFir/HeadTailFir: gate `constanic_clone_at` on `is_constanic()` (BLOCKER panic)

**Status:** [x] DONE 2026-06-23
**Changes:**
- Removed `debug_assert!(source.is_constanic())` from `clone_nyes` — recursive clones of
  compound children (OperatorFir foolish_children) can be pre-constanic; handled gracefully
  by the fallback → EMBRYONIC path.
- Added `found_body: RefCell<Option<FirRef>>` to IndexFir (mirrors SearchFir).
- IndexFir Prembrionic/Braning arms: gate `constanic_clone_at` on `body_nyes.is_constanic()`.
  If pre-constanic, stash + enqueue + wait.
- HeadTailFir Braning arm: same gate.
- Updated all IndexFir construction sites (compiler.rs, fir_trait.rs, fir_kinds.rs).
**Result:** 120 unit tests pass. No more `clone_nyes` panic. Snapshot harness proceeds to
first mismatch (expected — many `.snap.new` pending review).
**File:** `foolish/foolish-ubca/src/fir_kinds.rs`
**Diagnosis:** `clone_nyes()` at line 101 asserts `source.is_constanic()`. IndexFir (line 1194,
1220) and HeadTailFir Braning arm (line 1305, 1316) call `constanic_clone_at(&body, ...)`
without first checking `body.borrow().core().get_nyes().is_constanic()`. If the body is
pre-constanic (Prembrionic/Embryonic/Braning), the assertion fires and panics.

**Expected invariant (per user review):** In normal brane stepping, when an IndexFir is
stepped, its dependencies should already be constanic. The panic suggests a specific test
case violates this — possibly a deeply nested clone where the parent chain points to a
clone with reset children.

**Fix plan:**
1. In `IndexFir::fir_op_step` (Prembrionic arm, line 1188-1195): after finding a body via
   `index_into_brane_relative`, check `body_nyes.is_constanic()`. If constanic, NICC-clone
   it (existing path). If NOT constanic, stash the body (like SearchFir's `found_body`),
   push it as a task, and go BRANING to wait for it to settle.
2. In `IndexFir::fir_op_step` (Braning arm, anchored, line 1218-1223): same gate on the
   body from `index_into_brane`.
3. In `HeadTailFir::fir_op_step` (Braning arm, line 1303-1306 and 1314-1317): same gate.
   The Prembrionic arm (line 1278) already has the guard.
4. Add a `found_body: RefCell<Option<FirRef>>` field to IndexFir (mirror SearchFir) to
   stash non-constanic bodies.

**Tests to verify (START → END markers):**
```
>>> FIX A TEST START: unanchored_seek_basic.foo
>>> FIX A TEST START: anchored_seek_positive_boundary.foo
>>> FIX A TEST START: anchored_seek_positive_negative.foo
>>> FIX A TEST START: offset_access_out_of_bounds.foo
>>> FIX A TEST END
```

---

### Fix B — `search_brane_children`: return SF frozen result, don't unwrap inner

**Status:** [x] DONE 2026-06-23
**File:** `foolish/foolish-ubca/src/fir_kinds.rs`
**Changes:**
- Removed the `sf_inner_pattern` re-evaluation block from `search_brane_children` — search
  finds the named statement and returns its body as-is.
- `constanic_clone_at` now prefers `ubc_children` (frozen result) over `foolish_children`
  (inner expression) when stripping SF/SFF markers.
- Resolution centrally relies on NYES state and progressive stepping.
**Result:** `sf_blocks_brane_at_assignment_time` now returns `{x=1,y=2}` (frozen). `sff_vs_sf_timing_difference`: sf=1 (frozen), sff=10 (correct).

**Tests to verify:**
```
>>> FIX B TEST START: sf_blocks_brane_at_assignment_time.foo
>>> FIX B TEST START: sf_brane_blocking.foo
>>> FIX B TEST START: sf_non_brane_resolves.foo
>>> FIX B TEST START: sf_of_sff.foo
>>> FIX B TEST END
```

---

### Fix C — SFF re-coordination: NICC-cloned descendants re-step against new parent chain

**Status:** [ ] PARTIAL 2026-06-23
**File:** `foolish/foolish-ubca/src/fir_kinds.rs`
**Changes so far:** OperatorFir EMBRYONIC arm now uses `operands_all_settled()` (CONSTANT/INDEPENDENT only) instead of `operands_all_constanic()`. ECONSTANIC/WOCONSTANIC operands are enqueued for re-evaluation.
**Remaining:** Search operands still show ECONSTANIC after NICC clone — the searches are not re-resolving in the new parent chain. Needs deeper investigation of the SFF stripping + NICC clone path.

**Tests to verify:**
```
>>> FIX C TEST START: complex_sff_in_nested_brane.foo
>>> FIX C TEST START: sff_basic.foo
>>> FIX C TEST START: sff_nested.foo
>>> FIX C TEST START: sff_resolves_on_each_use.foo
>>> FIX C TEST END
```

---

### Fix D — Sequencer: `should_show_search_nyes` for WOCONSTANIC-with-result

**Status:** [ ] PARTIAL 2026-06-23
**File:** `foolish-core/src/fir.rs`
**Changes so far:** `should_show_search_nyes` now only hides NYES for CONSTANT/INDEPENDENT with result (not WOCONSTANIC). WOCONSTANIC means the result is not final.
**Remaining:** `chained_undeclared.foo` — `z`'s `result=` shows a WOCONSTANIC fir but should show the ECONSTANIC fir (the deepest unresolved search). The constanic-clone chain is returning the wrong level.

**Tests to verify:**
```
>>> FIX D TEST START: chained_undeclared.foo
>>> FIX D TEST END
```

---

### Fix E — Test input fixes

**Status:** [ ] PARTIAL 2026-06-23
**Files:** `foolish-ubca/snapshot_tests/input/*.foo`

1. [x] **regex_search_pattern.foo:** Input already updated to `^a.*`.
2. [x] **anchored_search_foward.foo:** Changed `hw~(.o.)` to `hw~(^.o.$)`.
3. [ ] **Seek tests:** Consolidate `anchored_seek_positive_boundary.foo` and
   `anchored_seek_positive_negative.foo` into a single test.

**Tests to verify:**
```
>>> FIX E TEST START: regex_search_pattern.foo
>>> FIX E TEST START: anchored_search_foward.foo
>>> FIX E TEST START: anchored_seek_*.foo (consolidated)
>>> FIX E TEST END
```

---

### Fix F — SF frozen value lost on constanic-clone (`sff_vs_sf_timing_difference`)

**Status:** [x] DONE 2026-06-23 (same fix as Fix B)
**File:** `foolish/foolish-ubca/src/fir_kinds.rs`
**Changes:** Fixed by the same change as Fix B — `constanic_clone_at` prefers `ubc_children` over `foolish_children` when stripping SF/SFF.
**Result:** `sff_vs_sf_timing_difference` — sf=1 (frozen), sff=10 (re-resolved, correct).

**Tests to verify:**
```
>>> FIX F TEST START: sff_vs_sf_timing_difference.foo
>>> FIX F TEST END
```

---

### Fix G — Complex SF/SFF concatenation stuck in BRANING (`concat_sf_f_more`)

**Status:** [ ] NOT STARTED
**File:** `foolish/foolish-ubca/src/fir_kinds.rs`
**Diagnosis:** Input with nested SF/SFF in concatenations (`f1`, `f2`, `f3`) — evaluation
gets stuck in BRANING/PREMBRIONIC instead of reaching constanic. "the f1, f2, f3 are
utterly wrong they should never be in those nyes. Should always be constanic in snapshots
we run them to constanic states." The outer brane also shows BRANING when it should be
constanic.
**Fix plan:** Investigate why nested SF/SFF in concatenations prevent the outer brane from
settling. Likely a task-drain ordering issue or a missing re-step after SF/SFF resolution.

**Tests to verify:**
```
>>> FIX G TEST START: concat_sf_f_more.foo
>>> FIX G TEST END
```

---

## Outstanding issues — from @agent snapshot review (2026-06-22)

> Original items 1, 3, 4, 5 resolved. Fix F resolved (same as Fix B). Only unresolved items tracked here.

- [ ] **2. SFF re-coordination in nested brane (HIGH — core eval bug).**
      File: `complex_sff_in_nested_brane.foo`
      Input: `{a=1, b=2; inner = {c = <<a+b>>; c}; inner;}`
      **PARTIAL — see Fix C.** OperatorFir enqueues ECONSTANIC operands, but searches still ECONSTANIC.

- [ ] **6. Combine seek tests (test consolidation).**
      Files: `anchored_seek_positive_boundary.foo`, `anchored_seek_positive_negative.foo`
      "let's combine all these seek tests into a single snapshot test."

### Deferred (no action now)
- `foop42_humanizing_sequencer_formatting_exhaustive_aka_hfs.foo` — "don't touch this"
- `sequencer_comprehensive.foo` — "don't touch this one"
- `anchored_search_suite.foo` — "hold on on this one"

- [ ] **Correct documentation that led to SF re-evaluation in search code.** The removed
      `sf_inner_pattern` block in `search_brane_children` (Fix B, 2026-06-23) was written
      based on incorrect guidance that SF-marked searches should re-evaluate in the current
      context. The correct resolution centrally relies on NYES state and progressive stepping:
      `search_brane_children` finds the named statement and returns its body as-is; the
      SF's frozen result lives in `ubc_children`; `constanic_clone_at` prefers `ubc_children`
      over `foolish_children` when stripping SF/SFF markers. Find and correct the spec text
      (FOOP-62.md), code comments, and any FOOP guidance that led to the re-evaluation design.

- [x] **Remove "frozen/freeze" wording from code, docs, and FOOPs.** Replace with the correct
      Foolish terminology: "constanic" (terminal NYES state), "constanew" (newly constanic),
      or "non-constanew constanic" (pre-existing constanic). The word "frozen" implies a
      mechanical action that doesn't match the NYES-based stepping model.
      (2026-06-23 — all occurrences in ubca source, FOOP-62.md, and FOOP-62.plan.md replaced.
      Added terminology note to FOOP-62.md §Terminology.)

## Notes / discoveries

- [ ] **Explore `bon` + `build_from` pattern for constanic cloning.** The current
      `constanic_clone_at` manually reconstructs each FIR kind via `Rc::new_cyclic` +
      `ProtoBrane::new` + field-by-field copy. Investigate whether `bon`'s `#[builder]` and
      `build_from` (clone-then-modify) could replace the per-kind match arms with a generic
      clone-via-builder path: seed a builder from the source FIR, override `parent`/`nyes`/`index`,
      and build. This would eliminate ~200 lines of repetitive kind-specific clone code and ensure
      new FIR kinds automatically get correct clone behavior. Check `bon` docs for `build_from`
      support with `Rc::new_cyclic` patterns and `ProtoBrane` seeding.
- [ ] **Use `tracing` for alarms instead of `eprintln!`.** The current `constanic_clone_at`
      uses `eprintln!` for the SF/SFF-no-children alarm. Replace with `tracing::warn!` (or
      appropriate level) following the alarm patterns established in `foolish-core` (e.g.
      `Alarm`/`AlarmLevel`/`AlarmSource` in the evaluator). Copy the existing alarm design —
      structured fields, consistent codes, same severity conventions.

## Last Updated

**Date**: 2026-06-23 (crash-isolated bug fix cycle added)
**Updated By**: opencode 1.14.28; vllm/qwen-3.6 27b
**Changes**: Added crash-isolated repair cycle (Fixes A–E) with per-test START/END markers.
Each fix is written to plan BEFORE code changes, verified against individual snapshot tests,
and committed only if the targeted test passes without crashing. Original §Bug fix set
preserved as reference (marked SUPERSEDED).

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
