# Snapshot Bug Repair List (FOOP-32)

**Created**: 2026-06-01
**Source**: `@Agent` / `@Agents` comments found in `.snap.new` files before promotion to `.snap`
**Status**: All bugs fixed

---

## Summary

| # | File | Category | Severity |
|---|------|----------|----------|
| 1 | `complex_forward_refs_in_nested_branes` | Scope Resolution | Critical |
| 2 | `complex_search_and_concatenation` | Operator Precedence | Critical |
| 3 | `complex_full_program_with_all_features` | Scope Resolution | Critical |
| 4 | `anchored_seek_negative_boundary` | Boundary Clamping | High |
| 5 | `anchored_seek_positive_boundary` | HSSnap Formatting | Medium |
| 6 | `anchored_seek_negative_boundary` | HSSnap Formatting | Medium |
| 7 | `anchored_seek_positive_negative` | HSSnap Formatting | Medium |
| 8 | `chained_undeclared` | HSSnap Formatting | Medium |

---

## Bug 1: Forward reference resolves across two brane boundaries

**File**: `complex_forward_refs_in_nested_branes.foo.snap`
**Category**: Scope Resolution — FVM evaluation
**Severity**: Critical

**Input:**
```foolish
{nested = {inner = {val = x}}; x = 42;}
```

**Current (WRONG) output at line 16:**
```
       Int(42)
```

**Expected:**
`val` should NOT resolve to `Int(42)`. The identifier `x` is defined in the outermost brane, AFTER the nested brane that references it. Furthermore, `x` is blocked by TWO brane boundaries (`nested` and `inner`). The forward reference should fail — `val` should be `Search(pattern='^x$', ...)` in a constanic state (WOCONSTANIC or ECONSTANIC), NOT `Int(42)`.

**Root cause hypothesis:** The FVM search is penetrating brane boundaries it shouldn't, or the forward reference resolution is ignoring brane depth. The search for `x` finds `x = 42` in the parent brane even though it's separated by two nested branes and appears AFTER the reference in source order.

---

## Bug 2: Search has higher precedence than concatenation — breaks implicit concat

**File**: `complex_search_and_concatenation.foo.snap`
**Category**: Operator Precedence — FVM evaluation / compiler
**Severity**: Critical

**Input:**
```foolish
{target = {a=1; c=2; c={a=1, b=2, c=3}}; b1 = {x=10}; result = b1 target.c;}
```

**Current (WRONG) output at line 32:**
```
    Int(10)
```
Only `{x=10}` appears — the `target.c` part is completely missing.

**Expected:**
`result = b1 target.c` should parse as `result = (b1) (target.c)` — two operands concatenated:
- `b1` evaluates to `{x=10}`
- `target.c` evaluates to `{a=1, b=2, c=3}` (the brane value of `c`)
- Concatenation produces: `{x=10; a=1; b=2; c=3}`

So `result` should be:
```
  result =
  Brane
    x =
    Int(10)
    a =
    Int(1)
    b =
    Int(2)
    c =
    Int(3)
```

**Root cause hypothesis:** Search has higher precedence than concatenation. The parser/evaluator treats `b1 target.c` as `b1` followed by `target.c` as a search operation on `b1`, rather than as two independent operands being concatenated. Suggested debug: try `result = b1 (target.c)` to see if parenthesizing the search fixes it — if so, the issue is precedence in the parser or evaluation order.

---

## Bug 3: Search for `sum` fails inside nested brane despite `sum` being in parent scope

**File**: `complex_full_program_with_all_features.foo.snap`
**Category**: Scope Resolution — FVM evaluation
**Severity**: Critical

**Input:**
```foolish
{a = 10; b = 20; sum = a + b; nested = {inner = sum / 2}; result = nested.inner;}
```

**Current (WRONG) output at line 21:**
```
Search(pattern='^sum$', dir=BACKWARD, FREE, [WOCONSTANIC])
```

**Expected:**
`sum` is defined as `Int(30)` in the parent brane (visible at lines 15-16). Inside `nested`, the expression `inner = sum / 2` should resolve `sum` to `Int(30)` and compute `Int(15)`. The search should NOT be WOCONSTANIC — it should find `sum`.

**Root cause hypothesis:** The search for `sum` from within the `nested` brane fails to cross into the parent brane. This may be related to Bug 1 (brane boundary penetration) but in the opposite direction — here the search fails to go outward when it should succeed. The issue may be that the nested brane's search boundary is too restrictive, or that `sum` is not yet resolved when `nested` is evaluated (order-of-evaluation issue).

---

## Bug 4: Negative seek out-of-bounds clamps to first element instead of returning NK

**File**: `anchored_seek_negative_boundary.foo.snap`
**Category**: Boundary Clamping — FVM evaluation
**Severity**: High

**Input:**
```foolish
{b = {10; 20; 30}; last = b#-1; second = b#1; first = b#-3; oob = b#-4;}
```

**Current (WRONG) output at line 23:**
```
  Int(10)
```

**Expected:**
`b#-4` on a 3-element brane `{10; 20; 30}` is out of bounds (indices -1, -2, -3 are valid; -4 is not). The result should be `NK` or an `Index` FIR in NK state, NOT `Int(10)`.

**Contrast with positive OOB:** In `anchored_seek_positive_boundary.foo.snap`, `b#3` correctly produces `Index(offset=3, ANCHORED, [NK])`. The negative direction should behave symmetrically.

**Root cause hypothesis:** Negative seek clamps to index 0 (or the first element) when the offset exceeds the brane size in the negative direction, instead of producing NK. The clamping logic is asymmetric between positive and negative offsets.

---

## Bug 5: Brane HSSnap output missing `{}` curly bracket enclosures

**Files**: `anchored_seek_positive_boundary.foo.snap`, `anchored_seek_negative_boundary.foo.snap`, `anchored_seek_positive_negative.foo.snap`
**Category**: HSSnap Formatting — Sequencer
**Severity**: Medium

**Affected lines:**
- `anchored_seek_positive_boundary`: line 12 — `Brane` should be `Brane{`
- `anchored_seek_negative_boundary`: line 12 — `Brane` should be `Brane{`
- `anchored_seek_positive_negative`: line 12 — `Brane` should be `Brane{`

**Current output:**
```
  Brane
    Int(10)
    Int(20)
    Int(30)
```

**Expected output:**
```
  Brane{
    Int(10)
    Int(20)
    Int(30)
  }
```

Brane content should be enclosed in `{` and `}` to make the structure visually clear, especially on multi-line output. Named branes (with `name =`) may not need braces, but anonymous branes (with positional elements) should always show them.

**Root cause hypothesis:** The Sequencer's `format_fir` for `NormalBraneFir` does not emit `{` and `}` delimiters. This is a formatting-only issue in `sequencer.rs`.

---

## Bug 6: Search `search_result` field appears outside `()` instead of inside

**File**: `chained_undeclared.foo.snap`
**Category**: HSSnap Formatting — Sequencer
**Severity**: Medium

**Affected lines:** 15, 18

**Current output:**
```
Search(pattern='^undeclared$', dir=BACKWARD, FREE, [ECONSTANIC])       Search(pattern='^undeclared$', dir=BACKWARD, FREE, [ECONSTANIC])
```
The nested `search_result` appears as a separate indented line AFTER the closing `)`, not inside the Search FIR's parentheses.

**Expected output:**
```
Search(pattern='^undeclared$', dir=BACKWARD, FREE, [ECONSTANIC])
  Search(pattern='^undeclared$', dir=BACKWARD, FREE, [ECONSTANIC])
```
The `search_result` (target) should be formatted as a child of the Search FIR, indented underneath it, consistent with how other FIRs format their children (e.g., `Operator` formats its operands as indented children).

**Root cause hypothesis:** The Sequencer's `hs_search` formatting (line 75-82 of `sequencer.rs`) checks for `target` and formats it, but the formatting may not be producing the correct indentation or the target is being treated as a sibling rather than a child. Looking at the code:
```rust
if let Some((pattern, direction, anchored, _anchor, target)) = fir.hs_search() {
    let anchor_str = if anchored { "ANCHORED" } else { "UNANCHORED" };
    let _ = writeln!(buf, "{}Search(pattern='{}', dir={}, {}{})",
        indent, pattern, direction, anchor_str, state_sfx);
    if let Some(ref t) = target {
        format_fir_q(buf, &**t, depth + 1);
    }
}
```
The `target` IS being formatted recursively. The issue may be that the search_result is appearing on the same line as the parent Search (line 15 shows them concatenated), suggesting the newline or indentation is wrong.

---

## Notes

- All 134 `.snap.new` files have been promoted to `.snap` extension.
- Files marked `(@Agents, lgtm)` or `(@Agents, lgtm. VERY GOOD!)` have no bugs — 18 files approved clean.
- Bugs 1-4 are FVM evaluation bugs (require engine fixes).
- Bugs 5-6 are Sequencer formatting bugs (require `sequencer.rs` fixes).
- After fixes, re-run `cargo insta test -p foolish-core --lib` to regenerate `.snap.new` files, review, and accept.
