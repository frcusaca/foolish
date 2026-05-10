# FOOP-61: UBCb State Machine — Implementation Plan

This plan covers building UBCb (the brane-driven UBC variant) as a new
parallel crate, sharing only the parser with `foolish-core`. The
state machine, FIR types, and clone operations are rewritten from
FOOP-61, FOOP-51, and FOOP-7.

## Dependency FOOPs

- [ ] Complete FOOP-51 (AB list, name resolution, search_result,
      short-circuit accumulation) — design Brewing → Implementing
- [ ] Complete FOOP-7 (Constanic Clone — split into constanic_clone
      and preconstanic_clone) — design Brewing → Implementing
- [ ] FOOP-61 (this FOOP, design) → Brewing

The above three FOOPs together specify UBCb's state machine, name
resolution, and clone operations. They are interdependent and should
be promoted together.

## Phase 0 — Prep / parser API verification

- [ ] Verify `foolish-parser` crate exposes a stable AST that does
      not depend on `foolish-core`'s FIR types.
- [ ] Confirm `Astn` (or equivalent) is `Clone + Debug + serde`
      such that UBCb's own AST→FIR compiler can consume it.
- [ ] If parser-AST is entangled with UBC's FIR: extract a clean
      AST type into `foolish-parser` (refactor in `foolish-core` to
      consume the cleaner AST). No behavior change for UBC.
- [ ] Verify `test-resources/` is at the workspace root and
      readable by both crates' integration tests. Move/relocate if
      currently nested in `foolish-core`.

## Phase 1 — Skeleton: foolish-core-ubcb crate

- [ ] Create `foolish/foolish-core-ubcb/Cargo.toml`. Add to workspace
      `Cargo.toml` `members`.
- [ ] Add dependencies: `foolish-parser` (path), `serde`, `anyhow`.
      Do NOT add `foolish-core` as a dependency — UBCb is independent
      from FIR-down.
- [ ] Create skeleton modules:
  - [ ] `lib.rs` — public API: `compile_and_run(source: &str) -> Fir`.
  - [ ] `nyes.rs` — the 9-state Nyes enum from FOOP-61.
  - [ ] `fir.rs` — empty FIR enum with all variant struct skeletons.
  - [ ] `ab.rs` — `Ab` type per FOOP-51 (immutable list of
        `(Rc<NormalBraneFir>, usize)`); `compress()` per FOOP-51's
        line-aware dedup rule.
  - [ ] `clone.rs` — `constanic_clone` and `preconstanic_clone`
        signatures with `unimplemented!()` bodies.
  - [ ] `stepping.rs` — `ProtoBrane::step()` skeleton dispatching on
        Nyes per FOOP-61 universal contract.
  - [ ] `compiler.rs` — `compile_astn(ast: Astn) -> Fir` skeleton
        that produces NK on every input (placeholder).
- [ ] `cargo build` passes.
- [ ] `lib.rs::compile_and_run` returns `Fir::Nk` for every input.

## Phase 2 — Cross-validation harness (early, drives development)

- [ ] Create `foolish/foolish-crossvalidation/Cargo.toml`. Add to
      workspace.
- [ ] Dev-dependencies: `foolish-core` and `foolish-core-ubcb` (path).
- [ ] Add an integration test that:
  - [ ] Reads each `.foo` from `test-resources/`.
  - [ ] Runs both VMs to completion.
  - [ ] Compares output **value only** (not the state field).
        Per FOOP-61 cross-validation note: "UBC uses ECONSTANIC and
        WOCONSTANIC; UBCb uses NOTFOUNDIC and the new constanic-clone
        semantics. Cross-validation must compare values, not state
        fields."
  - [ ] Reports per-test pass/fail with a diff on mismatch.
- [ ] Initially every UBCb-side test fails (it returns NK). The
      passing-set grows as variants are implemented.
- [ ] The harness becomes the primary "is UBCb done?" indicator.

## Phase 3 — FIR variants (in dependency order)

Each variant adds a chunk of `.foo` tests to the passing set.

### Phase 3.1 — Born-terminal variants

- [ ] `ConstantInt(i64)` — born INDEPENDENT. The simplest variant.
  - [ ] Implement struct with `value`, `state = INDEPENDENT`.
  - [ ] `step()` returns NoOp.
  - [ ] AST→FIR: integer literals produce ConstantInt.
  - [ ] Cross-validation: literal-only `.foo` tests pass.
- [ ] `NkFir(reason)` — born NK. Carries a reason string.
  - [ ] Implement struct.
  - [ ] `step()` returns NoOp.
  - [ ] No AST→FIR mapping (NK is produced at evaluation time).
- [ ] `StayFullyFoolish(<<expr>>)` — born INDEPENDENT.
  - [ ] Implement struct.
  - [ ] AST→FIR: `<<expr>>` produces StayFullyFoolish wrapping the
        inner.
  - [ ] `constanic_clone` on inner: shared by reference.

### Phase 3.2 — NormalBrane (the core container)

- [ ] Implement `NormalBraneFir` per FOOP-61:
  - [ ] Fields: `statements: ImmutableVec<StatementFir>`, `state`,
        `cursor`, `ab: Ab`, `parent: Option<FirRef>`,
        `line_in_parent: usize`, `luid: Option<Luid>`.
  - [ ] `StatementFir { name: Option<String>, body: FirRef }`.
- [ ] Implement PREMBRYONIC → EMBRYONIC transition: build the
      immutable StatementFir array from AST, set parent, assign LUID.
- [ ] Implement EMBRYONIC: step children to EMBRYONIC; gather and
      attempt local-IB-only resolution for searches; transition
      searches to WOBRANING when blocked on PREMBRYONIC siblings.
- [ ] Implement BRANING: step non-constanic children one at a time
      (cursor-based round-robin).
- [ ] Implement `compute_brane_state` per FOOP-61 priority order:
      NK > WOCONSTANIC > NOTFOUNDIC > CONSTANT.
- [ ] Implement WOCONSTANIC step: re-check children, advance.
- [ ] Implement CONSTANT → INDEPENDENT detach (FOOP-51).
- [ ] Cross-validation: simple brane tests pass (e.g., `{a=1; b=2}`).

### Phase 3.3 — Operator

- [ ] Implement `OperatorFir { op, operands, state, cursor, ab,
      parent, line_in_parent }` per FOOP-61.
- [ ] PREMBRYONIC, EMBRYONIC: pass-through.
- [ ] BRANING: round-robin step operands.
- [ ] `compute_operator_state` priority order matching brane.
- [ ] Arithmetic computation when all operands CONSTANT/INDEPENDENT.
- [ ] Special cases: division by zero → NK with reason
      `division-by-zero`.
- [ ] Cross-validation: arithmetic tests pass (e.g., `{x = 2 + 3}`).

### Phase 3.4 — Search (and AB walk)

- [ ] Implement `SearchFir { pattern, direction, anchored, anchor,
      search_result, state, ab, parent, line_in_parent,
      blocking_on }` per FOOP-61.
- [ ] EMBRYONIC: pass-through (parent brane drives local resolution).
- [ ] BRANING: cross-brane walk per FOOP-51 (own statements via
      parent → AB → live parent). Anchored on PREMBRYONIC brane →
      WOBRANING.
- [ ] WOBRANING step: poll `blocking_on.state()`; return to BRANING
      when EMBRYONIC+.
- [ ] WOCONSTANIC step: re-read target; advance.
- [ ] Cross-validation: bare-identifier tests pass (e.g.,
      `{a=1; b=a}`).

### Phase 3.5 — Index, HeadTail (positional access)

- [ ] Implement `IndexFir`. EMBRYONIC: pass-through. BRANING: anchor
      lookup. WOBRANING for PREMBRYONIC anchor. NOTFOUNDIC for
      no-parent (root-brane unanchored).
- [ ] Implement `HeadTailFir` similarly. NK for empty brane (with
      reason `head-of-empty` / `tail-of-empty`).
- [ ] Cross-validation: anchored-search tests pass.

### Phase 3.6 — Concatenation

- [ ] Implement `ConcatenationFir { elements, merged, state, cursor,
      ab, parent, line_in_parent }` per FOOP-61.
- [ ] Two-phase BRANING: pre-merge (step elements), build merge,
      post-merge (step merged brane).
- [ ] Cross-validation: concatenation tests pass.

### Phase 3.7 — StayFoolish

- [ ] Implement `StayFoolishFir { expr, state, ab, parent,
      line_in_parent }`.
- [ ] BRANING: step inner expr, mirror state.
- [ ] Cross-validation: SF tests pass.

## Phase 4 — Clone operations (FOOP-7)

- [ ] Implement `constanic_clone(source) -> FirBuilder` per FOOP-7:
  - [ ] Precondition check: panic on non-constanic source.
  - [ ] CONSTANT/INDEPENDENT/NK: return wrapping builder
        (share-by-reference semantics).
  - [ ] WOCONSTANIC/NOTFOUNDIC: shallow copy with state reset to
        BRANING; AB unchanged.
  - [ ] Recurse into children: rewrite parent pointers; reset
        WOCONSTANIC/NOTFOUNDIC descendants to BRANING; share
        CONSTANT/INDEPENDENT/NK descendants by reference.
- [ ] Implement `preconstanic_clone(source) -> FirBuilder` per
      FOOP-7:
  - [ ] Precondition check: panic on constanic source.
  - [ ] Extend AB by `(source.parent, source.line_in_parent)`.
  - [ ] Apply line-aware AB compression per FOOP-51.
  - [ ] Preserve NYES state.
  - [ ] Recurse into pre-constanic children with parent rewrites.
- [ ] Implement `Builder` and `BuilderFrom` patterns per FOOP-51.
- [ ] Wire `constanic_clone` into search resolution: every
      search_result is `constanic_clone(target).setParent(self_as_parent).build()`.
- [ ] Cross-validation: re-coordination tests pass (concatenation
      that requires resolving a brane in a new host).

## Phase 5 — FOOP-51 short-circuit accumulation

- [ ] Implement short-circuit collapse for chained Search
      `search_result` pointers per FOOP-51 §"Short-circuit
      accumulation".
- [ ] Confirm AB compression runs after every accumulation.

## Phase 6 — Determinism invariant tests (FOOP-51)

Mandatory unit tests per FOOP-51:

- [ ] Test 1: own-brane hit.
- [ ] Test 2: AB hit (single entry).
- [ ] Test 3: multi-AB hit.
- [ ] Test 4: AB-line-bound respected.
- [ ] Test 5: parent-chain hit.
- [ ] Test 6: NOTFOUNDIC (search exhausted).
- [ ] Test 7: WOCONSTANIC pointing at NYE target.
- [ ] Test 8: WOCONSTANIC pointing at CONSTANT target.
- [ ] Test 9: short-circuited chain — verify accumulated AB on
      collapsed target.
- [ ] Test 10: nested constanic clone (clone of a clone of a brane).

For each test: build two trees (A: stepped normally; B: same tree
but with `search_result` cleared mid-evaluation). Assert byte-
identical output.

## Phase 7 — Catch up to UBC

- [ ] Run cross-validation against the full `.foo` corpus
      (`test-resources/`).
- [ ] For each failing test: investigate, fix, re-run.
- [ ] Document remaining divergences (if any are intentional, e.g.,
      state-field differences) in FOOP-61's open-questions or as a
      new FOOP.
- [ ] Goal: every value-equivalence test passes.

## Phase 8 — Cleanup and review

- [ ] Code review: check for FOOP-7 caller-pattern compliance
      (`.setParent(...).build()` chain at every search-result site).
- [ ] Code review: check for FOOP-51 determinism invariant
      (search_result never re-resolved once set, except via clone
      reset to BRANING).
- [ ] Verify FOOP-61 stage-completion invariant in tests: a brane
      in any constanic state has a complete EMBRYONIC and BRANING
      history.
- [ ] Promote FOOP-61, FOOP-51, FOOP-7 from Brewing → Final →
      Implementing → complete.

## Worktree

- [ ] Create worktree at `${HOME}/tmp/foolish-worktrees/2120-foop-61`
      with branch `foop/61-ubcb-state-machine`
- [ ] Verify all work is complete in
      `${HOME}/tmp/foolish-worktrees/2120-foop-61` and committed to
      `foop/61-ubcb-state-machine`
- [ ] Merge `foop/61-ubcb-state-machine` to alpha

## Notes

- **Why a parallel crate, not a fork.** UBC's FIR and stepping
  evolve under FOOP-31 (SPA1). UBCb diverges from FIR-down. Two
  crates with shared parser keeps both implementations independent
  while sharing the syntactic frontend.
- **Why cross-validation drives development.** UBCb must produce
  the same VALUES as UBC on all `.foo` tests (state fields may
  differ; values must match). Driving development against the
  cross-validation harness from Phase 2 onward catches divergences
  early.
- **Why FIR-down rewrite.** UBCb's NYES, AB-aware FIR fields,
  proto-brane universal contract, and split clone operations differ
  enough from UBC that incremental migration would be confusing.
  Clean rewrite is faster.

## Last Updated

**Date**: 2026-05-09
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.7 xHigh effort
**Changes**: Initial plan. Sets up parallel crate
`foolish-core-ubcb` (rewriting from FIR down, sharing only parser
with `foolish-core`) and dedicated `foolish-crossvalidation` harness
for value-equivalence testing.
