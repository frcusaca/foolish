# FOOP-52 Plan: Repair FVM evaluation bugs — snapshot review round 2

**Created**: 2026-06-06
**Status**: In Progress
**Bugs**: 15 (6 groups, includes Bug 15 from FOOP-32)

---

## Phase 0: Scope Refactoring (Top Priority)

**Goal**: Replace the accumulating `Scope` struct with a lightweight Brane wrapper.
Scope holds a reference to the brane, the current statement position, and a parent
scope. No cloning for children. Backward search iterator delegates to parent when
local brane is exhausted.

**Must be completed before any other phase.**

- [ ] Define `Scope<'a>` struct: `brane: &'a NormalBraneFir`, `stmt_idx: usize`,
      `parent: Option<&'a Scope<'a>>`, `stmts: &'a [StatementFir]`
- [ ] Implement `IbStmtsIterator` — iterates backward from `stmt_idx-1` to `0`
      through current brane statements
- [ ] Implement `AbibStmtsIterator` — iterates `ib_stmts` then delegates to
      `parent.abib_stmts` (termination: parent is None)
- [ ] Implement `Scope::search_local(pattern)` using `ib_stmts`
- [ ] Implement `Scope::search(pattern)` using `abib_stmts`
- [ ] Implement `Scope::child(brane, stmt_idx)` for nested brane evaluation
- [ ] Implement `Scope::stmt_at_offset(offset)` for unanchored seek (`#-1`, `#-2`)
- [ ] Rename `re_step_brane_bodies` to `braning_step`
- [ ] Refactor `braning_step` to use new scope (no cloning, no entries
      accumulation)
- [ ] Remove `reset_searches` (ubc.rs:260) — scope handles forward reference
      prevention; `constanic_clone` handles NYES transitions
- [ ] Remove old `Scope` struct, `entries` field, and `push()` method
- [ ] Update all scope consumers (`step_boxed`, `step_with_scope`, etc.)
- [ ] Verify all 64 approved snapshots pass
- [ ] Verify all unit tests pass

---

## Phase 1: Source-Order Resolution / Backward Search (Bugs 1.1–1.3, 2.3, 6.1–6.2)

**Goal**: With Phase 0's scope in place, implement proper backward search. Each
statement loops backward from its position. If not found, ask the parent brane —
the parent knows where the search came from because depth-first evaluation means
the parent's statement currently being evaluated IS the starting position.

**Depends on**: Phase 0 (scope refactoring)

- [ ] Fix `braning_step`: use Phase 0's `Scope::child()` to create scopes
      for each statement (no stale brane references)
- [ ] Implement backward search: loop backward from current position using
      `Scope::backward_iter()`
- [ ] Implement parent brane search: iterator delegates to parent scope when local
      brane is exhausted
- [ ] Test Bug 1.1: `{y = x; x = 42;}` — `y` should be Search/ECONSTANIC, not
      `Int(42)`
- [ ] Test Bug 1.2: `{outer = {val = x}; x = 100;}` — `val` should be
      Search/ECONSTANIC, not `Int(100)`
- [ ] Test Bug 1.3: `{nested = {inner = {val = x}}; x = 42;}` — `val` should be
      Search/ECONSTANIC, not `Int(42)`
- [ ] Test Bug 2.3: `{x = 10; x; x = 20; x;}` — second `x` should be `10`, not
      `20`
- [ ] Test Bug 6.1: `{a=10, b=20, c=30, result=#-1+#-2, result2=#-1*#-2,
      result3=#-1-#-2;}` — no invariant violations
- [ ] Test Bug 6.2: `{a=1; b={c=#-1; d=2; e=#-1}; f=#-1;}` — no invariant
      violations
- [ ] Verify no regressions in 64 approved snapshots

---

## Phase 2: Scope Boundary Correctness (Bug 2.1)

**Goal**: Fix search to correctly cross parent brane boundaries when the
identifier IS defined before the nested brane in source order.

- [ ] Analyze how brane boundary crossing works in current search implementation
- [ ] Ensure search crosses brane boundaries for identifiers defined BEFORE the
      nested brane (but NOT for forward references — Phase 1)
- [ ] Test Bug 2.1: `{a=10; b=20; sum=a+b; nested={inner=sum/2};
      result=nested.inner;}` — `sum` should resolve to `Int(30)` inside `nested`,
      `inner` should be `Int(15)`, `result` should be `Int(15)`
- [ ] Verify no regressions in 64 approved snapshots

---

## Phase 3: Search Tree Resolution (Bugs 2.2, 3.2, 4.1)

**Goal**: When a search resolves to a CONSTANT value, collapse the search tree in
the substituted expression.

- [ ] Analyze how search results are substituted into expressions
- [ ] When a search finds a CONSTANT value, replace the Search FIR with the
      resolved value (not the full search tree)
- [ ] Test Bug 2.2: `{a=1; b={c=a+1; d=c+1};}` — `d` should NOT contain a
      search for `a` in its AST (only search for `c`)
- [ ] Test Bug 3.2: `{a={x=ref}; b={y=2}; c=a b;}` — `c` should contain
      `x=Search(ref, ECONSTANIC)` from `a`, not drop it
- [ ] Test Bug 4.1: `{x=10; y=20; z=30; sum=x+y+z; avg=sum/3;}` — `avg` should
      be `Int(20)`, not WOCONSTANIC
- [ ] Verify no regressions in 64 approved snapshots

---

## Phase 4: Concatenation Precedence (Bug 3.1)

**Goal**: Fix parser/evaluator to handle `a b` as concatenation of two operands.

- [ ] Analyze how `a b` is currently parsed (as `a` followed by search `b` on `a`?)
- [ ] Fix precedence: `a b` should be two independent operands being concatenated
- [ ] Test Bug 3.1: `{target={a=1; c=2; c={a=1,b=2,c=3}}; b1={x=10};
      result=b1 target.c;}` — `result` should be `{x=10; a=1; b=2; c=3}`
- [ ] Verify no regressions in 64 approved snapshots

---

## Phase 5: SFF/SF Marker Implementation (Bugs 5.1–5.3)

**Goal**: Implement SF/SF Marker Specification (UBC-specific) as defined in
FOOP-52 §"SF/SF Marker Specification".

**SFF (`<<...>>`):**
- [ ] Modify search initialization: inside SFF, set search state to ECONSTANIC
      directly (skip EMBRYONIC/BRANING)
- [ ] Verify SFFMark is transparent to constanic_clone (strips wrapper, clones
      inner content)
- [ ] Verify no internal cloning occurs (SFF doesn't perform searches)

**SF (`<...>`):**
- [ ] Modify constanic_clone: when sfcc=True, preserve ECONSTANIC/WOCONSTANIC
      states (instead of resetting to EMBRYONIC/BRANING)
- [ ] Modify constanic_clone: strip SFMark wrappers (clone inner content)
- [ ] Pass sfcc=True recursively when cloning inside SF context
- [ ] Verify CONSTANT stays CONSTANT either way

**Tests:**
- [ ] Test Bug 5.1: `{a=1, b=2; inner={c=<<a+b>>; c}; inner;}` — searches should
      be ECONSTANIC, not EMBRYONIC
- [ ] Test Bug 5.2: `{x=5; y=10; inner={calc=<<x+y>>; doubled=calc*2};}` — same
      ECONSTANIC requirement
- [ ] Test Bug 5.3: `{x=10; y=<x>; z=y+5;}` — verify SF behavior is correct
      (formalization, not behavioral fix)
- [ ] Test Example 3 (sfcc preservation): `{a=1; f=a*b; b=0; g=<f>; r=g;}` — g
      preserves ECONSTANIC for b, r resolves when b becomes available
- [ ] Test Example 4 (SFF then SF): `{f=<<z+zz+zzz>>; g=<f>; h=<<f>>;}` —
      SFFMark stripped during sfcc clone, SFF wrapping SFF
- [ ] Test Example 7 (SF with SFF inside): `{b=1,c=2;d=f; a=<b+<<c>>+d>;
      aa=a; f=1; aaa=<a>; aaaa=a;}` — sfcc preserves d's WOCONSTANIC state
- [ ] Test SFF edge case: `{f=<<{}^>>;}` — seek beyond brane bounds inside SFF
      should be ECONSTANIC (not NK)
- [ ] Verify no regressions in 64 approved snapshots

---

## Phase 6: Verification (Bugs 6.1–6.2)

**Goal**: Confirm that Phase 1's backward search fix resolves the invariant
violations. `constanic_clone` should NEVER encounter NYE — the invariant is
correct, the bug was the stale scope.

- [ ] Verify `constanic_clone` no longer encounters NYE after Phase 1 fix
- [ ] Test Bug 6.1: `{a=10, b=20, c=30, result=#-1+#-2, result2=#-1*#-2,
      result3=#-1-#-2;}` — all results should be CONSTANT
- [ ] Test Bug 6.2: `{a=1; b={c=#-1; d=2; e=#-1}; f=#-1;}` — no invariant
      violations
- [ ] Verify no regressions in 64 approved snapshots

---

## Final Verification

- [ ] Run `cargo insta test -p foolish-core --lib` to regenerate all `.snap.new`
      files
- [ ] Review all 15 regenerated snapshots for correctness
- [ ] Verify all 64 approved `.snap` files still pass
- [ ] Remove `!!! WIP FOOP-52 !!!` markers from input files
- [ ] Update FOOP-52.md status to `Final`

---

## Notes

- Phase 0 (scope refactoring) must be completed before any other phase
- Phases 1–3 are tightly coupled (all involve search behavior) and may be
  implemented together if the root cause is shared
- Phase 4 (concatenation precedence) may require parser changes
- Phase 5 (SFF/SF) is independent and can be done in parallel with other phases
- Phase 6 (verification) confirms Phase 1 fixes the invariant violations

## Last Updated

**Date**: 2026-06-06
**Updated By**: opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Added Phase 0 (scope refactoring). Added `reset_searches` removal.
Added `ib_stmts` and `abib_stmts` iterators. Updated bug count to 15. Updated
Phase 6 to be verification-only (no code changes needed).
