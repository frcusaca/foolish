---
foop: 25
title: Repair FVM evaluation bugs found in snapshot review round 2
author: opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4
status: Draft
type: Bugfix
created: 2026-06-06
phase: phase-2
supersedes: []
---

# FOOP-52: Repair FVM evaluation bugs found in snapshot review round 2

## Abstract

Fifteen bugs discovered during human review of `.snap.new` files in
`foolish-core/snapshot_tests/approved/`. These are the second batch of bugs
(after FOOP-32's eight bugs). They fall into six categories: forward reference
resolution, scope resolution across brane boundaries, search/concatenation
precedence, operator transparency, SFF/SF marker semantics, and unanchored seek
invariant violations. Includes Bug 15 (boundary clamping) from FOOP-32.

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
[SFF/SF Marker Specification](#sfsf-marker-specification-ubc-specific) section
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
specification is now defined in [SF/SF Marker Specification](#sfsf-marker-specification-ubc-specific).
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

## Scope Refactoring (Top Priority)

The current `Scope` implementation is an accumulating struct that clones for
children. This must be refactored before any other Phase 1 work.

### Current Design (to be replaced)

```rust
// ubc.rs:37-45
struct Scope {
    entries: Vec<(String, FirRef)>,  // accumulating flat list
    current_brane: Option<FirRef>,   // stale reference to reset brane
    current_stmt_idx: Option<usize>,
    block_brane_searches: bool,
    parent: Option<Box<Scope>>,      // cloned parent
    // ...
}
```

Problems:
- `entries` accumulates all names (including forward references)
- Cloning for children is expensive and creates stale references
- `current_brane` points to reset (EMBRYONIC) bodies, not evaluated ones
- Flat search doesn't respect positional backward search

### Proposed Design

Scope is a lightweight wrapper around a Brane reference. It holds the current
position (statement index) and a reference to the parent scope. No cloning
needed for children — just pass references.

```rust
struct Scope<'a> {
    brane: &'a NormalBraneFir,       // reference to the brane being evaluated
    stmt_idx: usize,                  // current statement position
    parent: Option<&'a Scope<'a>>,    // parent scope (for upward search)
    stmts: &'a [StatementFir],        // iterable slice of brane statements
}
```

### Backward Search Iterator

Each scope creates iterators that walk backward through brane statements:

**`ib_stmts`** (Immediate Brane statements): Iterates statements of the current
brane from `(stmt_idx - 1)` to `0` inclusive. This is the local backward search —
only the current brane, starting from the statement before the current one.

**`abib_stmts`** (Ancestral Brane + Immediate Brane statements): Iterates
`ib_stmts` first, then delegates to `parent.abib_stmts`. Termination condition is
when `parent` is `None` (no more ancestor branes). This is the full backward
search chain — local brane first, then parent branes all the way up.

```rust
struct IbStmtsIterator<'a> {
    stmts: &'a [StatementFir],
    current_idx: usize,  // starts at stmt_idx - 1, decrements to 0
}

impl<'a> Iterator for IbStmtsIterator<'a> {
    type Item = (&'a str, &'a FirRef);
    
    fn next(&mut self) -> Option<Self::Item> {
        while self.current_idx > 0 {
            self.current_idx -= 1;
            let stmt = &self.stmts[self.current_idx];
            if let Some(ref name) = stmt.name {
                return Some((name, &stmt.body));
            }
        }
        None
    }
}

struct AbibStmtsIterator<'a> {
    scope: &'a Scope<'a>,
    ib_iter: IbStmtsIterator<'a>,  // current ib iteration
    parent_exhausted: bool,
}

impl<'a> Iterator for AbibStmtsIterator<'a> {
    type Item = (&'a str, &'a FirRef);
    
    fn next(&mut self) -> Option<Self::Item> {
        // Try current ib_iter first
        if let Some(item) = self.ib_iter.next() {
            return Some(item);
        }
        // Current brane exhausted — delegate to parent
        if self.parent_exhausted {
            return None;
        }
        let parent = match self.scope.parent {
            Some(p) => p,
            None => return None,  // termination: no parent
        };
        // Start parent's ib iteration from ITS stmt_idx
        self.scope = parent;
        self.ib_iter = IbStmtsIterator {
            stmts: parent.stmts,
            current_idx: parent.stmt_idx,
        };
        self.parent_exhausted = parent.parent.is_none();
        self.ib_iter.next()
    }
}
```

### Search API

```rust
impl<'a> Scope<'a> {
    /// Search backward through immediate brane only (ib_stmts).
    fn search_local(&self, pattern: &str) -> Option<&'a FirRef> {
        let re = regex::Regex::new(pattern).ok()?;
        for (name, body) in self.ib_stmts() {
            if re.is_match(name) {
                return Some(body);
            }
        }
        None
    }
    
    /// Search backward through all ancestor branes (abib_stmts).
    fn search(&self, pattern: &str) -> Option<&'a FirRef> {
        let re = regex::Regex::new(pattern).ok()?;
        for (name, body) in self.abib_stmts() {
            if re.is_match(name) {
                return Some(body);
            }
        }
        None
    }
    
    /// Create a child scope for a nested brane evaluation.
    /// No cloning — just a reference to the child brane and its parent.
    fn child(&'a self, brane: &'a NormalBraneFir, stmt_idx: usize) -> Scope<'a> {
        Scope {
            brane,
            stmt_idx,
            parent: Some(self),
            stmts: &brane.statements,
        }
    }
    
    /// Immediate Brane iterator: backward from stmt_idx-1 to 0.
    fn ib_stmts(&'a self) -> IbStmtsIterator<'a> {
        IbStmtsIterator {
            stmts: self.stmts,
            current_idx: self.stmt_idx,
        }
    }
    
    /// Ancestral + Immediate Brane iterator: ib_stmts then parent.abib_stmts.
    fn abib_stmts(&'a self) -> AbibStmtsIterator<'a> {
        AbibStmtsIterator {
            scope: self,
            ib_iter: self.ib_stmts(),
            parent_exhausted: self.parent.is_none(),
        }
    }
    
    /// Get statement at offset from current position (for unanchored seek).
    fn stmt_at_offset(&self, offset: i32) -> Option<&'a FirRef> {
        let target = self.stmt_idx as i32 + offset;
        if target < 0 || target >= self.stmts.len() as i32 {
            return None;
        }
        Some(&self.stmts[target as usize].body)
    }
}
```

### Impact on braning_step

The current `braning_step` (ubc.rs:216) clones the scope and pushes all
names. With the new design, `reset_searches` is no longer needed:

1. **Forward reference prevention**: The scope's backward search from current
   position naturally prevents forward references — no reset needed
2. **No stale references**: The scope points to evaluated bodies, not reset ones
3. **`constanic_clone` handles resets**: When a brane is used in a new context,
   `constanic_clone` already resets ECONSTANIC→EMBRYONIC and WOCONSTANIC→BRANING

```rust
fn braning_step(brane: &mut NormalBraneFir, parent_scope: &Scope) {
    // No scope cloning needed
    // No reset_searches needed — scope handles forward reference prevention
    for (idx, stmt) in brane.statements.iter().enumerate() {
        let scope = parent_scope.child(brane, idx);
        let body = step_boxed(&stmt.body, &scope)?;
        // ... update statement body
    }
}
```

`reset_searches` (ubc.rs:260) should be removed entirely.

### Key Properties

- **No cloning**: Scope holds references, not owned data
- **No stale references**: `brane` points to the actual brane being evaluated
  (which contains evaluated bodies as stepping progresses)
- **Positional backward search**: Iterator starts at `stmt_idx` and goes backward
- **Parent delegation**: When local brane is exhausted, search continues in parent
  from the position where the brane was defined
- **Depth-first compatible**: Parent's `stmt_idx` IS the statement currently being
  evaluated — no ambiguity
- **Nested scoping**: Iteration through a brane constructs small scope objects for
  each line, recursively passed into the next level of evaluation. Each statement
  gets its own scope pointing to the brane at its position.

### Implementation Order

This refactoring is Phase 0 — it must be done before any other Phase 1 work.
All other phases depend on the scope working correctly.

- [ ] Define `Scope<'a>` struct with brane reference, stmt_idx, parent
- [ ] Implement `BackwardSearchIterator` with parent delegation
- [ ] Implement `Scope::search()` using the iterator
- [ ] Implement `Scope::child()` for nested brane evaluation
- [ ] Refactor `braning_step` to use new scope
- [ ] Remove `reset_searches` (ubc.rs:260) — no longer needed
- [ ] Remove old `Scope` struct and `entries` accumulation
- [ ] Verify all existing tests pass with new scope

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

## SF/SF Marker Specification (UBC-specific)

This section defines the operational semantics of SF (`<...>`) and SFF (`<<...>>`)
markers as implemented in the UBC (Unicellular Brane Computer). These markers
control how searches are initialized and how constanic_clone behaves during
evaluation.

### constanic_clone State Transitions

The base constanic_clone behavior (without markers):

| Source State | Result State |
| ------------ | ------------ |
| CONSTANT     | CONSTANT     |
| ECONSTANIC   | EMBRYONIC    |
| WOCONSTANIC  | BRANING      |

With SF marker context (`sfcc=True`):

| Source State | Result State |
| ------------ | ------------ |
| CONSTANT     | CONSTANT     |
| ECONSTANIC   | ECONSTANIC   |
| WOCONSTANIC  | WOCONSTANIC  |

SFF has no constanic_clone table because SFF does not perform searches — there
are no search results to clone.

### SFF (`<<...>>`) — Suppress Search / Code Template

SFF suppresses search execution. ALL searches inside `<<...>>` are generated as
Search FIRs but skip EMBRYONIC/BRANING — they enter ECONSTANIC directly. This
includes searches that would normally become NK (e.g., `{}^` — seek beyond brane
bounds). Inside SFF, these also start at ECONSTANIC. All other evaluation
(operators on literals, etc.) proceeds normally.

**Normative description (state machine)**: SFF suppresses search execution.
Searches inside `<<...>>` are generated as Search FIRs but skip EMBRYONIC/BRANING
— they enter ECONSTANIC directly. This is the implementation model.

**Alternate description (code template)**: When the RHS of an assignment is
completely enclosed in SFF markers, the meaning is that when the LHS identifier is
referred to later, the LHS symbol is replaced with the Foolish code within the
`<<...>>` markers. The SFF content is stored unevaluated and substituted as-is
when referenced.

These two descriptions are equivalent for simple cases. They diverge when SFF
content references names that are in scope at definition time:

```foolish
{
    a = 1;
    f = <<a + b>>;   !! State machine: a=ECONSTANIC, b=ECONSTANIC (both searches)
                       !! Code template: stores `a + b` unevaluated
    a = 2;           !! a is now 2
    g = f;           !! State machine: searches reset, find a=2, b=? → WOCONSTANIC or CONSTANT
                       !! Code template: substitute `a + b`, find a=2, b=? → same result
}
```

In this case both descriptions produce the same result because the searches are
reset when `f` is cloned to `g`. The state machine description is normative
because it maps directly to the FIR state transitions.

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

**Conceptual description**: SF Mark allows us to refer to code from elsewhere
(permit one level of search), but when the pieces are stitched together, their
naivety is maintained. What they didn't know before, they still don't know — even
if the new context might provide for some of their ECONSTANIC searches. SFMark
lets us find and combine code without being affected by the current environment.

Imagine assembling a creature: head from arctic snowball, foot from volcanic lava.
SFMark lets you find them and assemble in a factory in Foxconn. The assembled
creature doesn't know about Foxconn's environment. Only when you call on this
creature from, say, a moonbase, do the pieces try to find what they need.

Operationally: searches inside `<...>` happen normally. When the SFMark's result
is cloned (constanic_clone with sfcc=True), ECONSTANIC and WOCONSTANIC states are
preserved — the searches don't learn from the new context. Only when the assembled
result is later used in a normal (non-SF) context do the searches reset and
resolve against whatever is available.

**Two distinct constanic_clone concerns:**

**Concern 1 — Cloning an SFMark FIR itself**: When a later search finds an
SFMark, `constanic_clone` strips the SFMark wrapper and clones only the inner
content: `constanic_clone(SFMark(INSIDE))` → `constanic_clone(INSIDE)`.

**Concern 2 — Cloning used BY SFMark**: When SFMark performs searches inside
`<...>`, those search results need to be constanic_cloned into the SFMark's
location as children. These clones use `sfcc=True`, which is passed recursively
to all nested constanic_clones. With `sfcc=True`:
- ECONSTANIC stays ECONSTANIC (instead of resetting to EMBRYONIC)
- WOCONSTANIC stays WOCONSTANIC (instead of resetting to BRANING)
- CONSTANT stays CONSTANT (unchanged either way)

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

### Phase 1: Source-Order Resolution / Backward Search (Bugs 1.1–1.3, 2.3, 6.1–6.2)

Implement proper backward search: each statement loops backward from its position
in the brane's statement array. If not found, ask the parent brane — the parent
knows where the search came from because depth-first evaluation means the parent's
statement currently being evaluated IS the starting position.

Also resolves Bugs 6.1–6.2 (invariant violations) — when the scope's brane
references evaluated bodies, `constanic_clone` will never see NYE.

### Phase 2: Scope Boundary Correctness (Bug 2.1)

Fix the search to correctly cross parent brane boundaries when the identifier IS
defined before the nested brane in source order. This requires careful interaction
with Phase 1 — the search must check both source order AND brane depth.

### Phase 3: Search Tree Resolution (Bugs 2.2, 3.2, 4.1)

When a search resolves to a CONSTANT value, collapse the search tree in the
substituted expression. When a search is unresolved (ECONSTANIC/WOCONSTANIC),
preserve it through operations like concatenation. The rule: collapse what's
resolved, preserve what's not.
- Spurious inner searches from leaking into outer expressions (Bug 2.2)
- Unresolved searches from being dropped during concatenation (Bug 3.2)
- Resolved operators from staying WOCONSTANIC (Bug 4.1)

### Phase 4: Concatenation Precedence (Bug 3.1)

Fix the parser/evaluator to correctly handle `a b` as concatenation of two
operands, not as `a` followed by `b` as a search on `a`. This may require parser
changes to the precedence rules.

### Phase 5: SFF/SF Marker Implementation (Bugs 5.1–5.3)

Implement the SF/SF Marker Specification (UBC-specific):

**SFF (`<<...>>`)**:
- Searches inside SFF skip EMBRYONIC/BRANING — start at ECONSTANIC directly
- SFFMark is transparent to constanic_clone: strips wrapper, clones inner content
- No internal cloning (SFF doesn't perform searches)

**SF (`<...>`)**:
- Searches happen normally (through EMBRYONIC/BRANING)
- SFMark is transparent to constanic_clone: strips wrapper, clones inner content
- When SFMark clones search results into its location (Concern 2), use sfcc=True:
  ECONSTANIC stays ECONSTANIC, WOCONSTANIC stays WOCONSTANIC
- sfcc=True is passed recursively to all nested constanic_clones

**Implementation steps:**
- [ ] Modify search initialization: inside SFF, set search state to ECONSTANIC
      (not EMBRYONIC)
- [ ] Modify constanic_clone: when sfcc=True, preserve ECONSTANIC/WOCONSTANIC
      states (instead of resetting to EMBRYONIC/BRANING)
- [ ] Modify constanic_clone: strip SFMark/SFFMark wrappers (clone inner content)
- [ ] Pass sfcc=True recursively when cloning inside SF context
- [ ] Test Bug 5.1: `{a=1, b=2; inner={c=<<a+b>>; c}; inner;}` — searches should
      be ECONSTANIC, not EMBRYONIC
- [ ] Test Bug 5.2: `{x=5; y=10; inner={calc=<<x+y>>; doubled=calc*2};}` — same
      ECONSTANIC requirement
- [ ] Test Bug 5.3: `{x=10; y=<x>; z=y+5;}` — verify SF behavior is correct
- [ ] Verify no regressions in 64 approved snapshots

### Phase 6: Verification (Bugs 6.1–6.2)

Confirm that Phase 1's backward search fix resolves the invariant violations.
`constanic_clone` should NEVER encounter NYE — the invariant is correct, the bug
was the stale scope. No additional safety check is needed.

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

Description: Address all 14 bugs simultaneously without phasing.

Reason for rejection: The bugs have dependencies (e.g., source-order fix affects
scope resolution). Phasing reduces risk of introducing new bugs and makes each
change independently testable.

### B. Skip SFF/SF formalization

Description: Leave SFF/SF marker behavior underspecified and only fix the
immediate EMBRYONIC issue.

Reason for rejection: Without formal specification, future changes may
reintroduce the same bugs. The distinction between SF and SFF is fundamental to
the language.

## Open Questions

- Should the source-order check apply at parse time (AST annotation) or
  evaluation time (runtime search)?
- What is the exact precedence relationship between search, concatenation, and
  other operators?
- Should `constanic_clone` be allowed on NYE FIRs in any context, or is it
  always an error?

## Code Quality Note: fir_variant String Typing

The `fir_variant()` method on FIR types currently returns `&'static str` (string
comparisons like `fir_variant() == "StayFullyFoolish"`). This should be refactored
to return an enum for type safety and exhaustiveness checking. The `Variant` enum
already exists in `ubc.rs` (line 358) but is local — it should be promoted to a
shared type and used as the return type of `fir_variant()`. This is not a bug fix
but a code quality improvement that should be done as part of Phase 5
implementation.

## References

- Prior FOOPs: FOOP-32 (first batch of snapshot bugs)
- Code locations: `foolish-core/src/` (FVM evaluation), `foolish-core/src/ubc_snapshot_tester.rs`
- Bug catalog: `docs/foop/FOOP-52.bugs.md`

## Last Updated

**Date**: 2026-06-06
**Updated By**: opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Added `reset_searches` removal analysis (scope handles forward
reference prevention, `constanic_clone` handles NYES transitions). Added
Semantic Immutability Principle section. Added `ib_stmts` and `abib_stmts`
iterators to Scope specification. Added `Scope::stmt_at_offset` for unanchored
seek. Updated `braning_step` to not use `reset_searches`. Added `stmts: &'a
[StatementFir]` field to Scope struct. Restored `anchored_seek_negative_boundary`
as Bug 15 (FOOP-32 bug being fixed in FOOP-52). Updated bug count to 15.

**Date**: 2026-06-06
**Updated By**: opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Added conceptual description of SF Mark (arctic/volcanic creature
metaphor) — SFMark isolates code from current environment, assembled pieces
maintain naivety, only resolve when used in normal context.

**Date**: 2026-06-06
**Updated By**: opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Initial creation — documented 14 bugs from snapshot review round 2
