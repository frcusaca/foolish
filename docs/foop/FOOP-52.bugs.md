# Snapshot Bug Repair List (FOOP-52)

**Created**: 2026-06-06
**Source**: `@Agent` / `@Agents` comments found in `.snap.new` files before promotion to `.snap`
**Status**: All bugs documented, fixes pending

> These 15 bugs are the **acceptance test** for the owned-FIR evaluator rewrite (see
> FOOP-52.md and FOOP-52.plan.md). They are fixed in Phases 2+, *after* Phase 1
> (the rewrite) lands with all 64 existing snapshots passing byte-identical. The 15
> WIP input files (`!!! WIP FOOP-52 !!!`) are the only bug-scope files; the other
> pending `.snap.new` files are out of scope.

---

## Summary

| # | File | Category | Severity | Group |
|---|------|----------|----------|-------|
| 1 | `forward_reference_basic` | Forward Reference | Critical | 1 |
| 2 | `forward_reference_in_nested_brane` | Forward Reference | Critical | 1 |
| 3 | `complex_forward_refs_in_nested_branes` | Forward Reference | Critical | 1 |
| 4 | `complex_full_program_with_all_features` | Scope Resolution | Critical | 2 |
| 5 | `cross_scope_reference_chain` | Scope Resolution | Critical | 2 |
| 6 | `identifier_shadowing` | Scope Resolution | Critical | 2 |
| 7 | `complex_search_and_concatenation` | Precedence | Critical | 3 |
| 8 | `concatenation_with_unresolved_search` | Precedence | Critical | 3 |
| 9 | `complex_brane_with_operations_and_search` | Operator Transparency | High | 4 |
| 10 | `complex_sff_in_nested_brane` | SFF Marker | High | 5 |
| 11 | `complex_sff_with_nested_scope` | SFF Marker | High | 5 |
| 12 | `complex_sf_in_expression` | SF Marker | Medium | 5 |
| 13 | `complex_unanchored_seeks_with_operations` | Invariant Violation | Critical | 6 |
| 14 | `nested_brane_boundary` | Invariant Violation | Critical | 6 |
| 15 | `anchored_seek_negative_boundary` | Boundary Clamping | High | 6 (from FOOP-32) |

---

## Bug 1: Forward reference resolves in same brane

**File**: `forward_reference_basic.foo.snap.new`
**Input file**: `forward_reference_basic.foo`
**Category**: Forward Reference — FVM evaluation
**Severity**: Critical

**Input:**
```foolish
{y = x; x = 42;}
```

**Current (WRONG) output:**
```
y=42;
x=42
```

**Expected:**
`y` should be a Search FIR in ECONSTANIC state. `x` is defined AFTER `y` in source order.

---

## Bug 2: Forward reference resolves across one brane boundary

**File**: `forward_reference_in_nested_brane.foo.snap.new`
**Input file**: `forward_reference_in_nested_brane.foo`
**Category**: Forward Reference — FVM evaluation
**Severity**: Critical

**Input:**
```foolish
{outer = {val = x}; x = 100;}
```

**Current (WRONG) output:**
```
outer={
    val=100
};
x=100
```

**Expected:**
`val` should be a Search FIR in ECONSTANIC state. `x` is in the parent brane, defined AFTER `outer`.

---

## Bug 3: Forward reference resolves across two brane boundaries

**File**: `complex_forward_refs_in_nested_branes.foo.snap.new`
**Input file**: `complex_forward_refs_in_nested_branes.foo`
**Category**: Forward Reference — FVM evaluation
**Severity**: Critical

**Input:**
```foolish
{nested = {inner = {val = x}}; x = 42;}
```

**Current (WRONG) output:**
```
nested={
    inner={
        val=42
    }
};
x=42
```

**Expected:**
`val` should be a Search FIR in ECONSTANIC state. `x` is defined in the outermost brane AFTER `nested`, separated by TWO brane boundaries.

---

## Bug 4: Search fails to find parent-scope identifier

**File**: `complex_full_program_with_all_features.foo.snap.new`
**Input file**: `complex_full_program_with_all_features.foo`
**Category**: Scope Resolution — FVM evaluation
**Severity**: Critical

**Input:**
```foolish
{a = 10; b = 20; sum = a + b; nested = {inner = sum / 2}; result = nested.inner;}
```

**Current (WRONG) output:**
```
sum=30;
nested={WOCONSTANIC
    inner=Op/(...WOCONSTANIC...)
};
result=?(pattern='^inner$', ANCHORED, NK)
```

**Expected:**
`sum` is `Int(30)` in parent scope. `inner = sum / 2` should resolve to `Int(15)`. `nested` should be CONSTANIC. `result` should be `Int(15)`.

---

## Bug 5: Spurious search in AST for resolved identifier

**File**: `cross_scope_reference_chain.foo.snap.new`
**Input file**: `cross_scope_reference_chain.foo`
**Category**: Scope Resolution — FVM evaluation
**Severity**: Critical

**Input:**
```foolish
{a = 1; b = {c = a + 1; d = c + 1};}
```

**Current (WRONG) output:**
```
b={WOCONSTANIC
    c=2;
    d=Op+(?(result=Op+(?(pattern='^a$', UNANCHORED), 1, WOCONSTANIC), pattern='^c$', UNANCHORED, WOCONSTANIC), 1, WOCONSTANIC)
}
```

**Expected:**
`c = a + 1` correctly resolves to `Int(2)`. `d = c + 1` should resolve to `Int(3)`. The AST for `d` should contain only a search for `c`, NOT a search for `a`.

---

## Bug 6: Identifier shadowing looks ahead

**File**: `identifier_shadowing.foo.snap.new`
**Input file**: `identifier_shadowing.foo`
**Category**: Scope Resolution — FVM evaluation
**Severity**: Critical

**Input:**
```foolish
{x = 10; x; x = 20; x;}
```

**Current (WRONG) output:**
```
x=10;
20;
x=20;
20
```

**Expected:**
Second expression `x;` should resolve to `Int(10)` (value at that point). Fourth expression `x;` should resolve to `Int(20)`.

---

## Bug 7: Concatenation loses search result operand

**File**: `complex_search_and_concatenation.foo.snap.new`
**Input file**: `complex_search_and_concatenation.foo`
**Category**: Operator Precedence — FVM evaluation / parser
**Severity**: Critical

**Input:**
```foolish
{target = {a=1; c=2; c={a=1, b=2, c=3}}; b1 = {x=10}; result_1= b1(target.c); result_2 = b1 target.c;}
```

**Current (WRONG) output:**
```
result={
    x=10
}
```

**Expected:**
`b1 target.c` should concatenate `{x=10}` with `{a=1, b=2, c=3}` to produce `{x=10; a=1; b=2; c=3}`.

---

## Bug 8: Concatenation drops unresolved search from operand

**File**: `concatenation_with_unresolved_search.foo.snap.new`
**Input file**: `concatenation_with_unresolved_search.foo`
**Category**: Operator Precedence — FVM evaluation
**Severity**: Critical

**Input:**
```foolish
{a = {x=ref}; b = {y=2}; c = a b;}
```

**Current (WRONG) output:**
```
c={
    y=2
}
```

**Expected:**
`c` should contain `x=Search(ref, ECONSTANIC)` from `a`, not drop it.

---

## Bug 9: Operator stays WOCONSTANIC when operand is resolved

**File**: `complex_brane_with_operations_and_search.foo.snap.new`
**Input file**: `complex_brane_with_operations_and_search.foo`
**Category**: Operator Transparency — FVM evaluation
**Severity**: High

**Input:**
```foolish
{x=10; y=20; z=30; sum = x + y + z; avg = sum / 3;}
```

**Current (WRONG) output:**
```
sum=60;
avg=Op/(...WOCONSTANIC...)
```

**Expected:**
`sum` is `Int(60)`. `avg = sum / 3` should resolve to `Int(20)`.

---

## Bug 10: SFF searches stay EMBRYONIC instead of ECONSTANIC

**File**: `complex_sff_in_nested_brane.foo.snap.new`
**Input file**: `complex_sff_in_nested_brane.foo`
**Category**: SFF Marker — FVM evaluation
**Severity**: High

**Input:**
```foolish
{a=1, b=2; inner = {c = <<a+b>>; c}; inner;}
```

**Current (WRONG) output:**
```
c=Op+(?(pattern='^a$', UNANCHORED), ?(pattern='^b$', UNANCHORED), EMBRYONIC);
```

**Expected:**
SFF (`<<...>>`) suppresses search — searches start at ECONSTANIC directly, not
EMBRYONIC. `c` should be:
```
c=Op+(Search(pattern='^a$', ECONSTANIC), Search(pattern='^b$', ECONSTANIC), WOCONSTANIC)
```
The operator is WOCONSTANIC because its operands are not yet resolved. When `c`
is later referenced (via `inner`), the SFFMark is stripped and searches reset to
find `a=1, b=2` in the new context.

---

## Bug 11: SFF with nested scope — same EMBRYONIC issue

**File**: `complex_sff_with_nested_scope.foo.snap.new`
**Input file**: `complex_sff_with_nested_scope.foo`
**Category**: SFF Marker — FVM evaluation
**Severity**: High

**Input:**
```foolish
{x = 5; y = 10; inner = {calc = <<x + y>>; doubled = calc * 2};}
```

**Current (WRONG) output:**
```
calc=Op+(?(pattern='^x$', UNANCHORED), ?(pattern='^y$', UNANCHORED), EMBRYONIC);
```

**Expected:**
Same as Bug 10. `calc` should have ECONSTANIC searches:
```
calc=Op+(Search(pattern='^x$', ECONSTANIC), Search(pattern='^y$', ECONSTANIC), WOCONSTANIC)
```
Note that `doubled = calc * 2` correctly evaluates to `Int(30)` — this is because
`calc`'s searches find `x=5` and `y=10` during `doubled`'s evaluation (normal
clone resets searches). But the SFF marker should have set them to ECONSTANIC
from the start.

---

## Bug 12: SF marker semantics underspecified

**File**: `complex_sf_in_expression.foo.snap.new`
**Input file**: `complex_sf_in_expression.foo`
**Category**: SF Marker — FVM evaluation
**Severity**: Medium

**Input:**
```foolish
{x=10; y=<x>; z=y + 5;}
```

**Current output:**
```
y=?(result=10, pattern='^x$', UNANCHORED);
z=15
```

**Expected:**
Current output appears correct for this simple case. SF (`<...>`) performs searches
normally — `x` is found immediately, so `y` is CONSTANT(10). When `z = y + 5`
clones `y`, it's already CONSTANT, so `z = 15`.

The SF marker semantics are now formally specified in FOOP-52 §"SF/SFF Marker
Specification". The key behavior is constanic_clone with sfcc=True: when cloning
inside an SF context, ECONSTANIC and WOCONSTANIC states are preserved (not reset
to EMBRYONIC/BRANING). This test doesn't exercise that behavior because everything
is already resolved. See FOOP-52 Examples 3, 5, and 7 for cases where sfcc
preservation matters.

---

## Bug 13: Unanchored seek with operations triggers invariant violation

**File**: `complex_unanchored_seeks_with_operations.foo.snap.new`
**Input file**: `complex_unanchored_seeks_with_operations.foo`
**Category**: Invariant Violation — FVM evaluation
**Severity**: Critical

**Input:**
```foolish
{a=10, b=20, c=30, result=#-1 + #-2, result2=#-1 * #-2, result3=#-1 - #-2;}
```

**Current (WRONG) output:**
```
result=50;
result2=Op*(??? (constanic_clone called on NYE FIR), INVARIANT-VIOLATED: ..., 30, NK);
result3=Op-(??? (constanic_clone called on NYE FIR), INVARIANT-VIOLATED: ..., NK)
```

**Expected:**
All results should be CONSTANT. No invariant violations.

---

## Bug 14: Nested brane boundary seek triggers invariant violation

**File**: `nested_brane_boundary.foo.snap.new`
**Input file**: `nested_brane_boundary.foo`
**Category**: Invariant Violation — FVM evaluation
**Severity**: Critical

**Input:**
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

**Expected:**
No invariant violations. `c` should seek to `a`, `f` should seek to `b`'s brane.

---

## Bug 15: Negative seek out-of-bounds clamps to first element (FOOP-32)

**File**: `anchored_seek_negative_boundary.foo.snap.new`
**Input file**: `anchored_seek_negative_boundary.foo`
**Category**: Boundary Clamping — FVM evaluation
**Severity**: High
**Origin**: FOOP-32 (being fixed as part of FOOP-52)

**Input:**
```foolish
{b = {10; 20; 30}; last = b#-1; second = b#1; first = b#-3; oob = b#-4;}
```

**Current (WRONG) output:**
```
oob=10     !! @Agent, there is no forth item back, `b#-4` should fail here
```

**Expected:**
`b#-4` on a 3-element brane `{10; 20; 30}` is out of bounds (indices -1, -2, -3
are valid; -4 is not). The result should be NK, not `Int(10)`.

**Root cause:** Negative seek clamps to index 0 when the offset exceeds the brane
size, instead of producing NK. The clamping logic is asymmetric between positive
and negative offsets.

---

## Notes

- 15 bugs total across 6 groups (includes Bug 15 from FOOP-32)
- Bugs 1–6 are FVM evaluation bugs (search/scope)
- Bugs 7–8 are precedence/concatenation bugs
- Bug 9 is operator transparency
- Bugs 10–12 are SFF/SF marker semantics
- Bugs 13–14 are invariant violations in unanchored seek
- Bug 15 is boundary clamping (from FOOP-32)
- 15 input files marked with `!!! WIP FOOP-52 !!!`
  (includes `anchored_seek_negative_boundary.foo` from FOOP-32 — when complete,
  both FOOP-32 and FOOP-52 shall be updated to note its completion)
- 64 approved snapshots promoted to `.snap`
- 58 unreviewed snapshots remain as `.snap.new`

## Last Updated

**Date**: 2026-06-07
**Updated By**: Claude Code 2.1.119 (Claude Code); Sonnet 4.6
**Changes**: Added Bug 15 row to the summary table (was only in prose). Added the
acceptance-test framing note at top (these 15 are the acceptance test for the
owned-FIR rewrite; fixed in Phases 2+ after the Phase 1 gate; WIP files are the only
bug-scope files). No bug descriptions changed — they remain accurate.

**Date**: 2026-06-06
**Updated By**: opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Added Bug 15 (anchored_seek_negative_boundary from FOOP-32). Updated
bug count to 15. Restored WIP marker on anchored_seek_negative_boundary.foo.
Updated notes to reflect 15 bugs and 15 input files.

**Date**: 2026-06-06
**Updated By**: opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Updated Bugs 10–12 with detailed expected output referencing SF/SF
Marker Specification. Added expected HFS output format for SFF (ECONSTANIC
searches) and SF (sfcc preservation) cases.

**Date**: 2026-06-06
**Updated By**: opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Initial creation — documented 14 bugs from snapshot review round 2
