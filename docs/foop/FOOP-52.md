---
foop: 25
title: Owned-FIR evaluator rewrite + repair FVM evaluation bugs (snapshot review round 2)
author: opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4 (rewritten by Claude Code; Sonnet 4.6)
status: Draft
type: Major
created: 2026-06-06
phase: phase-2
supersedes: []
---

# FOOP-52: Owned-FIR evaluator rewrite + repair FVM evaluation bugs

## Abstract

This FOOP does two things, in order:

1. **Rework the FVM scope / search machinery.** The FIR keeps its
   `Rc<RefCell<Fir>>` children (shared, mutable-through-`RefCell` as evaluation
   progresses), and gains **used `Weak<RefCell<Fir>>` parent back-pointers**. Search
   becomes a recursion over the FIR graph itself — a brane searches its own
   statements backward from a line, and on a miss widens to the next enclosing brane
   (reached by walking the parent pointers up to the containing brane) — replacing the
   accumulating flat `Scope`. Evaluation trickles
   **down** from the root (`child.step()`), each FIR reads up and down the graph but
   **mutates only itself**, and the outermost owning call commits the result. The
   gate for this phase is that all 64 existing approved snapshots pass
   **byte-identical** — the proof that the rework is behavior-preserving.

2. **Repair 15 bugs** discovered during human review of `.snap.new` files in
   `foolish-core/snapshot_tests/approved/` (the second batch after FOOP-32's eight).
   They fall into six categories: forward reference resolution, scope resolution
   across brane boundaries, search/concatenation precedence, operator transparency,
   SF/SFF marker semantics, and unanchored seek invariant violations. Includes Bug 15
   (negative-seek boundary clamping) from FOOP-32. The 15 WIP files are the
   acceptance test for the rework.

**Why a rework, not a patch:** the bugs share root causes that the current scope
machinery makes hard to fix cleanly. The original `Scope` is an accumulating flat
list that pre-pushes all names (so forward references wrongly resolve) and clones
itself plus the entire statement vector inside its per-statement loop (O(n²)). The
fix is to let the FIR graph *be* the scope: each brane resolves names by searching
its own statements backward from the requesting line, then asking its parent. This
needs the parent pointers to be real and used (they exist today but are vestigial),
and it lets the flat `Scope` accumulation go away. The ownership model is unchanged —
`Rc` children, `Weak` parents — so this is a targeted rework of search + scope, not
an evaluator rewrite. See [Implementation Approach](#implementation-approach) and the
plan file `docs/foop/FOOP-52.plan.md`.

**Ownership model (settled):** each node owns its children (`Rc`) and holds a
readable, non-owning pointer to its structural parent (`Weak`); writes land on self.
- **Children:** `Rc<RefCell<Fir>>` — FIRs are shared (a resolved search holds a
  read-only reference to the immutable CONSTANT/INDEPENDENT node it found — shared,
  never copied) and mutate through `RefCell` as evaluation advances.
- **Parents:** `Weak<RefCell<Fir>>` — a child holds a *readable, non-owning*
  back-pointer to its **immediate structural parent**, which is usually NOT a brane
  (`x` and `y` → the `+` operator → the statement → the brane → the parent brane's
  statement → …). `Weak` makes the up-edge readable without an ownership cycle.
  Finding the nearest enclosing *brane* is `get_brane()`'s job (walk up until a brane
  is reached), distinct from `get_parent()`.
- **Access up and down, mutate only self.** A FIR may read its children and its
  ancestors to compute, but writes only itself; the owning caller commits.

(`Box<Fir>` owned bodies were considered and rejected: CONSTANT/INDEPENDENT nodes
are shared by read-only reference and children need readable parent pointers, so
unique `Box` ownership does not fit. `Rc`/`Weak` is the idiomatic and smaller change —
the code is already `Rc<RefCell>`.)

## Exception to the "no failing tests" rule (read first)

AGENTS.md states: **"NEVER start large project segment work WHEN ANY tests are
broken."** FOOP-52 is explicitly granted an **exception** to that rule, because the
broken tests ARE the work:

- The 15 WIP input files (`!!! WIP FOOP-52 !!!`) have no approved `.snap` yet, so
  under plain `cargo test -p foolish-core --lib` they fail (insta stops on the first,
  alphabetically `anchored_seek_negative_boundary`). This is by design — they are the
  acceptance test for the fixes, not a regression.
- The real baseline is GREEN: `cargo insta test -p foolish-core --lib` passes (insta
  defers snapshot mismatches to `.snap.new` rather than failing). The 64 approved
  snapshots are the stable oracle.

**Therefore:** do NOT halt FOOP-52 on the "no failing tests" rule, and do NOT try to
make the 15 WIP files pass by any means other than fixing the bugs. The discipline
this FOOP keeps in place instead:

1. **Phase 1 (the rewrite) must keep the 64 approved snapshots byte-identical.** That
   is the gate. The rewrite touches nothing the bugs touch; if a `.snap` moves, stop
   and investigate (likely `wo_short_circuit`).
2. The 15 WIP files stay failing through Phase 1 (expected) and are fixed + promoted
   one bug group at a time in Phases 2+.
3. Do not fix the ~58 other pending `.snap.new` files — out of scope.

The exception is narrow: it permits *starting* Major work with the 15 WIP files red.
It does NOT permit ignoring NEW breakage — any of the 64 approved snapshots breaking,
or any unit test breaking, halts work as usual.

## Motivation

During snapshot review, the human reviewer annotated `.snap.new` files with
`@Agent` comments indicating incorrect FVM output. These bugs affect correctness
of evaluation — forward references resolving when they shouldn't, scope searches
failing when they should succeed, concatenation losing operands, and internal
invariant violations producing `??? (constanic_clone called on NYE FIR)` in
output.

A companion bugs catalog exists at `docs/foop/FOOP-52.bugs.md`.

## Bug Groups

### Group 1: Forward Reference Resolution (3 bugs)

The FVM resolves forward references to their values when it should leave them as
Search FIRs in ECONSTANIC state. In Foolish, a reference to a name defined AFTER
the reference in source order should not resolve — it should remain a Search
with ECONSTANIC state.

#### Bug 1.1: Forward reference resolves in same brane

**File**: `forward_reference_basic.foo.snap.new`
**Input file**: `forward_reference_basic.foo`
```foolish
{y = x; x = 42;}
```

**Current (WRONG) output:**
```
y=42; !! @AGent, x's not defined yet this shoudl display: 1.) It's a search, 2.) search has no result and nyes of EConstanic
x=42
```

**Expected:** `y` should be a Search FIR in ECONSTANIC state (not `Int(42)`).
`x` is defined AFTER `y` in source order — the search should not find it.

**Root cause:** The FVM search does not respect source order. Each statement in a
brane knows its position (index). Backward search should loop backward from the
current statement's position, checking each previous statement. If not found, ask
the parent brane to search — the parent knows where the search came from because
depth-first evaluation means the parent's statement currently being evaluated IS
the position where backward search starts. The current implementation uses a flat
scope that doesn't respect positional backward search.

---

#### Bug 1.2: Forward reference resolves across one brane boundary

**File**: `forward_reference_in_nested_brane.foo.snap.new`
**Input file**: `forward_reference_in_nested_brane.foo`
```foolish
{outer = {val = x}; x = 100;}
```

**Current (WRONG) output:**
```
outer={
    val=100 !! @AGent wrong, x is s search and should be econstanic at this point
};
x=100
```

**Expected:** `val` should be a Search FIR in ECONSTANIC state. `x` is in the
parent brane, defined AFTER `outer`. Even without the brane boundary, the forward
reference should not resolve.

**Root cause:** Same as Bug 1.1 — forward reference resolution ignores source
order. Additionally, the search penetrates the brane boundary to find `x` in the
parent.

---

#### Bug 1.3: Forward reference resolves across two brane boundaries

**File**: `complex_forward_refs_in_nested_branes.foo.snap.new`
**Input file**: `complex_forward_refs_in_nested_branes.foo`
```foolish
{nested = {inner = {val = x}}; x = 42;}
```

**Current (WRONG) output:**
```
nested={
    inner={
        val=42  !!@Agent, this is wrong. X comes after the nested statement, it cannot be found at this point, not yet.
    }
};
x=42
```

**Expected:** `val` should be a Search FIR in ECONSTANIC state. `x` is defined
in the outermost brane AFTER `nested`, and is separated by TWO brane boundaries
(`nested` and `inner`).

**Root cause:** Same as Bugs 1.1/1.2, compounded by multi-level brane boundary
penetration.

---

### Group 2: Scope Resolution Failures (3 bugs)

The FVM either fails to find identifiers that ARE in scope, or finds identifiers
that should NOT be visible.

#### Bug 2.1: Search fails to find parent-scope identifier

**File**: `complex_full_program_with_all_features.foo.snap.new`
**Input file**: `complex_full_program_with_all_features.foo`
```foolish
{a = 10; b = 20; sum = a + b; nested = {inner = sum / 2}; result = nested.inner;}
```

**Current (WRONG) output:**
```
sum=30;
nested={WOCONSTANIC    !!@Agents, this is wrong. sum is known at the time of `nested` definition. inner should be calculable to a constant.
    inner=Op/(
      ?(result=Op+(...), pattern='^sum$', UNANCHORED, WOCONSTANIC),
      2,
      WOCONSTANIC
    )
};
result=?(pattern='^inner$', ANCHORED, NK)  !!Agent, when `nested` is calculated to constant, this search result, too, should be constant.
```

**Expected:** `sum` is `Int(30)` in the parent scope. Inside `nested`, `inner = sum / 2`
should resolve `sum` to `Int(30)` and compute `Int(15)`. The entire `nested` brane
should be CONSTANIC, not WOCONSTANIC. Consequently, `result = nested.inner` should
resolve to `Int(15)`.

**Root cause:** The search for `sum` from within the `nested` brane fails to cross
into the parent scope. This is the opposite of Bug 1.3 — here the search is too
restrictive instead of too permissive.

---

#### Bug 2.2: Spurious search in AST for resolved identifier

**File**: `cross_scope_reference_chain.foo.snap.new`
**Input file**: `cross_scope_reference_chain.foo`
```foolish
{a = 1; b = {c = a + 1; d = c + 1};}
```

**Current (WRONG) output:**
```
b={WOCONSTANIC
    c=2;
    d=Op+(   !! @Agent this is completely wrong. the AST should have no search for `a` under the `d =` statement.
      ?(result=Op+(?(pattern='^a$', UNANCHORED), 1, WOCONSTANIC), pattern='^c$', UNANCHORED, WOCONSTANIC),
      1,
      WOCONSTANIC
    )
}
```

**Expected:** `c = a + 1` correctly resolves to `Int(2)`. Then `d = c + 1` should
resolve `c` to `Int(2)` and compute `Int(3)`. The AST for `d` should contain only
a search for `c`, NOT a search for `a`. The `a` reference was already resolved in
the `c` expression — it should not leak into `d`'s AST.

**Root cause:** When `c`'s value (an Operator FIR wrapping a search for `a`) is
substituted into `d`'s expression, the inner search for `a` is not being resolved
or collapsed. The substitution preserves the unresolved search tree instead of
using the resolved value.

---

#### Bug 2.3: Identifier shadowing looks ahead

**File**: `identifier_shadowing.foo.snap.new`
**Input file**: `identifier_shadowing.foo`
```foolish
{x = 10; x; x = 20; x;}
```

**Current (WRONG) output:**
```
x=10;
20;  !! @Agent, NOPE this is wrong, at this point x is 10 not 20.
x=20;
20   !! @Agent THIS is right.
```

**Expected:** The second expression `x;` (bare reference) should resolve to `Int(10)`
— the value of `x` at that point in source order. The third expression `x = 20`
redefines `x`. The fourth expression `x;` should resolve to `Int(20)`.

**Root cause:** SSA (Static Single Assignment) semantics are broken. The bare `x`
reference at position 2 is finding the LATER definition `x = 20` instead of the
earlier `x = 10`. The search is scanning forward instead of backward.

---

### Group 3: Search/Concatenation/Precedence (2 bugs)

Search results are lost during concatenation, or search resolution fails when
operands are already resolved.

#### Bug 3.1: Concatenation loses search result operand

**File**: `complex_search_and_concatenation.foo.snap.new`
**Input file**: `complex_search_and_concatenation.foo`
```foolish
{target = {a=1; c=2; c={a=1, b=2, c=3}}; b1 = {x=10}; result_1= b1(target.c); result_2 = b1 target.c;}
```

**Current (WRONG) output:**
```
result={  !!@Agents, b1 is `{x=10}`, target.c is `{a=1,b=2,c=3}`, so their concatenation should result in `{x=10,a=1,b=2,c=3}`, this result here is missing something.
    x=10
}
```

**Expected:** `b1 target.c` should concatenate `{x=10}` with `{a=1, b=2, c=3}`
to produce `{x=10; a=1; b=2; c=3}`. Only `b1` appears — `target.c` is lost.

**Root cause:** The parser/evaluator treats `b1 target.c` as `b1` followed by
`target.c` as a search operation on `b1`, rather than as two independent operands
being concatenated. Search has higher precedence than concatenation.

---

#### Bug 3.2: Concatenation drops unresolved search from operand

**File**: `concatenation_with_unresolved_search.foo.snap.new`
**Input file**: `concatenation_with_unresolved_search.foo`
```foolish
{a = {x=ref}; b = {y=2}; c = a b;}
```

**Current (WRONG) output:**
```
c={   !!@AGent, there should be a WConstanic search `x=Search(...` as first element of this c.
    y=2
}
```

**Expected:** `c = a b` should concatenate `{x=Search(ref, ECONSTANIC)}` with
`{y=2}` to produce `{x=Search(...); y=2}`. The `x=Search(...)` from `a` is
completely dropped.

**Root cause:** When concatenating branes, unresolved (constanic) entries are
being silently dropped instead of carried through.

---

### Group 4: Operator Transparency / Search Resolution (1 bug)

#### Bug 4.1: Operator stays WOCONSTANIC when operand is already resolved

**File**: `complex_brane_with_operations_and_search.foo.snap.new`
**Input file**: `complex_brane_with_operations_and_search.foo`
```foolish
{x=10; y=20; z=30; sum = x + y + z; avg = sum / 3;}
```

**Current (WRONG) output:**
```
sum=60;
avg=Op/(?(result=Op+(30, ?(pattern='^z$', UNANCHORED), WOCONSTANIC), pattern='^sum$', UNANCHORED, WOCONSTANIC), 3, WOCONSTANIC)
!!@Agent, here, since sum is constant, the avg should be computable, a WOCOSTANIC result is wrong.
```

**Expected:** `sum` is already `Int(60)`. The expression `avg = sum / 3` should
resolve `sum` to `Int(60)` and compute `Int(20)`. The operator should be
CONSTANT, not WOCONSTANIC.

**Root cause:** The search for `sum` inside the `avg` expression finds the
Operator FIR for `sum` (which is already CONSTANT `Int(60)`), but the search
result itself is not being simplified. The search preserves the full Operator
tree instead of extracting the constant value.

---

### Group 5: SFF/SF Marker Semantics (3 bugs)

The SFF (`<<...>>`) and SF (`<...>`) markers have underspecified behavior
regarding how searches inside them should transition states. See the
[SFF/SF Marker Specification](#sfsff-marker-specification-ubc-specific) section
for the full operational definition.

#### Bug 5.1: SFF searches stay EMBRYONIC instead of ECONSTANIC

**File**: `complex_sff_in_nested_brane.foo.snap.new`
**Input file**: `complex_sff_in_nested_brane.foo`
```foolish
{a=1, b=2; inner = {c = <<a+b>>; c}; inner;}
```

**Current (WRONG) output:**
```
c=Op+(?(pattern='^a$', UNANCHORED), ?(pattern='^b$', UNANCHORED), EMBRYONIC);
!! @Agent, this is not completely correct. in SFF, the most important thing is for ALL SEARCHES within the markers to immediately enter ECONSTANIC state.
```

**Expected:** Inside `<<...>>`, all searches should immediately enter ECONSTANIC
state (see [SFF specification](#sff----suppress-search--code-template)). Normal
evaluation continues from there. So `c` should be
`Op+(Search(a, ECONSTANIC), Search(b, ECONSTANIC), WOCONSTANIC)` — the searches
are ECONSTANIC (not EMBRYONIC), and the operator is WOCONSTANIC because its
operands are not yet resolved.

**Root cause:** SFF marker processing does not transition searches to ECONSTANIC.
The searches remain in EMBRYONIC state, which prevents further evaluation.

---

#### Bug 5.2: SFF with nested scope — same EMBRYONIC issue

**File**: `complex_sff_with_nested_scope.foo.snap.new`
**Input file**: `complex_sff_with_nested_scope.foo`
```foolish
{x = 5; y = 10; inner = {calc = <<x + y>>; doubled = calc * 2};}
```

**Current (WRONG) output:**
```
calc=Op+(?(pattern='^x$', UNANCHORED), ?(pattern='^y$', UNANCHORED), EMBRYONIC);
!! @Agents, again, SFF marker means all Foolish code inside it enters ECONSTANIC immediately, then normal brane evaluation continues.
doubled=30
```

**Expected:** `calc` should have ECONSTANIC searches (not EMBRYONIC), per the
[SFF specification](#sff----suppress-search--code-template). Note that
`doubled = calc * 2` correctly evaluates to `Int(30)` — this is because `calc`'s
searches eventually find `x` and `y` during the `doubled` evaluation. But the
SFF marker should have set them to ECONSTANIC from the start.

**Root cause:** Same as Bug 5.1 — SFF marker does not transition searches to
ECONSTANIC.

---

#### Bug 5.3: SF marker semantics — formalization needed

**File**: `complex_sf_in_expression.foo.snap.new`
**Input file**: `complex_sf_in_expression.foo`
```foolish
{x=10; y=<x>; z=y + 5;}
```

**Current output:**
```
y=?(result=10, pattern='^x$', UNANCHORED); !! @Agent, SF marker behavior is underspecified...
z=15
```

**Expected:** The SF marker `<...>` performs searches normally. When `y` is later
used in `z = y + 5`, the search is re-evaluated in the new context. The current
output for `y` shows the search correctly resolved (`result=10`), and `z=15` is
correct. The SF marker semantics are now formally specified in
[SF specification](#sf----preserve-search-state) — the key behavior is
constanic_clone with `sfcc=True`, which preserves ECONSTANIC/WOCONSTANIC states.
This test doesn't exercise that behavior because everything is already resolved.
See the specification examples for cases where sfcc preservation matters.

**Root cause:** SF marker behavior was not formally specified. The specification is
now defined in [SF specification](#sf----preserve-search-state). The current
implementation appears correct for this simple case.

---

### Group 6: Unanchored Seek Invariant Violations (2 bugs)

The FVM produces `INVARIANT-VIOLATED: constanic_clone called on NYE FIR` errors,
indicating an internal invariant is being violated during unanchored seek
evaluation.

#### Bug 6.1: Unanchored seek with operations triggers invariant violation

**File**: `complex_unanchored_seeks_with_operations.foo.snap.new`
**Input file**: `complex_unanchored_seeks_with_operations.foo`
```foolish
{a=10, b=20, c=30, result=#-1 + #-2, result2=#-1 * #-2, result3=#-1 - #-2;}
```

**Current (WRONG) output:**
```
result=50;   !! @Agents, this is correct.
result2=Op*(??? (constanic_clone called on NYE FIR), INVARIANT-VIOLATED: ..., 30, NK);
!!@AGent this should find result and c to add to 80.
result3=Op-(??? (constanic_clone called on NYE FIR), INVARIANT-VIOLATED: ..., NK)
!!@Agents, something is wrong, this shouldn't happen, in UBC this should never happen
```

**Expected:** `result = #-1 + #-2` correctly evaluates to `50` (30 + 20).
`result2 = #-1 * #-2` should be `30 * 20 = 600`. `result3 = #-1 - #-2` should
be `30 - 20 = 10`. All three should be CONSTANT.

**Root cause:** Unanchored seek (`#-1`, `#-2`) in the context of binary operations
triggers a `constanic_clone` call on a NYE (Not Yet Evaluated) FIR. This is an
internal invariant violation — the FVM should not be cloning NYE FIRs. The issue
may be in how unanchored seeks are resolved when combined with arithmetic
operators.

---

#### Bug 6.2: Nested brane boundary seek triggers invariant violation

**File**: `nested_brane_boundary.foo.snap.new`
**Input file**: `nested_brane_boundary.foo`
```foolish
{a = 1; b = {c = #-1; d = 2; e = #-1}; f = #-1;}
```

**Current (WRONG) output:**
```
b={NK
    c=??? (constanic_clone called on NYE FIR), INVARIANT-VIOLATED: ...;
    d=2;
    e=2
};
f=??? (constanic_clone called on NYE FIR), INVARIANT-VIOLATED: ...
```

**Expected:** Inside `b`, `c = #-1` should seek backward to `a` (not to `b`'s
parent). `e = #-1` should seek backward to `d = 2` and resolve to `Int(2)` (this
one works). Outside `b`, `f = #-1` should seek backward to `b`'s brane value.

**Root cause:** Unanchored seek across brane boundaries triggers the same
`constanic_clone` invariant violation as Bug 6.1. The seek logic may be
attempting to clone FIRs that haven't been evaluated yet when crossing brane
boundaries.

---

## Cross-Cutting Observations

### Forward Reference vs Scope Resolution

Bugs 1.1–1.3 (forward references resolve when they shouldn't) and Bug 2.1 (scope
search fails when it should succeed) are opposite symptoms of the same root cause:
the FVM search does not correctly distinguish between "defined before" and
"defined after" in source order. Fixing the source-order check should address both
groups simultaneously.

### Search Tree Substitution

Bug 2.2 (spurious search for `a` in `d`'s AST) and Bug 3.2 (concatenation drops
unresolved search) both relate to how resolved values are substituted into
expressions. When a value is substituted, its internal search tree should be
collapsed to the resolved value, not preserved as-is.

### SFF/SF Marker State Machine

Bugs 5.1–5.3 all relate to underspecified SFF/SF marker behavior. The
specification is now defined in [SF/SFF Marker Specification](#sfsff-marker-specification-ubc-specific).
The implementation must:
- SFF: Searches start at ECONSTANIC directly (skip EMBRYONIC/BRANING)
- SF: constanic_clone uses sfcc=True to preserve ECONSTANIC/WOCONSTANIC states
- Both markers are transparent to external constanic_clone (wrapper stripped)

### Invariant Violations

Bugs 6.1–6.2 both trigger `INVARIANT-VIOLATED: constanic_clone called on NYE FIR`.
This is a single bug in the unanchored seek evaluation path that manifests in
different contexts.

**Root cause analysis**: `constanic_clone` should NEVER encounter NYE. The
invariant violation is correct — the bug is that `constanic_clone` is being
called on NYE at all. The cause is a stale scope: `braning_step` creates
the scope's `current_brane` from `brane.statements.clone()` (the original reset
statements with EMBRYONIC bodies), not the stepped statements. When `IndexFir`
calls `index_in_brane`, it retrieves EMBRYONIC bodies from the stale scope.

**Why this is a scope bug, not an evaluation order bug**: UBC IS depth-first —
each statement is fully evaluated before the next. By the time statement N is
evaluated, statements 0–N-1 have been fully evaluated. An unanchored seek `#-1`
should find the evaluated body of the previous statement (CONSTANT, WOCONSTANIC,
or ECONSTANIC — never EMBRYONIC). The fix is ensuring the scope's brane contains
evaluated bodies.

**Fix**: This is resolved by Phase 1's backward search fix. When the scope's
brane references evaluated statements, `constanic_clone` will never see NYE from
Index. No additional safety check is needed — the invariant is correct.

---

## Architecture: parent-linked FIR graph + recursive search

This is Phase 1. It must be done before any bug repair, and its gate is that all
64 existing snapshots pass byte-identical.

### Ownership model: Rc children, Weak parents, mutate-self

The FIR keeps `Rc<RefCell<Fir>>` children and gains *used* `Weak<RefCell<Fir>>`
parent back-pointers. Parent owns children; children read parent.

- **Children — `Rc<RefCell<Fir>>`.** FIRs are shared, not uniquely owned: a resolved
  search holds a **read-only reference** (an `Rc` handle) to the immutable
  CONSTANT/INDEPENDENT node it found — shared, never deep-copied (`constanic_clone`
  of CONSTANT/INDEPENDENT/NK already returns `Rc::clone(source)`, `ubc.rs:467`). FIRs
  mutate through `RefCell` as evaluation advances.
- **Parents — `Weak<RefCell<Fir>>`.** Each FIR holds a *readable, non-owning*
  back-pointer to its parent: `x` and `y` → the `+` operator → the statement → the
  brane → the parent brane's statement → … . `Weak` makes the up-edge readable
  (`upgrade()` to access) without creating an ownership cycle or leak.
- **Statements — `NormalBraneFir.statements: Vec<Rc<RefCell<StatementFir>>>`, a
  FIXED-SIZE vector.** A brane holds its statements as shared, mutable handles
  (changed from the current `Vec<StatementFir>`). The vector is allocated once when
  the brane is built from the AST; its length never changes during evaluation —
  statements are stepped/replaced in place, never appended or removed.
  `StatementFir` already exists (`fir.rs:195`): minimally an RHS `body: FirRef`,
  optionally an LHS `name: Option<String>` (the assignment identifier — plain or
  characterized identifier string; either is fine), plus `state`. It GAINS two fields,
  **both set at construction** (and re-set on recoordination/clone): `parent:
  Weak<RefCell<Fir>>` pointing to the owning brane, and `line_number: usize` — its own
  0-based index into the parent's `statements` vec. So a statement always knows both
  its brane and exactly where it sits: `parent.statements[line_number]` is itself.
  Search iterates the vector and matches on each statement's LHS `name`. The stored
  `line_number` makes "which line am I?" a field read — no scan — so an unanchored
  search gets its `from_line` directly, and `line_of_child` is a trivial lookup (or
  unnecessary).
- **Access up and down, mutate only self.** Evaluation trickles **down** from the
  root brane (`child.step()`; the parent reaches children through
  `RefCell::borrow_mut`). A FIR may *read* its children and its ancestors to compute,
  but *writes only itself*. The outermost owning call commits the result.

Two trait methods give every FIR its upward access:

- **`get_parent(&self) -> Option<FirRef>`** — the immediate structural parent
  (`upgrade()` of the `Weak`). For the search `c` in `{b = 1 + c}`, the chain is
  `c` → `+` → statement `b` → the brane.
- **`get_brane(&self) -> Option<FirRef>`** — the nearest enclosing brane, defined
  recursively: *return `get_parent()` if it is a brane, else
  `get_parent().get_brane()`*. For `c` in `{b = 1 + c}`, `get_brane()` skips past `+`
  and statement `b` and returns the brane that contains `b` — exactly the brane whose
  earlier statements `c` should search.

  ```rust
  fn get_brane(&self) -> Option<FirRef> {
      let parent = self.get_parent()?;
      if parent.borrow().kind() == FirKind::NormalBrane {
          Some(parent)
      } else {
          parent.borrow().get_brane()
      }
  }
  ```

  `get_brane()` is the bridge a Search FIR uses to *start* resolution: an unanchored
  search calls `get_brane()` to find its home brane, takes its originating line from
  the enclosing statement's stored `line_number`, then calls
  `search_ancestral_branes(pattern, line_number)`.

Why not owned `Box<Fir>` (considered and rejected): CONSTANT/INDEPENDENT nodes are
shared by read-only reference, and children need readable parent pointers — neither
fits unique `Box` ownership without deep-copying immutable nodes (wrong) or raw
self-referential pointers (`unsafe`, fragile). `Rc`/`Weak` is the idiomatic, safe,
and *smaller* change — the code is already `Rc<RefCell>`. For UBC this is arguably
heavier than strictly needed, but for UBCb (which shuffles and replaces FIRs) `Rc` is
the right choice anyway, so one model serves both.

### What replaces the flat Scope

The current `Scope` (`ubc.rs:37-45`) is the thing to retire:

```rust
struct Scope {
    entries: Vec<(String, FirRef)>,  // accumulating flat list — pre-pushes ALL names
    current_brane: Option<FirRef>,   // stale snapshot of reset (EMBRYONIC) bodies
    current_stmt_idx: Option<usize>,
    // ...
}
```

Problems: `entries` accumulates every name including forward references (root of
Bugs 1.x/2.3); it is cloned per statement (O(n²), `ubc.rs:238`); `current_brane`
points at reset EMBRYONIC bodies (root of the Bug 6.x NYE invariant violations).

**The FIR graph IS the scope.** There is no separate `Scope` object holding a name
list. A brane resolves names by searching its own statements, then delegating to its
parent through the `Weak` parent pointer.

### Search = recursion over local members

Search is plain recursion; each function touches only its own struct's members:

- **`Brane::search_immediate_brane(pattern, from_line, direction)`** — search this
  brane's own statements, starting at `from_line`, going `direction` (backward for
  normal name resolution). Uses **`Brane::iterate_immediate_brane(from_line,
  direction)`** to walk its statements. Nearest match in the requested direction
  wins (correct shadowing/SSA). It does **not** see statements past `from_line` in
  the backward case → forward references are simply out of range.
- **`Brane::search_ancestral_branes(...)`** — when the immediate-brane search does
  not resolve and the situation demands widening, follow the `Weak` parent up to the
  containing brane and call *its* `search_immediate_brane`, bounded by the line at
  which this brane sits in its parent. This is the up-walk: a name defined before the
  nested brane resolves; one defined after does not — one mechanism covers both
  forward-ref suppression (Bugs 1.x) and legitimate parent resolution (Bug 2.1).
- **Unanchored seek (`#-1`, `#-2`)** indexes the immediate brane relative to
  `from_line`, reading already-evaluated earlier statements — never a NYE body, which
  is what makes the Bug 6.x `constanic_clone`-on-NYE violations unreachable.

Each of these is a method on the brane, accessing only local members. **The final
mutation is performed by the outermost call** — a method on the struct being
mutated (the search FIR records its resolved target; the brane records its statement
bodies). Reads go up and down the graph; the write lands on `self`.

`reset_searches` (`ubc.rs:260-323`) is removed: with positional `from_line` search,
forward references are out of range and never resolve, so there is nothing to reset.
`constanic_clone` still performs the per-reuse state reset when a brane is reused in
a new context (`ubc.rs:466-477`: ECONSTANIC→EMBRYONIC, WOCONSTANIC→BRANING).

### The three brane search methods (normative)

Search is **not** a free-standing recursive walker over a separate scope object — it
is three methods **on `NormalBraneFir`**, each touching only its own members. The
same `iterate_immediate_brane` / `search_immediate_brane` pair serves **both anchored
and unanchored** searches (anchored = "search this specific brane"; unanchored =
"search my own brane, then ancestors"). That shared factoring is the most
straightforward to implement and is required, not optional.

Storage: `statements: Vec<Rc<RefCell<StatementFir>>>` — a **fixed-size** vector
(allocated once when the brane is built from the AST; statements are stepped/replaced
in place, never appended or removed during evaluation). Each statement's `parent`
points to the owning brane (set at construction). `from_line` is the 0-based
statement index the search starts from. The iterator yields **statement handles**
(`Rc` clones) — it cannot return borrows out of a `RefCell`, so callers `borrow()`
the handle to read `name`/`body`.

```rust
impl NormalBraneFir {
    /// Walk THIS brane's own statements from `from_line` in `direction`, yielding a
    /// handle to each NAMED statement. Touches only self.statements.
    /// Backward: from_line-1 down to 0. Forward: from_line+1 up to len-1.
    /// Shared by anchored and unanchored search, and by the SFF/SF machinery.
    fn iterate_immediate_brane(
        &self,
        from_line: usize,
        direction: SearchDirection,
    ) -> impl Iterator<Item = Rc<RefCell<StatementFir>>> + '_ {
        let range: Box<dyn Iterator<Item = usize>> = match direction {
            SearchDirection::Backward => Box::new((0..from_line).rev()),
            SearchDirection::Forward  => Box::new((from_line + 1)..self.statements.len()),
        };
        range.filter_map(move |i| {
            let stmt = &self.statements[i];
            if stmt.borrow().name().is_some() { Some(Rc::clone(stmt)) } else { None }
        })
    }

    /// Resolve `pattern` within THIS brane only (no ancestor delegation).
    /// Returns the first matching statement's RHS body, in `direction` from
    /// `from_line`. This is what an ANCHORED search (`a.foo`) calls on the anchor's
    /// brane, and what `search_ancestral_branes` calls at each level.
    fn search_immediate_brane(
        &self,
        pattern: &str,
        from_line: usize,
        direction: SearchDirection,
    ) -> Option<FirRef> {
        let re = regex::Regex::new(pattern).ok()?;
        for stmt in self.iterate_immediate_brane(from_line, direction) {
            let s = stmt.borrow();
            if s.name().as_deref().is_some_and(|n| re.is_match(n)) {
                return Some(Rc::clone(s.body()));
            }
        }
        None
    }

    /// Resolve `pattern` for an UNANCHORED search: this brane first, then walk up
    /// the `Weak` parent chain. `from_line` is where the search originates in THIS
    /// brane. When delegating, the parent is searched from the line at which THIS
    /// brane sits in it (so names defined after the nested brane are out of range).
    /// Each step touches only local members; ancestry is the `Weak` parent pointer.
    fn search_ancestral_branes(
        &self,
        pattern: &str,
        from_line: usize,
    ) -> Option<FirRef> {
        // 1. Try our own brane, backward from the originating line.
        if let Some(found) =
            self.search_immediate_brane(pattern, from_line, SearchDirection::Backward)
        {
            return Some(found);
        }
        // 2. Widen to the enclosing brane, searched from where THIS brane sits.
        //    This brane's parent is the STATEMENT whose body it is; that statement
        //    knows its own line_number and its owning brane. No scan.
        let stmt_cell = self.enclosing_statement()?;        // None at root
        let stmt = stmt_cell.borrow();
        let our_line = stmt.line_number();                  // stored 0-based index
        let outer_brane_cell = stmt.parent_brane()?;        // the statement's brane
        outer_brane_cell.borrow().as_normal_brane()?
            .search_ancestral_branes(pattern, our_line)
    }
}
```

Notes for the implementer:

- **Statements are a fixed-size `Vec<Rc<RefCell<StatementFir>>>`.** Built once from
  the AST; the count never changes during evaluation (statements are stepped in
  place). Search matches on each statement's LHS `name()` (the assignment
  identifier — plain or characterized identifier string).
- **Each statement stores `parent` (owning brane) and `line_number` (its 0-based
  index in that brane's vec), both set at construction.** So `parent.statements[
  line_number]` is the statement itself. Widening in `search_ancestral_branes` is
  therefore a field read, not a search: take the enclosing statement, read its
  `line_number`, hop to its brane, recurse. `line_of_child` (if kept at all) is just
  `child.line_number()`; `Rc::ptr_eq` remains available as a debug-assert that
  `parent.statements[line_number]` really is the child.
- **Recursion = the `search_ancestral_branes` → parent `search_ancestral_branes`
  call**, terminating when `self.parent.upgrade()` is `None` (root brane). Ordinary
  recursive method call; the only subtlety is borrowing the parent through the
  `Weak`/`RefCell` (`upgrade()` then `borrow()`), so do not hold a `borrow_mut` on a
  node while recursing into its parent.
- **How a search starts (uses `get_brane`/`line_of_child`):** an unanchored Search
  FIR for `foo` calls `self.get_brane()` to find its home brane, asks that brane
  `line_of_child(self_or_enclosing_statement)` for its originating line, then calls
  `home_brane.search_ancestral_branes("^foo$", that_line)`.
- **Anchored search reuses the same primitives:** `a.foo` evaluates `a` to a brane,
  then calls `search_immediate_brane("^foo$", len, Backward)` on THAT brane — no
  ancestral widening, no `get_brane`. One `iterate_immediate_brane` underlies both.
- **Mutation stays at the caller:** these methods are pure reads (they return a found
  `FirRef`); the *search FIR's* `step` records the result on itself, and the brane's
  `step` records statement bodies. Reads go up/down the graph; writes land on `self`.
- **`SearchDirection`** already exists (`fir.rs:175`). Backward is the default for
  name resolution; forward exists for the forward-seek cases.

### Required unit tests for the search methods

In addition to the snapshot tests, write **direct unit tests** (in `unit_tests.rs`)
for these brane methods, on both a flat brane and nested branes — they are the load-
bearing primitives and must be tested in isolation, not only through whole-program
snapshots:

- `iterate_immediate_brane`: backward and forward yields, skips anonymous statements,
  empty brane, `from_line` at 0 and at len.
- `search_immediate_brane`: hit / miss / nearest-match-wins (shadowing) / pattern is
  a regex / `from_line` excludes later statements (forward-ref not found).
- `search_ancestral_branes`: resolves in immediate brane; resolves in parent when not
  local; does NOT resolve a name defined in the parent AFTER the nested brane;
  terminates at root (returns None) without panic; two-level nesting (grandparent).
- `line_of_child`: finds the correct parent line; behaves for deeply nested branes.
- `get_parent` / `get_brane`: for `{b = 1 + c}`, `c`'s `get_parent()` is the `+` (or
  enclosing statement); `c`'s `get_brane()` skips operator + statement and returns the
  containing brane. `get_brane()` on a statement returns its brane; on the root brane
  returns None; nested-brane case returns the *immediate* enclosing brane.

Build the test branes with the parser + the root brane's `.search(...)` per AGENTS.md
"Unit Test Readability" so the tests read clearly to human reviewers.

### Key Properties

- **Linear, not quadratic** — no per-statement clone of a flat scope list; search
  reads the brane's own statements directly and walks parents by `Weak` pointer.
- **`Rc` children, `Weak` parents** — shared FIRs (incl. read-only shared CONSTANT
  nodes), readable non-owning up-edge, mutate-self via `RefCell`.
- **Positional search** — `from_line` excludes forward references; nearest match in
  `direction` wins (correct shadowing).
- **Parent delegation bounded by `line_of_child`** — one mechanism handles both
  forward-ref suppression and legitimate parent-scope resolution.
- **Seek reads evaluated bodies** — unanchored seek indexes already-stepped earlier
  statements, never a NYE body → the `constanic_clone`-on-NYE violation is unreachable.

### Phase 1 checklist

The full, annotated checklist lives in `docs/foop/FOOP-52.plan.md` (Phase 1). In
brief: make `SearchFir.parent` / `NormalBraneFir.parent` real `Weak<RefCell<Fir>>`
back-pointers, set during construction/recoordination and USED by search; implement
`iterate_immediate_brane` / `search_immediate_brane` / `search_ancestral_branes` (+
`line_of_child`) as above; retire the flat `Scope` (`entries`/`current_brane`); keep
`step_one(&mut self, ...)` mutating self (replacement only on type change); add
`is_search()` trait predicate (Search/Index/HeadTail → true); replace
`fir_variant() -> &str` with `kind() -> FirKind`; remove `reset_searches`; rename
`short_circuit` → `wo_short_circuit` and reframe it as a query
(`wo_short_circuit(&self) -> &Fir`: follow the WOCONSTANIC target chain, return the
first non-WOCONSTANIC terminus or `self`; call site
`self.target = search_result_target.wo_short_circuit()`); add the search-method unit
tests; **gate: 64 snapshots byte-identical**.

---

## Semantic Immutability Principle

**Foolish code is semantically immutable once written.** Foolish expressions and
statements have fixed meaning and eventual value once computed — they do not vary
except in specified ways (in particular, everything stays the same until context
changes).

**Foolish FIR may track state changes as evaluation progresses.** The FIR
(Foolish Internal Representation) may have state changes during evaluation — this
is the implementation's way of tracking evaluation progress. In all cases, the
Foolish that a FIR represents should always be equivalent semantically,
considering NYES (Not Yet Evaluated State).

This principle must be reflected in all documentation updated as part of FOOP-52
repairs.

---

## SF/SFF Marker Specification (UBC-specific)

This section defines the operational semantics of SF (`<...>`) and SFF (`<<...>>`)
markers as implemented in the UBC (Unicellular Brane Computer). These markers
control how searches are initialized and how constanic_clone behaves during
evaluation.

### constanic_clone State Transitions

The base constanic_clone behavior (without markers):

| Source State | Result State |
| ------------ | ------------ |
| CONSTANT     | CONSTANT     |
| INDEPENDENT  | INDEPENDENT  |
| NK           | NK           |
| ECONSTANIC   | EMBRYONIC    |
| WOCONSTANIC  | BRANING      |
| PREMBRYONIC / EMBRYONIC / BRANING (NYE) | **INVARIANT-VIOLATED** |

The NYE rows are the FOOP's central invariant: `constanic_clone` must NEVER be
called on a Not-Yet-Evaluated FIR. If it is, that is a bug in the caller (a stale
or unstepped body reached the clone), and the clone produces an NK with an
INVARIANT-VIOLATED alarm (`ubc.rs:478-492`). The owned-body rewrite makes this
unreachable for seeks — `Scope::stmt_at_offset` returns only stepped earlier
siblings (see Bugs 6.1/6.2).

With SF marker context (`sfcc=True`), the search-result states are preserved
instead of reset:

| Source State | Result State |
| ------------ | ------------ |
| CONSTANT     | CONSTANT     |
| ECONSTANIC   | ECONSTANIC   |
| WOCONSTANIC  | WOCONSTANIC  |

SFF has no constanic_clone table of its own: SFF suppresses search *execution* (it
sets searches ECONSTANIC at creation), so at clone time its searches follow the
sfcc rules like any other.

### SFF (`<<...>>`) — Suppress Search / Code Template

**What counts as a "search" (important):** a search is any FIR that consults the
surrounding brane to find a value — name **Search**, positional **Index/seek**
(`#-1`), and **HeadTail** (`{}^`). These are exactly the FIRs for which
`is_search()` is `true`. This matters because of the underlying invariant:

> Once a Foolish expression's *text* is composed, the ONLY thing still indeterminate
> is its searches. Everything else has singular invariant meaning from its text
> alone.

So suppressing searches turns SFF into a pure **Foolish-code copier**: the code is
held, and the only parts that defer (resolve later, in a new context) are the
searches.

SFF suppresses search execution. ALL `is_search()` FIRs inside `<<...>>` are
generated but skip EMBRYONIC/BRANING — they enter ECONSTANIC directly. This applies
uniformly to name searches, `#-1` seeks, and `{}^` head/tail. Example consequence:
`{ ...; b = <<#-1>>; ... }` leaves `b` ECONSTANIC at definition; it resolves only
when `b` is referenced/coordinated elsewhere. All other evaluation (operators on
literals, etc.) proceeds normally.

**Normative description (state machine)**: SFF suppresses search execution.
Searches inside `<<...>>` are generated as Search FIRs but skip EMBRYONIC/BRANING
— they enter ECONSTANIC directly. This is the implementation model.

**Alternate description (code template)**: When the RHS of an assignment is
completely enclosed in SFF markers, the meaning is that when the LHS identifier is
referred to later, the LHS symbol is replaced with the Foolish code within the
`<<...>>` markers. The SFF content is stored unevaluated and substituted as-is
when referenced.

These two descriptions agree when the SFF body contains only name searches and
literals. The key thing both capture: the stored code is **re-resolved at each
reference site**, not captured-by-value at definition. This example makes that
observable by referencing `f` twice, around a redefinition of `a`:

```foolish
{
    a = 1;
    f = <<a + b>>;   !! stored: Op+(Search(a, ECONSTANIC), Search(b, ECONSTANIC)), WOCONSTANIC
    g1 = f;          !! clone strips SFFMark, resets searches, re-resolves AT g1:
                       !!   nearest a = 1, b not found → g1 = `1 + b` (WOCONSTANIC, b ECONSTANIC)
    a = 2;           !! a is now 2
    g2 = f;          !! same, re-resolved AT g2: nearest a = 2 → g2 = `2 + b` (b still ECONSTANIC)
}
```

`g1` is `1 + b` and `g2` is `2 + b` — same stored code, different `a` because each
reference re-resolves at its own position. `b` (never defined) stays ECONSTANIC in
both. The state-machine description is normative because it maps directly to the FIR
state transitions.

**Where the descriptions diverge (open caveat):** the code-template phrasing
("store the Foolish code, substitute as text") implies *everything* inside `<<...>>`
is deferred, but the precise rule is narrower-and-exact: defer the `is_search()`
FIRs. For a body containing only searches + literals these coincide. For a body with
a non-search construct that the template reading would also defer but that has
singular meaning from its text, prefer the state-machine rule. (This is why `#-1`
and `{}^` are classified as searches — see above — so the two readings line up.)

```foolish
f = <<a + b>>;
```

This means: store the expression `a + b` as unevaluated code. When `f` is
referenced later (e.g., `g = f`), substitute `a + b` into the new context and
evaluate there. Searches are generated as Search FIRs starting at ECONSTANIC —
they are not performed during the initial assignment, only when the code is
substituted and evaluated in a later context.

Compiles to: WOCONSTANIC Op+(
  Search(a, ECONSTANIC),
  Search(b, ECONSTANIC)
)

**constanic_clone of SFFMark**: When a later search finds an SFFMark FIR,
`constanic_clone` strips the SFFMark wrapper and clones only the inner content:
`constanic_clone(SFFMark(INSIDE))` → `constanic_clone(INSIDE)`.

**No internal cloning**: Since SFF does not perform searches, there are no search
results to constanic_clone inside an SFF context. The sfcc flag is not used.

**SFF and anchored search**: SFF suppresses search execution, but anchored access
(`a.result`) still works — it retrieves the brane entry without performing a
search. The entry's value may be a Search FIR in ECONSTANIC state (if the SFF
content had unresolved references). Example:

```foolish
{
    a = <<{result=not_found}>>;  !! SFF: brane with entry, not_found is ECONSTANIC search
    b = a.result;                !! Anchored access: finds entry, b gets the Search FIR
    c = a.not_there;             !! Anchored access: entry not found → NK
}
```

Here `a.result` retrieves the entry `result` which contains a Search FIR for
`not_found` in ECONSTANIC state. `b` becomes WOCONSTANIC (waiting on the search).
`a.not_there` produces NK (entry doesn't exist in the brane).

### SF (`<...>`) — Preserve Search State

SF performs searches normally (through EMBRYONIC/BRANING). The key behavior is
how constanic_clone handles the search results inside an SF context.

**Conceptual description**: SF Mark lets us refer to code from elsewhere (permit
one level of search) and combine pieces, while keeping their *naivety* — but only
**while they remain sealed inside an SF marker**. What a sealed piece didn't know
before, it still doesn't know, even if the new context could provide for some of its
ECONSTANIC searches. The naivety is **conditional**, not absolute: a piece stays
naive as long as it is reused through another SF marker (`<...>`); a plain bare
reference re-resolves it normally against wherever it lands.

Imagine a Foxconn factory on the moon that gets an order to assemble a creature:

```foolish
{ foxconn_moon = <{head = arctic.snow_ball, body = california.redwood, legs = volcano.lava}> }
!! Illustrative only — arctic / california / volcano are evocative, not defined branes.
```

The factory *finds* these parts and describes what it would do with them, but the SF
marker says: don't let them touch the moon's context yet. If the parts actually
arrived into the factory's environment they would change irreversibly. They stay
naive **as long as they're held inside the SF crate**. Only when you later
*coordinate from* `foxconn_moon` in some real context does the thing materialize
there and the parts resolve against that environment. Keep shipping it sealed in an
SF crate (`<foxconn_moon>`) and it stays naive; unwrap it with a bare reference
(`x = foxconn_moon`) and it adapts to wherever it is.

Operationally: searches inside `<...>` happen normally at assembly. When the SF
body's results are cloned into the SFMark (sfcc=True), ECONSTANIC/WOCONSTANIC are
preserved. When the assembled value is later used **bare** (non-SF context), that
clone is normal (sfcc=False) and the searches reset and resolve.

**Two constanic_clone directions (the asymmetry IS the mechanism):**

**Concern 2 — ASSEMBLE (sfcc=True).** RHS is the SF marker, e.g. `a = <b>`. The
search for `b` runs; its result is `constanic_clone`d **with sfcc=True** into the
SFMark's result field. sfcc=True is passed recursively to all nested clones:
- ECONSTANIC stays ECONSTANIC (instead of resetting to EMBRYONIC)
- WOCONSTANIC stays WOCONSTANIC (instead of resetting to BRANING)
- CONSTANT stays CONSTANT (unchanged either way)

**Concern 1 — CONSUME (sfcc=False, normal).** A later bare reference finds an
SFMark, e.g. `c = a` where `a` was `<b>`. `constanic_clone` strips the SFMark
wrapper and clones the inner content with a **normal (sfcc=False)** clone:
`constanic_clone(SFMark(INSIDE))` → `constanic_clone(INSIDE)` — so the searches
reset and re-resolve in `c`'s context. (Stripping the wrapper applies whether the
consuming clone is normal or sfcc; what differs is the reset.)

This asymmetry is why `aa = a` re-resolves while `aaa = <a>` preserves (Example 7).
The sealed-crate naivety holds only along the sfcc=True path.

**SFFMark inside SF**: When SF contains a nested SFFMark (`<...<<...>>...>`),
the SFFMark's searches start at ECONSTANIC directly. When SF's constanic_clone
(with sfcc=True) encounters these, ECONSTANIC is preserved.

### Examples

#### Example 1: Basic SFF

```foolish
{
    a = 1; b = 2;
    f = <<a + b>>;   !! WOCONSTANIC Op+(Search(a, ECONSTANIC), Search(b, ECONSTANIC))
    g = f;           !! Normal clone: strips SFFMark, resets searches, finds a=1, b=2 → CONSTANT(3)
}
```

`f` is WOCONSTANIC because SFF suppresses search — `a` and `b` start at
ECONSTANIC but are not looked up. When `g = f` clones `f`, the SFFMark is
stripped, searches are reset, and they find `a=1` and `b=2` in the new context.
`g` evaluates to `Int(3)`.

#### Example 2: Basic SF

```foolish
{
    a = 1; b = 2;
    f = <a + b>;     !! CONSTANT(3) — searches find a and b immediately
    g = f;           !! Normal clone: f is already CONSTANT, g = 3
}
```

`f` is CONSTANT because SF performs searches normally — `a` and `b` are both
available. No sfcc behavior needed since everything is already resolved.

#### Example 3: SF with unresolved references

```foolish
{
    a = 1;
    f = a * b;       !! WOCONSTANIC — b not found
    b = 0;
    g = <f>;         !! sfcc=True clone: a stays CONSTANT(1), b stays ECONSTANIC
    r = g;           !! Normal clone: b now finds 0, r = 0
}
```

`f = a * b`: `a` resolves to 1 (CONSTANT), `b` not found (ECONSTANIC search).
`g = <f>`: SFMark strips, clone with sfcc=True. `a` stays CONSTANT(1), `b` stays
ECONSTANIC (sfcc preserves it). `g` is WOCONSTANIC. `r = g`: normal clone, `b`
searches and finds `b=0`. `r = 0`.

#### Example 4: SFF then SF

```foolish
{
    f = <<z + zz + zzz>>;   !! WOCONSTANIC, all three searches ECONSTANIC
    g = <f>;                 !! Strips SFFMark, clones with sfcc=True. z,zz,zzz stay ECONSTANIC
    h = <<f>>;               !! SFF: search for f starts at ECONSTANIC directly (not performed)
}
```

`f` is WOCONSTANIC with three ECONSTANIC searches (SFF). `g = <f>`: SFMark
strips the SFFMark, clones inner with sfcc=True — all three searches stay
ECONSTANIC. `h = <<f>>`: SFFMark, so the search for `f` is not performed — it
starts at ECONSTANIC directly.

#### Example 5: SF with resolved and unresolved

```foolish
{
    a = 1;
    f = a + b + c;   !! a=CONSTANT(1), b=ECONSTANIC, c=ECONSTANIC
    b = 1; c = 1;
    g = <f>;         !! sfcc=True: a stays 1, b stays ECONSTANIC, c stays ECONSTANIC
}
```

`f` has `a` resolved but `b` and `c` not yet found. `g = <f>` clones with
sfcc=True — `a` stays CONSTANT, `b` and `c` stay ECONSTANIC (sfcc preserves
them even though `b=1` and `c=1` are now defined).

#### Example 6: SFF nested inside SF

```foolish
{
    b = 1, c = 2; d = 3;
    a = <<b + <c> + d>>;   !! SFF: b=ECONSTANIC, <c>=ECONSTANIC (SFF dominates), d=ECONSTANIC
    aa = a;                 !! Normal clone: strips SFFMark, searches reset, find b=1, c=2, d=3 → CONSTANT(6)
}
```

`a` is SFF — all searches start at ECONSTANIC directly, including `<c>` (SFF
dominates over nested SF). `aa = a` strips the SFFMark, resets searches, and
they find all values. `aa` = `Int(6)`.

#### Example 7: SF with SFF inside, sfcc preservation

```foolish
{
    b = 1, c = 2; d = f;
    a = <b + <<c>> + d>;   !! SF: b=CONSTANT(1), <<c>>=ECONSTANIC (SFF inside SF), d=WOCONSTANIC (f not found)
    aa = a;                 !! Strips SFMark, clone: b=1, c finds 2, d still WOCONSTANIC
    f = 1;
    aaa = <a>;              !! sfcc=True: d was WOCONSTANIC → stays WOCONSTANIC (sfcc preserves)
    aaaa = a;               !! Normal clone: d finds f=1, everything resolves → CONSTANT
}
```

`a` is SF — `b` found (CONSTANT), `<<c>>` is ECONSTANIC (SFF inside SF, no
search), `d` searches for `f` → not found (WOCONSTANIC). `aa = a`: strips
SFMark, normal clone — `b` stays 1, `c` finds 2, `d` still can't find `f`.
`aaa = <a>`: sfcc=True — `d` was WOCONSTANIC, stays WOCONSTANIC (sfcc preserves
it even though `f=1` is now defined). `aaaa = a`: normal clone — `d` searches,
finds `f=1`, everything resolves to CONSTANT.

---

## Implementation Approach

The phase ordering, checkboxes, and per-task notes live in the plan file
`docs/foop/FOOP-52.plan.md`. The spine: **Task 0** organize AGENTS.md → **Phase 1**
scope/search rework — `Weak` parents + the three recursive brane search methods,
keep `Rc<RefCell>` children (gate: 64 snapshots byte-identical) → **Phases 2+**
repair the 15 WIP files, one bug group per phase. This section records *how each bug
group is fixed conceptually*; the plan records *the steps*.

### Backward search / source order (Bugs 1.1–1.3, 2.3, 6.1–6.2)

The positional backward-search `Scope` (Phase 1) fixes these at the root. A search
sees only earlier siblings of the immediate brane, then delegates to the parent
(bounded by the parent's position). Forward references are out of range → stay
ECONSTANIC; nearest-earlier wins → correct shadowing (Bug 2.3). Bugs 6.1–6.2 are the
same root cause via a different symptom: the old scope handed seeks RESET/EMBRYONIC
bodies, so `constanic_clone` hit NYE. With the scope reading stepped earlier
siblings, the seek never sees NYE — the invariant violation is unreachable, no
`permit_nye` hack. (Bug 15, negative-seek out-of-bounds, is a related `index_in_brane`
clamping fix — see the plan, Phase 2.)

### Scope boundary correctness (Bug 2.1)

The OPPOSITE symptom of Bugs 1.x (too restrictive vs too permissive) — the SAME
mechanism. Parent delegation bounded by the parent's `stmt_idx` lets a name defined
BEFORE the nested brane resolve, while still excluding forward references.

### Search-tree resolution (Bugs 2.2, 3.2, 4.1)

Rule: collapse what's resolved, preserve what's not. When a search resolves to
CONSTANT, the substituted expression uses the value (not the search tree) → no
spurious inner search leaks into an outer AST (Bug 2.2). When a search is unresolved,
it is preserved through concatenation (Bug 3.2). Bug 4.1 (operator stays WOCONSTANIC
when its operand is already resolved) likely improves for free once searches return
inlined owned values rather than Rc-wrapped trees — verify before adding logic.

### Concatenation precedence (Bug 3.1)

`a b` must parse/evaluate as concatenation of two operands, not `a` searched by `b`.
This is the one bug group likely to require **parser** changes (precedence rules).

### SF/SFF markers (Bugs 5.1–5.3)

Per the [SF/SFF Marker Specification](#sfsff-marker-specification-ubc-specific):
- **SFF**: mark ALL `is_search()` FIRs ECONSTANIC at creation (Search, `#-1`, `{}^`).
- **SF**: assemble-time clone of the body's search results uses sfcc=True (preserve);
  a later bare reference that finds an SFMark uses a normal clone (re-resolve).
- Both markers are transparent to `constanic_clone` (wrapper stripped, inner cloned).

---

## Test Plan

- 15 input `.foo` files are marked with `!!! WIP FOOP-52 !!!`
  (includes `anchored_seek_negative_boundary.foo` from FOOP-32 — when complete,
  both FOOP-32 and FOOP-52 shall be updated to note its completion)
- All 64 existing `.snap` files must continue to pass — no regressions
- After fixes, run `cargo insta test -p foolish-core --lib` to regenerate
  `.snap.new` files for the 15 WIP inputs
- Review all 15 regenerated snapshots for correctness
- The goal is to fix the WIP input `.foo` files so they produce correct output,
  then promote them to `.snap`

## Rejected Alternatives

### A. Fix all bugs in one pass

Description: Address all 15 bugs simultaneously without phasing.

Reason for rejection: The bugs have dependencies (e.g., source-order fix affects
scope resolution). Phasing reduces risk of introducing new bugs and makes each
change independently testable.

### C. Owned `Box<Fir>` bodies (drop `Rc`/`RefCell`)

Description: replace `Rc<RefCell<Fir>>` children with owned `Box<Fir>`, so
`step(&mut self, ...)` self-mutates a uniquely-owned subtree, using `split_at_mut`
for disjoint sibling access.

Reason for rejection: the FIR is **not** uniquely owned. Resolved searches hold a
read-only reference to immutable CONSTANT/INDEPENDENT nodes — shared, not copied
(`constanic_clone` returns `Rc::clone`, `ubc.rs:467`) — and every FIR needs a
*readable parent pointer*. Unique `Box` ownership fits neither: it would force
deep-copying immutable nodes (wrong — changes identity/cost) or `unsafe`
self-referential raw parent pointers (fragile). `Rc<RefCell>` children + `Weak`
parents is the idiomatic, safe model — and the smaller change, since the code is
already on it. (An earlier draft of this FOOP wrongly concluded the FIR was a pure
tree and proposed this; corrected during plan review.)

### D. Lifetime-borrowed `Scope<'a>` over the live brane

Description: a `Scope<'a>` holding `&'a` borrows into the brane being evaluated, with
backward-search iterators.

Reason for rejection: it does not compile against this codebase — a scope borrowing
the brane's statements needs that brane simultaneously immutably borrowed (scope)
and mutably borrowed (write-back), and `RefCell` only turns the compile error into a
runtime panic. The chosen design avoids a separate `Scope` object entirely: search is
recursive methods on the brane that read its own statements and walk the `Weak`
parent chain. See
[Architecture](#architecture-parent-linked-fir-graph--recursive-search).

### B. Skip SFF/SF formalization

Description: Leave SFF/SF marker behavior underspecified and only fix the
immediate EMBRYONIC issue.

Reason for rejection: Without formal specification, future changes may
reintroduce the same bugs. The distinction between SF and SFF is fundamental to
the language.

## Open Questions

- **Source-order check at parse time or eval time?** Resolved: evaluation time, via
  the positional backward-search `Scope` (the search only sees earlier siblings). No
  AST annotation needed.
- **`constanic_clone` on NYE — allowed?** Resolved: never. It is always an error
  (INVARIANT-VIOLATED). The owned-body rewrite makes it unreachable for seeks
  (`Scope::stmt_at_offset` returns only stepped earlier siblings). No `permit_nye`
  escape hatch needed.
- **Exact precedence of search vs concatenation vs other operators?** Still open —
  resolved during Phase 5 (concatenation precedence, Bug 3.1); may require parser
  changes. This is the one genuinely open design question.

## Code Quality (folded into Phase 1)

These are not separate cleanups — the rewrite touches these call sites, so they are
done as part of Phase 1:

- **`is_search()` predicate on the FIR trait.** A search is any FIR that consults the
  surrounding brane: name **Search**, positional **Index/seek** (`#-1`), **HeadTail**
  (`{}^`). `is_search()` defaults to `false` and is overridden `true` on those three.
  Its doc comment carries the invariant ("only searches are indeterminate once the
  text is composed"). This is the canonical home for the classification and is what
  SFF suppression and `has_unresolved_forward_refs` key off — fixing the latent gap
  where `has_unresolved_forward_refs` (`ubc.rs:160`) ignored Index/HeadTail.
- **`fir_variant() -> &'static str` → `kind() -> FirKind`.** String comparisons like
  `fir_variant() == "StayFullyFoolish"` become enum/match dispatch. A local `Variant`
  enum already exists (`ubc.rs:359`) — promote it to a shared `FirKind`. Keep `kind()`
  (dispatch) distinct from `is_search()` (classification predicate); they have
  different homes by design.
- **Drop dead fields:** `SearchFir.parent` (set, never read) and
  `NormalBraneFir.parent` (never set or read).
- **Encapsulation:** free functions in `ubc.rs` that reach into FIRs
  (`re_step_brane_bodies`, `reset_searches`, `step_except_brane_one`,
  `strip_sf_wrapper`) move onto the types per the AGENTS.md encapsulation rule
  (Task 0). `reset_searches` is removed entirely (positional search makes it moot).

## References

- Prior FOOPs: FOOP-32 (first batch of snapshot bugs)
- Code locations: `foolish-core/src/` (FVM evaluation), `foolish-core/src/ubc_snapshot_tester.rs`
- Bug catalog: `docs/foop/FOOP-52.bugs.md`

## Last Updated

**Date**: 2026-06-07 (later, correction)
**Updated By**: Claude Code 2.1.119 (Claude Code); Sonnet 4.6
**Changes**: CORRECTED the architecture — the prior entry's "FIR is a tree → owned
`Box<Fir>` mechanical rewrite" conclusion was WRONG. The FIR is a parent-linked graph:
CONSTANT/INDEPENDENT nodes are shared by read-only reference (`constanic_clone` returns
`Rc::clone`, `ubc.rs:467`), and every FIR has a readable parent back-pointer. Settled
model: `Rc<RefCell<Fir>>` children + `Weak<RefCell<Fir>>` parents, mutate-self;
`Box<Fir>` rejected. Search is now three recursive methods ON the brane
(`iterate_immediate_brane` / `search_immediate_brane` / `search_ancestral_branes` +
`line_of_child`), written out normatively, sharing one iterator for anchored AND
unanchored search; the flat `Scope` is retired. Added a REQUIRED unit-test list for
those methods (flat + nested branes), in addition to snapshot tests. Updated abstract,
Architecture section, plan Phase 1, rejected alternatives C (now = Box) and D, and the
worktree branch (`scope-search-rework-foop-52`). Added the "Foolish Semantic
Immutability vs FIR Evaluation State" principle to AGENTS.md.

**Date**: 2026-06-07
**Updated By**: Claude Code 2.1.119 (Claude Code); Sonnet 4.6
**Changes**: Reframed as Major. Added §"Exception to the 'no failing
tests' rule" — FOOP-52 is explicitly excepted from AGENTS.md's no-broken-tests rule
because the 15 WIP failures ARE the work (green oracle = `cargo insta test`); the
exception is narrow (new breakage still halts). [Note: this entry's "owned `Box<Fir>`"
architecture was superseded by the correction above.] Corrected the SF/SFF spec (review Findings A–E):
SFF suppresses ALL `is_search()` FIRs — Search, Index/`#-1`, HeadTail/`{}^` — per
the "only searches are indeterminate" invariant; replaced the vacuous SFF example
with the g1/g2 re-resolution example; made SF naivety conditional on SF-wrapped
reuse and fixed the Foxconn metaphor (`arctic.snow_ball`, illustrative-only); made
the SF Concern 1 (consume/normal) vs Concern 2 (assemble/sfcc=True) asymmetry
explicit; added NYE → INVARIANT-VIOLATED rows to the constanic_clone table; fixed
"SF/SF" → "SF/SFF". Folded in `is_search()` trait predicate, `fir_variant()` →
`FirKind`, dead-field removal. Normalized bug count to 15. Implementation Approach
now defers phase ordering to the plan file. Added rejected alternatives C
(patch-in-place) and D (lifetime-borrowed Scope).

**Date**: 2026-06-06
**Updated By**: opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Added conceptual description of SF Mark (arctic/volcanic creature
metaphor) — SFMark isolates code from current environment, assembled pieces
maintain naivety, only resolve when used in normal context.

**Date**: 2026-06-06
**Updated By**: opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Initial creation — documented 14 bugs from snapshot review round 2
