# FOOP-84.plan — deadbrane

Worktree variables (expanded):
- WORKTREE_ORIGIN_BRANCH=alpha
- WORKTREE_ORIGIN_PATH=/home/hcbusy/foolish-rust
- WORKTREE_BRANCH_NAME=foop-84-deadbrane
- WORKTREE_FULL_FS_PATH=/home/hcbusy/tmp/foolish-worktrees/foop-84-deadbrane

---

- [ ] Begin work: commit FOOP-84.md and FOOP-84.plan.md to origin, check `begun: [x]` in frontmatter
      (YYYY-MM-DD HH:MM)
- [ ] Create worktree at /home/hcbusy/tmp/foolish-worktrees/foop-84-deadbrane with branch `foop/foop-84-deadbrane`

### Part A: FirID cloning semantics tests

- [ ] (read §FirID cloning semantics of FOOP-84.md)
- [ ] Write unit test `firid_constant_clone_shares_identity` in `foolish-ubca/src/fir_kinds.rs` tests module — constanic-clone a `Constant` FIR, assert clone's FIRID equals original's
- [ ] Write unit test `firid_independent_clone_shares_identity` — constanic-clone an `Independent` FIR, assert clone's FIRID equals original's
- [ ] Write unit test `firid_nonconstanic_clone_gets_new_id` — constanic-clone a pre-constanic Operator (BRANING state), assert clone's FIRID differs
- [ ] Write unit test `firid_brane_clone_gets_new_id` — constanic-clone a Brane FIR (even if Constant), assert clone's FIRID differs
- [ ] Write unit test `firid_nk_clone_gets_new_id` — constanic-clone an NK FIR, assert clone's FIRID differs
- [ ] Write unit test `firid_econstanic_clone_gets_new_id` — constanic-clone an ECONSTANIC FIR, assert clone's FIRID differs
- [ ] Run `cargo test -p foolish-ubca -- firid` — all FirID tests pass

### Part B: FoolishIndex implementation

- [ ] (read §FoolishIndex of FOOP-84.md)
- [ ] Create `foolish-ubca/src/foolish_index.rs` — FoolishIndex module
  - [ ] Define `FoolishIndex(pub Vec<i32>)` — purely numerical, no searches, no names
  - [ ] Implement `FoolishIndex::resolve(&self, root: &FirRef) -> Option<FirRef>` — walk the brane tree following each signed index; return None on any out-of-bounds or non-brane intermediate
  - [ ] Implement `FoolishIndex::to_string(&self) -> String` — serialize to `(1,0,0,10,-1)` form
  - [ ] Implement `FoolishIndex::from_str(s: &str) -> Result<Self, _>` — parse from `(1,0,0,10,-1)` form
  - [ ] Implement `FoolishIndex::push(&mut self, index: i32)` and `FoolishIndex::parent(&self) -> Option<FoolishIndex>` — navigate up/down
- [ ] Wire `foolish_index` module into `foolish-ubca/src/lib.rs`

### Part C: Deadbrane analysis implementation

- [ ] (read §Deadbrane analysis of FOOP-84.md — frame of reference = (brane, FI))
- [ ] Create `foolish-ubca/src/deadbrane.rs` — Deadbrane analysis module
  - [ ] Define `DeadbraneReport` struct with `reachable: Vec<FoolishIndex>`, `directly_useless: Vec<(FoolishIndex, String)>`, `transitively_useless: Vec<(FoolishIndex, String, Vec<String>)>` fields
  - [ ] Implement `BraneFir::is_useless(&self, self_ref: &FirRef, fi: &FoolishIndex, query_fir: &FirRef) -> bool` — frame-of-reference check: from (this brane, fi), is query_fir referenced by any statement onward or any descendant?
  - [ ] Implement `BraneFir::deadbrane_report(&self, self_ref: &FirRef) -> DeadbraneReport` — full analysis: for each named statement at position p, check is_useless((p,), s.body); fixed-point iteration for transitive; produce FoolishIndex-addressed report
  - [ ] Implement FirRef-identity-based reference detection: walk a statement's body FIR subtree, check if any search/expression resolves to the query_fir (by pointer identity or FirID)
- [ ] Wire `deadbrane` module into `foolish-ubca/src/lib.rs`

### Part D: Unit tests

#### FoolishIndex tests

- [ ] Write unit test `fi_resolve_basic` — `(0)` resolves to first statement, `(-1)` resolves to last
- [ ] Write unit test `fi_resolve_nested` — `(1, 0)` resolves into body of 2nd statement
- [ ] Write unit test `fi_resolve_out_of_bounds` — returns None
- [ ] Write unit test `fi_resolve_non_brane_intermediate` — returns None when intermediate body is not a brane
- [ ] Write unit test `fi_serialization_roundtrip` — `(1,0,0,10,-1)` → parse → to_string → same
- [ ] Write unit test `fi_parent` — `(1, 0, 3).parent()` yields `(1, 0)`

#### Indexing modular arithmetic tests (§Formal indexing semantics)

- [ ] Write unit test `index_contextless_positive_in_bounds` — `{a;b;c;}#1` → `b`
- [ ] Write unit test `index_contextless_positive_wraps` — `{a;b;c;}#5` → `c` (5 mod 3 = 2), `#3` → `a` (3 mod 3 = 0)
- [ ] Write unit test `index_contextless_negative_wraps` — `{a;b;c;}#-1` → `c`, `#-4` → `c`, `#-3` → `a`
- [ ] Write unit test `index_contextless_zero` — `{a;b;c;}#0` → `a`
- [ ] Write unit test `index_contextless_single_element` — `{a;}#0`, `#1`, `#-1` all → `a`
- [ ] Write unit test `index_contextless_large_number` — `{a;b;c;}#100` → `b`, `#-100` → `c`
- [ ] Write unit test `index_contexted_forward_in_bounds` — find `b` at idx 1, `&1` → `c` ((1+1) mod 4 = 2)
- [ ] Write unit test `index_contexted_forward_wraps` — find `c` at idx 2, `&1` → `a` ((2+1) mod 3 = 0)
- [ ] Write unit test `index_contexted_backward_in_bounds` — find `c` at idx 2, `&-1` → `b` ((2-1) mod 4 = 1)
- [ ] Write unit test `index_contexted_backward_wraps` — find `a` at idx 0, `&-1` → `c` ((0-1) mod 3 = 2)
- [ ] Write unit test `index_contexted_zero` — find `b` at idx 1, `&0` → `b`
- [ ] Write unit test `index_contexted_large_offset` — find `a` at idx 0, `&100` → `b` ((0+100) mod 3 = 1)
- [ ] Write unit test `index_contexted_negative_large` — find `b` at idx 1, `&-100` → `c` ((1-100) mod 3 = 2)

#### Deadbrane tests

- [ ] Write unit test `deadbrane_directly_useless` — compile `{a=1; b=2;}` where `a` is never referenced from (brane, (0)); assert `a` in directly-useless set at FI `(0)`
- [ ] Write unit test `deadbrane_frame_of_reference` — nested brane where `a` is useless from (inner, (0)) but useful from (outer, (0)); assert inner report says useless, outer report says reachable
- [ ] Write unit test `deadbrane_transitively_useless` — compile `{a=1; b=a+1;}` where `b` is never referenced; assert both `a` and `b` are useless
- [ ] Write unit test `deadbrane_reachable` — compile `{a=1; b=a+1; c=b+1;}` where `c` is used externally; assert all reachable
- [ ] Write unit test `deadbrane_anonymous_excluded` — anonymous statements are not candidates
- [ ] Write unit test `deadbrane_empty_brane` — empty brane produces empty report
- [ ] Write unit test `deadbrane_cycle` — mutual reference cycle with no external refs; both transitively useless
- [ ] Run `cargo test -p foolish-ubca -- foolish_index` — all FoolishIndex tests pass
- [ ] Run `cargo test -p foolish-ubca -- index_contextless` — all contextless modular tests pass
- [ ] Run `cargo test -p foolish-ubca -- index_contexted` — all contexted modular tests pass
- [ ] Run `cargo test -p foolish-ubca -- deadbrane` — all Deadbrane tests pass

### Part E: Approval test

- [ ] Write `foolish-ubca/snapshot_tests/input/deadbrane_report.foo` — Foolish program with a mix of reachable, directly-useless, and transitively-useless statements (frame-of-reference demonstrated)
- [ ] Run `cargo test -p foolish-ubca --lib -- deadbrane_report` — approve snapshot
- [ ] Write and verify `foolish-ubca/snapshot_tests/input/foop_84_comprehensive.foo`

### Part F: Cleanup and merge

- [ ] Verify all work is complete in /home/hcbusy/tmp/foolish-worktrees/foop-84-deadbrane and committed to `foop/foop-84-deadbrane`
- [ ] Run `cargo clippy -D warnings` — zero warnings
- [ ] Run `cargo fmt --check` — no formatting needed
- [ ] Run `cargo test --workspace` — all tests pass
- [ ] Merge `foop/foop-84-deadbrane` to `alpha`
- [ ] STOP! STOP!! STOP!!! ASK HUMAN to check this box before continuing. UNDER NO CIRCUMSTANCES will Agent continue past this point automatically!!
  - [ ] Present human with the `cd /home/hcbusy/tmp/foolish-worktrees/foop-84-deadbrane` command and ask them to review snapshots BEFORE checking the parent checkbox.
- [ ] Cleanup /home/hcbusy/tmp/foolish-worktrees/foop-84-deadbrane
  - [ ] Check that .plan.md has all but Cleanup checkboxes completed
  - [ ] Remove /home/hcbusy/tmp/foolish-worktrees/foop-84-deadbrane
  - [ ] This is the last sub-task checkbox to be checked in this block
